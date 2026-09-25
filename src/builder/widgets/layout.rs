//! Layout widget builders
//!
//! This module provides builders for layout components including scroll views,
//! stack layouts, tabs, and other container widgets.

use super::super::specialized::{ScrollViewBuilder, StackBuilder};
use crate::component::Element;

/// Create a Scroll View builder
///
/// Returns a `ScrollViewBuilder` for creating scrollable content areas.
pub fn scroll_view() -> ScrollViewBuilder {
    ScrollViewBuilder::new()
}

/// Create a Stack layout builder
///
/// Returns a `StackBuilder` for creating stack layout containers.
pub fn stack() -> StackBuilder {
    StackBuilder::new()
}

/// Create a Tabs container builder
///
/// Returns a `TabsBuilder` for creating tabbed interfaces with multiple
/// content panels and navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::tabs;
/// use reactive_tui::component::Element;
///
/// let tabs = tabs()
///     .tab("Dashboard", Element::text("Dashboard content"))
///     .tab("Settings", Element::text("Settings panel"))
///     .build();
/// ```
pub fn tabs() -> TabsBuilder {
    TabsBuilder::new()
}

/// Builder for Tabs container components
///
/// Provides a fluent API for creating tabbed interfaces with multiple
/// content panels and navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::tabs;
/// use reactive_tui::component::Element;
///
/// let tabs = tabs()
///     .tab("Dashboard", Element::text("Dashboard content"))
///     .tab("Settings", Element::text("Settings panel"))
///     .active(0)
///     .build();
/// ```
pub struct TabsBuilder {
    tabs: Vec<(String, Element)>, // (title, content)
    active_tab: usize,
    closable: bool,
    class: Option<String>,
}

impl TabsBuilder {
    /// Create a new TabsBuilder with default values
    fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            closable: false,
            class: None,
        }
    }

    /// Add a tab with title and content
    ///
    /// # Arguments
    /// * `title` - The tab title displayed in the tab bar
    /// * `content` - The element to display when this tab is active
    pub fn tab(mut self, title: &str, content: Element) -> Self {
        self.tabs.push((title.to_string(), content));
        self
    }

    /// Set the active tab by index
    ///
    /// # Arguments
    /// * `index` - Zero-based index of the tab to make active
    pub fn active(mut self, index: usize) -> Self {
        self.active_tab = index;
        self
    }

    /// Enable or disable closable tabs
    ///
    /// # Arguments
    /// * `closable` - Whether tabs can be closed by the user
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the tabs container
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Tabs element
    ///
    /// Creates a tabs container element with the configured tabs and properties.
    ///
    /// # Returns
    /// An `Element` representing the tabs container
    pub fn build(self) -> Element {
        let props = crate::widgets::layout::TabsProps {
            tabs: self
                .tabs
                .into_iter()
                .map(|(title, content)| crate::widgets::layout::Tab::new(title, content))
                .collect(),
            active_tab: self.active_tab,
            closable: self.closable,
            ..Default::default()
        };
        let mut element = Element::typed::<crate::widgets::layout::Tabs>(props);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<TabsBuilder> for Element {
    fn from(builder: TabsBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (ScrollViewBuilder, StackBuilder)
// are now provided by the placeholders module
