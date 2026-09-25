//! The live chart component: owns the rasterizer worker and its snapshot,
//! drives motion, and handles pointer and keyboard selection. The main
//! thread never rasterizes shapes; it copies the latest picture into the
//! frame and lays the tooltip and crosshair over it.

use super::*;
use crate::{
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo, LayoutType, LifecycleEvent},
    event::{
        router::EventResult,
        types::{KeyCode, KeyEventKind, MouseEventKind},
        Event,
    },
    layout::style::StyleBuilder,
};
use plot::{Rect, Tooltip, TooltipRow};
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod canvas;
mod motion;
mod worker;

use crate::layout::paint_tree::cells::CellGrid;
use canvas::{Picture, RadialHit};

/// How long a chart that has just appeared or changed size, or a Sankey
/// chart whose selection changed, waits for its picture before the frame
/// goes out with the previous one or none. The wait is main-thread
/// time inside the App's frame (BAR-005). Charts up to 200 by 40 cells
/// draw in under 1 ms and appear painted; a larger one paints an empty
/// area for one frame, and the worker's finish signal redraws it.
const NEW_SIZE_WAIT: Duration = Duration::from_millis(4);

/// `picture` if it was drawn at `size`: a picture drawn for another size is
/// stale geometry (BAR-003), neither painted nor used for hit testing.
fn at_size(picture: Option<&Arc<Picture>>, size: (usize, usize)) -> Option<&Arc<Picture>> {
    picture.filter(|picture| (picture.width, picture.height) == size)
}

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: Arc<ChartProps>,
    pub seed: ChartState,
}

impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        // The same shared props need no point-by-point comparison.
        self.seed == other.seed
            && (Arc::ptr_eq(&self.config, &other.config) || *self.config == *other.config)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// What the last submitted job was drawn from, to avoid resubmitting.
#[derive(Clone)]
struct JobKey {
    version: u64,
    width: usize,
    height: usize,
    values: Arc<Vec<Vec<f64>>>,
    progress: f64,
    /// The selection the shapes show (a Sankey chart's faded links).
    selected: Option<(usize, usize)>,
}

impl PartialEq for JobKey {
    /// Values compare by bit pattern, so a NaN equals itself and a chart
    /// holding invalid data does not resubmit a job every frame.
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.width == other.width
            && self.height == other.height
            && self.progress.to_bits() == other.progress.to_bits()
            && self.selected == other.selected
            && (Arc::ptr_eq(&self.values, &other.values)
                || (self.values.len() == other.values.len()
                    && self.values.iter().zip(other.values.iter()).all(|(a, b)| {
                        a.len() == b.len()
                            && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
                    })))
    }
}

struct Latest {
    key: Option<JobKey>,
    next_id: u64,
    picture: Option<Arc<Picture>>,
    /// Whether the chart has shown valid data yet, for the reveal.
    shown: bool,
}

pub(super) struct LiveChart {
    viewport: Option<LayoutInfo>,
    config: Arc<ChartProps>,
    version: u64,
    seed: ChartState,
    worker: Option<worker::Worker>,
    latest: Mutex<Latest>,
    reveal: motion::Reveal,
    transition: motion::Transition,
}

/// Per-series values from the props, the transition's target.
fn target_values(props: &ChartProps) -> Vec<Vec<f64>> {
    props
        .series
        .iter()
        .map(|s| s.data.iter().map(|p| p.value).collect())
        .collect()
}

impl Component for LiveChart {
    type Props = LiveProps;
    type State = ChartState;

