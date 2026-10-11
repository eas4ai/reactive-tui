//! The number input (docs/spec/input-widgets.md, CTL-007 to CTL-010): a
//! field that holds a number between a `[-]` and a `[+]` button. The whole
//! piece is one focus stop and one screen reader node (a spin button).

use super::look;
use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, FocusProps, LayoutInfo, Props};
use crate::event::router::EventResult;
use crate::event::types::{
    FocusEventKind, KeyCode, KeyEvent, MouseButton, MouseEventKind, WheelDelta,
};
use crate::event::{CustomEvent, Event, MouseEvent};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;
use std::sync::Arc;

/// The events the number input takes for the screen reader's requests
/// (CTL-010): SetValue, whose data is the requested number as text,
/// Increment and Decrement.
const SET_VALUE_EVENT: &str = "reactive_tui.number_input.set_value";
const INCREMENT_EVENT: &str = "reactive_tui.number_input.increment";
const DECREMENT_EVENT: &str = "reactive_tui.number_input.decrement";

/// The cells of the row beside the field: `[-]`, the field's `[` and `]`,
/// and `[+]`.
const CHROME: usize = 3 + 2 + 3;

/// The row's width before the first layout, as a text input's field takes
/// it.
const DEFAULT_WIDTH: usize = 30;

/// Steps moved by Page Up and Page Down.
const PAGE_STEPS: f64 = 10.0;

type Callback = Arc<dyn Fn(f64) + Send + Sync>;

/// Properties for NumberInput component
#[derive(Clone, Debug, PartialEq)]
pub struct NumberInputProps {
    /// The value; it is clamped into `min` and `max` wherever it is used
    pub value: f64,
    /// The least value, or none
    pub min: Option<f64>,
    /// The greatest value, or none
    pub max: Option<f64>,
    /// The size of one step; 1 unless set
    pub step: f64,
    /// The name the screen reader hears
    pub aria_label: Option<String>,
    /// Whether the number input is disabled
    pub disabled: bool,
    /// The row's width in cells; none takes what the parent allots
    pub width: Option<u16>,
}

impl Default for NumberInputProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: None,
            max: None,
            step: 1.0,
            aria_label: None,
            disabled: false,
            width: None,
        }
    }
}

impl Props for NumberInputProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl NumberInputProps {
    /// The step in use: the props' step when it is positive and finite,
    /// else 1.
    fn step(&self) -> f64 {
        if self.step.is_finite() && self.step > 0.0 {
            self.step
        } else {
            1.0
        }
    }

    /// `value` clamped into the range; a value that is not a number reads
    /// as 0, an infinite one as the greatest finite number of its sign, so
    /// a step that overflows keeps its direction (CTL-009), and a range
    /// whose least is above its greatest clamps nothing.
    fn clamp(&self, value: f64) -> f64 {
        let value = if value.is_nan() {
            0.0
        } else {
            value.clamp(f64::MIN, f64::MAX)
        };
        let low = self.min.unwrap_or(f64::NEG_INFINITY);
        let high = self.max.unwrap_or(f64::INFINITY);
        if low <= high {
            value.clamp(low, high)
        } else {
            value
        }
    }

    /// The digits after the point in the step's shortest form: 1 has 0,
    /// 0.5 has 1 and 0.25 has 2. The value is written with that many.
    fn decimals(&self) -> usize {
        self.step()
            .to_string()
            .split_once('.')
            .map_or(0, |(_, digits)| digits.len())
    }

    /// `value`, clamped and written in the step's decimals.
    fn format(&self, value: f64) -> String {
        format!("{:.*}", self.decimals(), self.clamp(value))
    }

    /// `value` snapped to the nearest step and clamped into the range, as
    /// `Slider::snap` takes a value: the steps count from `min`, or from 0
    /// when there is no least value (CTL-005, CTL-010).
    fn snap(&self, value: f64) -> f64 {
        let step = self.step();
        let origin = self.min.unwrap_or(0.0);
        let snapped = origin + ((value - origin) / step).round() * step;
        self.clamp(if snapped.is_finite() { snapped } else { value })
    }

    /// Whether a step up (`up`) or down from `value` moves it.
    fn can_step(&self, value: f64, up: bool) -> bool {
        if up {
            self.max.is_none_or(|max| value < max)
        } else {
            self.min.is_none_or(|min| value > min)
        }
    }
}

