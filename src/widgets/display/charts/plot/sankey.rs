//! The Sankey layout (CHT-030): a port of d3-sankey as the reference ports
//! it (gpui-kit's `plot/shape/sankey.rs`). Nodes sit in columns by their
//! longest path from a source, placed by the alignment; each column's nodes
//! take heights proportional to their throughput under one value scale, and
//! a number of relaxation passes move them toward the flows they carry.
//! Link widths follow d3-sankey itself: a link is its value under the scale
//! wide at both ends, stacked at each node without overlap, so a node whose
//! incoming and outgoing totals differ has one side partly uncovered. The
//! reference instead stretches the smaller side to fill it.
//!
//! Positions are in the caller's units; the chart uses dot units, and
//! [`Sankey::snap_x`] puts the nodes on whole columns.

use std::fmt;

/// A flow of `value` from the node at index `source` to the node at index
/// `target` (d3-sankey's default node id is the index).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SankeyLink {
    /// Index of the node the flow leaves.
    pub source: usize,
    /// Index of the node the flow enters.
    pub target: usize,
    /// The flow amount.
    pub value: f64,
}

impl SankeyLink {
    /// A link carrying `value` from node `source` to node `target`.
    pub fn new(source: usize, target: usize, value: f64) -> Self {
        Self {
            source,
            target,
            value,
        }
    }
}

/// Which column a node takes (d3-sankey's `sankeyLeft`, `sankeyRight`,
/// `sankeyCenter` and `sankeyJustify`).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum SankeyAlign {
    /// Each node at its depth from a source.
    Left,
    /// Each node at its height above a sink, counted from the right.
    Right,
    /// Nodes with inputs at their depth; a pure source just before its
    /// nearest target.
    Center,
    /// Like `Left`, with every sink in the last column.
    #[default]
    Justify,
}

/// How flow values map to node heights and link widths.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum SankeyValueScale {
    /// Proportional to the value.
    #[default]
    Linear,
    /// Proportional to the square root of the value, so a dominant flow does
    /// not dwarf the small ones.
    Sqrt,
}

impl SankeyValueScale {
    /// The value in layout units.
    pub fn apply(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Sqrt => value.max(0.0).sqrt(),
        }
    }
}

/// Why links cannot be laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SankeyError {
    /// A link names a node index past the end of the nodes.
    MissingNode(usize),
    /// The links form a cycle.
    CircularLink,
}

impl fmt::Display for SankeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(index) => write!(f, "Sankey link names a missing node ({index})"),
            Self::CircularLink => write!(f, "Sankey links form a cycle"),
        }
    }
}

impl std::error::Error for SankeyError {}

/// A node with its computed place.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SankeyNode {
    /// Index into the nodes.
    pub index: usize,
    /// Throughput in layout units: the larger of the incoming and outgoing
    /// totals under the value scale.
    pub value: f64,
    /// Longest path from a source.
    pub depth: usize,
    /// Longest path to a sink.
    pub height: usize,
    /// Column after alignment.
    pub layer: usize,
    /// Left edge.
    pub x0: f64,
    /// Right edge.
    pub x1: f64,
    /// Top edge.
    pub y0: f64,
    /// Bottom edge.
    pub y1: f64,
    /// Indices of the outgoing links, top to bottom.
    pub source_links: Vec<usize>,
    /// Indices of the incoming links, top to bottom.
    pub target_links: Vec<usize>,
}

/// A link with its computed place.
#[derive(Clone, Debug, PartialEq)]
pub struct SankeyRibbon {
    /// Index into the links.
    pub index: usize,
    /// Source node index.
    pub source: usize,
    /// Target node index.
    pub target: usize,
    /// The flow in layout units.
    pub value: f64,
    /// Center of the ribbon at the source end.
    pub y0: f64,
    /// Center of the ribbon at the target end.
    pub y1: f64,
    /// Width at both ends: the value under the layout's scale.
    pub width: f64,
}

