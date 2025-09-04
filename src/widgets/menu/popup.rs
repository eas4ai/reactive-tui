use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme};
use std::any::Any;
use std::sync::Arc;

/// Placement options for popup menus
#[derive(Clone, Debug, PartialEq)]
pub enum PopupPlacement {
    /// Place popup at cursor position
    Cursor,
    /// Place popup at specific coordinates
    Position { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16 
    },
    /// Place popup relative to a widget area
    Widget { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16, 
        /// Widget width
        width: u16, 
        /// Widget height
        height: u16 
    },
    /// Place popup below the specified area
    Below { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16, 
        /// Area width
        width: u16 
    },
    /// Place popup above the specified area
    Above { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16, 
        /// Area width
        width: u16 
    },
    /// Place popup to the right of the specified area
    Right { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16, 
        /// Area height
        height: u16 
    },
    /// Place popup to the left of the specified area
    Left { 
        /// X coordinate
        x: u16, 
        /// Y coordinate
        y: u16, 
        /// Area height
        height: u16 
    },
}

impl Default for PopupPlacement {
    fn default() -> Self {
        Self::Cursor
    }
}

/// Properties for PopupMenu component
#[derive(Clone, Debug, PartialEq)]
pub struct PopupMenuProps {
    /// Menu items to display
    pub items: Vec<MenuItem>,
    /// Style configuration
    pub style: MenuStyle,
    /// Whether the popup is visible
    pub visible: bool,
    /// Whether the popup is enabled
    pub enabled: bool,
    /// Placement configuration
    pub placement: PopupPlacement,
    /// Whether to auto-close on item selection
    pub auto_close: bool,
    /// Whether to auto-close when clicking outside
    pub close_on_outside_click: bool,
    /// Maximum number of visible items (for scrolling)
    pub max_visible_items: usize,
    /// Fixed width for the popup
    pub width: Option<u16>,
    /// Whether to show a border around the popup
    pub show_border: bool,
    /// Whether to show a shadow
    pub show_shadow: bool,
}

impl Default for PopupMenuProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            visible: false,
            enabled: true,
            placement: PopupPlacement::default(),
            auto_close: true,
            close_on_outside_click: true,
            max_visible_items: 10,
            width: None,
            show_border: true,
            show_shadow: true,
        }
    }
}

impl Props for PopupMenuProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for PopupMenu component
#[derive(Clone, Debug, Default)]
pub struct PopupMenuState {
    /// Index of the currently selected item
    pub selected_index: Option<usize>,
    /// Whether the popup has focus
    pub is_focused: bool,
    /// Scroll offset for long menus
    pub scroll_offset: usize,
    /// Whether mouse is hovering over the popup
    pub is_hovered: bool,
    /// Position of the last mouse event
    pub mouse_position: Option<(u16, u16)>,
    /// Calculated popup area (set during rendering)
    pub popup_area: Option<(u16, u16, u16, u16)>, // x, y, width, height
}

impl PopupMenuState {
    /// Create a new popup menu state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the popup menu
    pub fn show(&mut self) {
        self.is_focused = true;
        self.selected_index = Some(0);
        self.scroll_offset = 0;
    }

    /// Hide the popup menu
    pub fn hide(&mut self) {
        self.is_focused = false;
        self.selected_index = None;
        self.scroll_offset = 0;
        self.is_hovered = false;
        self.mouse_position = None;
        self.popup_area = None;
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

    /// Check if a point is inside the popup area
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        if let Some((px, py, pw, ph)) = self.popup_area {
            x >= px && x < px + pw && y >= py && y < py + ph
        } else {
            false
        }
    }
}

/// PopupMenu component for context menus and dropdowns
#[derive(Default)]
pub struct PopupMenu {
    state: PopupMenuState,
    on_item_selected: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_show: Option<Arc<dyn Fn() + Send + Sync>>,
    on_hide: Option<Arc<dyn Fn() + Send + Sync>>,
}


