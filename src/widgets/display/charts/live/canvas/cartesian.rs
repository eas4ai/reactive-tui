//! Bar, line, area, scatter and candlestick rendering through the plot layer
//! (scales, ticks, axes, grid) into the shared mask canvas, or into the
//! scene of a plot picture where the terminal takes pixels (CHT-037). Every
//! cell, dot or pixel position comes from a scale; nothing here maps a
//! value itself.

use super::super::super::mask::Marker;
use super::super::super::plot::tick::spread_indices;
use super::super::super::plot::{
    self, band_ticks, decimate_min_max, fit_label, format_tick, label_skip, labeled_ticks,
    linear_ticks, point_ticks, spread_ticks, text_width, Axis, Grid, Ramp, Rect, Rgba, ScaleBand,
    ScaleLinear, ScalePoint, SizeClass, TextSink,
};
use super::super::super::{
    AxisLabelPlacement, BarGrowth, ChartAxis, ChartType, DataPoint, FillStyle, LineStyle,
};
use super::shapes::Shapes;
use super::{color, point_color, tick_count, Job, Picture, TextLayer};

const BULLISH: &str = "chart-bullish";
const BEARISH: &str = "chart-bearish";
/// The widest a mapped coordinate may wander outside the canvas, in dots:
/// far enough past any plot that clipping is exact, near enough that no
/// conversion overflows (CHT-026).
const FAR: f64 = 1.0e7;

/// The label of category `index`: a custom axis label, else the first point
/// label any visible series carries at that index.
fn category_label(job: &Job, axis: &ChartAxis, index: usize) -> Option<String> {
    axis.custom_labels.get(index).cloned().or_else(|| {
        job.props
            .series
            .iter()
            .filter(|s| s.visible)
            .find_map(|s| s.data.get(index).and_then(|p| p.label.clone()))
    })
}

/// Keep a mapped coordinate finite and near the canvas (CHT-026).
fn near(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-FAR, FAR)
    }
}

/// Grid lines at `positions`, in cells from the plot's left or top edge:
/// glyphs in the text's under layer on the mask, one-pixel lines in a
/// picture (CHT-037).
#[allow(clippy::too_many_arguments)]
fn grid_lines(
    target: &mut Shapes<'_>,
    text: &mut TextLayer,
    plot: Rect,
    positions: &[f64],
    vertical: bool,
    glyph: &'static str,
    dashed: bool,
    color: Option<Rgba>,
) {
    if target.is_pixels() {
        let (ux, uy) = target.units();
        for position in positions {
            let at = if vertical {
                (plot.x as f64 + position) * ux
            } else {
                (plot.y as f64 + position) * uy
            };
            target.grid_line(vertical, at, plot, dashed, color);
        }
        return;
    }
    let cells: Vec<usize> = positions.iter().map(|p| p.round() as usize).collect();
    let grid = if vertical {
        Grid::new(cells, Vec::new())
    } else {
        Grid::new(Vec::new(), cells)
    };
    grid.draw(text, plot, glyph, color);
}

/// The glyphs chrome is drawn with: Unicode, or ASCII when the builder
/// forces it or the terminal lacks the glyphs (CHT-028).
struct Glyphs {
    ascii: bool,
}

impl Glyphs {
    fn grid(&self, dashed: bool, vertical: bool) -> &'static str {
        match (self.ascii, dashed, vertical) {
            (true, true, _) => ".",
            (true, false, true) => "|",
            (true, false, false) => "-",
            (false, true, _) => "·",
            (false, false, true) => "│",
            (false, false, false) => "─",
        }
    }
    fn reference(&self) -> &'static str {
        if self.ascii {
            "-"
        } else {
            "┄"
        }
    }
    fn pattern_dot(&self) -> &'static str {
        if self.ascii {
            "."
        } else {
            "·"
        }
    }
}

/// Value labels placed so that none overlaps another or leaves the plot
/// (CHT-013): a label that cannot be placed whole is left out.
#[derive(Default)]
struct LabelPlacer {
    /// Column spans taken per row, as (row, first, last).
    taken: Vec<(usize, usize, usize)>,
}

impl LabelPlacer {
    /// Place `label` with its left edge at `x` on `row` inside `plot`,
    /// shifting it into the plot when it would leave it; `None` when it
    /// would then overlap a placed label or is wider than the plot.
    fn place(&mut self, plot: Rect, x: usize, row: usize, width: usize) -> Option<usize> {
        if width == 0 || width > plot.w || row < plot.y || row >= plot.bottom() {
            return None;
        }
        let x = x.max(plot.x).min(plot.right() - width);
        let (first, last) = (x, x + width - 1);
        let clash = self
            .taken
            .iter()
            .any(|(r, a, b)| *r == row && first <= *b + 1 && *a <= last + 1);
        if clash {
            return None;
        }
        self.taken.push((row, first, last));
        Some(x)
    }

    /// Place `label` on the first of `rows` with room at `x`; `None` when no
    /// row has room. The rows run outward from a bar's tip, so a label
    /// that would overlap a neighbour's steps one row further (CHT-013).
    fn place_rows(
        &mut self,
        plot: Rect,
        x: usize,
        rows: impl IntoIterator<Item = usize>,
        width: usize,
    ) -> Option<(usize, usize)> {
        rows.into_iter()
            .find_map(|row| self.place(plot, x, row, width).map(|x| (x, row)))
    }

