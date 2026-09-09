//! Connect owned element callbacks to acknowledged painter geometry.

use crate::{
    backend::PaintedNode,
    component::Element,
    event::{
        router::{EventPhase, EventResult, EventRouter, NodeId},
        types::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Hash, Eq, PartialEq)]
enum Slot {
    Key(String),
    Index(usize),
}

#[derive(Default)]
pub(super) struct EventTree {
    nodes: HashMap<Vec<Slot>, NodeId>,
}

struct Registration<'a> {
    router: &'a mut EventRouter,
    seen: HashSet<Vec<Slot>>,
    preorder: Vec<NodeId>,
    focus: super::focus_manager::FocusPlan,
}

impl EventTree {
    /// Resolve state variants without publishing a candidate event tree.
    pub(super) fn styled(&self, element: &Element, router: &EventRouter) -> Element {
        let focus = router.get_focus().and_then(|id| self.path_for(id));
        let hover = router.hovered_node().and_then(|id| self.path_for(id));
        Self::style_node(element.clone(), Vec::new(), 0, focus, hover)
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
    ) -> Element {
        path.push(
            element
                .key
                .as_ref()
                .map_or(Slot::Index(index), |key| Slot::Key(key.clone())),
        );
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
            .map(|(index, child)| Self::style_node(child, path.clone(), index, focus, hover))
            .collect();
        element
    }

    pub(super) fn sync(
        &mut self,
        element: &Element,
        geometry: &[PaintedNode],
        router: &mut EventRouter,
    ) -> super::focus_manager::FocusPlan {
        let mut frame = Registration {
            router,
            seen: HashSet::new(),
            preorder: Vec::new(),
            focus: super::focus_manager::FocusPlan::default(),
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
        for (order, node) in geometry.iter().enumerate() {
            if let Some(&id) = frame.preorder.get(node.element_index) {
                frame
                    .router
                    .add_hit_target(id, node.bounds, order.min(i32::MAX as usize) as i32);
                frame.router.update_spatial(
                    id,
                    node.bounds.x,
                    node.bounds.y,
                    node.bounds.width,
                    node.bounds.height,
                );
            }
        }
        frame.focus
    }

    fn visit(
        &mut self,
        element: &Element,
        mut path: Vec<Slot>,
        index: usize,
        parent: Option<NodeId>,
        frame: &mut Registration<'_>,
    ) {
        path.push(
            element
                .key
                .as_ref()
                .map_or(Slot::Index(index), |key| Slot::Key(key.clone())),
        );
        let id = *self
            .nodes
            .entry(path.clone())
            .or_insert_with(|| frame.router.create_node(parent));
        frame.seen.insert(path.clone());
        frame.preorder.push(id);
        frame.register(element, id);
        let trap = frame.focus.enter(element, id);
        for (index, child) in element.children.iter().enumerate() {
            self.visit(child, path.clone(), index, Some(id), frame);
        }
        frame.focus.leave(trap);
    }

    pub(super) fn clear(&mut self, router: &mut EventRouter) {
        let nodes: Vec<_> = self.nodes.drain().map(|(_, id)| id).collect();
        router.remove_nodes(&nodes);
    }
}

impl Registration<'_> {
    fn register(&mut self, element: &Element, id: NodeId) {
        self.router.remove_hit_target(id);
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
        if interactive {
            let callbacks = element.metadata.on_click.clone();
            let handler: crate::event::router::EventHandlerFn = Arc::new(move |event| {
                let activate = match event {
                    Event::Key(key) => {
                        key.kind == KeyEventKind::Press
                            && !key.repeat
                            && key.modifiers.is_empty()
                            && matches!(
                                key.code,
                                KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ')
                            )
                    }
                    Event::Mouse(mouse) => {
                        mouse.button == MouseButton::Left
                            && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                    }
                    _ => false,
                };
                if !activate {
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
