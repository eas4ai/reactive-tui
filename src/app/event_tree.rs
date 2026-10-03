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
}

struct Registration<'a> {
    inert: bool,
    keyboard_only: bool,
    inert_nodes: HashSet<NodeId>,
    router: &'a mut EventRouter,
    seen: HashSet<Vec<Slot>>,
    preorder: Vec<NodeId>,
    focus: super::focus_manager::FocusPlan,
    layouts: Vec<Vec<crate::component::element::LayoutCallback>>,
}

impl EventTree {
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
        Self::style_node(element.clone(), Vec::new(), 0, focus, hover, width)
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
        focus: Option<&[Slot]>,
        hover: Option<&[Slot]>,
        width: u16,
    ) -> Element {
        Slot::append(&mut path, &element, index);
        if let Some(class) = &element.class {
            element.class = Some(
                class
                    .split_whitespace()
                    .filter_map(|token| {
                        let mut base = token;
                        while let Some((variant, rest)) = base.split_once(':') {
                            let matches = match variant {
                                "focus" => {
                                    !element.metadata.disabled && focus == Some(path.as_slice())
                                }
                                "focus-within" => {
                                    focus.is_some_and(|focused| focused.starts_with(&path))
                                }
                                "hover" => hover.is_some_and(|hovered| hovered.starts_with(&path)),
                                "disabled" => element.metadata.disabled,
                                "sm" => width >= 40,
                                "md" => width >= 80,
                                "lg" => width >= 120,
                                "xl" => width >= 160,
                                _ => break,
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
        element.children = element
            .children
            .into_iter()
            .enumerate()
            .map(|(index, child)| Self::style_node(child, path.clone(), index, focus, hover, width))
            .collect();
        element
    }

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
            router,
            seen: HashSet::new(),
            preorder: Vec::new(),
            focus: super::focus_manager::FocusPlan::new(previous_focus),
            layouts: Vec::new(),
        };
        self.visit(element, Vec::new(), 0, None, &mut frame);
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
        frame.preorder.push(id);
        frame.layouts.push(element.metadata.layout.clone());
        let ancestor_inert = frame.inert;
        let ancestor_keyboard_only = frame.keyboard_only;
        frame.inert |= element.metadata.inert;
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
