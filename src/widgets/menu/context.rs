use crate::component::{Component, Element, Props};
use crate::widgets::menu::{MenuItem, MenuStyle, MenuTheme, PopupMenuState};
use std::any::Any;
use std::sync::Arc;

/// Properties for ContextMenu component
#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuProps {
    /// Menu items to display in the context menu
    pub items: Vec<MenuItem>,
    /// Style configuration
    pub style: MenuStyle,
    /// Whether the context menu is enabled
    pub enabled: bool,
    /// Whether to show the context menu on right-click
    pub show_on_right_click: bool,
    /// Whether to show the context menu on long press (for touch interfaces)
    pub show_on_long_press: bool,
    /// Duration for long press detection (in milliseconds)
    pub long_press_duration: u64,
    /// Whether to auto-close when clicking outside
    pub close_on_outside_click: bool,
    /// Maximum number of visible items (for scrolling)
    pub max_visible_items: usize,
    /// Fixed width for the context menu
    pub width: Option<u16>,
    /// Whether to show a border around the menu
    pub show_border: bool,
    /// Whether to show a shadow
    pub show_shadow: bool,
    /// Custom trigger areas (if not using global right-click)
    pub trigger_areas: Vec<(u16, u16, u16, u16)>, // x, y, width, height
}

impl Default for ContextMenuProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            enabled: true,
            show_on_right_click: true,
            show_on_long_press: false,
            long_press_duration: 500,
            close_on_outside_click: true,
            max_visible_items: 10,
            width: None,
            show_border: true,
            show_shadow: true,
            trigger_areas: Vec::new(),
        }
    }
}

impl Props for ContextMenuProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for ContextMenu component
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContextMenuState {
    /// Whether the context menu is currently visible
    pub is_visible: bool,
    /// Position where the context menu should appear
    pub position: Option<(u16, u16)>,
    /// Internal popup menu state
    pub popup_state: PopupMenuState,
    /// Timestamp of mouse press for long press detection
    pub press_start_time: Option<std::time::Instant>,
    /// Whether a long press is in progress
    pub long_press_active: bool,
    /// Last mouse position for tracking movement during long press
    pub last_mouse_position: Option<(u16, u16)>,
}

impl ContextMenuState {
    /// Create a new context menu state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the context menu at the specified position
    pub fn show_at(&mut self, x: u16, y: u16) {
        self.is_visible = true;
        self.position = Some((x, y));
        self.popup_state.show();
        self.popup_state.popup_area = Some((x, y, 0, 0)); // Width/height will be calculated during render
    }

    /// Hide the context menu
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.position = None;
        self.popup_state.hide();
        self.press_start_time = None;
        self.long_press_active = false;
        self.last_mouse_position = None;
    }

    /// Check if a point is within any of the trigger areas
    pub fn is_in_trigger_area(
        &self,
        x: u16,
        y: u16,
        trigger_areas: &[(u16, u16, u16, u16)],
    ) -> bool {
        if trigger_areas.is_empty() {
            return true; // Global context menu
        }

        trigger_areas
            .iter()
            .any(|rect| super::state::contains(*rect, (x, y)))
    }

    /// Start long press detection
    pub fn start_long_press(&mut self, x: u16, y: u16) {
        self.press_start_time = Some(std::time::Instant::now());
        self.last_mouse_position = Some((x, y));
        self.long_press_active = false;
    }

    /// Update long press detection
    pub fn update_long_press(&mut self, x: u16, y: u16, duration_ms: u64) -> bool {
        if let Some(start_time) = self.press_start_time {
            if let Some((last_x, last_y)) = self.last_mouse_position {
                // Check if mouse moved too much (cancel long press)
                let dx = x.abs_diff(last_x);
                let dy = y.abs_diff(last_y);
                if dx > 5 || dy > 5 {
                    self.cancel_long_press();
                    return false;
                }
            }

            // Check if enough time has passed
            if start_time.elapsed().as_millis() >= duration_ms as u128 && !self.long_press_active {
                self.long_press_active = true;
                return true;
            }
        }
        false
    }

    /// Cancel long press detection
    pub fn cancel_long_press(&mut self) {
        self.press_start_time = None;
        self.long_press_active = false;
        self.last_mouse_position = None;
    }
}

/// ContextMenu component for right-click and long-press menus
#[derive(Default)]
pub struct ContextMenu {
    on_item_selected: Option<super::TextCallback>,
    on_show: Option<Arc<dyn Fn(u16, u16) + Send + Sync>>,
    on_hide: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl ContextMenu {
    pub(crate) fn element_with_callbacks(
        config: ContextMenuProps,
        visible: bool,
        auto_close: bool,
        selected: Option<super::TextCallback>,
        shown: Option<Arc<dyn Fn() + Send + Sync>>,
        hidden: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Element {
        Element::typed::<super::context_live::LiveContext>(super::context_live::LiveProps {
            config,
            seed: ContextMenuState {
                is_visible: visible,
                ..Default::default()
            },
            auto_close,
            selected,
            shown: shown.map(|callback| {
                Arc::new(move |_, _| callback()) as Arc<dyn Fn(u16, u16) + Send + Sync>
            }),
            hidden,
        })
    }

    /// Set callback for when a menu item is selected
    pub fn with_on_item_selected(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        let callback: Arc<dyn Fn(&str) + Send + Sync> = Arc::new(f);
        self.on_item_selected = Some(Arc::clone(&callback));
        self
    }

    /// Set callback for when the context menu is shown
    pub fn with_on_show(mut self, f: impl Fn(u16, u16) + Send + Sync + 'static) -> Self {
        self.on_show = Some(Arc::new(f));
        self
    }

    /// Set callback for when the context menu is hidden
    pub fn with_on_hide(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(f);
        self.on_hide = Some(Arc::clone(&callback));
        self
    }
}

impl Component for ContextMenu {
    type Props = ContextMenuProps;
    type State = ContextMenuState;

    fn new(_props: Self::Props) -> Self {
        Self {
            on_item_selected: None,
            on_show: None,
            on_hide: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, _: &mut Self::State) -> bool {
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<super::context_live::LiveContext>(super::context_live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
            auto_close: true,
            selected: self.on_item_selected.clone(),
            shown: self.on_show.clone(),
            hidden: self.on_hide.clone(),
        })
    }
}

/// Builder for creating ContextMenu components with a fluent API
pub struct ContextMenuBuilder {
    props: ContextMenuProps,
}

impl ContextMenuBuilder {
    /// Create a new context menu builder
    pub fn new() -> Self {
        Self {
            props: ContextMenuProps::default(),
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

    /// Set whether the context menu is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.props.enabled = enabled;
        self
    }

    /// Set whether to show on right-click
    pub fn show_on_right_click(mut self, show: bool) -> Self {
        self.props.show_on_right_click = show;
        self
    }

    /// Set whether to show on long press
    pub fn show_on_long_press(mut self, show: bool) -> Self {
        self.props.show_on_long_press = show;
        self
    }

    /// Set long press duration
    pub fn long_press_duration(mut self, duration: u64) -> Self {
        self.props.long_press_duration = duration;
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

    /// Add a trigger area
    pub fn trigger_area(mut self, x: u16, y: u16, width: u16, height: u16) -> Self {
        self.props.trigger_areas.push((x, y, width, height));
        self
    }

    /// Build the context menu props
    pub fn build(self) -> ContextMenuProps {
        self.props
    }
}

impl Default for ContextMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}
