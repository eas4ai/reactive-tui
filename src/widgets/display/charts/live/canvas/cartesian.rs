//! Bar, line, area, scatter and candlestick rendering through the plot layer
//! (scales, ticks, axes, grid) and the shared mask canvas. Every cell or dot
//! position comes from a scale; nothing here maps a value itself.

use super::super::super::mask::{Marker, MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::{
    self, band_ticks, decimate_min_max, fit_label, format_tick, label_skip, labeled_ticks,
    linear_ticks, point_ticks, polyline, text_width, Axis, Grid, Rect, Rgba, ScaleBand,
    ScaleLinear, ScalePoint, SizeClass, TextSink,
};
use super::super::super::{BarGrowth, ChartAxis, ChartType, FillStyle, LineStyle};
use super::{color, point_color, tick_count, Job, Picture, TextLayer};

const BULLISH: &str = "chart-bullish";
const BEARISH: &str = "chart-bearish";

/// Value-axis domain from the target data, honoring explicit limits.
fn value_domain(axis: &ChartAxis, job: &Job, candles: bool) -> Result<(f64, f64), &'static str> {
    let props = job.props;
    let values = props.series.iter().filter(|s| s.visible).flat_map(|s| {
        s.data.iter().flat_map(move |p| match (candles, p.candle) {
            (true, Some(c)) => vec![c.low, c.high],
            _ => vec![p.value],
        })
    });
    let auto = if candles {
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        for v in values {
            low = low.min(v);
            high = high.max(v);
        }
        if low > high {
            (0.0, 1.0)
        } else if low == high {
            (low - 1.0, high + 1.0)
        } else {
            (low, high)
        }
    } else {
        ScaleLinear::domain_including_zero(values)
    };
    let mut low = axis.min.unwrap_or(auto.0);
    let mut high = axis.max.unwrap_or(auto.1);
    if low == high {
        let pad = low.abs().max(1.0) * 0.1;
        if axis.min.is_none() {
            low -= pad;
        }
        if axis.max.is_none() {
            high += pad;
        }
    }
    if low >= high || !(high - low).is_finite() {
        return Err("Axis range must have a finite positive span");
    }
    Ok((low, high))
}

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

