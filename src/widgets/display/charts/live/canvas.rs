//! Chart rasterization: validation, layout by size class, the text layer,
//! and composition of the shared mask canvas into a finished picture. Runs
//! on the chart worker thread; the main thread only copies the picture.

use super::super::mask::{GlyphSet, MaskCanvas};
use super::super::plot::{self, Legend, LegendEntry, Rect, Rgba, SizeClass, TextSink};
use super::super::{ChartAxis, ChartProps, ChartType, DataPoint, DataSeries, LegendPosition};
use crate::builder::ElementBuilder;
use crate::component::{Element, ElementType};
use crate::layout::style::StyleBuilder;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod cartesian;
mod pie;

/// One finished cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Cell {
    /// Glyph, or empty for a blank cell (or a wide glyph's continuation).
    pub text: String,
    pub color: Option<Rgba>,
    /// The (series, point) drawn here, for pointer hit testing.
    pub owner: Option<(usize, usize)>,
}

/// A run of adjacent cells sharing one color on one row.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Run {
    pub y: usize,
    pub x: usize,
    pub width: usize,
    pub text: String,
    pub color: Option<Rgba>,
}

/// The rasterized chart plus what the main thread needs for interaction.
#[derive(Clone, Debug, Default)]
pub(super) struct Picture {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub runs: Vec<Run>,
    /// The size class the picture was drawn at.
    pub class: Option<SizeClass>,
    /// The plot rectangle.
    pub plot: Rect,
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
/// then the series color, then the palette entry for the series.
pub(super) fn point_color(props: &ChartProps, series: usize, point: usize) -> Option<Rgba> {
    let data = &props.series[series];
    data.data
        .get(point)
        .and_then(|p| p.color.as_deref())
        .or(data.color.as_deref())
        .or_else(|| palette_token(props, series))
        .and_then(color)
}

/// The color of a whole series.
pub(super) fn series_color(props: &ChartProps, series: usize) -> Option<Rgba> {
    props.series[series]
        .color
        .as_deref()
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
            if !point.value.is_finite()
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
    let glyphs = if props.ascii {
        GlyphSet::Ascii
    } else {
        GlyphSet::Unicode
    };
    let mut picture = Picture {
        width,
        height,
        ..Default::default()
    };
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
    if let Some(title) = &props.title {
        let strip = area.take_top(1);
        text.text(strip.x, strip.y, strip.w, title, None);
    }
    let visible = visible(props);
    if visible.iter().all(|(_, s)| s.data.is_empty()) {
        text.text(area.x, area.y, area.w, "No data to display", None);
        return compose(picture, &mask, &text, glyphs);
    }
    let legend = legend_area(props, &visible, class, &mut area);
    if !area.is_empty() {
        if matches!(props.chart_type, ChartType::Pie | ChartType::Donut) {
            pie::pie(&mut mask, &mut text, &mut picture, job, area);
        } else if let Err(error) =
            cartesian::cartesian(&mut mask, &mut text, &mut picture, job, area, class)
        {
            text.text(area.x, area.y, area.w, error, None);
        }
    }
    if let Some((rect, flowed)) = legend {
        let legend = Legend::new(
            visible
                .iter()
                .map(|(index, series)| LegendEntry {
                    name: series.name.clone(),
                    color: series_color(props, *index),
                })
                .collect(),
        );
        if flowed {
            legend.draw_flowed(
                &mut text,
                rect,
                rect.w.saturating_sub(2),
                class.multi_row_legend(),
            );
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
    visible: &[(usize, &DataSeries)],
    class: SizeClass,
    area: &mut Rect,
) -> Option<(Rect, bool)> {
    if !props.legend.visible || visible.is_empty() || !class.has_legend() {
        return None;
    }
    let entries = Legend::new(
        visible
            .iter()
            .map(|(_, s)| LegendEntry {
                name: s.name.clone(),
                color: None,
            })
            .collect(),
    );
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

/// Merge the mask and the text layers into cells and color runs.
fn compose(mut picture: Picture, mask: &MaskCanvas, text: &TextLayer, glyphs: GlyphSet) -> Picture {
    let (width, height) = (picture.width, picture.height);
    let mut cells = vec![Cell::default(); width * height];
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            let resolved = mask.resolve(x, y, glyphs);
            let cell = if let Some((t, c)) = &text.over[i] {
                Cell {
                    text: t.clone(),
                    color: *c,
                    owner: resolved.owner,
                }
            } else if let Some(glyph) = resolved.glyph {
                Cell {
                    text: glyph.to_string(),
                    color: resolved.color,
                    owner: resolved.owner,
                }
            } else if let Some((t, c)) = &text.under[i] {
                Cell {
                    text: t.clone(),
                    color: *c,
                    owner: None,
                }
            } else {
                Cell::default()
            };
            cells[i] = cell;
        }
    }
    picture.runs = runs(&cells, width, height);
    picture.cells = cells;
    picture
}

/// Run-length encode `cells` by row: a run keeps one color and continues
/// over blank cells, ending only where a visible cell of another color
/// starts. Every element the App lays out costs the same, so a row with one
/// series stroke is one element however often the stroke crosses it.
pub(super) fn runs(cells: &[Cell], width: usize, height: usize) -> Vec<Run> {
    let mut runs = Vec::new();
    let visible = |cell: &Cell| !cell.text.trim().is_empty();
    for y in 0..height {
        let row = &cells[y * width..(y + 1) * width];
        let mut x = 0;
        while x < width {
            // Skip leading blanks.
            while x < width && !visible(&row[x]) {
                x += 1;
            }
            if x >= width {
                break;
            }
            let start = x;
            let color = row[x].color;
            let mut text = String::new();
            let mut end = x;
            while x < width {
                let cell = &row[x];
                if visible(cell) && cell.color != color {
                    break;
                }
                if cell.text.is_empty() {
                    if x == 0 || UnicodeWidthStr::width(row[x - 1].text.as_str()) < 2 {
                        text.push(' ');
                    }
                } else {
                    text.push_str(&cell.text);
                }
                if visible(cell) {
                    end = x + 1;
                }
                x += 1;
            }
            // Drop trailing blanks so the next run can start there.
            let keep: String = text.graphemes(true).take(end - start).collect();
            x = end;
            runs.push(Run {
                y,
                x: start,
                width: end - start,
                text: keep,
                color,
            });
        }
    }
    runs
}

/// Absolutely positioned text elements for `runs`.
pub(super) fn elements(runs: &[Run]) -> Vec<Element> {
    runs.iter()
        .map(|run| {
            let mut style = StyleBuilder::new()
                .position_absolute()
                .inset_left(run.x as f32)
                .inset_top(run.y as f32)
                .width_px(run.width as f32)
                .height_px(1.0)
                .overflow_hidden();
            if let Some((r, g, b, a)) = run.color {
                style = style.fg_rgba(r, g, b, a);
            }
            ElementBuilder::new(ElementType::Text(run.text.clone()))
                .styles(style)
                .class("whitespace-pre")
                .build()
        })
        .collect()
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

    fn text_of(picture: &Picture) -> String {
        let mut out = String::new();
        for y in 0..picture.height {
            for x in 0..picture.width {
                let t = &picture.cells[y * picture.width + x].text;
                out.push_str(if t.is_empty() { " " } else { t });
            }
            out.push('\n');
        }
        out
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
        });
        let text = text_of(&picture);
        assert!(text.contains('█'), "{text}");
        assert!(!picture.runs.is_empty());
        let mut empty = props.clone();
        empty.series.clear();
        let picture = draw(&Job {
            props: &empty,
            width: 40,
            height: 12,
            values: &[],
            progress: 1.0,
        });
        assert!(
            text_of(&picture).contains("No data"),
            "{}",
            text_of(&picture)
        );
    }
}
