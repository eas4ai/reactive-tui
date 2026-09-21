use super::*;

fn range(axis: &ChartAxis, min: f64, max: f64) -> Result<(f64, f64), &'static str> {
    let mut low = axis.min.unwrap_or(min);
    let mut high = axis.max.unwrap_or(max);
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
fn mapped(value: f64, limits: (f64, f64), cells: usize) -> usize {
    (((value - limits.0) / (limits.1 - limits.0)).clamp(0.0, 1.0) * cells.saturating_sub(1) as f64)
        .round() as usize
}
fn tick(axis: &ChartAxis, index: usize, value: f64) -> String {
    axis.custom_labels.get(index).cloned().unwrap_or_else(|| {
        if value.abs() >= 1e6 || (value != 0.0 && value.abs() < 0.001) {
            format!("{value:.1e}")
        } else {
            format!("{value:.1}")
        }
    })
}

pub(super) fn cartesian(
    canvas: &mut Canvas,
    props: &ChartProps,
    mut area: Rect,
    progress: f64,
) -> Result<(), &'static str> {
    let points: Vec<_> = props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .flat_map(|(s, series)| {
            series
                .data
                .iter()
                .enumerate()
                .map(move |(p, v)| (s, p, v.value))
        })
        .collect();
    let low = points.iter().map(|(_, _, v)| *v).fold(0.0, f64::min);
    let high = points.iter().map(|(_, _, v)| *v).fold(0.0, f64::max);
    let count = props
        .series
        .iter()
        .filter(|s| s.visible)
        .map(|s| s.data.len())
        .max()
        .unwrap_or(1);
    let horizontal = props.chart_type == ChartType::BarHorizontal;
    let (xr, yr) = if horizontal {
        (
            range(&props.x_axis, low, high)?,
            range(
                &props.y_axis,
                0.0,
                points.len().saturating_sub(1).max(1) as f64,
            )?,
        )
    } else {
        (
            range(&props.x_axis, 0.0, count.saturating_sub(1).max(1) as f64)?,
            range(&props.y_axis, low, high)?,
        )
    };
    if let Some(title) = &props.y_axis.title {
        canvas.text(area.x, area.y, area.w, title, None);
        area.y += 1;
        area.h = area.h.saturating_sub(1);
    }
    if let Some(title) = &props.x_axis.title {
        if area.h > 0 {
            canvas.text(area.x, area.y + area.h - 1, area.w, title, None);
            area.h -= 1;
        }
    }
    let mut label_width = if props.y_axis.show_labels && props.y_axis.tick_count > 0 {
        (0..props.y_axis.tick_count.min(area.h))
            .map(|i| {
                let fraction = i as f64 / props.y_axis.tick_count.saturating_sub(1).max(1) as f64;
                UnicodeWidthStr::width(
                    tick(&props.y_axis, i, yr.1 - (yr.1 - yr.0) * fraction).as_str(),
                ) + 1
            })
            .max()
            .unwrap_or(0)
            .min(area.w / 3)
    } else {
        0
    };
    if horizontal && props.y_axis.show_labels && props.y_axis.tick_count > 0 {
        label_width = label_width.max(
            points
                .iter()
                .map(|(s, p, _)| {
                    UnicodeWidthStr::width(
                        props.series[*s].data[*p]
                            .label
                            .as_deref()
                            .unwrap_or(&props.series[*s].name),
                    ) + 1
                })
                .max()
                .unwrap_or(0)
                .min(area.w / 3),
        );
    }
    let xlabels = usize::from(props.x_axis.show_labels && props.x_axis.tick_count > 0);
    let plot = Rect {
        x: area.x + label_width,
        y: area.y,
        w: area.w.saturating_sub(label_width),
        h: area.h.saturating_sub(xlabels),
    };
    if plot.w == 0 || plot.h == 0 {
        return Ok(());
    }
    let yticks =
        props
            .y_axis
            .tick_count
            .min(plot.h)
            .min(if horizontal { points.len() } else { usize::MAX });
    for i in 0..yticks {
        let y = i * plot.h.saturating_sub(1) / yticks.saturating_sub(1).max(1);
        let fraction = i as f64 / yticks.saturating_sub(1).max(1) as f64;
        let value = if horizontal {
            yr.0 + (yr.1 - yr.0) * fraction
        } else {
            yr.1 - (yr.1 - yr.0) * fraction
        };
        let label = if horizontal
            && props.y_axis.custom_labels.is_empty()
            && value.fract().abs() < f64::EPSILON
        {
            points
                .get(value as usize)
                .map(|(s, p, _)| {
                    props.series[*s].data[*p]
                        .label
                        .as_deref()
                        .unwrap_or(&props.series[*s].name)
                        .to_string()
                })
                .unwrap_or_else(|| tick(&props.y_axis, i, value))
        } else {
            tick(&props.y_axis, i, value)
        };
        if props.y_axis.show_labels {
            canvas.text(
                area.x,
                plot.y + y,
                label_width.saturating_sub(1),
                &label,
                None,
            );
        }
        if props.y_axis.show_grid {
            for x in 0..plot.w {
                canvas.put(plot.x + x, plot.y + y, "·", None, None);
            }
        }
    }
    let xticks = props.x_axis.tick_count.min(plot.w);
    let mut label_end = 0;
    for i in 0..xticks {
        let x = i * plot.w.saturating_sub(1) / xticks.saturating_sub(1).max(1);
        if props.x_axis.show_grid {
            for y in 0..plot.h {
                canvas.put(plot.x + x, plot.y + y, "·", None, None);
            }
        }
        if props.x_axis.show_labels {
            let value = xr.0 + (xr.1 - xr.0) * i as f64 / xticks.saturating_sub(1).max(1) as f64;
            let label = if !horizontal
                && props.x_axis.custom_labels.is_empty()
                && value.fract().abs() < f64::EPSILON
            {
                props
                    .series
                    .iter()
                    .filter(|s| s.visible)
                    .find_map(|s| s.data.get(value as usize).and_then(|p| p.label.clone()))
                    .unwrap_or_else(|| tick(&props.x_axis, i, value))
            } else {
                tick(&props.x_axis, i, value)
            };
            let start = x.min(
                plot.w
                    .saturating_sub(UnicodeWidthStr::width(label.as_str())),
            );
            if start >= label_end {
                canvas.text(
                    plot.x + start,
                    plot.y + plot.h,
                    plot.w - start,
                    &label,
                    None,
                );
                label_end = start + UnicodeWidthStr::width(label.as_str()) + 1;
            }
        }
    }
    let visible: Vec<_> = props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
        .map(|(i, _)| i)
        .collect();
    if matches!(props.chart_type, ChartType::Line | ChartType::Area) {
        lines(canvas, props, plot, xr, yr, progress);
        return Ok(());
    }
    for (ordinal, &(s, p, value)) in points.iter().enumerate() {
        let tint = point_color(props, s, p);
        let point = Some((s, p));
        if horizontal {
            let category = ordinal as f64;
            if category < yr.0 || category > yr.1 {
                continue;
            }
            if value == 0.0
                || progress == 0.0
                || (value * progress).max(0.0) <= xr.0
                || (value * progress).min(0.0) >= xr.1
            {
                continue;
            }
            let y = mapped(category, yr, plot.h);
            let zero = mapped(0.0, xr, plot.w);
            let end = mapped(value * progress, xr, plot.w);
            for x in zero.min(end)..=zero.max(end) {
                canvas.put(plot.x + x, plot.y + y, "█", tint, point);
            }
        } else {
            if (p as f64) < xr.0 || (p as f64) > xr.1 {
                continue;
            }
            let x = mapped(p as f64, xr, plot.w);
            let y = plot.h - 1 - mapped(value * progress, yr, plot.h);
            let zero = plot.h - 1 - mapped(0.0, yr, plot.h);
            match props.chart_type {
                ChartType::BarVertical => {
                    if value == 0.0
                        || progress == 0.0
                        || (value * progress).max(0.0) <= yr.0
                        || (value * progress).min(0.0) >= yr.1
                    {
                        continue;
                    }
                    let group = ((plot.w as f64 / (xr.1 - xr.0 + 1.0)).floor() as usize).max(1);
                    let bar_width = (group / visible.len().max(1)).max(1);
                    let series = visible.iter().position(|index| *index == s).unwrap_or(0);
                    let start =
                        ((((p as f64 - xr.0) / (xr.1 - xr.0 + 1.0)) * plot.w as f64).floor()
                            as usize
                            + series * bar_width)
                            .min(plot.w - 1);
                    for dx in 0..bar_width.min(plot.w - start) {
                        for dy in zero.min(y)..=zero.max(y) {
                            canvas.put(plot.x + start + dx, plot.y + dy, "█", tint, point);
                        }
                    }
                }
                ChartType::Scatter if value * progress >= yr.0 && value * progress <= yr.1 => {
                    canvas.put(plot.x + x, plot.y + y, "•", tint, point);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
fn interpolate(a: f64, b: f64, fraction: f64) -> f64 {
    a * (1.0 - fraction) + b * fraction
}

// Clip in data space before converting to cell coordinates. Clamping each
// endpoint independently would invent edge points and alter crossing slopes.
fn clipped(
    a: (f64, f64),
    b: (f64, f64),
    xr: (f64, f64),
    yr: (f64, f64),
) -> Option<((f64, f64), (f64, f64))> {
    let mut begin: f64 = 0.0;
    let mut end: f64 = 1.0;
    for (first, last, (low, high)) in [(a.0, b.0, xr), (a.1, b.1, yr)] {
        if first.min(last) > high || first.max(last) < low {
            return None;
        }
        let fraction = |value: f64| {
            let scale = first.abs().max(last.abs()).max(value.abs()).max(1.0);
            (value / scale - first / scale) / (last / scale - first / scale)
        };
        if first < low {
            begin = begin.max(fraction(low));
        }
        if first > high {
            begin = begin.max(fraction(high));
        }
        if last < low {
            end = end.min(fraction(low));
        }
        if last > high {
            end = end.min(fraction(high));
        }
    }
    if begin > end {
        return None;
    }
    Some((
        (interpolate(a.0, b.0, begin), interpolate(a.1, b.1, begin)),
        (interpolate(a.0, b.0, end), interpolate(a.1, b.1, end)),
    ))
}

fn lines(
    canvas: &mut Canvas,
    props: &ChartProps,
    plot: Rect,
    xr: (f64, f64),
    yr: (f64, f64),
    progress: f64,
) {
    let zero = plot.h - 1 - mapped(0.0, yr, plot.h);
    for (series_index, series) in props.series.iter().enumerate().filter(|(_, s)| s.visible) {
        for (point_index, pair) in series.data.windows(2).enumerate() {
            let a = (point_index as f64, pair[0].value * progress);
            let b = ((point_index + 1) as f64, pair[1].value * progress);
            let tint = point_color(props, series_index, point_index + 1);
            let point = Some((series_index, point_index + 1));
            if props.chart_type == ChartType::Area {
                if let Some((a, b)) = clipped(a, b, xr, (f64::NEG_INFINITY, f64::INFINITY)) {
                    let (left, right) = (mapped(a.0, xr, plot.w), mapped(b.0, xr, plot.w));
                    for column in left..=right {
                        let t = (column - left) as f64 / right.saturating_sub(left).max(1) as f64;
                        let y = plot.h - 1 - mapped(interpolate(a.1, b.1, t), yr, plot.h);
                        fill(
                            canvas,
                            plot,
                            (column, y, zero),
                            &series.fill_style,
                            tint,
                            point,
                        );
                    }
                }
            }
            if let Some((a, b)) = clipped(a, b, xr, yr) {
                let (ax, ay) = (
                    mapped(a.0, xr, plot.w),
                    plot.h - 1 - mapped(a.1, yr, plot.h),
                );
                let (bx, by) = (
                    mapped(b.0, xr, plot.w),
                    plot.h - 1 - mapped(b.1, yr, plot.h),
                );
                let steps = ax.abs_diff(bx).max(ay.abs_diff(by)).max(1);
                for step in 0..=steps {
                    let t = step as f64 / steps as f64;
                    let x = interpolate(ax as f64, bx as f64, t).round() as usize;
                    let y = interpolate(ay as f64, by as f64, t).round() as usize;
                    let mark = match series.line_style {
                        LineStyle::Solid => Some("─"),
                        LineStyle::Dashed if step % 4 < 2 => Some("─"),
                        LineStyle::Dotted if step % 2 == 0 => Some("·"),
                        _ => None,
                    };
                    if let Some(mark) = mark {
                        canvas.put(plot.x + x, plot.y + y, mark, tint, point);
                    }
                }
            }
        }
    }
    // Markers are painted after connecting lines and fills so shared endpoints
    // remain visible and keep their own tooltip identity and color.
    for (s, series) in props.series.iter().enumerate().filter(|(_, s)| s.visible) {
        for (p, point) in series.data.iter().enumerate() {
            let value = point.value * progress;
            if (p as f64) < xr.0 || (p as f64) > xr.1 || value < yr.0 || value > yr.1 {
                continue;
            }
            let x = mapped(p as f64, xr, plot.w);
            let y = plot.h - 1 - mapped(value, yr, plot.h);
            if series.data.len() == 1 && props.chart_type == ChartType::Area {
                fill(
                    canvas,
                    plot,
                    (x, y, zero),
                    &series.fill_style,
                    point_color(props, s, p),
                    Some((s, p)),
                );
            }
            canvas.put(
                plot.x + x,
                plot.y + y,
                "●",
                point_color(props, s, p),
                Some((s, p)),
            );
        }
    }
}

fn fill(
    canvas: &mut Canvas,
    plot: Rect,
    (x, y, zero): (usize, usize, usize),
    style: &FillStyle,
    color: Option<Color>,
    point: Option<Point>,
) {
    if *style == FillStyle::None {
        return;
    }
    let tiles: Vec<_> = match style {
        FillStyle::Pattern(pattern) => pattern
            .graphemes(true)
            .filter(|s| UnicodeWidthStr::width(*s) == 1 && !s.chars().any(char::is_control))
            .take(plot.w + plot.h)
            .collect(),
        _ => Vec::new(),
    };
    for row in y.min(zero)..=y.max(zero) {
        let mark = match style {
            FillStyle::Gradient => {
                if row.abs_diff(zero) * 3 < y.abs_diff(zero) {
                    "░"
                } else if row.abs_diff(zero) * 3 < y.abs_diff(zero) * 2 {
                    "▒"
                } else {
                    "▓"
                }
            }
            FillStyle::Pattern(pattern) if pattern == "diagonal" => {
                if (x + row) % 3 == 0 {
                    "/"
                } else {
                    " "
                }
            }
            FillStyle::Pattern(pattern) if pattern == "dots" => {
                if (x + row) % 2 == 0 {
                    "·"
                } else {
                    " "
                }
            }
            FillStyle::Pattern(_) => tiles
                .get((x + row) % tiles.len().max(1))
                .copied()
                .unwrap_or("▒"),
            _ => "█",
        };
        canvas.put(plot.x + x, plot.y + row, mark, color, point);
    }
}
