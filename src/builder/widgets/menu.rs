//! Menu widget builders
//!
//! This module provides builders for creating various types of menus including
//! menubars, context menus, popup menus, and dialog menus with fluent APIs.

use crate::component::Element;
use std::sync::Arc;

/// Type alias for menu action callback to reduce complexity
type MenuActionCallback = Arc<dyn Fn() + Send + Sync>;

/// Type alias for item selection callback to reduce complexity
type ItemSelectionCallback = Arc<dyn Fn(&str) + Send + Sync>;
/// Type alias for dropdown callback to reduce complexity
type DropdownCallback = Arc<dyn Fn(usize) + Send + Sync>;
/// Type alias for close callback to reduce complexity
type CloseCallback = Arc<dyn Fn() + Send + Sync>;

/// Keyboard shortcut for menu items
#[derive(Clone, Debug, PartialEq)]
pub struct MenuShortcut {
    /// Display text for the shortcut (e.g., "Ctrl+S")
    pub display: String,
    /// Key combination that triggers this shortcut
    pub keys: Vec<String>,
}

impl MenuShortcut {
    /// Create a new menu shortcut
    pub fn new(display: impl Into<String>, keys: Vec<impl Into<String>>) -> Self {
        Self {
            display: display.into(),
            keys: keys.into_iter().map(|k| k.into()).collect(),
        }
    }
}

/// Action that can be triggered by a menu item
#[derive(Clone)]
pub struct MenuAction {
    /// Unique identifier for this action
    pub id: String,
    /// Callback function to execute when action is triggered
    pub callback: MenuActionCallback,
}

impl std::fmt::Debug for MenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuAction")
            .field("id", &self.id)
            .field("callback", &"<function>")
            .finish()
    }
}

impl PartialEq for MenuAction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.callback, &other.callback)
    }
}

impl MenuAction {
    /// Create a new menu action
    pub fn new(id: impl Into<String>, callback: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            id: id.into(),
            callback: Arc::new(callback),
        }
    }

    /// Execute the action
    pub fn execute(&self) {
        (self.callback)();
    }
}

/// Type of separator to display after a menu item
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MenuSeparator {
    /// No separator
    #[default]
    None,
    /// Simple line separator
    Line,
    /// Thick line separator
    ThickLine,
    /// Double line separator
    DoubleLine,
    /// Dashed line separator
    Dashed,
    /// Dotted line separator
    Dotted,
    /// Empty space separator
    Space,
}

/// Type of menu item
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MenuItemType {
    /// Regular clickable item
    #[default]
    Action,
    /// Item with submenu
    Submenu,
    /// Checkable item (checkbox)
    Checkbox {
        /// Whether the checkbox is checked
        checked: bool,
    },
    /// Radio button item
    Radio {
        /// Whether this radio button is selected
        selected: bool,
        /// Radio button group name
        group: String,
    },
    /// Separator only (no text)
    Separator,
}

/// A single menu item
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    /// Unique identifier for this item
    pub id: String,
    /// Display text for the item
    pub text: String,
    /// Type of menu item
    pub item_type: MenuItemType,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Whether the item is visible
    pub visible: bool,
    /// Keyboard shortcut for this item
    pub shortcut: Option<MenuShortcut>,
    /// Action to execute when item is selected
    pub action: Option<MenuAction>,
    /// Submenu items (if item_type is Submenu)
    pub submenu: Vec<MenuItem>,
    /// Separator to display after this item
    pub separator: MenuSeparator,
    /// Icon or symbol to display before the text
    pub icon: Option<String>,
    /// Additional description or tooltip text
    pub description: Option<String>,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            item_type: MenuItemType::Action,
            enabled: true,
            visible: true,
            shortcut: None,
            action: None,
            submenu: Vec::new(),
            separator: MenuSeparator::None,
            icon: None,
            description: None,
        }
    }
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            ..Default::default()
        }
    }

    /// Check if this item has a submenu
    pub fn has_submenu(&self) -> bool {
        matches!(self.item_type, MenuItemType::Submenu) && !self.submenu.is_empty()
    }

    /// Check if this item is selectable
    pub fn is_selectable(&self) -> bool {
        self.enabled && self.visible && !matches!(self.item_type, MenuItemType::Separator)
    }

    /// Execute the item's action if it has one
    pub fn execute(&self) {
        if let Some(action) = &self.action {
            action.execute();
        }
    }
}

