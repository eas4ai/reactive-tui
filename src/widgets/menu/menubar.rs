use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme};
use std::any::Any;
use std::sync::Arc;

/// Properties for MenuBar component
#[derive(Clone, Debug, PartialEq)]
pub struct MenuBarProps {
    /// Menu items to display in the menubar
    pub items: Vec<MenuItem>,
    /// Style configuration for the menubar
    pub style: MenuStyle,
    /// Whether the menubar is enabled
    pub enabled: bool,
    /// Whether the menubar is visible
    pub visible: bool,
    /// Title text to display on the left side of the menubar
    pub title: Option<String>,
    /// Whether to show keyboard shortcuts in submenus
    pub show_shortcuts: bool,
    /// Maximum number of visible items in dropdown menus
    pub max_dropdown_height: usize,
}

impl Default for MenuBarProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            enabled: true,
            visible: true,
            title: None,
            show_shortcuts: true,
            max_dropdown_height: 10,
        }
    }
}

impl Props for MenuBarProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for MenuBar component
#[derive(Clone, Debug, Default)]
pub struct MenuBarState {
    /// Index of the currently selected menu item
    pub selected_index: Option<usize>,
    /// Index of the currently highlighted submenu item
    pub submenu_selected_index: Option<usize>,
    /// Whether a dropdown menu is currently open
    pub dropdown_open: bool,
    /// Whether the menubar has focus
    pub is_focused: bool,
    /// Scroll offset for the dropdown menu
    pub dropdown_scroll_offset: usize,
    /// Whether mouse is hovering over the menubar
    pub is_hovered: bool,
    /// Position of the last mouse event
    pub mouse_position: Option<(u16, u16)>,
}

impl MenuBarState {
    /// Create a new menubar state
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dropdown for the currently selected item
    pub fn open_dropdown(&mut self) {
        if self.selected_index.is_some() {
            self.dropdown_open = true;
            self.submenu_selected_index = Some(0);
            self.dropdown_scroll_offset = 0;
        }
    }

    /// Close the dropdown menu
    pub fn close_dropdown(&mut self) {
        self.dropdown_open = false;
        self.submenu_selected_index = None;
        self.dropdown_scroll_offset = 0;
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

    /// Select the next submenu item
    pub fn select_next_submenu(&mut self, submenu_count: usize) {
        if submenu_count == 0 {
            return;
        }
        
        self.submenu_selected_index = Some(match self.submenu_selected_index {
            Some(idx) => (idx + 1) % submenu_count,
            None => 0,
        });
    }

    /// Select the previous submenu item
    pub fn select_previous_submenu(&mut self, submenu_count: usize) {
        if submenu_count == 0 {
            return;
        }
        
        self.submenu_selected_index = Some(match self.submenu_selected_index {
            Some(idx) => {
                if idx == 0 {
                    submenu_count - 1
                } else {
                    idx - 1
                }
            }
            None => submenu_count - 1,
        });
    }

    /// Update scroll offset to keep selected submenu item visible
    pub fn update_submenu_scroll(&mut self, max_visible: usize) {
        if let Some(selected) = self.submenu_selected_index {
            if selected < self.dropdown_scroll_offset {
                self.dropdown_scroll_offset = selected;
            } else if selected >= self.dropdown_scroll_offset + max_visible {
                self.dropdown_scroll_offset = selected - max_visible + 1;
            }
        }
    }
}

/// MenuBar component with dropdown submenus
#[derive(Default)]
pub struct MenuBar {
    state: MenuBarState,
    on_item_selected: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_dropdown_opened: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_dropdown_closed: Option<Arc<dyn Fn() + Send + Sync>>,
}


impl MenuBar {
    /// Set callback for when a menu item is selected
    pub fn with_on_item_selected(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(f));
        self
    }

    /// Set callback for when a dropdown is opened
    pub fn with_on_dropdown_opened(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_dropdown_opened = Some(Arc::new(f));
        self
    }

    /// Set callback for when a dropdown is closed
    pub fn with_on_dropdown_closed(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_dropdown_closed = Some(Arc::new(f));
        self
    }

