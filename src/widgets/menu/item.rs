use std::sync::Arc;

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
    pub callback: Arc<dyn Fn() + Send + Sync>,
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
        self.id == other.id
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
#[derive(Clone, Debug, PartialEq)]
pub enum MenuSeparator {
    /// No separator
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

impl Default for MenuSeparator {
    fn default() -> Self {
        Self::None
    }
}

/// Type of menu item
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemType {
    /// Regular clickable item
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

impl Default for MenuItemType {
    fn default() -> Self {
        Self::Action
    }
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

    /// Create an action menu item with callback
    pub fn action(
        id: impl Into<String>,
        text: impl Into<String>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let id_string = id.into();
        Self {
            id: id_string.clone(),
            text: text.into(),
            item_type: MenuItemType::Action,
            action: Some(MenuAction::new(id_string, callback)),
            ..Default::default()
        }
    }

    /// Create a submenu item
    pub fn submenu(id: impl Into<String>, text: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            item_type: MenuItemType::Submenu,
            submenu: items,
            ..Default::default()
        }
    }

    /// Create a checkbox menu item
    pub fn checkbox(
        id: impl Into<String>,
        text: impl Into<String>,
        checked: bool,
        callback: impl Fn(bool) + Send + Sync + 'static,
    ) -> Self {
        let id_str = id.into();
        let callback_id = id_str.clone();
        Self {
            id: id_str,
            text: text.into(),
            item_type: MenuItemType::Checkbox { checked },
            action: Some(MenuAction::new(callback_id, move || callback(!checked))),
            ..Default::default()
        }
    }

    /// Create a radio button menu item
    pub fn radio(
        id: impl Into<String>,
        text: impl Into<String>,
        selected: bool,
        group: impl Into<String>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let id_str = id.into();
        let callback_id = id_str.clone();
        Self {
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
    pub fn separator() -> Self {
        Self {
            item_type: MenuItemType::Separator,
            separator: MenuSeparator::Line,
            ..Default::default()
        }
    }

    /// Set whether the item is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether the item is visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the keyboard shortcut
    pub fn shortcut(mut self, shortcut: MenuShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set the separator type
    pub fn separator_type(mut self, separator: MenuSeparator) -> Self {
        self.separator = separator;
        self
    }

    /// Set an icon for the item
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set a description for the item
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
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

/// Builder for creating menu items with a fluent API
#[allow(dead_code)]
pub struct MenuItemBuilder {
    item: MenuItem,
}

#[allow(dead_code)]
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
