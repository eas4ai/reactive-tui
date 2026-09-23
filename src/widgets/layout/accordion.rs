//! Production-Ready Accordion Widget
//!
//! A sophisticated accordion component with:
//! - Smooth animations with spring physics
//! - Full accessibility support (ARIA, screen readers)
//! - Advanced keyboard navigation
//! - Customizable themes and styling
//! - Performance optimizations
//! - Event delegation and bubbling
//! - Nested accordion support
//! - Auto-collapse modes

use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent};
use crate::event::Event;
use std::any::Any;
use std::collections::HashMap;
use std::time::Duration;

/// Accordion expansion mode
#[derive(Clone, Debug, PartialEq, Default)]
pub enum AccordionMode {
    /// Only one section can be expanded at a time
    #[default]
    Single,
    /// Multiple sections can be expanded simultaneously
    Multiple,
    /// At least one section must always be expanded
    AlwaysOne,
}

/// Animation configuration for accordion transitions
#[derive(Clone, Debug, PartialEq)]
pub struct AccordionAnimation {
    /// Duration for the animation
    pub duration: Duration,
    /// Whether to stagger animations when multiple sections change
    pub stagger: bool,
    /// Stagger delay between sections
    pub stagger_delay: Duration,
}

impl Default for AccordionAnimation {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(300),
            stagger: true,
            stagger_delay: Duration::from_millis(50),
        }
    }
}

/// Individual accordion section configuration
#[derive(Clone, Debug, PartialEq)]
pub struct AccordionSection {
    /// Unique identifier for the section
    pub id: String,
    /// Display title for the section header
    pub title: String,
    /// Optional icon for the section
    pub icon: Option<String>,
    /// Content element to display when expanded
    pub content: Element,
    /// Whether this section is disabled
    pub disabled: bool,
    /// Custom CSS classes for the section
    pub class: Option<String>,
    /// Whether this section is initially expanded
    pub expanded: bool,
    /// Custom header element (overrides title/icon)
    pub custom_header: Option<Element>,
    /// Accessibility label for screen readers
    pub aria_label: Option<String>,
}

impl AccordionSection {
    /// Create a new accordion section
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            content: Element::empty(),
            disabled: false,
            class: None,
            expanded: false,
            custom_header: None,
            aria_label: None,
        }
    }

    /// Set the content for this section
    pub fn content(mut self, content: Element) -> Self {
        self.content = content;
        self
    }

    /// Set an icon for this section
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Mark this section as initially expanded
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Mark this section as disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add custom CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Set a custom header element
    pub fn custom_header(mut self, header: Element) -> Self {
        self.custom_header = Some(header);
        self
    }

    /// Set accessibility label
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
}

/// Accordion widget properties
#[derive(Clone, Debug, PartialEq)]
pub struct AccordionProps {
    /// List of accordion sections
    pub sections: Vec<AccordionSection>,
    /// Expansion mode (single, multiple, always-one)
    pub mode: AccordionMode,
    /// Animation configuration
    pub animation: AccordionAnimation,
    /// Whether to show expand/collapse icons
    pub show_icons: bool,
    /// Custom expand/collapse icons
    pub expand_icon: String,
    /// Icon to show when section is expanded
    pub collapse_icon: String,
    /// Whether keyboard navigation is enabled
    pub keyboard_navigation: bool,
    /// Custom CSS classes for the accordion container
    pub class: Option<String>,
    /// Whether to persist state across re-renders
    pub persist_state: bool,
    /// CustomEvent name delivered to the App root when expansion changes.
    /// JSON data contains section_id, expanded, and ordered expanded_sections.
    pub on_change: Option<String>, // Event handler ID
    /// Whether to use reduced motion (accessibility)
    pub reduced_motion: bool,
}

impl Default for AccordionProps {
    fn default() -> Self {
        Self {
            sections: Vec::new(),
            mode: AccordionMode::Single,
            animation: AccordionAnimation::default(),
            show_icons: true,
            expand_icon: "▼".to_string(),
            collapse_icon: "▲".to_string(),
            keyboard_navigation: true,
            class: None,
            persist_state: true,
            on_change: None,
            reduced_motion: false,
        }
    }
}

impl Props for AccordionProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Accordion widget state
#[derive(Clone, Debug, Default)]
pub struct AccordionState {
    /// Currently expanded sections
    pub expanded_sections: HashMap<String, bool>,
    /// Currently focused section (for keyboard navigation)
    pub focused_section: Option<String>,
    /// Animation states for each section
    pub animations: HashMap<String, bool>,
    /// Whether the accordion has been initialized
    pub initialized: bool,
    /// Last interaction timestamp (for performance)
    pub last_interaction: Option<std::time::Instant>,
}

/// Production-ready Accordion widget
pub struct Accordion;

mod live;

impl Component for Accordion {
    type Props = AccordionProps;
    type State = AccordionState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = AccordionState::default();
        self.initialize_state(props, &mut state);
        state
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if !state.initialized || !props.persist_state {
            self.initialize_state(props, state);
        } else {
            self.sync_sections(props, state);
        }
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveAccordion>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Key(key) = event {
            if props.keyboard_navigation {
                return self.handle_keyboard_event(key, props, state);
            }
        }
        EventResult::Ignored
    }
}