/// A laid-out graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SankeyGraph {
    /// The nodes, indexed like the input.
    pub nodes: Vec<SankeyNode>,
    /// The links, indexed like the input.
    pub links: Vec<SankeyRibbon>,
}

impl SankeyGraph {
    /// The number of columns, 0 for no nodes.
    pub fn layer_count(&self) -> usize {
        self.nodes.iter().map(|n| n.layer + 1).max().unwrap_or(0)
    }

    /// The ribbon of link `link` at `x`, between its source node's right
    /// edge and its target node's left edge: `(top, bottom, across)`, where
    /// `across` is how far along the ribbon `x` lies, from 0 at the source
    /// to 1 at the target. The edges follow d3-sankey's horizontal link, a
    /// cubic with both control points at the middle, and the ribbon is at
    /// least `min_width` wide. `None` outside the ribbon's span.
    pub fn ribbon_at(&self, link: usize, x: f64, min_width: f64) -> Option<(f64, f64, f64)> {
        let ribbon = self.links.get(link)?;
        let (sx, tx) = (self.nodes[ribbon.source].x1, self.nodes[ribbon.target].x0);
        if tx.partial_cmp(&sx) != Some(std::cmp::Ordering::Greater) || x < sx || x > tx {
            return None;
        }
        let across = (x - sx) / (tx - sx);
        let eased = ease(across);
        let half = ribbon.width.max(min_width) / 2.0;
        let center = ribbon.y0 + (ribbon.y1 - ribbon.y0) * eased;
        Some((center - half, center + half, across))
    }
}

/// The vertical share at horizontal share `u` along a cubic whose control
/// points both sit at the middle: x(t) = 1.5t - 1.5t² + t³ and y(t) =
/// 3t² - 2t³, solved for t by Newton's method (x is strictly increasing).
fn ease(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    let mut t = u;
    for _ in 0..8 {
        let x = t * (1.5 - 1.5 * t + t * t) - u;
        let slope = 1.5 - 3.0 * t + 3.0 * t * t;
        t = (t - x / slope).clamp(0.0, 1.0);
    }
    t * t * (3.0 - 2.0 * t)
}

/// The Sankey layout generator.
#[derive(Clone, Debug, PartialEq)]
pub struct Sankey {
    node_width: f64,
    node_padding: f64,
    align: SankeyAlign,
    iterations: usize,
    value_scale: SankeyValueScale,
    snap_x: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl Default for Sankey {
    fn default() -> Self {
        Self {
            node_width: 24.0,
            node_padding: 8.0,
            align: SankeyAlign::default(),
            iterations: 6,
            value_scale: SankeyValueScale::default(),
            snap_x: 0.0,
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 1.0,
        }
    }
}

impl Sankey {
    /// A generator with d3-sankey's defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// The width of every node.
    pub fn node_width(mut self, width: f64) -> Self {
        self.node_width = width;
        self
    }

    /// The vertical gap between nodes in a column.
    pub fn node_padding(mut self, padding: f64) -> Self {
        self.node_padding = padding;
        self
    }

    /// The column alignment.
    pub fn node_align(mut self, align: SankeyAlign) -> Self {
        self.align = align;
        self
    }

    /// The number of relaxation passes.
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    /// How values map to heights.
    pub fn value_scale(mut self, scale: SankeyValueScale) -> Self {
        self.value_scale = scale;
        self
    }

    /// Round each column's left edge to a multiple of `step` (0 keeps
    /// fractions), so nodes start on whole cells.
    pub fn snap_x(mut self, step: f64) -> Self {
        self.snap_x = step;
        self
    }

    /// The layout bounds.
    pub fn extent(mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        (self.x0, self.y0, self.x1, self.y1) = (x0, y0, x1, y1);
        self
    }

