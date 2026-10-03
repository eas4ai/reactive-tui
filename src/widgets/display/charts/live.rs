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

#[cfg(feature = "wgpu-graphics")]
use crate::graphics::{Canvas, CanvasOutput, CanvasProps, HostReport};
use crate::layout::paint_tree::cells::CellGrid;
use canvas::{HoverMarks, Picture, PlotPixels, RadialHit};

/// How long a chart that has just appeared or changed size, or a Sankey
/// chart whose selection changed, waits for its picture before the frame
/// goes out with the previous one or none. The wait is main-thread
/// time inside the App's frame (BAR-005). Charts up to 200 by 40 cells
/// draw in under 1 ms and appear painted; a larger one paints an empty
/// area for one frame, and the worker's finish signal redraws it.
const NEW_SIZE_WAIT: Duration = Duration::from_millis(4);
/// How long a frame waits for the repaint after a theme change, so the next
/// frame presented is in the new theme's colors (THM-003); a picture that
/// takes longer shows up on the frame after, and the chart reads as busy.
const THEME_WAIT: Duration = Duration::from_millis(200);

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

/// The picture part of a job's key: the cell size, and the hover's series,
/// index, focus bits and band bits, so every frame of a glide or an
/// ease-in is its own picture (CHT-038).
type PixelsKey = Option<((u16, u16), Option<(usize, usize, u32, u64)>)>;

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
    /// How many themes had been set when the picture was drawn: colors
    /// resolve on the worker, so a new theme needs a new picture (THM-003).
    theme: u64,
    /// The plot as a picture: the cell size and the hover marks, as bits,
    /// so every frame of the hover's motion is a job of its own (CHT-037,
    /// CHT-038).
    pixels: PixelsKey,
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
            && self.theme == other.theme
            && self.pixels == other.pixels
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
    /// Why the worker could not start, shown in the chart's area (CHT-026).
    worker_error: Option<String>,
    latest: Mutex<Latest>,
    reveal: motion::Reveal,
    transition: motion::Transition,
    /// The hover's motion in a plot picture (CHT-038).
    hover: motion::Hover,
}

/// Per-series values from the props, the transition's target. A candle
/// series carries open, high, low and close for every point, four values
/// per point in that order, so the reveal and the transitions move whole
/// candles (CHT-014, CHT-022); the renderer reads them back per point.
fn target_values(props: &ChartProps) -> Vec<Vec<f64>> {
    props
        .series
        .iter()
        .map(|s| {
            if s.data.iter().any(|p| p.candle.is_some()) {
                s.data
                    .iter()
                    .flat_map(|p| {
                        p.candle
                            .map_or([p.value; 4], |c| [c.open, c.high, c.low, c.close])
                    })
                    .collect()
            } else {
                s.data.iter().map(|p| p.value).collect()
            }
        })
        .collect()
}

impl Component for LiveChart {
    type Props = LiveProps;
    type State = ChartState;

