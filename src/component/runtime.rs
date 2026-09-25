//! App-owned instances. Registry entries are factories, not owners of this tree.

use super::{
    registry::get_global_registry, AnyComponentInstance, Element, ElementType, LifecycleEvent,
};
use crate::error::{ReactiveError, Result};
use crate::reactive::component_scope::ComponentScope;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Segment {
    Key(String),
    Index(usize),
    Output(String),
}

type Path = Vec<Segment>;

/// A component's rendered output waiting to expand, with what the component
/// element contributes to the result. It stays boxed while the output
/// expands, so each level of a deep component chain holds a pointer rather
/// than several `Element`s: a debug build gives every `Element` temporary
/// its own stack slot, and 128 levels of them needed about 2 MiB. That
/// overflowed a 2 MiB test thread once the embedded-terminal feature's
/// terminal library took 256 KiB of its stack for thread-local storage.
struct Rendered {
    /// Keeps the component's scope entered while its output expands, so its
    /// descendants see the contexts it provides.
    _binding: crate::reactive::component_scope::ScopeGuard,
    caller: Element,
    output: Option<Element>,
    output_path: Path,
    identity: u64,
    events: crate::event::router::EventHandlerFn,
    layout: super::element::LayoutCallback,
}

struct LiveComponent {
    identity: u64,
    name: String,
    route_id: String,
    instance: Arc<Mutex<AnyComponentInstance>>,
    scope: Arc<ComponentScope>,
    bounds: Arc<Mutex<Option<super::LayoutInfo>>>,
}

#[derive(Default)]
pub(crate) struct ComponentRuntime {
    next_identity: u64,
    instances: HashMap<Path, LiveComponent>,
    radio_groups: Arc<crate::widgets::input::named_radio::RadioGroups>,
    pub(crate) anchors: Arc<super::anchors::Anchors>,
    mouse: crate::hooks::processor::MouseEventProcessor,
}

impl ComponentRuntime {
    pub(crate) fn resolve(&mut self, element: Element) -> Result<Element> {
        self.anchors.begin_render();
        assert!(
            crate::reactive::component_scope::provide(self.anchors.clone()).is_ok(),
            "App component expansion requires its resource scope"
        );
        assert!(
            crate::reactive::component_scope::provide(self.radio_groups.clone()).is_ok(),
            "App component expansion requires its resource scope"
        );
        let mut seen = HashSet::new();
        let path = vec![Self::slot(&element, 0)];
        let result = self.expand(element, path, &mut seen, 0)?;
        self.remove_where(|path| !seen.contains(path));
        Ok(result)
    }

    fn slot(element: &Element, index: usize) -> Segment {
        element
            .key
            .as_ref()
            .map_or(Segment::Index(index), |key| Segment::Key(key.clone()))
    }

    fn expand(
        &mut self,
        element: Element,
        path: Path,
        seen: &mut HashSet<Path>,
        depth: usize,
    ) -> Result<Element> {
        if depth > 128 {
            return Err(ReactiveError::invalid_state(
                "component tree exceeds expansion depth 128",
            ));
        }
        // Each level of a deep tree keeps only this frame and a small one
        // below it on the stack: mounting, rendering and merging run in
        // their own frames, which end before the next level begins.
        if let Some(newly_created) = self.mount(&element, &path)? {
            let rendered = self.render_mounted(element, path, newly_created, seen)?;
            return self.expand_rendered(rendered, seen, depth);
        }
        self.expand_children(element, path, seen, depth)
    }