    /// The columns and throughputs only, without vertical placement, so a
    /// caller can measure the first and last columns' labels before it
    /// fixes the extent. Fails when a link names a missing node or the
    /// links form a cycle.
    pub fn topology(
        &self,
        node_count: usize,
        links: &[SankeyLink],
    ) -> Result<SankeyGraph, SankeyError> {
        if let Some(missing) = links
            .iter()
            .flat_map(|l| [l.source, l.target])
            .find(|index| *index >= node_count)
        {
            return Err(SankeyError::MissingNode(missing));
        }
        let mut graph = SankeyGraph {
            nodes: (0..node_count)
                .map(|index| SankeyNode {
                    index,
                    ..SankeyNode::default()
                })
                .collect(),
            links: links
                .iter()
                .enumerate()
                .map(|(index, link)| SankeyRibbon {
                    index,
                    source: link.source,
                    target: link.target,
                    value: self.value_scale.apply(link.value),
                    y0: 0.0,
                    y1: 0.0,
                    width: 0.0,
                })
                .collect(),
        };
        for (index, link) in links.iter().enumerate() {
            graph.nodes[link.source].source_links.push(index);
            graph.nodes[link.target].target_links.push(index);
        }
        for index in 0..node_count {
            let total = |links: &[usize]| links.iter().map(|l| graph.links[*l].value).sum::<f64>();
            let node = &graph.nodes[index];
            let value = total(&node.source_links).max(total(&node.target_links));
            graph.nodes[index].value = value;
        }
        ranks(&mut graph)?;
        self.layers(&mut graph);
        Ok(graph)
    }

    /// The full layout: the topology plus vertical placement and link
    /// breadths.
    pub fn layout(
        &self,
        node_count: usize,
        links: &[SankeyLink],
    ) -> Result<SankeyGraph, SankeyError> {
        Ok(self.layout_from(self.topology(node_count, links)?))
    }

    /// Finish a graph from [`Sankey::topology`] for this generator's extent.
    pub fn layout_from(&self, mut graph: SankeyGraph) -> SankeyGraph {
        if graph.nodes.is_empty() {
            return graph;
        }
        self.layers(&mut graph);
        let mut columns = vec![Vec::new(); graph.layer_count()];
        for node in &graph.nodes {
            columns[node.layer].push(node.index);
        }
        self.breadths(&mut graph, &mut columns);
        link_breadths(&mut graph);
        self.center_columns(&mut graph);
        graph
    }

    fn layer_of(&self, graph: &SankeyGraph, index: usize, n: usize) -> usize {
        let node = &graph.nodes[index];
        match self.align {
            SankeyAlign::Left => node.depth,
            SankeyAlign::Right => (n - 1).saturating_sub(node.height),
            SankeyAlign::Justify => {
                if node.source_links.is_empty() {
                    n - 1
                } else {
                    node.depth
                }
            }
            SankeyAlign::Center => {
                if !node.target_links.is_empty() {
                    node.depth
                } else if !node.source_links.is_empty() {
                    node.source_links
                        .iter()
                        .map(|link| graph.nodes[graph.links[*link].target].depth)
                        .min()
                        .unwrap_or(1)
                        .saturating_sub(1)
                } else {
                    0
                }
            }
        }
    }

    fn layers(&self, graph: &mut SankeyGraph) {
        let n = graph.nodes.iter().map(|n| n.depth + 1).max().unwrap_or(0);
        if n == 0 {
            return;
        }
        let kx = if n > 1 {
            (self.x1 - self.x0 - self.node_width) / (n - 1) as f64
        } else {
            0.0
        };
        let layers: Vec<usize> = (0..graph.nodes.len())
            .map(|index| self.layer_of(graph, index, n).min(n - 1))
            .collect();
        for (index, layer) in layers.into_iter().enumerate() {
            let mut x0 = self.x0 + layer as f64 * kx;
            if self.snap_x > 0.0 {
                x0 = (x0 / self.snap_x).round() * self.snap_x;
            }
            let node = &mut graph.nodes[index];
            node.layer = layer;
            node.x0 = x0;
            node.x1 = x0 + self.node_width;
        }
    }

