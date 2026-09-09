//! Declarative focus bookkeeping; the router is the only owner of current focus.

use crate::{
    component::Element,
    event::router::{EventRouter, NodeId},
};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct FocusManager {
    /// Creation order, retained when keyed containers move in the rendered tree.
    traps: Vec<NodeId>,
    autofocus: HashSet<NodeId>,
}

#[derive(Default)]
pub(super) struct FocusPlan {
    order: Vec<NodeId>,
    autofocus: Vec<NodeId>,
    traps: Vec<Trap>,
    ancestors: Vec<usize>,
}

struct Trap {
    id: NodeId,
    members: Vec<NodeId>,
    restore: bool,
    focusable: bool,
}

impl FocusPlan {
    pub(super) fn enter(&mut self, element: &Element, id: NodeId) -> bool {
        self.order.push(id);
        let focusable = !element.metadata.disabled
            && element
                .focus
                .as_ref()
                .map_or(!element.metadata.on_click.is_empty(), |focus| {
                    focus.focusable
                });
        if focusable {
            for &index in &self.ancestors {
                self.traps[index].members.push(id);
            }
        }
        let Some(focus) = &element.focus else {
            return false;
        };
        if focus.auto_focus && focusable && !focus.trap_focus {
            self.autofocus.push(id);
        }
        if !focus.trap_focus {
            return false;
        }
        self.ancestors.push(self.traps.len());
        self.traps.push(Trap {
            id,
            members: Vec::new(),
            restore: focus.restore_focus,
            focusable,
        });
        true
    }

    pub(super) fn leave(&mut self, trap: bool) {
        if trap {
            self.ancestors.pop();
        }
    }
}

impl FocusManager {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn apply(&mut self, router: &mut EventRouter, plan: FocusPlan) {
        router.set_focus_order(&plan.order);
        let present: HashSet<_> = plan.traps.iter().map(|trap| trap.id).collect();
        for &id in self.traps.iter().rev() {
            if !present.contains(&id) {
                router.remove_focus_trap(id);
            }
        }
        self.traps.retain(|id| present.contains(id));
        for mut trap in plan.traps {
            if trap.members.is_empty() && trap.focusable {
                trap.members.push(trap.id);
            }
            let preferred = plan
                .autofocus
                .iter()
                .find(|id| trap.members.contains(id))
                .copied();
            router.set_declarative_trap(trap.id, trap.members, trap.restore, preferred);
            if !self.traps.contains(&trap.id) {
                self.traps.push(trap.id);
            }
        }
        if let Some(id) = plan.autofocus.iter().find(|id| {
            (!self.autofocus.contains(id) || router.get_focus().is_none()) && router.can_focus(**id)
        }) {
            router.set_focus(Some(*id));
        }
        self.autofocus = plan.autofocus.into_iter().collect();
    }
}