    /// Mounts the component `element` names at `path`, replacing one of
    /// another type there, and says whether an instance now lives there
    /// and whether it was just created. `None` for an element that is not
    /// a component or names one nothing can create: it expands as a
    /// container.
    #[inline(never)]
    fn mount(&mut self, element: &Element, path: &Path) -> Result<Option<bool>> {
        let ElementType::Component(name) = &element.element_type else {
            return Ok(None);
        };
        if self
            .instances
            .get(path)
            .is_some_and(|live| &live.name != name)
        {
            self.remove_where(|candidate| candidate.starts_with(path));
        }
        let newly_created = !self.instances.contains_key(path);
        if newly_created {
            let scope = ComponentScope::child(
                crate::reactive::component_scope::current()
                    .ok_or_else(|| {
                        ReactiveError::invalid_state(
                            "App component expansion requires its resource scope",
                        )
                    })?
                    .scheduler(),
            );
            let _binding = scope.enter(true);
            // create_by_name releases registry locks before calling user code.
            let instance = if let Some(factory) = &element.metadata.factory {
                Some(factory(element.props.as_ref())?)
            } else {
                get_global_registry()
                    .create_by_name(name, element.props.as_ref())?
                    .or_else(|| super::builtin::create(name, element.props.as_ref()))
            };
            if let Some(mut instance) = instance {
                self.next_identity = self.next_identity.checked_add(1).ok_or_else(|| {
                    ReactiveError::invalid_state("component mount identity exhausted")
                })?;
                instance.on_lifecycle(LifecycleEvent::Mount);
                let route_id = element.key.clone().unwrap_or_else(|| name.clone());
                self.instances.insert(
                    path.clone(),
                    LiveComponent {
                        identity: self.next_identity,
                        name: name.clone(),
                        instance: Arc::new(Mutex::new(instance)),
                        scope: scope.clone(),
                        bounds: Arc::new(Mutex::new(None)),
                        route_id,
                    },
                );
            }
        }
        // Unknown names retain the existing container behavior.
        Ok(self.instances.contains_key(path).then_some(newly_created))
    }

    /// Renders the mounted component at `path` inside its scope and boxes
    /// the output with what `element` contributes to the expanded result.
    #[inline(never)]
    fn render_mounted(
        &mut self,
        mut element: Element,
        path: Path,
        newly_created: bool,
        seen: &mut HashSet<Path>,
    ) -> Result<Box<Rendered>> {
        let live = self.instances.get_mut(&path).ok_or_else(|| {
            ReactiveError::invalid_state("component instance left between mount and render")
        })?;
        let binding = live.scope.enter(!newly_created);
        assert!(
            crate::reactive::component_scope::provide(crate::hooks::processor::MouseHookContext {
                owner: live.mouse_owner(),
                processor: self.mouse.clone(),
            },)
            .is_ok(),
            "component mouse hooks require their resource scope"
        );
        seen.insert(path.clone());
        let mut output = {
            let mut instance = live
                .instance
                .lock()
                .map_err(|_| ReactiveError::invalid_state("component instance lock poisoned"))?;
            instance.update_shared(&element.props);
            instance.try_render()?
        };
        let (events, layout) = live.handlers();
        let identity = live.identity;
        let ElementType::Component(name) = &element.element_type else {
            return Err(ReactiveError::invalid_state(
                "a mounted component element lost its component type",
            ));
        };
        let name = name.clone();
        output.children.append(&mut element.children);
        let mut output_path = path;
        output_path.push(Segment::Output(name));
        output_path.push(Self::slot(&output, 0));
        Ok(Box::new(Rendered {
            _binding: binding,
            caller: element,
            output: Some(output),
            output_path,
            identity,
            events,
            layout,
        }))
    }

    /// Expands a component's rendered output with the component's scope
    /// still entered, then merges in what the component element carries.
    fn expand_rendered(
        &mut self,
        mut rendered: Box<Rendered>,
        seen: &mut HashSet<Path>,
        depth: usize,
    ) -> Result<Element> {
        let output = rendered
            .output
            .take()
            .ok_or_else(|| ReactiveError::invalid_state("component output expanded twice"))?;
        let path = std::mem::take(&mut rendered.output_path);
        let resolved = self.expand(output, path, seen, depth + 1)?;
        Ok(Self::merge(resolved, rendered))
    }

