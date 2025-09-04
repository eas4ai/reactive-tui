use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyCode, KeyEvent, MouseEventKind};
use crate::event::{Event, MouseEvent};
use std::any::Any;
use std::sync::Arc;

/// Builder for creating Select components with a fluent API
#[derive(Clone, Debug)]
pub struct SelectBuilder<T: Clone + PartialEq + Send + Sync + 'static> {
    options: Vec<SelectOption<T>>,
    selected: Option<T>,
    placeholder: Option<String>,
    disabled: bool,
    width: Option<u16>,
    max_visible_items: usize,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> SelectBuilder<T> {
    /// Create a new SelectBuilder
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            placeholder: Some("Select an option...".to_string()),
            disabled: false,
            width: Some(30),
            max_visible_items: 5,
        }
    }

    /// Set the options
    pub fn options(mut self, options: Vec<SelectOption<T>>) -> Self {
        self.options = options;
        self
    }

    /// Add a single option
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// Add an option from value and label
    pub fn add_option(mut self, value: T, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::new(value, label));
        self
    }

    /// Set the selected value
    pub fn selected(mut self, selected: T) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set whether the select is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the maximum number of visible items in dropdown
    pub fn max_visible_items(mut self, max: usize) -> Self {
        self.max_visible_items = max;
        self
    }

    /// Build the SelectProps
    pub fn build(self) -> SelectProps<T> {
        SelectProps {
            options: self.options,
            selected: self.selected,
            placeholder: self.placeholder,
            disabled: self.disabled,
            width: self.width,
            max_visible_items: self.max_visible_items,
        }
    }

    /// Build and render as an Element (convenience method)
    pub fn render(self) -> Element {
        Element::component("Select")
            .with_props(self.build())
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Default for SelectBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A single option in the select dropdown
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption<T: Clone + PartialEq + Send + Sync + 'static> {
    /// The value associated with this option
    pub value: T,
    /// Display label for the option
    pub label: String,
    /// Whether this option is disabled
    pub disabled: bool,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> SelectOption<T> {
    /// Create a new select option
    ///
    /// # Arguments
    /// * `value` - The value associated with this option
    /// * `label` - Display label for the option
    ///
    /// # Returns
    /// A new `SelectOption` with the option enabled by default
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// Set whether this option is disabled
    ///
    /// # Arguments
    /// * `disabled` - Whether the option should be disabled
    ///
    /// # Returns
    /// Self for method chaining
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Properties for Select component
#[derive(Clone, Debug, PartialEq)]
pub struct SelectProps<T: Clone + PartialEq + Send + Sync + 'static> {
    /// Available options in the dropdown
    pub options: Vec<SelectOption<T>>,
    /// Currently selected value
    pub selected: Option<T>,
    /// Placeholder text when no option is selected
    pub placeholder: Option<String>,
    /// Whether the select is disabled
    pub disabled: bool,
    /// Maximum number of visible items in dropdown
    pub max_visible_items: usize,
    /// Fixed width of the select component
    pub width: Option<u16>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Default for SelectProps<T> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            placeholder: Some("Select an option...".to_string()),
            disabled: false,
            max_visible_items: 5,
            width: Some(30),
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Props for SelectProps<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Select component
#[derive(Clone, Debug, Default)]
pub struct SelectState {
    /// Whether the dropdown is open
    pub is_open: bool,
    /// Index of the highlighted option
    pub highlighted_index: usize,
    /// Scroll offset for the dropdown
    pub scroll_offset: usize,
    /// Whether the select is focused
    pub is_focused: bool,
}

/// Select dropdown component with full keyboard navigation
pub struct Select<T: Clone + PartialEq + Send + Sync + 'static> {
    _phantom: std::marker::PhantomData<T>,
    state: SelectState,
    on_change: Option<Arc<dyn Fn(T) + Send + Sync>>,
    on_open: Option<Arc<dyn Fn() + Send + Sync>>,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    /// Set the onChange callback for when the selection changes
    ///
    /// # Arguments
    /// * `f` - Callback function that receives the newly selected value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_on_change(mut self, f: impl Fn(T) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Set the onOpen callback for when the dropdown opens
    ///
    /// # Arguments
    /// * `f` - Callback function called when the dropdown opens
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_on_open(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_open = Some(Arc::new(f));
        self
    }

    /// Set the onClose callback for when the dropdown closes
    ///
    /// # Arguments
    /// * `f` - Callback function called when the dropdown closes
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_on_close(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }

    /// Update scroll offset to keep highlighted item visible
    fn update_scroll(&mut self, state: &mut SelectState, max_visible: usize) {
        if state.highlighted_index < state.scroll_offset {
            state.scroll_offset = state.highlighted_index;
        } else if state.highlighted_index >= state.scroll_offset + max_visible {
            state.scroll_offset = state.highlighted_index - max_visible + 1;
        }
    }

    /// Find next selectable (non-disabled) option
    fn find_next_selectable(
        &self,
        options: &[SelectOption<T>],
        start: usize,
        direction: i32,
    ) -> Option<usize> {
        let len = options.len();
        if len == 0 {
            return None;
        }

        let mut index = start as i32;
        for _ in 0..len {
            index = (index + direction).rem_euclid(len as i32);
            if !options[index as usize].disabled {
                return Some(index as usize);
            }
        }
        None
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Component for Select<T> {
    type Props = SelectProps<T>;
    type State = SelectState;

    fn new(_props: Self::Props) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            state: SelectState::default(),
            on_change: None,
            on_open: None,
            on_close: None,
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let width = props.width.unwrap_or(30) as usize;

        // Get display text for selected value
        let display_text = if let Some(ref selected) = props.selected {
            props
                .options
                .iter()
                .find(|opt| opt.value == *selected)
                .map(|opt| opt.label.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        } else if let Some(ref placeholder) = props.placeholder {
            placeholder.clone()
        } else {
            String::new()
        };

        let mut result = String::new();

        // Add state indicator
        if props.disabled {
            result.push_str("🔒 ");
        } else if state.is_open {
            result.push_str("▼ ");
        } else if state.is_focused {
            result.push_str("▶ ");
        } else {
            result.push_str("  ");
        }

        // Truncate display text if needed
        let display = if display_text.len() > width - 2 {
            format!("{}...", &display_text[..width - 5])
        } else {
            display_text
        };

        result.push('[');
        result.push_str(&display);
        result.push(']');

        // Show dropdown if open
        if state.is_open && !props.options.is_empty() {
            result.push('\n');

            // Calculate visible range
            let visible_end =
                (state.scroll_offset + props.max_visible_items).min(props.options.len());

            // Add scroll indicator if needed
            if state.scroll_offset > 0 {
                result.push_str("  ▲ ..more..\n");
            }

            // Render visible options
            for i in state.scroll_offset..visible_end {
                let option = &props.options[i];
                let is_selected = props.selected.as_ref().is_some_and(|s| *s == option.value);
                let is_highlighted = i == state.highlighted_index;

                result.push_str("  ");

                // Add selection/highlight markers
                if is_highlighted && is_selected {
                    result.push_str("▶●");
                } else if is_highlighted {
                    result.push_str("▶ ");
                } else if is_selected {
                    result.push_str(" ●");
                } else {
                    result.push_str("  ");
                }

                // Add option label
                if option.disabled {
                    result.push_str(&format!("({})", option.label));
                } else {
                    result.push_str(&option.label);
                }

                result.push('\n');
            }

            // Add scroll indicator if needed
            if visible_end < props.options.len() {
                result.push_str("  ▼ ..more..\n");
            }
        }

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
            Event::Key(key_event) => {
                if !state.is_focused {
                    return EventResult::Ignored;
                }

                self.handle_key_event(key_event, props, state)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(_) => {
                state.is_focused = true;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut SelectProps<T>,
        state: &mut SelectState,
    ) -> EventResult {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if state.is_open {
                    // Select current option
                    if !props.options.is_empty() && state.highlighted_index < props.options.len() {
                        let option = &props.options[state.highlighted_index];
                        if !option.disabled {
                            props.selected = Some(option.value.clone());
                            if let Some(on_change) = &self.on_change {
                                on_change(option.value.clone());
                            }
                            state.is_open = false;
                            if let Some(on_close) = &self.on_close {
                                on_close();
                            }
                        }
                    }
                } else {
                    // Open dropdown
                    state.is_open = true;

                    // Set initial highlighted index to selected item or first selectable
                    if let Some(ref selected) = props.selected {
                        if let Some(index) =
                            props.options.iter().position(|opt| opt.value == *selected)
                        {
                            state.highlighted_index = index;
                        }
                    } else if let Some(index) = self.find_next_selectable(&props.options, 0, 0) {
                        state.highlighted_index = index;
                    }

                    if let Some(on_open) = &self.on_open {
                        on_open();
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Escape => {
                if state.is_open {
                    state.is_open = false;
                    if let Some(on_close) = &self.on_close {
                        on_close();
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Up => {
                if state.is_open {
                    // Move to previous selectable option
                    if let Some(index) =
                        self.find_next_selectable(&props.options, state.highlighted_index, -1)
                    {
                        state.highlighted_index = index;
                        self.update_scroll(state, props.max_visible_items);
                    }
                } else {
                    // Open dropdown and select previous option
                    state.is_open = true;
                    if let Some(ref selected) = props.selected {
                        if let Some(current_index) =
                            props.options.iter().position(|opt| opt.value == *selected)
                        {
                            if let Some(index) =
                                self.find_next_selectable(&props.options, current_index, -1)
                            {
                                state.highlighted_index = index;
                            }
                        }
                    }
                    if let Some(on_open) = &self.on_open {
                        on_open();
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if state.is_open {
                    // Move to next selectable option
                    if let Some(index) =
                        self.find_next_selectable(&props.options, state.highlighted_index, 1)
                    {
                        state.highlighted_index = index;
                        self.update_scroll(state, props.max_visible_items);
                    }
                } else {
                    // Open dropdown and select next option
                    state.is_open = true;
                    if let Some(ref selected) = props.selected {
                        if let Some(current_index) =
                            props.options.iter().position(|opt| opt.value == *selected)
                        {
                            if let Some(index) =
                                self.find_next_selectable(&props.options, current_index, 1)
                            {
                                state.highlighted_index = index;
                            }
                        }
                    } else if let Some(index) = self.find_next_selectable(&props.options, 0, 0) {
                        state.highlighted_index = index;
                    }
                    if let Some(on_open) = &self.on_open {
                        on_open();
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                if state.is_open {
                    // Go to first selectable option
                    if let Some(index) = self.find_next_selectable(&props.options, 0, 0) {
                        state.highlighted_index = index;
                        state.scroll_offset = 0;
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::End => {
                if state.is_open {
                    // Go to last selectable option
                    if let Some(index) =
                        self.find_next_selectable(&props.options, props.options.len() - 1, 0)
                    {
                        state.highlighted_index = index;
                        self.update_scroll(state, props.max_visible_items);
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::PageUp => {
                if state.is_open {
                    // Move up by page
                    let new_index = state
                        .highlighted_index
                        .saturating_sub(props.max_visible_items);
                    if let Some(index) = self.find_next_selectable(&props.options, new_index, 0) {
                        state.highlighted_index = index;
                        self.update_scroll(state, props.max_visible_items);
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::PageDown => {
                if state.is_open {
                    // Move down by page
                    let new_index = (state.highlighted_index + props.max_visible_items)
                        .min(props.options.len() - 1);
                    if let Some(index) = self.find_next_selectable(&props.options, new_index, 0) {
                        state.highlighted_index = index;
                        self.update_scroll(state, props.max_visible_items);
                    }
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut SelectProps<T>,
        state: &mut SelectState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Click => {
                let click_y = event.position.y() as usize;

                if click_y == 0 {
                    // Clicked on the select box itself
                    if state.is_open {
                        state.is_open = false;
                        if let Some(on_close) = &self.on_close {
                            on_close();
                        }
                    } else {
                        state.is_open = true;
                        state.is_focused = true;
                        if let Some(on_open) = &self.on_open {
                            on_open();
                        }
                    }
                } else if state.is_open && click_y > 0 {
                    // Clicked on an option
                    let option_index = state.scroll_offset + click_y - 1;

                    if option_index < props.options.len() {
                        let option = &props.options[option_index];
                        if !option.disabled {
                            props.selected = Some(option.value.clone());
                            if let Some(on_change) = &self.on_change {
                                on_change(option.value.clone());
                            }
                            state.is_open = false;
                            if let Some(on_close) = &self.on_close {
                                on_close();
                            }
                        }
                    }
                }

                EventResult::Consumed
            }
            MouseEventKind::Wheel => {
                if state.is_open {
                    // Handle mouse wheel scrolling
                    // Most terminals report wheel as button 4 (up) and 5 (down)
                    match event.button {
                        crate::event::types::MouseButton::Back => {
                            // Button 4 - Wheel up
                            if state.scroll_offset > 0 {
                                state.scroll_offset -= 1;
                                EventResult::Consumed
                            } else {
                                EventResult::Ignored
                            }
                        }
                        crate::event::types::MouseButton::Forward => {
                            // Button 5 - Wheel down
                            let max_scroll =
                                props.options.len().saturating_sub(props.max_visible_items);
                            if state.scroll_offset < max_scroll {
                                state.scroll_offset += 1;
                                EventResult::Consumed
                            } else {
                                EventResult::Ignored
                            }
                        }
                        _ => {
                            // Unknown wheel direction, try to be smart about it
                            // If we're at top, assume down; if at bottom, assume up
                            let max_scroll =
                                props.options.len().saturating_sub(props.max_visible_items);
                            if state.scroll_offset == 0 && max_scroll > 0 {
                                state.scroll_offset += 1;
                                EventResult::Consumed
                            } else if state.scroll_offset > 0 {
                                state.scroll_offset -= 1;
                                EventResult::Consumed
                            } else {
                                EventResult::Ignored
                            }
                        }
                    }
                } else {
                    EventResult::Ignored
                }
            }
            MouseEventKind::Move => {
                // Track mouse hover for visual feedback when over options
                if state.is_open {
                    let hover_y = event.position.y() as usize;
                    if hover_y > 0 {
                        let option_index = state.scroll_offset + hover_y - 1;
                        if option_index < props.options.len()
                            && !props.options[option_index].disabled
                        {
                            state.highlighted_index = option_index;
                            EventResult::Consumed
                        } else {
                            EventResult::Ignored
                        }
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}
