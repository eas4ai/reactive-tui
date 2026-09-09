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
    autofocus: Option<NodeId>,
}

impl EventTree {
    pub(super) fn sync(
        &mut self,
        element: &Element,
        geometry: &[PaintedNode],
        router: &mut EventRouter,
    ) {
        let mut frame = Registration {
            router,
            seen: HashSet::new(),
            preorder: Vec::new(),
            autofocus: None,
        };
        self.visit(element, Vec::new(), 0, None, &mut frame);
        let removed: Vec<_> = self
            .nodes
            .keys()
            .filter(|path| !frame.seen.contains(*path))
            .cloned()
            .collect();
        for path in removed {
            if let Some(id) = self.nodes.remove(&path) {
                frame.router.remove_node(id);
            }
        }
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
        if frame.router.get_focus().is_none() {
            if let Some(id) = frame.autofocus {
                frame.router.set_focus(Some(id));
            }
        }
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
        for (index, child) in element.children.iter().enumerate() {
            self.visit(child, path.clone(), index, Some(id), frame);
        }
    }

    pub(super) fn clear(&mut self, router: &mut EventRouter) {
        for (_, id) in self.nodes.drain() {
            router.remove_node(id);
        }
    }
}

impl Registration<'_> {
    fn register(&mut self, element: &Element, id: NodeId) {
        self.router.clear_handlers(id);
        self.router.remove_hit_target(id);
        let interactive = !element.metadata.on_click.is_empty();
        let focusable = element
            .focus
            .as_ref()
            .map_or(interactive, |focus| focus.focusable);
        if focusable {
            self.router
                .add_focusable(id, element.focus.as_ref().map(|focus| focus.tab_index));
        } else {
            self.router.remove_focusable(id);
        }
        if element.focus.as_ref().is_some_and(|focus| focus.auto_focus)
            && focusable
            && self.autofocus.is_none()
        {
            self.autofocus = Some(id);
        }
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
}
