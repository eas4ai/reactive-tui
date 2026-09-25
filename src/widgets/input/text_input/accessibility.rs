use super::{InputMode, TextInput, TextInputProps, TextInputState};
use crate::{
    accessibility::{Node, Role},
    component::Element,
};

impl TextInput {
    pub(super) fn render_accessible(
        &self,
        props: &TextInputProps,
        state: &TextInputState,
    ) -> Element {
        let password = props.mode == InputMode::Password;
        let mut accessible = Node::new(match props.mode {
            InputMode::Password => Role::PasswordInput,
            InputMode::MultiLine { .. } => Role::MultilineTextInput,
            InputMode::Numeric => Role::NumberInput,
            InputMode::SingleLine => Role::TextInput,
        });
        if self.is_read_only() {
            accessible.set_read_only();
        }
        if let Some(placeholder) = &props.placeholder {
            accessible.set_description(placeholder.clone());
        }
        let mut element = self.render_control(props, state);
        let anchor = state
            .selection
            .as_ref()
            .map_or(state.cursor.byte_offset, |s| s.start.byte_offset);
        let focus = state
            .selection
            .as_ref()
            .map_or(state.cursor.byte_offset, |s| s.end.byte_offset);
        crate::accessibility::text::append_text_runs(
            &mut element,
            &mut accessible,
            &props.value,
            password,
            Some([anchor, focus]),
        );
        element.with_accessibility(accessible)
    }
}
