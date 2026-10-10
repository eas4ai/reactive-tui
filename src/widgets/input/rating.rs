use super::look;
use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{CustomEvent, Event, MouseEvent};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;
use std::sync::Arc;

/// The events a rating takes for the screen reader's requests (CTL-005,
/// CTL-010): SetValue, whose data is the requested number as text,
/// Increment and Decrement.
const SET_VALUE_EVENT: &str = "reactive_tui.rating.set_value";
const INCREMENT_EVENT: &str = "reactive_tui.rating.increment";
const DECREMENT_EVENT: &str = "reactive_tui.rating.decrement";

/// Properties for a Rating component: a row of `max` stars, the first
/// `value` of them lit (CTL-007).
#[derive(Clone, Debug, PartialEq)]
pub struct RatingProps {
    /// The number of lit stars, from 0 to `max`
    pub value: usize,
    /// The number of stars
    pub max: usize,
    /// The name the screen reader hears
    pub aria_label: Option<String>,
    /// Whether the rating is disabled
    pub disabled: bool,
}

impl Default for RatingProps {
    fn default() -> Self {
        Self {
            value: 0,
            max: 5,
            aria_label: None,
            disabled: false,
        }
    }
}

impl Props for RatingProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for a Rating component
#[derive(Clone, Debug, Default)]
pub struct RatingState {
    /// Whether the rating has keyboard focus
    pub is_focused: bool,
}