    fn new(props: Self::Props) -> Self {
        Self {
            viewport: None,
            config: props.config,
            version: 0,
            seed: props.seed,
            worker: worker::Worker::new().ok(),
            latest: Mutex::new(Latest {
                key: None,
                next_id: 0,
                picture: None,
                shown: false,
            }),
            reveal: motion::Reveal::new(),
            transition: motion::Transition::new(),
        }
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        props.seed.clone()
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if self.seed != props.seed {
            *state = props.seed.clone();
            self.seed = props.seed.clone();
        }
        if !Arc::ptr_eq(&self.config, &props.config) && *self.config != *props.config {
            let shape_changed = self.config.series.len() != props.config.series.len()
                || self
                    .config
                    .series
                    .iter()
                    .zip(&props.config.series)
                    .any(|(a, b)| a.data.len() != b.data.len());
            let type_changed = self.config.chart_type != props.config.chart_type;
            if shape_changed || type_changed {
                state.hovered_point = None;
                state.tooltip = None;
            }
            self.config = props.config.clone();
            self.version += 1;
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut Self::State) -> bool {
        let changed = self
            .viewport
            .is_none_or(|old| old.content_size() != layout.content_size());
        self.viewport = Some(layout);
        changed
    }
    fn render(&self, _props: &Self::Props, state: &Self::State) -> Element {
        let config = self.config.clone();
        if let Some(worker) = &self.worker {
            worker.observe();
        }
        let (width, height) = self.size();
        let valid = canvas::validate(&config).is_ok()
            && width > 0
            && height > 0
            && config
                .series
                .iter()
                .any(|series| series.visible && !series.data.is_empty());
        let target = target_values(&config);
        let (values, _transitioning) = self.transition.values(&config, &target);
        let mut latest = self.latest.lock().unwrap_or_else(|e| e.into_inner());
        // The reveal plays once, when valid data first shows; data that
        // returns after an invalid frame transitions from the last valid
        // rendering instead of revealing again (CHT-022).
        if valid && !latest.shown {
            self.reveal.restart();
            latest.shown = true;
        }
        let progress = self.reveal.fraction(&config, valid);
        let values: Vec<Vec<f64>> = if progress < 1.0 {
            values
                .iter()
                .map(|s| s.iter().map(|v| v * progress).collect())
                .collect()
        } else {
            values
        };
        // A Sankey chart draws its selection into the shapes (CHT-032);
        // other charts patch it over the finished picture.
        let selected = state
            .hovered_point
            .filter(|_| config.show_tooltips && config.chart_type == ChartType::Sankey);
        let key = JobKey {
            version: self.version,
            width,
            height,
            values: Arc::new(values),
            progress,
            selected,
        };
        // Before its first layout the chart has no size and nothing to draw.
        let drawable = width > 0 && height > 0;
        if drawable && latest.key.as_ref() != Some(&key) {
            latest.next_id += 1;
            let id = latest.next_id;
            if let Some(worker) = &self.worker {
                worker.submit(worker::Job {
                    id,
                    props: config.clone(),
                    width,
                    height,
                    values: key.values.clone(),
                    progress,
                    selected,
                });
                // A picture at a new size, or a Sankey chart's new
                // selection, is worth a short wait so a small chart never
                // paints empty and the faded links arrive with the tooltip;
                // other frames at the same size (animation, hover) copy
                // whatever is finished.
                let fresh = at_size(latest.picture.as_ref(), (width, height))
                    .is_some_and(|picture| picture.selected == selected);
                let wait = if fresh { Duration::ZERO } else { NEW_SIZE_WAIT };
                if let Some((_, picture)) = worker.wait_for(id, wait) {
                    latest.picture = Some(picture);
                }
            }
            latest.key = Some(key);
        } else if let Some((_, picture)) = self
            .worker
            .as_ref()
            .filter(|_| drawable)
            .and_then(|w| w.latest())
        {
            latest.picture = Some(picture);
        }
        let picture = at_size(latest.picture.as_ref(), (width, height)).cloned();
        drop(latest);

        let hovered = state.hovered_point.filter(|_| config.show_tooltips);
        let (tooltip, announcement) = picture
            .as_ref()
            .zip(hovered)
            .map(|(picture, (series, index))| self.tooltip(&config, picture, series, index))
            .unwrap_or((None, String::new()));
        // The whole picture is one element: the painter blits its cell grid
        // (BAR-005). The tooltip and crosshair are patched into a copy.
        let grid = picture.as_ref().map(|picture| match &tooltip {
            Some(overlay) => Arc::new(overlay_grid(picture, overlay)),
            None => picture.grid.clone(),
        });
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let mut content = ElementBuilder::new(ElementType::Text(String::new()))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .class("whitespace-pre")
            .build();
        if let Some(grid) = grid {
            content = content.with_cells(grid);
        }
        let mut sizing = StyleBuilder::new()
            .display_flex()
            .max_width_percent(100.0)
            .max_height_percent(100.0)
            .min_width_px(0.0)
            .min_height_px(0.0)
            .overflow_hidden();
        sizing = if config.width == 0 {
            sizing.width_percent(100.0)
        } else {
            sizing.width_px(config.width as f32)
        };
        sizing = if config.height == 0 {
            sizing.height_percent(100.0)
        } else {
            sizing.height_px(config.height as f32)
        };
        let mut element = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(sizing)
            .children(vec![content])
            .build();
        element.focus = Some(FocusProps::button());
        let mut node = crate::accessibility::Node::new(crate::accessibility::Role::Image);
        node.set_label(self.description(&config, &announcement));
        // The worker is still drawing the picture for this size, or for a
        // Sankey chart's new selection; its finish signal redraws the chart.
        let stale = picture
            .as_ref()
            .is_none_or(|picture| picture.selected != selected);
        if stale && drawable && self.worker.is_some() {
            node.set_busy();
        }
        element.metadata.accessibility = Some(node);
        if config.show_tooltips && !announcement.is_empty() {
            element = element.with_child(
                Element::text(announcement)
                    .with_key("chart-point-announcement")
                    .class("sr-only aria-live-polite"),
            );
        }
        if let Some(class) = &config.class {
            element = element.with_class(class);
        }
        element
    }
    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if !props.config.show_tooltips {
            return EventResult::Ignored;
        }
        match event {
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    MouseEventKind::Move | MouseEventKind::Enter | MouseEventKind::Leave
                ) =>
            {
                let hit = if mouse.kind == MouseEventKind::Leave {
                    None
                } else {
                    self.viewport.and_then(|v| {
                        let x = mouse.position.x() as f32;
                        let y = mouse.position.y() as f32;
                        // App dispatch has already converted the event to this
                        // component's local coordinates. Only clipping is global.
                        let [a, b, c, d, tx, ty] = v.transform;
                        let screen = crate::event::hit::Point {
                            x: a * x + c * y + tx,
                            y: b * x + d * y + ty,
                        };
                        if !v.clip.contains(screen) {
                            return None;
                        }
                        let x = (x - v.insets[0]).floor();
                        let y = (y - v.insets[1]).floor();
                        if x < 0.0 || y < 0.0 {
                            return None;
                        }
                        self.nearest(&props.config, x as usize, y as usize)
                    })
                };
                if hit == state.hovered_point && hit.is_some() {
                    return EventResult::Consumed;
                }
                state.hovered_point = hit;
                state.tooltip = hit.and_then(|(s, p)| {
                    self.tooltip_text(&props.config, s, p)
                        .map(|text| (text, mouse.position.x() as u16, mouse.position.y() as u16))
                });
                EventResult::Consumed
            }
            Event::Key(key)
                if key.kind != KeyEventKind::Release
                    && matches!(
                        key.code,
                        KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Home
                            | KeyCode::End
                            | KeyCode::Escape
                    ) =>
            {
                if key.code == KeyCode::Escape {
                    state.hovered_point = None;
                    state.tooltip = None;
                    return EventResult::Consumed;
                }
                // A pie or donut steps through its drawn slices in order,
                // across series and past points that draw no slice, and a
                // Sankey chart through its nodes by layer, top to bottom;
                // one that draws none has nothing to select.
                if matches!(
                    props.config.chart_type,
                    ChartType::Pie | ChartType::Donut | ChartType::Sankey
                ) {
                    let keys: Vec<(usize, usize)> = {
                        let latest = self.latest.lock().unwrap_or_else(|e| e.into_inner());
                        at_size(latest.picture.as_ref(), self.size())
                            .map(|picture| match &picture.sankey {
                                Some(hit) => hit.order.iter().map(|node| (0, *node)).collect(),
                                None => picture.radial.as_ref().map_or_else(Vec::new, |radial| {
                                    radial.slices.iter().map(|s| s.key).collect()
                                }),
                            })
                            .unwrap_or_default()
                    };
                    if keys.is_empty() {
                        return EventResult::Ignored;
                    }
                    let current = state
                        .hovered_point
                        .and_then(|hovered| keys.iter().position(|key| *key == hovered));
                    let last = keys.len() - 1;
                    let at = match key.code {
                        KeyCode::Home => 0,
                        KeyCode::End => last,
                        KeyCode::Left => current.unwrap_or(1).saturating_sub(1),
                        _ => current.map_or(0, |i| (i + 1).min(last)),
                    };
                    state.hovered_point = Some(keys[at]);
                    state.tooltip = None;
                    return EventResult::Consumed;
                }
                let count = props
                    .config
                    .series
                    .iter()
                    .filter(|s| s.visible)
                    .map(|s| s.data.len())
                    .max()
                    .unwrap_or(0);
                if count == 0 {
                    return EventResult::Ignored;
                }
                let current = state.hovered_point.map(|(_, p)| p);
                let index = match key.code {
                    KeyCode::Home => 0,
                    KeyCode::End => count - 1,
                    KeyCode::Left => current.unwrap_or(1).saturating_sub(1),
                    _ => current.map_or(0, |i| (i + 1).min(count - 1)),
                };
                let series = first_series_with(&props.config, index).unwrap_or(0);
                state.hovered_point = Some((series, index));
                state.tooltip = None;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        if matches!(event, LifecycleEvent::Unmount) {
            self.reveal.cancel();
            self.transition.cancel();
        }
    }
}

/// The (series, index) a pointer at cell (`x`, `y`) selects on a radial
/// chart (CHT-031): for a pie or donut, the drawn slice covering most of the
/// cell's samples within its drawn angles, taken where the fill took them,
/// so a cell that shows a slice selects it and a pad's gap selects nothing;
/// for a radar, the category of the spoke nearest the cell's angle. A cell
/// with no sample inside the outer radius, or only samples in a donut's
/// hole, selects nothing.
fn radial_pick(
    radial: &RadialHit,
    props: &ChartProps,
    x: usize,
    y: usize,
) -> Option<(usize, usize)> {
    use super::mask::{DOTS_X, DOTS_Y};
    use std::f64::consts::TAU;
    let (pw, ph) = (
        radial.samples.0.max(1) as usize,
        radial.samples.1.max(1) as usize,
    );
    let polar = |px: f64, py: f64| {
        let (dx, dy) = (px - radial.center.0, py - radial.center.1);
        (dx.hypot(dy), dx.atan2(-dy).rem_euclid(TAU))
    };
    let samples: Vec<(f64, f64)> = (0..ph)
        .flat_map(|sy| {
            (0..pw).map(move |sx| {
                polar(
                    (x as f64 + (sx as f64 + 0.5) / pw as f64) * DOTS_X as f64,
                    (y as f64 + (sy as f64 + 0.5) / ph as f64) * DOTS_Y as f64,
                )
            })
        })
        .collect();
    if !radial.slices.is_empty() {
        let mut counts: Vec<((usize, usize), usize)> = Vec::new();
        for (r, angle) in &samples {
            if *r > radial.outer || *r < radial.inner {
                continue;
            }
            if let Some(slice) = radial
                .slices
                .iter()
                .find(|s| *angle >= s.start && *angle < s.end)
            {
                match counts.iter_mut().find(|(key, _)| *key == slice.key) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((slice.key, 1)),
                }
            }
        }
        return counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(key, _)| key);
    }
    if !samples.iter().any(|(r, _)| *r <= radial.outer) {
        return None;
    }
    let (_, angle) = polar(
        (x as f64 + 0.5) * DOTS_X as f64,
        (y as f64 + 0.5) * DOTS_Y as f64,
    );
    let gap = |spoke: f64| {
        let d = (spoke - angle).rem_euclid(TAU);
        d.min(TAU - d)
    };
    let index = (0..radial.spokes.len())
        .min_by(|a, b| gap(radial.spokes[*a]).total_cmp(&gap(radial.spokes[*b])))?;
    Some((first_series_with(props, index)?, index))
}

