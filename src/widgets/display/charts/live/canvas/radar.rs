//! Radar charts (CHT-016): one spoke per category at equal angles from
//! twelve o'clock, clockwise, and one polygon per series whose vertex on
//! each spoke lies at the category's value under a linear radial scale from
//! the plot layer, from zero to the largest value or `max_value`. Each
//! series' fill is a filled shape of the shared mask canvas (CHT-025) and
//! its outline a stroke. Series are drawn in order: before a series fills,
//! the earlier series' strokes inside its polygon are cleared, so a later
//! series lies over an earlier one. The grid levels and spokes are drawn in
//! the text layer under the shapes, at the medium and large size classes.

use super::super::super::mask::{Marker, MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::{Rect, ScaleLinear, SizeClass, TextSink};
use super::{series_color, visible, Job, Picture, RadialHit, TextLayer};
use std::f64::consts::TAU;
use unicode_width::UnicodeWidthStr;

/// The widest a category label may be, in columns.
const LABEL_WIDTH: usize = 12;

pub(super) fn radar(
    mask: &mut MaskCanvas,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    area: Rect,
    class: SizeClass,
) {
    let props = job.props;
    let series = visible(props);
    let categories = series.iter().map(|(_, s)| s.data.len()).max().unwrap_or(0);
    if categories == 0 {
        text.text(area.x, area.y, area.w, "No data to display", None);
        return;
    }
    let labels: Vec<String> = (0..categories)
        .map(|k| {
            series
                .iter()
                .find_map(|(_, s)| s.data.get(k).and_then(|p| p.label.clone()))
                .unwrap_or_else(|| k.to_string())
        })
        .collect();
    let labelled = class != SizeClass::Mini;
    let (margin_x, margin_y) = if labelled {
        let widest = labels
            .iter()
            .map(|l| UnicodeWidthStr::width(l.as_str()))
            .max()
            .unwrap_or(0)
            .min(LABEL_WIDTH);
        ((widest + 1).min(area.w / 4), 1.min(area.h / 4))
    } else {
        (0, 0)
    };
    // One dot short of the fit, so a vertex on the bottom or right edge
    // stays in the last row or column and its label fits beyond it.
    let fit = (((area.h.saturating_sub(2 * margin_y)) * DOTS_Y) as f64 / 2.0)
        .min(((area.w.saturating_sub(2 * margin_x)) * DOTS_X) as f64 / 2.0)
        - 1.0;
    let radius = fit * props.radial.outer_radius.clamp(0.0, 1.0);
    if radius <= 0.0 {
        return;
    }
    picture.plot = area;
    mask.set_clip_cells(area.x, area.y, area.w, area.h);
    let cx = (area.x * DOTS_X) as f64 + (area.w * DOTS_X) as f64 / 2.0;
    let cy = (area.y * DOTS_Y) as f64 + (area.h * DOTS_Y) as f64 / 2.0;
    let largest = series
        .iter()
        .flat_map(|(s, data)| {
            (0..data.data.len()).map(|k| {
                job.values
                    .get(*s)
                    .and_then(|v| v.get(k))
                    .copied()
                    .unwrap_or(0.0)
            })
        })
        .fold(0.0_f64, f64::max);
    let max = props
        .radial
        .max_value
        .filter(|m| m.is_finite() && *m > 0.0)
        .unwrap_or(if largest > 0.0 { largest } else { 1.0 });
    // The radial scale comes from the plot layer (CHT-010), and the reveal
    // grows the polygons from the center.
    let scale = ScaleLinear::new((0.0, max), (0.0, radius * job.progress.clamp(0.0, 1.0)));
    let spokes: Vec<f64> = (0..categories)
        .map(|k| k as f64 * TAU / categories as f64)
        .collect();
    let at = |angle: f64, r: f64| (cx + angle.sin() * r, cy - angle.cos() * r);
    if labelled && props.radial.grid {
        let levels = props.radial.grid_levels.max(1);
        for level in 1..=levels {
            let r = radius * level as f64 / levels as f64;
            for k in 0..categories {
                grid_segment(text, at(spokes[k], r), at(spokes[(k + 1) % categories], r));
            }
        }
        for angle in &spokes {
            grid_segment(text, (cx, cy), at(*angle, radius));
        }
    }
    for (order, (s, data)) in series.iter().enumerate() {
        let vertices: Vec<(f64, f64)> = (0..categories)
            .map(|k| {
                let value = if k < data.data.len() {
                    job.values
                        .get(*s)
                        .and_then(|v| v.get(k))
                        .copied()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                at(spokes[k], scale.map(value.clamp(0.0, max)))
            })
            .collect();
        let color = series_color(props, *s);
        let fill = match props.radial.fills.get(*s).and_then(|f| f.as_deref()) {
            Some("none") => None,
            Some(token) => Some(super::color(token).or(color)),
            None => Some(color),
        };
        if order > 0 {
            mask.clear_dots_inside(&vertices);
        }
        if let Some(fill) = fill {
            mask.fill_polygon(&vertices, fill, Some((*s, 0)));
        }
        let mut outline = vertices.clone();
        outline.push(vertices[0]);
        mask.polyline(&outline, color, Some((*s, 0)));
        for (k, vertex) in vertices.iter().enumerate() {
            if k < data.data.len() {
                if props.dots {
                    mask.marker(vertex.0, vertex.1, Marker::Dot, color, Some((*s, k)));
                }
                picture.anchors.insert(
                    (*s, k),
                    (
                        (vertex.0 / DOTS_X as f64) as usize,
                        (vertex.1 / DOTS_Y as f64) as usize,
                    ),
                );
            }
        }
    }
    if labelled {
        for (k, label) in labels.iter().enumerate() {
            let (x, y) = at(spokes[k], radius + DOTS_Y as f64);
            let (col, row) = ((x / DOTS_X as f64) as usize, (y / DOTS_Y as f64) as usize);
            let width = UnicodeWidthStr::width(label.as_str()).min(LABEL_WIDTH);
            let sin = spokes[k].sin();
            let start = if sin > 0.2 {
                col
            } else if sin < -0.2 {
                (col + 1).saturating_sub(width)
            } else {
                col.saturating_sub(width / 2)
            };
            let start = start.clamp(area.x, (area.x + area.w).saturating_sub(width));
            if (area.y..area.y + area.h).contains(&row) {
                text.text(start, row, width, label, None);
            }
        }
    }
    picture.radial = Some(RadialHit {
        center: (cx, cy),
        inner: 0.0,
        outer: radius,
        slices: Vec::new(),
        spokes,
    });
}

/// Mark the cells a grid segment (dot units) crosses with the grid glyph,
/// under the shapes.
fn grid_segment(text: &mut TextLayer, a: (f64, f64), b: (f64, f64)) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 2.0)
        .ceil()
        .max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        if x >= 0.0 && y >= 0.0 {
            text.under(
                (x / DOTS_X as f64) as usize,
                (y / DOTS_Y as f64) as usize,
                "·",
                None,
            );
        }
    }
}