/// Menu style configuration
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MenuStyle {
    /// Background color
    pub background: Option<String>,
    /// Text color
    pub text_color: Option<String>,
    /// Selected item background color
    pub selected_background: Option<String>,
    /// Selected item text color
    pub selected_text_color: Option<String>,
    /// Disabled item text color
    pub disabled_text_color: Option<String>,
    /// Border style
    pub border: Option<String>,
    /// Padding
    pub padding: Option<String>,
    /// Margin
    pub margin: Option<String>,
}

/// Placement options for popup menus
#[derive(Clone, Debug, PartialEq, Default)]
pub enum PopupPlacement {
    /// Below the trigger element
    Below,
    /// Above the trigger element
    Above,
    /// To the left of the trigger element
    Left,
    /// To the right of the trigger element
    Right,
    /// Automatically choose best placement
    #[default]
    Auto,
}

/// Create a menu item with fluent configuration
pub fn menu_item(id: impl Into<String>, text: impl Into<String>) -> MenuItemBuilder {
    MenuItemBuilder::new(id, text)
}

/// Create an action menu item
pub fn action_item(
    id: impl Into<String>,
    text: impl Into<String>,
    callback: impl Fn() + Send + Sync + 'static,
) -> MenuItem {
    let id_string = id.into();
    MenuItem {
        id: id_string.clone(),
        text: text.into(),
        item_type: MenuItemType::Action,
        action: Some(MenuAction::new(id_string, callback)),
        ..Default::default()
    }
}

/// Create a submenu item
pub fn submenu_item(
    id: impl Into<String>,
    text: impl Into<String>,
    items: Vec<MenuItem>,
) -> MenuItem {
    MenuItem {
        id: id.into(),
        text: text.into(),
        item_type: MenuItemType::Submenu,
        submenu: items,
        ..Default::default()
    }
}

/// Create a checkbox menu item
pub fn checkbox_item(
    id: impl Into<String>,
    text: impl Into<String>,
    checked: bool,
    callback: impl Fn(bool) + Send + Sync + 'static,
) -> MenuItem {
    let id_str = id.into();
    let callback_id = id_str.clone();
    MenuItem {
        id: id_str,
        text: text.into(),
        item_type: MenuItemType::Checkbox { checked },
        action: Some(MenuAction::new(callback_id, move || callback(!checked))),
        ..Default::default()
    }
}

/// Create a radio button menu item
pub fn radio_item(
    id: impl Into<String>,
    text: impl Into<String>,
    selected: bool,
    group: impl Into<String>,
    callback: impl Fn() + Send + Sync + 'static,
) -> MenuItem {
    let id_str = id.into();
    let callback_id = id_str.clone();
    MenuItem {
        id: id_str,
        text: text.into(),
        item_type: MenuItemType::Radio {
            selected,
            group: group.into(),
        },
        action: Some(MenuAction::new(callback_id, callback)),
        ..Default::default()
    }
}

/// Create a separator item
pub fn separator() -> MenuItem {
    MenuItem {
        item_type: MenuItemType::Separator,
        separator: MenuSeparator::Line,
        ..Default::default()
    }
}

/// Create a menubar with fluent configuration
pub fn menubar() -> MenuBarBuilder {
    MenuBarBuilder::new()
}

/// Create a context menu with fluent configuration
pub fn context_menu() -> ContextMenuBuilder {
    ContextMenuBuilder::new()
}

/// Create a popup menu with fluent configuration
pub fn popup_menu() -> PopupMenuBuilder {
    PopupMenuBuilder::new()
}

/// Builder for creating menu items with a fluent API
pub struct MenuItemBuilder {
    item: MenuItem,
}