/// The node a pointer at cell (`x`, `y`) selects on a Sankey chart
/// (CHT-032): the node covering most of the cell's samples, taken where the
/// fill took them, so a cell that shows a node selects it and a cell over a
/// ribbon or empty space selects nothing.
fn sankey_pick(hit: &canvas::SankeyHit, x: usize, y: usize) -> Option<(usize, usize)> {
    use super::mask::{DOTS_X, DOTS_Y};
    let (pw, ph) = (hit.samples.0.max(1) as usize, hit.samples.1.max(1) as usize);
    let mut counts = vec![0usize; hit.nodes.len()];
    for sy in 0..ph {
        for sx in 0..pw {
            let px = (x as f64 + (sx as f64 + 0.5) / pw as f64) * DOTS_X as f64;
            let py = (y as f64 + (sy as f64 + 0.5) / ph as f64) * DOTS_Y as f64;
            if let Some(node) = hit
                .nodes
                .iter()
                .position(|(x0, y0, x1, y1)| px >= *x0 && px < *x1 && py >= *y0 && py < *y1)
            {
                counts[node] += 1;
            }
        }
    }
    counts
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .max_by_key(|(_, n)| **n)
        .map(|(node, _)| (0, node))
}

/// The cells of the ray that marks a radial selection outside the tooltip
/// (CHT-018, CHT-031): from the inner radius to the outer one along the
/// selected slice's middle angle, or along the selected radar spoke.
fn radial_ray(radial: &RadialHit, selected: (usize, usize)) -> Vec<(usize, usize)> {
    use super::mask::{DOTS_X, DOTS_Y};
    let angle = if radial.slices.is_empty() {
        radial.spokes.get(selected.1).copied()
    } else {
        radial
            .slices
            .iter()
            .find(|s| s.key == selected)
            .map(|s| (s.start + s.end) / 2.0)
    };
    let Some(angle) = angle else {
        return Vec::new();
    };
    let mut cells = Vec::new();
    let mut r = radial.inner;
    while r <= radial.outer {
        let x = radial.center.0 + angle.sin() * r;
        let y = radial.center.1 - angle.cos() * r;
        if x >= 0.0 && y >= 0.0 {
            let cell = ((x / DOTS_X as f64) as usize, (y / DOTS_Y as f64) as usize);
            if cells.last() != Some(&cell) {
                cells.push(cell);
            }
        }
        r += 1.0;
    }
    cells
}