    /// Place `label` on `row` at `x`, or at the nearest free span past the
    /// labels already on that row in the growth direction (`rightward`),
    /// inside the plot; `None` when the row has no room left (CHT-013).
    fn place_along(
        &mut self,
        plot: Rect,
        x: usize,
        row: usize,
        width: usize,
        rightward: bool,
    ) -> Option<usize> {
        if width == 0 || width > plot.w {
            return None;
        }
        let mut x = x.max(plot.x).min(plot.right() - width);
        loop {
            if let Some(placed) = self.place(plot, x, row, width) {
                return Some(placed);
            }
            let (first, last) = (x, x + width - 1);
            let clashes = self
                .taken
                .iter()
                .filter(|(r, a, b)| *r == row && first <= *b + 1 && *a <= last + 1);
            if rightward {
                let past = clashes.map(|(_, _, b)| *b + 2).max()?;
                if past + width > plot.right() {
                    return None;
                }
                x = past;
            } else {
                let before = clashes.map(|(_, a, _)| *a).min()?;
                x = before.checked_sub(width + 1)?;
                if x < plot.x {
                    return None;
                }
            }
        }
    }
}

pub(super) fn cartesian(
    target: &mut Shapes<'_>,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    mut area: Rect,
    class: SizeClass,
) -> Result<(), &'static str> {
    let props = job.props;
    // A cell measures two by four dots on the mask and the terminal's pixels
    // in a picture: every position below is in those units (CHT-037).
    let (ux, uy) = target.units();
    let pixels = target.is_pixels();
    // The hover marks a picture draws (CHT-038).
    let hover = job.pixels.as_ref().and_then(|p| p.hover).filter(|_| pixels);
    let glyphs = Glyphs {
        ascii: props.ascii || !job.unicode_glyphs,
    };
    let border = color("border");
    let muted = color("text-muted");
    let grid_color = color("chart-grid").or(border);
    let background = color("background");
    let vis: Vec<usize> = props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .map(|(i, _)| i)
        .collect();
    let data_count = vis
        .iter()
        .map(|s| props.series[*s].data.len())
        .max()
        .unwrap_or(0);
    let bars = matches!(
        props.chart_type,
        ChartType::BarVertical | ChartType::BarHorizontal
    );
    let candles = props.chart_type == ChartType::Candlestick;
    let scatter = props.chart_type == ChartType::Scatter;
    // The category axis is laid out for the data, or for more slots when
    // the builder asked (CHT-034); the slots past the data stay empty.
    let slots = if bars {
        props.band_count
    } else {
        props.point_count
    };
    let count = slots.map_or(data_count, |n| n.max(data_count));
    let horizontal =
        props.chart_type == ChartType::BarHorizontal || (bars && props.growth.is_horizontal());
    let growth = if horizontal && !props.growth.is_horizontal() {
        BarGrowth::Left
    } else if !horizontal && props.growth.is_horizontal() {
        BarGrowth::Bottom
    } else {
        props.growth
    };
    // The value axis is `y_axis` for vertical charts and `x_axis` for
    // horizontal bars; the other axis carries the categories.
    let (value_axis, category_axis) = if horizontal {
        (&props.x_axis, &props.y_axis)
    } else {
        (&props.y_axis, &props.x_axis)
    };
    // During a transition the automatic range moves from the range of the
    // values it started from to the target's, at the transition's own
    // progress, so a shape keeps its height while both grow and never dips
    // below its previous rendering (CHT-022); a pinned range stays, and a
    // reveal keeps the target range.
    let domain = match job.transition {
        Some((t, from)) => {
            let start = plot::value_domain_with(props, value_axis, Some(from))?;
            let end = plot::value_domain_with(props, value_axis, None)?;
            let t = t.clamp(0.0, 1.0);
            (
                start.0 + (end.0 - start.0) * t,
                start.1 + (end.1 - start.1) * t,
            )
        }
        None => plot::value_domain_with(props, value_axis, None)?,
    };
    // A scatter with numeric x places its points on a linear x axis
    // (CHT-033); one without keeps its categories.
    let numeric_x = scatter
        && vis
            .iter()
            .any(|s| props.series[*s].data.iter().any(|p| p.x.is_some()));
    let x_domain = numeric_x.then(|| {
        let xs = vis.iter().flat_map(|s| {
            props.series[*s]
                .data
                .iter()
                .enumerate()
                .map(|(i, p)| p.x.unwrap_or(i as f64))
        });
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        for x in xs.filter(|x| x.is_finite()) {
            low = low.min(x);
            high = high.max(x);
        }
        let domain = if low > high {
            (0.0, 1.0)
        } else if low == high {
            (low - 1.0, high + 1.0)
        } else {
            (low, high)
        };
        // Exactly the x tick count of round ticks, as on the value axis
        // (CHT-034): the free ends widen to a round step grid.
        plot::nice_domain(domain, (false, false), category_axis.tick_count)
    });
    let axes = class.has_axes();
    if let Some(title) = &props.y_axis.title {
        if axes {
            let strip = area.take_top(1);
            text.text(strip.x, strip.y, strip.w, title, muted);
        }
    }
    // Explicit ticks from a tick format (CHT-034), else custom labels, else
    // round values.
    let value_cells = if horizontal { area.w } else { area.h };
    let value_ticks = |scale: &ScaleLinear| -> Vec<plot::Tick> {
        if !value_axis.ticks.is_empty() {
            // No more of them than the plot has cells, as for the round
            // values below (CHT-040).
            let shown: Vec<&(f64, String)> = value_axis
                .ticks
                .iter()
                .filter(|(value, _)| scale.contains(*value))
                .collect();
            spread_indices(shown.len(), tick_count(value_axis, value_cells))
                .map(|i| {
                    let (value, label) = shown[i];
                    plot::Tick {
                        value: *value,
                        position: scale.map(*value),
                        label: label.clone(),
                    }
                })
                .collect()
        } else if value_axis.custom_labels.is_empty() {
            linear_ticks(scale, tick_count(value_axis, value_cells))
        } else {
            labeled_ticks(scale, &value_axis.custom_labels)
        }
    };
    let inside = value_axis.placement == AxisLabelPlacement::Inside;
    let mut label_width = 0;
    let mut x_rows = 0;
    if axes {
        if !horizontal && value_axis.show_labels && value_axis.tick_count > 0 && !inside {
            let probe = ScaleLinear::new(domain, (0.0, 1.0));
            label_width = value_ticks(&probe)
                .iter()
                .map(|t| text_width(&t.label))
                .max()
                .unwrap_or(0)
                .min(area.w / 3)
                + 1;
        }
        if horizontal && category_axis.show_labels {
            label_width = (0..data_count)
                .filter_map(|i| category_label(job, category_axis, i))
                .map(|l| text_width(&l))
                .max()
                .unwrap_or(0)
                .min(area.w / 3)
                + 1;
        }
        let bottom_axis = if horizontal {
            value_axis
        } else {
            category_axis
        };
        let bottom_labels = bottom_axis.show_labels && !(horizontal && inside);
        if bottom_labels && (horizontal || count > 0) {
            x_rows = 1 + usize::from(props.x_axis.title.is_some());
        } else if props.x_axis.title.is_some() {
            x_rows = 1;
        }
    }
    let label_rect = area.take_left(label_width.min(area.w.saturating_sub(1)));
    let x_rect = area.take_bottom(x_rows.min(area.h.saturating_sub(1)));
    let plot = area;
    if plot.is_empty() {
        return Ok(());
    }
    // Shapes keep off the axis line cells at the plot's left and bottom.
    let left_line = axes
        && if horizontal {
            category_axis.show_labels
        } else {
            value_axis.show_labels
        };
    let bottom_line = axes
        && (if horizontal {
            value_axis.show_labels
        } else {
            category_axis.show_labels
        } || props.x_axis.title.is_some());
    let inner = Rect {
        x: plot.x + usize::from(left_line),
        y: plot.y,
        w: plot.w.saturating_sub(usize::from(left_line)),
        h: plot.h.saturating_sub(usize::from(bottom_line)),
    };
    if inner.is_empty() {
        return Ok(());
    }
    target.begin(inner);
    picture.plot = plot;
    picture.inner = inner;
    picture.scatter = scatter;
    let value_labels = props
        .value_labels
        .unwrap_or_else(|| class.has_value_labels());
    // Headroom: rows (or columns, for horizontal bars) kept clear past the
    // extreme values (CHT-034), plus room for the value labels past the
    // longest bar (CHT-013): as many rows as labels wider than a bar's lane
    // need to stagger without overlapping, or the widest label's columns
    // beside a horizontal bar. The shapes' range shrinks; the axis line
    // keeps the whole plot.
    let (head, foot) = (usize::from(props.headroom.0), usize::from(props.headroom.1));
    let widest_label = if bars && value_labels {
        vis.iter()
            .flat_map(|&s| props.series[s].data.iter())
            .map(|p| {
                text_width(
                    &p.value_label
                        .clone()
                        .unwrap_or_else(|| format_tick(p.value)),
                )
            })
            .max()
            .unwrap_or(0)
    } else {
        0
    };
    let lanes = if props.stacked || vis.len() < 2 {
        1
    } else {
        vis.len()
    };
    let label_room = if bars && value_labels && !horizontal {
        let lane_cells = (inner.w / count.max(1).saturating_mul(lanes)).max(1);
        (widest_label + 1).div_ceil(lane_cells).max(1)
    } else {
        0
    };
    let label_cols = if bars && value_labels && horizontal {
        widest_label + 1
    } else {
        0
    };
    let shapes = if horizontal {
        // Tips point right when bars grow from the left edge.
        let (mut left, mut right) = (foot, head);
        if growth == BarGrowth::Left {
            right += label_cols;
        } else {
            left += label_cols;
        }
        let (left, right) = (left.min(inner.w / 2), right.min(inner.w / 2));
        Rect {
            x: inner.x + left,
            w: inner.w.saturating_sub(left + right).max(1),
            ..inner
        }
    } else {
        let top = (head + label_room).min(inner.h / 2);
        let bottom = foot.min(inner.h / 2);
        Rect {
            y: inner.y + top,
            h: inner.h.saturating_sub(top + bottom).max(1),
            ..inner
        }
    };

    // Scales in dot units.
    let x_dots = (shapes.x as f64 * ux, shapes.right() as f64 * ux);
    let y_dots = (shapes.y as f64 * uy, shapes.bottom() as f64 * uy);
    let (y_dots, x_dots) = if bars || candles {
        (y_dots, x_dots)
    } else {
        // Strokes and markers land on dots, so the range ends on the last
        // dot inside the plot rather than on its far edge.
        ((y_dots.0, (y_dots.1 - 1.0).max(y_dots.0)), x_dots)
    };
    let value_scale = match growth {
        BarGrowth::Bottom => ScaleLinear::new(domain, (y_dots.1, y_dots.0)),
        BarGrowth::Top => ScaleLinear::new(domain, (y_dots.0, y_dots.1)),
        BarGrowth::Left => ScaleLinear::new(domain, (x_dots.0, x_dots.1)),
        BarGrowth::Right => ScaleLinear::new(domain, (x_dots.1, x_dots.0)),
    };
    let category_range = if horizontal { y_dots } else { x_dots };
    // Band padding: the builder's, else the agreed defaults of 0.4 inside
    // and 0.2 outside of a band, at every class and series count (CHT-013).
    let padding_inner = props.padding_inner.unwrap_or(0.4);
    let padding_outer = props.padding_outer.unwrap_or(0.2);
    let band = ScaleBand::new(count, category_range)
        .padding_inner(padding_inner)
        .padding_outer(padding_outer);
    let points = ScalePoint::new(
        count,
        (
            category_range.0,
            (category_range.1 - 1.0).max(category_range.0),
        ),
    );
    let x_scale = x_domain.map(|d| ScaleLinear::new(d, (x_dots.0, (x_dots.1 - 1.0).max(x_dots.0))));
    let use_band = bars || candles;
    // A band capped at `max_band_width` cells, centred in its slot
    // (CHT-013, CHT-014).
    let band_unit = if horizontal { uy } else { ux };
    let capped_band = |i: usize| -> (f64, f64) {
        let (a, b) = band.band(i);
        match props.max_band_width {
            Some(max) if (b - a) > f64::from(max) * band_unit => {
                let width = f64::from(max) * band_unit;
                let center = (a + b) / 2.0;
                (center - width / 2.0, center + width / 2.0)
            }
            _ => (a, b),
        }
    };
    let category_center = |i: usize| -> f64 {
        if use_band {
            let (a, b) = capped_band(i);
            (a + b) / 2.0
        } else {
            points.map(i)
        }
    };
    if !numeric_x {
        if horizontal {
            picture.index_rows = (0..data_count).map(|i| category_center(i) / uy).collect();
        } else {
            picture.index_columns = (0..data_count).map(|i| category_center(i) / ux).collect();
        }
    }

    // Grid, axes and labels.
    if axes {
        let ticks = value_ticks(&value_scale);
        // `tick_margin` is a stride over the category labels, whatever the
        // orientation (CHT-034); `label_count` spreads that many labels from
        // the first to the last instead.
        let stride = props.tick_margin.max(1);
        let show_grid = |axis: &ChartAxis| axis.show_grid && class.has_grid();
        let category_ticks: Vec<plot::Tick> = {
            // Labels and ticks for the data alone: the slots past it stay
            // empty, however many the builder asked for (CHT-034, CHT-040).
            let labels: Vec<Option<String>> = (0..data_count.min(count))
                .map(|i| category_label(job, category_axis, i))
                .collect();
            let mut all = if numeric_x {
                x_scale
                    .as_ref()
                    .map(|scale| linear_ticks(scale, tick_count(category_axis, shapes.w)))
                    .unwrap_or_default()
            } else if use_band {
                band_ticks(&band, &labels, data_count)
            } else {
                point_ticks(&points, &labels, data_count)
            };
            // Slots past the data carry no label.
            if !numeric_x {
                all.truncate(data_count);
            }
            match category_axis.label_count {
                Some(n) => spread_ticks(all, n),
                None => all,
            }
        };
        let category_skip = if category_axis.label_count.is_some() || numeric_x {
            1
        } else {
            stride
        };
        // Grid lines: the value ticks' rows (or columns), and the category
        // ticks' columns or `grid_columns` even divisions (CHT-034).
        // Grid positions in cells from the plot's edge: exact in a picture,
        // rounded to a cell on the mask by `grid_lines`.
        let grid_columns_of = |axis_ticks: &[plot::Tick], plot_w: usize| -> Vec<f64> {
            // More columns than the plot has dots across, a picture's pixels
            // or a mask's cells, draw the same lines: the count is held to
            // them and the lines stay spread over the plot (CHT-040).
            let resolvable = plot_w.saturating_mul(ux.ceil().max(1.0) as usize).max(1);
            match category_axis
                .grid_columns
                .or(props.x_axis.grid_columns)
                .map(|n| n.min(resolvable))
            {
                Some(n) if n > 0 => (0..n)
                    .map(|k| {
                        if pixels {
                            k as f64 * plot_w as f64 / n as f64
                        } else {
                            (k * plot_w / n) as f64
                        }
                    })
                    .collect(),
                _ => axis_ticks
                    .iter()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position))
                    .collect(),
            }
        };
        let to_cells = |t: &plot::Tick, vertical: bool| plot::Tick {
            value: t.value,
            position: if vertical {
                t.position / uy - plot.y as f64
            } else {
                t.position / ux - plot.x as f64
            },
            label: t.label.clone(),
        };
        // Reference lines at their values, in units: a row on a vertical
        // chart, a column on a horizontal one.
        let mut reference_rows: Vec<(f64, bool)> = Vec::new();
        for value in &props.reference_lines {
            if value_scale.contains(*value) {
                reference_rows.push((near(value_scale.map(*value)), !horizontal));
            }
        }
        if horizontal {
            let mut vertical =
                Axis::vertical(category_ticks.iter().map(|t| to_cells(t, true)).collect())
                    .with_skip(category_skip)
                    .with_line_color(border)
                    .with_ascii(glyphs.ascii);
            vertical.line = category_axis.show_labels;
            let horizontal_axis =
                Axis::horizontal(ticks.iter().map(|t| to_cells(t, false)).collect())
                    .with_title(props.x_axis.title.clone())
                    .with_line_color(border)
                    .with_ascii(glyphs.ascii);
            // Value labels along the bottom thin only to avoid overlap.
            let skip = label_skip(
                &horizontal_axis
                    .ticks
                    .iter()
                    .map(|t| t.position)
                    .collect::<Vec<_>>(),
                &horizontal_axis
                    .ticks
                    .iter()
                    .map(|t| text_width(&t.label))
                    .collect::<Vec<_>>(),
                1,
            );
            let horizontal_axis = horizontal_axis.with_skip(skip);
            if show_grid(value_axis) {
                let columns: Vec<f64> = horizontal_axis
                    .shown()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position))
                    .collect();
                grid_lines(
                    target,
                    text,
                    plot,
                    &columns,
                    true,
                    glyphs.grid(value_axis.dashed, true),
                    value_axis.dashed,
                    grid_color,
                );
            }
            if show_grid(category_axis) {
                let rows: Vec<f64> = vertical
                    .ticks
                    .iter()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position))
                    .collect();
                grid_lines(
                    target,
                    text,
                    plot,
                    &rows,
                    false,
                    glyphs.grid(category_axis.dashed, false),
                    category_axis.dashed,
                    grid_color,
                );
            }
            if category_axis.show_labels {
                vertical.draw(text, label_rect, plot, muted);
            }
            if value_axis.show_labels || props.x_axis.title.is_some() {
                if inside && value_axis.show_labels {
                    // Value labels inside the plot, on its bottom row beside
                    // their grid columns (CHT-034).
                    let row = plot.bottom().saturating_sub(1);
                    for tick in horizontal_axis.shown() {
                        let col = plot.x + tick.position.round().max(0.0) as usize;
                        let width = text_width(&tick.label);
                        if col + width <= plot.right() {
                            text.text(col, row, width, &tick.label, muted);
                        }
                    }
                    if props.x_axis.title.is_some() {
                        Axis::horizontal(Vec::new())
                            .with_title(props.x_axis.title.clone())
                            .with_line_color(border)
                            .with_ascii(glyphs.ascii)
                            .draw(text, x_rect, plot, muted);
                    }
                } else {
                    horizontal_axis.draw(text, x_rect, plot, muted);
                }
            }
        } else {
            let vertical = Axis::vertical(ticks.iter().map(|t| to_cells(t, true)).collect())
                .with_line_color(border)
                .with_ascii(glyphs.ascii);
            let horizontal_axis =
                Axis::horizontal(category_ticks.iter().map(|t| to_cells(t, false)).collect())
                    .with_title(props.x_axis.title.clone())
                    .with_line_color(border)
                    .with_ascii(glyphs.ascii);
            let skip = if props.tick_margin > 0 && !numeric_x && category_axis.label_count.is_none()
            {
                stride
            } else {
                label_skip(
                    &horizontal_axis
                        .ticks
                        .iter()
                        .map(|t| t.position)
                        .collect::<Vec<_>>(),
                    &horizontal_axis
                        .ticks
                        .iter()
                        .map(|t| text_width(&t.label))
                        .collect::<Vec<_>>(),
                    1,
                )
            };
            let horizontal_axis = horizontal_axis.with_skip(skip);
            if show_grid(value_axis) {
                let rows: Vec<f64> = vertical
                    .shown()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position))
                    .collect();
                grid_lines(
                    target,
                    text,
                    plot,
                    &rows,
                    false,
                    glyphs.grid(value_axis.dashed, false),
                    value_axis.dashed,
                    grid_color,
                );
            }
            if show_grid(category_axis) || category_axis.grid_columns.is_some() {
                let shown: Vec<plot::Tick> = horizontal_axis.shown().cloned().collect();
                let columns = grid_columns_of(&shown, plot.w);
                grid_lines(
                    target,
                    text,
                    plot,
                    &columns,
                    true,
                    glyphs.grid(category_axis.dashed, true),
                    category_axis.dashed,
                    grid_color,
                );
            }
            if value_axis.show_labels {
                if inside {
                    // Value labels inside the plot beside their grid rows
                    // (CHT-034); the axis line still marks the plot's edge.
                    let mut line_only = Axis::vertical(Vec::new())
                        .with_line_color(border)
                        .with_ascii(glyphs.ascii);
                    line_only.line = true;
                    line_only.draw(text, label_rect, plot, muted);
                    for tick in vertical.shown() {
                        let row = tick.position.round();
                        if row < 0.0 || row as usize >= plot.h {
                            continue;
                        }
                        let width = text_width(&tick.label);
                        let x = plot.x + 1;
                        if x + width <= plot.right() {
                            text.text(x, plot.y + row as usize, width, &tick.label, muted);
                        }
                    }
                } else {
                    vertical.draw(text, label_rect, plot, muted);
                }
            }
            if category_axis.show_labels || props.x_axis.title.is_some() {
                horizontal_axis.draw(text, x_rect, plot, muted);
            }
        }
        // Reference lines: dashed across the plot at their values, under the
        // shapes (CHT-034).
        for (at, is_row) in reference_rows {
            if pixels {
                // A one-pixel dashed line at the exact value (CHT-037).
                target.grid_line(!is_row, at, inner, true, border);
                continue;
            }
            let at = (at / if is_row { uy } else { ux }).round() as usize;
            if is_row {
                if at >= plot.y && at < plot.bottom() {
                    for col in inner.x..inner.right() {
                        text.under(col, at, glyphs.reference(), border);
                    }
                }
            } else if at >= plot.x && at < plot.right() {
                for row in inner.y..inner.bottom() {
                    text.under(at, row, glyphs.reference(), border);
                }
            }
        }
    }

    // Explicit limits on the category axis clip by index.
    let index_visible = |i: usize| {
        category_axis.min.is_none_or(|m| i as f64 >= m)
            && category_axis.max.is_none_or(|m| i as f64 <= m)
    };
    let cell_of = |x: f64, y: f64| -> (usize, usize) {
        (
            (near(x) / ux).floor().max(0.0) as usize,
            (near(y) / uy).floor().max(0.0) as usize,
        )
    };
    // Mapped coordinates stay within a few plot spans of the plot, so a
    // value far outside tight limits becomes a short clipped segment rather
    // than a walk toward infinity in the curve densifier or the fill loop
    // (CHT-026).
    let guarded = |m: f64, range: (f64, f64)| -> f64 {
        let (lo, hi) = (range.0.min(range.1), range.0.max(range.1));
        let guard = 8.0 * (hi - lo).max(uy);
        if m.is_nan() {
            lo
        } else {
            m.clamp(lo - guard, hi + guard)
        }
    };
    let map = |v: f64| guarded(value_scale.map(v), value_scale.range());

    if bars {
        // A picture rounds the corners by the builder's radius (CHT-013).
        let radius = if pixels {
            props.corner_radius * uy as f32
        } else {
            0.0
        };
        // The hover band sits over the hovered band and under the bars, its
        // center where the glide has it; every bar fades toward the
        // background by its distance from that center, a band's step away
        // in full (CHT-038).
        let step_cells = band.step() / band_unit;
        if let Some(h) = hover {
            let center = h.band * band_unit;
            let half = band.step() / 2.0;
            if horizontal {
                target.rect(
                    inner.x as f64 * ux,
                    center - half,
                    inner.right() as f64 * ux,
                    center + half,
                    color("hover"),
                    None,
                    0.0,
                    h.focus,
                );
            } else {
                target.rect(
                    center - half,
                    inner.y as f64 * uy,
                    center + half,
                    inner.bottom() as f64 * uy,
                    color("hover"),
                    None,
                    0.0,
                    h.focus,
                );
            }
        }
        let emphasis = |i: usize| -> f32 {
            match hover {
                // The hovered bar keeps its full color from the first frame
                // of the hover, while the band is still gliding toward it
                // (CHT-038).
                Some(h) if h.index == i => 1.0,
                Some(h) if step_cells > 0.0 => {
                    let center = category_center(i) / band_unit;
                    let distance = ((center - h.band).abs() / step_cells).min(1.0) as f32;
                    1.0 - 0.45 * h.focus * distance
                }
                _ => 1.0,
            }
        };
        // Stacks for the data's indices alone (CHT-040).
        let mut stack_pos = vec![0.0f64; data_count];
        let mut stack_neg = vec![0.0f64; data_count];
        let mut placer = LabelPlacer::default();
        // Labels are placed after every bar is drawn, tallest first, so the
        // bars that matter most keep theirs when room is short.
        let mut labels: Vec<(usize, usize, String, Option<Rgba>, bool, f64)> = Vec::new();
        for (slot, &s) in vis.iter().enumerate() {
            let series = &props.series[s];
            for (i, point) in series.data.iter().enumerate() {
                if !index_visible(i) {
                    continue;
                }
                let Some(value) = job.values.get(s).and_then(|v| v.get(i)).copied() else {
                    continue;
                };
                let tint = point_color(props, s, i);
                let (lane_start, lane_end) = {
                    let (a, b) = capped_band(i);
                    let (a, b) = if props.stacked || vis.len() < 2 {
                        (a, b)
                    } else {
                        let lanes = ScaleBand::new(vis.len(), (a, b)).padding_inner(0.15);
                        lanes.band(slot)
                    };
                    // Bar sides sit on cell boundaries so only the tip is
                    // fractional; a lane always keeps at least one cell. In
                    // a picture they sit on whole pixels, so every edge is
                    // crisp, and a lane keeps at least one pixel.
                    if pixels {
                        let start = a.round();
                        (start, b.round().max(start + 1.0))
                    } else {
                        let start = (a / band_unit).round() * band_unit;
                        let end = ((b / band_unit).round() * band_unit).max(start + band_unit);
                        (start, end)
                    }
                };
                let base = if props.stacked {
                    if value >= 0.0 {
                        stack_pos[i]
                    } else {
                        stack_neg[i]
                    }
                } else {
                    0.0
                };
                let top = base + value;
                if props.stacked {
                    if value >= 0.0 {
                        stack_pos[i] = top;
                    } else {
                        stack_neg[i] = top;
                    }
                }
                // Bar edges snap to eighths of a cell so every tip resolves
                // to an eighth block (CHT-013) while 3.5 of 8 still differs
                // from 3 and 4.
                let dots = if horizontal { ux } else { uy };
                // In a picture a tip lands on its nearest whole pixel, so
                // 3.5 of 10 still differs from 3 and 4 and every edge is
                // crisp (CHT-013).
                let snap = |v: f64| {
                    if pixels {
                        v.round()
                    } else {
                        (v * 8.0 / dots).round() * dots / 8.0
                    }
                };
                let (from, mut to) = (snap(map(base)), snap(map(top)));
                // A bar shorter than the minimum length grows to it, in the
                // direction it grows (CHT-013).
                let min_dots = props.min_bar_length.max(0.0) * dots;
                if (to - from).abs() < min_dots && min_dots > 0.0 && value != 0.0 {
                    let sign = if (to - from) != 0.0 {
                        (to - from).signum()
                    } else {
                        let up = matches!(growth, BarGrowth::Bottom | BarGrowth::Right);
                        if (value >= 0.0) == up {
                            -1.0
                        } else {
                            1.0
                        }
                    };
                    to = snap(from + sign * min_dots);
                }
                if (from - to).abs() < 1e-9 {
                    continue;
                }
                let ramp = Ramp::new(
                    point
                        .gradient
                        .iter()
                        .filter_map(|(offset, token)| color(token).map(|c| (*offset, c))),
                );
                let alpha = emphasis(i);
                if ramp.is_empty() {
                    if horizontal {
                        target.rect(
                            from,
                            lane_start,
                            to,
                            lane_end,
                            tint,
                            Some((s, i)),
                            radius,
                            alpha,
                        );
                    } else {
                        target.rect(
                            lane_start,
                            from,
                            lane_end,
                            to,
                            tint,
                            Some((s, i)),
                            radius,
                            alpha,
                        );
                    }
                } else {
                    target.gradient_bar(
                        from,
                        to,
                        lane_start,
                        lane_end,
                        horizontal,
                        &ramp,
                        tint,
                        Some((s, i)),
                        radius,
                        alpha,
                    );
                }
                let tip = if horizontal {
                    cell_of(
                        to - if to > from { 0.01 } else { -0.01 },
                        (lane_start + lane_end) / 2.0,
                    )
                } else {
                    cell_of(
                        (lane_start + lane_end) / 2.0,
                        to - if to > from { 0.01 } else { -0.01 },
                    )
                };
                picture.anchors.insert((s, i), tip);
                if value_labels {
                    // A builder-supplied label (typed `.label(..)`) replaces
                    // the formatted value (CHT-020, CHT-013), in the label's
                    // own color when one is set.
                    let label = point
                        .value_label
                        .clone()
                        .unwrap_or_else(|| format_tick(point.value));
                    let label_color = point.label_color.as_deref().and_then(color).or(tint);
                    labels.push((tip.0, tip.1, label, label_color, value >= 0.0, value.abs()));
                }
            }
        }
        labels.sort_by(|a, b| b.5.total_cmp(&a.5));
        for (tip_x, tip_y, label, label_color, positive, _) in labels {
            let width = text_width(&label);
            if horizontal {
                // Beside the tip, past it in the direction the bar grew (a
                // positive bar from the left and a negative bar from the
                // right grow rightward), on the bar's row; a bar sharing its
                // row with another takes the next free span along the row.
                let rightward = (growth == BarGrowth::Left) == positive;
                let x = if rightward {
                    tip_x + 1
                } else {
                    tip_x.saturating_sub(width)
                };
                if let Some(x) = placer.place_along(plot, x, tip_y, width, rightward) {
                    text.text(x, tip_y, width, &label, label_color);
                }
            } else {
                // Above the tip (or below a bar that hangs or is negative),
                // in the headroom rows the plot kept, one row further out
                // when a neighbour's label is in the way; the tip row itself
                // when no row past it has room.
                let outward = (growth == BarGrowth::Bottom) == positive;
                let rows: Vec<usize> = if outward {
                    (1..=label_room.max(1))
                        .filter_map(|k| tip_y.checked_sub(k))
                        .filter(|y| plot.contains(tip_x, *y))
                        .collect()
                } else {
                    (1..=label_room.max(1))
                        .map(|k| tip_y + k)
                        .filter(|y| plot.contains(tip_x, *y))
                        .collect()
                };
                let x = (tip_x + 1).saturating_sub(width.div_ceil(2));
                if let Some((x, y)) = placer.place_rows(
                    plot,
                    x,
                    rows.into_iter().chain(std::iter::once(tip_y)),
                    width,
                ) {
                    text.text(x, y, width, &label, label_color);
                }
            }
        }
        return Ok(());
    }

    if candles {
        let bullish = color(BULLISH);
        let bearish = color(BEARISH);
        for &s in &vis {
            let series = &props.series[s];
            let animated = job
                .values
                .get(s)
                .filter(|v| v.len() == series.data.len() * 4);
            for (i, point) in series.data.iter().enumerate() {
                if !index_visible(i) {
                    continue;
                }
                let Some(candle) = point.candle else { continue };
                // The candle as motion has it now: the four values the
                // transition and the reveal carry (CHT-014, CHT-022).
                let (open, high, low, close) = match animated {
                    Some(values) => (
                        values[i * 4],
                        values[i * 4 + 1],
                        values[i * 4 + 2],
                        values[i * 4 + 3],
                    ),
                    None => (candle.open, candle.high, candle.low, candle.close),
                };
                let tint = point
                    .color
                    .as_deref()
                    .or(series.color.as_deref())
                    .and_then(color)
                    .or(if close > open { bullish } else { bearish });
                let (a, b) = capped_band(i);
                // The body takes a ratio of its band (CHT-014): on the mask
                // at least one cell, on cell boundaries like bars; in a
                // picture at least one pixel, on whole pixels, so narrow
                // bands keep their ratio and their gaps. The wick is one
                // dot wide at the band's centre.
                let unit = if pixels { 1.0 } else { ux };
                let ratio = f64::from(props.body_width_ratio.clamp(0.05, 1.0));
                let center = (a + b) / 2.0;
                let half = ((b - a) * ratio / 2.0).max(unit / 2.0);
                let a = ((center - half) / unit).round() * unit;
                let b = (((center + half) / unit).round() * unit).max(a + unit);
                let center = (a + b) / 2.0;
                let (wick_top, wick_bottom) = (map(high), map(low));
                // The wick is one dot wide on the mask and an eighth of the
                // cell width, at least a pixel, in a picture (CHT-014).
                let wick = if pixels { (ux / 16.0).max(0.5) } else { 0.5 };
                target.rect(
                    center - wick,
                    wick_top,
                    center + wick,
                    wick_bottom,
                    tint,
                    Some((s, i)),
                    0.0,
                    1.0,
                );
                let (mut open_at, mut close_at) = (map(open), map(close));
                if (open_at - close_at).abs() < 0.5 {
                    open_at -= 0.25;
                    close_at += 0.25;
                }
                target.rect(a, open_at, b, close_at, tint, Some((s, i)), 0.0, 1.0);
                picture
                    .anchors
                    .insert((s, i), cell_of(center, open_at.min(close_at)));
            }
        }
        if let Some(h) = hover {
            // The crosshair through the hovered candle (CHT-038).
            target.crosshair(
                category_center(h.index),
                inner,
                color("text-muted"),
                h.focus,
            );
        }
        return Ok(());
    }

    // Line, area and scatter.
    // Stacked areas stack at every index before decimation, so an upper
    // series' kept samples read the lower series' top at the same index
    // whichever indices each series keeps (CHT-012).
    let stacked_area = props.stacked && props.chart_type == ChartType::Area;
    let mut tops: Vec<Vec<f64>> = Vec::new();
    if stacked_area {
        let mut acc = vec![0.0f64; data_count];
        for &s in &vis {
            if let Some(values) = job.values.get(s) {
                for (a, v) in acc.iter_mut().zip(values) {
                    *a += v;
                }
            }
            tops.push(acc.clone());
        }
    }
    picture.kept = vec![Vec::new(); props.series.len()];
    let x_of = |s: usize, index: usize| -> f64 {
        match &x_scale {
            Some(scale) => {
                let x = props.series[s]
                    .data
                    .get(index)
                    .and_then(|p| p.x)
                    .unwrap_or(index as f64);
                guarded(scale.map(x), scale.range())
            }
            None => points.map(index),
        }
    };
    for (position, &s) in vis.iter().enumerate() {
        let series = &props.series[s];
        let Some(values) = job.values.get(s) else {
            continue;
        };
        let stacked_values = if stacked_area {
            &tops[position]
        } else {
            values
        };
        // A scatter on a numeric x is not in column order, so it is thinned
        // per column of its mapped x, a cell on the mask and a pixel in a
        // picture: each column keeps its lowest and highest point
        // (CHT-027); every other point keeps its anchor below.
        let column_unit = if pixels { 1.0 } else { ux };
        let samples: Vec<_> = if numeric_x {
            plot::decimate_by_column(
                values
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| index_visible(*index))
                    .map(|(index, value)| {
                        let column = (x_of(s, index) / column_unit).floor().max(0.0) as usize;
                        (index, column, *value)
                    }),
            )
        } else {
            decimate_min_max(
                stacked_values,
                // A column per cell on the mask, per pixel in a picture
                // (CHT-027).
                if pixels {
                    (shapes.w as f64 * ux) as usize
                } else {
                    shapes.w
                },
            )
            .into_iter()
            .filter(|k| index_visible(k.index))
            .collect()
        };
        picture.kept[s] = samples.iter().map(|k| k.index).collect();
        let tint = point_color(props, s, 0);
        let curve = series.curve.unwrap_or(props.curve);
        let mut dots: Vec<(f64, f64)> = Vec::with_capacity(samples.len());
        let mut bases: Vec<f64> = Vec::with_capacity(samples.len());
        for sample in &samples {
            let base = match (stacked_area, position) {
                (true, p) if p > 0 => tops[p - 1][sample.index],
                _ => 0.0,
            };
            dots.push((x_of(s, sample.index), map(sample.value)));
            bases.push(map(base));
        }
        if props.chart_type == ChartType::Area && series.fill_style != FillStyle::None {
            // The fill's own color at its opacity over the background, or
            // the stroke's (CHT-012).
            let fill_tint = series.fill.as_deref().and_then(color).or(tint);
            let base_line: Vec<(f64, f64)> =
                dots.iter().zip(&bases).map(|(d, b)| (d.0, *b)).collect();
            let mut covered: Vec<(usize, usize)> = Vec::new();
            target.fill_between(
                &dots,
                &base_line,
                curve,
                fill_tint,
                background,
                series.fill_opacity,
                &series.fill_style,
                Some((s, 0)),
                &mut covered,
                (shapes.y, shapes.bottom()),
            );
            if let FillStyle::Pattern(pattern) = &series.fill_style {
                // In a picture the marks are drawn inside the area.
                let clipped = target.clip_between(&dots, &base_line, curve);
                let tiles: Vec<&str> = pattern
                    .split("")
                    .filter(|g| {
                        !g.is_empty() && text_width(g) == 1 && !g.chars().any(char::is_control)
                    })
                    .collect();
                for (col, row) in covered {
                    let mark = match pattern.as_str() {
                        "diagonal" => {
                            if (col + row) % 3 == 0 {
                                "/"
                            } else {
                                " "
                            }
                        }
                        "dots" => {
                            if (col + row) % 2 == 0 {
                                glyphs.pattern_dot()
                            } else {
                                " "
                            }
                        }
                        // A custom tile outside ASCII is `#` in ASCII mode
                        // (CHT-028).
                        _ => tiles
                            .get((col + row) % tiles.len().max(1))
                            .copied()
                            .filter(|tile| !glyphs.ascii || tile.is_ascii())
                            .unwrap_or(if glyphs.ascii { "#" } else { "▒" }),
                    };
                    if !target.pattern_mark(col, row, mark, fill_tint) {
                        text.text(col, row, 1, mark, fill_tint);
                    }
                }
                if clipped {
                    target.end_clip();
                }
            }
        }
        if !scatter && series.line_style != LineStyle::None {
            target.polyline(&dots, curve, tint, Some((s, 0)), &series.line_style);
        }
        let series_dots = series.dots.unwrap_or(props.dots.unwrap_or(false));
        let marker = match props.chart_type {
            ChartType::Scatter => Some(Marker::Dot),
            _ if series_dots && samples.len() == values.len() => Some(Marker::Disc),
            _ => None,
        };
        for (sample, dot) in samples.iter().zip(&dots) {
            let owner = Some((s, sample.index));
            let cell = cell_of(dot.0, dot.1);
            if !inner.contains(cell.0, cell.1) {
                continue;
            }
            picture.anchors.insert((s, sample.index), cell);
            if let Some(marker) = marker {
                target.marker(
                    dot.0,
                    dot.1,
                    marker,
                    point_color(props, s, sample.index),
                    owner,
                );
            }
        }
        // A scatter's thinned points can still be hovered (CHT-027): every
        // original point keeps an anchor at its own cell.
        if scatter && samples.len() < values.len() {
            for (index, value) in values.iter().enumerate() {
                if !value.is_finite() || !index_visible(index) {
                    continue;
                }
                let cell = cell_of(x_of(s, index), map(*value));
                if inner.contains(cell.0, cell.1) {
                    picture.anchors.entry((s, index)).or_insert(cell);
                }
            }
        }
    }
    if let Some(h) = hover {
        // The hover in the picture (CHT-038): a scatter rings its selected
        // point; a line or area chart draws the crosshair through the
        // hovered index with a dot in a halo on every series there.
        if scatter {
            if let Some(value) = job.values.get(h.series).and_then(|v| v.get(h.index)) {
                target.ring(x_of(h.series, h.index), map(*value), color("ring"), h.focus);
            }
        } else {
            target.crosshair(
                category_center(h.index),
                inner,
                color("text-muted"),
                h.focus,
            );
            for (position, &s) in vis.iter().enumerate() {
                let Some(value) = job.values.get(s).and_then(|v| v.get(h.index)) else {
                    continue;
                };
                let value = if stacked_area {
                    tops.get(position)
                        .and_then(|t| t.get(h.index))
                        .copied()
                        .unwrap_or(*value)
                } else {
                    *value
                };
                target.hover_dot(
                    x_of(s, h.index),
                    map(value),
                    point_color(props, s, h.index),
                    h.focus,
                );
            }
        }
    }
    Ok(())
}

/// Fit a label into a width for callers in this module.
#[allow(dead_code)]
fn fit(label: &str, width: usize) -> String {
    fit_label(label, width)
}

/// Keep the text sink trait and the point type in scope for this module's
/// path checks.
#[allow(dead_code)]
fn _sink(_: &dyn TextSink, _: &DataPoint) {}