impl MenuItemBuilder {
    /// Create a new menu item builder
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            item: MenuItem::new(id, text),
        }
    }

    /// Set the item type
    pub fn item_type(mut self, item_type: MenuItemType) -> Self {
        self.item.item_type = item_type;
        self
    }

    /// Set whether the item is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.item.enabled = enabled;
        self
    }

    /// Set whether the item is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.item.visible = visible;
        self
    }

    /// Set the keyboard shortcut
    pub fn shortcut(mut self, shortcut: MenuShortcut) -> Self {
        self.item.shortcut = Some(shortcut);
        self
    }

    /// Set the action callback
    pub fn action(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.item.action = Some(MenuAction::new(&self.item.id, callback));
        self
    }

    /// Set submenu items
    pub fn submenu(mut self, items: Vec<MenuItem>) -> Self {
        self.item.submenu = items;
        self.item.item_type = MenuItemType::Submenu;
        self
    }

    /// Set the separator type
    pub fn separator(mut self, separator: MenuSeparator) -> Self {
        self.item.separator = separator;
        self
    }

    /// Set an icon for the item
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.item.icon = Some(icon.into());
        self
    }

    /// Set a description for the item
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.item.description = Some(description.into());
        self
    }

    /// Build the menu item
    pub fn build(self) -> MenuItem {
        self.item
    }
}

/// Builder for MenuBar components with fluent API
pub struct MenuBarBuilder {
    items: Vec<MenuItem>,
    style: MenuStyle,
    enabled: bool,
    visible: bool,
    title: Option<String>,
    show_shortcuts: bool,
    max_dropdown_height: usize,
    on_item_selected: Option<ItemSelectionCallback>,
    on_dropdown_opened: Option<DropdownCallback>,
    on_dropdown_closed: Option<CloseCallback>,
    class: Option<String>,
}

impl Default for MenuBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBarBuilder {
    /// Create a new MenuBar builder
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            enabled: true,
            visible: true,
            title: None,
            show_shortcuts: true,
            max_dropdown_height: 10,
            on_item_selected: None,
            on_dropdown_opened: None,
            on_dropdown_closed: None,
            class: None,
        }
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple menu items
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Set the menu style
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether the menubar is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether the menubar is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the title text
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set whether to show keyboard shortcuts
    pub fn show_shortcuts(mut self, show: bool) -> Self {
        self.show_shortcuts = show;
        self
    }

    /// Set the maximum dropdown height
    pub fn max_dropdown_height(mut self, height: usize) -> Self {
        self.max_dropdown_height = height;
        self
    }

    /// Set callback for when a menu item is selected
    pub fn on_item_selected(mut self, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(callback));
        self
    }

    /// Set callback for when a dropdown is opened
    pub fn on_dropdown_opened(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_dropdown_opened = Some(Arc::new(callback));
        self
    }

    /// Set callback for when a dropdown is closed
    pub fn on_dropdown_closed(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_dropdown_closed = Some(Arc::new(callback));
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Build the menubar element
    pub fn build(self) -> Element {
        use crate::widgets::menu::MenuBarProps;

        // Convert builder MenuItem to widget MenuItem
        let widget_items: Vec<crate::widgets::menu::MenuItem> = self
            .items
            .clone()
            .into_iter()
            .map(convert_menu_item)
            .collect();

        // Convert builder MenuStyle to widget MenuStyle
        let widget_style = convert_menu_style(self.style.clone());

        // Create MenuBar props from builder configuration
        let props = MenuBarProps {
            items: widget_items,
            style: widget_style,
            enabled: self.enabled,
            visible: self.visible,
            title: self.title,
            show_shortcuts: self.show_shortcuts,
            max_dropdown_height: self.max_dropdown_height,
        };

        let mut element = crate::widgets::menu::MenuBar::element_with_callbacks(
            props,
            self.on_item_selected,
            self.on_dropdown_opened,
            self.on_dropdown_closed,
        );
        element.class = self.class;
        element
    }
}

/// Builder for ContextMenu components with fluent API
pub struct ContextMenuBuilder {
    items: Vec<MenuItem>,
    style: MenuStyle,
    enabled: bool,
    visible: bool,
    trigger_on_right_click: bool,
    auto_close: bool,
    on_item_selected: Option<ItemSelectionCallback>,
    on_opened: Option<CloseCallback>,
    on_closed: Option<CloseCallback>,
    class: Option<String>,
}