/// State for NumberInput component
#[derive(Clone, Debug, Default)]
pub struct NumberInputState {
    /// Whether the number input has keyboard focus
    pub is_focused: bool,
    /// The text the user is typing; none while the field shows the value
    pub typed: Option<String>,
    /// The cursor's place in `typed`, in characters
    pub cursor: usize,
    /// The number `on_change` last reported from the typed text; none
    /// while the value shown is the last one reported
    pub reported: Option<f64>,
}

/// Builder for the NumberInput props: the value, the range, the step and
/// the name the screen reader hears.
#[derive(Clone, Debug, Default)]
pub struct NumberInputBuilder {
    props: NumberInputProps,
}

impl NumberInputBuilder {
    /// Create a new NumberInputBuilder with the default props
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value
    pub fn value(mut self, value: f64) -> Self {
        self.props.value = value;
        self
    }

    /// Set the least value
    pub fn min(mut self, min: f64) -> Self {
        self.props.min = Some(min);
        self
    }

    /// Set the greatest value
    pub fn max(mut self, max: f64) -> Self {
        self.props.max = Some(max);
        self
    }

    /// Set the least and the greatest value at once
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.props.min = Some(min);
        self.props.max = Some(max);
        self
    }

    /// Set the size of one step
    pub fn step(mut self, step: f64) -> Self {
        self.props.step = step;
        self
    }

    /// Set the name the screen reader hears
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.props.aria_label = Some(label.into());
        self
    }

    /// Set whether the number input is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Set the row's width in cells
    pub fn width(mut self, width: u16) -> Self {
        self.props.width = Some(width);
        self
    }

    /// Build the NumberInputProps
    pub fn build(self) -> NumberInputProps {
        self.props
    }
}

/// Number input component: a number between `[-]` and `[+]`
pub struct NumberInput {
    viewport: Option<LayoutInfo>,
    on_change: Option<Callback>,
    on_submit: Option<Callback>,
}

impl NumberInput {
    /// Create a number input from its props
    pub fn new(_props: NumberInputProps) -> Self {
        Self {
            viewport: None,
            on_change: None,
            on_submit: None,
        }
    }

