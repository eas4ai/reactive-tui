//! Chart rasterization: validation, layout by size class, the text layer,
//! and composition of the shared mask canvas into a finished picture. Runs
//! on the chart worker thread; the main thread only copies the picture.

use super::super::mask::{GlyphSet, MaskCanvas};
use super::super::plot::{self, Legend, LegendEntry, Rect, Rgba, SizeClass, TextSink};
use super::super::{ChartAxis, ChartProps, ChartType, DataPoint, DataSeries, LegendPosition};
use crate::layout::paint_tree::cells::CellGrid;
use std::collections::HashMap;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod cartesian;
mod pie;
#[cfg(feature = "wgpu-graphics")]
mod pixels;
mod radar;
mod sankey;
mod shapes;

pub(super) use sankey::node_text as sankey_node_text;

/// One drawn slice of a pie or donut: a slice whose fill set a sample.
#[derive(Clone, Debug)]
pub(super) struct RadialSlice {
    /// Where the slice's drawn angle range starts, after its pad.
    pub start: f64,
    /// Where it ends.
    pub end: f64,
    /// The (series, index) the slice draws.
    pub key: (usize, usize),
    /// The slice's color, for its tooltip swatch.
    pub color: Option<Rgba>,
}

/// Where a radial chart's slices or spokes lie, for pointer selection
/// (CHT-031). Everything is in dot units; angles are radians clockwise from
/// twelve o'clock.
#[derive(Clone, Debug, Default)]
pub(super) struct RadialHit {
    /// The circle's center.
    pub center: (f64, f64),
    /// Inner radius: the donut hole.
    pub inner: f64,
    /// Outer radius.
    pub outer: f64,
    /// Pie and donut: each drawn slice, in order around the circle.
    pub slices: Vec<RadialSlice>,
    /// Radar: each category's spoke angle.
    pub spokes: Vec<f64>,
    /// The fill blitter's pixels per cell (columns, rows), so the pointer
    /// samples a cell where the fill did.
    pub samples: (u32, u32),
}

/// Where a Sankey chart's nodes lie, for selection (CHT-032). Positions are
/// in dot units.
#[derive(Clone, Debug, Default)]
pub(super) struct SankeyHit {
    /// Each node's rectangle `(x0, y0, x1, y1)`, indexed like the nodes.
    pub nodes: Vec<(f64, f64, f64, f64)>,
    /// Node indices by layer, then top to bottom: the key order.
    pub order: Vec<usize>,
    /// Each node's color, for its tooltip swatch.
    pub colors: Vec<Option<Rgba>>,
    /// The fill blitter's pixels per cell (columns, rows), so the pointer
    /// samples a cell where the fill did.
    pub samples: (u32, u32),
}

/// The hover a plot picture draws (CHT-038): the selected series and index,
/// how far the marks have eased in, and where the band's center is along
/// the category axis, in cells, as the glide has it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct HoverMarks {
    pub series: usize,
    pub index: usize,
    /// 0 to 1: the dots, halo and fade easing in when the hover begins.
    pub focus: f32,
    /// The band's center in cells along the category axis.
    pub band: f64,
}

/// A plot drawn as a picture (CHT-037): the pixels of a cell the scene is
/// built for, and the hover it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct PlotPixels {
    pub cell: (u16, u16),
    pub hover: Option<HoverMarks>,
}