impl Default for ContextMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenuBuilder {
    /// Create a new ContextMenu builder
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            enabled: true,
            visible: true,
            trigger_on_right_click: true,
            auto_close: true,
            on_item_selected: None,
            on_opened: None,
            on_closed: None,
            class: None,
        }
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple menu items
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Set the menu style
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether the context menu is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether the context menu is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether to trigger on right click
    pub fn trigger_on_right_click(mut self, trigger: bool) -> Self {
        self.trigger_on_right_click = trigger;
        self
    }

    /// Set whether to auto-close when an item is selected
    pub fn auto_close(mut self, auto_close: bool) -> Self {
        self.auto_close = auto_close;
        self
    }

    /// Set callback for when a menu item is selected
    pub fn on_item_selected(mut self, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(callback));
        self
    }

    /// Set callback for when the context menu is opened
    pub fn on_opened(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_opened = Some(Arc::new(callback));
        self
    }

    /// Set callback for when the context menu is closed
    pub fn on_closed(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_closed = Some(Arc::new(callback));
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Build the context menu element
    pub fn build(self) -> Element {
        use crate::widgets::menu::{ContextMenu, ContextMenuProps};
        let element = ContextMenu::element_with_callbacks(
            ContextMenuProps {
                items: self.items.into_iter().map(convert_menu_item).collect(),
                style: convert_menu_style(self.style),
                enabled: self.enabled,
                show_on_right_click: self.trigger_on_right_click,
                ..Default::default()
            },
            self.visible,
            self.auto_close,
            self.on_item_selected,
            self.on_opened,
            self.on_closed,
        );
        if let Some(class) = self.class {
            element.with_class(class)
        } else {
            element
        }
    }
}

/// Builder for PopupMenu components with fluent API
pub struct PopupMenuBuilder {
    items: Vec<MenuItem>,
    style: MenuStyle,
    enabled: bool,
    visible: bool,
    placement: PopupPlacement,
    auto_close: bool,
    close_on_outside_click: bool,
    on_item_selected: Option<ItemSelectionCallback>,
    on_opened: Option<CloseCallback>,
    on_closed: Option<CloseCallback>,
    class: Option<String>,
}

