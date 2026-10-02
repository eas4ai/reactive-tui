use crate::component::{Component, Element, Props};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme};
use std::any::Any;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

/// Type of dialog menu
#[derive(Clone, Debug, PartialEq, Default)]
pub enum DialogMenuType {
    /// Simple selection dialog with a list of options
    #[default]
    Selection,
    /// Multi-selection dialog with checkboxes
    MultiSelection,
    /// Confirmation dialog with Yes/No/Cancel options
    Confirmation,
    /// Input dialog with text field and action buttons
    Input,
    /// Custom dialog with arbitrary menu items
    Custom,
}

/// Properties for DialogMenu component
#[derive(Clone, Debug, PartialEq)]
pub struct DialogMenuProps {
    /// Type of dialog menu
    pub dialog_type: DialogMenuType,
    /// Menu items to display
    pub items: Vec<MenuItem>,
    /// Style configuration
    pub style: MenuStyle,
    /// Whether the dialog is visible
    pub visible: bool,
    /// Whether the dialog is enabled
    pub enabled: bool,
    /// Dialog title
    pub title: Option<String>,
    /// Dialog message/description
    pub message: Option<String>,
    /// Whether the dialog is modal (blocks interaction with other elements)
    pub modal: bool,
    /// Whether to show a close button
    pub show_close_button: bool,
    /// Whether to close on escape key
    pub close_on_escape: bool,
    /// Whether to close when clicking outside (if not modal)
    pub close_on_outside_click: bool,
    /// Fixed width for the dialog
    pub width: Option<u16>,
    /// Fixed height for the dialog
    pub height: Option<u16>,
    /// Whether to center the dialog on screen
    pub centered: bool,
    /// Custom position (if not centered)
    pub position: Option<(u16, u16)>,
    /// Whether to show a border around the dialog
    pub show_border: bool,
    /// Whether to show a shadow
    pub show_shadow: bool,
    /// Default button index (for Enter key)
    pub default_button: Option<usize>,
    /// Cancel button index (for Escape key)
    pub cancel_button: Option<usize>,
}

impl Default for DialogMenuProps {
    fn default() -> Self {
        Self {
            dialog_type: DialogMenuType::default(),
            items: Vec::new(),
            style: MenuStyle::default(),
            visible: false,
            enabled: true,
            title: None,
            message: None,
            modal: true,
            show_close_button: true,
            close_on_escape: true,
            close_on_outside_click: false,
            width: Some(40),
            height: None,
            centered: true,
            position: None,
            show_border: true,
            show_shadow: true,
            default_button: None,
            cancel_button: None,
        }
    }
}

impl Props for DialogMenuProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for DialogMenu component
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogMenuState {
    /// Index of the currently selected item
    pub selected_index: Option<usize>,
    /// Whether the dialog has focus
    pub is_focused: bool,
    /// Scroll offset for long menus
    pub scroll_offset: usize,
    /// Whether mouse is hovering over the dialog
    pub is_hovered: bool,
    /// Position of the last mouse event
    pub mouse_position: Option<(u16, u16)>,
    /// Calculated dialog area (set during rendering)
    pub dialog_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
    /// Selected items for multi-selection dialogs
    pub selected_items: Vec<usize>,
    /// Input text for input dialogs
    pub input_text: String,
    /// UTF-8 byte offset of the cursor, normalized to a grapheme boundary by editing methods
    pub input_cursor: usize,
}