/// A rating: stars the user sets by key, digit or click (CTL-009).
pub struct Rating {
    viewport: Option<crate::component::LayoutInfo>,
    on_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Rating {
    /// Create a rating from its props, with no callback
    pub fn new(_props: RatingProps) -> Self {
        Self {
            viewport: None,
            on_change: None,
        }
    }

    /// Set the callback called with the new value on each change
    pub fn with_on_change(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Replace the onChange callback, or remove it with `None`: a builder's
    /// rating takes the one its latest render carried (CMP-009).
    pub(crate) fn set_on_change(&mut self, f: Option<Arc<dyn Fn(usize) + Send + Sync>>) {
        self.on_change = f;
    }

    /// The lit stars: the value, never more than `max`.
    fn lit(props: &RatingProps) -> usize {
        props.value.min(props.max)
    }

    /// Set the value and report the change; an unchanged value reports
    /// nothing.
    fn change(&self, value: usize, props: &mut RatingProps) {
        if value != props.value {
            props.value = value;
            if let Some(callback) = &self.on_change {
                callback(value);
            }
        }
    }

    /// The star number (1 for the first) under a click, or none when the
    /// click falls off the stars' row.
    fn star_at(&self, event: &MouseEvent, max: usize) -> Option<usize> {
        let insets = self.viewport.map_or([0.0; 4], |v| v.insets);
        let x = event.position.x() as f32 - insets[0];
        let y = event.position.y() as f32 - insets[1];
        if !(0.0..1.0).contains(&y) || x < 0.0 || x >= max as f32 {
            return None;
        }
        Some(x as usize + 1)
    }
}

impl Component for Rating {
    type Props = RatingProps;
    type State = RatingState;

    fn new(props: Self::Props) -> Self {
        Rating::new(props)
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if props.disabled {
            *state = RatingState::default();
        }
        true
    }

    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        self.viewport = Some(bounds);
        false
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let focused = state.is_focused && !props.disabled;
        let lit = Self::lit(props);
        // A lit star is `text-warning`, an unlit one `border` (`ring` while
        // focused); all of them `text-muted` while disabled. The glyph is
        // the star or the outline, never a change of text (CTL-001, CTL-007).
        let stars: Vec<(&str, &str)> = (0..props.max)
            .map(|index| {
                let glyph = if index < lit {
                    Icon::Star.glyph()
                } else {
                    Icon::StarOutline.glyph()
                };
                let classes = if props.disabled {
                    look::LABEL_DISABLED
                } else if index < lit {
                    look::RATING_LIT
                } else if focused {
                    look::RATING_UNLIT_FOCUSED
                } else {
                    look::RATING_UNLIT
                };
                (glyph, classes)
            })
            .collect();

        use crate::accessibility::{Node, Role};
        let mut accessible = Node::new(Role::Slider);
        if let Some(label) = &props.aria_label {
            accessible.set_label(label.clone());
        }
        accessible.inner.set_numeric_value(lit as f64);
        accessible.inner.set_min_numeric_value(0.0);
        accessible.inner.set_max_numeric_value(props.max as f64);
        accessible.inner.set_numeric_value_step(1.0);
        accessible.set_value(format!("{lit} of {}", props.max));
        if props.disabled {
            accessible.set_disabled();
        }
        // The screen reader moves the rating a step either way or sets its
        // value; a rating that cannot move advertises neither (CTL-005).
        let movable = !props.disabled && props.max > 0;
        if movable {
            for action in [
                accesskit::Action::Increment,
                accesskit::Action::Decrement,
                accesskit::Action::SetValue,
            ] {
                accessible.inner.add_action(action);
            }
        }
        let mut element = look::row_of(look::pieces(&stars), "")
            .with_accessibility(accessible)
            .with_focus(crate::component::FocusProps::input())
            .disabled(props.disabled || props.max == 0);
        if movable {
            let options = element
                .metadata
                .accessibility_options
                .get_or_insert_default();
            options.set_value_event = Some(CustomEvent::new(SET_VALUE_EVENT, Vec::new()));
            options.increment_event = Some(CustomEvent::new(INCREMENT_EVENT, Vec::new()));
            options.decrement_event = Some(CustomEvent::new(DECREMENT_EVENT, Vec::new()));
        }
        element
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(event) = event {
            if !matches!(
                event.kind,
                crate::event::types::FocusEventKind::Gained
                    | crate::event::types::FocusEventKind::Lost
            ) {
                return EventResult::Ignored;
            }
            state.is_focused = event.kind == crate::event::types::FocusEventKind::Gained
                && !props.disabled
                && props.max > 0;
            return EventResult::Consumed;
        }
        if props.disabled || props.max == 0 {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(event) if state.is_focused => self.handle_key_event(event, props),
            Event::Mouse(event) => self.handle_mouse_event(event, props),
            Event::Custom(event) => self.handle_request(event, props),
            _ => EventResult::Ignored,
        }
    }
}

impl Rating {
    /// Take the screen reader's request (CTL-005): Increment and Decrement
    /// move one star as the arrows do, SetValue takes the requested number
    /// rounded to a star and clamped to the range. A SetValue that names no
    /// finite number changes nothing.
    fn handle_request(&mut self, event: &CustomEvent, props: &mut RatingProps) -> EventResult {
        let value = Self::lit(props);
        let new_value = match event.name.as_str() {
            INCREMENT_EVENT => (value + 1).min(props.max),
            DECREMENT_EVENT => value.saturating_sub(1),
            SET_VALUE_EVENT => {
                let requested = std::str::from_utf8(&event.data)
                    .ok()
                    .and_then(|text| text.trim().parse::<f64>().ok())
                    .filter(|requested| requested.is_finite());
                match requested {
                    Some(requested) => requested.round().clamp(0.0, props.max as f64) as usize,
                    None => return EventResult::Ignored,
                }
            }
            _ => return EventResult::Ignored,
        };
        self.change(new_value, props);
        EventResult::Consumed
    }