/// The rasterized chart plus what the main thread needs for interaction.
#[derive(Clone, Debug)]
pub(super) struct Picture {
    pub width: usize,
    pub height: usize,
    /// The finished cells, painted by one element (CHT-021, BAR-005). When
    /// the plot is a picture they hold nothing inside `inner`.
    pub grid: Arc<CellGrid>,
    /// The text inside the plot of a picture (value labels, tick labels
    /// placed inside), painted over the picture (CHT-037).
    pub over: Option<Arc<CellGrid>>,
    /// The plot picture's scene, in the chart's pixels, when the plot is
    /// one (CHT-037).
    #[cfg(feature = "wgpu-graphics")]
    pub scene: Option<Arc<crate::graphics::Scene>>,
    /// The pixels per cell the scene was built for.
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub cell: (u16, u16),
    /// The size class the picture was drawn at.
    pub class: Option<SizeClass>,
    /// The plot rectangle.
    pub plot: Rect,
    /// The plot without its axis lines: where the shapes are, and where
    /// the picture sits.
    pub inner: Rect,
    /// Column of each category index (cartesian charts), for nearest lookup.
    pub index_columns: Vec<f64>,
    /// Row of each category index (horizontal bars), for nearest lookup.
    pub index_rows: Vec<f64>,
    /// Cell of every drawn point, keyed by (series, index).
    pub anchors: HashMap<(usize, usize), (usize, usize)>,
    /// Whether the chart is a scatter plot (nearest in both axes).
    pub scatter: bool,
    /// Kept samples per series after decimation: (original index, value).
    pub kept: Vec<Vec<usize>>,
    /// Slice and spoke geometry of a pie, donut or radar chart.
    pub radial: Option<RadialHit>,
    /// Node geometry of a Sankey chart.
    pub sankey: Option<SankeyHit>,
    /// The selection the shapes were drawn with (a Sankey chart's faded
    /// links, CHT-032).
    pub selected: Option<(usize, usize)>,
    /// The theme generation the colors were resolved under, so a picture
    /// drawn under an older theme is replaced, not shown (THM-003).
    pub theme: u64,
}

impl Picture {
    fn blank(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: Arc::new(CellGrid::new(
                u16::try_from(width).unwrap_or(u16::MAX),
                u16::try_from(height).unwrap_or(u16::MAX),
            )),
            over: None,
            #[cfg(feature = "wgpu-graphics")]
            scene: None,
            cell: (0, 0),
            class: None,
            plot: Rect::default(),
            inner: Rect::default(),
            index_columns: Vec::new(),
            index_rows: Vec::new(),
            anchors: HashMap::new(),
            scatter: false,
            kept: Vec::new(),
            radial: None,
            sankey: None,
            selected: None,
            theme: 0,
        }
    }

    /// Whether the plot is a picture.
    pub fn has_scene(&self) -> bool {
        #[cfg(feature = "wgpu-graphics")]
        {
            self.scene.is_some()
        }
        #[cfg(not(feature = "wgpu-graphics"))]
        {
            false
        }
    }
}

/// The text layers: `under` shows where the mask is empty, `over` covers it.
#[derive(Clone, Debug)]
pub(super) struct TextLayer {
    width: usize,
    height: usize,
    under: Vec<Option<(String, Option<Rgba>)>>,
    over: Vec<Option<(String, Option<Rgba>)>>,
}

impl TextLayer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            under: vec![None; width * height],
            over: vec![None; width * height],
        }
    }

    fn put(&mut self, x: usize, y: usize, text: &str, color: Option<Rgba>) {
        if x < self.width && y < self.height {
            self.over[y * self.width + x] = Some((text.to_string(), color));
        }
    }

    fn wrapped(&mut self, text: &str, y: usize) {
        for (row, line) in wrapped_lines(text, self.width)
            .iter()
            .take(self.height.saturating_sub(y))
            .enumerate()
        {
            self.text(0, y + row, self.width, line, None);
        }
    }
}

impl TextSink for TextLayer {
    fn text(&mut self, x: usize, y: usize, width: usize, text: &str, color: Option<Rgba>) {
        let mut offset = 0;
        for grapheme in text.graphemes(true) {
            let safe = if grapheme.chars().any(char::is_control) {
                "\u{fffd}"
            } else {
                grapheme
            };
            let cells = UnicodeWidthStr::width(safe);
            if cells == 0 {
                continue;
            }
            if offset + cells > width || x + offset + cells > self.width {
                break;
            }
            self.put(x + offset, y, safe, color);
            for continuation in 1..cells {
                self.put(x + offset + continuation, y, "", color);
            }
            offset += cells;
        }
    }

    fn under(&mut self, x: usize, y: usize, glyph: &str, color: Option<Rgba>) {
        if x < self.width && y < self.height {
            self.under[y * self.width + x] = Some((glyph.to_string(), color));
        }
    }
}