/// The first visible series that has a point at `index`.
fn first_series_with(props: &ChartProps, index: usize) -> Option<usize> {
    props
        .series
        .iter()
        .enumerate()
        .find(|(_, s)| s.visible && s.data.len() > index)
        .map(|(i, _)| i)
}

/// A laid-out tooltip with its crosshair column, ready to draw over cells.
/// The mini class has no box (shapes only, CHT-024); the hovered value is
/// still spoken through the description (CHT-018).
struct Overlay {
    boxed: Option<plot::TooltipBox>,
    at: (usize, usize),
    crosshair: Option<(usize, usize, usize)>,
    band: Option<(usize, usize, usize)>,
    /// A radial selection's ray from the center, in cells.
    ray: Vec<(usize, usize)>,
}

impl LiveChart {
    /// The chart's size in cells: the props' size where set, else the
    /// allotted rectangle.
    fn size(&self) -> (usize, usize) {
        let (w, h) = self.viewport.map_or((0.0, 0.0), |v| v.content_size());
        let (w, h) = (w.max(0.0) as usize, h.max(0.0) as usize);
        (
            if self.config.width == 0 {
                w
            } else {
                w.min(self.config.width as usize)
            },
            if self.config.height == 0 {
                h
            } else {
                h.min(self.config.height as usize)
            },
        )
    }

