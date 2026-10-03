//! Declarative focus bookkeeping; the router is the only owner of current focus.

use crate::{
    component::Element,
    event::router::{EventRouter, NodeId},
};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct FocusManager {
    /// Creation order, retained when keyed containers move in the rendered tree.
    traps: Vec<NodeId>,
    autofocus: HashSet<NodeId>,
    scopes: Vec<ActiveScope>,
}

#[derive(Default)]
pub(crate) struct FocusPlan {
    order: Vec<NodeId>,
    autofocus: Vec<NodeId>,
    traps: Vec<Trap>,
    ancestors: Vec<usize>,
    scopes: Vec<Scope>,
    scope_ancestors: Vec<usize>,
    previous_focus: Option<NodeId>,
}

struct Scope {
    id: NodeId,
    members: Vec<NodeId>,
    fallback: bool,
}

struct ActiveScope {
    scope: Scope,
    restore: Option<NodeId>,
    entered: bool,
}

struct Trap {
    id: NodeId,
    members: Vec<NodeId>,
    restore: bool,
    focusable: bool,
}

impl FocusPlan {
    pub(crate) fn new(previous_focus: Option<NodeId>) -> Self {
        Self {
            previous_focus,
            ..Self::default()
        }
    }

    pub(crate) fn enter(&mut self, element: &Element, id: NodeId) -> (bool, bool) {
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
            for &index in &self.scope_ancestors {
                self.scopes[index].members.push(id);
            }
        }
        let scope = element.metadata.focus_scope;
        if scope {
            self.scope_ancestors.push(self.scopes.len());
            self.scopes.push(Scope {
                id,
                members: Vec::new(),
                fallback: focusable,
            });
        }
        let Some(focus) = &element.focus else {
            return (false, scope);
        };
        if focus.auto_focus && focusable && !focus.trap_focus {
            self.autofocus.push(id);
        }
        if !focus.trap_focus {
            return (false, scope);
        }
        self.ancestors.push(self.traps.len());
        self.traps.push(Trap {
            id,
            members: Vec::new(),
            restore: focus.restore_focus,
            focusable,
        });
        (true, scope)
    }

    pub(crate) fn leave(&mut self, (trap, scope): (bool, bool)) {
        if trap {
            self.ancestors.pop();
        }
        if scope {
            self.scope_ancestors.pop();
        }
    }
}

impl FocusManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn apply(&mut self, router: &mut EventRouter, plan: FocusPlan) {
        router.set_focus_order(&plan.order);
        let present: HashSet<_> = plan.traps.iter().map(|trap| trap.id).collect();
        for &id in self.traps.iter().rev() {
            if !present.contains(&id) {
                router.remove_focus_trap(id);
            }
        }
        self.traps.retain(|id| present.contains(id));
        let mut previous = plan.previous_focus;
        for active in self.scopes.iter().rev() {
            if !plan.scopes.iter().any(|scope| scope.id == active.scope.id)
                && previous
                    .is_none_or(|id| id == active.scope.id || active.scope.members.contains(&id))
            {
                if let Some(id) = active.restore.filter(|id| router.can_focus(*id)) {
                    router.set_focus(Some(id));
                    previous = Some(id);
                }
            }
        }
        self.scopes
            .retain(|active| plan.scopes.iter().any(|scope| scope.id == active.scope.id));
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
        for scope in plan.scopes {
            let active = if let Some(index) = self
                .scopes
                .iter()
                .position(|active| active.scope.id == scope.id)
            {
                self.scopes[index].scope = scope;
                &mut self.scopes[index]
            } else {
                self.scopes.push(ActiveScope {
                    scope,
                    restore: router.get_focus(),
                    entered: false,
                });
                self.scopes.last_mut().unwrap()
            };
            if !active.entered {
                let target = plan
                    .autofocus
                    .iter()
                    .copied()
                    .find(|id| active.scope.members.contains(id) && router.can_focus(*id))
                    .or_else(|| {
                        active
                            .scope
                            .members
                            .iter()
                            .copied()
                            .find(|id| router.can_focus(*id))
                    })
                    .or_else(|| {
                        (active.scope.fallback && router.can_focus(active.scope.id))
                            .then_some(active.scope.id)
                    });
                if let Some(id) = target {
                    router.set_focus(Some(id));
                    active.entered = true;
                }
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
