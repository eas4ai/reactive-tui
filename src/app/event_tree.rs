//! Connect owned element callbacks to acknowledged painter geometry.

#[cfg(target_os = "linux")]
mod accessibility;

#[cfg(test)]
mod capture_tests;

use crate::{
    backend::PaintedNode,
    component::Element,
    event::{
        router::{EventPhase, EventResult, EventRouter, NodeId},
        types::Event,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Hash, Eq, PartialEq)]
enum Slot {
    Component(u64),
    Key(String),
    Index(usize),
}

impl Slot {
    fn append(path: &mut Vec<Self>, element: &Element, index: usize) {
        path.extend(
            element
                .metadata
                .component_instances
                .iter()
                .copied()
                .map(Self::Component),
        );
        path.push(
            element
                .key
                .as_ref()
                .map_or(Self::Index(index), |key| Self::Key(key.clone())),
        );
    }
}

#[derive(Default)]
pub(crate) struct EventTree {
    nodes: HashMap<Vec<Slot>, NodeId>,
    pressed: Option<(crate::event::types::MouseButton, Vec<Slot>)>,
}

struct StyleState<'a> {
    focus: Option<&'a [Slot]>,
    hover: Option<&'a [Slot]>,
    active: Option<&'a [Slot]>,
    width: u16,
}

impl StyleState<'_> {
    fn matches(
        &self,
        variant: &str,
        element: &Element,
        path: &[Slot],
        position: (usize, usize),
        group: [bool; 3],
    ) -> Option<bool> {
        let (index, siblings) = position;
        Some(match variant {
            "focus" => !element.metadata.disabled && self.focus == Some(path),
            "focus-within" => self.focus.is_some_and(|focused| focused.starts_with(path)),
            "hover" => self.hover.is_some_and(|hovered| hovered.starts_with(path)),
            "active" => self.active.is_some_and(|pressed| pressed.starts_with(path)),
            "visited" => false,
            "first" => siblings > 0 && index == 0,
            "last" => siblings > 0 && index + 1 == siblings,
            "odd" => siblings > 0 && index.is_multiple_of(2),
            "even" => siblings > 0 && !index.is_multiple_of(2),
            "group-hover" => group[0],
            "group-focus" => group[1],
            "group-active" => group[2],
            "disabled" => element.metadata.disabled,
            "sm" => self.width >= 40,
            "md" => self.width >= 80,
            "lg" => self.width >= 120,
            "xl" => self.width >= 160,
            _ => return None,
        })
    }
}

struct Registration<'a> {
    inert: bool,
    keyboard_only: bool,
    inert_nodes: HashSet<NodeId>,
    /// The element indices the frame painted, when it painted any: an
    /// element missing from them was hidden by `display: none` (STY-002).
    present: Option<HashSet<usize>>,
    router: &'a mut EventRouter,
    seen: HashSet<Vec<Slot>>,
    preorder: Vec<NodeId>,
    focus: super::focus_manager::FocusPlan,
    layouts: Vec<Vec<crate::component::element::LayoutCallback>>,
}

