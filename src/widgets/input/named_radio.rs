//! Single-radio builder controls and their App-owned group membership.

use crate::component::{Component, Element, FocusProps, LifecycleEvent, Props};
use crate::event::{
    router::EventResult,
    types::{Event, FocusEventKind, KeyCode, MouseButton, MouseEventKind},
};
use std::{
    any::Any,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Weak,
    },
};

#[derive(Default)]
pub(crate) struct RadioGroups(Mutex<HashMap<String, Vec<Weak<AtomicBool>>>>);

impl RadioGroups {
    fn join(&self, name: &str, member: &Arc<AtomicBool>) {
        let mut groups = self.0.lock().unwrap();
        let members = groups.entry(name.to_owned()).or_default();
        members.retain(|member| member.strong_count() > 0);
        if members
            .iter()
            .filter_map(Weak::upgrade)
            .any(|member| member.load(Ordering::Relaxed))
        {
            member.store(false, Ordering::Relaxed);
        }
        members.push(Arc::downgrade(member));
    }

    fn leave(&self, name: &str, member: &Arc<AtomicBool>) {
        let mut groups = self.0.lock().unwrap();
        if let Some(members) = groups.get_mut(name) {
            let token = Arc::downgrade(member);
            members.retain(|other| other.strong_count() > 0 && !other.ptr_eq(&token));
            if members.is_empty() {
                groups.remove(name);
            }
        }
    }

    fn select(&self, name: &str, selected: &Arc<AtomicBool>) {
        let groups = self.0.lock().unwrap();
        if let Some(members) = groups.get(name) {
            for member in members.iter().filter_map(Weak::upgrade) {
                member.store(Arc::ptr_eq(&member, selected), Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct NamedRadioProps {
    pub value: String,
    pub label: Option<String>,
    pub checked: bool,
    pub disabled: bool,
    pub group: Option<String>,
}
impl Props for NamedRadioProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub(crate) struct NamedRadioState {
    focused: bool,
}

pub(crate) struct NamedRadio {
    groups: Arc<RadioGroups>,
    group: Option<String>,
    selected: Arc<AtomicBool>,
}

impl NamedRadio {
    fn leave(&mut self) {
        if let Some(group) = self.group.take() {
            self.groups.leave(&group, &self.selected);
        }
    }
}

impl Component for NamedRadio {
    type Props = NamedRadioProps;
    type State = NamedRadioState;

    fn new(props: Self::Props) -> Self {
        let groups =
            crate::reactive::component_scope::lookup::<Arc<RadioGroups>>().unwrap_or_default();
        let selected = Arc::new(AtomicBool::new(props.checked));
        if let Some(group) = &props.group {
            groups.join(group, &selected);
        }
        Self {
            groups,
            group: props.group,
            selected,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if props.group != self.group {
            self.leave();
            self.group = props.group.clone();
            if let Some(group) = &self.group {
                self.groups.join(group, &self.selected);
            }
        }
        if props.disabled {
            *state = NamedRadioState::default();
        }
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let selected = self.selected.load(Ordering::Relaxed);
        let label = props.label.as_deref().unwrap_or(&props.value);
        use crate::accessibility::{Node, Role, Toggled};
        let mut accessible = Node::new(Role::RadioButton);
        accessible.set_label(label);
        accessible.set_toggled(if selected {
            Toggled::True
        } else {
            Toggled::False
        });
        if props.disabled {
            accessible.set_disabled();
        } else {
            accessible.set_clickable();
        }
        Element::text(format!(
            "{}({}) {}",
            if state.focused { "▶ " } else { "  " },
            if selected { '●' } else { '○' },
            label
        ))
        .with_accessibility(accessible)
        .with_class("whitespace-pre overflow-hidden")
        .with_focus(FocusProps::input())
        .disabled(props.disabled)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(focus) = event {
            if !matches!(focus.kind, FocusEventKind::Gained | FocusEventKind::Lost) {
                return EventResult::Ignored;
            }
            state.focused = focus.kind == FocusEventKind::Gained && !props.disabled;
            return EventResult::Consumed;
        }
        if props.disabled {
            return EventResult::Ignored;
        }
        let select = match event {
            Event::Key(key) => {
                state.focused
                    && matches!(
                        key.code,
                        KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ')
                    )
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down | MouseEventKind::Click => mouse.button == MouseButton::Left,
                _ => false,
            },
            _ => false,
        };
        if !select {
            return EventResult::Ignored;
        }
        if let Some(group) = &self.group {
            self.groups.select(group, &self.selected);
        } else {
            self.selected.store(true, Ordering::Relaxed);
        }
        EventResult::Consumed
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if matches!(event, LifecycleEvent::Unmount) {
            self.leave();
        }
    }
}

impl Drop for NamedRadio {
    fn drop(&mut self) {
        self.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::{
        component_scope::{provide, ComponentScope},
        scheduler::Scheduler,
    };

    #[test]
    fn unmount_releases_membership_before_the_instance_is_dropped() {
        let scope = ComponentScope::new(Arc::new(Scheduler::new()));
        let _binding = scope.enter(true);
        let groups = Arc::new(RadioGroups::default());
        assert!(provide(groups.clone()).is_ok());
        let props = NamedRadioProps {
            value: "one".into(),
            label: None,
            checked: true,
            disabled: false,
            group: Some("choice".into()),
        };
        let mut radio = NamedRadio::new(props.clone());
        assert_eq!(groups.0.lock().unwrap().len(), 1);
        radio.on_lifecycle(LifecycleEvent::Unmount, &mut NamedRadioState::default());
        assert!(groups.0.lock().unwrap().is_empty());
        let replacement = NamedRadio::new(props);
        assert!(replacement.selected.load(Ordering::Relaxed));
        drop(radio);
        assert_eq!(groups.0.lock().unwrap().len(), 1);
        drop(replacement);
        assert!(groups.0.lock().unwrap().is_empty());
    }
}
