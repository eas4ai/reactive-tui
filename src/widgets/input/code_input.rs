//! The code input (docs/spec/input-widgets.md, CTL-007 to CTL-010): a row of
//! digit cells for a one-time code or a verification number. Each digit is a
//! cell of three cells `[d]`, with one space between groups when `group` is
//! set. The whole code is one focus stop and one accessibility node.

use super::look;
use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, FocusProps, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use crate::event::{CustomEvent, Event};
use crate::keymap::{Action, Keymap};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;
use std::sync::Arc;

/// The event a code input takes for the screen reader's SetValue request
/// (CTL-005, CTL-010); its data is the requested text, taken as a paste.
pub(crate) const SET_VALUE_EVENT: &str = "reactive_tui.code_input.set_value";

/// A code input's callback, called with the code the input holds.
type Callback = Arc<dyn Fn(String) + Send + Sync>;

/// The digits of `text`, in order, other characters dropped, at most
/// `length` of them: the code a code input holds for `text`.
fn digits_of(text: &str, length: usize) -> String {
    text.chars()
        .filter(|c| c.is_ascii_digit())
        .take(length)
        .collect()
}

/// Properties for a code input.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeInputProps {
    /// How many digits the code holds (6 unless set).
    pub length: usize,
    /// Digits per group, with one cell of space between groups; `None`
    /// sets the cells with no gap.
    pub group: Option<usize>,
    /// The digits typed so far.
    pub value: String,
    /// Whether the digits are hidden: each filled cell shows a dot.
    pub masked: bool,
    /// The name's first part; "Code" when unset.
    pub label: Option<String>,
    /// The name the screen reader hears instead of the label.
    pub aria_label: Option<String>,
    /// Whether the code input is disabled.
    pub disabled: bool,
}

impl Default for CodeInputProps {
    fn default() -> Self {
        Self {
            length: 6,
            group: None,
            value: String::new(),
            masked: false,
            label: None,
            aria_label: None,
            disabled: false,
        }
    }
}

impl Props for CodeInputProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for a code input.
#[derive(Clone, Debug, Default)]
pub struct CodeInputState {
    /// Whether the code input has keyboard focus.
    pub is_focused: bool,
}

/// Builder for a code input's properties.
#[derive(Clone, Debug, Default)]
pub struct CodeInputBuilder {
    props: CodeInputProps,
}

impl CodeInputBuilder {
    /// A builder with the default properties.
    pub fn new() -> Self {
        Self {
            props: CodeInputProps::default(),
        }
    }

    /// Set how many digits the code holds.
    pub fn length(mut self, length: usize) -> Self {
        self.props.length = length;
        self
    }

    /// Set the digits per group.
    pub fn group(mut self, group: usize) -> Self {
        self.props.group = Some(group);
        self
    }

    /// Set the digits typed so far.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.props.value = value.into();
        self
    }

    /// Hide the digits behind dots.
    pub fn masked(mut self, masked: bool) -> Self {
        self.props.masked = masked;
        self
    }

    /// Set the name's first part.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.props.label = Some(label.into());
        self
    }

    /// Set the name the screen reader hears.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.props.aria_label = Some(label.into());
        self
    }

    /// Set whether the code input is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Build the properties.
    pub fn build(self) -> CodeInputProps {
        self.props
    }

    /// Build and render as an element.
    pub fn render(self) -> Element {
        Element::typed::<CodeInput>(self.build())
    }
}

/// A row of digit cells that takes a code typed, pasted or set by the
/// screen reader.
pub struct CodeInput {
    on_change: Option<Callback>,
    on_complete: Option<Callback>,
}

impl CodeInput {
    /// A code input with no callbacks.
    pub fn new(_props: CodeInputProps) -> Self {
        Self {
            on_change: None,
            on_complete: None,
        }
    }