pub(super) fn cartesian(
    mask: &mut MaskCanvas,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    mut area: Rect,
    class: SizeClass,
) -> Result<(), &'static str> {
    let props = job.props;
    let vis: Vec<usize> = props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .map(|(i, _)| i)
        .collect();
    let count = vis
        .iter()
        .map(|s| props.series[*s].data.len())
        .max()
        .unwrap_or(0);
    let bars = matches!(
        props.chart_type,
        ChartType::BarVertical | ChartType::BarHorizontal
    );
    let candles = props.chart_type == ChartType::Candlestick;
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
    let domain = value_domain(value_axis, job, candles)?;
    let axes = class.has_axes();
    if let Some(title) = &props.y_axis.title {
        if axes {
            let strip = area.take_top(1);
            text.text(strip.x, strip.y, strip.w, title, None);
        }
    }
    // Tentative plot rectangle to measure ticks against.
    let value_cells = if horizontal { area.w } else { area.h };
    let value_ticks = |scale: &ScaleLinear| {
        if value_axis.custom_labels.is_empty() {
            linear_ticks(scale, tick_count(value_axis, value_cells))
        } else {
            labeled_ticks(scale, &value_axis.custom_labels)
        }
    };
    let mut label_width = 0;
    let mut x_rows = 0;
    if axes {
        if !horizontal && value_axis.show_labels && value_axis.tick_count > 0 {
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
            label_width = (0..count)
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
        if bottom_axis.show_labels && (horizontal || count > 0) {
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
    mask.set_clip_cells(inner.x, inner.y, inner.w, inner.h);
    picture.plot = plot;
    picture.scatter = props.chart_type == ChartType::Scatter;

    // Scales in dot units.
    let x_dots = ((inner.x * DOTS_X) as f64, (inner.right() * DOTS_X) as f64);
    let y_dots = ((inner.y * DOTS_Y) as f64, (inner.bottom() * DOTS_Y) as f64);
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
    let band = ScaleBand::new(count, category_range)
        .padding_inner(if vis.len() > 1 && !props.stacked {
            0.3
        } else {
            0.4
        })
        .padding_outer(if class == SizeClass::Mini { 0.0 } else { 0.2 });
    let points = ScalePoint::new(
        count,
        (
            category_range.0,
            (category_range.1 - 1.0).max(category_range.0),
        ),
    );
    let use_band = bars || candles;
    let category_center = |i: usize| -> f64 {
        if use_band {
            band.center(i)
        } else {
            points.map(i)
        }
    };
    if horizontal {
        picture.index_rows = (0..count)
            .map(|i| category_center(i) / DOTS_Y as f64)
            .collect();
    } else {
        picture.index_columns = (0..count)
            .map(|i| category_center(i) / DOTS_X as f64)
            .collect();
    }

    // Grid, axes and labels.
    if axes {
        let ticks = value_ticks(&value_scale);
        let stride = if props.tick_margin > 0 {
            props.tick_margin
        } else {
            1
        };
        // Grid lines belong to the large class; medium draws axes and ticks
        // only (CHT-024). An axis can still turn its grid off at large.
        let show_grid = |axis: &ChartAxis| axis.show_grid && class.has_grid();
        let category_ticks: Vec<plot::Tick> = {
            let labels: Vec<Option<String>> = (0..count)
                .map(|i| category_label(job, category_axis, i))
                .collect();
            if use_band {
                band_ticks(&band, &labels)
            } else {
                point_ticks(&points, &labels)
            }
        };
        if horizontal {
            let mut vertical = Axis::vertical(
                category_ticks
                    .iter()
                    .map(|t| plot::Tick {
                        value: t.value,
                        position: t.position / DOTS_Y as f64 - plot.y as f64,
                        label: t.label.clone(),
                    })
                    .collect(),
            );
            vertical.line = category_axis.show_labels;
            let horizontal_axis = Axis::horizontal(
                ticks
                    .iter()
                    .map(|t| plot::Tick {
                        value: t.value,
                        position: t.position / DOTS_X as f64 - plot.x as f64,
                        label: t.label.clone(),
                    })
                    .collect(),
            )
            .with_title(props.x_axis.title.clone());
            let skip = if props.tick_margin > 0 {
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
                let columns = horizontal_axis
                    .shown()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position.round() as usize))
                    .collect();
                Grid::new(columns, Vec::new()).draw(text, plot, "·", None);
            }
            if show_grid(category_axis) {
                let rows = vertical
                    .ticks
                    .iter()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position.round() as usize))
                    .collect();
                Grid::new(Vec::new(), rows).draw(text, plot, "·", None);
            }
            if category_axis.show_labels {
                vertical.draw(text, label_rect, plot, None);
            }
            if value_axis.show_labels || props.x_axis.title.is_some() {
                horizontal_axis.draw(text, x_rect, plot, None);
            }
        } else {
            let vertical = Axis::vertical(
                ticks
                    .iter()
                    .map(|t| plot::Tick {
                        value: t.value,
                        position: t.position / DOTS_Y as f64 - plot.y as f64,
                        label: t.label.clone(),
                    })
                    .collect(),
            )
            .with_skip(stride);
            let horizontal_axis = Axis::horizontal(
                category_ticks
                    .iter()
                    .map(|t| plot::Tick {
                        value: t.value,
                        position: t.position / DOTS_X as f64 - plot.x as f64,
                        label: t.label.clone(),
                    })
                    .collect(),
            )
            .with_title(props.x_axis.title.clone());
            let skip = if props.tick_margin > 0 {
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
                let rows = vertical
                    .shown()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position.round() as usize))
                    .collect();
                Grid::new(Vec::new(), rows).draw(text, plot, "·", None);
            }
            if show_grid(category_axis) {
                let columns = horizontal_axis
                    .shown()
                    .filter_map(|t| (t.position >= 0.0).then_some(t.position.round() as usize))
                    .collect();
                Grid::new(columns, Vec::new()).draw(text, plot, "·", None);
            }
            if value_axis.show_labels {
                vertical.draw(text, label_rect, plot, None);
            }
            if category_axis.show_labels || props.x_axis.title.is_some() {
                horizontal_axis.draw(text, x_rect, plot, None);
            }
        }
    }

    let value_labels = props
        .value_labels
        .unwrap_or_else(|| class.has_value_labels());
    // Explicit limits on the category axis clip by index.
    let index_visible = |i: usize| {
        category_axis.min.is_none_or(|m| i as f64 >= m)
            && category_axis.max.is_none_or(|m| i as f64 <= m)
    };
    let cell_of = |x: f64, y: f64| -> (usize, usize) {
        (
            (x / DOTS_X as f64).floor().max(0.0) as usize,
            (y / DOTS_Y as f64).floor().max(0.0) as usize,
        )
    };

    if bars {
        let mut stack_pos = vec![0.0f64; count];
        let mut stack_neg = vec![0.0f64; count];
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
                    let (a, b) = band.band(i);
                    let (a, b) = if props.stacked || vis.len() < 2 {
                        (a, b)
                    } else {
                        let lanes = ScaleBand::new(vis.len(), (a, b)).padding_inner(0.15);
                        lanes.band(slot)
                    };
                    // Bar sides sit on cell boundaries so only the tip is
                    // fractional; a lane always keeps at least one cell.
                    let unit = if horizontal { DOTS_Y } else { DOTS_X } as f64;
                    let start = (a / unit).round() * unit;
                    let end = ((b / unit).round() * unit).max(start + unit);
                    (start, end)
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
                let (from, to) = (value_scale.map(base), value_scale.map(top));
                if (from - to).abs() < 1e-9 {
                    continue;
                }
                if horizontal {
                    mask.rect(from, lane_start, to, lane_end, tint, Some((s, i)));
                } else {
                    mask.rect(lane_start, from, lane_end, to, tint, Some((s, i)));
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
                    // the formatted value (CHT-020, CHT-013).
                    let label = point
                        .metadata
                        .get("label")
                        .cloned()
                        .unwrap_or_else(|| format_tick(point.value));
                    let width = text_width(&label);
                    if horizontal {
                        let x = if growth == BarGrowth::Left {
                            tip.0 + 1
                        } else {
                            tip.0.saturating_sub(width)
                        };
                        text.text(x, tip.1, width, &label, tint);
                    } else {
                        let outward = if (growth == BarGrowth::Bottom) == (value >= 0.0) {
                            tip.1.checked_sub(1)
                        } else {
                            Some(tip.1 + 1)
                        };
                        // A bar whose tip is the plot's edge row keeps its
                        // label, drawn on the tip row itself (CHT-013).
                        let y = outward
                            .filter(|y| plot.contains(tip.0, *y))
                            .unwrap_or(tip.1);
                        let x = (tip.0 + 1).saturating_sub(width.div_ceil(2)).max(plot.x);
                        text.text(x, y, width, &label, tint);
                    }
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
            for (i, point) in series.data.iter().enumerate() {
                if !index_visible(i) {
                    continue;
                }
                let Some(candle) = point.candle else { continue };
                let tint = point
                    .color
                    .as_deref()
                    .or(series.color.as_deref())
                    .and_then(color)
                    .or(if candle.is_bullish() {
                        bullish
                    } else {
                        bearish
                    });
                let (a, b) = band.band(i);
                // Bodies sit on cell boundaries like bars; the wick is one
                // dot wide at the band's centre.
                let unit = DOTS_X as f64;
                let a = (a / unit).round() * unit;
                let b = ((b / unit).round() * unit).max(a + unit);
                let center = (a + b) / 2.0;
                let (wick_top, wick_bottom) =
                    (value_scale.map(candle.high), value_scale.map(candle.low));
                mask.rect(
                    center - 0.5,
                    wick_top,
                    center + 0.5,
                    wick_bottom,
                    tint,
                    Some((s, i)),
                );
                let (mut open, mut close) =
                    (value_scale.map(candle.open), value_scale.map(candle.close));
                if (open - close).abs() < 0.5 {
                    open -= 0.25;
                    close += 0.25;
                }
                mask.rect(a, open, b, close, tint, Some((s, i)));
                picture
                    .anchors
                    .insert((s, i), cell_of(center, open.min(close)));
            }
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
        let mut acc = vec![0.0f64; count];
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
    for (position, &s) in vis.iter().enumerate() {
        let series = &props.series[s];
        let Some(values) = job.values.get(s) else {
            continue;
        };
        let stacked_values = if stacked_area { &tops[position] } else { values };
        let samples: Vec<_> = decimate_min_max(stacked_values, inner.w)
            .into_iter()
            .filter(|k| index_visible(k.index))
            .collect();
        picture.kept[s] = samples.iter().map(|k| k.index).collect();
        let tint = point_color(props, s, 0);
        let mut dots: Vec<(f64, f64)> = Vec::with_capacity(samples.len());
        let mut bases: Vec<f64> = Vec::with_capacity(samples.len());
        for sample in &samples {
            let base = match (stacked_area, position) {
                (true, p) if p > 0 => tops[p - 1][sample.index],
                _ => 0.0,
            };
            let x = points.map(sample.index);
            dots.push((x, value_scale.map(sample.value)));
            bases.push(value_scale.map(base));
        }
        if props.chart_type == ChartType::Area && series.fill_style != FillStyle::None {
            let dense = polyline(&dots, props.curve, 1.0);
            let base_line: Vec<(f64, f64)> =
                dots.iter().zip(&bases).map(|(d, b)| (d.0, *b)).collect();
            let dense_base = polyline(&base_line, props.curve, 1.0);
            let mut covered: Vec<(usize, usize)> = Vec::new();
            fill_between(
                mask,
                &dense,
                &dense_base,
                tint,
                &series.fill_style,
                Some((s, 0)),
                &mut covered,
                value_scale.range(),
            );
            if let FillStyle::Pattern(pattern) = &series.fill_style {
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
                                "·"
                            } else {
                                " "
                            }
                        }
                        _ => tiles
                            .get((col + row) % tiles.len().max(1))
                            .copied()
                            .unwrap_or("▒"),
                    };
                    text.text(col, row, 1, mark, tint);
                }
            }
        }
        if props.chart_type != ChartType::Scatter && series.line_style != LineStyle::None {
            let dense = polyline(&dots, props.curve, 1.0);
            match series.line_style {
                LineStyle::Solid => mask.polyline(&dense, tint, Some((s, 0))),
                LineStyle::Dashed => {
                    for (k, pair) in dense.windows(2).enumerate() {
                        if k % 8 < 4 {
                            mask.line(pair[0], pair[1], tint, Some((s, 0)));
                        }
                    }
                }
                LineStyle::Dotted => {
                    for (k, p) in dense.iter().enumerate() {
                        if k % 3 == 0 {
                            mask.dot(p.0.round() as i64, p.1.round() as i64, tint, Some((s, 0)));
                        }
                    }
                }
                LineStyle::None => {}
            }
        }
        let marker = match props.chart_type {
            ChartType::Scatter => Some(Marker::Dot),
            _ if props.dots && samples.len() == values.len() => Some(Marker::Disc),
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
                mask.marker(
                    dot.0,
                    dot.1,
                    marker,
                    point_color(props, s, sample.index),
                    owner,
                );
            }
        }
    }
    Ok(())
}

