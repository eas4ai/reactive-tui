use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, LayoutInfo, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{FocusEventKind, KeyCode, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Tab orientation options
#[derive(Clone, Debug, PartialEq)]
pub enum TabOrientation {
    /// Tabs arranged horizontally
    Horizontal,
    /// Tabs arranged vertically
    Vertical,
}

/// Tab visual style variants
#[derive(Clone, Debug, PartialEq)]
pub enum TabVariant {
    /// Underline/border style
    Line,
    /// Box/card style with borders
    Enclosed,
    /// Subtle background style
    Soft,
    /// Filled background style
    Solid,
    /// No decoration
    Unstyled,
}

/// Tab size variants
#[derive(Clone, Debug, PartialEq)]
pub enum TabSize {
    /// Small tab size
    Small,
    /// Medium tab size (default)
    Medium,
    /// Large tab size
    Large,
}

/// Position of tabs relative to content
#[derive(Clone, Debug, PartialEq)]
pub enum TabPosition {
    /// Tabs positioned at the top
    Top,
    /// Tabs positioned at the bottom
    Bottom,
    /// Tabs positioned on the left side
    Left,
    /// Tabs positioned on the right side
    Right,
}

/// How tabs are activated via keyboard
#[derive(Clone, Debug, PartialEq)]
pub enum TabKeyboardActivation {
    /// Activate tab immediately when focused
    Automatic,
    /// Activate tab only on Enter/Space key press
    Manual,
}

/// Tab badge visual variants
#[derive(Clone, Debug, PartialEq)]
pub enum TabBadgeVariant {
    /// Default badge style
    Default,
    /// Success badge style (green)
    Success,
    /// Warning badge style (yellow)
    Warning,
    /// Error badge style (red)
    Error,
    /// Info badge style (blue)
    Info,
}

/// Badge displayed on tabs for notifications or status
#[derive(Clone, Debug, PartialEq)]
pub struct TabBadge {
    /// Text content of the badge
    pub text: String,
    /// Visual style variant of the badge
    pub variant: TabBadgeVariant,
}

impl TabBadge {
    /// Create a new tab badge
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: TabBadgeVariant::Default,
        }
    }

    /// Set the badge variant
    pub fn with_variant(mut self, variant: TabBadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

/// Individual tab definition
#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    /// Display label for the tab
    pub label: String,
    /// Content element to display when active
    pub content: Element,
    /// Whether the tab is disabled
    pub disabled: bool,
    /// Whether the tab can be closed
    pub closable: bool,
    /// Optional icon identifier
    pub icon: Option<String>,
    /// Optional badge for notifications
    pub badge: Option<TabBadge>,
    /// Optional tooltip text
    pub tooltip: Option<String>,
}

impl Tab {
    /// Create a new tab with label and content
    pub fn new(label: impl Into<String>, content: Element) -> Self {
        Self {
            label: label.into(),
            content,
            disabled: false,
            closable: false,
            icon: None,
            badge: None,
            tooltip: None,
        }
    }