    /// Handle keyboard navigation
    fn handle_key_event(&mut self, key: &KeyEvent, props: &MenuBarProps, state: &mut MenuBarState) -> EventResult {
        if !props.enabled {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Left => {
                if state.dropdown_open {
                    // Navigate between main menu items while dropdown is open
                    state.select_previous(props.items.len());
                    if let Some(selected) = state.selected_index {
                        if props.items[selected].has_submenu() {
                            state.submenu_selected_index = Some(0);
                            state.dropdown_scroll_offset = 0;
                        } else {
                            state.close_dropdown();
                        }
                    }
                } else {
                    state.select_previous(props.items.len());
                }
                EventResult::Handled
            }
            KeyCode::Right => {
                if state.dropdown_open {
                    // Navigate between main menu items while dropdown is open
                    state.select_next(props.items.len());
                    if let Some(selected) = state.selected_index {
                        if props.items[selected].has_submenu() {
                            state.submenu_selected_index = Some(0);
                            state.dropdown_scroll_offset = 0;
                        } else {
                            state.close_dropdown();
                        }
                    }
                } else {
                    state.select_next(props.items.len());
                }
                EventResult::Handled
            }
            KeyCode::Up => {
                if state.dropdown_open {
                    if let Some(selected) = state.selected_index {
                        if selected < props.items.len() && props.items[selected].has_submenu() {
                            state.select_previous_submenu(props.items[selected].submenu.len());
                            state.update_submenu_scroll(props.max_dropdown_height);
                        }
                    }
                }
                EventResult::Handled
            }
            KeyCode::Down => {
                if state.dropdown_open {
                    if let Some(selected) = state.selected_index {
                        if selected < props.items.len() && props.items[selected].has_submenu() {
                            state.select_next_submenu(props.items[selected].submenu.len());
                            state.update_submenu_scroll(props.max_dropdown_height);
                        }
                    }
                } else if state.selected_index.is_some() {
                    // Open dropdown when pressing down
                    state.open_dropdown();
                    if let Some(callback) = &self.on_dropdown_opened {
                        if let Some(idx) = state.selected_index {
                            callback(idx);
                        }
                    }
                }
                EventResult::Handled
            }
            KeyCode::Enter => {
                if state.dropdown_open {
                    // Execute submenu item
                    if let (Some(main_idx), Some(sub_idx)) = (state.selected_index, state.submenu_selected_index) {
                        if main_idx < props.items.len() {
                            let main_item = &props.items[main_idx];
                            if sub_idx < main_item.submenu.len() {
                                let sub_item = &main_item.submenu[sub_idx];
                                if sub_item.is_selectable() {
                                    sub_item.execute();
                                    if let Some(callback) = &self.on_item_selected {
                                        callback(&sub_item.id);
                                    }
                                    state.close_dropdown();
                                    if let Some(callback) = &self.on_dropdown_closed {
                                        callback();
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(selected) = state.selected_index {
                    // Execute main menu item or open dropdown
                    if selected < props.items.len() {
                        let item = &props.items[selected];
                        if item.has_submenu() {
                            state.open_dropdown();
                            if let Some(callback) = &self.on_dropdown_opened {
                                callback(selected);
                            }
                        } else if item.is_selectable() {
                            item.execute();
                            if let Some(callback) = &self.on_item_selected {
                                callback(&item.id);
                            }
                        }
                    }
                }
                EventResult::Handled
            }
            KeyCode::Escape => {
                if state.dropdown_open {
                    state.close_dropdown();
                    if let Some(callback) = &self.on_dropdown_closed {
                        callback();
                    }
                } else {
                    state.selected_index = None;
                    state.is_focused = false;
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: &MouseEvent, props: &MenuBarProps, state: &mut MenuBarState) -> EventResult {
        if !props.enabled {
            return EventResult::Ignored;
        }

        state.mouse_position = match mouse.position {
            crate::event::types::Position::Cell { x, y } => Some((x, y)),
            crate::event::types::Position::Pixel { x, y } => Some((x as u16, y as u16)),
        };

        match mouse.kind {
            MouseEventKind::Down => {
                // Handle clicks on menu items
                // This would need actual layout information to determine which item was clicked
                // For now, we'll just focus the menubar
                state.is_focused = true;
                EventResult::Handled
            }
            MouseEventKind::Move => {
                state.is_hovered = true;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Component for MenuBar {
    type Props = MenuBarProps;
    type State = MenuBarState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: MenuBarState::default(),
            on_item_selected: None,
            on_dropdown_opened: None,
            on_dropdown_closed: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        // Update internal state from external state
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        if !props.visible {
            return Element::empty();
        }

        // Create the menubar element
        // This is a simplified version - in a real implementation, you'd need to:
        // 1. Calculate layout for menu items
        // 2. Render the main menubar
        // 3. Render dropdown menus if open
        // 4. Handle styling and theming
        
        Element::layout(crate::component::element::LayoutType::Flex)
            .with_key("menubar")
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

/// Builder for creating MenuBar components with a fluent API
pub struct MenuBarBuilder {
    props: MenuBarProps,
}

impl MenuBarBuilder {
    /// Create a new menubar builder
    pub fn new() -> Self {
        Self {
            props: MenuBarProps::default(),
        }
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

    /// Set whether the menubar is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.props.enabled = enabled;
        self
    }

    /// Set whether the menubar is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.props.visible = visible;
        self
    }

    /// Set the title text
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.props.title = Some(title.into());
        self
    }

    /// Set whether to show shortcuts
    pub fn show_shortcuts(mut self, show: bool) -> Self {
        self.props.show_shortcuts = show;
        self
    }

    /// Set maximum dropdown height
    pub fn max_dropdown_height(mut self, height: usize) -> Self {
        self.props.max_dropdown_height = height;
        self
    }

    /// Build the menubar props
    pub fn build(self) -> MenuBarProps {
        self.props
    }
}

impl Default for MenuBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}