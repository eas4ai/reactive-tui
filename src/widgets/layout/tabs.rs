use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{FocusEventKind, KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Properties for Tabs component
#[derive(Clone, Debug, PartialEq)]
pub struct TabsProps {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub orientation: TabOrientation,
    pub variant: TabVariant,
    pub size: TabSize,
    pub position: TabPosition,
    pub closable: bool,
    pub disabled: bool,
    pub lazy_loading: bool, // Only render active tab content
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

#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub label: String,
    pub content: Element,
    pub disabled: bool,
    pub closable: bool,
    pub icon: Option<String>,
    pub badge: Option<TabBadge>,
    pub tooltip: Option<String>,
}

impl Tab {
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_badge(mut self, badge: TabBadge) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabBadge {
    pub text: String,
    pub variant: TabBadgeVariant,
}

impl TabBadge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: TabBadgeVariant::Default,
        }
    }

    pub fn with_variant(mut self, variant: TabBadgeVariant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabBadgeVariant {
    Default,
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabVariant {
    Line,     // Underline/border style
    Enclosed, // Box/card style
    Soft,     // Subtle background style
    Solid,    // Filled background style
    Unstyled, // No decoration
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabKeyboardActivation {
    Automatic, // Activate on focus
    Manual,    // Activate on Enter/Space
}

/// State for Tabs component
#[derive(Clone, Debug, Default)]
pub struct TabsState {
    pub focused_tab: Option<usize>,
    pub hover_tab: Option<usize>,
    pub is_focused: bool,
}

/// Tabs component with full keyboard and mouse support
pub struct Tabs {
    state: TabsState,
    on_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_close: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Tabs {
    /// Set the onChange callback (called when active tab changes)
    pub fn with_on_change(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the onClose callback (called when a tab is closed)
    pub fn with_on_close(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
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

        let mut index = current;
        for _ in 0..len {
            index = if direction > 0 {
                (index + 1) % len
            } else if index == 0 {
                len - 1
            } else {
                index - 1
            };

            if !props.tabs[index].disabled {
                return Some(index);
            }
        }
        None
    }

    fn render_tab_header(
        &self,
        tab: &Tab,
        index: usize,
        props: &TabsProps,
        state: &TabsState,
    ) -> String {
        let is_active = index == props.active_tab;
        let is_focused = state.focused_tab == Some(index);
        let is_hovered = state.hover_tab == Some(index);

        let mut header = String::new();

        // Add focus/hover indicator
        if is_focused && state.is_focused {
            header.push('▶');
        } else if is_hovered {
            header.push('→');
        } else {
            header.push(' ');
        }

        // Add icon if present
        if let Some(icon) = &tab.icon {
            header.push_str(icon);
            header.push(' ');
        }

        // Style the tab based on variant and state
        match props.variant {
            TabVariant::Line => {
                if is_active {
                    header.push_str(&format!("┌─{}─┐", "─".repeat(tab.label.len())));
                    header.push('\n');
                    header.push_str(&format!("│ {} │", tab.label));
                    header.push('\n');
                    header.push_str(&format!("└─{}─┘", "─".repeat(tab.label.len())));
                } else if is_focused {
                    header.push_str(&format!("[{}]", tab.label));
                } else {
                    header.push_str(&tab.label);
                }
            }
            TabVariant::Enclosed => {
                if is_active || is_focused {
                    header.push_str(&format!("┌{}┐", "─".repeat(tab.label.len() + 2)));
                    header.push('\n');
                    header.push_str(&format!("│ {} │", tab.label));
                    header.push('\n');
                    header.push_str(&format!("└{}┘", "─".repeat(tab.label.len() + 2)));
                } else {
                    header.push_str(&tab.label);
                }
            }
            TabVariant::Soft => {
                if is_active {
                    header.push_str(&format!("⟦{}⟧", tab.label));
                } else if is_focused {
                    header.push_str(&format!("⟨{}⟩", tab.label));
                } else {
                    header.push_str(&tab.label);
                }
            }
            TabVariant::Solid => {
                if is_active {
                    header.push_str(&format!("■ {} ■", tab.label));
                } else if is_focused {
                    header.push_str(&format!("▣ {} ▣", tab.label));
                } else {
                    header.push_str(&format!("□ {} □", tab.label));
                }
            }
            TabVariant::Unstyled => {
                header.push_str(&tab.label);
            }
        }

        // Add badge if present
        if let Some(badge) = &tab.badge {
            let badge_char = match badge.variant {
                TabBadgeVariant::Default => "●",
                TabBadgeVariant::Success => "✓",
                TabBadgeVariant::Warning => "⚠",
                TabBadgeVariant::Error => "✗",
                TabBadgeVariant::Info => "ⓘ",
            };
            header.push_str(&format!(" {}{}", badge_char, badge.text));
        }

        // Add close button if closable
        if (tab.closable || props.closable) && !tab.disabled {
            header.push_str(" ✕");
        }

        // Show disabled state
        if tab.disabled {
            header = format!("~{header}~");
        }

        header
    }

    fn render_tab_list(&self, props: &TabsProps, state: &TabsState) -> String {
        if props.tabs.is_empty() {
            return String::new();
        }

        let mut tab_headers = Vec::new();

        for (index, tab) in props.tabs.iter().enumerate() {
            let header = self.render_tab_header(tab, index, props, state);
            tab_headers.push(header);
        }

        match props.orientation {
            TabOrientation::Horizontal => {
                // Join tabs horizontally with spacing
                match props.position {
                    TabPosition::Top | TabPosition::Bottom => tab_headers.join("  "),
                    TabPosition::Left | TabPosition::Right => tab_headers.join("\n"),
                }
            }
            TabOrientation::Vertical => {
                // Join tabs vertically
                tab_headers.join("\n")
            }
        }
    }

    fn render_tab_content(&self, props: &TabsProps) -> String {
        if props.tabs.is_empty() || props.active_tab >= props.tabs.len() {
            return String::new();
        }

        let active_tab = &props.tabs[props.active_tab];
        // For now, render only the active tab's content as plain text by walking the Element tree.
        self.render_element_text(&active_tab.content)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn render_element_text(&self, element: &crate::component::Element) -> String {
        match &element.element_type {
            crate::component::ElementType::Text(t) => t.clone(),
            _ => element
                .children
                .iter()
                .map(|child| self.render_element_text(child))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    fn get_tab_at_position(&self, props: &TabsProps, x: usize, y: usize) -> Option<usize> {
        if props.tabs.is_empty() {
            return None;
        }

        // Calculate actual tab positions based on rendered text
        let mut tab_positions = Vec::new();
        let mut current_x = 0;
        let mut current_y = 0;

        for tab in props.tabs.iter() {
            // Calculate tab width including decorations
            let label_width = tab.label.len();
            let icon_width = tab.icon.as_ref().map(|i| i.len() + 1).unwrap_or(0);
            let badge_width = tab.badge.as_ref().map(|b| b.text.len() + 2).unwrap_or(0);
            let close_width = if tab.closable || props.closable { 2 } else { 0 };
            let focus_indicator = 1; // Space for focus/hover indicator

            let total_width =
                focus_indicator + icon_width + label_width + badge_width + close_width + 2; // +2 for padding

            match props.orientation {
                TabOrientation::Horizontal => {
                    tab_positions.push((
                        current_x,
                        current_y,
                        current_x + total_width,
                        current_y + 1,
                    ));
                    current_x += total_width + 2; // Add spacing between tabs
                }
                TabOrientation::Vertical => {
                    let line_height = match props.variant {
                        TabVariant::Line | TabVariant::Enclosed => 3, // Multi-line variants
                        _ => 1,                                       // Single line variants
                    };
                    tab_positions.push((0, current_y, total_width, current_y + line_height));
                    current_y += line_height;
                }
            }
        }

        // Find which tab was clicked
        for (index, (x1, y1, x2, y2)) in tab_positions.iter().enumerate() {
            if x >= *x1 && x < *x2 && y >= *y1 && y < *y2 {
                return Some(index);
            }
        }

        None
    }

    fn is_close_button_clicked(
        &self,
        props: &TabsProps,
        tab_index: usize,
        x: usize,
        y: usize,
    ) -> bool {
        if tab_index >= props.tabs.len() {
            return false;
        }

        let tab = &props.tabs[tab_index];
        if !tab.closable && !props.closable {
            return false;
        }

        // Calculate the exact position of the close button
        let mut tab_end_x = 1; // Focus indicator
        tab_end_x += tab.icon.as_ref().map(|i| i.len() + 1).unwrap_or(0);
        tab_end_x += tab.label.len();
        tab_end_x += tab.badge.as_ref().map(|b| b.text.len() + 2).unwrap_or(0);
        tab_end_x += 1; // Space before close button

        // For horizontal tabs, accumulate x position
        if props.orientation == TabOrientation::Horizontal && tab_index > 0 {
            for i in 0..tab_index {
                let prev_tab = &props.tabs[i];
                let prev_width = 1
                    + prev_tab.icon.as_ref().map(|i| i.len() + 1).unwrap_or(0)
                    + prev_tab.label.len()
                    + prev_tab
                        .badge
                        .as_ref()
                        .map(|b| b.text.len() + 2)
                        .unwrap_or(0)
                    + (if prev_tab.closable || props.closable {
                        2
                    } else {
                        0
                    })
                    + 2;
                tab_end_x += prev_width + 2; // +2 for spacing
            }
        }

        // Check if click is on the close button (✕ is 1 char wide)
        match props.orientation {
            TabOrientation::Horizontal => x >= tab_end_x && x <= tab_end_x + 1,
            TabOrientation::Vertical => {
                // For vertical tabs, check both x and y
                let line_height = match props.variant {
                    TabVariant::Line | TabVariant::Enclosed => 3,
                    _ => 1,
                };
                let tab_y = tab_index * line_height;
                y >= tab_y && y < tab_y + line_height && x >= tab_end_x && x <= tab_end_x + 1
            }
        }
    }
}

impl Component for Tabs {
    type Props = TabsProps;
    type State = TabsState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: TabsState::default(),
            on_change: None,
            on_close: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        if props.tabs.is_empty() {
            return Element::text("No tabs");
        }

        let tab_list = self.render_tab_list(props, state);
        let tab_content = self.render_tab_content(props);

        let result = match props.position {
            TabPosition::Top => {
                format!("{}\n{}\n{}", tab_list, "─".repeat(80), tab_content)
            }
            TabPosition::Bottom => {
                format!("{}\n{}\n{}", tab_content, "─".repeat(80), tab_list)
            }
            TabPosition::Left => {
                // Split content and render side by side
                let tab_lines: Vec<&str> = tab_list.lines().collect();
                let content_lines: Vec<&str> = tab_content.lines().collect();
                let max_lines = tab_lines.len().max(content_lines.len());

                let mut combined_lines = Vec::new();
                for i in 0..max_lines {
                    let tab_line = tab_lines.get(i).unwrap_or(&"");
                    let content_line = content_lines.get(i).unwrap_or(&"");
                    combined_lines.push(format!("{tab_line:<20} │ {content_line}"));
                }
                combined_lines.join("\n")
            }
            TabPosition::Right => {
                // Split content and render side by side (content first)
                let tab_lines: Vec<&str> = tab_list.lines().collect();
                let content_lines: Vec<&str> = tab_content.lines().collect();
                let max_lines = tab_lines.len().max(content_lines.len());

                let mut combined_lines = Vec::new();
                for i in 0..max_lines {
                    let tab_line = tab_lines.get(i).unwrap_or(&"");
                    let content_line = content_lines.get(i).unwrap_or(&"");
                    combined_lines.push(format!("{content_line} │ {tab_line:<20}"));
                }
                combined_lines.join("\n")
            }
        };

        Element::text(result)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(key_event) => self.handle_key_event(key_event, props, state),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(focus_event) => {
                match focus_event.kind {
                    FocusEventKind::Gained => {
                        state.is_focused = true;
                        state.focused_tab = Some(props.active_tab);
                    }
                    FocusEventKind::Lost => {
                        state.is_focused = false;
                        state.focused_tab = None;
                        state.hover_tab = None;
                    }
                    _ => {}
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Tabs {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut TabsProps,
        state: &mut TabsState,
    ) -> EventResult {
        if !state.is_focused {
            return EventResult::Ignored;
        }

        let current_focused = state.focused_tab.unwrap_or(props.active_tab);

        match event.code {
            KeyCode::Left | KeyCode::Up => {
                // Move focus to previous tab
                if let Some(prev_tab) = self.find_next_enabled_tab(props, current_focused, -1) {
                    state.focused_tab = Some(prev_tab);

                    // Auto-activate if enabled
                    if props.keyboard_activation == TabKeyboardActivation::Automatic {
                        props.active_tab = prev_tab;
                        if let Some(on_change) = &self.on_change {
                            on_change(prev_tab);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Right | KeyCode::Down => {
                // Move focus to next tab
                if let Some(next_tab) = self.find_next_enabled_tab(props, current_focused, 1) {
                    state.focused_tab = Some(next_tab);

                    // Auto-activate if enabled
                    if props.keyboard_activation == TabKeyboardActivation::Automatic {
                        props.active_tab = next_tab;
                        if let Some(on_change) = &self.on_change {
                            on_change(next_tab);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                // Focus first enabled tab
                if let Some(first_tab) = props.tabs.iter().position(|tab| !tab.disabled) {
                    state.focused_tab = Some(first_tab);

                    if props.keyboard_activation == TabKeyboardActivation::Automatic {
                        props.active_tab = first_tab;
                        if let Some(on_change) = &self.on_change {
                            on_change(first_tab);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::End => {
                // Focus last enabled tab
                if let Some(last_tab) = props.tabs.iter().rposition(|tab| !tab.disabled) {
                    state.focused_tab = Some(last_tab);

                    if props.keyboard_activation == TabKeyboardActivation::Automatic {
                        props.active_tab = last_tab;
                        if let Some(on_change) = &self.on_change {
                            on_change(last_tab);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Activate focused tab (for manual activation mode)
                if let Some(focused) = state.focused_tab {
                    if !props.tabs[focused].disabled {
                        props.active_tab = focused;
                        if let Some(on_change) = &self.on_change {
                            on_change(focused);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Delete | KeyCode::Char('x') => {
                // Close focused tab if closable
                if let Some(focused) = state.focused_tab {
                    let tab = &props.tabs[focused];
                    if (tab.closable || props.closable) && !tab.disabled {
                        if let Some(on_close) = &self.on_close {
                            on_close(focused);
                        }
                        // Note: actual tab removal should be handled by parent component
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Quick access to tabs by number (1-9)
                let tab_index = (c.to_digit(10).unwrap() as usize).saturating_sub(1);
                if tab_index < props.tabs.len() && !props.tabs[tab_index].disabled {
                    state.focused_tab = Some(tab_index);
                    props.active_tab = tab_index;
                    if let Some(on_change) = &self.on_change {
                        on_change(tab_index);
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut TabsProps,
        state: &mut TabsState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Click => {
                let x = event.position.x() as usize;
                let y = event.position.y() as usize;

                if let Some(tab_index) = self.get_tab_at_position(props, x, y) {
                    // Check if close button was clicked
                    if self.is_close_button_clicked(props, tab_index, x, y) {
                        let tab = &props.tabs[tab_index];
                        if (tab.closable || props.closable) && !tab.disabled {
                            if let Some(on_close) = &self.on_close {
                                on_close(tab_index);
                            }
                            return EventResult::Consumed;
                        }
                    }

                    // Regular tab click
                    if !props.tabs[tab_index].disabled {
                        state.focused_tab = Some(tab_index);
                        state.is_focused = true;
                        props.active_tab = tab_index;
                        if let Some(on_change) = &self.on_change {
                            on_change(tab_index);
                        }
                    }
                }
                EventResult::Consumed
            }
            MouseEventKind::Move => {
                // Track hover state
                let x = event.position.x() as usize;
                let y = event.position.y() as usize;

                state.hover_tab = self.get_tab_at_position(props, x, y);
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;
    use crate::event::types::KeyModifiers;

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
    fn test_tabs_render_with_variants() {
        let tabs = Tabs::new(TabsProps::default());
        let props = TabsProps {
            tabs: vec![Tab::new("Tab 1", Element::text("Content 1"))],
            variant: TabVariant::Enclosed,
            ..Default::default()
        };
        let state = TabsState::default();

        let element = tabs.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            assert!(content.contains("Tab 1"));
            assert!(content.contains("Content 1"));
        }
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