    /// The (series, index) nearest to cell (`x`, `y`): nearest on the
    /// category axis, or in both axes for scatter charts (CHT-019).
    fn nearest(&self, props: &ChartProps, x: usize, y: usize) -> Option<(usize, usize)> {
        let latest = self.latest.lock().unwrap_or_else(|e| e.into_inner());
        let picture = at_size(latest.picture.as_ref(), self.size())?;
        if let Some(radial) = &picture.radial {
            return radial_pick(radial, props, x, y);
        }
        if let Some(hit) = &picture.sankey {
            return sankey_pick(hit, x, y);
        }
        if !picture.plot.contains(x, y) && picture.anchors.is_empty() {
            return None;
        }
        if picture.scatter || (picture.index_columns.is_empty() && picture.index_rows.is_empty()) {
            return picture
                .anchors
                .iter()
                .min_by_key(|(_, (ax, ay))| {
                    let dx = ax.abs_diff(x) as u64;
                    let dy = ay.abs_diff(y) as u64;
                    dx * dx + 4 * dy * dy
                })
                .map(|(key, _)| *key);
        }
        if !picture.plot.contains(x, y) {
            return None;
        }
        let index = if picture.index_rows.is_empty() {
            nearest_index(&picture.index_columns, x as f64 + 0.5)
        } else {
            nearest_index(&picture.index_rows, y as f64 + 0.5)
        }?;
        let series = picture
            .anchors
            .keys()
            .filter(|(_, i)| *i == index)
            .map(|(s, _)| *s)
            .min()
            .or_else(|| first_series_with(props, index))?;
        Some((series, index))
    }