    /// Set the callback called with the code after each change.
    pub fn with_on_change(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the code when it is complete.
    pub fn with_on_complete(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_complete = Some(Arc::new(f));
        self
    }

    /// Replace the change callback; a builder's code input takes the one
    /// its latest render carried (CMP-009).
    pub(crate) fn set_on_change(&mut self, f: Option<Callback>) {
        self.on_change = f;
    }

    /// Replace the completion callback; a builder's code input takes the
    /// one its latest render carried (CMP-009).
    pub(crate) fn set_on_complete(&mut self, f: Option<Callback>) {
        self.on_complete = f;
    }

    /// Store `next` as the code when it differs, then report the change; a
    /// code that is now full is reported through `on_complete` as well.
    fn change_to(&self, next: String, props: &mut CodeInputProps) {
        if next == props.value {
            return;
        }
        props.value = next.clone();
        let full = props.length > 0 && next.chars().count() == props.length;
        if let Some(callback) = &self.on_change {
            callback(next.clone());
        }
        if full {
            if let Some(callback) = &self.on_complete {
                callback(next);
            }
        }
    }

    fn handle_key_event(&mut self, event: &KeyEvent, props: &mut CodeInputProps) -> EventResult {
        // Confirm is read through the keymap (KEY-001); digits and Backspace
        // are the code input's own keys.
        if matches!(Keymap::active().action(event), Some(Action::Confirm)) {
            let code = digits_of(&props.value, props.length);
            if props.length > 0 && code.chars().count() == props.length {
                if let Some(callback) = &self.on_complete {
                    callback(code);
                }
                return EventResult::Consumed;
            }
            return EventResult::Ignored;
        }
        if event.modifiers.ctrl || event.modifiers.alt {
            return EventResult::Ignored;
        }
        match event.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut next = digits_of(&props.value, props.length);
                if next.chars().count() < props.length {
                    next.push(c);
                    self.change_to(next, props);
                }
                EventResult::Consumed
            }
            KeyCode::Backspace => {
                let mut next = digits_of(&props.value, props.length);
                next.pop();
                self.change_to(next, props);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    /// The row of digit cells where the terminal takes pixels (PIX-007):
    /// each cell a picture over its three cells, its outer two holding no
    /// glyph and its middle one the digit, `_` or dot as cell text. The
    /// cell is `input` under the picture, its flat color until the picture
    /// is ready and the color its middle cell takes; the text names no
    /// background of its own.
    #[cfg(feature = "wgpu-graphics")]
    fn pictured_cells(
        props: &CodeInputProps,
        digits: &str,
        current: usize,
        focused: bool,
    ) -> Element {
        use crate::component::LayoutType;
        let digit_class = if props.disabled {
            look::MUTED_PICTURE
        } else {
            look::FIELD_PICTURE
        };
        let mut children = Vec::new();
        for index in 0..props.length {
            if index > 0
                && props
                    .group
                    .is_some_and(|group| group > 0 && index % group == 0)
            {
                children.extend(look::pieces(&[(" ", look::LABEL)]));
            }
            let (glyph, class) = match digits.chars().nth(index) {
                Some(_) if props.masked => (Icon::Dot.glyph().to_owned(), digit_class),
                Some(digit) => (digit.to_string(), digit_class),
                None => ("_".to_owned(), look::MUTED_PICTURE),
            };
            let mut cell = Element::layout(LayoutType::Flex)
                .class("flex flex-row h-1 w-3 shrink-0 bg-input")
                .children(vec![
                    look::blank(1),
                    Element::text(glyph).class(format!("shrink-0 whitespace-pre {class}")),
                    look::blank(1),
                ]);
            cell.metadata.look = Some(Arc::new(crate::graphics::look::Look::code_cell(
                index == current,
                focused,
                props.disabled,
            )));
            children.push(cell);
        }
        look::row_of(children, "")
    }
}

impl Component for CodeInput {
    type Props = CodeInputProps;
    type State = CodeInputState;

    fn new(props: Self::Props) -> Self {
        CodeInput::new(props)
    }

    fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let digits = digits_of(&props.value, props.length);
        let focused = state.is_focused && !props.disabled;
        // The current cell is the first empty one, or the last when full.
        let current = digits.chars().count().min(props.length.saturating_sub(1));
        let digit_class = if props.disabled {
            look::FIELD_DISABLED
        } else {
            look::FIELD
        };
        let mut owned: Vec<(String, &str)> = Vec::new();
        for index in 0..props.length {
            if index > 0
                && props
                    .group
                    .is_some_and(|group| group > 0 && index % group == 0)
            {
                owned.push((" ".to_owned(), look::LABEL));
            }
            let frame = if index == current {
                look::field_frame(focused)
            } else {
                look::FIELD_FRAME
            };
            let (glyph, class) = match digits.chars().nth(index) {
                Some(_) if props.masked => (Icon::Dot.glyph().to_owned(), digit_class),
                Some(digit) => (digit.to_string(), digit_class),
                None => ("_".to_owned(), look::MUTED),
            };
            owned.push(("[".to_owned(), frame));
            owned.push((glyph, class));
            owned.push(("]".to_owned(), frame));
        }
        let pieces: Vec<(&str, &str)> = owned
            .iter()
            .map(|(text, class)| (text.as_str(), *class))
            .collect();
        // With pixels each digit cell is a picture over its three cells and
        // no `[` or `]`: a square in `input`, bordered in `border`, in
        // `ring` for the current cell while focused; the digit, `_` or dot
        // stays cell text in its middle cell (PIX-007).
        #[cfg(feature = "wgpu-graphics")]
        let pictured =
            look::pixels().then(|| Self::pictured_cells(props, &digits, current, focused));
        #[cfg(not(feature = "wgpu-graphics"))]
        let pictured: Option<Element> = None;
        let row = pictured.unwrap_or_else(|| look::row(&pieces, ""));

        let name = props
            .aria_label
            .as_deref()
            .or(props.label.as_deref())
            .unwrap_or("Code");
        let mut accessible = Node::new(if props.masked {
            Role::PasswordInput
        } else {
            Role::TextInput
        });
        accessible.set_label(format!("{name}, {} digits", props.length));
        if props.disabled {
            accessible.set_disabled();
        } else {
            accessible.inner.add_action(accesskit::Action::SetValue);
        }
        // A masked code gives the reader one bullet per digit, as the
        // password text field does, so a one-time code is neither spoken nor
        // read off the accessibility bus (CTL-010).
        accessible.set_value(if props.masked {
            "•".repeat(digits.chars().count())
        } else {
            digits.to_string()
        });

        let mut element = row
            .with_accessibility(accessible)
            .with_focus(FocusProps::input())
            .disabled(props.disabled);
        if !props.disabled {
            element
                .metadata
                .accessibility_options
                .get_or_insert_default()
                .set_value_event = Some(CustomEvent::new(SET_VALUE_EVENT, Vec::new()));
        }
        element
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled && !matches!(event, Event::Focus(_)) {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(key) if state.is_focused => self.handle_key_event(key, props),
            Event::Paste(paste) if state.is_focused => {
                self.change_to(digits_of(&paste.content, props.length), props);
                EventResult::Consumed
            }
            Event::Custom(custom) if custom.name == SET_VALUE_EVENT => {
                match std::str::from_utf8(&custom.data) {
                    Ok(requested) => {
                        self.change_to(digits_of(requested, props.length), props);
                        EventResult::Consumed
                    }
                    Err(_) => EventResult::Ignored,
                }
            }
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down && mouse.button == MouseButton::Left =>
            {
                state.is_focused = true;
                EventResult::Consumed
            }
            Event::Focus(event)
                if matches!(
                    event.kind,
                    crate::event::types::FocusEventKind::Gained
                        | crate::event::types::FocusEventKind::Lost
                ) =>
            {
                state.is_focused = event.kind == crate::event::types::FocusEventKind::Gained;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyModifiers, MouseButton, PasteEvent, Position};
    use crate::event::MouseEvent;
    use std::sync::Mutex;
    use std::time::Instant;

    type Log = Arc<Mutex<Vec<String>>>;

    /// A code input of `props` with its change and completion logs.
    fn logged(props: CodeInputProps) -> (CodeInput, Log, Log) {
        let changes: Log = Arc::default();
        let complete: Log = Arc::default();
        let (c, d) = (changes.clone(), complete.clone());
        let input = CodeInput::new(props)
            .with_on_change(move |code| c.lock().unwrap().push(code))
            .with_on_complete(move |code| d.lock().unwrap().push(code));
        (input, changes, complete)
    }

    fn held(log: &Log) -> Vec<String> {
        log.lock().unwrap().clone()
    }

    fn focused() -> CodeInputState {
        CodeInputState { is_focused: true }
    }

    fn key(
        input: &mut CodeInput,
        props: &mut CodeInputProps,
        state: &mut CodeInputState,
        code: KeyCode,
    ) -> EventResult {
        input.handle_event(&Event::Key(KeyEvent::new(code)), props, state)
    }

    /// The text a render paints, cell by cell, in order.
    fn painted(element: &Element) -> String {
        fn collect(element: &Element, out: &mut String) {
            if let crate::component::ElementType::Text(text) = &element.element_type {
                out.push_str(text);
            }
            for child in &element.children {
                collect(child, out);
            }
        }
        let mut out = String::new();
        collect(element, &mut out);
        out
    }

    fn node(element: &Element) -> &crate::accessibility::Node {
        element
            .metadata
            .accessibility
            .as_ref()
            .expect("the code input has one node")
    }

    /// CTL-009: typing fills the first empty cell, Backspace clears the
    /// last filled one, and other characters change nothing.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_typing_fills_the_current_cell_and_backspace_clears_the_last() {
        let mut props = CodeInputProps {
            length: 3,
            ..Default::default()
        };
        let (mut input, changes, complete) = logged(props.clone());
        let mut state = focused();
        key(&mut input, &mut props, &mut state, KeyCode::Char('1'));
        key(&mut input, &mut props, &mut state, KeyCode::Char('x'));
        key(&mut input, &mut props, &mut state, KeyCode::Char('2'));
        assert_eq!(props.value, "12");
        key(&mut input, &mut props, &mut state, KeyCode::Backspace);
        assert_eq!(props.value, "1");
        key(&mut input, &mut props, &mut state, KeyCode::Char('3'));
        assert_eq!(props.value, "13");
        key(&mut input, &mut props, &mut state, KeyCode::Char('4'));
        assert_eq!(props.value, "134");
        key(&mut input, &mut props, &mut state, KeyCode::Char('5'));
        assert_eq!(
            props.value, "134",
            "a digit past the last cell is not taken"
        );
        assert_eq!(held(&changes), vec!["1", "12", "1", "13", "134"]);
        assert_eq!(
            held(&complete),
            vec!["134"],
            "filling the last cell reports the code once"
        );
    }

    /// CTL-009: a paste replaces the whole code, drops the characters that
    /// are not digits, and filling the last cell reports the code once.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_a_paste_replaces_the_code_and_completes_once() {
        let mut props = CodeInputProps::default();
        let (mut input, changes, complete) = logged(props.clone());
        let mut state = focused();
        key(&mut input, &mut props, &mut state, KeyCode::Char('1'));
        input.handle_event(
            &Event::Paste(PasteEvent::new("98-7654".to_owned())),
            &mut props,
            &mut state,
        );
        assert_eq!(props.value, "987654");
        assert_eq!(held(&complete), vec!["987654"]);
        assert_eq!(held(&changes).last().map(String::as_str), Some("987654"));

        let (mut short, _, short_complete) = logged(CodeInputProps::default());
        let mut short_props = CodeInputProps::default();
        short.handle_event(
            &Event::Paste(PasteEvent::new("12".to_owned())),
            &mut short_props,
            &mut state,
        );
        assert_eq!(short_props.value, "12");
        assert!(
            held(&short_complete).is_empty(),
            "a short paste completes nothing"
        );
    }

    /// CTL-009: Confirm reports a full code through on_complete and does
    /// nothing while a cell is empty.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_confirm_completes_only_a_full_code() {
        let mut props = CodeInputProps {
            length: 2,
            value: "1".to_owned(),
            ..Default::default()
        };
        let (mut input, _, complete) = logged(props.clone());
        let mut state = focused();
        assert_eq!(
            key(&mut input, &mut props, &mut state, KeyCode::Enter),
            EventResult::Ignored
        );
        assert!(held(&complete).is_empty());
        key(&mut input, &mut props, &mut state, KeyCode::Char('2'));
        assert_eq!(
            key(&mut input, &mut props, &mut state, KeyCode::Enter),
            EventResult::Consumed
        );
        assert_eq!(
            held(&complete),
            vec!["12", "12"],
            "filling it and Confirm each report"
        );
    }

