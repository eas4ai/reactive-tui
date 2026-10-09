//! Single-radio builder controls and their App-owned group membership.

use crate::component::{Component, Element, FocusProps, LifecycleEvent, Props};
use crate::event::{
    router::EventResult,
    types::{Event, FocusEventKind, MouseButton, MouseEventKind},
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

#[derive(Clone)]
pub(crate) struct NamedRadioProps {
    pub value: String,
    pub label: Option<String>,
    pub aria_label: Option<String>,
    pub checked: bool,
    pub disabled: bool,
    pub group: Option<String>,
    /// Called with `value` when the user chooses this radio while it was
    /// not chosen (CMP-009).
    pub on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

/// Equal over the settings and not the callback, so a rebuild that changes
/// only the callback keeps the mounted radio and its new callback acts from
/// the next event on (CMP-008).
impl PartialEq for NamedRadioProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            value,
            label,
            aria_label,
            checked,
            disabled,
            group,
            on_change: _,
        } = self;
        *value == other.value
            && *label == other.label
            && *aria_label == other.aria_label
            && *checked == other.checked
            && *disabled == other.disabled
            && *group == other.group
    }
}
impl Props for NamedRadioProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub(crate) struct NamedRadioState {
    focused: bool,
    hover: bool,
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

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if crate::component::same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let selected = self.selected.load(Ordering::Relaxed);
        let label = props.label.as_deref().unwrap_or(&props.value);
        use crate::accessibility::{Node, Role, Toggled};
        let mut accessible = Node::new(Role::RadioButton);
        accessible.set_label(props.aria_label.as_deref().unwrap_or(label));
        accessible.set_toggled(if selected {
            Toggled::True
        } else {
            Toggled::False
        });
        accessible.set_selected(selected);
        if props.disabled {
            accessible.set_disabled();
        } else {
            accessible.set_clickable();
        }
        // The same row as a `RadioButton` option: frame, dot, label, and
        // the hover fill (CTL-001).
        use super::look;
        let focused = state.focused && !props.disabled;
        let hover = if state.hover && !props.disabled {
            look::HOVER
        } else {
            ""
        };
        let label_pieces = [(" ", look::LABEL), (label, look::label(props.disabled))];
        // With pixels the circle is a picture over its three cells and no
        // glyph: a bordered circle with a dot when chosen (PIX-004).
        #[cfg(feature = "wgpu-graphics")]
        let pixel_circle = look::pixels().then(|| {
            look::spacer(
                3,
                crate::graphics::look::Look::radio(selected, focused, props.disabled),
            )
        });
        #[cfg(not(feature = "wgpu-graphics"))]
        let pixel_circle: Option<Element> = None;
        let row = match pixel_circle {
            Some(pixel_circle) => {
                let mut children = vec![pixel_circle];
                children.extend(look::pieces(&label_pieces));
                look::row_of(children, hover)
            }
            None => look::row(
                &[
                    ("(", look::frame(focused)),
                    (if selected { "●" } else { " " }, look::MARK),
                    (")", look::frame(focused)),
                    label_pieces[0],
                    label_pieces[1],
                ],
                hover,
            ),
        };
        row.with_accessibility(accessible)
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
        if let Event::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::Enter | MouseEventKind::Move => state.hover = true,
                MouseEventKind::Leave => state.hover = false,
                _ => {}
            }
        }
        if props.disabled {
            return EventResult::Ignored;
        }
        let select = match event {
            // The keys mean what the active keymap says (KEY-001).
            Event::Key(key) => {
                state.focused
                    && matches!(
                        crate::keymap::Keymap::active().action(key),
                        Some(crate::keymap::Action::Confirm | crate::keymap::Action::Activate)
                    )
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down => mouse.button == MouseButton::Left,
                _ => false,
            },
            _ => false,
        };
        if !select {
            return EventResult::Ignored;
        }
        let chosen = !self.selected.load(Ordering::Relaxed);
        if let Some(group) = &self.group {
            self.groups.select(group, &self.selected);
        } else {
            self.selected.store(true, Ordering::Relaxed);
        }
        // As a `RadioButton` does, the callback hears a choice that changes
        // the selection, not a press on the radio already chosen.
        if chosen {
            if let Some(callback) = &props.on_change {
                callback(props.value.clone());
            }
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
            aria_label: None,
            checked: true,
            disabled: false,
            group: Some("choice".into()),
            on_change: None,
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

    /// KEY-001: the radio is chosen on the key the active keymap binds to
    /// Confirm, and no longer on the key it replaced.
    #[test]
    #[serial_test::serial(keymap)]
    fn key_001_named_radio_reads_its_keys_through_the_keymap() {
        use crate::event::types::{KeyCode, KeyEvent};
        use crate::keymap::{Action, KeyBinding, Keymap};
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Confirm, [KeyBinding::new(KeyCode::F(2))]);
        let _scope = Keymap::scoped(keymap);
        let mut props = NamedRadioProps {
            value: "one".into(),
            label: None,
            aria_label: None,
            checked: false,
            disabled: false,
            group: None,
            on_change: None,
        };
        let mut old_radio = NamedRadio::new(props.clone());
        let mut new_radio = NamedRadio::new(props.clone());
        let mut state = NamedRadioState {
            focused: true,
            hover: false,
        };
        let old = old_radio.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::Enter)),
            &mut props,
            &mut state,
        );
        let rebound = new_radio.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::F(2))),
            &mut props,
            &mut state,
        );
        // The scope above restores the default keymap when it drops.
        assert_eq!(rebound, EventResult::Consumed);
        assert!(
            new_radio.selected.load(Ordering::Relaxed),
            "the new Confirm key chooses the radio"
        );
        assert_eq!(old, EventResult::Ignored);
        assert!(
            !old_radio.selected.load(Ordering::Relaxed),
            "Enter, no longer Confirm, does nothing"
        );
    }
}