    /// The tooltip rows for `index` and their one-line announcement.
    fn tooltip_rows(&self, props: &ChartProps, index: usize) -> (Tooltip, String) {
        let mut rows = Vec::new();
        let mut spoken = Vec::new();
        for (s, series) in props.series.iter().enumerate().filter(|(_, s)| s.visible) {
            if let Some(point) = series.data.get(index) {
                let value = canvas::point_text(point, index);
                spoken.push(format!("{} / {value}", series.name));
                rows.push(TooltipRow {
                    color: canvas::point_color(props, s, index),
                    name: series.name.clone(),
                    value,
                });
            }
        }
        (Tooltip { title: None, rows }, spoken.join("; "))
    }

    fn tooltip_text(&self, props: &ChartProps, series: usize, index: usize) -> Option<String> {
        let point = props
            .series
            .get(series)
            .filter(|s| s.visible)?
            .data
            .get(index)?;
        Some(format!(
            "{} / {}",
            props.series[series].name,
            canvas::point_text(point, index)
        ))
    }

    /// Lay the tooltip out beside the hovered point and return it with the
    /// text announced through the live region.
    fn tooltip(
        &self,
        props: &ChartProps,
        picture: &Picture,
        series: usize,
        index: usize,
    ) -> (Option<Overlay>, String) {
        // A pie or donut's tooltip names the selected slice alone, in its
        // own color; other charts list every series at the index.
        let slice = picture
            .radial
            .as_ref()
            .and_then(|radial| radial.slices.iter().find(|s| s.key == (series, index)));
        // A Sankey chart's tooltip names the selected node with its
        // throughput, in the node's color (CHT-032).
        // The picture can be older than the props, so an index taken from
        // it is looked up in the props, never assumed there.
        let node = picture
            .sankey
            .as_ref()
            .filter(|hit| series == 0 && index < hit.nodes.len())
            .and_then(|hit| canvas::sankey_node_text(props, index).map(|text| (hit, text)));
        let (tooltip, spoken) = match (node, slice) {
            (Some((hit, (name, value))), _) => {
                let spoken = format!("{name} / {value}");
                (
                    Tooltip {
                        title: None,
                        rows: vec![TooltipRow {
                            color: hit.colors.get(index).copied().flatten(),
                            name,
                            value,
                        }],
                    },
                    spoken,
                )
            }
            (None, Some(slice)) => {
                let Some(point) = props.series.get(series).and_then(|s| s.data.get(index)) else {
                    return (None, String::new());
                };
                let value = canvas::point_text(point, index);
                let spoken = format!("{} / {value}", props.series[series].name);
                (
                    Tooltip {
                        title: None,
                        rows: vec![TooltipRow {
                            color: slice.color,
                            name: props.series[series].name.clone(),
                            value,
                        }],
                    },
                    spoken,
                )
            }
            (None, None) => self.tooltip_rows(props, index),
        };
        if tooltip.rows.is_empty() {
            return (None, spoken);
        }
        let area = Rect::sized(picture.width, picture.height);
        let anchor = picture
            .anchors
            .get(&(series, index))
            .copied()
            .or_else(|| {
                picture
                    .index_columns
                    .get(index)
                    .map(|c| (*c as usize, picture.plot.y + picture.plot.h / 2))
            })
            .unwrap_or((picture.plot.x, picture.plot.y));
        let boxed = tooltip.layout(area);
        let at = boxed.place(anchor, area);
        let boxed = (picture.class != Some(plot::SizeClass::Mini)).then_some(boxed);
        let crosshair = (!picture.scatter
            && picture.index_rows.is_empty()
            && !picture.index_columns.is_empty())
        .then(|| (anchor.0, picture.plot.y, picture.plot.bottom()));
        let band = (!picture.index_rows.is_empty())
            .then(|| (anchor.1, picture.plot.x, picture.plot.right()));
        let ray = picture
            .radial
            .as_ref()
            .map_or_else(Vec::new, |radial| radial_ray(radial, (series, index)));
        (
            Some(Overlay {
                boxed,
                at,
                crosshair,
                band,
                ray,
            }),
            spoken,
        )
    }