    /// CTL-009: a click gives the code input the focus; a disabled code
    /// input takes nothing, not even a key.
    #[test]
    #[serial_test::parallel(keymap)]
    fn ctl_009_a_click_focuses_and_a_disabled_code_input_takes_nothing() {
        let mut props = CodeInputProps::default();
        let (mut input, changes, _) = logged(props.clone());
        let mut state = CodeInputState::default();
        let click = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down,
            button: MouseButton::Left,
            position: Position::cell(0, 0),
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
            wheel: None,
        });
        input.handle_event(&click, &mut props, &mut state);
        assert!(state.is_focused, "a click focuses the code input");
        key(&mut input, &mut props, &mut state, KeyCode::Char('5'));
        assert_eq!(props.value, "5");

        let mut disabled = CodeInputProps {
            disabled: true,
            ..Default::default()
        };
        let (mut off, off_changes, _) = logged(disabled.clone());
        let mut off_state = focused();
        assert_eq!(
            key(&mut off, &mut disabled, &mut off_state, KeyCode::Char('5')),
            EventResult::Ignored
        );
        assert!(disabled.value.is_empty());
        assert!(held(&off_changes).is_empty());
        assert_eq!(held(&changes), vec!["5"]);
    }

    /// CTL-010 (SetValue): the screen reader's request is taken as a paste.
    #[test]
    fn ctl_010_a_set_value_request_replaces_the_code_as_a_paste() {
        let mut props = CodeInputProps {
            value: "1".to_owned(),
            ..Default::default()
        };
        let (mut input, _, complete) = logged(props.clone());
        let mut state = focused();
        input.handle_event(
            &Event::Custom(crate::event::CustomEvent::new(
                SET_VALUE_EVENT,
                b"987654".to_vec(),
            )),
            &mut props,
            &mut state,
        );
        assert_eq!(props.value, "987654");
        assert_eq!(held(&complete), vec!["987654"]);
    }

    /// CTL-010: one node, a text input named "<label>, <length> digits"
    /// with the digits as its value and SetValue advertised; masked, a
    /// password input; disabled, no SetValue and marked disabled.
    #[test]
    fn ctl_010_the_node_is_one_named_text_input_with_its_digits() {
        let props = CodeInputProps {
            label: Some("Code".to_owned()),
            value: "123".to_owned(),
            ..Default::default()
        };
        let (input, _, _) = logged(props.clone());
        let element = input.render(&props, &CodeInputState::default());
        let accessible = node(&element);
        assert_eq!(accessible.inner.role(), accesskit::Role::TextInput);
        assert_eq!(accessible.inner.label(), Some("Code, 6 digits"));
        assert_eq!(accessible.inner.value(), Some("123"));
        assert!(accessible
            .inner
            .supports_action(accesskit::Action::SetValue));
        assert!(!accessible.inner.is_disabled());
        assert_eq!(
            element
                .metadata
                .accessibility_options
                .as_ref()
                .and_then(|options| options.set_value_event.as_ref())
                .map(|event| event.name.as_str()),
            Some(SET_VALUE_EVENT),
            "the SetValue request reaches the code input"
        );

        let masked = CodeInputProps {
            masked: true,
            value: "12".to_owned(),
            aria_label: Some("One-time code".to_owned()),
            ..Default::default()
        };
        let element = input.render(&masked, &CodeInputState::default());
        assert_eq!(node(&element).inner.role(), accesskit::Role::PasswordInput);
        assert_eq!(
            node(&element).inner.label(),
            Some("One-time code, 6 digits")
        );
        assert_eq!(
            node(&element).inner.value(),
            Some("••"),
            "a masked code gives bullets, one per digit, not the digits"
        );

        let disabled = CodeInputProps {
            disabled: true,
            ..Default::default()
        };
        let element = input.render(&disabled, &CodeInputState::default());
        assert!(node(&element).inner.is_disabled());
        assert!(!node(&element)
            .inner
            .supports_action(accesskit::Action::SetValue));
    }

    /// CTL-007: a masked filled cell shows the catalog's dot, never a digit,
    /// and the cells are in groups with one gap between them.
    #[test]
    fn ctl_007_a_masked_code_shows_the_catalog_dot_in_groups() {
        let props = CodeInputProps {
            length: 6,
            group: Some(3),
            value: "123".to_owned(),
            masked: true,
            ..Default::default()
        };
        let text =
            painted(&CodeInput::new(props.clone()).render(&props, &CodeInputState::default()));
        let dot = Icon::Dot.glyph();
        assert_eq!(
            text,
            format!("[{dot}][{dot}][{dot}] [_][_][_]"),
            "three dots, a gap, three empty cells"
        );
        assert!(!text.chars().any(|c| c.is_ascii_digit()));
    }
}