    /// The expanded output of a component, carrying the component element's
    /// handlers, flags, styles, class, focus and key; the caller's styling
    /// belongs to the rendered root and takes precedence.
    #[inline(never)]
    fn merge(mut resolved: Element, rendered: Box<Rendered>) -> Element {
        let Rendered {
            _binding,
            caller: element,
            identity,
            events,
            layout,
            ..
        } = *rendered;
        resolved.metadata.component_instances.push(identity);
        resolved.metadata.events.push(events);
        resolved.metadata.events.extend(element.metadata.events);
        resolved
            .metadata
            .capture_events
            .extend(element.metadata.capture_events);
        resolved.metadata.layout.push(layout);
        resolved.metadata.on_click.extend(element.metadata.on_click);
        resolved.metadata.disabled |= element.metadata.disabled;
        resolved.metadata.inert |= element.metadata.inert;
        resolved
            .metadata
            .animation_values
            .extend(element.metadata.animation_values);
        resolved.metadata.focus_scope |= element.metadata.focus_scope;
        if let Some(accessibility) = element.metadata.accessibility {
            resolved.metadata.accessibility = Some(accessibility);
        }
        if let Some(options) = element.metadata.accessibility_options {
            let target = resolved
                .metadata
                .accessibility_options
                .get_or_insert_default();
            target.focus |= options.focus;
            target.clickable |= options.clickable;
            if options.focus_event.is_some() {
                target.focus_event = options.focus_event;
            }
            if options.click_event.is_some() {
                target.click_event = options.click_event;
            }
            if options.label.is_some() {
                target.label = options.label;
            }
        }
        if element.metadata.styles.is_some() {
            resolved.metadata.styles = element.metadata.styles;
        }
        if let Some(outer) = element.metadata.inline_styles {
            resolved.metadata.inline_styles = Some(match resolved.metadata.inline_styles {
                Some(inner) => format!("{inner};{outer}"),
                None => outer,
            });
        }
        if element.metadata.gradient.is_some() {
            resolved.metadata.gradient = element.metadata.gradient;
        }
        if element.metadata.gradient_border.is_some() {
            resolved.metadata.gradient_border = element.metadata.gradient_border;
        }
        // Caller styling belongs to the rendered root and takes precedence.
        if let Some(class) = element.class {
            resolved.class = Some(match resolved.class {
                Some(inner) => format!("{inner} {class}"),
                None => class,
            });
        }
        if element.focus.is_some() {
            resolved.focus = element.focus;
        }
        if element.key.is_some() {
            resolved.key = element.key;
        }
        resolved
    }

    /// Expands each child of a container element in order.
    fn expand_children(
        &mut self,
        mut element: Element,
        path: Path,
        seen: &mut HashSet<Path>,
        depth: usize,
    ) -> Result<Element> {
        let mut keys = HashSet::new();
        let children = std::mem::take(&mut element.children);
        for (index, child) in children.into_iter().enumerate() {
            if let Some(key) = &child.key {
                if !keys.insert(key.clone()) {
                    return Err(ReactiveError::invalid_state(format!(
                        "duplicate sibling key: {key}"
                    )));
                }
            }
            let mut child_path = path.clone();
            child_path.push(Self::slot(&child, index));
            element
                .children
                .push(self.expand(child, child_path, seen, depth + 1)?);
        }
        Ok(element)
    }

    fn remove_where(&mut self, remove: impl Fn(&Path) -> bool) -> usize {
        let mut removed: Vec<_> = self
            .instances
            .keys()
            .filter(|path| remove(path))
            .cloned()
            .collect();
        // A parent cleanup may observe its children: release children first.
        removed.sort_by_key(|path| std::cmp::Reverse(path.len()));
        let count = removed.len();
        for path in removed {
            if let Some(live) = self.instances.remove(&path) {
                self.mouse.unregister_component(&live.mouse_owner());
                let _binding = live.scope.enter(false);
                live.instance
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .on_lifecycle(LifecycleEvent::Unmount);
                live.scope.close();
                drop(live);
            }
        }
        count
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.remove_where(|_| true)
    }