pub(super) fn wrapped_lines(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty()
            && UnicodeWidthStr::width(line.as_str()) + 1 + UnicodeWidthStr::width(word) > width
        {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        for grapheme in word.graphemes(true) {
            if UnicodeWidthStr::width(line.as_str()) + UnicodeWidthStr::width(grapheme) > width
                && !line.is_empty()
            {
                lines.push(std::mem::take(&mut line));
            }
            line.push_str(grapheme);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Resolve a color token through the active theme (CHT-017).
pub(super) fn color(value: &str) -> Option<Rgba> {
    crate::theme::Theme::active().resolve_color(value)
}

/// The color of point `point` in series `series`: the point's own color,
/// then (for a bar series) the series fill, then the series color, then the
/// palette entry for the series. A bar's body is a fill, so `.fill()` colors
/// it on every builder route alike (CHT-013, CHT-035).
pub(super) fn point_color(props: &ChartProps, series: usize, point: usize) -> Option<Rgba> {
    let data = &props.series[series];
    data.data
        .get(point)
        .and_then(|p| p.color.as_deref())
        .or_else(|| bar_fill(props, series))
        .or(data.color.as_deref())
        .or_else(|| palette_token(props, series))
        .and_then(color)
}

/// The fill token of a bar series, which colors its bars (CHT-013); `None`
/// for other chart types, whose fill is the area under the stroke.
fn bar_fill(props: &ChartProps, series: usize) -> Option<&str> {
    matches!(
        props.chart_type,
        ChartType::BarVertical | ChartType::BarHorizontal
    )
    .then(|| props.series[series].fill.as_deref())
    .flatten()
}

/// The color of a whole series, for its legend swatch: a bar series' fill
/// (the color of its bars), else the series color, else the palette.
pub(super) fn series_color(props: &ChartProps, series: usize) -> Option<Rgba> {
    bar_fill(props, series)
        .or(props.series[series].color.as_deref())
        .or_else(|| palette_token(props, series))
        .and_then(color)
}

fn palette_token(props: &ChartProps, series: usize) -> Option<&str> {
    props
        .color_palette
        .get(series % props.color_palette.len().max(1))
        .map(String::as_str)
}

pub(super) fn validate(props: &ChartProps) -> Result<(), &'static str> {
    for axis in [&props.x_axis, &props.y_axis] {
        if axis.min.is_some_and(|v| !v.is_finite()) || axis.max.is_some_and(|v| !v.is_finite()) {
            return Err("Axis limits must be finite");
        }
        if axis
            .min
            .zip(axis.max)
            .is_some_and(|(min, max)| min >= max || !(max - min).is_finite())
        {
            return Err("Axis minimum must be below maximum with a finite span");
        }
    }
    for series in &props.series {
        if !series.visible {
            continue;
        }
        if series.color.as_deref().is_some_and(|s| color(s).is_none()) {
            return Err("Invalid series color");
        }
        for point in &series.data {
            // A value, a numeric x (CHT-033) or a candle's four values that
            // is NaN or infinite is an error, never a shape (CHT-026).
            if !point.value.is_finite()
                || point.x.is_some_and(|x| !x.is_finite())
                || point.candle.is_some_and(|c| {
                    ![c.open, c.high, c.low, c.close]
                        .iter()
                        .all(|v| v.is_finite())
                })
            {
                return Err("Chart values must be finite");
            }
            if matches!(props.chart_type, ChartType::Pie | ChartType::Donut) && point.value < 0.0 {
                return Err("Pie values must be nonnegative");
            }
            if point.color.as_deref().is_some_and(|s| color(s).is_none()) {
                return Err("Invalid point color");
            }
        }
    }
    if props.color_palette.iter().any(|s| color(s).is_none()) {
        return Err("Invalid palette color");
    }
    if matches!(
        props.chart_type,
        ChartType::Pie | ChartType::Donut | ChartType::Radar
    ) {
        let radial = &props.radial;
        if !radial.outer_radius.is_finite()
            || !radial.pad_angle.is_finite()
            || radial.inner_radius.is_some_and(|r| !r.is_finite())
            || radial.max_value.is_some_and(|m| !m.is_finite())
        {
            return Err("Radial options must be finite");
        }
        if radial
            .fills
            .iter()
            .flatten()
            .any(|token| token != "none" && color(token).is_none())
        {
            return Err("Invalid radar fill color");
        }
    }
    if props.chart_type == ChartType::Sankey {
        let sankey = &props.sankey;
        if sankey
            .links
            .iter()
            .any(|link| !link.value.is_finite() || link.value < 0.0)
        {
            return Err("Sankey link values must be finite and not negative");
        }
        if !sankey.link_opacity.is_finite() || !sankey.min_link_width.is_finite() {
            return Err("Sankey options must be finite");
        }
        if sankey
            .labels
            .iter()
            .flatten()
            .any(|line| line.color.as_deref().is_some_and(|c| color(c).is_none()))
        {
            return Err("Invalid Sankey label color");
        }
        // A missing node or a cycle is an error before any shape (CHT-030).
        let nodes = props.series.first().map_or(0, |s| s.data.len());
        if let Err(error) = sankey::generator(sankey).topology(nodes, &sankey.links) {
            return Err(match error {
                crate::widgets::display::charts::plot::SankeyError::MissingNode(_) => {
                    "Sankey link names a missing node"
                }
                crate::widgets::display::charts::plot::SankeyError::CircularLink => {
                    "Sankey links form a cycle"
                }
            });
        }
    }
    Ok(())
}

/// The visible series with their original indices.
pub(super) fn visible(props: &ChartProps) -> Vec<(usize, &DataSeries)> {
    props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .collect()
}

/// What one rasterization needs beyond the props.
pub(super) struct Job<'a> {
    pub props: &'a ChartProps,
    pub width: usize,
    pub height: usize,
    /// Values per series (indexed like `props.series`) after motion.
    pub values: &'a [Vec<f64>],
    /// Reveal progress, 0 to 1.
    pub progress: f64,
    /// A running transition's progress and the values it started from, so
    /// the automatic range moves from their range to the target's in step
    /// with `values` (CHT-022).
    pub transition: Option<(f64, &'a [Vec<f64>])>,
    /// Whether the terminal draws braille and block glyphs (the capability
    /// report); false resolves cells with ASCII (CHT-028).
    pub unicode_glyphs: bool,
    /// The selected (series, point), for charts whose shapes show the
    /// selection: a Sankey chart fades the links of other nodes (CHT-032).
    pub selected: Option<(usize, usize)>,
    /// The plot as a picture, where the terminal takes one (CHT-037): a
    /// cartesian chart then draws its shapes into a scene, with the hover
    /// marks (CHT-038), and nothing inside the plot on the mask.
    pub pixels: Option<PlotPixels>,
}

/// The size class for `props` in a `width` by `height` rectangle.
pub(super) fn size_class(props: &ChartProps, width: usize, height: usize) -> SizeClass {
    props
        .size_class
        .unwrap_or_else(|| SizeClass::for_size(width, height))
}

/// Rasterize one picture.
pub(super) fn draw(job: &Job) -> Picture {
    let props = job.props;
    let (width, height) = (job.width, job.height);
    let height = height.min(1_000_000 / width.max(1));
    let mut text = TextLayer::new(width, height);
    let mut mask = MaskCanvas::new(width, height);
    let glyphs = if props.ascii || !job.unicode_glyphs {
        GlyphSet::Ascii
    } else {
        GlyphSet::Unicode
    };
    // Filled shapes resolve through the blitter image fallback uses, with
    // its overrides (CHT-025, BLT-002); ASCII samples the dot grid instead.
    mask.set_fill_blitter(match glyphs {
        GlyphSet::Ascii => ::suprtui::blit::Blitter::Braille,
        GlyphSet::Unicode => crate::widgets::display::image::image_blitter(),
    });
    let mut picture = Picture::blank(width, height);
    picture.selected = job.selected;
    if width == 0 || height == 0 {
        return picture;
    }
    if let Err(error) = validate(props) {
        text.wrapped(error, 0);
        return compose(picture, &mask, &text, glyphs);
    }
    let class = size_class(props, width, height);
    picture.class = Some(class);
    let mut area = Rect::sized(width, height);
    // The title row, except at the mini class, which is shapes only
    // (CHT-024); the screen reader still hears the title as the name.
    if let Some(title) = props.title.as_ref().filter(|_| class.has_title()) {
        let strip = area.take_top(1);
        // The chart title takes the `foreground` role (CHT-017).
        text.text(strip.x, strip.y, strip.w, title, color("foreground"));
    }
    let visible = visible(props);
    if visible.iter().all(|(_, s)| s.data.is_empty()) {
        text.text(area.x, area.y, area.w, "No data to display", None);
        return compose(picture, &mask, &text, glyphs);
    }
    // A pie or donut's legend names its slices; a Sankey chart's labels
    // name its nodes; other charts name series.
    let entries: Vec<LegendEntry> = if matches!(props.chart_type, ChartType::Pie | ChartType::Donut)
    {
        pie::legend_entries(job)
    } else if props.chart_type == ChartType::Sankey {
        Vec::new()
    } else {
        visible
            .iter()
            .map(|(index, series)| LegendEntry {
                name: series.name.clone(),
                color: series_color(props, *index),
            })
            .collect()
    };
    let legend = legend_area(props, &entries, class, &mut area);
    // The slices a pie or donut labelled on the chart, when it drew them.
    let mut labelled = None;
    if !area.is_empty() {
        if matches!(props.chart_type, ChartType::Pie | ChartType::Donut) {
            labelled = Some(pie::pie(
                &mut mask,
                &mut text,
                &mut picture,
                job,
                area,
                class,
            ));
        } else if props.chart_type == ChartType::Radar {
            radar::radar(&mut mask, &mut text, &mut picture, job, area, class);
        } else if props.chart_type == ChartType::Sankey {
            sankey::sankey(&mut mask, &mut text, &mut picture, job, area, class);
        } else {
            // The shapes go to the mask, or to the scene of a plot picture
            // (CHT-037).
            #[cfg(feature = "wgpu-graphics")]
            let mut target = match job.pixels {
                Some(pixels) => {
                    shapes::Shapes::Pixels(pixels::PlotScene::new(pixels.cell, color("foreground")))
                }
                None => shapes::Shapes::Mask(&mut mask),
            };
            #[cfg(not(feature = "wgpu-graphics"))]
            let mut target = shapes::Shapes::Mask(&mut mask);
            let drawn =
                cartesian::cartesian(&mut target, &mut text, &mut picture, job, area, class);
            #[cfg(feature = "wgpu-graphics")]
            if let shapes::Shapes::Pixels(scene) = target {
                if drawn.is_ok() && !picture.inner.is_empty() {
                    picture.scene = Some(scene.finish());
                    picture.cell = job.pixels.map_or((0, 0), |pixels| pixels.cell);
                }
            }
            if let Err(error) = drawn {
                text.text(area.x, area.y, area.w, error, None);
            }
        }
    }
    if let Some((rect, flowed)) = legend {
        let (max_name, wrap) = (rect.w.saturating_sub(2), class.multi_row_legend());
        let drawn = |legend: &Legend| {
            if flowed {
                legend.flowed_count(rect, max_name, wrap)
            } else {
                legend.stacked_count(rect)
            }
        };
        // A full legend keeps the slices whose labels were left out.
        let entries = match &labelled {
            Some(labelled) => pie::legend_shown(job, entries, labelled, |shown| {
                drawn(&Legend::new(shown.to_vec())) == shown.len()
            }),
            None => entries,
        };
        // Legend names take the `text-muted` role (CHT-017).
        let legend = Legend::new(entries)
            .with_name_color(color("text-muted"))
            .with_ascii(glyphs == GlyphSet::Ascii);
        if flowed {
            legend.draw_flowed(&mut text, rect, max_name, wrap);
        } else {
            legend.draw_stacked(&mut text, rect);
        }
    }
    compose(picture, &mask, &text, glyphs)
}

/// Reserve the legend rectangle out of `area` and say whether it flows along
/// rows (top or bottom) or stacks one entry per row (sides and floating).
fn legend_area(
    props: &ChartProps,
    entries: &[LegendEntry],
    class: SizeClass,
    area: &mut Rect,
) -> Option<(Rect, bool)> {
    if !props.legend.visible || entries.is_empty() || !class.has_legend() {
        return None;
    }
    let entries = Legend::new(entries.to_vec());
    let max_name = props
        .legend
        .max_width
        .map_or(usize::MAX, |w| (w as usize).saturating_sub(2));
    let stacked = entries.stacked_size(max_name);
    let stacked = (stacked.0.min(area.w), stacked.1.min(area.h));
    match props.legend.position {
        LegendPosition::Top | LegendPosition::Bottom => {
            let (w, h) = entries.flowed_size(area.w, max_name, class.multi_row_legend());
            let h = h.min(area.h.saturating_sub(2));
            let rect = if props.legend.position == LegendPosition::Top {
                area.take_top(h)
            } else {
                area.take_bottom(h)
            };
            Some((
                Rect {
                    w: w.min(rect.w),
                    ..rect
                },
                true,
            ))
        }
        // The medium class has a single-row legend (CHT-024): a side legend
        // takes one row at the top of the area, aligned to its side, and a
        // floating one is one row at its place; the large class stacks them.
        LegendPosition::Left | LegendPosition::Right if !class.multi_row_legend() => {
            let (w, h) = entries.flowed_size(area.w, max_name, false);
            let h = h.min(area.h.saturating_sub(2));
            let strip = area.take_top(h);
            let w = w.min(strip.w);
            let x = if props.legend.position == LegendPosition::Left {
                strip.x
            } else {
                strip.right().saturating_sub(w)
            };
            Some((Rect { x, w, ..strip }, true))
        }
        LegendPosition::Floating(x, y) if !class.multi_row_legend() => {
            let room = area.w.saturating_sub(x as usize);
            let (w, h) = entries.flowed_size(room, max_name, false);
            Some((
                Rect {
                    x: area.x + x as usize,
                    y: area.y + y as usize,
                    w: w.min(room),
                    h: h.min(area.h.saturating_sub(y as usize)),
                },
                true,
            ))
        }
        LegendPosition::Left => {
            let w = stacked.0.min(area.w / 2);
            let rect = area.take_left(w);
            Some((
                Rect {
                    h: stacked.1,
                    ..rect
                },
                false,
            ))
        }
        LegendPosition::Right => {
            let w = stacked.0.min(area.w / 2);
            let rect = area.take_right(w);
            Some((
                Rect {
                    h: stacked.1,
                    ..rect
                },
                false,
            ))
        }
        LegendPosition::Floating(x, y) => Some((
            Rect {
                x: area.x + x as usize,
                y: area.y + y as usize,
                w: stacked.0.min(area.w.saturating_sub(x as usize)),
                h: stacked.1.min(area.h.saturating_sub(y as usize)),
            },
            false,
        )),
    }
}

/// Merge the mask and the text layers into the cell grid one element paints.
fn compose(mut picture: Picture, mask: &MaskCanvas, text: &TextLayer, glyphs: GlyphSet) -> Picture {
    let (width, height) = (picture.width, picture.height);
    let (columns, rows) = (
        u16::try_from(width).unwrap_or(u16::MAX),
        u16::try_from(height).unwrap_or(u16::MAX),
    );
    let mut grid = CellGrid::new(columns, rows);
    // Inside a plot that is a picture the cells stay blank and the text
    // goes to the grid painted over the picture (CHT-037).
    let inner = picture.has_scene().then_some(picture.inner);
    let mut over = inner.map(|_| CellGrid::new(columns, rows));
    for y in 0..height.min(usize::from(u16::MAX)) {
        for x in 0..width.min(usize::from(u16::MAX)) {
            let i = y * width + x;
            if let (Some(inner), Some(over)) = (inner, over.as_mut()) {
                if inner.contains(x, y) {
                    if let Some((t, c)) = &text.over[i] {
                        over.set(x as u16, y as u16, t, *c);
                    }
                    continue;
                }
            }
            let (glyph, color, background): (&str, Option<Rgba>, Option<Rgba>) =
                if let Some((t, c)) = &text.over[i] {
                    (t.as_str(), *c, None)
                } else {
                    let resolved = mask.resolve(x, y, glyphs);
                    match resolved.glyph {
                        Some(glyph) => (glyph, resolved.color, resolved.background),
                        None => match &text.under[i] {
                            Some((t, c)) => (t.as_str(), *c, None),
                            None => continue,
                        },
                    }
                };
            match background {
                Some(background) => {
                    grid.set_with_background(x as u16, y as u16, glyph, color, Some(background))
                }
                None => grid.set(x as u16, y as u16, glyph, color),
            }
        }
    }
    picture.grid = Arc::new(grid);
    picture.over = over.map(Arc::new);
    picture
}

/// The label for a point in the tooltip: `label: value; key=value`.
pub(super) fn point_text(point: &DataPoint, index: usize) -> String {
    let label = point.label.clone().unwrap_or_else(|| index.to_string());
    let mut text = match point.candle {
        Some(c) => format!(
            "{label}: O {} H {} L {} C {}",
            c.open, c.high, c.low, c.close
        ),
        None => format!("{label}: {}", point.value),
    };
    let mut metadata: Vec<_> = point.metadata.iter().collect();
    metadata.sort();
    for (key, value) in metadata {
        text.push_str(&format!("; {key}={value}"));
    }
    text
}

/// Tick count for an axis, honoring an explicit count.
pub(super) fn tick_count(axis: &ChartAxis, cells: usize) -> usize {
    axis.tick_count.min(cells.max(1)).max(1)
}

/// Unused-import guard for the plot re-export in this module's path checks.
#[allow(dead_code)]
fn _plot_marker() -> plot::Curve {
    plot::Curve::Linear
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::display::charts::DataSeries;

    fn props(kind: ChartType, values: &[f64]) -> ChartProps {
        ChartProps {
            chart_type: kind,
            series: vec![DataSeries::new(
                "s",
                values.iter().map(|v| DataPoint::new(*v)).collect(),
            )],
            legend: crate::widgets::display::charts::ChartLegend {
                visible: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// CHT-013: a horizontal bar's value label starts in the cell after its
    /// tip, in the direction the bar grew, on the mask and in a picture; it
    /// never sits inside the bar.
    fn label_cell(grid: &CellGrid, row: usize, label: &str) -> Option<usize> {
        (0..grid.width())
            .find(|&x| grid.get(x, row as u16).is_some_and(|(g, _)| g == label))
            .map(usize::from)
    }

    #[test]
    fn cht_013_a_horizontal_bars_label_starts_after_its_tip() {
        let mut p = props(ChartType::BarHorizontal, &[2.0, 8.0]);
        p.value_labels = Some(true);
        let values = vec![vec![2.0, 8.0]];
        let job = |pixels| Job {
            props: &p,
            width: 40,
            height: 8,
            values: &values,
            progress: 1.0,
            transition: None,
            unicode_glyphs: true,
            selected: None,
            pixels,
        };
        let cells = draw(&job(None));
        for (i, label) in [(0usize, "2"), (1, "8")] {
            let (tip_x, tip_y) = cells.anchors[&(0, i)];
            let at = label_cell(&cells.grid, tip_y, label)
                .unwrap_or_else(|| panic!("the label {label} is on the bar's row {tip_y}"));
            assert_eq!(
                at,
                tip_x + 1,
                "on the mask the label {label} starts right after the tip cell {tip_x}:\n{}",
                cells.grid.to_text()
            );
            let (tip_glyph, _) = cells
                .grid
                .get(tip_x as u16, tip_y as u16)
                .unwrap_or(("", None));
            assert!(
                tip_glyph
                    .chars()
                    .all(|c| ('\u{2588}'..='\u{258f}').contains(&c)),
                "the tip cell keeps its block, found {tip_glyph:?}"
            );
        }
        #[cfg(feature = "wgpu-graphics")]
        {
            let picture = draw(&job(Some(PlotPixels {
                cell: (8, 16),
                hover: None,
            })));
            let over = picture
                .over
                .as_ref()
                .expect("the in-plot text over the picture");
            for (i, label) in [(0usize, "2"), (1, "8")] {
                let (tip_x, tip_y) = picture.anchors[&(0, i)];
                assert_eq!(
                    label_cell(over, tip_y, label),
                    Some(tip_x + 1),
                    "in a picture the label {label} starts right after the tip cell {tip_x}:\n{}",
                    over.to_text()
                );
            }
        }
    }

    /// CHT-027: a plot drawn as a picture is thinned per pixel column, so a
    /// 10,000-point line on a plot 160 pixels wide is drawn from at most two
    /// points per column.
    #[cfg(feature = "wgpu-graphics")]
    #[test]
    fn cht_027_a_plot_picture_keeps_two_points_per_pixel_column() {
        let values: Vec<f64> = (0..10_000)
            .map(|i| 5.0 + 4.0 * ((i as f64) * 0.013).sin())
            .collect();
        let mut p = props(ChartType::Line, &values);
        p.x_axis.show_labels = false;
        p.y_axis.show_labels = false;
        p.y_axis.min = Some(0.0);
        p.y_axis.max = Some(10.0);
        let picture = draw(&Job {
            props: &p,
            width: 20,
            height: 12,
            values: std::slice::from_ref(&values),
            progress: 1.0,
            transition: None,
            unicode_glyphs: true,
            selected: None,
            pixels: Some(PlotPixels {
                cell: (8, 16),
                hover: None,
            }),
        });
        let scene = picture.scene.expect("the plot is a picture");
        let segments = scene
            .commands
            .iter()
            .filter_map(|command| match command {
                crate::graphics::Command::Stroke(path, _, _) => Some(path.segments.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        // 20 cells of 8 pixels are 160 columns: two points each, at most.
        assert!(
            (100..=321).contains(&segments),
            "the line's path has {segments} segments for 160 pixel columns"
        );
        assert!(
            picture.kept[0].len() <= 320,
            "{} points kept of 10,000",
            picture.kept[0].len()
        );
    }

    fn text_of(picture: &Picture) -> String {
        picture.grid.to_text()
    }

    #[test]
    fn a_bar_chart_paints_blocks_and_an_empty_chart_says_so() {
        let props = props(ChartType::BarVertical, &[2.0, 8.0, 5.0]);
        let values = vec![vec![2.0, 8.0, 5.0]];
        let picture = draw(&Job {
            props: &props,
            width: 40,
            height: 12,
            values: &values,
            progress: 1.0,
            transition: None,
            unicode_glyphs: true,
            selected: None,
            pixels: None,
        });
        let text = text_of(&picture);
        assert!(text.contains('█'), "{text}");
        assert!(picture.grid.iter().count() > 0);
        let mut empty = props.clone();
        empty.series.clear();
        let picture = draw(&Job {
            props: &empty,
            width: 40,
            height: 12,
            values: &[],
            progress: 1.0,
            transition: None,
            unicode_glyphs: true,
            selected: None,
            pixels: None,
        });
        assert!(
            text_of(&picture).contains("No data"),
            "{}",
            text_of(&picture)
        );
    }
    /// CHT-028: a capability report without braille or block glyphs makes
    /// the canvas resolve with ASCII, whatever the builder says.
    #[test]
    fn a_terminal_without_glyph_support_gets_ascii_shapes() {
        let props = props(ChartType::Line, &[1.0, 8.0, 2.0, 9.0]);
        let values = vec![vec![1.0, 8.0, 2.0, 9.0]];
        let picture = draw(&Job {
            props: &props,
            width: 30,
            height: 10,
            values: &values,
            progress: 1.0,
            transition: None,
            unicode_glyphs: false,
            selected: None,
            pixels: None,
        });
        let text = text_of(&picture);
        assert!(
            !text
                .chars()
                .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c) || "▁▂▃▄▅▆▇█".contains(c)),
            "no braille or block glyph without glyph support:\n{text}"
        );
        assert!(
            text.chars().any(|c| "#|-.".contains(c)),
            "ASCII shapes expected:\n{text}"
        );
    }
}
