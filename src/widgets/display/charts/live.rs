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

use canvas::{Cell, Picture, Run};

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub config: ChartProps,
    pub seed: ChartState,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// What the last submitted job was drawn from, to avoid resubmitting.
#[derive(Clone, PartialEq)]
struct JobKey {
    version: u64,
    width: usize,
    height: usize,
    values: Arc<Vec<Vec<f64>>>,
    progress: f64,
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
            config: Arc::new(props.config),
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
        if *self.config != props.config {
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
            self.config = Arc::new(props.config.clone());
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
        if valid && !latest.shown {
            self.reveal.restart();
            latest.shown = true;
        } else if !valid {
            latest.shown = false;
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
        let key = JobKey {
            version: self.version,
            width,
            height,
            values: Arc::new(values),
            progress,
        };
        if latest.key.as_ref() != Some(&key) {
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
                });
                // A picture at a new size is worth a short wait so the chart
                // never paints empty or stale for long; frames at the same
                // size (animation, hover) copy whatever is finished.
                let wait = if latest
                    .picture
                    .as_ref()
                    .is_none_or(|p| (p.width, p.height) != (width, height))
                {
                    Duration::from_millis(60)
                } else {
                    Duration::ZERO
                };
                if let Some((_, picture)) = worker.wait_for(id, wait) {
                    latest.picture = Some(picture);
                }
            }
            latest.key = Some(key);
        } else if let Some((_, picture)) = self.worker.as_ref().and_then(|w| w.latest()) {
            latest.picture = Some(picture);
        }
        let picture = latest.picture.clone();
        drop(latest);

        let hovered = state.hovered_point.filter(|_| config.show_tooltips);
        let (tooltip, announcement) = picture
            .as_ref()
            .zip(hovered)
            .map(|(picture, (series, index))| self.tooltip(&config, picture, series, index))
            .unwrap_or((None, String::new()));
        let mut children = picture
            .as_ref()
            .map(|picture| {
                let runs = match &tooltip {
                    Some(overlay) => overlay_runs(picture, overlay),
                    None => picture.runs.clone(),
                };
                canvas::elements(&runs)
            })
            .unwrap_or_default();
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .children(std::mem::take(&mut children))
            .build();
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
struct Overlay {
    boxed: plot::TooltipBox,
    at: (usize, usize),
    crosshair: Option<(usize, usize, usize)>,
    band: Option<(usize, usize, usize)>,
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
        let picture = latest.picture.as_ref()?;
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
        let (tooltip, spoken) = self.tooltip_rows(props, index);
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
        let crosshair = (!picture.scatter
            && picture.index_rows.is_empty()
            && !picture.index_columns.is_empty())
        .then(|| (anchor.0, picture.plot.y, picture.plot.bottom()));
        let band = (!picture.index_rows.is_empty())
            .then(|| (anchor.1, picture.plot.x, picture.plot.right()));
        (
            Some(Overlay {
                boxed,
                at,
                crosshair,
                band,
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

/// The picture's runs with the tooltip box, crosshair and band drawn over
/// the rows they touch; untouched rows reuse the worker's runs.
fn overlay_runs(picture: &Picture, overlay: &Overlay) -> Vec<Run> {
    struct Sink<'a> {
        cells: &'a mut [Cell],
        width: usize,
        height: usize,
    }
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
                if offset >= width || x + offset >= self.width || y >= self.height {
                    break;
                }
                self.cells[y * self.width + x + offset] = Cell {
                    text: grapheme.to_string(),
                    color,
                    owner: None,
                };
            }
        }
        fn under(&mut self, x: usize, y: usize, glyph: &str, color: Option<plot::Rgba>) {
            if x < self.width && y < self.height {
                let cell = &mut self.cells[y * self.width + x];
                if cell.text.trim().is_empty() || cell.text == "·" {
                    cell.text = glyph.to_string();
                    cell.color = color;
                }
            }
        }
    }
    use plot::TextSink as _;
    let mut cells = picture.cells.clone();
    let mut sink = Sink {
        cells: &mut cells,
        width: picture.width,
        height: picture.height,
    };
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
    overlay.boxed.draw(
        &mut sink,
        overlay.at,
        Rect::sized(picture.width, picture.height),
        None,
    );
    canvas::runs(&cells, picture.width, picture.height)
}