    pub(crate) fn process_mouse_event(
        &self,
        identity: Option<u64>,
        event: &crate::event::types::MouseEvent,
    ) {
        let Some(live) = identity.and_then(|identity| {
            self.instances
                .values()
                .find(|live| live.identity == identity)
        }) else {
            self.mouse.process_routed_event(event, None, None, None);
            return;
        };
        let mut local = event.clone();
        // A poisoned bounds lock degrades to "no bounds", exactly like a
        // component that has not been measured yet.
        let cached = live.bounds.lock().map(|guard| *guard).unwrap_or(None);
        if let (crate::event::types::Position::Cell { x, y }, Some(bounds)) =
            (event.position, cached)
        {
            let Some((x, y)) = bounds.local_cell(f32::from(x), f32::from(y)) else {
                self.mouse.process_routed_event(event, None, None, None);
                return;
            };
            local.position = crate::event::types::Position::cell(x, y);
        }
        self.mouse.process_routed_event(
            event,
            Some(&live.mouse_owner()),
            Some(&live.route_id),
            Some(local.position),
        );
    }
}

impl LiveComponent {
    fn mouse_owner(&self) -> String {
        format!("component-mount-{}", self.identity)
    }

    fn handlers(
        &self,
    ) -> (
        crate::event::router::EventHandlerFn,
        super::element::LayoutCallback,
    ) {
        let weak = Arc::downgrade(&self.instance);
        let scope = self.scope.clone();
        let bounds = self.bounds.clone();
        let event_handler = Arc::new(move |event: &crate::event::Event| {
            use crate::event::{
                router::EventResult,
                types::{Event, KeyEventKind, Position},
            };
            if matches!(event, Event::Key(key) if key.kind == KeyEventKind::Release) {
                return EventResult::Ignored;
            }
            let Some(instance) = weak.upgrade() else {
                return EventResult::Ignored;
            };
            let mut local = event.clone();
            if let Event::Mouse(mouse) = &mut local {
                let cached = bounds.lock().map(|guard| *guard).unwrap_or(None);
                if let (Position::Cell { x, y }, Some(bounds)) = (mouse.position, cached) {
                    let (x, y) = match bounds.local_cell(f32::from(x), f32::from(y)) {
                        Some(position) => position,
                        // A boundary leave necessarily lies outside the component.
                        // Its position is not an actionable cell inside the control.
                        None if mouse.kind == crate::event::types::MouseEventKind::Leave => {
                            (u16::MAX, u16::MAX)
                        }
                        None => return EventResult::Ignored,
                    };
                    mouse.position = Position::cell(x, y);
                }
            }
            let _binding = scope.enter(false);
            instance
                .lock()
                .map(|mut instance| instance.handle_event(&local))
                .unwrap_or(EventResult::Ignored)
        });
        let weak = Arc::downgrade(&self.instance);
        let scope = self.scope.clone();
        let bounds = self.bounds.clone();
        let layout_handler = Arc::new(move |new_bounds| {
            let Ok(mut previous) = bounds.lock() else {
                return false;
            };
            let changed = *previous != Some(new_bounds);
            *previous = Some(new_bounds);
            if !changed {
                return false;
            }
            let Some(instance) = weak.upgrade() else {
                return false;
            };
            let _binding = scope.enter(false);
            instance
                .lock()
                .map(|mut instance| instance.layout(new_bounds))
                .unwrap_or(false)
        });
        (event_handler, layout_handler)
    }
}

impl Drop for ComponentRuntime {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        component::{props::EmptyProps, Component},
        hooks::use_hover,
        reactive::{Hooks, Scheduler},
    };

    struct MouseOwner {
        hooks: Hooks,
    }

    impl Component for MouseOwner {
        type Props = EmptyProps;
        type State = ();

        fn new(_props: Self::Props) -> Self {
            Self {
                hooks: Hooks::new(),
            }
        }

        fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
            self.hooks.reset();
            let _hover = use_hover(&self.hooks);
            Element::text("owner")
        }
    }

    #[test]
    fn api019_component_removal_unregisters_owned_mouse_hooks() {
        let scope = ComponentScope::new(Arc::new(Scheduler::new()));
        let _binding = scope.enter(true);
        let mut runtime = ComponentRuntime::default();
        runtime
            .resolve(Element::typed::<MouseOwner>(EmptyProps))
            .unwrap();
        assert_eq!(runtime.mouse.registration_count(), 1);

        runtime.resolve(Element::text("removed")).unwrap();
        assert_eq!(runtime.mouse.registration_count(), 0);
        scope.close();
    }
}