    fn handle_key_event(&mut self, event: &KeyEvent, props: &mut RatingProps) -> EventResult {
        // The arrows, Home and End mean what the active keymap says (KEY-001);
        // a digit is the rating's own key and matches its character.
        use crate::keymap::Action;
        let value = Self::lit(props);
        let new_value = match crate::keymap::Keymap::active().action(event) {
            Some(Action::Left | Action::Down) => value.saturating_sub(1),
            Some(Action::Right | Action::Up) => (value + 1).min(props.max),
            Some(Action::Home) => 0,
            Some(Action::End) => props.max,
            _ => match event.code {
                KeyCode::Char(digit) if digit.is_ascii_digit() && event.modifiers.is_empty() => {
                    (digit as usize - '0' as usize).min(props.max)
                }
                _ => return EventResult::Ignored,
            },
        };
        self.change(new_value, props);
        EventResult::Consumed
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent, props: &mut RatingProps) -> EventResult {
        use crate::event::types::MouseButton;
        if event.kind != MouseEventKind::Down || event.button != MouseButton::Left {
            return EventResult::Ignored;
        }
        let Some(star) = self.star_at(event, props.max) else {
            return EventResult::Ignored;
        };
        self.change(star, props);
        EventResult::Consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyEventKind, KeyModifiers, MouseButton, Position};
    use std::sync::Mutex;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        })
    }

    fn click(column: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down,
            button: MouseButton::Left,
            position: Position::cell(column, 0),
            modifiers: KeyModifiers::empty(),
            timestamp: std::time::Instant::now(),
            wheel: None,
        })
    }

    fn request(name: &str, data: &str) -> Event {
        Event::Custom(CustomEvent::new(name, data.as_bytes().to_vec()))
    }

    /// A focused rating of five stars at `value`, with the values its
    /// callback reported.
    fn focused(value: usize) -> (Rating, RatingProps, RatingState, Arc<Mutex<Vec<usize>>>) {
        let reported = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&reported);
        let rating = Rating::new(RatingProps::default()).with_on_change(move |value| {
            sink.lock().unwrap().push(value);
        });
        let props = RatingProps {
            value,
            ..RatingProps::default()
        };
        let state = RatingState { is_focused: true };
        (rating, props, state, reported)
    }

    /// CTL-009: the arrows move one star, Home and End jump to 0 and the
    /// maximum, each change is reported, and an unchanged value reports
    /// nothing.
    #[test]
    fn ctl_009_keys_move_the_value_by_one_and_jump_to_the_ends() {
        let (mut rating, mut props, mut state, reported) = focused(3);
        for (code, expected) in [
            (KeyCode::Right, 4),
            (KeyCode::Up, 5),
            (KeyCode::Right, 5),
            (KeyCode::Left, 4),
            (KeyCode::Down, 3),
            (KeyCode::Home, 0),
            (KeyCode::Home, 0),
            (KeyCode::End, 5),
        ] {
            let result = rating.handle_event(&key(code.clone()), &mut props, &mut state);
            assert_eq!(result, EventResult::Consumed);
            assert_eq!(props.value, expected, "after {code:?}");
        }
        assert_eq!(
            *reported.lock().unwrap(),
            vec![4, 5, 4, 3, 0, 5],
            "CTL-009: each change is reported once"
        );
    }

    /// CTL-009: a digit sets the value, clamped to the maximum.
    #[test]
    fn ctl_009_a_digit_sets_the_value_clamped_to_max() {
        let (mut rating, mut props, mut state, reported) = focused(0);
        rating.handle_event(&key(KeyCode::Char('2')), &mut props, &mut state);
        assert_eq!(props.value, 2);
        rating.handle_event(&key(KeyCode::Char('9')), &mut props, &mut state);
        assert_eq!(props.value, 5, "a digit above max is clamped");
        assert_eq!(*reported.lock().unwrap(), vec![2, 5]);
    }

    /// CTL-009: a click on star n sets n, and a click off the stars changes
    /// nothing.
    #[test]
    fn ctl_009_a_click_on_a_star_sets_its_number() {
        let (mut rating, mut props, mut state, reported) = focused(0);
        rating.handle_event(&click(4), &mut props, &mut state);
        assert_eq!(props.value, 5, "the fifth star's click sets 5");
        rating.handle_event(&click(0), &mut props, &mut state);
        assert_eq!(props.value, 1, "the first star's click sets 1");
        let result = rating.handle_event(&click(7), &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored, "a click past the stars");
        assert_eq!(props.value, 1);
        assert_eq!(*reported.lock().unwrap(), vec![5, 1]);
    }

    /// CTL-009: a disabled rating takes no key, digit or click.
    #[test]
    fn ctl_009_a_disabled_rating_takes_nothing() {
        let mut rating = Rating::new(RatingProps::default());
        let mut props = RatingProps {
            value: 3,
            disabled: true,
            ..RatingProps::default()
        };
        let mut state = RatingState { is_focused: true };
        for event in [key(KeyCode::Right), key(KeyCode::Char('1')), click(4)] {
            let result = rating.handle_event(&event, &mut props, &mut state);
            assert_eq!(result, EventResult::Ignored);
        }
        assert_eq!(props.value, 3);
    }

    /// CTL-010: the rating is a slider whose value reads `n of m`, with a
    /// minimum of 0, a maximum of `max` and a step of 1, and it advertises
    /// Increment, Decrement and SetValue.
    #[test]
    fn ctl_010_a_rating_is_a_slider_reading_n_of_m() {
        let props = RatingProps {
            value: 3,
            aria_label: Some("Quality".to_string()),
            ..RatingProps::default()
        };
        let element = Rating::new(props.clone()).render(&props, &RatingState::default());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the rating has a node");
        assert_eq!(node.inner.role(), accesskit::Role::Slider);
        assert_eq!(node.inner.label(), Some("Quality"));
        assert_eq!(node.inner.value(), Some("3 of 5"));
        assert_eq!(node.inner.numeric_value(), Some(3.0));
        assert_eq!(node.inner.min_numeric_value(), Some(0.0));
        assert_eq!(node.inner.max_numeric_value(), Some(5.0));
        assert_eq!(node.inner.numeric_value_step(), Some(1.0));
        for action in [
            accesskit::Action::Increment,
            accesskit::Action::Decrement,
            accesskit::Action::SetValue,
        ] {
            assert!(
                node.inner.supports_action(action),
                "CTL-010: the rating advertises {action:?}"
            );
        }
    }

    /// CTL-010: the screen reader's requests move the rating: Increment and
    /// Decrement one star, SetValue a number rounded and clamped.
    #[test]
    fn ctl_010_the_screen_readers_requests_move_the_rating() {
        let (mut rating, mut props, mut state, _) = focused(3);
        rating.handle_event(&request(INCREMENT_EVENT, ""), &mut props, &mut state);
        assert_eq!(props.value, 4, "Increment moves one star");
        rating.handle_event(&request(DECREMENT_EVENT, ""), &mut props, &mut state);
        assert_eq!(props.value, 3, "Decrement moves one star");
        rating.handle_event(&request(SET_VALUE_EVENT, "4"), &mut props, &mut state);
        assert_eq!(props.value, 4, "SetValue of 4 shows four stars");
        rating.handle_event(&request(SET_VALUE_EVENT, "9"), &mut props, &mut state);
        assert_eq!(props.value, 5, "SetValue above max is clamped");
        let result = rating.handle_event(&request(SET_VALUE_EVENT, "x"), &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored, "a SetValue naming no number");
        assert_eq!(props.value, 5);
    }

    /// CTL-010: a disabled rating's node is disabled and advertises nothing.
    #[test]
    fn ctl_010_a_disabled_rating_is_disabled_and_advertises_no_action() {
        let props = RatingProps {
            value: 3,
            disabled: true,
            ..RatingProps::default()
        };
        let element = Rating::new(props.clone()).render(&props, &RatingState::default());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the rating has a node");
        assert!(
            node.inner.is_disabled(),
            "CTL-010: a disabled rating is disabled"
        );
        assert!(
            !node.inner.supports_action(accesskit::Action::Increment),
            "CTL-010: a disabled rating advertises no Increment"
        );
    }
}