    /// Set whether the tab is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the tab can be closed
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Add an icon to the tab
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Add a badge to the tab
    pub fn with_badge(mut self, badge: TabBadge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Add a tooltip to the tab
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

/// Builder for creating Tabs components with a fluent API
#[derive(Clone, Debug)]
pub struct TabsBuilder {
    tabs: Vec<Tab>,
    active_tab: usize,
    orientation: TabOrientation,
    variant: TabVariant,
    size: TabSize,
    position: TabPosition,
    closable: bool,
    disabled: bool,
    lazy_loading: bool,
    keyboard_activation: TabKeyboardActivation,
}

impl TabsBuilder {
    /// Create a new TabsBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tab
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Add multiple tabs
    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    /// Add a tab from label and content
    pub fn add_tab(mut self, label: impl Into<String>, content: Element) -> Self {
        self.tabs.push(Tab::new(label, content));
        self
    }

    /// Set the active tab index
    pub fn active_tab(mut self, index: usize) -> Self {
        self.active_tab = index;
        self
    }

    /// Set the tab orientation
    pub fn orientation(mut self, orientation: TabOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the visual variant
    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the tab size
    pub fn size(mut self, size: TabSize) -> Self {
        self.size = size;
        self
    }

    /// Set the tab position
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Set whether tabs are closable
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set whether tabs are disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether to use lazy loading
    pub fn lazy_loading(mut self, lazy: bool) -> Self {
        self.lazy_loading = lazy;
        self
    }

    /// Set keyboard activation behavior
    pub fn keyboard_activation(mut self, activation: TabKeyboardActivation) -> Self {
        self.keyboard_activation = activation;
        self
    }

    /// Build the TabsProps
    pub fn build(self) -> TabsProps {
        TabsProps {
            tabs: self.tabs,
            active_tab: self.active_tab,
            orientation: self.orientation,
            variant: self.variant,
            size: self.size,
            position: self.position,
            closable: self.closable,
            disabled: self.disabled,
            lazy_loading: self.lazy_loading,
            keyboard_activation: self.keyboard_activation,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::typed::<Tabs>(self.build())
    }
}

impl Default for TabsBuilder {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            orientation: TabOrientation::Horizontal,
            variant: TabVariant::Line,
            size: TabSize::Medium,
            position: TabPosition::Top,
            closable: false,
            disabled: false,
            lazy_loading: true,
            keyboard_activation: TabKeyboardActivation::Automatic,
        }
    }
}

/// Properties for Tabs component
#[derive(Clone, Debug, PartialEq)]
pub struct TabsProps {
    /// List of tab definitions
    pub tabs: Vec<Tab>,
    /// Index of currently active tab
    pub active_tab: usize,
    /// Orientation of the tab bar
    pub orientation: TabOrientation,
    /// Visual variant of the tabs
    pub variant: TabVariant,
    /// Size of the tabs
    pub size: TabSize,
    /// Position of the tab bar
    pub position: TabPosition,
    /// Whether tabs can be closed
    pub closable: bool,
    /// Whether tabs are disabled
    pub disabled: bool,
    /// Whether to only render active tab content
    pub lazy_loading: bool,
    /// Keyboard activation behavior
    pub keyboard_activation: TabKeyboardActivation,
}

impl Default for TabsProps {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            orientation: TabOrientation::Horizontal,
            variant: TabVariant::Line,
            size: TabSize::Medium,
            position: TabPosition::Top,
            closable: false,
            disabled: false,
            lazy_loading: true,
            keyboard_activation: TabKeyboardActivation::Automatic,
        }
    }
}

impl Props for TabsProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Tabs component
#[derive(Clone, Debug, Default)]
pub struct TabsState {
    /// Index of the currently focused tab
    pub focused_tab: Option<usize>,
    /// Index of the tab currently being hovered over
    pub hover_tab: Option<usize>,
    /// Whether the tabs component has focus
    pub is_focused: bool,
}

#[derive(Clone, Copy, Default)]
struct TabTarget {
    header: Option<LayoutInfo>,
    close: Option<LayoutInfo>,
}

/// Tabs component with full keyboard and mouse support
pub struct Tabs {
    authored: TabsProps,
    live: TabsProps,
    identities: Vec<TabIdentity>,
    closed: HashSet<TabIdentity>,
    received_event: bool,
    viewport: Option<LayoutInfo>,
    targets: Arc<Mutex<Vec<TabTarget>>>,
    on_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_close: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TabIdentity {
    Key(String),
    Position(usize),
}

fn identities(props: &TabsProps) -> Vec<TabIdentity> {
    props
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            tab.content
                .key
                .clone()
                .map_or(TabIdentity::Position(i), TabIdentity::Key)
        })
        .collect()
}