impl DialogMenuState {
    /// Create a new dialog menu state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.is_focused = true;
        self.selected_index = Some(0);
        self.scroll_offset = 0;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.is_focused = false;
        self.selected_index = None;
        self.scroll_offset = 0;
        self.is_hovered = false;
        self.mouse_position = None;
        self.dialog_area = None;
        self.selected_items.clear();
        self.input_text.clear();
        self.input_cursor = 0;
    }

    /// Select the next menu item
    pub fn select_next(&mut self, item_count: usize) {
        self.selected_index = super::state::step(self.selected_index, item_count, true);
    }

    /// Select the previous menu item
    pub fn select_previous(&mut self, item_count: usize) {
        self.selected_index = super::state::step(self.selected_index, item_count, false);
    }

    /// Toggle selection for multi-selection dialogs
    pub fn toggle_selection(&mut self, index: usize) {
        if let Some(pos) = self.selected_items.iter().position(|&x| x == index) {
            self.selected_items.remove(pos);
        } else {
            self.selected_items.push(index);
        }
    }

    /// Check if an item is selected in multi-selection mode
    pub fn is_item_selected(&self, index: usize) -> bool {
        self.selected_items.contains(&index)
    }

    /// Update scroll offset to keep selected item visible
    pub fn update_scroll(&mut self, max_visible: usize) {
        super::state::keep_visible(self.selected_index, &mut self.scroll_offset, max_visible);
    }

    /// Check if a point is inside the dialog area
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        self.dialog_area
            .is_some_and(|rect| super::state::contains(rect, (x, y)))
    }

    pub(super) fn input_boundary(&self) -> usize {
        self.input_text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain(std::iter::once(self.input_text.len()))
            .take_while(|index| *index <= self.input_cursor)
            .last()
            .unwrap_or(0)
    }

    /// Insert a character and move past its complete grapheme cluster.
    pub fn insert_char(&mut self, ch: char) {
        self.insert_text(ch.encode_utf8(&mut [0; 4]));
    }

    pub(super) fn insert_text(&mut self, text: &str) {
        let cursor = self.input_boundary();
        self.input_text.insert_str(cursor, text);
        let after = cursor + text.len();
        self.input_cursor = self
            .input_text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .find(|index| *index >= after)
            .unwrap_or(self.input_text.len());
    }

    /// Delete the grapheme cluster before the cursor.
    pub fn delete_char(&mut self) {
        let cursor = self.input_boundary();
        let previous = self.input_text[..cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(index, _)| index);
        self.input_text.replace_range(previous..cursor, "");
        self.input_cursor = previous;
    }

    /// Move left by one grapheme cluster.
    pub fn move_cursor_left(&mut self) {
        let cursor = self.input_boundary();
        self.input_cursor = self.input_text[..cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(index, _)| index);
    }

    /// Move right by one grapheme cluster.
    pub fn move_cursor_right(&mut self) {
        let cursor = self.input_boundary();
        self.input_cursor = self
            .input_text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .find(|index| *index > cursor)
            .unwrap_or(self.input_text.len());
    }
}

/// DialogMenu component for modal dialogs and complex interactions
#[derive(Default)]
pub struct DialogMenu {
    state: DialogMenuState,
    on_item_selected: Option<super::TextCallback>,
    on_confirmed: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    on_cancelled: Option<Arc<dyn Fn() + Send + Sync>>,
    on_input_submitted: Option<super::TextCallback>,
    on_show: Option<Arc<dyn Fn() + Send + Sync>>,
    on_hide: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl DialogMenu {
    /// Set callback for when a menu item is selected
    pub fn with_on_item_selected(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(f));
        self
    }

    /// Set callback for when the dialog is confirmed (multi-selection)
    pub fn with_on_confirmed(mut self, f: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.on_confirmed = Some(Arc::new(f));
        self
    }

    /// Set callback for when the dialog is cancelled
    pub fn with_on_cancelled(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancelled = Some(Arc::new(f));
        self
    }

    /// Set callback for when input is submitted
    pub fn with_on_input_submitted(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_input_submitted = Some(Arc::new(f));
        self
    }

    /// Set callback for when the dialog is shown
    pub fn with_on_show(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_show = Some(Arc::new(f));
        self
    }

    /// Set callback for when the dialog is hidden
    pub fn with_on_hide(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_hide = Some(Arc::new(f));
        self
    }
}

impl Component for DialogMenu {
    type Props = DialogMenuProps;
    type State = DialogMenuState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: DialogMenuState::default(),
            on_item_selected: None,
            on_confirmed: None,
            on_cancelled: None,
            on_input_submitted: None,
            on_show: None,
            on_hide: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<super::dialog_live::LiveDialog>(super::dialog_live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
            selected: self.on_item_selected.clone(),
            confirmed: self.on_confirmed.clone(),
            cancelled: self.on_cancelled.clone(),
            submitted: self.on_input_submitted.clone(),
            shown: self.on_show.clone(),
            hidden: self.on_hide.clone(),
        })
    }
}

/// Builder for creating DialogMenu components with a fluent API
pub struct DialogMenuBuilder {
    props: DialogMenuProps,
}

