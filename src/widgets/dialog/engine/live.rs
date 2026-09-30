use super::*;
use crate::component::{Component, LifecycleEvent, Props};
use std::any::Any;

#[derive(Clone)]
pub(super) struct HostProps(pub DialogEngine);
impl PartialEq for HostProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0.core, &other.0.core)
    }
}
impl Props for HostProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct Host {
    engine: DialogEngine,
    owns_mount: bool,
}
impl Host {
    fn release(&mut self) {
        if self.owns_mount {
            self.owns_mount = false;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.engine.core.cancel_all()
            }));
            {
                let mut state = self.engine.core.state.lock().unwrap();
                state.mounted = false;
                state.scheduler = None;
            }
            if let Err(payload) = result {
                std::panic::resume_unwind(payload);
            }
        }
    }
}
impl Component for Host {
    type Props = HostProps;
    type State = ();
    fn new(props: HostProps) -> Self {
        let owns_mount = {
            let mut state = props.0.core.state.lock().unwrap();
            let owns = !state.mounted;
            if owns {
                state.mounted = true;
                state.scheduler = crate::reactive::component_scope::current()
                    .map(|scope| Arc::downgrade(&scope.scheduler()));
            }
            owns
        };
        Self {
            engine: props.0,
            owns_mount,
        }
    }
    fn update(&mut self, props: &HostProps, _: &mut ()) -> bool {
        if !Arc::ptr_eq(&self.engine.core, &props.0.core) {
            self.release();
            *self = Self::new(props.clone());
        }
        true
    }
    fn render(&self, _: &HostProps, _: &()) -> Element {
        if !self.owns_mount {
            return Element::text(
                "DialogEngine is already mounted; use a separate engine for each App",
            );
        }
        self.engine.core.changed.get();
        let (mut entries, config, custom) = {
            let state = self.engine.core.state.lock().unwrap();
            let mut entries: Vec<_> = state
                .order
                .iter()
                .filter_map(|id| {
                    state.dialogs.get(id).map(|entry| {
                        (
                            *id,
                            entry.priority,
                            entry.content.clone(),
                            entry.activity.clone(),
                        )
                    })
                })
                .collect();
            entries.extend(state.retiring.iter().map(|(id, entry)| {
                (
                    *id,
                    entry.priority,
                    entry.content.clone(),
                    entry.activity.clone(),
                )
            }));
            let custom =
                if let DialogAnimation::Custom(name) = &state.config.default_theme.animation {
                    state.animations.get(name).cloned()
                } else {
                    None
                };
            (entries, state.config.clone(), custom)
        };
        entries.sort_by_key(|(id, priority, _, _)| (*priority, id.0));
        // Where each box goes beside the others (OVL-003): a dialog opened
        // over another one row lower than it; a toast under the earlier
        // toasts at its position, one row apart, or over them at a bottom
        // position.
        let heights = self.engine.core.state.lock().unwrap().heights.clone();
        let mut dialogs = 0i16;
        let mut stacked: HashMap<ToastPosition, i16> = HashMap::new();
        let offsets: Vec<(i16, i16)> = entries
            .iter()
            .map(|(id, _, content, _)| match content.toast_position() {
                Some(position) => {
                    let rows = stacked.entry(position).or_default();
                    let offset = *rows;
                    let height = heights.get(id).copied().unwrap_or(0);
                    *rows += i16::try_from(height).unwrap_or(i16::MAX).saturating_add(1);
                    let down = matches!(
                        position,
                        ToastPosition::TopLeft | ToastPosition::TopCenter | ToastPosition::TopRight
                    );
                    (0, if down { offset } else { -offset })
                }
                None => {
                    let offset = dialogs;
                    dialogs = dialogs.saturating_add(1);
                    (0, offset)
                }
            })
            .collect();
        Element::fragment().with_children(
            entries
                .into_iter()
                .zip(offsets)
                .enumerate()
                .map(|(rank, ((id, _, content, activity), offset))| {
                    let core = self.engine.core.clone();
                    Element::typed::<Layer>(LayerProps {
                        content: content.render(id, &config.default_theme),
                        presentation: Presentation {
                            // A toast is painted over the dialogs and the
                            // popovers, under the menu panels (OVL-003).
                            z_index: if content.toast_position().is_some() {
                                TOAST_LAYER
                            } else {
                                config.base_z_index
                            } + (rank * 2) as u16,
                            focus_trap: config.focus_trap,
                            escape_to_close: config.escape_to_close,
                            backdrop_blur: config.backdrop_blur,
                            animated: config.default_theme.animation != DialogAnimation::None,
                            motion: motion::motion(&self.engine.core, id, &config, custom.clone()),
                            activity,
                            offset,
                            placed: Some(Arc::new(move |_, (_, height)| core.placed(id, height))),
                        },
                    })
                    .with_key(format!("dialog-{}", id.0))
                })
                .collect(),
        )
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.release();
        }
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.release();
    }
}

/// Where a toast is painted: over a modal or a dialog (1000 and up) and a
/// popover (2000), under a menu panel (3000).
pub(in crate::widgets::dialog) const TOAST_LAYER: u16 = 2500;

#[derive(Clone)]
pub(in crate::widgets::dialog) struct Presentation {
    pub z_index: u16,
    pub focus_trap: bool,
    pub escape_to_close: bool,
    pub backdrop_blur: bool,
    pub animated: bool,
    pub motion: crate::widgets::display::modal::Motion,
    pub activity: Activity,
    /// Cells the box is moved from the place its position names.
    pub offset: (i16, i16),
    /// Told the box's position and size each time they change.
    pub placed: Option<crate::widgets::display::modal::PlacedCallback>,
}
impl PartialEq for Presentation {
    fn eq(&self, other: &Self) -> bool {
        self.z_index == other.z_index
            && self.focus_trap == other.focus_trap
            && self.escape_to_close == other.escape_to_close
            && self.backdrop_blur == other.backdrop_blur
            && self.animated == other.animated
            && self.motion == other.motion
            && self.activity == other.activity
            && self.offset == other.offset
            && crate::widgets::display::overlay::same_callback(&self.placed, &other.placed)
    }
}
#[derive(Clone, PartialEq)]
struct LayerProps {
    content: Element,
    presentation: Presentation,
}
impl Props for LayerProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
struct Layer;
impl Component for Layer {
    type Props = LayerProps;
    type State = ();
    fn new(_: LayerProps) -> Self {
        Self
    }
    fn render(&self, props: &LayerProps, _: &()) -> Element {
        let _ = crate::reactive::component_scope::provide(props.presentation.clone());
        let mut element = props.content.clone();
        let activity = props.presentation.activity.clone();
        element.metadata.capture_events.push(Arc::new(move |_| {
            if activity.active() {
                crate::event::router::EventResult::Ignored
            } else {
                crate::event::router::EventResult::Consumed
            }
        }));
        element
    }
}
