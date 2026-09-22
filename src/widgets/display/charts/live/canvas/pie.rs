use super::*;

pub(super) fn pie(canvas: &mut Canvas, props: &ChartProps, area: Rect, progress: f64) {
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
                .filter(|(_, p)| p.value > 0.0)
                .map(move |(p, v)| (s, p, v.value))
        })
        .collect();
    let scale = points.iter().map(|(_, _, v)| *v).fold(0.0, f64::max);
    if scale == 0.0 {
        canvas.text(area.x, area.y, area.w, "Nothing here", None);
        return;
    }
    let total: f64 = points.iter().map(|(_, _, v)| v / scale).sum();
    let mut cumulative = 0.0;
    let sectors: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(index, &(series, point, value))| {
            cumulative += (value / scale) / total;
            let data = &props.series[series];
            let tint = data.data[point]
                .color
                .as_deref()
                .or(data.color.as_deref())
                .or_else(|| {
                    props
                        .color_palette
                        .get(index % props.color_palette.len().max(1))
                        .map(String::as_str)
                })
                .and_then(color);
            (cumulative, series, point, tint)
        })
        .collect();
    let radius = (area.w as f64 / 2.0).min(area.h as f64);
    if radius <= 0.0 {
        return;
    }
    for y in 0..area.h {
        for x in 0..area.w {
            let dx = (x as f64 + 0.5 - area.w as f64 / 2.0) / radius;
            let dy = (y as f64 + 0.5 - area.h as f64 / 2.0) * 2.0 / radius;
            let distance = dx * dx + dy * dy;
            if distance > 1.0 || (props.chart_type == ChartType::Donut && distance < 0.25) {
                continue;
            }
            let fraction = (dy.atan2(dx) + std::f64::consts::FRAC_PI_2)
                .rem_euclid(std::f64::consts::TAU)
                / std::f64::consts::TAU;
            if fraction >= progress {
                continue;
            }
            let index = sectors
                .partition_point(|(end, _, _, _)| *end < fraction)
                .min(sectors.len() - 1);
            let (_, series, point, tint) = sectors[index];
            canvas.put(area.x + x, area.y + y, "█", tint, Some((series, point)));
        }
    }
}