/// Fill the dots between `top` and `bottom` polylines column by column,
/// shading toward the baseline for a gradient fill.
#[allow(clippy::too_many_arguments)]
fn fill_between(
    mask: &mut MaskCanvas,
    top: &[(f64, f64)],
    bottom: &[(f64, f64)],
    tint: Option<Rgba>,
    style: &FillStyle,
    owner: Option<(usize, usize)>,
    covered: &mut Vec<(usize, usize)>,
    range: (f64, f64),
) {
    if top.is_empty() || bottom.is_empty() {
        return;
    }
    let (x0, x1) = (top.first().unwrap().0, top.last().unwrap().0);
    let mut x = x0.round() as i64;
    let end = x1.round() as i64;
    let mut ti = 0;
    let mut bi = 0;
    let far = range.0.max(range.1);
    let near = range.0.min(range.1);
    while x <= end {
        let xf = x as f64;
        while ti + 1 < top.len() && top[ti + 1].0 < xf {
            ti += 1;
        }
        while bi + 1 < bottom.len() && bottom[bi + 1].0 < xf {
            bi += 1;
        }
        let y_top = interpolate_at(top, ti, xf);
        let y_bottom = interpolate_at(bottom, bi, xf);
        let (a, b) = (y_top.min(y_bottom), y_top.max(y_bottom));
        let mut y = a.round() as i64;
        let last = b.round() as i64;
        while y <= last {
            let shade = match style {
                FillStyle::Gradient => {
                    let extent = (far - near).max(1.0);
                    let t = ((y as f64 - near) / extent).clamp(0.0, 1.0);
                    let k = (0.45 + 0.55 * t) as f32;
                    tint.map(|(r, g, bl, al)| (r * k, g * k, bl * k, al))
                }
                _ => tint,
            };
            mask.dot(x, y, shade, owner);
            let cell = ((x.max(0) as usize) / DOTS_X, (y.max(0) as usize) / DOTS_Y);
            if covered.last() != Some(&cell) {
                covered.push(cell);
            }
            y += 1;
        }
        x += 1;
    }
    covered.sort_unstable();
    covered.dedup();
}

fn interpolate_at(line: &[(f64, f64)], i: usize, x: f64) -> f64 {
    let a = line[i];
    let Some(b) = line.get(i + 1) else { return a.1 };
    let dx = b.0 - a.0;
    if dx.abs() < 1e-9 {
        return b.1;
    }
    let t = ((x - a.0) / dx).clamp(0.0, 1.0);
    a.1 + t * (b.1 - a.1)
}

/// Fit a label into a width for callers in this module.
#[allow(dead_code)]
fn fit(label: &str, width: usize) -> String {
    fit_label(label, width)
}

/// Keep the text sink trait in scope for `text.text` calls above.
#[allow(dead_code)]
fn _sink(_: &dyn TextSink) {}
