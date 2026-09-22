//! Pie and donut slices drawn as sectors into the shared mask canvas. Angles
//! come from the values' share of the total through an ordinal position on
//! the circle; the mask tests each dot's polar coordinates in cell-square
//! space, so a full circle is as wide as it is tall on screen.

use super::super::super::mask::{MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::{Rect, TextSink};
use super::super::super::ChartType;
use super::{Job, Picture, TextLayer};

pub(super) fn pie(
    mask: &mut MaskCanvas,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    area: Rect,
) {
    let props = job.props;
    let mut slices: Vec<(usize, usize, f64)> = Vec::new();
    for (s, series) in props.series.iter().enumerate().filter(|(_, s)| s.visible) {
        for (i, _) in series.data.iter().enumerate() {
            let value = job
                .values
                .get(s)
                .and_then(|v| v.get(i))
                .copied()
                .unwrap_or(0.0);
            if value > 0.0 {
                slices.push((s, i, value));
            }
        }
    }
    let total: f64 = slices.iter().map(|(_, _, v)| *v).sum();
    if total <= 0.0 {
        text.text(area.x, area.y, area.w, "", None);
        return;
    }
    picture.plot = area;
    mask.set_clip_cells(area.x, area.y, area.w, area.h);
    // Radius in dot rows; a cell is twice as tall as wide, so the horizontal
    // reach in dot columns is half the vertical reach in dot rows.
    let aspect = DOTS_Y as f64 / DOTS_X as f64;
    let radius = ((area.h * DOTS_Y) as f64 / 2.0).min((area.w * DOTS_X) as f64 * aspect / 2.0);
    if radius <= 0.0 {
        return;
    }
    let cx = (area.x * DOTS_X) as f64 + (area.w * DOTS_X) as f64 / 2.0;
    let cy = (area.y * DOTS_Y) as f64 + (area.h * DOTS_Y) as f64 / 2.0;
    let inner = if props.chart_type == ChartType::Donut {
        radius * 0.5
    } else {
        0.0
    };
    let sweep = std::f64::consts::TAU * job.progress.clamp(0.0, 1.0);
    let mut start = 0.0;
    for (index, (s, i, value)) in slices.iter().enumerate() {
        let end = start + sweep * value / total;
        // A slice takes its point's color, then its series color, then the
        // palette entry for the slice (not the series) so slices differ.
        let series = &props.series[*s];
        let tint = series.data[*i]
            .color
            .as_deref()
            .or(series.color.as_deref())
            .or_else(|| {
                props
                    .color_palette
                    .get(index % props.color_palette.len().max(1))
                    .map(String::as_str)
            })
            .and_then(super::color);
        mask.sector(cx, cy, inner, radius, start, end, tint, Some((*s, *i)));
        let mid = (start + end) / 2.0;
        let r = (inner + radius) / 2.0;
        let ax = cx + mid.sin() * r / aspect;
        let ay = cy - mid.cos() * r;
        picture.anchors.insert(
            (*s, *i),
            ((ax / DOTS_X as f64) as usize, (ay / DOTS_Y as f64) as usize),
        );
        start = end;
    }
}
