//! Pie and donut slices drawn as filled sectors of the shared mask canvas
//! (CHT-015, CHT-025). Angles come from each value's share of the total
//! through a linear scale on the plot layer (CHT-010). A canvas dot is as
//! wide as it is tall on screen, so the circle needs no aspect factor: it
//! spans twice as many columns as rows. At the medium and large size classes
//! each slice's label sits beside the circle, joined to its slice by a leader
//! line in the text layer that ends beside a cell of its own slice and
//! crosses no slice, label or other leader; a label with no row where that
//! fits is left out, and its slice stays in the legend.

use super::super::super::mask::{MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::{
    fit_label, LegendEntry, Rect, Rgba, ScaleLinear, SizeClass, TextSink,
};
use super::super::super::{ChartProps, ChartType};
use super::{Job, Picture, RadialHit, RadialSlice, TextLayer};
use std::collections::HashSet;
use std::f64::consts::{PI, TAU};
use unicode_width::UnicodeWidthStr;

/// The widest a side label may be, in columns.
const LABEL_WIDTH: usize = 16;

/// A slice's label to place: its middle angle, its (series, point), its
/// text and its color.
type SliceLabel = (f64, (usize, usize), String, Option<Rgba>);

/// The visible slices with a positive value: (series, index, value).
pub(super) fn slices(job: &Job) -> Vec<(usize, usize, f64)> {
    let mut slices = Vec::new();
    for (s, series) in job
        .props
        .series
        .iter()
        .enumerate()
        .filter(|(_, s)| s.visible)
    {
        for i in 0..series.data.len() {
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
    slices
}

/// A slice's color: its point's color, then its series color, then the
/// palette entry for its place among the `count` slices, so slices differ.
/// When the palette wraps so the last slice would take the first slice's
/// color, and the two touch, the last one takes the next entry instead.
pub(super) fn slice_color(
    props: &ChartProps,
    order: usize,
    count: usize,
    s: usize,
    i: usize,
) -> Option<Rgba> {
    let series = &props.series[s];
    let colors = props.color_palette.len().max(1);
    let entry = if count > 1 && order + 1 == count && order.is_multiple_of(colors) && colors > 1 {
        1
    } else {
        order % colors
    };
    series.data[i]
        .color
        .as_deref()
        .or(series.color.as_deref())
        .or_else(|| props.color_palette.get(entry).map(String::as_str))
        .and_then(super::color)
}

/// A slice's label: its point's label, else its index.
pub(super) fn slice_label(props: &ChartProps, s: usize, i: usize) -> String {
    props.series[s].data[i]
        .label
        .clone()
        .unwrap_or_else(|| i.to_string())
}

/// The legend of a pie or donut: one entry per slice.
pub(super) fn legend_entries(job: &Job) -> Vec<LegendEntry> {
    let slices = slices(job);
    slices
        .iter()
        .enumerate()
        .map(|(order, (s, i, _))| LegendEntry {
            name: slice_label(job.props, *s, *i),
            color: slice_color(job.props, order, slices.len(), *s, *i),
        })
        .collect()
}

pub(super) fn pie(
    mask: &mut MaskCanvas,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    area: Rect,
    class: SizeClass,
) {
    let props = job.props;
    let slices = slices(job);
    let total: f64 = slices.iter().map(|(_, _, v)| *v).sum();
    if !total.is_finite() {
        text.text(
            area.x,
            area.y,
            area.w,
            "Pie values must have a finite total",
            None,
        );
        return;
    }
    if total <= 0.0 {
        text.text(area.x, area.y, area.w, "No data to display", None);
        return;
    }
    let labelled = class != SizeClass::Mini;
    // The large class shows each label in full with its value.
    let label_text = |s: usize, i: usize| {
        let label = slice_label(props, s, i);
        if class == SizeClass::Large {
            format!("{label} {}", props.series[s].data[i].value)
        } else {
            label
        }
    };
    let widest_allowed = if class == SizeClass::Large {
        usize::MAX
    } else {
        LABEL_WIDTH
    };
    let gap = usize::from(props.radial.label_gap);
    // Columns kept free on each side for labels and their leaders.
    let reserve = if labelled {
        let widest = slices
            .iter()
            .map(|(s, i, _)| UnicodeWidthStr::width(label_text(*s, *i).as_str()))
            .max()
            .unwrap_or(0)
            .min(widest_allowed);
        (widest + gap + 1).min(area.w / 4)
    } else {
        0
    };
    let circle_w = area.w.saturating_sub(2 * reserve);
    let fit = ((area.h * DOTS_Y) as f64 / 2.0).min((circle_w * DOTS_X) as f64 / 2.0);
    let outer = fit * props.radial.outer_radius.clamp(0.0, 1.0);
    if outer <= 0.0 {
        return;
    }
    let inner_fraction =
        props
            .radial
            .inner_radius
            .unwrap_or(if props.chart_type == ChartType::Donut {
                0.5
            } else {
                0.0
            });
    let inner = outer * inner_fraction.clamp(0.0, 0.95);
    picture.plot = area;
    mask.set_clip_cells(area.x, area.y, area.w, area.h);
    let cx = (area.x * DOTS_X) as f64 + (area.w * DOTS_X) as f64 / 2.0;
    let cy = (area.y * DOTS_Y) as f64 + (area.h * DOTS_Y) as f64 / 2.0;
    // Angles come from the plot layer: a linear scale from the cumulative
    // value share to the revealed sweep (CHT-010).
    let sweep = TAU * job.progress.clamp(0.0, 1.0);
    let angle = ScaleLinear::new((0.0, total), (0.0, sweep));
    let pad = props.radial.pad_angle.max(0.0);
    let mut hit = RadialHit {
        center: (cx, cy),
        inner,
        outer,
        samples: mask.fill_blitter().cell_pixels(),
        ..RadialHit::default()
    };
    let mut labels = Vec::new();
    let (mut start, mut cumulative) = (0.0, 0.0);
    for (order, (s, i, value)) in slices.iter().enumerate() {
        cumulative += value;
        let end = angle.map(cumulative);
        let (from, to) = if end - start > pad {
            (start + pad / 2.0, end - pad / 2.0)
        } else {
            ((start + end) / 2.0, (start + end) / 2.0)
        };
        let tint = slice_color(props, order, slices.len(), *s, *i);
        // A slice is drawn when its fill set a sample; the keys, the pointer
        // and the labels reach drawn slices alone (CHT-031), over the angles
        // the fill took.
        let drawn = mask.fill_sector(cx, cy, inner, outer, from, to, tint, Some((*s, *i))) > 0;
        if drawn {
            hit.slices.push(RadialSlice {
                start: from,
                end: to,
                key: (*s, *i),
                color: tint,
            });
        }
        let mid = (start + end) / 2.0;
        let r = (inner + outer) / 2.0;
        picture.anchors.insert(
            (*s, *i),
            (
                ((cx + mid.sin() * r) / DOTS_X as f64) as usize,
                ((cy - mid.cos() * r) / DOTS_Y as f64) as usize,
            ),
        );
        if labelled && drawn {
            labels.push((mid, (*s, *i), label_text(*s, *i), tint));
        }
        start = end;
    }
    picture.radial = Some(hit);
    if labelled {
        place_labels(
            text,
            mask,
            area,
            (cx, cy),
            outer,
            gap,
            widest_allowed,
            labels,
        );
    }
}

/// Place each slice's label beside the circle on the side its middle angle
/// faces, cut to the columns left there, and join it to the slice with a
/// leader line. The leader starts past the outermost painted cell of a row
/// on that side only when the slice alone paints that cell, so it ends
/// beside its own slice and never beside a neighbour's or a blend of two:
/// on the row of the slice's outer edge, else the nearest row where the
/// slice alone paints it. The label takes the nearest free row to that
/// start whose leader crosses no painted cell and no label or leader placed
/// before it. A label with no such rows is left out.
#[allow(clippy::too_many_arguments)]
fn place_labels(
    text: &mut TextLayer,
    mask: &MaskCanvas,
    area: Rect,
    (cx, cy): (f64, f64),
    outer: f64,
    gap: usize,
    widest_allowed: usize,
    labels: Vec<SliceLabel>,
) {
    let row_end = area.y + area.h;
    let col_end = area.x + area.w;
    let center_col = ((cx / DOTS_X as f64) as usize).clamp(area.x, col_end.saturating_sub(1));
    // The first column past the painted cells on `row`, outward from the
    // center on the label's side, when slice `key` alone paints the
    // outermost of them; `None` when a neighbour paints any of that cell or
    // the row is unpainted.
    let reach = |row: usize, right: bool, key: (usize, usize)| -> Option<usize> {
        let outermost = if right {
            (center_col..col_end)
                .rev()
                .find(|c| mask.is_painted(*c, row))
        } else {
            (area.x..=center_col).find(|c| mask.is_painted(*c, row))
        }?;
        if mask.sole_fill_owner(outermost, row) != Some(key) {
            return None;
        }
        if right {
            Some(outermost + 1)
        } else {
            outermost.checked_sub(1)
        }
    };
    let (left_edge, right_edge) = (
        ((cx - outer) / DOTS_X as f64).floor().max(area.x as f64) as usize,
        (((cx + outer) / DOTS_X as f64).ceil() as usize).min(col_end),
    );
    let mut taken = [Vec::new(), Vec::new()];
    // The cells of the labels and leaders placed so far.
    let mut placed: HashSet<(usize, usize)> = HashSet::new();
    for (mid, key, label, color) in labels {
        let right = mid < PI;
        let side = usize::from(right);
        let anchor_row = ((cy - mid.cos() * outer) / DOTS_Y as f64)
            .floor()
            .clamp(area.y as f64, row_end.saturating_sub(1) as f64)
            as usize;
        // The label starts `gap` columns past the circle and takes what is
        // left of the row on its side, cut with an ellipsis when too long.
        let room = if right {
            col_end.saturating_sub(right_edge + gap)
        } else {
            left_edge.saturating_sub(area.x + gap)
        };
        let full = UnicodeWidthStr::width(label.as_str()).min(widest_allowed);
        let width = full.min(room);
        if width < 2 {
            continue;
        }
        let shown = fit_label(&label, width);
        let column = if right {
            right_edge + gap
        } else {
            left_edge - gap - width
        };
        // The rows where a leader can start beside the slice, nearest the
        // anchor first; from each, the nearest free row to it whose label
        // and leader cross no slice and nothing placed before.
        let Some((row, path)) = nearest(anchor_row, area)
            .filter_map(|start| reach(start, right, key).map(|from| (start, from)))
            .find_map(|(start, from)| {
                nearest(start, area)
                    .filter(|row| !taken[side].contains(row))
                    .filter(|row| !(column..column + width).any(|x| placed.contains(&(x, *row))))
                    .find_map(|row| {
                        leader(mask, &placed, from, right, start, row, column, width)
                            .map(|path| (row, path))
                    })
            })
        else {
            continue;
        };
        taken[side].push(row);
        text.text(column, row, width, &shown, color);
        placed.extend((column..column + width).map(|x| (x, row)));
        for (x, y, glyph) in path {
            text.put(x, y, glyph, None);
            placed.insert((x, y));
        }
    }
}

/// The rows of `area` by their distance from `row`, nearest first, below
/// before above.
fn nearest(row: usize, area: Rect) -> impl Iterator<Item = usize> {
    (0..area.h)
        .flat_map(move |d| [Some(row + d), (d > 0).then(|| row.wrapping_sub(d))])
        .flatten()
        .filter(move |r| (area.y..area.y + area.h).contains(r))
}

/// The cells of the leader from `edge`, the first unpainted column past the
/// slice on `anchor_row`, to the label at `column` on `row`: a horizontal
/// run, and a bend down or up when the label moved rows. `None` when the
/// leader would have no cell, because the label touches its slice, or a
/// cell would cover a painted cell, its own label, or a cell in `placed`:
/// so every placed label has a leader, and no leader draws over a slice,
/// a label or another leader.
#[allow(clippy::too_many_arguments)]
fn leader(
    mask: &MaskCanvas,
    placed: &HashSet<(usize, usize)>,
    edge: usize,
    right: bool,
    anchor_row: usize,
    row: usize,
    column: usize,
    width: usize,
) -> Option<Vec<(usize, usize, &'static str)>> {
    // `near` is the column beside the label, `bend` the one past it.
    let (near, bend) = if right {
        let near = column.checked_sub(1)?;
        (near, near.checked_sub(1))
    } else {
        (column + width, Some(column + width + 1))
    };
    // The columns on `anchor_row` from `edge` outward to `to`.
    let run = |to: usize| -> Vec<usize> {
        if right {
            (edge..=to).collect()
        } else {
            (to..=edge).collect()
        }
    };
    let mut path = Vec::new();
    if row == anchor_row {
        path.extend(run(near).into_iter().map(|x| (x, row, "─")));
    } else {
        let bend = bend?;
        let horizontal = run(bend);
        if horizontal.is_empty() {
            return None;
        }
        let down = row > anchor_row;
        let (turn, corner) = match (right, down) {
            (true, true) => ("╮", "╰"),
            (true, false) => ("╯", "╭"),
            (false, true) => ("╭", "╯"),
            (false, false) => ("╰", "╮"),
        };
        path.extend(
            horizontal
                .into_iter()
                .map(|x| (x, anchor_row, if x == bend { turn } else { "─" })),
        );
        path.extend((anchor_row.min(row) + 1..anchor_row.max(row)).map(|y| (bend, y, "│")));
        path.push((bend, row, corner));
        path.push((near, row, "─"));
    }
    let label = column..column + width;
    if path.is_empty()
        || path.iter().any(|(x, y, _)| {
            mask.is_painted(*x, *y)
                || (*y == row && label.contains(x))
                || placed.contains(&(*x, *y))
        })
    {
        return None;
    }
    Some(path)
}