impl DialogMenuBuilder {
    /// Create a new dialog menu builder
    pub fn new(dialog_type: DialogMenuType) -> Self {
        Self {
            props: DialogMenuProps {
                dialog_type,
                ..Default::default()
            },
        }
    }

    /// Create a selection dialog builder
    pub fn selection() -> Self {
        Self::new(DialogMenuType::Selection)
    }

    /// Create a multi-selection dialog builder
    pub fn multi_selection() -> Self {
        Self::new(DialogMenuType::MultiSelection)
    }

    /// Create a confirmation dialog builder
    pub fn confirmation() -> Self {
        Self::new(DialogMenuType::Confirmation)
    }

    /// Create an input dialog builder
    pub fn input() -> Self {
        Self::new(DialogMenuType::Input)
    }

    /// Create a custom dialog builder
    pub fn custom() -> Self {
        Self::new(DialogMenuType::Custom)
    }

    /// Set the menu items
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.props.items = items;
        self
    }

    /// Add a single menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.props.items.push(item);
        self
    }

    /// Set the style
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.props.style = style;
        self
    }

    /// Set the theme
    pub fn theme(mut self, theme: MenuTheme) -> Self {
        self.props.style = theme.to_style();
        self
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.props.title = Some(title.into());
        self
    }

    /// Set the message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.props.message = Some(message.into());
        self
    }

    /// Set whether the dialog is modal
    pub fn modal(mut self, modal: bool) -> Self {
        self.props.modal = modal;
        self
    }

    /// Set whether to show close button
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.props.show_close_button = show;
        self
    }

    /// Set whether to close on escape
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.props.close_on_escape = close;
        self
    }

    /// Set whether to close on outside click
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.props.close_on_outside_click = close;
        self
    }

    /// Set fixed width
    pub fn width(mut self, width: Option<u16>) -> Self {
        self.props.width = width;
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: Option<u16>) -> Self {
        self.props.height = height;
        self
    }

    /// Set whether to center the dialog
    pub fn centered(mut self, centered: bool) -> Self {
        self.props.centered = centered;
        self
    }

    /// Set custom position
    pub fn position(mut self, x: u16, y: u16) -> Self {
        self.props.position = Some((x, y));
        self.props.centered = false;
        self
    }

    /// Set whether to show border
    pub fn show_border(mut self, show: bool) -> Self {
        self.props.show_border = show;
        self
    }

    /// Set whether to show shadow
    pub fn show_shadow(mut self, show: bool) -> Self {
        self.props.show_shadow = show;
        self
    }

    /// Set default button index
    pub fn default_button(mut self, index: usize) -> Self {
        self.props.default_button = Some(index);
        self
    }

    /// Set cancel button index
    pub fn cancel_button(mut self, index: usize) -> Self {
        self.props.cancel_button = Some(index);
        self
    }

    /// Build the dialog menu props
    pub fn build(self) -> DialogMenuProps {
        self.props
    }
}

impl Default for DialogMenuBuilder {
    fn default() -> Self {
        Self::new(DialogMenuType::Selection)
    }
}

#[cfg(test)]
mod input_tests {
    use super::DialogMenuState;

    #[test]
    fn input_edits_and_moves_by_grapheme_with_byte_cursor_positions() {
        let mut state = DialogMenuState::default();
        for ch in "界e\u{301}👩\u{200d}💻".chars() {
            state.insert_char(ch);
        }
        assert_eq!(state.input_text, "界e\u{301}👩\u{200d}💻");
        assert_eq!(state.input_cursor, state.input_text.len());
        state.move_cursor_left();
        assert_eq!(state.input_cursor, "界e\u{301}".len());
        state.delete_char();
        assert_eq!(state.input_text, "界👩\u{200d}💻");
        assert_eq!(state.input_cursor, "界".len());
        state.move_cursor_right();
        state.delete_char();
        assert_eq!(state.input_text, "界");
        state.delete_char();
        assert_eq!(state.input_cursor, 0);
        assert!(state.input_text.is_empty());
    }

    #[test]
    fn externally_supplied_cursor_is_clamped_to_a_grapheme_boundary() {
        let mut state = DialogMenuState {
            input_text: "界X".into(),
            input_cursor: 1,
            ..Default::default()
        };
        state.insert_char('A');
        assert_eq!(state.input_text, "A界X");
        state.input_cursor = usize::MAX;
        state.delete_char();
        assert_eq!(state.input_text, "A界");
    }
}