    fn new(props: Self::Props) -> Self {
        let (worker, worker_error) = match worker::Worker::new() {
            Ok(worker) => (Some(worker), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Self {
            viewport: None,
            config: props.config,
            version: 0,
            seed: props.seed,
            worker,
            worker_error,
            latest: Mutex::new(Latest {
                key: None,
                next_id: 0,
                picture: None,
                shown: false,
            }),
            reveal: motion::Reveal::new(),
            transition: motion::Transition::new(),
            hover: motion::Hover::new(),
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
        // A new cell size, or a terminal that now takes pixels, redraws the
        // plot picture (CHT-037).
        let changed = self.viewport.is_none_or(|old| {
            old.content_size() != layout.content_size() || old.terminal != layout.terminal
        });
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
        let (values, _) = self.transition.values(&config, &target);
        let transition = self
            .transition
            .in_progress()
            .map(|(t, from)| (t, Arc::new(from)));
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
        // other charts patch it over the finished picture, unless the plot
        // is a picture, which draws its hover marks itself (CHT-038).
        let selected = state
            .hovered_point
            .filter(|_| config.show_tooltips && config.chart_type == ChartType::Sankey);
        let cell = self.plot_pixels(&config);
        let hovered = state.hovered_point.filter(|_| config.show_tooltips);
        // The hovered datum with its band's center in cells, from the
        // latest picture of this size; none until that picture exists.
        let target = hovered.and_then(|(series, index)| {
            let picture = at_size(latest.picture.as_ref(), (width, height))?;
            let center = if picture.index_rows.is_empty() {
                picture.index_columns.get(index)
            } else {
                picture.index_rows.get(index)
            };
            Some((series, index, center.copied().unwrap_or(0.0)))
        });
        let marks = match cell {
            Some(_) => self.hover.frame(&config, target),
            None => {
                self.hover.frame(&config, None);
                None
            }
        };
        let pixels = cell.map(|cell| PlotPixels {
            cell,
            hover: marks.map(|m| HoverMarks {
                series: m.series,
                index: m.index,
                focus: m.focus,
                band: m.band,
            }),
        });
        let key = JobKey {
            version: self.version,
            width,
            height,
            values: Arc::new(values),
            progress,
            selected,
            theme: crate::theme::Theme::generation(),
            pixels: pixels.map(|p| {
                (
                    p.cell,
                    p.hover
                        .map(|h| (h.series, h.index, h.focus.to_bits(), h.band.to_bits())),
                )
            }),
        };
        // Before its first layout the chart has no size and nothing to draw.
        let drawable = width > 0 && height > 0;
        let theme = key.theme;
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
                    theme,
                    transition: transition.clone(),
                    pixels,
                });
                // A picture at a new size, or a Sankey chart's new
                // selection, is worth a short wait so a small chart never
                // paints empty and the faded links arrive with the tooltip.
                // A theme change waits for the repaint itself, since the
                // next frame presented must be in the new colors (THM-003)
                // and the picture at this size was drawn under the old ones.
                // Other frames at the same size (animation, hover) copy
                // whatever is finished.
                let theme_changed = latest.key.as_ref().is_some_and(|k| k.theme != theme);
                let fresh = !theme_changed
                    && at_size(latest.picture.as_ref(), (width, height))
                        .is_some_and(|picture| picture.selected == selected);
                let wait = if theme_changed {
                    THEME_WAIT
                } else if fresh {
                    Duration::ZERO
                } else {
                    NEW_SIZE_WAIT
                };
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

        let (tooltip, announcement) = picture
            .as_ref()
            .zip(hovered)
            .map(|(picture, (series, index))| self.tooltip(&config, picture, series, index))
            .unwrap_or((None, String::new()));
        // The whole picture is one element: the painter blits its cell grid
        // (BAR-005). In cells, the tooltip and the crosshair are patched
        // into a copy; with the plot a picture, the hover marks are in the
        // picture and the tooltip joins the text inside the plot in a grid
        // painted over it (CHT-037, CHT-038).
        let ascii = config.ascii || !super::glyph_support();
        let in_picture = picture.as_ref().is_some_and(|picture| picture.has_scene());
        let grid = picture
            .as_ref()
            .map(|picture| match (&tooltip, in_picture) {
                (Some(overlay), false) => Arc::new(overlay_grid(
                    &picture.grid,
                    (picture.width, picture.height),
                    overlay,
                    ascii,
                    true,
                )),
                _ => picture.grid.clone(),
            });
        let over = picture.as_ref().filter(|_| in_picture).map(|picture| {
            let base = picture.over.clone().unwrap_or_else(|| {
                Arc::new(CellGrid::new(
                    u16::try_from(picture.width).unwrap_or(u16::MAX),
                    u16::try_from(picture.height).unwrap_or(u16::MAX),
                ))
            });
            match &tooltip {
                Some(overlay) => Arc::new(overlay_grid(
                    &base,
                    (picture.width, picture.height),
                    overlay,
                    ascii,
                    false,
                )),
                None => base,
            }
        });
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        // A chart whose worker could not start says so where its picture
        // would be, instead of staying blank (CHT-026).
        let message = self
            .worker_error
            .as_ref()
            .map(|error| format!("Chart worker could not start: {error}"))
            .unwrap_or_default();
        let mut content = ElementBuilder::new(ElementType::Text(message))
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
        let mut children = vec![content];
        // The plot picture: a canvas on the inner plot rectangle, drawn by
        // the drawing thread from the worker's scene, part of the chart for
        // the screen reader and the mouse (CHT-037).
        #[cfg(feature = "wgpu-graphics")]
        if let Some(scene) = picture
            .as_ref()
            .filter(|_| in_picture)
            .and_then(|picture| picture.scene.clone())
        {
            let inner = picture.as_ref().map_or(Rect::default(), |p| p.inner);
            let mut holder = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(insets[0] + inner.x as f32)
                        .inset_top(insets[1] + inner.y as f32)
                        .width_px(inner.w as f32)
                        .height_px(inner.h as f32)
                        .overflow_hidden(),
                )
                .children(vec![Element::typed::<Canvas>(
                    CanvasProps::new(scene)
                        .options(super::graphics_options())
                        .described_by_parent(),
                )])
                .build();
            holder.metadata.inert = true;
            children.push(holder);
        }
        // The text inside the plot, and the tooltip, over the picture.
        if let Some(over) = over {
            let mut element = ElementBuilder::new(ElementType::Text(String::new()))
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
                .build()
                .with_cells(over);
            element.metadata.inert = true;
            children.push(element);
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
            .children(children)
            .build();
        element.focus = Some(FocusProps::button());
        // The screen reader is told the chart's name from `aria_label`,
        // else its title, with no fixed English name; its type, series and
        // state in the description; the selection as its value; and each
        // visible series as a child (CHT-036).
        let mut node = crate::accessibility::Node::new(crate::accessibility::Role::Image);
        if let Some(name) = config.aria_label.as_deref().or(config.title.as_deref()) {
            node.set_label(name);
        }
        node.set_description(self.description(&config, &announcement));
        if !announcement.is_empty() {
            node.set_value(announcement.clone());
        }
        // The worker still owes the picture for this size, for a Sankey
        // chart's new selection, or under the theme now active (THM-003):
        // the chart reads as busy until it arrives (CHT-036), and the
        // worker's finish signal redraws the chart.
        let stale = picture
            .as_ref()
            .is_none_or(|picture| picture.selected != selected || picture.theme != theme);
        if stale && drawable && self.worker.is_some() {
            node.set_busy();
        }
        element.metadata.accessibility = Some(node);
        for (index, series) in config.series.iter().enumerate().filter(|(_, s)| s.visible) {
            let mut child = Element::text(format!("{}, {} points", series.name, series.data.len()))
                .with_key(format!("chart-series-{index}"))
                .class("sr-only");
            let mut item = crate::accessibility::Node::new(crate::accessibility::Role::ListItem);
            item.set_label(format!("{}, {} points", series.name, series.data.len()));
            if hovered.is_some_and(|(s, _)| s == index) {
                item.set_selected(true);
            }
            child.metadata.accessibility = Some(item);
            element = element.with_child(child);
        }
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
                            | KeyCode::Up
                            | KeyCode::Down
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
                let radial = matches!(
                    props.config.chart_type,
                    ChartType::Pie | ChartType::Donut | ChartType::Sankey
                );
                if matches!(key.code, KeyCode::Up | KeyCode::Down) {
                    if radial {
                        return EventResult::Ignored;
                    }
                    // Up and Down choose the series the selection follows,
                    // keeping the index (CHT-036).
                    let visible: Vec<usize> = props
                        .config
                        .series
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| s.visible && !s.data.is_empty())
                        .map(|(i, _)| i)
                        .collect();
                    if visible.is_empty() {
                        return EventResult::Ignored;
                    }
                    let (series, index) = state.hovered_point.unwrap_or((visible[0], 0));
                    let at = visible.iter().position(|s| *s == series).unwrap_or(0);
                    let next = if key.code == KeyCode::Down {
                        (at + 1).min(visible.len() - 1)
                    } else {
                        at.saturating_sub(1)
                    };
                    let series = visible[next];
                    let last = props.config.series[series].data.len() - 1;
                    state.hovered_point = Some((series, index.min(last)));
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
                // The keys walk the followed series (CHT-036): its points in x
                // order on a scatter with numeric x, by index otherwise; with
                // no series followed yet, the first that has the index.
                let followed = state
                    .hovered_point
                    .map(|(s, _)| s)
                    .filter(|s| props.config.series.get(*s).is_some_and(|d| d.visible));
                let order: Vec<usize> = match followed {
                    Some(s) => {
                        let data = &props.config.series[s].data;
                        let mut order: Vec<usize> = (0..data.len()).collect();
                        if data.iter().any(|p| p.x.is_some()) {
                            order.sort_by(|a, b| {
                                let x = |i: usize| data[i].x.unwrap_or(i as f64);
                                x(*a).total_cmp(&x(*b))
                            });
                        }
                        order
                    }
                    None => (0..count).collect(),
                };
                if order.is_empty() {
                    return EventResult::Ignored;
                }
                let last = order.len() - 1;
                let current = state
                    .hovered_point
                    .and_then(|(_, p)| order.iter().position(|i| *i == p));
                let at = match key.code {
                    KeyCode::Home => 0,
                    KeyCode::End => last,
                    KeyCode::Left => current.unwrap_or(1).saturating_sub(1),
                    _ => current.map_or(0, |i| (i + 1).min(last)),
                };
                let index = order[at];
                let series = followed
                    .or_else(|| first_series_with(&props.config, index))
                    .unwrap_or(0);
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
            self.hover.cancel();
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
    /// The cell of a scatter's selected point, ringed (CHT-018).
    ring: Option<(usize, usize)>,
    /// A radial selection's ray from the center, in cells.
    ray: Vec<(usize, usize)>,
}

impl LiveChart {
    /// The pixels of a cell when the plot is drawn as a picture (CHT-037):
    /// the chart is cartesian, the terminal takes Kitty graphics or Sixel
    /// as the backend learned at startup, and neither the environment nor
    /// the application's graphics options switched the plots back to cells.
    #[cfg(feature = "wgpu-graphics")]
    fn plot_pixels(&self, props: &ChartProps) -> Option<(u16, u16)> {
        if !matches!(
            props.chart_type,
            ChartType::Line
                | ChartType::Area
                | ChartType::Scatter
                | ChartType::BarVertical
                | ChartType::BarHorizontal
                | ChartType::Candlestick
        ) {
            return None;
        }
        let terminal = self.viewport?.terminal;
        let cell = terminal.cell_pixels?;
        let host = HostReport {
            kitty: terminal.kitty_graphics,
            kitty_shared_memory: terminal.kitty_shared_memory,
            sixel: terminal.sixel,
            cell,
        };
        let options = super::graphics_options();
        match CanvasOutput::choose(Some(host), options.output, CanvasOutput::from_environment()) {
            CanvasOutput::Kitty | CanvasOutput::Sixel => Some((cell.0.max(1), cell.1.max(1))),
            CanvasOutput::Blocks => None,
        }
    }

    /// Without the graphics feature every plot is drawn in cells.
    #[cfg(not(feature = "wgpu-graphics"))]
    fn plot_pixels(&self, _: &ChartProps) -> Option<(u16, u16)> {
        None
    }

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

    /// The title row of a point's tooltip: the point's own title, else its
    /// label, else its numeric x, else its index (CHT-018).
    fn point_title(props: &ChartProps, series: usize, index: usize) -> String {
        let point = props.series.get(series).and_then(|s| s.data.get(index));
        point
            .and_then(|p| p.tooltip.title.clone())
            .or_else(|| point.and_then(|p| p.label.clone()))
            .or_else(|| point.and_then(|p| p.x).map(plot::format_tick))
            .unwrap_or_else(|| index.to_string())
    }

    /// A point's value text: its own tooltip value, else the value, with
    /// any metadata after it (CHT-018).
    fn point_value(point: &DataPoint) -> String {
        let mut text = point
            .tooltip
            .value
            .clone()
            .unwrap_or_else(|| point.value.to_string());
        let mut metadata: Vec<_> = point.metadata.iter().collect();
        metadata.sort();
        for (key, value) in metadata {
            text.push_str(&format!("; {key}={value}"));
        }
        text
    }

    /// The rows of a point's tooltip and their announcement (CHT-018): a
    /// title row naming the category, then one swatch, name and value row
    /// per visible series at `index`, the followed `series` first; a candle
    /// as four rows, open, high, low and close; a point's own content lines
    /// in place of its row when it has them.
    fn tooltip_rows(&self, props: &ChartProps, series: usize, index: usize) -> (Tooltip, String) {
        let title = Self::point_title(props, series, index);
        let mut rows = Vec::new();
        let mut spoken = Vec::new();
        let order = std::iter::once(series).chain((0..props.series.len()).filter(|s| *s != series));
        for s in order {
            let Some(data) = props.series.get(s).filter(|d| d.visible) else {
                continue;
            };
            let Some(point) = data.data.get(index) else {
                continue;
            };
            let swatch = canvas::point_color(props, s, index);
            let value_color = point.tooltip.color.as_deref().and_then(canvas::color);
            if !point.tooltip.lines.is_empty() {
                for line in &point.tooltip.lines {
                    rows.push(TooltipRow {
                        color: None,
                        name: String::new(),
                        value: line.clone(),
                        value_color,
                        numeric: None,
                    });
                }
                spoken.push(format!(
                    "{} / {}",
                    data.name,
                    point.tooltip.lines.join(", ")
                ));
                continue;
            }
            if let (Some(candle), None) = (point.candle, &point.tooltip.value) {
                for (name, value) in [
                    ("open", candle.open),
                    ("high", candle.high),
                    ("low", candle.low),
                    ("close", candle.close),
                ] {
                    rows.push(TooltipRow {
                        color: swatch,
                        name: name.into(),
                        value: value.to_string(),
                        value_color,
                        numeric: Some(value),
                    });
                }
                spoken.push(format!(
                    "{} / open {} high {} low {} close {}",
                    data.name, candle.open, candle.high, candle.low, candle.close
                ));
                continue;
            }
            let value = Self::point_value(point);
            spoken.push(format!("{} / {value}", data.name));
            rows.push(TooltipRow {
                color: swatch,
                name: data.name.clone(),
                value,
                value_color,
                numeric: Some(point.value),
            });
        }
        let spoken = if spoken.is_empty() {
            String::new()
        } else {
            format!("{title}: {}", spoken.join("; "))
        };
        (
            Tooltip {
                title: Some(title),
                rows,
                ascii: ascii_glyphs(props),
            },
            spoken,
        )
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
            Self::point_value(point)
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
        // A Sankey node spans columns, so its box goes beside the whole node.
        let node_left = node
            .as_ref()
            .map(|(hit, _)| (hit.nodes[index].0 / super::mask::DOTS_X as f64) as usize);
        let (tooltip, spoken) = match (node, slice) {
            (Some((hit, (name, value))), _) => {
                let spoken = format!("{name} / {value}");
                (
                    Tooltip {
                        title: None,
                        ascii: ascii_glyphs(props),
                        rows: vec![TooltipRow {
                            color: hit.colors.get(index).copied().flatten(),
                            name,
                            value,
                            value_color: None,
                            // One row: nothing to summarize.
                            numeric: None,
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
                        ascii: ascii_glyphs(props),
                        rows: vec![TooltipRow {
                            color: slice.color,
                            name: props.series[series].name.clone(),
                            value,
                            value_color: None,
                            numeric: Some(point.value),
                        }],
                    },
                    spoken,
                )
            }
            (None, None) => self.tooltip_rows(props, series, index),
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
        let at = match node_left {
            Some(left) => boxed.place_beside((left, anchor.0), anchor.1, area),
            None => boxed.place(anchor, area),
        };
        let boxed = (picture.class != Some(plot::SizeClass::Mini)).then_some(boxed);
        let crosshair = (!picture.scatter
            && picture.index_rows.is_empty()
            && !picture.index_columns.is_empty())
        .then(|| (anchor.0, picture.plot.y, picture.plot.bottom()));
        let band = (!picture.index_rows.is_empty())
            .then(|| (anchor.1, picture.plot.x, picture.plot.right()));
        // A scatter marks its selected point with a ring (CHT-018).
        let ring = picture
            .scatter
            .then(|| picture.anchors.get(&(series, index)).copied())
            .flatten();
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
                ring,
                ray,
            }),
            spoken,
        )
    }

    /// The accessibility description: the chart's type, its series and its
    /// state, with the selection at every size class (CHT-018, CHT-036).
    fn description(&self, props: &ChartProps, announcement: &str) -> String {
        let kind = match props.chart_type {
            ChartType::BarVertical | ChartType::BarHorizontal => "bar chart",
            ChartType::Line => "line chart",
            ChartType::Area => "area chart",
            ChartType::Scatter => "scatter chart",
            ChartType::Candlestick => "candlestick chart",
            ChartType::Pie => "pie chart",
            ChartType::Donut => "donut chart",
            ChartType::Radar => "radar chart",
            ChartType::Sankey => "sankey chart",
        };
        let names: Vec<&str> = props
            .series
            .iter()
            .filter(|s| s.visible)
            .map(|s| s.name.as_str())
            .collect();
        let mut text = kind.to_string();
        if let Some(error) = &self.worker_error {
            text.push_str(&format!("; chart worker could not start: {error}"));
        } else if let Err(error) = canvas::validate(props) {
            text.push_str(&format!("; {error}"));
        } else if props
            .series
            .iter()
            .filter(|s| s.visible)
            .all(|s| s.data.is_empty())
        {
            text.push_str("; no data");
        }
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

/// Whether the chart draws with ASCII glyphs only: forced by the builder or
/// because the terminal lacks braille and block glyphs (CHT-028).
fn ascii_glyphs(props: &ChartProps) -> bool {
    props.ascii || !super::glyph_support()
}

/// A copy of the picture's grid with the tooltip box, crosshair, band and
/// ring drawn over it, in the chrome roles (CHT-017) and in ASCII when the
/// chart is (CHT-028).
/// `base` with `overlay` drawn into a copy: the tooltip box always, and the
/// crosshair, band, ring and ray when `marks`, which a plot picture draws
/// itself (CHT-038). `size` is the chart's size in cells.
fn overlay_grid(
    base: &CellGrid,
    size: (usize, usize),
    overlay: &Overlay,
    ascii: bool,
    marks: bool,
) -> CellGrid {
    use unicode_width::UnicodeWidthStr;
    /// Writes into the grid; a filled rectangle keeps its background under
    /// whatever is written into it afterwards, so a box stays opaque.
    struct Sink<'a> {
        grid: &'a mut CellGrid,
        fills: Vec<(usize, usize, usize, usize, Option<plot::Rgba>)>,
    }
    impl Sink<'_> {
        fn background_at(&self, x: usize, y: usize) -> Option<plot::Rgba> {
            self.fills
                .iter()
                .rev()
                .find(|(fx, fy, w, h, _)| x >= *fx && x < fx + w && y >= *fy && y < fy + h)
                .and_then(|fill| fill.4)
        }
        fn put(&mut self, x: usize, y: usize, glyph: &str, color: Option<plot::Rgba>) {
            let (Ok(cx), Ok(cy)) = (u16::try_from(x), u16::try_from(y)) else {
                return;
            };
            match self
                .background_at(x, y)
                .or_else(|| self.grid.background(cx, cy))
            {
                Some(bg) => self
                    .grid
                    .set_with_background(cx, cy, glyph, color, Some(bg)),
                None => self.grid.set(cx, cy, glyph, color),
            }
        }
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
            // Each grapheme takes its cell width, so a wide character
            // neither overlaps its neighbour nor is cut (CHT-018).
            let mut offset = 0;
            for grapheme in text.graphemes(true) {
                let cells = UnicodeWidthStr::width(grapheme).max(1);
                if offset + cells > width {
                    break;
                }
                self.put(x + offset, y, grapheme, color);
                for continuation in 1..cells {
                    self.put(x + offset + continuation, y, "", color);
                }
                offset += cells;
            }
        }
        fn under(&mut self, x: usize, y: usize, glyph: &str, color: Option<plot::Rgba>) {
            let (Ok(cx), Ok(cy)) = (u16::try_from(x), u16::try_from(y)) else {
                return;
            };
            let free = self
                .grid
                .get(cx, cy)
                .is_none_or(|(current, _)| current.is_empty() || current == "·" || current == ".");
            if free {
                self.put(x, y, glyph, color);
            }
        }
        fn fill(
            &mut self,
            x: usize,
            y: usize,
            width: usize,
            height: usize,
            background: Option<plot::Rgba>,
        ) {
            self.fills.push((x, y, width, height, background));
            for row in y..y + height {
                for col in x..x + width {
                    let (Ok(cx), Ok(cy)) = (u16::try_from(col), u16::try_from(row)) else {
                        continue;
                    };
                    let (glyph, fg) =
                        self.grid
                            .get(cx, cy)
                            .map_or((String::from(" "), None), |(g, fg)| {
                                (
                                    if g.is_empty() {
                                        " ".into()
                                    } else {
                                        g.to_string()
                                    },
                                    fg,
                                )
                            });
                    self.grid
                        .set_with_background(cx, cy, &glyph, fg, background);
                }
            }
        }
    }
    use plot::TextSink as _;
    let mut grid = base.clone();
    let mut sink = Sink {
        grid: &mut grid,
        fills: Vec::new(),
    };
    let muted = canvas::color("text-muted");
    let (vertical, horizontal, ring_glyph, dot) = if ascii {
        ("|", "-", "o", ".")
    } else {
        ("│", "─", "◌", "·")
    };
    let crosshair = overlay.crosshair.filter(|_| marks);
    let band = overlay.band.filter(|_| marks);
    let ring = overlay.ring.filter(|_| marks);
    let ray: &[(usize, usize)] = if marks { &overlay.ray } else { &[] };
    if let Some((col, top, bottom)) = crosshair {
        for row in top..bottom {
            sink.under(col, row, vertical, muted);
        }
    }
    if let Some((row, left, right)) = band {
        // The band tints its row in the `hover` role and marks the empty
        // cells with a line.
        sink.fill(
            left,
            row,
            right.saturating_sub(left),
            1,
            canvas::color("hover"),
        );
        for col in left..right {
            sink.under(col, row, horizontal, muted);
        }
    }
    // A radial selection's ray is drawn over the shapes it crosses.
    for (col, row) in ray {
        sink.text(*col, *row, 1, dot, muted);
    }
    if let Some((col, row)) = ring {
        sink.text(col, row, 1, ring_glyph, canvas::color("ring"));
    }
    if let Some(boxed) = &overlay.boxed {
        boxed.draw(
            &mut sink,
            overlay.at,
            Rect::sized(size.0, size.1),
            plot::TooltipStyle {
                frame: canvas::color("border"),
                text: canvas::color("foreground"),
                background: canvas::color("surface"),
                ascii,
            },
        );
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CHT-027: a scatter on a numeric x with far more points than columns
    /// is drawn from at most two samples per column, and every point keeps
    /// an anchor so hover and the keys still reach it.
    #[test]
    fn a_numeric_x_scatter_is_thinned_per_column_and_keeps_every_anchor() {
        let points: Vec<DataPoint> = (0..10_000)
            .map(|i| {
                DataPoint::xy(
                    (i as f64 * 0.37) % 100.0,
                    ((i as f64) * 0.01).sin() * 4.0 + 5.0,
                )
            })
            .collect();
        let props = ChartProps {
            chart_type: ChartType::Scatter,
            width: 60,
            height: 16,
            series: vec![DataSeries::new("s", points)],
            legend: ChartLegend {
                visible: false,
                ..Default::default()
            },
            animated: false,
            ..Default::default()
        };
        let values: Vec<Vec<f64>> = vec![props.series[0].data.iter().map(|p| p.value).collect()];
        let picture = canvas::draw(&canvas::Job {
            props: &props,
            width: 60,
            height: 16,
            values: &values,
            progress: 1.0,
            transition: None,
            unicode_glyphs: true,
            selected: None,
            pixels: None,
        });
        assert!(
            picture.kept[0].len() <= 2 * picture.plot.w,
            "{} samples drawn for a plot {} columns wide",
            picture.kept[0].len(),
            picture.plot.w
        );
        assert_eq!(
            picture.anchors.len(),
            10_000,
            "every point keeps an anchor for hover"
        );
    }

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
                transition: None,
                unicode_glyphs: true,
                selected: None,
                pixels: None,
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

    /// CHT-037: a layout whose terminal changed, a new cell size or pixels
    /// now taken, changes the chart, so the next picture is drawn for it.
    #[test]
    fn cht_037_a_new_cell_size_changes_the_charts_layout() {
        use crate::component::{Component, LayoutInfo, TerminalInfo};
        let mut props = LiveProps {
            config: Arc::new(ChartProps::default()),
            seed: ChartState::default(),
        };
        let mut state = ChartState::default();
        let mut chart = LiveChart::new(props.clone());
        let bounds = crate::event::hit::Bounds::new(0.0, 0.0, 80.0, 24.0);
        let mut layout = LayoutInfo::from_bounds(bounds);
        assert!(chart.layout(layout, &mut props, &mut state));
        assert!(!chart.layout(layout, &mut props, &mut state));
        layout.terminal = TerminalInfo::new((9, 18), true, false, false);
        assert!(chart.layout(layout, &mut props, &mut state));
        assert!(!chart.layout(layout, &mut props, &mut state));
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
            theme: 0,
            pixels: None,
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
