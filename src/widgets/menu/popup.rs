use crate::component::{Component, Element, Props};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme};
use std::any::Any;
use std::sync::Arc;

/// Placement options for popup menus
#[derive(Clone, Debug, PartialEq, Default)]
pub enum PopupPlacement {
    /// Place popup at cursor position
    #[default]
    Cursor,
    /// Place popup at specific coordinates
    Position {
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
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
        height: u16,
    },
    /// Place popup below the specified area
    Below {
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
        /// Area width
        width: u16,
    },
    /// Place popup above the specified area
    Above {
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
        /// Area width
        width: u16,
    },
    /// Place popup to the right of the specified area
    Right {
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
        /// Area height
        height: u16,
    },
    /// Place popup to the left of the specified area
    Left {
        /// X coordinate
        x: u16,
        /// Y coordinate
        y: u16,
        /// Area height
        height: u16,
    },
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
#[derive(Clone, Debug, Default, PartialEq)]
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
        self.selected_index = super::state::step(self.selected_index, item_count, true);
    }

    /// Select the previous menu item
    pub fn select_previous(&mut self, item_count: usize) {
        self.selected_index = super::state::step(self.selected_index, item_count, false);
    }

    /// Update scroll offset to keep selected item visible
    pub fn update_scroll(&mut self, max_visible: usize) {
        super::state::keep_visible(self.selected_index, &mut self.scroll_offset, max_visible);
    }

    /// Check if a point is inside the popup area
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        self.popup_area
            .is_some_and(|rect| super::state::contains(rect, (x, y)))
    }
}

/// PopupMenu component for context menus and dropdowns
#[derive(Default)]
pub struct PopupMenu {
    state: PopupMenuState,
    on_item_selected: Option<super::TextCallback>,
    on_show: Option<Arc<dyn Fn() + Send + Sync>>,
    on_hide: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PopupMenu {
    pub(crate) fn element_with_callbacks(
        config: PopupMenuProps,
        relative_placement: Option<super::popup_live::RelativePlacement>,
        selected: Option<super::TextCallback>,
        shown: Option<Arc<dyn Fn() + Send + Sync>>,
        hidden: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Element {
        Element::typed::<super::popup_live::LivePopup>(super::popup_live::LiveProps {
            config,
            relative_placement,
            seed: PopupMenuState::default(),
            selected,
            shown,
            hidden,
        })
    }

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

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<super::popup_live::LivePopup>(super::popup_live::LiveProps {
            config: props.clone(),
            relative_placement: None,
            seed: state.clone(),
            selected: self.on_item_selected.clone(),
            shown: self.on_show.clone(),
            hidden: self.on_hide.clone(),
        })
    }
}

/// Builder for creating PopupMenu components with a fluent API
pub struct PopupMenuBuilder {
    props: PopupMenuProps,
}

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
