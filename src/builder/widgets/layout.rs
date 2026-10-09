//! Layout widget builders
//!
//! This module provides builders for layout components including scroll views,
//! stack layouts, tabs, and other container widgets.

use super::super::specialized::{ScrollViewBuilder, StackBuilder};
use crate::component::{same_callback, Element};
use std::sync::Arc;

/// The callback a tab set built through its builder calls with the new
/// active tab's index.
type ChangeCallback = Arc<dyn Fn(usize) + Send + Sync>;

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
    aria_label: Option<String>,
    on_change: Option<ChangeCallback>,
}

impl TabsBuilder {
    /// Create a new TabsBuilder with default values
    fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            closable: false,
            class: None,
            aria_label: None,
            on_change: None,
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

    /// The name the screen reader gives the tab list (NAV-004).
    pub fn aria_label(mut self, label: &str) -> Self {
        self.aria_label = Some(label.to_string());
        self
    }

    /// Set the callback called with the new active tab's index each time
    /// the user selects another tab, as `Tabs::with_on_change` is (CMP-009).
    pub fn on_change(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
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
            aria_label: self.aria_label,
            ..Default::default()
        };
        let mut element = Element::typed::<ConfiguredTabs>(ConfiguredTabsProps {
            tabs: props,
            on_change: self.on_change,
        });

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

/// The props of a tab set built through its builder: the tab set's own
/// props and the builder's callback, which equality leaves out so that a
/// rebuild changing only the callback keeps the mounted tab set (CMP-008).
#[derive(Clone)]
struct ConfiguredTabsProps {
    tabs: crate::widgets::layout::TabsProps,
    on_change: Option<ChangeCallback>,
}

impl PartialEq for ConfiguredTabsProps {
    fn eq(&self, other: &Self) -> bool {
        self.tabs == other.tabs
    }
}

impl crate::component::Props for ConfiguredTabsProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A tab set built through its builder: the tab set itself, with the
/// builder's callback handed to it (CMP-009).
struct ConfiguredTabs(crate::widgets::layout::Tabs);

impl crate::component::Component for ConfiguredTabs {
    type Props = ConfiguredTabsProps;
    type State = crate::widgets::layout::TabsState;

    fn new(props: Self::Props) -> Self {
        let mut inner = crate::widgets::layout::Tabs::new(props.tabs);
        inner.set_on_change(props.on_change);
        Self(inner)
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        self.0.initial_state(&props.tabs)
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.update(&props.tabs, state)
    }
    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.tabs, state)
    }
    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.0.layout(bounds, &mut props.tabs, state)
    }
    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> crate::event::router::EventResult {
        // The callback a rebuild adopted acts from this event on (CMP-008).
        self.0.set_on_change(props.on_change.clone());
        self.0.handle_event(event, &mut props.tabs, state)
    }
}

impl From<TabsBuilder> for Element {
    fn from(builder: TabsBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (ScrollViewBuilder, StackBuilder)
// are now provided by the placeholders module
