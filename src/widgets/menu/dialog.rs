use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme};
use std::any::Any;
use std::sync::Arc;

/// Type of dialog menu
#[derive(Clone, Debug, PartialEq)]
pub enum DialogMenuType {
    /// Simple selection dialog with a list of options
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

impl Default for DialogMenuType {
    fn default() -> Self {
        Self::Selection
    }
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
#[derive(Clone, Debug, Default)]
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
    /// Cursor position in input text
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
        if item_count == 0 {
            return;
        }
        
        self.selected_index = Some(match self.selected_index {
            Some(idx) => (idx + 1) % item_count,
            None => 0,
        });
    }

    /// Select the previous menu item
    pub fn select_previous(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        
        self.selected_index = Some(match self.selected_index {
            Some(idx) => {
                if idx == 0 {
                    item_count - 1
                } else {
                    idx - 1
                }
            }
            None => item_count - 1,
        });
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
        if let Some(selected) = self.selected_index {
            if selected < self.scroll_offset {
                self.scroll_offset = selected;
            } else if selected >= self.scroll_offset + max_visible {
                self.scroll_offset = selected - max_visible + 1;
            }
        }
    }

    /// Check if a point is inside the dialog area
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        if let Some((dx, dy, dw, dh)) = self.dialog_area {
            x >= dx && x < dx + dw && y >= dy && y < dy + dh
        } else {
            false
        }
    }

    /// Insert character at cursor position in input text
    pub fn insert_char(&mut self, ch: char) {
        self.input_text.insert(self.input_cursor, ch);
        self.input_cursor += 1;
    }

    /// Delete character before cursor in input text
    pub fn delete_char(&mut self) {
        if self.input_cursor > 0 {
            self.input_text.remove(self.input_cursor - 1);
            self.input_cursor -= 1;
        }
    }

    /// Move cursor left in input text
    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// Move cursor right in input text
    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_text.len() {
            self.input_cursor += 1;
        }
    }
}