    /// Set the callback called with the value each time it changes
    pub fn with_on_change(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the value when Confirm submits it
    pub fn with_on_submit(mut self, f: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// Replace the change callback, or remove it with `None`: a builder's
    /// number input takes the one its latest render carried (CMP-009).
    pub(crate) fn set_on_change(&mut self, f: Option<Callback>) {
        self.on_change = f;
    }

    /// Replace the submit callback, or remove it with `None` (CMP-009).
    pub(crate) fn set_on_submit(&mut self, f: Option<Callback>) {
        self.on_submit = f;
    }

    /// The row's width: the props' width when set, else what the layout
    /// gives the number input.
    fn row_width(&self, props: &NumberInputProps) -> usize {
        if let Some(width) = props.width {
            return usize::from(width);
        }
        self.viewport
            .map_or(DEFAULT_WIDTH, |bounds| bounds.content_size().0 as usize)
    }

    /// The field where the terminal takes pixels, drawn as a text input's
    /// field is (PIX-004, PIX-007): a picture in `input` over its `field`
    /// cells and its two frame cells, whose frame cells and the blank run
    /// after the text hold no glyph, so the picture shows there. The text
    /// and the cursor stay cells; the text names no background, so it takes
    /// the picture's fill under it, and is `text-muted` when disabled.
    #[cfg(feature = "wgpu-graphics")]
    fn pictured_field(
        props: &NumberInputProps,
        chars: &[char],
        offset: usize,
        cursor: Option<usize>,
        field: usize,
        focused: bool,
    ) -> Element {
        let text = if props.disabled {
            look::MUTED_PICTURE
        } else {
            look::FIELD_PICTURE
        };
        // Runs of one style: the text, the cursor cell, and the blank run
        // after the text, which never joins the text before it.
        let mut runs: Vec<(String, &'static str)> = Vec::new();
        for index in 0..field {
            let at = offset + index;
            let (glyph, style) = match chars.get(at).copied() {
                glyph if cursor == Some(at) => (glyph.unwrap_or(' '), look::CURSOR),
                Some(glyph) => (glyph, text),
                None => (' ', look::FIELD_PAD),
            };
            match runs.last_mut() {
                Some((run, last)) if *last == style => run.push(glyph),
                _ => runs.push((glyph.to_string(), style)),
            }
        }
        let mut children = vec![look::blank(1)];
        children.extend(runs.into_iter().map(|(run, style)| {
            if style == look::FIELD_PAD {
                look::blank(run.chars().count())
            } else {
                Element::text(run).class(format!("shrink-0 whitespace-pre {style}"))
            }
        }));
        children.push(look::blank(1));
        // `input` under the field: its flat color until its first picture
        // is ready, and what the picture's fill is drawn in.
        let mut element = Element::layout(crate::component::LayoutType::Flex)
            .class(format!(
                "flex flex-row h-1 shrink-0 w-{} bg-input",
                field + 2
            ))
            .children(children);
        element.metadata.look = Some(Arc::new(crate::graphics::look::Look::field(
            focused,
            false,
            props.disabled,
        )));
        element
    }

    /// Calls `on_change` with `value`.
    fn report(&self, value: f64) {
        if let Some(callback) = &self.on_change {
            callback(value);
        }
    }

    /// The value `on_change` last reported: the number last reported from
    /// the typed text, else the value shown.
    fn last_reported(props: &NumberInputProps, state: &NumberInputState) -> f64 {
        state.reported.unwrap_or_else(|| props.clamp(props.value))
    }

    /// Sets `value` as the new value, clamped, and reports it when it
    /// differs from the value last reported (CTL-009).
    fn change(&self, value: f64, props: &mut NumberInputProps, state: &mut NumberInputState) {
        let value = props.clamp(value);
        let reported = Self::last_reported(props, state);
        state.reported = None;
        props.value = value;
        if value != reported {
            self.report(value);
        }
    }

    /// Reports the number the typed text reads as, when it reads as one
    /// inside the range and differs from the value last reported: each
    /// typed change of the number is a change of the value (CTL-009). A
    /// typed number outside the range is not a value the input holds, so
    /// an application never receives one; the clamp reports the value it
    /// becomes when the text is committed.
    fn report_typed(&self, props: &NumberInputProps, state: &mut NumberInputState) {
        let Some(typed) = state.typed.as_deref().and_then(parse) else {
            return;
        };
        if props.clamp(typed) != typed {
            return;
        }
        if typed != Self::last_reported(props, state) {
            state.reported = Some(typed);
            self.report(typed);
        }
    }

    /// Commits the typed text: a text that reads as a number becomes the
    /// value, clamped into the range; any other text leaves the value as it
    /// was. Either way the value is reported when it differs from the
    /// number last reported from the typed text (CTL-009).
    fn settle(&self, props: &mut NumberInputProps, state: &mut NumberInputState) {
        if let Some(text) = state.typed.take() {
            state.cursor = 0;
            let value = parse(&text).unwrap_or(props.value);
            self.change(value, props, state);
        }
    }

    /// Moves the value by `steps` steps, after a typed text is committed.
    fn step(
        &self,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
        steps: f64,
    ) -> EventResult {
        self.settle(props, state);
        let from = props.clamp(props.value);
        self.change(from + steps * props.step(), props, state);
        EventResult::Consumed
    }

    /// Moves the value to `to` (the least or the greatest), when there is
    /// one, after a typed text is committed.
    fn jump(
        &self,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
        to: Option<f64>,
    ) -> EventResult {
        self.settle(props, state);
        if let Some(to) = to {
            self.change(to, props, state);
        }
        EventResult::Consumed
    }

    /// Starts the typed text from the value shown, with the cursor at its
    /// end, unless the user is already typing.
    fn begin_typing(props: &NumberInputProps, state: &mut NumberInputState) {
        if state.typed.is_none() {
            let text = props.format(props.value);
            state.cursor = text.chars().count();
            state.typed = Some(text);
        }
    }

    fn handle_focus(
        &self,
        kind: &FocusEventKind,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
    ) -> EventResult {
        match kind {
            FocusEventKind::Gained => state.is_focused = !props.disabled,
            FocusEventKind::Lost => {
                self.settle(props, state);
                state.is_focused = false;
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }

    fn handle_key(
        &self,
        event: &KeyEvent,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
    ) -> EventResult {
        use crate::keymap::Action;
        // A character the number input takes is text first, whatever the
        // keymap binds it to (KEY-001: typing wins); every other key means
        // what the active keymap says.
        let typed = !event.modifiers.ctrl
            && !event.modifiers.alt
            && matches!(event.code, KeyCode::Char(c) if takes_typed(c));
        let action = if typed {
            None
        } else {
            crate::keymap::Keymap::active().action(event)
        };
        match action {
            Some(Action::Up) => return self.step(props, state, 1.0),
            Some(Action::Down) => return self.step(props, state, -1.0),
            Some(Action::PageUp) => return self.step(props, state, PAGE_STEPS),
            Some(Action::PageDown) => return self.step(props, state, -PAGE_STEPS),
            Some(Action::Home) => return self.jump(props, state, props.min),
            Some(Action::End) => return self.jump(props, state, props.max),
            Some(Action::Confirm) => {
                self.settle(props, state);
                if let Some(callback) = &self.on_submit {
                    callback(props.clamp(props.value));
                }
                return EventResult::Consumed;
            }
            Some(Action::Left) => {
                Self::begin_typing(props, state);
                state.cursor = state.cursor.saturating_sub(1);
                return EventResult::Consumed;
            }
            Some(Action::Right) => {
                Self::begin_typing(props, state);
                let length = state.typed.as_ref().map_or(0, |text| text.chars().count());
                state.cursor = (state.cursor + 1).min(length);
                return EventResult::Consumed;
            }
            Some(Action::Delete) => {
                Self::begin_typing(props, state);
                let cursor = state.cursor;
                if let Some(text) = state.typed.as_mut() {
                    remove_char(text, cursor);
                }
                self.report_typed(props, state);
                return EventResult::Consumed;
            }
            _ => {}
        }
        match event.code {
            KeyCode::Backspace => {
                Self::begin_typing(props, state);
                if state.cursor > 0 {
                    state.cursor -= 1;
                    let cursor = state.cursor;
                    if let Some(text) = state.typed.as_mut() {
                        remove_char(text, cursor);
                    }
                }
                self.report_typed(props, state);
                EventResult::Consumed
            }
            KeyCode::Char(c) if !event.modifiers.ctrl && !event.modifiers.alt && takes_typed(c) => {
                Self::begin_typing(props, state);
                let cursor = state.cursor;
                if let Some(text) = state.typed.as_mut() {
                    insert_char(text, cursor, c);
                }
                state.cursor += 1;
                self.report_typed(props, state);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse(
        &self,
        event: &MouseEvent,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Wheel => {
                let Some(delta) = event.wheel.as_ref().map(|wheel| match wheel.delta {
                    WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. } => y,
                }) else {
                    return EventResult::Ignored;
                };
                if delta == 0.0 {
                    return EventResult::Ignored;
                }
                // A wheel up (a negative vertical delta) steps up.
                let steps = if delta < 0.0 { 1.0 } else { -1.0 };
                self.step(props, state, steps)
            }
            MouseEventKind::Down if event.button == MouseButton::Left => {
                let insets = self.viewport.map_or([0.0; 4], |bounds| bounds.insets);
                let x = event.position.x() as f32 - insets[0];
                let width = self.row_width(props) as f32;
                if (0.0..3.0).contains(&x) {
                    self.step(props, state, -1.0)
                } else if (width - 3.0..width).contains(&x) {
                    self.step(props, state, 1.0)
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// Takes the screen reader's request (CTL-010): Increment and Decrement
    /// step as the Up and Down keys do; SetValue takes the requested number
    /// snapped to the step and clamped into the range, as a slider takes it
    /// (CTL-005). A SetValue that names no finite number changes nothing.
    fn handle_request(
        &self,
        event: &CustomEvent,
        props: &mut NumberInputProps,
        state: &mut NumberInputState,
    ) -> EventResult {
        match event.name.as_str() {
            INCREMENT_EVENT => self.step(props, state, 1.0),
            DECREMENT_EVENT => self.step(props, state, -1.0),
            SET_VALUE_EVENT => {
                let Some(requested) = std::str::from_utf8(&event.data).ok().and_then(parse) else {
                    return EventResult::Ignored;
                };
                state.typed = None;
                state.cursor = 0;
                self.change(props.snap(requested), props, state);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// The finite number `text` reads as, if it reads as one.
fn parse(text: &str) -> Option<f64> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// Whether typing `c` puts it in the field: digits, `.` and `-`, as a text
/// input in its Numeric mode takes them.
fn takes_typed(c: char) -> bool {
    c.is_numeric() || c == '.' || c == '-'
}

/// Removes the character at `index`, if there is one.
fn remove_char(text: &mut String, index: usize) {
    if let Some((byte, _)) = text.char_indices().nth(index) {
        text.remove(byte);
    }
}

/// Inserts `c` before the character at `index`; at the end when there is
/// none.
fn insert_char(text: &mut String, index: usize, c: char) {
    let byte = text
        .char_indices()
        .nth(index)
        .map_or(text.len(), |(byte, _)| byte);
    text.insert(byte, c);
}

impl Component for NumberInput {
    type Props = NumberInputProps;
    type State = NumberInputState;

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if props.disabled {
            *state = NumberInputState::default();
        }
        true
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let previous = self.row_width(props);
        self.viewport = Some(bounds);
        previous != self.row_width(props)
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let width = self.row_width(props);
        let field = width.saturating_sub(CHROME);
        let focused = state.is_focused && !props.disabled;
        let value = props.clamp(props.value);
        // The field's text is the typed text while the user types, else
        // the value in the step's decimals; the cursor shows only while
        // the user types.
        let (text, cursor) = match &state.typed {
            Some(text) => (text.clone(), focused.then_some(state.cursor)),
            None => (props.format(value), None),
        };
        let chars: Vec<char> = text.chars().collect();
        // The field's text scrolls so that the cursor stays in view.
        let offset = match cursor {
            Some(cursor) if cursor >= field => cursor + 1 - field,
            _ => 0,
        };
        let frame = look::field_frame(focused);
        let field_look = if props.disabled {
            look::FIELD_DISABLED
        } else {
            look::FIELD
        };
        let minus_look = if props.disabled || !props.can_step(value, false) {
            look::STEPPER_MUTED
        } else {
            look::STEPPER
        };
        let plus_look = if props.disabled || !props.can_step(value, true) {
            look::STEPPER_MUTED
        } else {
            look::STEPPER
        };
        // With pixels each button is a picture over its three cells and
        // the field a picture over its own, as a text input's (PIX-007).
        #[cfg(feature = "wgpu-graphics")]
        let pictured = look::pixels().then(|| {
            use crate::graphics::look::Look;
            vec![
                look::spacer(
                    3,
                    Look::spin_button(false, props.can_step(value, false), props.disabled),
                ),
                Self::pictured_field(props, &chars, offset, cursor, field, focused),
                look::spacer(
                    3,
                    Look::spin_button(true, props.can_step(value, true), props.disabled),
                ),
            ]
        });
        #[cfg(not(feature = "wgpu-graphics"))]
        let pictured: Option<Vec<Element>> = None;
        let children = match pictured {
            Some(children) => children,
            None => {
                let mut pieces: Vec<(String, &'static str)> = Vec::new();
                pieces.push((format!("[{}]", Icon::Minus.glyph()), minus_look));
                pieces.push(("[".to_owned(), frame));
                for index in 0..field {
                    let at = offset + index;
                    let glyph = chars.get(at).copied().unwrap_or(' ');
                    let style = if cursor == Some(at) {
                        look::CURSOR
                    } else {
                        field_look
                    };
                    pieces.push((glyph.to_string(), style));
                }
                pieces.push(("]".to_owned(), frame));
                pieces.push((format!("[{}]", Icon::Plus.glyph()), plus_look));
                let refs: Vec<(&str, &str)> = pieces
                    .iter()
                    .map(|(text, style)| (text.as_str(), *style))
                    .collect();
                look::pieces(&refs)
            }
        };

        let mut accessible = Node::new(Role::SpinButton);
        if let Some(label) = &props.aria_label {
            accessible.set_label(label.clone());
        }
        accessible.inner.set_numeric_value(value);
        if let Some(min) = props.min {
            accessible.inner.set_min_numeric_value(min);
        }
        if let Some(max) = props.max {
            accessible.inner.set_max_numeric_value(max);
        }
        accessible.inner.set_numeric_value_step(props.step());
        if props.disabled {
            accessible.set_disabled();
        }
        // The screen reader moves the value or sets it; a disabled number
        // input advertises neither (CTL-010).
        let movable = !props.disabled;
        if movable {
            for action in [
                accesskit::Action::Increment,
                accesskit::Action::Decrement,
                accesskit::Action::SetValue,
            ] {
                accessible.inner.add_action(action);
            }
        }
        let mut element = look::row_of(children, "w-full")
            .with_accessibility(accessible)
            .with_focus(FocusProps::input())
            .disabled(props.disabled);
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
            return self.handle_focus(&event.kind, props, state);
        }
        if props.disabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(event) if state.is_focused => self.handle_key(event, props, state),
            Event::Mouse(event) => self.handle_mouse(event, props, state),
            Event::Custom(event) => self.handle_request(event, props, state),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyEventKind, KeyModifiers};
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

    fn focused() -> NumberInputState {
        NumberInputState {
            is_focused: true,
            ..Default::default()
        }
    }

    fn range_props(value: f64) -> NumberInputProps {
        NumberInputProps {
            value,
            min: Some(0.0),
            max: Some(10.0),
            step: 0.5,
            ..Default::default()
        }
    }

    /// CTL-009: Up and Down step by the step, Page Up and Page Down by ten
    /// steps, End goes to the maximum and nothing passes the range; each
    /// change is reported once.
    #[test]
    fn ctl_009_a_number_input_steps_within_its_range_and_reports_each_change() {
        let log: Arc<Mutex<Vec<f64>>> = Arc::default();
        let sink = log.clone();
        let mut input = NumberInput::new(range_props(2.0))
            .with_on_change(move |value| sink.lock().unwrap().push(value));
        let mut props = range_props(2.0);
        let mut state = focused();
        input.handle_event(&key(KeyCode::Up), &mut props, &mut state);
        input.handle_event(&key(KeyCode::PageUp), &mut props, &mut state);
        assert_eq!(props.value, 7.5, "CTL-009: Up then Page Up step 0.5 and 5");
        input.handle_event(&key(KeyCode::End), &mut props, &mut state);
        input.handle_event(&key(KeyCode::Up), &mut props, &mut state);
        assert_eq!(props.value, 10.0, "CTL-009: Up at the maximum keeps it");
        assert_eq!(
            *log.lock().unwrap(),
            vec![2.5, 7.5, 10.0],
            "CTL-009: only the changes are reported"
        );
    }

    /// CTL-009: a typed value is clamped into the range when the focus
    /// leaves, and Confirm reports the value through on_submit.
    #[test]
    fn ctl_009_a_typed_value_is_clamped_when_the_focus_leaves() {
        let submitted: Arc<Mutex<Vec<f64>>> = Arc::default();
        let sink = submitted.clone();
        let mut input = NumberInput::new(range_props(0.0))
            .with_on_submit(move |value| sink.lock().unwrap().push(value));
        let mut props = range_props(0.0);
        let mut state = focused();
        for _ in 0..3 {
            input.handle_event(&key(KeyCode::Backspace), &mut props, &mut state);
        }
        input.handle_event(&key(KeyCode::Char('4')), &mut props, &mut state);
        input.handle_event(&key(KeyCode::Char('2')), &mut props, &mut state);
        assert_eq!(
            state.typed.as_deref(),
            Some("42"),
            "the typed text shows as typed"
        );
        input.handle_event(&key(KeyCode::Char('x')), &mut props, &mut state);
        assert_eq!(state.typed.as_deref(), Some("42"), "a letter is not taken");
        input.handle_event(
            &Event::Focus(crate::event::types::FocusEvent {
                kind: FocusEventKind::Lost,
                timestamp: std::time::Instant::now(),
            }),
            &mut props,
            &mut state,
        );
        assert_eq!(props.value, 10.0, "CTL-009: 42 is clamped to the maximum");
        assert!(state.typed.is_none());
        input.handle_event(
            &Event::Focus(crate::event::types::FocusEvent {
                kind: FocusEventKind::Gained,
                timestamp: std::time::Instant::now(),
            }),
            &mut props,
            &mut state,
        );
        input.handle_event(&key(KeyCode::Enter), &mut props, &mut state);
        assert_eq!(*submitted.lock().unwrap(), vec![10.0]);
    }

    fn lost() -> Event {
        Event::Focus(crate::event::types::FocusEvent {
            kind: FocusEventKind::Lost,
            timestamp: std::time::Instant::now(),
        })
    }

    /// A number input of `range_props(value)` that logs each value
    /// `on_change` reports.
    fn logged(value: f64) -> (NumberInput, Arc<Mutex<Vec<f64>>>) {
        let log: Arc<Mutex<Vec<f64>>> = Arc::default();
        let sink = log.clone();
        let input = NumberInput::new(range_props(value))
            .with_on_change(move |value| sink.lock().unwrap().push(value));
        (input, log)
    }

    /// CTL-009: each typed keystroke that changes the number the text reads
    /// as reports it while it lies inside the range, a number outside the
    /// range is not reported, and the clamp when the focus leaves reports
    /// the value it becomes.
    #[test]
    fn ctl_009_a_typed_change_of_the_number_is_reported() {
        let (mut input, log) = logged(0.0);
        let mut props = range_props(0.0);
        let mut state = focused();
        // "0.0" to "0." to "0" to "": no other number, then no number.
        for _ in 0..3 {
            input.handle_event(&key(KeyCode::Backspace), &mut props, &mut state);
        }
        assert!(log.lock().unwrap().is_empty(), "0. and 0 are still 0");
        input.handle_event(&key(KeyCode::Char('4')), &mut props, &mut state);
        input.handle_event(&key(KeyCode::Char('2')), &mut props, &mut state);
        input.handle_event(&key(KeyCode::Char('x')), &mut props, &mut state);
        assert_eq!(
            *log.lock().unwrap(),
            vec![4.0],
            "4; 42 lies outside 0 to 10 and is not reported"
        );
        input.handle_event(&lost(), &mut props, &mut state);
        assert_eq!(props.value, 10.0);
        assert_eq!(
            *log.lock().unwrap(),
            vec![4.0, 10.0],
            "the clamp to 10 is a change from the 4 reported"
        );
    }

    /// CTL-009: a typed text that reads as no number when the focus leaves
    /// keeps the value, and reports it when a typed number was reported in
    /// between.
    #[test]
    fn ctl_009_a_typed_text_that_is_no_number_reports_the_value_kept() {
        let (mut input, log) = logged(0.0);
        let mut props = range_props(0.0);
        let mut state = focused();
        for _ in 0..3 {
            input.handle_event(&key(KeyCode::Backspace), &mut props, &mut state);
        }
        input.handle_event(&key(KeyCode::Char('7')), &mut props, &mut state);
        input.handle_event(&key(KeyCode::Backspace), &mut props, &mut state);
        input.handle_event(&lost(), &mut props, &mut state);
        assert_eq!(props.value, 0.0, "the empty text keeps the value");
        assert_eq!(*log.lock().unwrap(), vec![7.0, 0.0]);
    }

    /// CTL-010: a SetValue snaps the requested number to the step and
    /// clamps it into the range, as a slider takes it (CTL-005).
    #[test]
    fn ctl_010_a_set_value_request_snaps_to_the_step() {
        let (mut input, log) = logged(2.0);
        let mut props = range_props(2.0);
        let mut state = NumberInputState::default();
        for (requested, expected) in [("2.3", 2.5), ("2.2", 2.0), ("-3", 0.0), ("9.9", 10.0)] {
            let set = CustomEvent::new(SET_VALUE_EVENT, requested.as_bytes().to_vec());
            input.handle_event(&Event::Custom(set), &mut props, &mut state);
            assert_eq!(props.value, expected, "SetValue of {requested}");
        }
        assert_eq!(*log.lock().unwrap(), vec![2.5, 2.0, 0.0, 10.0]);

        // With no least value the steps count from 0.
        let mut props = NumberInputProps {
            step: 0.25,
            ..Default::default()
        };
        let set = CustomEvent::new(SET_VALUE_EVENT, b"1.3".to_vec());
        input.handle_event(&Event::Custom(set), &mut props, &mut state);
        assert_eq!(props.value, 1.25);
    }

    /// CTL-009: a disabled number input takes nothing.
    #[test]
    fn ctl_009_a_disabled_number_input_takes_no_key() {
        let mut input = NumberInput::new(range_props(2.0));
        let mut props = NumberInputProps {
            disabled: true,
            ..range_props(2.0)
        };
        let mut state = focused();
        let result = input.handle_event(&key(KeyCode::Up), &mut props, &mut state);
        assert_eq!(result, EventResult::Ignored);
        assert_eq!(props.value, 2.0);
    }

    /// CTL-010: the node is a spin button with its label, value, bounds and
    /// step, and it advertises Increment, Decrement and SetValue.
    #[test]
    fn ctl_010_a_number_input_node_is_a_spin_button() {
        let props = NumberInputProps {
            aria_label: Some("Volume".to_owned()),
            ..range_props(2.0)
        };
        let input = NumberInput::new(props.clone());
        let element = input.render(&props, &NumberInputState::default());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("the number input has a node");
        assert_eq!(node.inner.role(), accesskit::Role::SpinButton);
        assert_eq!(node.inner.label(), Some("Volume"));
        assert_eq!(node.inner.numeric_value(), Some(2.0));
        assert_eq!(node.inner.min_numeric_value(), Some(0.0));
        assert_eq!(node.inner.max_numeric_value(), Some(10.0));
        assert_eq!(node.inner.numeric_value_step(), Some(0.5));
        for action in [
            accesskit::Action::Increment,
            accesskit::Action::Decrement,
            accesskit::Action::SetValue,
        ] {
            assert!(node.inner.supports_action(action), "CTL-010: {action:?}");
        }
    }

    /// CTL-010: SetValue takes the requested number and clamps it; an
    /// Increment steps the value.
    #[test]
    fn ctl_010_a_number_input_takes_set_value_and_increment_requests() {
        let mut input = NumberInput::new(range_props(2.0));
        let mut props = range_props(2.0);
        let mut state = NumberInputState::default();
        let set = CustomEvent::new(SET_VALUE_EVENT, b"42".to_vec());
        input.handle_event(&Event::Custom(set), &mut props, &mut state);
        assert_eq!(props.value, 10.0, "CTL-010: SetValue of 42 clamps to 10");
        let increment = CustomEvent::new(INCREMENT_EVENT, Vec::new());
        input.handle_event(&Event::Custom(increment), &mut props, &mut state);
        assert_eq!(
            props.value, 10.0,
            "CTL-010: Increment at the maximum keeps it"
        );
        let decrement = CustomEvent::new(DECREMENT_EVENT, Vec::new());
        input.handle_event(&Event::Custom(decrement), &mut props, &mut state);
        assert_eq!(props.value, 9.5);
    }

    /// KEY-001, CTL-009: a character the number input takes is typed even
    /// when the active keymap binds it to an action: with Up rebound to
    /// `7`, typing 7 types it and does not step.
    #[test]
    fn ctl_009_a_digit_bound_to_an_action_is_typed() {
        use crate::keymap::{Action, KeyBinding, Keymap};
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Up, [KeyBinding::new(KeyCode::Char('7'))]);
        let _scope = Keymap::scoped(keymap);
        let mut input = NumberInput::new(range_props(0.0));
        let mut props = range_props(0.0);
        let mut state = focused();
        for _ in 0..3 {
            input.handle_event(&key(KeyCode::Backspace), &mut props, &mut state);
        }
        input.handle_event(&key(KeyCode::Char('7')), &mut props, &mut state);
        assert_eq!(
            state.typed.as_deref(),
            Some("7"),
            "KEY-001: the 7 is typed, not read as Up"
        );
        assert_eq!(props.value, 0.0, "the value did not step");
    }

    /// CTL-009: a step past the greatest finite number keeps the bound, as a
    /// step at the bound does; it never wraps to 0.
    #[test]
    fn ctl_009_a_step_that_overflows_keeps_the_bound() {
        let at = |value: f64, min: Option<f64>, max: Option<f64>| NumberInputProps {
            value,
            min,
            max,
            step: 1e308,
            ..Default::default()
        };
        let mut input = NumberInput::new(at(1e308, Some(0.0), Some(1e308)));
        let mut state = focused();
        let mut props = at(1e308, Some(0.0), Some(1e308));
        input.handle_event(&key(KeyCode::Up), &mut props, &mut state);
        assert_eq!(props.value, 1e308, "CTL-009: Up at the maximum keeps it");
        let mut props = at(-1e308, Some(-1e308), Some(0.0));
        input.handle_event(&key(KeyCode::Down), &mut props, &mut state);
        assert_eq!(props.value, -1e308, "CTL-009: Down at the minimum keeps it");
        let mut props = at(f64::MAX, None, None);
        input.handle_event(&key(KeyCode::Up), &mut props, &mut state);
        assert_eq!(
            props.value,
            f64::MAX,
            "CTL-009: with no maximum, Up from the greatest number keeps it"
        );
    }
}