    /// The accessibility description: title, series names and the hovered
    /// values, at every size class (CHT-018).
    fn description(&self, props: &ChartProps, announcement: &str) -> String {
        let names: Vec<&str> = props
            .series
            .iter()
            .filter(|s| s.visible)
            .map(|s| s.name.as_str())
            .collect();
        let mut text = props.title.clone().unwrap_or_else(|| "Chart".to_string());
        if !names.is_empty() {
            text.push_str(&format!("; series {}", names.join(", ")));
        }
        if !announcement.is_empty() {
            text.push_str(&format!("; selected {announcement}"));
        }
        text
    }
}

fn nearest_index(positions: &[f64], at: f64) -> Option<usize> {
    positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (*a - at)
                .abs()
                .partial_cmp(&(*b - at).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// A copy of the picture's grid with the tooltip box, crosshair and band
/// drawn over it.
fn overlay_grid(picture: &Picture, overlay: &Overlay) -> CellGrid {
    struct Sink<'a>(&'a mut CellGrid);
    impl plot::TextSink for Sink<'_> {
        fn text(
            &mut self,
            x: usize,
            y: usize,
            width: usize,
            text: &str,
            color: Option<plot::Rgba>,
        ) {
            use unicode_segmentation::UnicodeSegmentation;
            for (offset, grapheme) in text.graphemes(true).enumerate() {
                if offset >= width {
                    break;
                }
                let (Ok(x), Ok(y)) = (u16::try_from(x + offset), u16::try_from(y)) else {
                    break;
                };
                self.0.set(x, y, grapheme, color);
            }
        }
        fn under(&mut self, x: usize, y: usize, glyph: &str, color: Option<plot::Rgba>) {
            let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
                return;
            };
            let free = self
                .0
                .get(x, y)
                .is_some_and(|(current, _)| current.is_empty() || current == "·");
            if free {
                self.0.set(x, y, glyph, color);
            }
        }
    }
    use plot::TextSink as _;
    let mut grid = (*picture.grid).clone();
    let mut sink = Sink(&mut grid);
    if let Some((col, top, bottom)) = overlay.crosshair {
        for row in top..bottom {
            sink.under(col, row, "│", None);
        }
    }
    if let Some((row, left, right)) = overlay.band {
        for col in left..right {
            sink.under(col, row, "─", None);
        }
    }
    // A radial selection's ray is drawn over the shapes it crosses.
    for (col, row) in &overlay.ray {
        sink.text(*col, *row, 1, "·", None);
    }
    if let Some(boxed) = &overlay.boxed {
        boxed.draw(
            &mut sink,
            overlay.at,
            Rect::sized(picture.width, picture.height),
            None,
        );
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture drawn from older props, with more nodes or slices than the
    /// props now hold, never makes the tooltip index past the new data.
    #[test]
    fn a_picture_from_older_props_never_indexes_past_the_new_data() {
        let draw = |props: &ChartProps| {
            let values: Vec<Vec<f64>> = props
                .series
                .iter()
                .map(|s| s.data.iter().map(|p| p.value).collect())
                .collect();
            canvas::draw(&canvas::Job {
                props,
                width: 60,
                height: 16,
                values: &values,
                progress: 1.0,
                unicode_glyphs: true,
                selected: None,
            })
        };
        let nodes = |count: usize| {
            DataSeries::new(
                "nodes",
                (0..count)
                    .map(|i| DataPoint::with_label(0.0, format!("n{i}")))
                    .collect(),
            )
        };
        let sankey = |count: usize| {
            ChartsBuilder::sankey()
                .series(nodes(count))
                .sankey_options(SankeyOptions {
                    links: (1..count).map(|i| SankeyLink::new(i - 1, i, 1.0)).collect(),
                    ..SankeyOptions::default()
                })
                .build()
        };
        let pie = |count: usize| {
            ChartsBuilder::pie()
                .series(DataSeries::new(
                    "slices",
                    (0..count).map(|_| DataPoint::new(1.0)).collect(),
                ))
                .build()
        };
        for (old, new, kind) in [(sankey(4), sankey(2), "Sankey"), (pie(4), pie(1), "pie")] {
            let picture = draw(&old);
            let chart = LiveChart::new(LiveProps {
                config: Arc::new(new.clone()),
                seed: ChartState::default(),
            });
            let (overlay, spoken) = chart.tooltip(&new, &picture, 0, 3);
            assert!(
                overlay.is_none() && spoken.is_empty(),
                "{kind}: an index from the older picture selects nothing in the new props"
            );
        }
    }

    /// A NaN value equals itself in the job key, so a chart holding invalid
    /// data does not resubmit a worker job every frame.
    #[test]
    fn a_job_key_with_nan_equals_its_clone() {
        let key = JobKey {
            version: 1,
            width: 30,
            height: 12,
            values: Arc::new(vec![vec![f64::NAN, 5.0]]),
            progress: 1.0,
            selected: None,
        };
        let same = JobKey {
            values: Arc::new(vec![vec![f64::NAN, 5.0]]),
            ..key.clone()
        };
        let other = JobKey {
            values: Arc::new(vec![vec![4.0, 5.0]]),
            ..key.clone()
        };
        assert!(key == key.clone(), "the same allocation compares equal");
        assert!(
            key == same,
            "equal bit patterns compare equal across allocations"
        );
        assert!(key != other, "different values differ");
    }
}
