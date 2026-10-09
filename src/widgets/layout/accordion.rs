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

use crate::component::{CallbackSlot, Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::KeyEvent;
use crate::event::Event;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// The callback an accordion calls with a section's index and whether it
/// is open.
type ToggleCallback = Arc<dyn Fn(usize, bool) + Send + Sync>;

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
#[derive(Clone)]
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
    /// Called with the section's index in `sections` and whether it is now
    /// open, each time the user opens or closes a section: the change
    /// `on_change` names an event for (CMP-009). Equality leaves it out, so
    /// a rerender that changes only this callback keeps the mounted
    /// accordion and the new callback acts from the next event on (CMP-008).
    pub on_toggle: Option<ToggleCallback>,
    /// The name the screen reader gives the accordion; none when unset.
    pub aria_label: Option<String>,
    /// Whether to use reduced motion (accessibility)
    pub reduced_motion: bool,
}

impl PartialEq for AccordionProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            sections,
            mode,
            animation,
            show_icons,
            expand_icon,
            collapse_icon,
            keyboard_navigation,
            class,
            persist_state,
            on_change,
            on_toggle: _,
            aria_label,
            reduced_motion,
        } = self;
        *sections == other.sections
            && *mode == other.mode
            && *animation == other.animation
            && *show_icons == other.show_icons
            && *expand_icon == other.expand_icon
            && *collapse_icon == other.collapse_icon
            && *keyboard_navigation == other.keyboard_navigation
            && *class == other.class
            && *persist_state == other.persist_state
            && *on_change == other.on_change
            && *aria_label == other.aria_label
            && *reduced_motion == other.reduced_motion
    }
}

impl std::fmt::Debug for AccordionProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionProps")
            .field("sections", &self.sections)
            .field("mode", &self.mode)
            .field("animation", &self.animation)
            .field("show_icons", &self.show_icons)
            .field("expand_icon", &self.expand_icon)
            .field("collapse_icon", &self.collapse_icon)
            .field("keyboard_navigation", &self.keyboard_navigation)
            .field("class", &self.class)
            .field("persist_state", &self.persist_state)
            .field("on_change", &self.on_change)
            .field("on_toggle", &self.on_toggle.as_ref().map(|_| "Fn"))
            .field("aria_label", &self.aria_label)
            .field("reduced_motion", &self.reduced_motion)
            .finish()
    }
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
            on_toggle: None,
            aria_label: None,
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
    /// The `on_toggle` callback, as the latest update or adoption left it;
    /// the live child that handles the input shares the slot (CMP-008).
    pub callbacks: CallbackSlot<Option<ToggleCallback>>,
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
        state.callbacks.set(props.on_toggle.clone());
        state
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if !state.initialized || !props.persist_state {
            self.initialize_state(props, state);
        } else {
            self.sync_sections(props, state);
        }
        state.callbacks.set(props.on_toggle.clone());
        true
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if crate::component::same_callback(&props.on_toggle, &supplied.on_toggle) {
            return false;
        }
        props.on_toggle = supplied.on_toggle.clone();
        state.callbacks.set(props.on_toggle.clone());
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
        // The callback slot stays the one the live child shares (CMP-008).
        *state = AccordionState {
            callbacks: state.callbacks.clone(),
            ..AccordionState::default()
        };
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
        // The keys mean what the active keymap says (KEY-001).
        let action = crate::keymap::Keymap::active().action(event);
        use crate::keymap::Action;
        let next = match action {
            Some(Action::Down) => (current + 1) % enabled.len(),
            Some(Action::Up) => (current + enabled.len() - 1) % enabled.len(),
            Some(Action::Home) => 0,
            Some(Action::End) => enabled.len() - 1,
            Some(Action::Confirm | Action::Activate) => {
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
        if let Some(callback) = state.callbacks.get() {
            if let Some(index) = props.sections.iter().position(|s| s.id == id) {
                callback(index, expanded);
            }
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

    /// The name the screen reader gives the accordion (NAV-004).
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.props.aria_label = Some(label.into());
        self
    }

    /// Set the callback called with a section's index and whether it is
    /// now open, each time the user opens or closes a section: the change
    /// the props' `on_change` event reports (CMP-009). It sets the props'
    /// `on_toggle`.
    pub fn on_change(mut self, f: impl Fn(usize, bool) + Send + Sync + 'static) -> Self {
        self.props.on_toggle = Some(Arc::new(f));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyCode;

    /// KEY-001: the accordion moves its focus on the key the active keymap
    /// binds to Down, and no longer on the key it replaced.
    #[test]
    #[serial_test::serial(keymap)]
    fn key_001_accordion_reads_its_keys_through_the_keymap() {
        use crate::keymap::{Action, KeyBinding, Keymap};
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Down, [KeyBinding::new(KeyCode::F(2))]);
        let _scope = Keymap::scoped(keymap);
        let mut props = AccordionBuilder::new()
            .section(AccordionSection::new("one", "One"))
            .section(AccordionSection::new("two", "Two"))
            .section(AccordionSection::new("three", "Three"))
            .props;
        let mut accordion = Accordion;
        let mut state = accordion.initial_state(&props);
        let rebound = accordion.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::F(2))),
            &mut props,
            &mut state,
        );
        let after_rebound = state.focused_section.clone();
        let old = accordion.handle_event(
            &Event::Key(KeyEvent::new(KeyCode::Down)),
            &mut props,
            &mut state,
        );
        let after_old = state.focused_section.clone();
        // The scope above restores the default keymap when it drops.
        assert_eq!(rebound, EventResult::Consumed);
        assert_eq!(
            after_rebound.as_deref(),
            Some("two"),
            "the new Down key moves the focus"
        );
        assert_eq!(old, EventResult::Ignored);
        assert_eq!(
            after_old.as_deref(),
            Some("two"),
            "the arrow, no longer Down, does nothing"
        );
    }
}
