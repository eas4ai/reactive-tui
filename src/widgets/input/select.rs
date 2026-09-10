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
        Element::typed::<Select<T>>(self.build())
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
    viewport: Option<crate::component::LayoutInfo>,
    search: String,
    search_updated: Option<std::time::Instant>,
    _phantom: std::marker::PhantomData<T>,
    state: SelectState,
    on_change: Option<Arc<dyn Fn(T) + Send + Sync>>,
    on_open: Option<Arc<dyn Fn() + Send + Sync>>,
    on_close: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    pub(crate) fn render_control(
        &self,
        props: &SelectProps<T>,
        state: &SelectState,
        multiple: Option<&[T]>,
    ) -> Element {
        let width = self.view_width(props);

        // Get display text for selected value
        let display_text = if let Some(selected) = multiple.filter(|values| !values.is_empty()) {
            props
                .options
                .iter()
                .filter(|option| selected.contains(&option.value))
                .map(|option| option.label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        } else if let Some(selected) = props.selected.as_ref().filter(|_| multiple.is_none()) {
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
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;
        let available = width.saturating_sub(4);
        let mut used = 0;
        let display: String = display_text
            .graphemes(true)
            .take_while(|part| {
                used += UnicodeWidthStr::width(*part);
                used <= available
            })
            .collect();

        result.push('[');
        result.push_str(&display);
        result.push(']');

        use crate::accessibility::{Node, Role};
        let mut semantic = Node::new(Role::ComboBox);
        semantic.set_value(display_text);
        semantic.set_expanded(state.is_open);
        if multiple.is_some() {
            semantic.inner.set_multiselectable();
        }
        if props.disabled {
            semantic.set_disabled();
        } else {
            semantic.set_clickable();
        }
        let mut root = Element::layout(crate::component::LayoutType::Flex)
            .class(format!(
                "flex flex-col w-{} max-w-full overflow-hidden whitespace-pre",
                props.width.unwrap_or(30)
            ))
            .with_focus(crate::component::FocusProps::input())
            .disabled(props.disabled)
            .with_accessibility(semantic)
            .with_child(Element::text(result).class("shrink-0 aria-hidden"));
        root.metadata
            .accessibility_options
            .get_or_insert_default()
            .click_event = Some(crate::event::CustomEvent::new(
            "reactive_tui.select.toggle",
            Vec::new(),
        ));

        // Show dropdown if open
        if state.is_open && !props.options.is_empty() {
            let mut list_semantic = Node::new(Role::ListBox);
            if multiple.is_some() {
                list_semantic.inner.set_multiselectable();
            }
            let mut list = Element::layout(crate::component::LayoutType::Flex)
                .class("flex flex-col shrink-0")
                .with_accessibility(list_semantic);
            // Every option occupies one row below the header.
            let visible_end = state
                .scroll_offset
                .saturating_add(self.visible_items(props))
                .min(props.options.len());

            // Render visible options
            for i in state.scroll_offset..visible_end {
                let option = &props.options[i];
                let is_selected = multiple.map_or_else(
                    || props.selected.as_ref() == Some(&option.value),
                    |selected| selected.contains(&option.value),
                );
                let is_highlighted = i == state.highlighted_index;
                let mut result = String::from("  ");

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
                if multiple.is_some() {
                    result.push_str(if is_selected { "[✓] " } else { "[ ] " });
                }
                if option.disabled {
                    result.push_str(&format!("({})", option.label));
                } else {
                    result.push_str(&option.label);
                }

                let disabled = props.disabled || option.disabled;
                let mut semantic = Node::new(Role::ListBoxOption);
                semantic.set_label(option.label.clone());
                semantic.set_selected(is_selected);
                if disabled {
                    semantic.set_disabled();
                } else {
                    semantic.set_clickable();
                }
                let mut child = Element::text(result)
                    .with_key(format!("select:{i}"))
                    .class("shrink-0")
                    .with_accessibility(semantic);
                if !disabled {
                    let options = child.metadata.accessibility_options.get_or_insert_default();
                    options.focus = state.is_focused && is_highlighted;
                    options.focus_event = Some(crate::event::CustomEvent::new(
                        "reactive_tui.select.focus",
                        i.to_string().into_bytes(),
                    ));
                }
                list = list.with_child(child);
            }
            root = root.with_child(list);
        }

        root
    }

    fn content_row(&self, position: crate::event::types::Position) -> Option<usize> {
        let (x, y) = (position.x() as usize, position.y() as usize);
        let Some(layout) = self.viewport else {
            return Some(y);
        };
        let x = x.checked_sub(layout.insets[0] as usize)?;
        let y = y.checked_sub(layout.insets[1] as usize)?;
        let (width, height) = layout.content_size();
        (x < width as usize && y < height as usize).then_some(y)
    }

    pub(crate) fn option_at(
        &self,
        position: crate::event::types::Position,
        props: &SelectProps<T>,
        state: &SelectState,
    ) -> Option<usize> {
        let row = self.content_row(position)?.checked_sub(1)?;
        let index = state.scroll_offset.saturating_add(row);
        (state.is_open && row < self.visible_items(props) && index < props.options.len())
            .then_some(index)
    }

    fn visible_items(&self, props: &SelectProps<T>) -> usize {
        let room = self
            .viewport
            .map_or(props.max_visible_items.max(1), |layout| {
                (layout.clip.y + layout.clip.height - layout.transform[5] - layout.insets[1] - 1.0)
                    .max(1.0) as usize
            });
        props.max_visible_items.max(1).min(room)
    }

    fn view_width(&self, props: &SelectProps<T>) -> usize {
        self.viewport
            .map_or(usize::from(props.width.unwrap_or(30)), |layout| {
                layout.content_size().0 as usize
            })
    }

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
        let max_visible = max_visible.max(1);
        if state.highlighted_index < state.scroll_offset {
            state.scroll_offset = state.highlighted_index;
        } else if state.highlighted_index >= state.scroll_offset.saturating_add(max_visible) {
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

        if direction == 0 {
            return (start.min(len - 1)..len)
                .chain(0..start.min(len - 1))
                .find(|&index| !options[index].disabled);
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
            viewport: None,
            search: String::new(),
            search_updated: None,
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

    fn layout(
        &mut self,
        layout: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let previous = (self.view_width(props), self.visible_items(props));
        self.viewport = Some(layout);
        let current = (self.view_width(props), self.visible_items(props));
        state.highlighted_index = state
            .highlighted_index
            .min(props.options.len().saturating_sub(1));
        state.scroll_offset = state
            .scroll_offset
            .min(props.options.len().saturating_sub(current.1));
        self.update_scroll(state, current.1);
        previous != current
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.render_control(props, state, None)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if props.disabled && !matches!(event, Event::Focus(_)) {
            return EventResult::Ignored;
        }

        match event {
            Event::Custom(event) if event.name == "reactive_tui.select.toggle" => {
                if state.is_open {
                    self.set_open(false, state);
                } else {
                    self.open(0, props, state);
                }
                EventResult::Consumed
            }
            Event::Custom(event) if event.name == "reactive_tui.select.focus" => {
                let index = std::str::from_utf8(&event.data)
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok());
                if let Some(index) = index
                    .filter(|&i| state.is_open && props.options.get(i).is_some_and(|o| !o.disabled))
                {
                    state.highlighted_index = index;
                    state.is_focused = true;
                    self.update_scroll(state, self.visible_items(props));
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(key_event) => {
                if !state.is_focused {
                    return EventResult::Ignored;
                }

                self.handle_key_event(key_event, props, state)
            }
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, props, state),
            Event::Focus(event)
                if matches!(
                    event.kind,
                    crate::event::types::FocusEventKind::Gained
                        | crate::event::types::FocusEventKind::Lost
                ) =>
            {
                state.is_focused = event.kind == crate::event::types::FocusEventKind::Gained;
                if !state.is_focused {
                    self.set_open(false, state);
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    fn set_open(&mut self, open: bool, state: &mut SelectState) {
        if state.is_open == open {
            return;
        }
        state.is_open = open;
        self.search.clear();
        self.search_updated = None;
        let callback = if open { &self.on_open } else { &self.on_close };
        if let Some(callback) = callback {
            callback();
        }
    }

    fn open(&mut self, direction: i32, props: &SelectProps<T>, state: &mut SelectState) {
        let selected = props.selected.as_ref().and_then(|selected| {
            props
                .options
                .iter()
                .position(|option| &option.value == selected)
        });
        state.highlighted_index = self
            .find_next_selectable(
                &props.options,
                selected.unwrap_or(0),
                if selected.is_some() { direction } else { 0 },
            )
            .unwrap_or(0);
        self.update_scroll(state, self.visible_items(props));
        self.set_open(true, state);
    }

    fn choose(&mut self, index: usize, props: &mut SelectProps<T>, state: &mut SelectState) {
        let Some(option) = props.options.get(index).filter(|option| !option.disabled) else {
            return;
        };
        if props.selected.as_ref() != Some(&option.value) {
            props.selected = Some(option.value.clone());
            if let Some(callback) = &self.on_change {
                callback(option.value.clone());
            }
        }
        state.highlighted_index = index;
        self.set_open(false, state);
    }

    fn handle_key_event(
        &mut self,
        event: &KeyEvent,
        props: &mut SelectProps<T>,
        state: &mut SelectState,
    ) -> EventResult {
        if !matches!(event.code, KeyCode::Char(_)) {
            self.search.clear();
            self.search_updated = None;
        }
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Space => {
                if state.is_open {
                    self.choose(state.highlighted_index, props, state);
                } else {
                    self.open(0, props, state);
                }
            }
            KeyCode::Escape if state.is_open => self.set_open(false, state),
            KeyCode::Up | KeyCode::Down => {
                let direction = if event.code == KeyCode::Up { -1 } else { 1 };
                if state.is_open {
                    if let Some(index) = self.find_next_selectable(
                        &props.options,
                        state.highlighted_index,
                        direction,
                    ) {
                        state.highlighted_index = index;
                        self.update_scroll(state, self.visible_items(props));
                    }
                } else {
                    self.open(direction, props, state);
                }
            }
            KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown if state.is_open => {
                let index = match event.code {
                    KeyCode::Home => props.options.iter().position(|option| !option.disabled),
                    KeyCode::End => props.options.iter().rposition(|option| !option.disabled),
                    KeyCode::PageUp => {
                        let end = state
                            .highlighted_index
                            .saturating_sub(self.visible_items(props));
                        (0..=end).rev().find(|&index| {
                            props
                                .options
                                .get(index)
                                .is_some_and(|option| !option.disabled)
                        })
                    }
                    _ => {
                        let start = state
                            .highlighted_index
                            .saturating_add(self.visible_items(props))
                            .min(props.options.len().saturating_sub(1));
                        (start..props.options.len()).find(|&index| !props.options[index].disabled)
                    }
                };
                if let Some(index) = index {
                    state.highlighted_index = index;
                    self.update_scroll(state, self.visible_items(props));
                }
            }
            KeyCode::Char(character)
                if !character.is_control()
                    && !event.modifiers.ctrl
                    && !event.modifiers.alt
                    && !event.modifiers.meta =>
            {
                self.search_option(character, std::time::Instant::now(), props, state);
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }

    fn search_option(
        &mut self,
        character: char,
        now: std::time::Instant,
        props: &SelectProps<T>,
        state: &mut SelectState,
    ) {
        if !state.is_open {
            self.open(0, props, state);
        }
        if self.search_updated.is_none_or(|last| {
            now.saturating_duration_since(last) >= std::time::Duration::from_secs(1)
        }) {
            self.search.clear();
        }
        let character: String = character.to_lowercase().collect();
        if self.search.len() + character.len() > 128 {
            return;
        }
        self.search.push_str(&character);
        self.search_updated = Some(now);
        let find = |prefix: &str, start: usize| {
            let length = prefix.chars().count();
            (start..props.options.len()).chain(0..start).find(|&index| {
                let option = &props.options[index];
                !option.disabled
                    && option
                        .label
                        .chars()
                        .flat_map(char::to_lowercase)
                        .take(length)
                        .eq(prefix.chars())
            })
        };
        let mut found = find(
            &self.search,
            state.highlighted_index.min(props.options.len()),
        );
        if found.is_none() {
            self.search.clone_from(&character);
            found = find(
                &self.search,
                state
                    .highlighted_index
                    .saturating_add(1)
                    .min(props.options.len()),
            );
        }
        if let Some(index) = found {
            state.highlighted_index = index;
            self.update_scroll(state, self.visible_items(props));
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: &MouseEvent,
        props: &mut SelectProps<T>,
        state: &mut SelectState,
    ) -> EventResult {
        match event.kind {
            MouseEventKind::Down | MouseEventKind::Click
                if event.button == crate::event::types::MouseButton::Left =>
            {
                let Some(click_y) = self.content_row(event.position) else {
                    return EventResult::Ignored;
                };

                if click_y == 0 {
                    if state.is_open {
                        self.set_open(false, state);
                    } else {
                        state.is_focused = true;
                        self.open(0, props, state);
                    }
                } else if let Some(index) = self.option_at(event.position, props, state) {
                    self.choose(index, props, state);
                }

                EventResult::Consumed
            }
            MouseEventKind::Wheel if state.is_open => {
                use crate::event::types::{MouseButton, WheelDelta};
                let delta = event
                    .wheel
                    .as_ref()
                    .map(|wheel| match wheel.delta {
                        WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. } => y,
                    })
                    .unwrap_or_else(|| match event.button {
                        MouseButton::Back => -1.0,
                        MouseButton::Forward => 1.0,
                        _ => 0.0,
                    });
                if delta == 0.0 {
                    return EventResult::Ignored;
                }
                let visible = self.visible_items(props);
                let amount = delta.abs().ceil().max(1.0) as usize;
                let maximum = props.options.len().saturating_sub(visible);
                state.scroll_offset = if delta < 0.0 {
                    state.scroll_offset.saturating_sub(amount)
                } else {
                    state.scroll_offset.saturating_add(amount).min(maximum)
                };
                let end = state
                    .scroll_offset
                    .saturating_add(visible)
                    .min(props.options.len());
                if !(state.scroll_offset..end).contains(&state.highlighted_index) {
                    if let Some(index) =
                        (state.scroll_offset..end).find(|&index| !props.options[index].disabled)
                    {
                        state.highlighted_index = index;
                    }
                }
                EventResult::Consumed
            }
            MouseEventKind::Move => {
                if let Some(index) = self.option_at(event.position, props, state) {
                    if !props.options[index].disabled {
                        state.highlighted_index = index;
                        return EventResult::Consumed;
                    }
                }
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_expires_without_a_timer_and_bounds_retained_input() {
        let props = SelectBuilder::new()
            .add_option(0, "Alpha")
            .add_option(1, "Bravo")
            .add_option(2, "Rome")
            .add_option(3, "x".repeat(200))
            .build();
        let mut select = Select::new(props.clone());
        let mut state = SelectState::default();
        let now = std::time::Instant::now();
        select.search_option('b', now, &props, &mut state);
        assert_eq!(state.highlighted_index, 1);
        select.search_option(
            'r',
            now + std::time::Duration::from_secs(1),
            &props,
            &mut state,
        );
        assert_eq!(state.highlighted_index, 2);
        for _ in 0..300 {
            select.search_option(
                'x',
                now + std::time::Duration::from_secs(2),
                &props,
                &mut state,
            );
        }
        assert_eq!(state.highlighted_index, 3);
        assert!(select.search.len() <= 128);
        select.set_open(false, &mut state);
        assert!(select.search.is_empty());
        assert!(select.search_updated.is_none());
    }
}