impl Accordion {
    fn initialize_state(&self, props: &AccordionProps, state: &mut AccordionState) {
        *state = AccordionState::default();
        state.expanded_sections = props
            .sections
            .iter()
            .map(|s| (s.id.clone(), s.expanded))
            .collect();
        state.initialized = true;
        self.sync_sections(props, state);
    }

    fn sync_sections(&self, props: &AccordionProps, state: &mut AccordionState) {
        state
            .expanded_sections
            .retain(|id, _| props.sections.iter().any(|s| &s.id == id));
        state
            .animations
            .retain(|id, _| props.sections.iter().any(|s| &s.id == id));
        for section in &props.sections {
            state
                .expanded_sections
                .entry(section.id.clone())
                .or_insert(section.expanded);
            state.animations.entry(section.id.clone()).or_insert(false);
        }
        match props.mode {
            AccordionMode::Single => {
                let mut found = false;
                for section in &props.sections {
                    let expanded = state.expanded_sections.get_mut(&section.id).unwrap();
                    if *expanded {
                        *expanded = !found;
                        found = true;
                    }
                }
            }
            AccordionMode::AlwaysOne if !state.expanded_sections.values().any(|&v| v) => {
                if let Some(section) = props
                    .sections
                    .iter()
                    .find(|s| !s.disabled)
                    .or(props.sections.first())
                {
                    state.expanded_sections.insert(section.id.clone(), true);
                }
            }
            _ => {}
        }
        if state
            .focused_section
            .as_ref()
            .is_none_or(|id| !props.sections.iter().any(|s| &s.id == id && !s.disabled))
        {
            state.focused_section = props
                .sections
                .iter()
                .find(|s| !s.disabled)
                .map(|s| s.id.clone());
        }
    }

    fn handle_keyboard_event(
        &mut self,
        event: &KeyEvent,
        props: &AccordionProps,
        state: &mut AccordionState,
    ) -> EventResult {
        if event.kind == crate::event::types::KeyEventKind::Release {
            return EventResult::Ignored;
        }
        let enabled: Vec<_> = props.sections.iter().filter(|s| !s.disabled).collect();
        if enabled.is_empty() {
            return EventResult::Ignored;
        }
        let current = enabled
            .iter()
            .position(|s| state.focused_section.as_ref() == Some(&s.id))
            .unwrap_or(0);
        let next = match event.code {
            KeyCode::Down => (current + 1) % enabled.len(),
            KeyCode::Up => (current + enabled.len() - 1) % enabled.len(),
            KeyCode::Home => 0,
            KeyCode::End => enabled.len() - 1,
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_section(&enabled[current].id, props, state);
                return EventResult::Consumed;
            }
            _ => return EventResult::Ignored,
        };
        state.focused_section = Some(enabled[next].id.clone());
        EventResult::Consumed
    }

    fn toggle_section(&mut self, id: &str, props: &AccordionProps, state: &mut AccordionState) {
        if !props.sections.iter().any(|s| s.id == id && !s.disabled) {
            return;
        }
        let expanded = !state.expanded_sections.get(id).copied().unwrap_or(false);
        if !expanded
            && props.mode == AccordionMode::AlwaysOne
            && state.expanded_sections.values().filter(|&&v| v).count() <= 1
        {
            return;
        }
        if expanded && props.mode == AccordionMode::Single {
            for value in state.expanded_sections.values_mut() {
                *value = false;
            }
        }
        state.expanded_sections.insert(id.to_owned(), expanded);
        state.last_interaction = Some(std::time::Instant::now());
        if let Some(callback) = &props.on_change {
            let expanded_ids: Vec<_> = props
                .sections
                .iter()
                .filter(|s| state.expanded_sections.get(&s.id) == Some(&true))
                .map(|s| &s.id)
                .collect();
            crate::event::notifications::emit(crate::event::CustomEvent::new(callback, serde_json::json!({"section_id": id, "expanded": expanded, "expanded_sections": expanded_ids}).to_string().into_bytes()));
        }
    }
}

/// Builder for creating accordion widgets with fluent API
pub struct AccordionBuilder {
    props: AccordionProps,
}

impl AccordionBuilder {
    /// Create a new accordion builder
    pub fn new() -> Self {
        Self {
            props: AccordionProps::default(),
        }
    }

    /// Add a section to the accordion
    pub fn section(mut self, section: AccordionSection) -> Self {
        self.props.sections.push(section);
        self
    }

    /// Set the accordion mode
    pub fn mode(mut self, mode: AccordionMode) -> Self {
        self.props.mode = mode;
        self
    }

    /// Enable/disable animations
    pub fn animated(mut self, animated: bool) -> Self {
        self.props.reduced_motion = !animated;
        self
    }

    /// Set custom expand/collapse icons
    pub fn icons(mut self, expand: impl Into<String>, collapse: impl Into<String>) -> Self {
        self.props.expand_icon = expand.into();
        self.props.collapse_icon = collapse.into();
        self
    }

    /// Add custom CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.props.class = Some(class.into());
        self
    }

    /// Enable/disable keyboard navigation
    pub fn keyboard_navigation(mut self, enabled: bool) -> Self {
        self.props.keyboard_navigation = enabled;
        self
    }

    /// Build the accordion element
    pub fn build(self) -> Element {
        Element::component_with_props("Accordion", self.props)
    }
}

impl Default for AccordionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