/// DialogMenu component for modal dialogs and complex interactions
#[derive(Default)]
pub struct DialogMenu {
    state: DialogMenuState,
    on_item_selected: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_confirmed: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    on_cancelled: Option<Arc<dyn Fn() + Send + Sync>>,
    on_input_submitted: Option<Arc<dyn Fn(&str) + Send + Sync>>,
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

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: &KeyEvent, props: &DialogMenuProps, state: &mut DialogMenuState) -> EventResult {
        if !props.enabled || !props.visible {
            return EventResult::Ignored;
        }

        match props.dialog_type {
            DialogMenuType::Input => {
                match key.code {
                    KeyCode::Char(ch) => {
                        state.insert_char(ch);
                        EventResult::Handled
                    }
                    KeyCode::Backspace => {
                        state.delete_char();
                        EventResult::Handled
                    }
                    KeyCode::Left => {
                        state.move_cursor_left();
                        EventResult::Handled
                    }
                    KeyCode::Right => {
                        state.move_cursor_right();
                        EventResult::Handled
                    }
                    KeyCode::Enter => {
                        if let Some(callback) = &self.on_input_submitted {
                            callback(&state.input_text);
                        }
                        state.hide();
                        if let Some(callback) = &self.on_hide {
                            callback();
                        }
                        EventResult::Handled
                    }
                    KeyCode::Escape => {
                        if props.close_on_escape {
                            if let Some(callback) = &self.on_cancelled {
                                callback();
                            }
                            state.hide();
                            if let Some(callback) = &self.on_hide {
                                callback();
                            }
                        }
                        EventResult::Handled
                    }
                    _ => EventResult::Ignored,
                }
            }
            _ => {
                match key.code {
                    KeyCode::Up => {
                        state.select_previous(props.items.len());
                        // Calculate scroll based on dialog height and visible items
                        let visible_items = self.calculate_visible_items(props);
                        state.update_scroll(visible_items);
                        EventResult::Handled
                    }
                    KeyCode::Down => {
                        state.select_next(props.items.len());
                        // Calculate scroll based on dialog height and visible items
                        let visible_items = self.calculate_visible_items(props);
                        state.update_scroll(visible_items);
                        EventResult::Handled
                    }
                    KeyCode::Enter => {
                        match props.dialog_type {
                            DialogMenuType::Selection | DialogMenuType::Custom => {
                                if let Some(selected) = state.selected_index {
                                    if selected < props.items.len() {
                                        let item = &props.items[selected];
                                        if item.is_selectable() {
                                            item.execute();
                                            if let Some(callback) = &self.on_item_selected {
                                                callback(&item.id);
                                            }
                                            state.hide();
                                            if let Some(callback) = &self.on_hide {
                                                callback();
                                            }
                                        }
                                    }
                                }
                            }
                            DialogMenuType::MultiSelection => {
                                if let Some(selected) = state.selected_index {
                                    state.toggle_selection(selected);
                                }
                            }
                            DialogMenuType::Confirmation => {
                                if let Some(default_btn) = props.default_button {
                                    if default_btn < props.items.len() {
                                        let item = &props.items[default_btn];
                                        item.execute();
                                        if let Some(callback) = &self.on_item_selected {
                                            callback(&item.id);
                                        }
                                        state.hide();
                                        if let Some(callback) = &self.on_hide {
                                            callback();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        EventResult::Handled
                    }
                    KeyCode::Escape => {
                        if props.close_on_escape {
                            if let Some(cancel_btn) = props.cancel_button {
                                if cancel_btn < props.items.len() {
                                    let item = &props.items[cancel_btn];
                                    item.execute();
                                }
                            }
                            if let Some(callback) = &self.on_cancelled {
                                callback();
                            }
                            state.hide();
                            if let Some(callback) = &self.on_hide {
                                callback();
                            }
                        }
                        EventResult::Handled
                    }
                    KeyCode::Tab => {
                        // For multi-selection, Tab could confirm selection
                        if matches!(props.dialog_type, DialogMenuType::MultiSelection) {
                            let selected_items: Vec<String> = state.selected_items
                                .iter()
                                .filter_map(|&i| {
                                    if i < props.items.len() {
                                        Some(props.items[i].id.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            
                            if let Some(callback) = &self.on_confirmed {
                                callback(selected_items);
                            }
                            state.hide();
                            if let Some(callback) = &self.on_hide {
                                callback();
                            }
                        }
                        EventResult::Handled
                    }
                    KeyCode::Char(' ') => {
                        // Space toggles selection in multi-selection mode
                        if matches!(props.dialog_type, DialogMenuType::MultiSelection) {
                            if let Some(selected) = state.selected_index {
                                state.toggle_selection(selected);
                            }
                        }
                        EventResult::Handled
                    }
                    _ => EventResult::Ignored,
                }
            }
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: &MouseEvent, props: &DialogMenuProps, state: &mut DialogMenuState) -> EventResult {
        if !props.enabled || !props.visible {
            return EventResult::Ignored;
        }

        let (mouse_x, mouse_y) = match mouse.position {
            crate::event::types::Position::Cell { x, y } => (x, y),
            crate::event::types::Position::Pixel { x, y } => (x as u16, y as u16),
        };
        
        state.mouse_position = Some((mouse_x, mouse_y));

        match mouse.kind {
            MouseEventKind::Down => {
                if state.contains_point(mouse_x, mouse_y) {
                    // Click inside dialog - handle item selection
                    state.is_focused = true;

                    // Calculate which item was clicked based on layout
                    if let Some(clicked_item) = self.calculate_clicked_item(mouse_x, mouse_y, props, state) {
                        state.selected_index = Some(clicked_item);

                        // If it's a selection dialog, trigger selection
                        if props.dialog_type == DialogMenuType::Selection {
                            if let Some(callback) = &self.on_item_selected {
                                if let Some(item) = props.items.get(clicked_item) {
                                    callback(&item.text);
                                }
                            }
                        }
                    }

                    EventResult::Handled
                } else if props.close_on_outside_click && !props.modal {
                    // Click outside dialog - close it
                    if let Some(callback) = &self.on_cancelled {
                        callback();
                    }
                    state.hide();
                    if let Some(callback) = &self.on_hide {
                        callback();
                    }
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            MouseEventKind::Move => {
                if state.contains_point(mouse_x, mouse_y) {
                    state.is_hovered = true;

                    // Update selection based on mouse position
                    if let Some(hovered_item) = self.calculate_clicked_item(mouse_x, mouse_y, props, state) {
                        // Only update selection if it's different from current
                        if state.selected_index != Some(hovered_item) {
                            state.selected_index = Some(hovered_item);
                        }
                    }

                    EventResult::Handled
                } else {
                    state.is_hovered = false;
                    // Clear selection when mouse leaves dialog area
                    if state.selected_index.is_some() {
                        state.selected_index = None;
                    }
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// Calculate the number of visible items based on dialog height
    fn calculate_visible_items(&self, props: &DialogMenuProps) -> usize {
        // Default dialog height calculation
        let default_height: usize = 20; // Default dialog height in lines

        // Calculate available space for items
        // Reserve space for: title (1), borders (2), padding (2), buttons area (3)
        let reserved_space: usize = if props.title.is_some() { 1 } else { 0 } +
                                   if props.show_border { 2 } else { 0 } +
                                   2 + // padding
                                   3; // buttons/input area

        let available_height = default_height.saturating_sub(reserved_space);

        // Ensure at least 3 items are visible
        available_height.max(3)
    }

    /// Calculate which menu item was clicked based on mouse position
    fn calculate_clicked_item(&self, _mouse_x: u16, mouse_y: u16, props: &DialogMenuProps, state: &DialogMenuState) -> Option<usize> {
        // This is a simplified calculation - in a real implementation, you'd want to
        // track the exact layout coordinates during rendering

        // Calculate dialog content area
        // Use a default position since we don't have access to the actual dialog position
        let dialog_start_y = 5 + if props.show_border { 1 } else { 0 } +
                            if props.title.is_some() { 1 } else { 0 } + 1; // padding

        // Check if click is within the items area
        if mouse_y < dialog_start_y {
            return None;
        }

        // Calculate which item was clicked (each item takes 1 line)
        let item_index = (mouse_y - dialog_start_y) as usize + state.scroll_offset;

        // Check if the calculated index is valid
        if item_index < props.items.len() {
            Some(item_index)
        } else {
            None
        }
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

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let was_visible = self.state.is_focused;
        self.state = state.clone();
        
        // Trigger callbacks for visibility changes
        if props.visible && !was_visible {
            if let Some(callback) = &self.on_show {
                callback();
            }
        } else if !props.visible && was_visible {
            if let Some(callback) = &self.on_hide {
                callback();
            }
        }
        
        true
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        if !props.visible {
            return Element::empty();
        }

        // Create the dialog element
        // This is a simplified version - in a real implementation, you'd need to:
        // 1. Calculate dialog position and size
        // 2. Render title, message, and menu items
        // 3. Handle different dialog types (input field, checkboxes, etc.)
        // 4. Render borders, shadows, and close button if enabled
        // 5. Handle modal overlay if modal is true
        
        Element::layout(crate::component::element::LayoutType::Flex)
            .with_key("dialog-menu")
            .with_class(&props.style.base_classes)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, props, state),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            _ => EventResult::Ignored,
        }
    }
}

/// Builder for creating DialogMenu components with a fluent API
#[allow(dead_code)]
pub struct DialogMenuBuilder {
    props: DialogMenuProps,
}

#[allow(dead_code)]
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