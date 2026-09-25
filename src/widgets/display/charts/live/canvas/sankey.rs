//! Sankey charts (CHT-030): nodes and links laid out by the plot layer's
//! d3-sankey port and drawn as filled shapes of the shared mask canvas
//! (CHT-025). Each link is a ribbon shaded from its source node's color to
//! its target's, blended toward the chart background by the link opacity;
//! while a node is selected, its own links keep that opacity and the rest
//! fade further (CHT-032). Nodes are drawn over the ribbons. At the medium
//! and large size classes each node's label sits beside it in the text
//! layer: a first-layer node's on its left, a last-layer node's on its
//! right and any other's centered above it, with the throughput at large.

use super::super::super::mask::{MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::sankey::Sankey;
use super::super::super::plot::{fit_label, Rect, Rgba, SizeClass, TextSink};
use super::super::super::{ChartProps, SankeyOptions};
use super::{Job, Picture, SankeyHit, TextLayer};
use unicode_width::UnicodeWidthStr;

/// How far the links a selection does not touch fade, as a share of the
/// link opacity (the reference's hover dimming).
const DIM: f32 = 0.7;
/// Color steps along a ribbon's shading.
const SHADES: f64 = 24.0;

/// The layout generator for `options`, in dot units with nodes on whole
/// columns.
pub(super) fn generator(options: &SankeyOptions) -> Sankey {
    Sankey::new()
        .node_align(options.node_align)
        .iterations(options.iterations)
        .value_scale(options.value_scale)
        .node_width(f64::from(options.node_width.max(1)) * DOTS_X as f64)
        .node_padding(f64::from(options.node_padding) * DOTS_Y as f64)
        .snap_x(DOTS_X as f64)
}

/// A node's color: its point's color token, else the palette entry for its
/// index.
pub(super) fn node_color(props: &ChartProps, index: usize) -> Option<Rgba> {
    let colors = props.color_palette.len().max(1);
    props.series[0].data[index]
        .color
        .as_deref()
        .or_else(|| props.color_palette.get(index % colors).map(String::as_str))
        .and_then(super::color)
}

/// A node's throughput: the larger of its raw incoming and outgoing totals,
/// from the links (the reference's raw throughput), whatever its point's
/// value says.
fn throughput(props: &ChartProps, index: usize) -> f64 {
    let (mut incoming, mut outgoing) = (0.0, 0.0);
    for link in &props.sankey.links {
        if link.source == index {
            outgoing += link.value;
        }
        if link.target == index {
            incoming += link.value;
        }
    }
    f64::max(incoming, outgoing)
}

/// A node's name and throughput text, as the tooltip and the large labels
/// show them.
pub(in super::super) fn node_text(props: &ChartProps, index: usize) -> (String, String) {
    let point = &props.series[0].data[index];
    let name = point.label.clone().unwrap_or_else(|| index.to_string());
    let value = props
        .sankey
        .value_labels
        .get(index)
        .cloned()
        .unwrap_or_else(|| throughput(props, index).to_string());
    (name, value)
}

/// The label lines beside node `index` at `class`: the `labels` accessor's
/// lines when it gave any, else the node's name, with its throughput at the
/// large class.
fn label_lines(props: &ChartProps, index: usize, class: SizeClass) -> Vec<(String, Option<Rgba>)> {
    match props
        .sankey
        .labels
        .get(index)
        .filter(|lines| !lines.is_empty())
    {
        Some(lines) => lines
            .iter()
            .map(|line| {
                (
                    line.text.clone(),
                    line.color.as_deref().and_then(super::color),
                )
            })
            .collect(),
        None => {
            let (name, value) = node_text(props, index);
            let text = if class == SizeClass::Large {
                format!("{name} {value}")
            } else {
                name
            };
            if text.is_empty() {
                Vec::new()
            } else {
                vec![(text, None)]
            }
        }
    }
}

/// `color` blended toward the theme's `background`: `opacity` of the
/// color. Without a background color the color is kept whole.
fn blend(color: Rgba, background: Option<Rgba>, opacity: f32) -> Rgba {
    let Some(background) = background else {
        return color;
    };
    let mix = |c: f32, b: f32| b + (c - b) * opacity;
    (
        mix(color.0, background.0),
        mix(color.1, background.1),
        mix(color.2, background.2),
        1.0,
    )
}

/// The color `share` of the way from `from` to `to`.
fn shade(from: Rgba, to: Rgba, share: f32) -> Rgba {
    let mix = |a: f32, b: f32| a + (b - a) * share;
    (mix(from.0, to.0), mix(from.1, to.1), mix(from.2, to.2), 1.0)
}

pub(super) fn sankey(
    mask: &mut MaskCanvas,
    text: &mut TextLayer,
    picture: &mut Picture,
    job: &Job,
    area: Rect,
    class: SizeClass,
) {
    let props = job.props;
    let options = &props.sankey;
    let count = props.series[0].data.len();
    let generator = generator(options);
    // validate() reported a missing node or a cycle before any shape.
    let Ok(topology) = generator.topology(count, &options.links) else {
        return;
    };
    if topology.nodes.iter().all(|node| node.value <= 0.0) {
        text.text(area.x, area.y, area.w, "No data to display", None);
        return;
    }
    let last = topology.layer_count().saturating_sub(1);
    let lines: Vec<Vec<(String, Option<Rgba>)>> = (0..count)
        .map(|index| {
            if class == SizeClass::Mini {
                Vec::new()
            } else {
                label_lines(props, index, class)
            }
        })
        .collect();
    let gap = usize::from(options.label_gap);
    // Columns kept free beside the first and last layers for their labels,
    // and rows above for the labels of the layers between.
    let widest = |layer: usize| {
        topology
            .nodes
            .iter()
            .filter(|node| node.layer == layer)
            .flat_map(|node| &lines[node.index])
            .map(|(line, _)| UnicodeWidthStr::width(line.as_str()))
            .max()
            .unwrap_or(0)
    };
    let side = |layer: usize| match widest(layer) {
        0 => 0,
        w => (w + gap).min(area.w / 5),
    };
    let left = side(0);
    let right = if last > 0 { side(last) } else { 0 };
    let above = topology
        .nodes
        .iter()
        .filter(|node| node.layer > 0 && node.layer < last)
        .map(|node| lines[node.index].len())
        .max()
        .unwrap_or(0)
        .min(area.h / 4);
    let plot = Rect {
        x: area.x + left,
        y: area.y + above,
        w: area.w.saturating_sub(left + right),
        h: area.h.saturating_sub(above),
    };
    if plot.w <= 2 * usize::from(options.node_width.max(1)) || plot.h == 0 {
        return;
    }
    let graph = generator
        .extent(
            (plot.x * DOTS_X) as f64,
            (plot.y * DOTS_Y) as f64,
            (plot.right() * DOTS_X) as f64,
            (plot.bottom() * DOTS_Y) as f64,
        )
        .layout_from(topology);
    picture.plot = area;
    // The reveal uncovers the chart from the left.
    let shown = ((area.w as f64 * job.progress.clamp(0.0, 1.0)).ceil() as usize).min(area.w);
    mask.set_clip_cells(area.x, area.y, shown, area.h);
    let colors: Vec<Option<Rgba>> = (0..count).map(|index| node_color(props, index)).collect();
    let background = super::color("background");
    let selected = job
        .selected
        .map(|(_, index)| index)
        .filter(|index| *index < count);
    let (pw, ph) = mask.fill_blitter().cell_pixels();
    // One fill pixel, in dots.
    let (step, pixel) = (
        DOTS_X as f64 / f64::from(pw.max(1)),
        DOTS_Y as f64 / f64::from(ph.max(1)),
    );
    let min_width = f64::from(options.min_link_width.max(0.0)) * DOTS_Y as f64;
    let opacity = options.link_opacity.clamp(0.0, 1.0);
    // Ribbons that skip columns go first, under the ribbons that end at the
    // nodes they pass, so every ribbon's ends show.
    let mut order: Vec<usize> = (0..graph.links.len()).collect();
    order.sort_by_key(|index| {
        let ribbon = &graph.links[*index];
        std::cmp::Reverse(graph.nodes[ribbon.target].layer - graph.nodes[ribbon.source].layer)
    });
    for index in order {
        let ribbon = &graph.links[index];
        if ribbon.value <= 0.0 {
            continue;
        }
        let touches = |node: usize| ribbon.source == node || ribbon.target == node;
        let opacity = if selected.is_some_and(|node| !touches(node)) {
            opacity * (1.0 - DIM)
        } else {
            opacity
        };
        // validate() rejected color tokens that do not resolve.
        let (Some(from), Some(to)) = (colors[ribbon.source], colors[ribbon.target]) else {
            continue;
        };
        let (start, end) = (graph.nodes[ribbon.source].x1, graph.nodes[ribbon.target].x0);
        let mut x = start;
        while x < end {
            let Some((top, bottom, _)) =
                graph.ribbon_at(index, (x + step / 2.0).min(end), min_width)
            else {
                break;
            };
            // One shade per cell, taken at the cell's middle, so the pixel
            // columns of a cell share a color.
            let middle = ((x / DOTS_X as f64).floor() + 0.5) * DOTS_X as f64;
            let across = graph
                .ribbon_at(index, middle.clamp(start, end), min_width)
                .map_or(0.0, |(_, _, across)| across);
            let share = ((across * SHADES).round() / SHADES) as f32;
            let color = blend(shade(from, to, share), background, opacity);
            // The ribbon's edges curve across the strip, so every sample
            // is tested at its own position.
            mask.fill_where(
                (x, top - pixel, x + step, bottom + pixel),
                Some(color),
                None,
                |px, py| {
                    graph
                        .ribbon_at(index, px, min_width)
                        .is_some_and(|(top, bottom, _)| py >= top && py < bottom)
                },
            );
            x += step;
        }
    }
    let mut hit = SankeyHit {
        nodes: Vec::with_capacity(count),
        order: Vec::new(),
        colors: colors.clone(),
        samples: (pw, ph),
    };
    for node in &graph.nodes {
        // A node too small for one fill pixel keeps one.
        let bottom = node.y1.max(node.y0 + pixel);
        mask.fill_where(
            (node.x0, node.y0, node.x1, bottom),
            colors[node.index],
            Some((0, node.index)),
            |_, _| true,
        );
        hit.nodes.push((node.x0, node.y0, node.x1, bottom));
        picture.anchors.insert(
            (0, node.index),
            (
                (node.x1 / DOTS_X as f64) as usize,
                ((node.y0 + bottom) / 2.0 / DOTS_Y as f64) as usize,
            ),
        );
    }
    hit.order = (0..count).collect();
    hit.order.sort_by(|a, b| {
        let (na, nb) = (&graph.nodes[*a], &graph.nodes[*b]);
        na.layer.cmp(&nb.layer).then(na.y0.total_cmp(&nb.y0))
    });
    picture.sankey = Some(hit);
    for node in &graph.nodes {
        let lines = &lines[node.index];
        if lines.is_empty() {
            continue;
        }
        let (left_col, right_col) = (
            (node.x0 / DOTS_X as f64).floor() as usize,
            (node.x1 / DOTS_X as f64).ceil() as usize,
        );
        let middle_row = ((node.y0 + node.y1) / 2.0 / DOTS_Y as f64) as usize;
        for (k, (line, color)) in lines.iter().enumerate() {
            let full = UnicodeWidthStr::width(line.as_str());
            let (row, start, room) = if node.layer == 0 {
                let end = left_col.saturating_sub(gap);
                let row = (middle_row + k).saturating_sub((lines.len() - 1) / 2);
                let room = end.saturating_sub(area.x);
                (row, end.saturating_sub(full.min(room)), room)
            } else if node.layer == last {
                let start = right_col + gap;
                let row = (middle_row + k).saturating_sub((lines.len() - 1) / 2);
                (row, start, area.right().saturating_sub(start))
            } else {
                // The row of the node's first drawn pixel: the first fill
                // sample whose center is at or below the node's top, so a
                // top a rounding error short of a row keeps its label.
                let first = (node.y0 / pixel - 0.5).ceil().max(0.0);
                let top_row = (first / f64::from(ph.max(1))) as usize;
                let Some(row) = (top_row + k).checked_sub(lines.len()) else {
                    continue;
                };
                let center = (left_col + right_col) / 2;
                let width = full.min(area.w);
                let start = center
                    .saturating_sub(width / 2)
                    .clamp(area.x, area.right().saturating_sub(width));
                (row, start, area.right().saturating_sub(start))
            };
            let width = full.min(room);
            // A label that fits is drawn whole, however short; one cut to a
            // single column says nothing and is left out.
            let cut_to_nothing = width == 0 || (width < full && width < 2);
            if cut_to_nothing || !(area.y..area.bottom()).contains(&row) {
                continue;
            }
            text.text(start, row, width, &fit_label(line, width), *color);
        }
    }
}
