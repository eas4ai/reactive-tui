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
        Element::fragment().with_children(
            entries
                .into_iter()
                .enumerate()
                .map(|(rank, (id, _, content, activity))| {
                    Element::typed::<Layer>(LayerProps {
                        content: content.render(id, &config.default_theme),
                        presentation: Presentation {
                            z_index: config.base_z_index + (rank * 2) as u16,
                            focus_trap: config.focus_trap,
                            escape_to_close: config.escape_to_close,
                            backdrop_blur: config.backdrop_blur,
                            animated: config.default_theme.animation != DialogAnimation::None,
                            motion: motion::motion(&self.engine.core, id, &config, custom.clone()),
                            activity,
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

#[derive(Clone, PartialEq)]
pub(in crate::widgets::dialog) struct Presentation {
    pub z_index: u16,
    pub focus_trap: bool,
    pub escape_to_close: bool,
    pub backdrop_blur: bool,
    pub animated: bool,
    pub motion: crate::widgets::display::modal::Motion,
    pub activity: Activity,
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
