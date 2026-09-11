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

struct LiveComponent {
    identity: u64,
    name: String,
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
        mut element: Element,
        path: Path,
        seen: &mut HashSet<Path>,
        depth: usize,
    ) -> Result<Element> {
        if depth > 128 {
            return Err(ReactiveError::invalid_state(
                "component tree exceeds expansion depth 128",
            ));
        }
        if let ElementType::Component(name) = &element.element_type {
            let name = name.clone();
            if self
                .instances
                .get(&path)
                .is_some_and(|live| live.name != name)
            {
                self.remove_where(|candidate| candidate.starts_with(&path));
            }
            let newly_created = !self.instances.contains_key(&path);
            if newly_created {
                let scope = ComponentScope::child(
                    crate::reactive::component_scope::current()
                        .expect("App component expansion requires its resource scope")
                        .scheduler(),
                );
                let _binding = scope.enter(true);
                // create_by_name releases registry locks before calling user code.
                let instance = if let Some(factory) = &element.metadata.factory {
                    Some(factory(element.props.as_ref())?)
                } else {
                    get_global_registry()
                        .create_by_name(&name, element.props.as_ref())?
                        .or_else(|| super::builtin::create(&name, element.props.as_ref()))
                };
                if let Some(mut instance) = instance {
                    self.next_identity = self.next_identity.checked_add(1).ok_or_else(|| {
                        ReactiveError::invalid_state("component mount identity exhausted")
                    })?;
                    instance.on_lifecycle(LifecycleEvent::Mount);
                    self.instances.insert(
                        path.clone(),
                        LiveComponent {
                            identity: self.next_identity,
                            name: name.clone(),
                            instance: Arc::new(Mutex::new(instance)),
                            scope: scope.clone(),
                            bounds: Arc::new(Mutex::new(None)),
                        },
                    );
                }
            }
            if let Some(live) = self.instances.get_mut(&path) {
                let _binding = live.scope.enter(!newly_created);
                seen.insert(path.clone());
                let mut output = {
                    let mut instance = live.instance.lock().unwrap();
                    instance.update(element.props.as_ref());
                    instance.render()
                };
                let (events, layout) = live.handlers();
                let identity = live.identity;
                output.children.append(&mut element.children);
                let mut output_path = path;
                output_path.push(Segment::Output(name));
                output_path.push(Self::slot(&output, 0));
                let mut resolved = self.expand(output, output_path, seen, depth + 1)?;
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
                return Ok(resolved);
            }
            // Unknown names retain the existing container behavior.
        }
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
}

impl LiveComponent {
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
                if let (Position::Cell { x, y }, Some(bounds)) =
                    (mouse.position, *bounds.lock().unwrap())
                {
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
            let result = instance.lock().unwrap().handle_event(&local);
            result
        });
        let weak = Arc::downgrade(&self.instance);
        let scope = self.scope.clone();
        let bounds = self.bounds.clone();
        let layout_handler = Arc::new(move |new_bounds| {
            let changed = {
                let mut previous = bounds.lock().unwrap();
                let changed = *previous != Some(new_bounds);
                *previous = Some(new_bounds);
                changed
            };
            if !changed {
                return false;
            }
            let Some(instance) = weak.upgrade() else {
                return false;
            };
            let _binding = scope.enter(false);
            let changed = instance.lock().unwrap().layout(new_bounds);
            changed
        });
        (event_handler, layout_handler)
    }
}

impl Drop for ComponentRuntime {
    fn drop(&mut self) {
        self.clear();
    }
}