    fn breadths(&self, graph: &mut SankeyGraph, columns: &mut [Vec<usize>]) {
        let tallest = columns.iter().map(Vec::len).max().unwrap_or(0);
        let py = if tallest > 1 {
            self.node_padding
                .min((self.y1 - self.y0) / (tallest - 1) as f64)
        } else {
            self.node_padding
        };
        // The one scale from value to height: the tightest column's.
        let ky = columns
            .iter()
            .filter_map(|column| {
                let total: f64 = column.iter().map(|i| graph.nodes[*i].value).sum();
                (total > 0.0).then(|| (self.y1 - self.y0 - (column.len() - 1) as f64 * py) / total)
            })
            .fold(f64::INFINITY, f64::min);
        let ky = if ky.is_finite() { ky.max(0.0) } else { 0.0 };
        for column in columns.iter() {
            let mut y = self.y0;
            for &index in column {
                let node = &mut graph.nodes[index];
                node.y0 = y;
                node.y1 = y + node.value * ky;
                y = node.y1 + py;
            }
            // Spread the leftover height evenly between the nodes.
            let leftover = (self.y1 - y + py) / (column.len() + 1) as f64;
            for (i, &index) in column.iter().enumerate() {
                let node = &mut graph.nodes[index];
                node.y0 += leftover * (i + 1) as f64;
                node.y1 += leftover * (i + 1) as f64;
            }
        }
        for link in &mut graph.links {
            link.width = link.value * ky;
        }
        for column in columns.iter() {
            for &index in column {
                sort_source_links(graph, index);
                sort_target_links(graph, index);
            }
        }
        for i in 0..self.iterations {
            let alpha = 0.99_f64.powi(i as i32);
            let beta = (1.0 - alpha).max((i + 1) as f64 / self.iterations as f64);
            self.relax(graph, columns, alpha, beta, py, false);
            self.relax(graph, columns, alpha, beta, py, true);
        }
    }

    /// One relaxation sweep: right to left moves each node toward its
    /// outgoing links' slots, left to right toward its incoming ones'.
    fn relax(
        &self,
        graph: &mut SankeyGraph,
        columns: &mut [Vec<usize>],
        alpha: f64,
        beta: f64,
        py: f64,
        left_to_right: bool,
    ) {
        let order: Vec<usize> = if left_to_right {
            (1..columns.len()).collect()
        } else {
            (0..columns.len().saturating_sub(1)).rev().collect()
        };
        for i in order {
            for &index in &columns[i] {
                let (mut y, mut w) = (0.0, 0.0);
                let links = if left_to_right {
                    graph.nodes[index].target_links.clone()
                } else {
                    graph.nodes[index].source_links.clone()
                };
                for link in links {
                    let ribbon = &graph.links[link];
                    let (source, target) = (ribbon.source, ribbon.target);
                    let v = ribbon.value
                        * (graph.nodes[target].layer as f64 - graph.nodes[source].layer as f64);
                    let top = if left_to_right {
                        target_top(graph, source, target, py)
                    } else {
                        source_top(graph, source, target, py)
                    };
                    y += top * v;
                    w += v;
                }
                if w <= 0.0 {
                    continue;
                }
                let dy = (y / w - graph.nodes[index].y0) * alpha;
                graph.nodes[index].y0 += dy;
                graph.nodes[index].y1 += dy;
                reorder_node_links(graph, index);
            }
            columns[i].sort_by(|a, b| graph.nodes[*a].y0.total_cmp(&graph.nodes[*b].y0));
            self.resolve_collisions(graph, &columns[i], beta, py);
        }
    }

    /// d3-sankey's middle-out collision resolution, then clamping to the
    /// extent.
    fn resolve_collisions(&self, graph: &mut SankeyGraph, column: &[usize], beta: f64, py: f64) {
        if column.is_empty() {
            return;
        }
        let i = column.len() >> 1;
        let (top, bottom) = (graph.nodes[column[i]].y0, graph.nodes[column[i]].y1);
        push_up(graph, &column[..i], top - py, beta, py);
        push_down(graph, &column[i + 1..], bottom + py, beta, py);
        push_up(graph, column, self.y1, beta, py);
        push_down(graph, column, self.y0, beta, py);
    }