impl Tabs {
    fn reconcile(&mut self, props: &TabsProps, state: &mut TabsState) {
        let focused = state
            .focused_tab
            .and_then(|i| self.identities.get(i))
            .cloned();
        let selected = Self::active(&self.live)
            .and_then(|i| self.identities.get(i))
            .cloned();
        let incoming = identities(props);
        self.closed.retain(|id| incoming.contains(id));
        let authored_selection_changed = props.active_tab != self.authored.active_tab;
        let selected = if authored_selection_changed {
            let selected = incoming.get(props.active_tab).cloned();
            if let Some(id) = &selected {
                self.closed.remove(id);
            }
            selected
        } else {
            selected
        };
        self.live = props.clone();
        self.live.tabs = props
            .tabs
            .iter()
            .zip(&incoming)
            .filter(|(_, id)| !self.closed.contains(id))
            .map(|(tab, _)| tab.clone())
            .collect();
        self.identities = incoming
            .into_iter()
            .filter(|id| !self.closed.contains(id))
            .collect();
        self.live.active_tab = selected
            .and_then(|id| self.identities.iter().position(|i| *i == id))
            .unwrap_or(0);
        state.focused_tab = focused
            .and_then(|id| self.identities.iter().position(|i| *i == id))
            .filter(|&i| !self.live.tabs[i].disabled)
            .or_else(|| Self::active(&self.live));
        self.authored = props.clone();
    }
    /// Set the callback for an active tab change.
    pub fn with_on_change(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Request that the parent close a tab. The parent owns tab removal.
    pub fn with_on_close(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }

    fn active(props: &TabsProps) -> Option<usize> {
        props
            .tabs
            .get(props.active_tab)
            .filter(|t| !t.disabled)
            .map(|_| props.active_tab)
            .or_else(|| props.tabs.iter().position(|t| !t.disabled))
    }

    fn find_next_enabled_tab(
        &self,
        props: &TabsProps,
        current: usize,
        direction: i32,
    ) -> Option<usize> {
        let len = props.tabs.len();
        if len == 0 {
            return None;
        }
        let current = current.min(len - 1);
        (1..=len)
            .map(|offset| {
                if direction > 0 {
                    (current + offset) % len
                } else {
                    (current + len - offset) % len
                }
            })
            .find(|&i| !props.tabs[i].disabled)
    }

    fn activate(&self, index: usize, props: &mut TabsProps, state: &mut TabsState) {
        if props.tabs.get(index).is_none_or(|t| t.disabled) {
            return;
        }
        state.focused_tab = Some(index);
        let changed = Self::active(props) != Some(index);
        props.active_tab = index;
        if changed {
            if let Some(callback) = &self.on_change {
                callback(index);
            }
        }
    }

    fn close(&mut self, index: usize, props: &mut TabsProps, state: &mut TabsState) {
        if props
            .tabs
            .get(index)
            .is_none_or(|tab| tab.disabled || !(tab.closable || props.closable))
        {
            return;
        }
        if let Some(callback) = &self.on_close {
            callback(index);
            return;
        }
        // Without a parent close callback, the built-in control owns removal.
        let active = Self::active(props);
        if index < self.identities.len() {
            self.closed.insert(self.identities.remove(index));
        }
        props.tabs.remove(index);
        props.active_tab = active.map_or(0, |active| {
            if index < active {
                active - 1
            } else {
                active.min(props.tabs.len().saturating_sub(1))
            }
        });
        state.focused_tab = Self::active(props);
        state.hover_tab = None;
        if active == Some(index) {
            if let (Some(next), Some(callback)) = (Self::active(props), &self.on_change) {
                callback(next);
            }
        }
    }

    fn render_tab_header(
        &self,
        tab: &Tab,
        index: usize,
        props: &TabsProps,
        state: &TabsState,
    ) -> String {
        let focus = if state.is_focused && state.focused_tab == Some(index) {
            "▶"
        } else if state.hover_tab == Some(index) {
            "→"
        } else {
            " "
        };
        let mut label = String::from(focus);
        if let Some(icon) = &tab.icon {
            label.push_str(icon);
            label.push(' ');
        }
        label.push_str(&tab.label);
        if let Some(badge) = &tab.badge {
            let mark = match badge.variant {
                TabBadgeVariant::Default => "●",
                TabBadgeVariant::Success => "✓",
                TabBadgeVariant::Warning => "⚠",
                TabBadgeVariant::Error => "✗",
                TabBadgeVariant::Info => "ⓘ",
            };
            label.push_str(&format!(" {mark}{}", badge.text));
        }
        if props.disabled || tab.disabled {
            label = format!("~{label}~");
        }
        label
    }

    fn measured(&self, mut element: Element, index: usize, close: bool) -> Element {
        let targets = self.targets.clone();
        element.metadata.layout.push(Arc::new(move |layout| {
            if let Some(target) = targets.lock().unwrap().get_mut(index) {
                if close {
                    target.close = Some(layout);
                } else {
                    target.header = Some(layout);
                }
            }
            false
        }));
        element
    }

    fn target_at(&self, event: &MouseEvent) -> Option<(usize, bool)> {
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (event.position.x() as f32, event.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        let contains = |layout: LayoutInfo| {
            let clip = layout.clip;
            x >= clip.x
                && y >= clip.y
                && x < clip.x + clip.width
                && y < clip.y + clip.height
                && layout.local_cell(x, y).is_some()
        };
        self.targets
            .lock()
            .unwrap()
            .iter()
            .enumerate()
            .find_map(|(i, target)| {
                if target.close.is_some_and(contains) {
                    Some((i, true))
                } else if target.header.is_some_and(contains) {
                    Some((i, false))
                } else {
                    None
                }
            })
    }
}

impl Component for Tabs {
    type Props = TabsProps;
    type State = TabsState;

    fn new(props: Self::Props) -> Self {
        Self {
            authored: props.clone(),
            identities: identities(&props),
            live: props,
            closed: HashSet::new(),
            received_event: false,
            viewport: None,
            targets: Arc::new(Mutex::new(Vec::new())),
            on_change: None,
            on_close: None,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.reconcile(props, state);
        let props = &self.live;
        if state
            .focused_tab
            .is_none_or(|i| props.tabs.get(i).is_none_or(|t| t.disabled))
        {
            state.focused_tab = Self::active(props);
        }
        true
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        self.viewport = Some(layout);
        false
    }

    fn render(&self, _props: &Self::Props, state: &Self::State) -> Element {
        let props = &self.live;
        // Discard targets from the previous structure; callbacks replace them after presentation.
        *self.targets.lock().unwrap() = vec![TabTarget::default(); props.tabs.len()];
        let active = Self::active(props);
        let vertical = props.orientation == TabOrientation::Vertical
            || matches!(props.position, TabPosition::Left | TabPosition::Right);
        let mut headers = Vec::new();
        for (index, tab) in props.tabs.iter().enumerate() {
            let mut label = Element::text(self.render_tab_header(tab, index, props, state))
                .with_class("whitespace-pre shrink-0");
            if active == Some(index) {
                label = label.with_class(match props.variant {
                    TabVariant::Line => "underline",
                    TabVariant::Enclosed => "border",
                    TabVariant::Soft => "bg-gray-700",
                    TabVariant::Solid => "bg-blue-600 text-white",
                    TabVariant::Unstyled => "",
                });
            }
            let padding = match props.size {
                TabSize::Small => "px-0",
                TabSize::Medium => "px-0.5",
                TabSize::Large => "px-1",
            };
            let mut children = vec![label];
            if (props.closable || tab.closable) && !props.disabled && !tab.disabled {
                let mut semantic = Node::new(Role::Button);
                semantic.set_label(format!("Close {} tab", tab.label));
                semantic.set_clickable();
                let mut close = Element::text(" ✕")
                    .with_class("whitespace-pre shrink-0")
                    .with_accessibility(semantic);
                close
                    .metadata
                    .accessibility_options
                    .get_or_insert_default()
                    .click_event = Some(crate::event::CustomEvent::new(
                    "reactive_tui.tabs.close",
                    index.to_string().into_bytes(),
                ));
                children.push(self.measured(close, index, true));
            }
            let enabled = !props.disabled && !tab.disabled;
            let mut semantic = Node::new(Role::Tab);
            semantic.set_label(&tab.label);
            semantic.set_selected(active == Some(index));
            if enabled {
                semantic.set_clickable();
            } else {
                semantic.set_disabled();
            }
            let mut header = Element::layout(LayoutType::Flex)
                .with_key(format!("header-{:?}", self.identities[index]))
                .with_class(format!("flex flex-row items-center shrink-0 {padding}"))
                .with_children(children)
                .with_accessibility(semantic);
            if enabled {
                let options = header
                    .metadata
                    .accessibility_options
                    .get_or_insert_default();
                options.focus = state.is_focused && state.focused_tab == Some(index);
                options.focus_event = Some(crate::event::CustomEvent::new(
                    "reactive_tui.tabs.focus",
                    index.to_string().into_bytes(),
                ));
                options.click_event = Some(crate::event::CustomEvent::new(
                    "reactive_tui.tabs.activate",
                    index.to_string().into_bytes(),
                ));
            }
            headers.push(self.measured(header, index, false));
        }
        let header = Element::layout(LayoutType::Flex)
            .with_key("headers")
            .with_class(if vertical {
                "flex flex-col shrink-0"
            } else {
                "flex flex-row shrink-0"
            })
            .with_children(headers)
            .with_accessibility(Node::new(Role::TabList))
            .with_accessibility_label("Tabs");
        let panels = props
            .tabs
            .iter()
            .enumerate()
            .filter_map(|(index, tab)| {
                if props.lazy_loading && active != Some(index) {
                    return None;
                }
                Some(
                    Element::layout(LayoutType::Flex)
                        .with_key(format!("panel-{:?}", self.identities[index]))
                        .with_class(if active == Some(index) {
                            "flex flex-col min-w-0"
                        } else {
                            "hidden"
                        })
                        .with_child(tab.content.clone())
                        .with_accessibility(Node::new(Role::TabPanel))
                        .with_accessibility_label(format!("{} panel", tab.label)),
                )
            })
            .collect();
        let content = Element::layout(LayoutType::Flex)
            .with_key("panels")
            .with_class("flex flex-col flex-1 min-w-0 overflow-hidden")
            .with_children(panels);
        let mut children = if matches!(props.position, TabPosition::Bottom | TabPosition::Right) {
            vec![content, header]
        } else {
            vec![header, content]
        };
        if let Some(tooltip) = state
            .hover_tab
            .and_then(|index| props.tabs.get(index))
            .and_then(|tab| tab.tooltip.as_ref())
        {
            children.push(Element::text(tooltip).with_key("tooltip")
                .with_class("absolute bottom-0 left-0 z-10 bg-gray-800 text-white whitespace-pre overflow-hidden"));
        }
        let mut root = Element::layout(LayoutType::Flex)
            .with_class(
                if matches!(props.position, TabPosition::Left | TabPosition::Right) {
                    "flex flex-row min-w-0 overflow-hidden"
                } else {
                    "flex flex-col min-w-0 overflow-hidden"
                },
            )
            .with_children(children)
            .with_accessibility(Node::new(Role::Group));
        root.focus = Some(crate::component::FocusProps::input());
        root.metadata.disabled = props.disabled || active.is_none();
        root
    }

    fn handle_event(
        &mut self,
        event: &Event,
        supplied: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        // Direct Component users may pass their initial props on first delivery.
        if !self.received_event && self.authored != *supplied {
            self.reconcile(supplied, state);
        }
        self.received_event = true;
        let mut current = self.live.clone();
        let result = self.dispatch(event, &mut current, state);
        self.live = current.clone();
        *supplied = current;
        result
    }
}

impl Tabs {
    fn dispatch(
        &mut self,
        event: &Event,
        props: &mut TabsProps,
        state: &mut TabsState,
    ) -> EventResult {
        if let Event::Focus(focus) = event {
            match focus.kind {
                FocusEventKind::Gained if !props.disabled => {
                    state.is_focused = true;
                    state.focused_tab = Self::active(props);
                }
                FocusEventKind::Lost => {
                    state.is_focused = false;
                    state.hover_tab = None;
                }
                _ => return EventResult::Ignored,
            }
            return EventResult::Consumed;
        }
        if props.disabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Custom(event)
                if matches!(
                    event.name.as_str(),
                    "reactive_tui.tabs.focus"
                        | "reactive_tui.tabs.activate"
                        | "reactive_tui.tabs.close"
                ) =>
            {
                let Some(index) = std::str::from_utf8(&event.data)
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|&i| props.tabs.get(i).is_some_and(|tab| !tab.disabled))
                else {
                    return EventResult::Ignored;
                };
                if event.name == "reactive_tui.tabs.close" {
                    self.close(index, props, state);
                    return EventResult::Consumed;
                }
                state.is_focused = true;
                state.focused_tab = Some(index);
                if event.name == "reactive_tui.tabs.activate"
                    || props.keyboard_activation == TabKeyboardActivation::Automatic
                {
                    self.activate(index, props, state);
                }
                EventResult::Consumed
            }
            Event::Key(key)
                if state.is_focused && key.kind != crate::event::types::KeyEventKind::Release =>
            {
                let Some(current) = state
                    .focused_tab
                    .filter(|&i| props.tabs.get(i).is_some_and(|t| !t.disabled))
                    .or_else(|| Self::active(props))
                else {
                    return EventResult::Ignored;
                };
                let next = match key.code {
                    KeyCode::Left | KeyCode::Up => self.find_next_enabled_tab(props, current, -1),
                    KeyCode::Right | KeyCode::Down => self.find_next_enabled_tab(props, current, 1),
                    KeyCode::Home => props.tabs.iter().position(|t| !t.disabled),
                    KeyCode::End => props.tabs.iter().rposition(|t| !t.disabled),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.activate(current, props, state);
                        return EventResult::Consumed;
                    }
                    KeyCode::Delete | KeyCode::Char('x') => {
                        self.close(current, props, state);
                        return EventResult::Consumed;
                    }
                    KeyCode::Char(c @ '1'..='9') => {
                        self.activate(c as usize - '1' as usize, props, state);
                        return EventResult::Consumed;
                    }
                    _ => return EventResult::Ignored,
                };
                if let Some(next) = next {
                    state.focused_tab = Some(next);
                    if props.keyboard_activation == TabKeyboardActivation::Automatic {
                        self.activate(next, props, state);
                    }
                }
                EventResult::Consumed
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down | MouseEventKind::Click
                    if mouse.button == crate::event::types::MouseButton::Left =>
                {
                    if let Some((index, close)) = self.target_at(mouse) {
                        if close {
                            self.close(index, props, state);
                        } else {
                            self.activate(index, props, state);
                        }
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                }
                MouseEventKind::Move | MouseEventKind::Enter => {
                    state.hover_tab = self.target_at(mouse).map(|t| t.0);
                    EventResult::Ignored
                }
                MouseEventKind::Leave => {
                    state.hover_tab = None;
                    EventResult::Ignored
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyEvent, KeyModifiers};

    #[test]
    fn authored_updates_preserve_keyed_choice_and_replace_content() {
        let mut props = TabsProps {
            tabs: vec![
                Tab::new("A", Element::text("A1").with_key("a")),
                Tab::new("B", Element::text("B1").with_key("b")),
            ],
            closable: true,
            ..Default::default()
        };
        let mut tabs = Tabs::new(props.clone());
        let mut state = TabsState {
            is_focused: true,
            focused_tab: Some(0),
            ..Default::default()
        };
        tabs.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::Right)),
            &mut props,
            &mut state,
        );
        assert_eq!(tabs.live.active_tab, 1);
        let mut authored = TabsProps {
            tabs: vec![
                Tab::new("B", Element::text("B2").with_key("b")),
                Tab::new("A", Element::text("A2").with_key("a")),
            ],
            closable: true,
            ..Default::default()
        };
        tabs.update(&authored, &mut state);
        assert_eq!(tabs.live.active_tab, 0);
        assert_eq!(state.focused_tab, Some(0));
        assert_eq!(tabs.live.tabs[0].content, Element::text("B2").with_key("b"));
        authored.active_tab = 1;
        tabs.update(&authored, &mut state);
        assert_eq!(tabs.live.active_tab, 1);
        state.focused_tab = Some(1);
        tabs.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::Delete)),
            &mut props,
            &mut state,
        );
        tabs.update(&authored, &mut state);
        assert_eq!(tabs.live.tabs.len(), 1);
        assert_eq!(tabs.live.tabs[0].label, "B");
        authored.tabs.remove(1);
        tabs.update(&authored, &mut state);
        authored
            .tabs
            .push(Tab::new("A", Element::text("A3").with_key("a")));
        tabs.update(&authored, &mut state);
        assert_eq!(tabs.live.tabs.len(), 2);
    }

    #[test]
    fn test_tabs_basic_functionality() {
        let mut tabs = Tabs::new(TabsProps::default());
        let mut props = TabsProps {
            tabs: vec![
                Tab::new("Tab 1", Element::text("Content 1")),
                Tab::new("Tab 2", Element::text("Content 2")),
                Tab::new("Tab 3", Element::text("Content 3")),
            ],
            active_tab: 0,
            ..Default::default()
        };
        let mut state = TabsState {
            is_focused: true,
            focused_tab: Some(0),
            ..Default::default()
        };

        // Test keyboard navigation
        let event = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        tabs.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.active_tab, 1);
        assert_eq!(state.focused_tab, Some(1));
    }

    #[test]
    fn test_tabs_disabled_navigation() {
        let tabs = Tabs::new(TabsProps::default());
        let props = TabsProps {
            tabs: vec![
                Tab::new("Tab 1", Element::text("Content 1")),
                Tab::new("Tab 2", Element::text("Content 2")).disabled(true),
                Tab::new("Tab 3", Element::text("Content 3")),
            ],
            ..Default::default()
        };

        // Should skip disabled tab
        let next_tab = tabs.find_next_enabled_tab(&props, 0, 1);
        assert_eq!(next_tab, Some(2)); // Skip index 1 (disabled)
    }

    #[test]
    fn test_tabs_with_badges() {
        let tabs = Tabs::new(TabsProps::default());
        let tab = Tab::new("Test", Element::text("Content"))
            .with_badge(TabBadge::new("5").with_variant(TabBadgeVariant::Error));

        let props = TabsProps {
            tabs: vec![tab],
            ..Default::default()
        };
        let state = TabsState {
            is_focused: true,
            focused_tab: Some(0),
            ..Default::default()
        };

        let header = tabs.render_tab_header(&props.tabs[0], 0, &props, &state);
        assert!(header.contains("Test"));
        assert!(header.contains("✗5")); // Error badge
    }

    #[test]
    fn test_tabs_closable() {
        let mut tabs = Tabs::new(TabsProps::default());
        let mut props = TabsProps {
            tabs: vec![Tab::new("Tab 1", Element::text("Content 1")).closable(true)],
            ..Default::default()
        };
        let mut state = TabsState {
            is_focused: true,
            focused_tab: Some(0),
            ..Default::default()
        };

        let _close_called = false;
        tabs.on_close = Some(Arc::new(|_| {}));

        // Test close with Delete key
        let event = Event::Key(KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        let result = tabs.handle_event(&event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
    }

    #[test]
    fn test_tabs_keyboard_activation_modes() {
        let mut tabs = Tabs::new(TabsProps::default());
        let mut props = TabsProps {
            tabs: vec![
                Tab::new("Tab 1", Element::text("Content 1")),
                Tab::new("Tab 2", Element::text("Content 2")),
            ],
            keyboard_activation: TabKeyboardActivation::Manual,
            ..Default::default()
        };
        let mut state = TabsState {
            is_focused: true,
            focused_tab: Some(0),
            ..Default::default()
        };

        // Move focus but don't activate (manual mode)
        let event = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        tabs.handle_event(&event, &mut props, &mut state);
        assert_eq!(state.focused_tab, Some(1)); // Focus moved
        assert_eq!(props.active_tab, 0); // But not activated

        // Now activate with Enter
        let event = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        tabs.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.active_tab, 1); // Now activated
    }

    #[test]
    fn test_tabs_quick_access() {
        let mut tabs = Tabs::new(TabsProps::default());
        let mut props = TabsProps {
            tabs: vec![
                Tab::new("Tab 1", Element::text("Content 1")),
                Tab::new("Tab 2", Element::text("Content 2")),
                Tab::new("Tab 3", Element::text("Content 3")),
            ],
            ..Default::default()
        };
        let mut state = TabsState {
            is_focused: true,
            ..Default::default()
        };

        // Quick access to tab 3 with '3'
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::empty(),
            kind: crate::event::types::KeyEventKind::Press,
            repeat: false,
            timestamp: std::time::Instant::now(),
        });

        tabs.handle_event(&event, &mut props, &mut state);
        assert_eq!(props.active_tab, 2); // Tab 3 (0-indexed)
        assert_eq!(state.focused_tab, Some(2));
    }
}