impl PopupMenu {
    /// Set callback for when a menu item is selected
    pub fn with_on_item_selected(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(f));
        self
    }

    /// Set callback for when the popup is shown
    pub fn with_on_show(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_show = Some(Arc::new(f));
        self
    }

    /// Set callback for when the popup is hidden
    pub fn with_on_hide(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_hide = Some(Arc::new(f));
        self
    }

    /// Handle keyboard navigation
    fn handle_key_event(&mut self, key: &KeyEvent, props: &PopupMenuProps, state: &mut PopupMenuState) -> EventResult {
        if !props.enabled || !props.visible {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Up => {
                state.select_previous(props.items.len());
                state.update_scroll(props.max_visible_items);
                EventResult::Handled
            }
            KeyCode::Down => {
                state.select_next(props.items.len());
                state.update_scroll(props.max_visible_items);
                EventResult::Handled
            }
            KeyCode::Enter => {
                if let Some(selected) = state.selected_index {
                    if selected < props.items.len() {
                        let item = &props.items[selected];
                        if item.is_selectable() {
                            item.execute();
                            if let Some(callback) = &self.on_item_selected {
                                callback(&item.id);
                            }
                            if props.auto_close {
                                state.hide();
                                if let Some(callback) = &self.on_hide {
                                    callback();
                                }
                            }
                        }
                    }
                }
                EventResult::Handled
            }
            KeyCode::Escape => {
                state.hide();
                if let Some(callback) = &self.on_hide {
                    callback();
                }
                EventResult::Handled
            }
            KeyCode::Home => {
                state.selected_index = Some(0);
                state.scroll_offset = 0;
                EventResult::Handled
            }
            KeyCode::End => {
                if !props.items.is_empty() {
                    state.selected_index = Some(props.items.len() - 1);
                    state.update_scroll(props.max_visible_items);
                }
                EventResult::Handled
            }
            KeyCode::PageUp => {
                if let Some(selected) = state.selected_index {
                    let new_selected = selected.saturating_sub(props.max_visible_items);
                    state.selected_index = Some(new_selected);
                    state.update_scroll(props.max_visible_items);
                }
                EventResult::Handled
            }
            KeyCode::PageDown => {
                if let Some(selected) = state.selected_index {
                    let new_selected = (selected + props.max_visible_items).min(props.items.len() - 1);
                    state.selected_index = Some(new_selected);
                    state.update_scroll(props.max_visible_items);
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: &MouseEvent, props: &PopupMenuProps, state: &mut PopupMenuState) -> EventResult {
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
                    // Click inside popup - handle item selection
                    // This would need actual layout information to determine which item was clicked
                    state.is_focused = true;
                    EventResult::Handled
                } else if props.close_on_outside_click {
                    // Click outside popup - close it
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
                    // This would need actual layout information
                    EventResult::Handled
                } else {
                    state.is_hovered = false;
                    EventResult::Ignored
                }
            }
            MouseEventKind::Wheel => {
                if state.contains_point(mouse_x, mouse_y) {
                    // TODO: Determine scroll direction from wheel data
                    // For now, just handle as generic wheel event
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Component for PopupMenu {
    type Props = PopupMenuProps;
    type State = PopupMenuState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: PopupMenuState::default(),
            on_item_selected: None,
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

        // Create the popup menu element
        // This is a simplified version - in a real implementation, you'd need to:
        // 1. Calculate popup position based on placement
        // 2. Render menu items with proper styling
        // 3. Handle scrolling for long menus
        // 4. Render borders and shadows if enabled
        
        Element::layout(crate::component::element::LayoutType::Flex)
            .with_key("popup-menu")
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

/// Builder for creating PopupMenu components with a fluent API
#[allow(dead_code)]
pub struct PopupMenuBuilder {
    props: PopupMenuProps,
}

#[allow(dead_code)]
impl PopupMenuBuilder {
    /// Create a new popup menu builder
    pub fn new() -> Self {
        Self {
            props: PopupMenuProps::default(),
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

    /// Set whether the popup is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.props.visible = visible;
        self
    }

    /// Set whether the popup is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.props.enabled = enabled;
        self
    }

    /// Set the placement
    pub fn placement(mut self, placement: PopupPlacement) -> Self {
        self.props.placement = placement;
        self
    }

    /// Set whether to auto-close on selection
    pub fn auto_close(mut self, auto_close: bool) -> Self {
        self.props.auto_close = auto_close;
        self
    }

    /// Set whether to close on outside click
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.props.close_on_outside_click = close;
        self
    }

    /// Set maximum visible items
    pub fn max_visible_items(mut self, max: usize) -> Self {
        self.props.max_visible_items = max;
        self
    }

    /// Set fixed width
    pub fn width(mut self, width: Option<u16>) -> Self {
        self.props.width = width;
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

    /// Build the popup menu props
    pub fn build(self) -> PopupMenuProps {
        self.props
    }
}

impl Default for PopupMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}