    /// Center each column's stack of nodes in the extent, as the reference
    /// does, so sparse columns do not sit high.
    fn center_columns(&self, graph: &mut SankeyGraph) {
        let layers = graph.layer_count();
        let mut lo = vec![f64::INFINITY; layers];
        let mut hi = vec![f64::NEG_INFINITY; layers];
        for node in &graph.nodes {
            lo[node.layer] = lo[node.layer].min(node.y0);
            hi[node.layer] = hi[node.layer].max(node.y1);
        }
        let offset: Vec<f64> = (0..layers)
            .map(|l| {
                if lo[l].is_finite() && hi[l] > lo[l] {
                    (self.y0 + self.y1 - lo[l] - hi[l]) / 2.0
                } else {
                    0.0
                }
            })
            .collect();
        let by_node: Vec<f64> = graph.nodes.iter().map(|n| offset[n.layer]).collect();
        for node in &mut graph.nodes {
            node.y0 += by_node[node.index];
            node.y1 += by_node[node.index];
        }
        for link in &mut graph.links {
            link.y0 += by_node[link.source];
            link.y1 += by_node[link.target];
        }
    }
}

/// Longest-path depth from a source and height to a sink, from one
/// topological order; a node left out of the order sits on a cycle.
fn ranks(graph: &mut SankeyGraph) -> Result<(), SankeyError> {
    let n = graph.nodes.len();
    let mut incoming: Vec<usize> = graph.nodes.iter().map(|n| n.target_links.len()).collect();
    let mut order: Vec<usize> = (0..n).filter(|i| incoming[*i] == 0).collect();
    let mut depths = vec![0usize; n];
    let mut visited = 0;
    while visited < order.len() {
        let index = order[visited];
        visited += 1;
        for &link in &graph.nodes[index].source_links {
            let target = graph.links[link].target;
            depths[target] = depths[target].max(depths[index] + 1);
            incoming[target] -= 1;
            if incoming[target] == 0 {
                order.push(target);
            }
        }
    }
    if order.len() != n {
        return Err(SankeyError::CircularLink);
    }
    let mut heights = vec![0usize; n];
    for &index in order.iter().rev() {
        for &link in &graph.nodes[index].source_links {
            heights[index] = heights[index].max(heights[graph.links[link].target] + 1);
        }
    }
    for (node, (depth, height)) in graph.nodes.iter_mut().zip(depths.into_iter().zip(heights)) {
        node.depth = depth;
        node.height = height;
    }
    Ok(())
}

/// Sort a node's outgoing links by their targets' tops, then by index.
fn sort_source_links(graph: &mut SankeyGraph, index: usize) {
    let mut links = std::mem::take(&mut graph.nodes[index].source_links);
    links.sort_by(|a, b| {
        let (ya, yb) = (
            graph.nodes[graph.links[*a].target].y0,
            graph.nodes[graph.links[*b].target].y0,
        );
        ya.total_cmp(&yb).then(a.cmp(b))
    });
    graph.nodes[index].source_links = links;
}

/// Sort a node's incoming links by their sources' tops, then by index.
fn sort_target_links(graph: &mut SankeyGraph, index: usize) {
    let mut links = std::mem::take(&mut graph.nodes[index].target_links);
    links.sort_by(|a, b| {
        let (ya, yb) = (
            graph.nodes[graph.links[*a].source].y0,
            graph.nodes[graph.links[*b].source].y0,
        );
        ya.total_cmp(&yb).then(a.cmp(b))
    });
    graph.nodes[index].target_links = links;
}

/// After a node moved, re-sort its neighbors' link lists.
fn reorder_node_links(graph: &mut SankeyGraph, index: usize) {
    for i in 0..graph.nodes[index].target_links.len() {
        let source = graph.links[graph.nodes[index].target_links[i]].source;
        sort_source_links(graph, source);
    }
    for i in 0..graph.nodes[index].source_links.len() {
        let target = graph.links[graph.nodes[index].source_links[i]].target;
        sort_target_links(graph, target);
    }
}

fn push_down(graph: &mut SankeyGraph, column: &[usize], mut y: f64, alpha: f64, py: f64) {
    for &index in column {
        let node = &mut graph.nodes[index];
        let dy = (y - node.y0) * alpha;
        if dy > 1e-6 {
            node.y0 += dy;
            node.y1 += dy;
        }
        y = node.y1 + py;
    }
}

fn push_up(graph: &mut SankeyGraph, column: &[usize], mut y: f64, alpha: f64, py: f64) {
    for &index in column.iter().rev() {
        let node = &mut graph.nodes[index];
        let dy = (node.y1 - y) * alpha;
        if dy > 1e-6 {
            node.y0 -= dy;
            node.y1 -= dy;
        }
        y = node.y0 - py;
    }
}

/// The top `target` would need so its ribbon from `source` lines up with
/// the ribbon's slot in the source's outgoing stack (d3's targetTop).
fn target_top(graph: &SankeyGraph, source: usize, target: usize, py: f64) -> f64 {
    let node = &graph.nodes[source];
    let mut y = node.y0 - node.source_links.len().saturating_sub(1) as f64 * py / 2.0;
    for &link in &node.source_links {
        if graph.links[link].target == target {
            break;
        }
        y += graph.links[link].width + py;
    }
    for &link in &graph.nodes[target].target_links {
        if graph.links[link].source == source {
            break;
        }
        y -= graph.links[link].width;
    }
    y
}

/// The top `source` would need so its ribbon to `target` lines up with the
/// ribbon's slot in the target's incoming stack (d3's sourceTop).
fn source_top(graph: &SankeyGraph, source: usize, target: usize, py: f64) -> f64 {
    let node = &graph.nodes[target];
    let mut y = node.y0 - node.target_links.len().saturating_sub(1) as f64 * py / 2.0;
    for &link in &node.target_links {
        if graph.links[link].source == source {
            break;
        }
        y += graph.links[link].width + py;
    }
    for &link in &graph.nodes[source].source_links {
        if graph.links[link].target == target {
            break;
        }
        y -= graph.links[link].width;
    }
    y
}

/// Stack each node's links from its top, each as wide as its value under
/// the scale (d3-sankey's computeLinkBreadths).
fn link_breadths(graph: &mut SankeyGraph) {
    for index in 0..graph.nodes.len() {
        let mut y = graph.nodes[index].y0;
        for i in 0..graph.nodes[index].source_links.len() {
            let link = &mut graph.links[graph.nodes[index].source_links[i]];
            link.y0 = y + link.width / 2.0;
            y += link.width;
        }
        let mut y = graph.nodes[index].y0;
        for i in 0..graph.nodes[index].target_links.len() {
            let link = &mut graph.links[graph.nodes[index].target_links[i]];
            link.y1 = y + link.width / 2.0;
            y += link.width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn links(links: &[(usize, usize, f64)]) -> Vec<SankeyLink> {
        links
            .iter()
            .map(|(s, t, v)| SankeyLink::new(*s, *t, *v))
            .collect()
    }

    fn layout(links_: &[(usize, usize, f64)], nodes: usize) -> SankeyGraph {
        Sankey::new()
            .node_width(4.0)
            .node_padding(4.0)
            .extent(0.0, 0.0, 100.0, 100.0)
            .layout(nodes, &links(links_))
            .unwrap()
    }

    #[test]
    fn a_chain_takes_one_column_per_step_and_full_height() {
        let graph = layout(&[(0, 1, 5.0), (1, 2, 5.0)], 3);
        assert_eq!(
            graph.nodes.iter().map(|n| n.layer).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        for node in &graph.nodes {
            assert!((node.y1 - node.y0 - 100.0).abs() < 1e-6, "{node:?}");
        }
        assert_eq!(graph.nodes[2].x1, 100.0);
    }

    #[test]
    fn node_heights_are_their_larger_total_and_links_their_values() {
        // Node 1 takes 6 in and sends 2 out: height 6, links 4 and 2 into
        // node 1 stack without overlap.
        let graph = layout(&[(0, 1, 4.0), (3, 1, 2.0), (1, 2, 2.0)], 4);
        let ky = (graph.nodes[1].y1 - graph.nodes[1].y0) / 6.0;
        for link in &graph.links {
            assert!((link.width - link.value * ky).abs() < 1e-9);
        }
        let into: Vec<&SankeyRibbon> = graph.nodes[1]
            .target_links
            .iter()
            .map(|l| &graph.links[*l])
            .collect();
        assert!(
            (into[0].y1 + into[0].width / 2.0 - (into[1].y1 - into[1].width / 2.0)).abs() < 1e-9
        );
    }

    #[test]
    fn alignment_moves_a_short_branch() {
        let data = [(0, 1, 2.0), (1, 2, 2.0), (3, 2, 1.0)];
        let at = |align| {
            Sankey::new()
                .node_align(align)
                .extent(0.0, 0.0, 100.0, 100.0)
                .layout(4, &links(&data))
                .unwrap()
                .nodes[3]
                .layer
        };
        assert_eq!(at(SankeyAlign::Left), 0);
        assert_eq!(at(SankeyAlign::Right), 1);
        assert_eq!(at(SankeyAlign::Center), 1);
        assert_eq!(at(SankeyAlign::Justify), 0);
    }

    #[test]
    fn the_square_root_scale_compresses_a_dominant_flow() {
        let data = links(&[(0, 1, 100.0), (0, 2, 1.0)]);
        let height = |scale| {
            let graph = Sankey::new()
                .value_scale(scale)
                .node_padding(0.0)
                .extent(0.0, 0.0, 10.0, 110.0)
                .layout(3, &data)
                .unwrap();
            graph.nodes[2].y1 - graph.nodes[2].y0
        };
        assert!(height(SankeyValueScale::Sqrt) > 5.0 * height(SankeyValueScale::Linear));
    }

    #[test]
    fn a_cycle_or_a_missing_node_is_an_error() {
        let sankey = Sankey::new();
        assert_eq!(
            sankey.layout(2, &links(&[(0, 1, 1.0), (1, 0, 1.0)])),
            Err(SankeyError::CircularLink)
        );
        assert_eq!(
            sankey.layout(2, &links(&[(0, 2, 1.0)])),
            Err(SankeyError::MissingNode(2))
        );
    }

    #[test]
    fn a_ribbon_runs_from_its_source_slot_to_its_target_slot() {
        let graph = layout(&[(0, 1, 3.0), (0, 2, 1.0)], 3);
        let link = &graph.links[1];
        let (sx, tx) = (graph.nodes[0].x1, graph.nodes[2].x0);
        let (top, bottom, across) = graph.ribbon_at(1, sx, 0.0).unwrap();
        assert!((top - (link.y0 - link.width / 2.0)).abs() < 1e-9 && across == 0.0);
        assert!((bottom - top - link.width).abs() < 1e-9);
        let (top, _, across) = graph.ribbon_at(1, tx, 0.0).unwrap();
        assert!((top - (link.y1 - link.width / 2.0)).abs() < 1e-6 && (across - 1.0).abs() < 1e-12);
        assert!(graph.ribbon_at(1, sx - 1.0, 0.0).is_none());
        let min = link.width * 2.0;
        let (top, bottom, _) = graph.ribbon_at(1, (sx + tx) / 2.0, min).unwrap();
        assert!(
            (bottom - top - min).abs() < 1e-9,
            "a thin ribbon takes the minimum width"
        );
    }
}