impl EventTree {
    pub(crate) fn track_press(&mut self, event: &Event, router: &EventRouter) -> bool {
        use crate::event::types::MouseEventKind;
        let Event::Mouse(mouse) = event else {
            return false;
        };
        if !matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Up) {
            return false;
        }
        let previous = self.pressed.clone();
        match mouse.kind {
            MouseEventKind::Down
                if self
                    .pressed
                    .as_ref()
                    .is_none_or(|(button, _)| *button == mouse.button) =>
            {
                // Retain the pressed path even if the pointer leaves it (STY-001).
                self.pressed = self
                    .path_for(router.target_under_pointer(event))
                    .map(|path| (mouse.button, path.to_vec()));
            }
            MouseEventKind::Up
                if self
                    .pressed
                    .as_ref()
                    .is_some_and(|(button, _)| *button == mouse.button) =>
            {
                self.pressed = None;
            }
            _ => {}
        }
        previous != self.pressed
    }

    pub(crate) fn innermost_component(&self, id: NodeId) -> Option<u64> {
        let path = self.path_for(id)?;
        let mut innermost = None;
        for slot in path.iter().rev() {
            match slot {
                Slot::Component(identity) => innermost = Some(*identity),
                Slot::Key(_) | Slot::Index(_) if innermost.is_some() => return innermost,
                Slot::Key(_) | Slot::Index(_) => {}
            }
        }
        innermost
    }

    /// Resolve state variants without publishing a candidate event tree.
    pub(crate) fn styled(&self, element: &Element, router: &EventRouter, width: u16) -> Element {
        let focus = router.get_focus().and_then(|id| self.path_for(id));
        let hover = router.hovered_node().and_then(|id| self.path_for(id));
        let state = StyleState {
            focus,
            hover,
            active: self.pressed.as_ref().map(|(_, path)| path.as_slice()),
            width,
        };
        Self::style_node(element.clone(), Vec::new(), 0, 0, &state, [false; 3])
    }

    fn path_for(&self, id: NodeId) -> Option<&[Slot]> {
        self.nodes
            .iter()
            .find_map(|(path, node)| (*node == id).then_some(path.as_slice()))
    }

    fn style_node(
        mut element: Element,
        mut path: Vec<Slot>,
        index: usize,
        siblings: usize,
        state: &StyleState<'_>,
        group: [bool; 3],
    ) -> Element {
        Slot::append(&mut path, &element, index);
        let focused = !element.metadata.disabled && state.focus == Some(path.as_slice());
        let hovered = state
            .hover
            .is_some_and(|hovered| hovered.starts_with(&path));
        let pressed = state
            .active
            .is_some_and(|pressed| pressed.starts_with(&path));
        // Children use the nearest group ancestor, never the element itself (STY-001).
        let child_group = if element
            .class
            .as_ref()
            .is_some_and(|class| class.split_whitespace().any(|token| token == "group"))
        {
            [hovered, focused, pressed]
        } else {
            group
        };
        if let Some(class) = &element.class {
            element.class = Some(
                class
                    .split_whitespace()
                    .filter_map(|token| {
                        let mut base = token;
                        while let Some((variant, rest)) = base.split_once(':') {
                            let Some(matches) =
                                state.matches(variant, &element, &path, (index, siblings), group)
                            else {
                                break;
                            };
                            if !matches {
                                return None;
                            }
                            base = rest;
                        }
                        Some(base)
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        let siblings = element.children.len();
        element.children = element
            .children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                Self::style_node(child, path.clone(), index, siblings, state, child_group)
            })
            .collect();
        element
    }

    /// Register the frame's elements with the router. `geometry` is what
    /// the frame painted: the hit targets, and, when it holds anything, the
    /// elements on screen at all, since the painter leaves an element hidden
    /// by `display: none` out of it. Such an element, and everything under
    /// it, is inert for the frame: no handlers, no focus, and no hold on a
    /// press it took while it was shown (STY-002). An empty `geometry`
    /// carries no frame and hides nothing.
    pub(crate) fn sync(
        &mut self,
        element: &Element,
        geometry: &[PaintedNode],
        layouts: Option<&[crate::backend::PresentedLayout]>,
        cell_hits: Option<(&[u32], u16)>,
        router: &mut EventRouter,
    ) -> (super::focus_manager::FocusPlan, bool) {
        let previous_focus = router.get_focus();
        let mut frame = Registration {
            inert: false,
            keyboard_only: false,
            inert_nodes: HashSet::new(),
            present: (!geometry.is_empty())
                .then(|| geometry.iter().map(|node| node.element_index).collect()),
            router,
            seen: HashSet::new(),
            preorder: Vec::new(),
            focus: super::focus_manager::FocusPlan::new(previous_focus),
            layouts: Vec::new(),
        };
        self.visit(element, Vec::new(), 0, None, &mut frame);
        frame.router.release_pointer_from(&frame.inert_nodes);
        let removed: Vec<_> = self
            .nodes
            .keys()
            .filter(|path| !frame.seen.contains(*path))
            .cloned()
            .collect();
        let removed: Vec<_> = removed
            .iter()
            .filter_map(|path| self.nodes.remove(path))
            .collect();
        frame.router.set_focus_order(&frame.preorder);
        frame.router.remove_nodes(&removed);
        let mut layout_changed = false;
        let layouts: HashMap<_, _> = layouts
            .unwrap_or_default()
            .iter()
            .map(|node| (node.element_index, node.layout))
            .collect();
        for (order, node) in geometry.iter().enumerate() {
            if let Some(&id) = frame.preorder.get(node.element_index) {
                for callback in &frame.layouts[node.element_index] {
                    let layout = layouts
                        .get(&node.element_index)
                        .copied()
                        .unwrap_or_else(|| crate::component::LayoutInfo::from_bounds(node.bounds));
                    layout_changed |= callback(layout);
                }
                if !frame.inert_nodes.contains(&id) {
                    frame.router.add_hit_target(
                        id,
                        node.bounds,
                        order.min(i32::MAX as usize) as i32,
                    );
                    frame.router.update_spatial(
                        id,
                        node.bounds.x,
                        node.bounds.y,
                        node.bounds.width,
                        node.bounds.height,
                    );
                }
            }
        }
        // Element index to node for the backend's per-cell hit grid; an inert
        // element maps to nothing so the bounds tree answers there (PNT-002).
        let nodes = frame
            .preorder
            .iter()
            .map(|id| (!frame.inert_nodes.contains(id)).then_some(*id))
            .collect();
        frame.router.set_cell_hits(cell_hits, nodes);
        layout_changed |= frame.router.refresh_hover();
        (frame.focus, layout_changed)
    }

    fn visit(
        &mut self,
        element: &Element,
        mut path: Vec<Slot>,
        index: usize,
        parent: Option<NodeId>,
        frame: &mut Registration<'_>,
    ) {
        Slot::append(&mut path, element, index);
        let id = *self
            .nodes
            .entry(path.clone())
            .or_insert_with(|| frame.router.create_node(parent));
        frame.seen.insert(path.clone());
        // An element the frame did not paint was hidden by `display: none`,
        // its own or an ancestor's: off the screen, it takes no event, no
        // focus and no hold on the pointer, nor do its children (STY-002).
        let hidden = frame
            .present
            .as_ref()
            .is_some_and(|present| !present.contains(&frame.preorder.len()));
        frame.preorder.push(id);
        frame.layouts.push(element.metadata.layout.clone());
        let ancestor_inert = frame.inert;
        let ancestor_keyboard_only = frame.keyboard_only;
        frame.inert |= element.metadata.inert || hidden;
        frame.keyboard_only |= element
            .metadata
            .accessibility_options
            .as_ref()
            .is_some_and(|o| o.keyboard_only);
        frame.register(element, id);
        let trap = if frame.inert {
            (false, false)
        } else {
            frame.focus.enter(element, id)
        };
        for (index, child) in element.children.iter().enumerate() {
            self.visit(child, path.clone(), index, Some(id), frame);
        }
        frame.focus.leave(trap);
        frame.inert = ancestor_inert;
        frame.keyboard_only = ancestor_keyboard_only;
    }

    /// Hand each element's layout callbacks its layout from a frame that
    /// was laid out but not presented, indexed in the preorder `sync` uses,
    /// so each component sizes itself before that frame is painted. Hit
    /// targets and focus wait for the presented frame. Returns whether any
    /// component's layout changed.
    pub(crate) fn publish_layouts(
        element: &Element,
        layouts: &[crate::backend::PresentedLayout],
    ) -> bool {
        fn preorder<'a>(
            element: &'a Element,
            callbacks: &mut Vec<&'a [crate::component::element::LayoutCallback]>,
        ) {
            callbacks.push(&element.metadata.layout);
            for child in &element.children {
                preorder(child, callbacks);
            }
        }
        let mut callbacks = Vec::new();
        preorder(element, &mut callbacks);
        let mut changed = false;
        for presented in layouts {
            for callback in callbacks
                .get(presented.element_index)
                .copied()
                .unwrap_or_default()
            {
                changed |= callback(presented.layout);
            }
        }
        changed
    }

    pub(crate) fn clear(&mut self, router: &mut EventRouter) {
        self.pressed = None;
        let nodes: Vec<_> = self.nodes.drain().map(|(_, id)| id).collect();
        router.remove_nodes(&nodes);
    }
}

