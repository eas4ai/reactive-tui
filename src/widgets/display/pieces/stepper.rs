//! The stepper display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-003, DIS-004, DIS-005, DIS-006.
//!
//! A stepper marks the steps before the current one as passed, the current one
//! and the rest as still to come, joined by a line. With a change callback set,
//! the arrows move the focused step and Confirm or a click chooses it.

use std::{any::Any, sync::Arc};

use crate::accessibility::{Node, Role};
use crate::builder::core::{div, span};
use crate::component::{same_callback, Component, Element, FocusProps, Props};
use crate::event::{
    router::EventResult,
    types::{Event, FocusEventKind},
};
use crate::keymap::{Action, Keymap};
use crate::widgets::display::look;
use crate::widgets::display::pieces::icon::Icon;

/// The settings of a stepper. Callbacks are left out of equality, so a rebuild
/// that changes only the callback keeps the mounted stepper (CMP-008).
#[derive(Clone)]
pub struct StepperProps {
    /// The labels of the steps, in order.
    pub steps: Vec<String>,
    /// The current step, 1-based.
    pub current: usize,
    /// Stack the steps top to bottom, joined by `│`, instead of in a row.
    pub vertical: bool,
    /// Classes added to the stepper's element, after its own.
    pub class: String,
    /// Called with the step, 1-based, when a click or Confirm chooses it (CMP-009).
    pub on_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Default for StepperProps {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            current: 1,
            vertical: false,
            class: String::new(),
            on_change: None,
        }
    }
}

impl PartialEq for StepperProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            steps,
            current,
            vertical,
            class,
            on_change: _,
        } = self;
        *steps == other.steps
            && *current == other.current
            && *vertical == other.vertical
            && *class == other.class
    }
}

impl Props for StepperProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub(crate) struct StepperState {
    focused: bool,
    /// The focused step, 0-based.
    index: usize,
}

/// The mark and the color of the step numbered `number` (1-based) when
/// `current` is the current one (DIS-001, DIS-003).
pub(crate) fn mark(number: usize, current: usize) -> (Icon, &'static str) {
    if number < current {
        (Icon::Check, "text-success")
    } else if number == current {
        (Icon::Dot, look::TEXT)
    } else {
        (Icon::Circle, look::MUTED)
    }
}

pub(crate) struct Stepper;

impl Component for Stepper {
    type Props = StepperProps;
    type State = StepperState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let count = props.steps.len();
        let current = props.current.clamp(1, count.max(1));
        let mut children = Vec::new();
        for (index, label) in props.steps.iter().enumerate() {
            let number = index + 1;
            if index > 0 {
                let connector = if props.vertical { "│" } else { "──" };
                children.push(
                    span()
                        .class(&format!("{} whitespace-pre", look::BORDER))
                        .text(connector)
                        .build(),
                );
            }
            let (icon, color) = mark(number, current);
            let focus_ring = if state.focused && state.index == index {
                " ring"
            } else {
                ""
            };
            let mut node = Node::new(Role::ListItem);
            node.set_label(label.clone());
            node.inner.set_position_in_set(number);
            node.inner.set_size_of_set(count);
            if number == current {
                node.inner.set_aria_current(accesskit::AriaCurrent::Step);
            }
            let mut step = if props.vertical {
                span()
                    .class(&format!("{color}{focus_ring}"))
                    .text(&format!("{} {label}", icon.glyph()))
                    .build()
            } else {
                div()
                    .class(&format!("flex flex-row shrink-0{focus_ring}"))
                    .children(vec![
                        span().class(color).text(icon.glyph()).build(),
                        span().class(color).text(&format!(" {label}")).build(),
                    ])
                    .build()
            };
            if let Some(callback) = props.on_change.clone() {
                step = div()
                    .class("flex flex-row")
                    .child(step)
                    .on_click(move || callback(number))
                    .build();
            }
            children.push(step.with_accessibility(node).with_focus(not_focusable()));
        }

        let mut root = if props.vertical {
            div().class("flex flex-col")
        } else {
            div().class("flex flex-row")
        };
        // Horizontal fills the width its parent allots, vertical the height,
        // unless the classes set the size (DIS-002).
        let sized = |prefix: &str| {
            props
                .class
                .split_whitespace()
                .any(|t| t.starts_with(prefix))
        };
        let fill = match (props.vertical, sized("w-"), sized("h-")) {
            (false, false, _) => "w-full",
            (true, _, false) => "h-full",
            _ => "",
        };
        root = root
            .class(&format!("{fill} {}", props.class))
            .children(children);
        let mut list = Node::new(Role::List);
        list.set_label("Steps");
        let element = root.build().with_accessibility(list);
        if props.on_change.is_some() {
            element.with_focus(FocusProps::input())
        } else {
            element
        }
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(focus) = event {
            match focus.kind {
                FocusEventKind::Gained => {
                    state.focused = true;
                    state.index = props.current.clamp(1, props.steps.len().max(1)) - 1;
                }
                FocusEventKind::Lost => state.focused = false,
                _ => return EventResult::Ignored,
            }
            return EventResult::Consumed;
        }
        let (Event::Key(key), Some(callback)) = (event, props.on_change.clone()) else {
            return EventResult::Ignored;
        };
        let last = props.steps.len().saturating_sub(1);
        let (back, forward) = if props.vertical {
            (Action::Up, Action::Down)
        } else {
            (Action::Left, Action::Right)
        };
        match Keymap::active().action(key) {
            Some(action) if action == back => {
                state.index = state.index.saturating_sub(1);
            }
            Some(action) if action == forward => {
                state.index = (state.index + 1).min(last);
            }
            Some(Action::Confirm | Action::Activate) if !props.steps.is_empty() => {
                callback(state.index + 1);
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}

/// Focus props that take no focus: a pointer target that is not a stop of the keyboard.
fn not_focusable() -> FocusProps {
    FocusProps {
        focusable: false,
        ..FocusProps::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dis_001_the_marks_pass_the_current_step_and_leave_the_rest() {
        assert_eq!(mark(1, 2), (Icon::Check, "text-success"));
        assert_eq!(mark(2, 2), (Icon::Dot, look::TEXT));
        assert_eq!(mark(3, 2), (Icon::Circle, look::MUTED));
    }

    #[test]
    fn dis_005_the_marks_are_the_catalog_check_dot_and_circle() {
        assert_eq!(Icon::Check.unicode(), "✓");
        assert_eq!(Icon::Dot.unicode(), "●");
        assert_eq!(Icon::Circle.unicode(), "○");
    }

    #[test]
    fn dis_004_a_step_is_a_list_item_with_its_position_and_count() {
        let props = StepperProps {
            steps: vec!["Compose".into(), "Capture".into(), "Review".into()],
            current: 2,
            ..StepperProps::default()
        };
        let element = render_steps(&props);
        let items: Vec<_> = element
            .children
            .iter()
            .filter_map(|child| child.metadata.accessibility.as_ref())
            .filter(|node| node.role() == Role::ListItem)
            .collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[1].inner.label(), Some("Capture"));
        assert_eq!(items[1].inner.position_in_set(), Some(2));
        assert_eq!(items[1].inner.size_of_set(), Some(3));
    }

    fn render_steps(props: &StepperProps) -> Element {
        Stepper.render(props, &StepperState::default())
    }
}