impl Default for PopupMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupMenuBuilder {
    /// Create a new PopupMenu builder
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            style: MenuStyle::default(),
            enabled: true,
            visible: true,
            placement: PopupPlacement::Auto,
            auto_close: true,
            close_on_outside_click: true,
            on_item_selected: None,
            on_opened: None,
            on_closed: None,
            class: None,
        }
    }

    /// Add a menu item
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple menu items
    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Set the menu style
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }

    /// Set whether the popup menu is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether the popup menu is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the placement of the popup menu
    pub fn placement(mut self, placement: PopupPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set whether to auto-close when an item is selected
    pub fn auto_close(mut self, auto_close: bool) -> Self {
        self.auto_close = auto_close;
        self
    }

    /// Set whether to close when clicking outside
    pub fn close_on_outside_click(mut self, close: bool) -> Self {
        self.close_on_outside_click = close;
        self
    }

    /// Set callback for when a menu item is selected
    pub fn on_item_selected(mut self, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_item_selected = Some(Arc::new(callback));
        self
    }

    /// Set callback for when the popup menu is opened
    pub fn on_opened(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_opened = Some(Arc::new(callback));
        self
    }

    /// Set callback for when the popup menu is closed
    pub fn on_closed(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_closed = Some(Arc::new(callback));
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Build the popup menu element
    pub fn build(self) -> Element {
        use crate::widgets::menu::{PopupMenu, PopupMenuProps, RelativePlacement};
        let side = match self.placement {
            PopupPlacement::Above => RelativePlacement::Above,
            PopupPlacement::Below => RelativePlacement::Below,
            PopupPlacement::Left => RelativePlacement::Left,
            PopupPlacement::Right => RelativePlacement::Right,
            PopupPlacement::Auto => RelativePlacement::Auto,
        };
        let element = PopupMenu::element_with_callbacks(
            PopupMenuProps {
                items: self.items.into_iter().map(convert_menu_item).collect(),
                style: convert_menu_style(self.style),
                enabled: self.enabled,
                visible: self.visible,
                auto_close: self.auto_close,
                close_on_outside_click: self.close_on_outside_click,
                ..Default::default()
            },
            Some(side),
            self.on_item_selected,
            self.on_opened,
            self.on_closed,
        );
        if let Some(class) = self.class {
            element.with_class(class)
        } else {
            element
        }
    }
}

/// Convert builder MenuItem to widget MenuItem
fn convert_menu_item(builder_item: MenuItem) -> crate::widgets::menu::MenuItem {
    use crate::widgets::menu::MenuItem as WidgetMenuItem;

    let widget_item_type = match builder_item.item_type {
        crate::builder::widgets::menu::MenuItemType::Action => {
            crate::widgets::menu::MenuItemType::Action
        }
        crate::builder::widgets::menu::MenuItemType::Submenu => {
            crate::widgets::menu::MenuItemType::Submenu
        }
        crate::builder::widgets::menu::MenuItemType::Separator => {
            crate::widgets::menu::MenuItemType::Separator
        }
        crate::builder::widgets::menu::MenuItemType::Checkbox { checked } => {
            crate::widgets::menu::MenuItemType::Checkbox { checked }
        }
        crate::builder::widgets::menu::MenuItemType::Radio { selected, group } => {
            crate::widgets::menu::MenuItemType::Radio { selected, group }
        }
    };

    let widget_separator = match builder_item.separator {
        crate::builder::widgets::menu::MenuSeparator::None => {
            crate::widgets::menu::MenuSeparator::None
        }
        crate::builder::widgets::menu::MenuSeparator::Line => {
            crate::widgets::menu::MenuSeparator::Line
        }
        crate::builder::widgets::menu::MenuSeparator::ThickLine => {
            crate::widgets::menu::MenuSeparator::ThickLine
        }
        crate::builder::widgets::menu::MenuSeparator::DoubleLine => {
            crate::widgets::menu::MenuSeparator::DoubleLine
        }
        crate::builder::widgets::menu::MenuSeparator::Dashed => {
            crate::widgets::menu::MenuSeparator::Dashed
        }
        crate::builder::widgets::menu::MenuSeparator::Dotted => {
            crate::widgets::menu::MenuSeparator::Dotted
        }
        crate::builder::widgets::menu::MenuSeparator::Space => {
            crate::widgets::menu::MenuSeparator::Space
        }
    };

    let widget_action = builder_item
        .action
        .map(|action| crate::widgets::menu::MenuAction {
            id: action.id,
            callback: action.callback,
        });

    let widget_submenu: Vec<crate::widgets::menu::MenuItem> = builder_item
        .submenu
        .into_iter()
        .map(convert_menu_item)
        .collect();

    WidgetMenuItem {
        id: builder_item.id,
        text: builder_item.text,
        item_type: widget_item_type,
        enabled: builder_item.enabled,
        visible: builder_item.visible,
        shortcut: builder_item
            .shortcut
            .map(|s| crate::widgets::menu::MenuShortcut {
                display: s.display,
                keys: s.keys,
            }),
        action: widget_action,
        submenu: widget_submenu,
        separator: widget_separator,
        icon: builder_item.icon,
        description: builder_item.description,
    }
}

fn menu_color(value: &str, prefix: &str) -> String {
    if crate::layout::colors::parse_color_token(value).is_some() {
        format!("{prefix}-{value}")
    } else {
        value.to_owned()
    }
}

/// Convert builder colors and utility classes to the shared menu style.
fn convert_menu_style(builder_style: MenuStyle) -> crate::widgets::menu::MenuStyle {
    let base_classes = format!(
        "{} {} {} {}",
        menu_color(builder_style.background.as_deref().unwrap_or("white"), "bg"),
        menu_color(
            builder_style.text_color.as_deref().unwrap_or("black"),
            "text"
        ),
        builder_style.padding.as_deref().unwrap_or(""),
        builder_style.margin.as_deref().unwrap_or("")
    );
    let selected_classes = format!(
        "{} {}",
        menu_color(
            builder_style
                .selected_background
                .as_deref()
                .unwrap_or("blue-500"),
            "bg"
        ),
        menu_color(
            builder_style
                .selected_text_color
                .as_deref()
                .unwrap_or("white"),
            "text"
        )
    );
    crate::widgets::menu::MenuStyle {
        base_classes,
        focused_classes: selected_classes.clone(),
        selected_classes,
        disabled_classes: menu_color(
            builder_style
                .disabled_text_color
                .as_deref()
                .unwrap_or("gray-400"),
            "text",
        ),
        border_classes: builder_style
            .border
            .unwrap_or_else(|| "border border-gray-300".into()),
        ..Default::default()
    }
}
