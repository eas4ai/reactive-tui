use crate::component::{Component, Element, Props};
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
#[derive(Clone, Debug, Default, PartialEq)]
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
        self.selected_index = super::state::step(self.selected_index, item_count, true);
    }

    /// Select the previous menu item
    pub fn select_previous(&mut self, item_count: usize) {
        self.selected_index = super::state::step(self.selected_index, item_count, false);
    }

    /// Select the next submenu item
    pub fn select_next_submenu(&mut self, submenu_count: usize) {
        self.submenu_selected_index =
            super::state::step(self.submenu_selected_index, submenu_count, true);
    }

    /// Select the previous submenu item
    pub fn select_previous_submenu(&mut self, submenu_count: usize) {
        self.submenu_selected_index =
            super::state::step(self.submenu_selected_index, submenu_count, false);
    }

    /// Update scroll offset to keep selected submenu item visible
    pub fn update_submenu_scroll(&mut self, max_visible: usize) {
        super::state::keep_visible(
            self.submenu_selected_index,
            &mut self.dropdown_scroll_offset,
            max_visible,
        );
    }
}

/// MenuBar component with dropdown submenus
#[derive(Default)]
pub struct MenuBar {
    state: MenuBarState,
    on_item_selected: Option<super::TextCallback>,
    on_dropdown_opened: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_dropdown_closed: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl MenuBar {
    pub(crate) fn element_with_callbacks(
        config: MenuBarProps,
        selected: Option<super::TextCallback>,
        opened: Option<Arc<dyn Fn(usize) + Send + Sync>>,
        closed: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Element {
        Element::typed::<super::menubar_live::LiveMenuBar>(super::menubar_live::LiveProps {
            config,
            seed: MenuBarState::default(),
            selected,
            opened,
            closed,
        })
    }
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

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<super::menubar_live::LiveMenuBar>(super::menubar_live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
            selected: self.on_item_selected.clone(),
            opened: self.on_dropdown_opened.clone(),
            closed: self.on_dropdown_closed.clone(),
        })
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