impl Registration<'_> {
    fn register(&mut self, element: &Element, id: NodeId) {
        self.router.remove_hit_target(id);
        if self.keyboard_only {
            self.inert_nodes.insert(id);
        }
        if self.inert {
            self.inert_nodes.insert(id);
            self.router.remove_focusable(id);
            self.router.clear_handlers(id);
            return;
        }
        let interactive = !element.metadata.disabled && !element.metadata.on_click.is_empty();
        let focusable = !element.metadata.disabled
            && element
                .focus
                .as_ref()
                .map_or(interactive, |focus| focus.focusable);
        if focusable {
            self.router
                .add_focusable(id, element.focus.as_ref().map(|focus| focus.tab_index));
        } else {
            self.router.remove_focusable(id);
        }
        self.router.clear_handlers(id);
        self.register_focus_callbacks(element, id);
        if !element.metadata.disabled {
            for handler in &element.metadata.capture_events {
                for event_type in ["key", "mouse", "focus", "paste", "custom"] {
                    self.router
                        .add_handler(id, event_type, EventPhase::Capture, handler.clone());
                }
            }
            for handler in &element.metadata.events {
                for event_type in ["key", "mouse", "focus", "paste", "custom"] {
                    self.router
                        .add_handler(id, event_type, EventPhase::Bubble, handler.clone());
                }
            }
        }
        if interactive {
            let callbacks = element.metadata.on_click.clone();
            let handler: crate::event::router::EventHandlerFn = Arc::new(move |event| {
                if !event.activates_control() {
                    return EventResult::Ignored;
                }
                for callback in &callbacks {
                    callback();
                }
                EventResult::Consumed
            });
            self.router
                .add_handler(id, "key", EventPhase::Bubble, handler.clone());
            self.router
                .add_handler(id, "mouse", EventPhase::Bubble, handler);
        }
    }
    fn register_focus_callbacks(&mut self, element: &Element, id: NodeId) {
        if let Some(focus) = &element.focus {
            let gained = focus.on_focus.clone();
            let lost = focus.on_blur.clone();
            if gained.is_some() || lost.is_some() {
                self.router.add_handler(
                    id,
                    "focus",
                    EventPhase::Target,
                    Arc::new(move |event| {
                        use crate::event::types::FocusEventKind;
                        let callback = match event {
                            Event::Focus(event) if event.kind == FocusEventKind::Gained => &gained,
                            Event::Focus(event) if event.kind == FocusEventKind::Lost => &lost,
                            _ => return EventResult::Ignored,
                        };
                        if let Some(callback) = callback {
                            callback();
                            return EventResult::Handled;
                        }
                        EventResult::Ignored
                    }),
                );
            }
        }
    }
}
