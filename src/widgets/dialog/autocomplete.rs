//! Autocomplete Dialog Implementation
//!
//! Provides autocomplete dialogs with HTTP backend support, keyboard navigation,
//! and customizable suggestion rendering.

use super::{
    BaseDialogState, DialogBounds, DialogComponent, DialogEventResult, DialogId, DialogPosition,
    DialogResult, DialogTheme, FocusableElementInfo,
};
use crate::component::Element;
use crate::core::geometry::{Point, Rect, Size};
use crate::event::types::{Event, KeyCode, MouseEventKind};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

mod live;

// Type aliases for complex function pointer types
type OnSelectCallback = Arc<dyn Fn(&str) -> bool + Send + Sync>;
type OnChangeCallback = Arc<dyn Fn(&str) + Send + Sync>;
type SuggestionRenderer = Arc<dyn Fn(&AutocompleteSuggestion) -> Element + Send + Sync>;
type FilterFunction = Arc<dyn Fn(&str, &[String]) -> Vec<String> + Send + Sync>;

/// Configuration options for autocomplete dialogs
#[derive(Clone)]
pub struct AutocompleteDialogOptions {
    /// Dialog title
    pub title: String,
    /// Prompt message
    pub prompt: String,
    /// Autocomplete configuration
    pub autocomplete: AutocompleteConfig,
    /// Dialog size
    pub size: Option<Size>,
    /// Position on screen
    pub position: DialogPosition,
    /// Whether dialog is modal
    pub modal: bool,
    /// Whether backdrop can be clicked to close
    pub backdrop_closable: bool,
    /// Whether escape key closes dialog
    pub escape_closable: bool,
    /// Custom CSS classes
    pub css_classes: HashMap<String, String>,
    /// Callback for suggestion selection
    pub on_select: Option<OnSelectCallback>,
    /// Callback for input change
    pub on_change: Option<OnChangeCallback>,
    /// Callback for dialog close
    pub on_close: Option<Arc<dyn Fn(DialogResult) + Send + Sync>>,
}

impl PartialEq for AutocompleteDialogOptions {
    fn eq(&self, other: &Self) -> bool {
        use crate::widgets::display::overlay::same_callback;
        self.title == other.title
            && self.prompt == other.prompt
            && self.autocomplete == other.autocomplete
            && self.size == other.size
            && self.position == other.position
            && self.modal == other.modal
            && self.backdrop_closable == other.backdrop_closable
            && self.escape_closable == other.escape_closable
            && self.css_classes == other.css_classes
            && same_callback(&self.on_select, &other.on_select)
            && same_callback(&self.on_change, &other.on_change)
            && same_callback(&self.on_close, &other.on_close)
    }
}

impl std::fmt::Debug for AutocompleteDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutocompleteDialogOptions")
            .field("title", &self.title)
            .field("prompt", &self.prompt)
            .field("autocomplete", &self.autocomplete)
            .field("backdrop_closable", &self.backdrop_closable)
            .field("escape_closable", &self.escape_closable)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("on_select", &"<function>")
            .field("on_change", &"<function>")
            .field("on_close", &"<function>")
            .finish()
    }
}

/// Autocomplete configuration
#[derive(Clone)]
pub struct AutocompleteConfig {
    /// Placeholder text for input
    pub placeholder: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Minimum grapheme clusters before triggering autocomplete
    pub min_chars: usize,
    /// Debounce delay for API calls
    pub debounce_delay: Duration,
    /// Maximum number of suggestions to show
    pub max_suggestions: usize,
    /// HTTP endpoint for suggestions
    pub suggestions_url: Option<String>,
    /// HTTP headers for requests
    pub headers: Option<HashMap<String, String>>,
    /// Static suggestions (used if no URL provided)
    pub static_suggestions: Vec<String>,
    /// Custom suggestion renderer
    pub suggestion_renderer: Option<SuggestionRenderer>,
    /// Whether to show suggestion descriptions
    pub show_descriptions: bool,
    /// Whether to highlight matching text
    pub highlight_matches: bool,
    /// Custom filter function for static suggestions
    pub filter_function: Option<FilterFunction>,
}

impl PartialEq for AutocompleteConfig {
    fn eq(&self, other: &Self) -> bool {
        use crate::widgets::display::overlay::same_callback;
        self.placeholder == other.placeholder
            && self.default_value == other.default_value
            && self.min_chars == other.min_chars
            && self.debounce_delay == other.debounce_delay
            && self.max_suggestions == other.max_suggestions
            && self.suggestions_url == other.suggestions_url
            && self.headers == other.headers
            && self.static_suggestions == other.static_suggestions
            && self.show_descriptions == other.show_descriptions
            && self.highlight_matches == other.highlight_matches
            && same_callback(&self.suggestion_renderer, &other.suggestion_renderer)
            && same_callback(&self.filter_function, &other.filter_function)
    }
}

impl std::fmt::Debug for AutocompleteConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutocompleteConfig")
            .field("placeholder", &self.placeholder)
            .field("default_value", &self.default_value)
            .field("min_chars", &self.min_chars)
            .field("debounce_delay", &self.debounce_delay)
            .field("max_suggestions", &self.max_suggestions)
            .field("show_descriptions", &self.show_descriptions)
            .field("suggestions_url", &self.suggestions_url)
            .field("suggestion_renderer", &"<function>")
            .field("filter_function", &"<function>")
            .finish()
    }
}

/// Individual autocomplete suggestion
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct AutocompleteSuggestion {
    /// Suggestion text/value
    pub value: String,
    /// Display text (if different from value)
    pub display: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Optional icon or category
    pub icon: Option<String>,
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Autocomplete dialog implementation
#[derive(Debug)]
pub struct AutocompleteDialog {
    /// Base dialog state
    state: BaseDialogState,
    /// Dialog options
    options: AutocompleteDialogOptions,
    /// Current input value
    input_value: String,
    /// Cursor position in input
    cursor_position: usize,
    /// Current suggestions
    suggestions: Vec<AutocompleteSuggestion>,
    /// Selected suggestion index
    selected_suggestion: Option<usize>,
    /// Whether suggestions are visible
    suggestions_visible: bool,
    /// Whether input is focused
    input_focused: bool,
    /// Dialog bounds
    bounds: DialogBounds,
    /// Last input time for debouncing
    last_input_time: Option<Instant>,
    /// Pending request
    pending_request: bool,
    /// Loading state
    loading: bool,
}

impl AutocompleteDialog {
    /// Create a new autocomplete dialog
    pub fn new(id: DialogId, options: AutocompleteDialogOptions) -> Self {
        let input_value = options
            .autocomplete
            .default_value
            .clone()
            .unwrap_or_default();
        let cursor_position = input_value.len();

        let bounds = DialogBounds {
            size: options.size,
            min_size: Some(Size::new(400, 200)),
            max_size: Some(Size::new(800, 600)),
            position: options.position.clone(),
            resizable: false,
            draggable: true,
            ..Default::default()
        };

        Self {
            state: BaseDialogState::new(id),
            options,
            input_value,
            cursor_position,
            suggestions: Vec::new(),
            selected_suggestion: None,
            suggestions_visible: false,
            input_focused: true,
            bounds,
            last_input_time: None,
            pending_request: false,
            loading: false,
        }
    }

    /// Handle text input
    fn handle_text_input(&mut self, text: &str) -> DialogEventResult {
        self.input_value.insert_str(self.cursor_position, text);
        let inserted_end = self.cursor_position + text.len();
        self.cursor_position = self
            .input_value
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .chain(std::iter::once(self.input_value.len()))
            .find(|index| *index >= inserted_end)
            .unwrap_or(self.input_value.len());
        self.last_input_time = Some(Instant::now());

        // Reset suggestions
        self.selected_suggestion = None;

        // Call change callback
        if let Some(callback) = &self.options.on_change {
            callback(&self.input_value);
        }

        // Trigger autocomplete if minimum characters reached
        if self.input_value.graphemes(true).count() >= self.options.autocomplete.min_chars {
            self.trigger_autocomplete();
        } else {
            self.suggestions.clear();
            self.suggestions_visible = false;
        }

        DialogEventResult::StateChanged
    }

    /// Handle backspace
    fn handle_backspace(&mut self) -> DialogEventResult {
        if self.cursor_position > 0 {
            let start = self.input_value[..self.cursor_position]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(index, _)| index);
            self.input_value
                .replace_range(start..self.cursor_position, "");
            self.cursor_position = start;
            self.last_input_time = Some(Instant::now());

            // Reset suggestions
            self.selected_suggestion = None;

            // Call change callback
            if let Some(callback) = &self.options.on_change {
                callback(&self.input_value);
            }

            // Trigger autocomplete if minimum characters reached
            if self.input_value.graphemes(true).count() >= self.options.autocomplete.min_chars {
                self.trigger_autocomplete();
            } else {
                self.suggestions.clear();
                self.suggestions_visible = false;
            }
        }

        DialogEventResult::StateChanged
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) -> DialogEventResult {
        if self.cursor_position > 0 {
            self.cursor_position = self.input_value[..self.cursor_position]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(index, _)| index);
        }
        DialogEventResult::StateChanged
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) -> DialogEventResult {
        if self.cursor_position < self.input_value.len() {
            self.cursor_position += self.input_value[self.cursor_position..]
                .graphemes(true)
                .next()
                .map_or(0, str::len);
        }
        DialogEventResult::StateChanged
    }

    /// Move selection up in suggestions
    fn move_selection_up(&mut self) -> DialogEventResult {
        if !self.suggestions.is_empty() && self.suggestions_visible {
            self.selected_suggestion = match self.selected_suggestion {
                Some(index) => {
                    if index > 0 {
                        Some(index - 1)
                    } else {
                        Some(self.suggestions.len() - 1)
                    }
                }
                None => Some(self.suggestions.len() - 1),
            };
            DialogEventResult::StateChanged
        } else {
            DialogEventResult::NotHandled
        }
    }

    /// Move selection down in suggestions
    fn move_selection_down(&mut self) -> DialogEventResult {
        if !self.suggestions.is_empty() && self.suggestions_visible {
            self.selected_suggestion = match self.selected_suggestion {
                Some(index) => {
                    if index < self.suggestions.len() - 1 {
                        Some(index + 1)
                    } else {
                        Some(0)
                    }
                }
                None => Some(0),
            };
            DialogEventResult::StateChanged
        } else {
            DialogEventResult::NotHandled
        }
    }

    /// Select current suggestion
    fn select_suggestion(&mut self) -> DialogEventResult {
        if let Some(index) = self.selected_suggestion {
            if let Some(suggestion) = self.suggestions.get(index) {
                if self
                    .options
                    .on_select
                    .as_ref()
                    .is_some_and(|callback| !callback(&suggestion.value))
                {
                    return DialogEventResult::Handled;
                }
                self.input_value = suggestion.value.clone();
                self.cursor_position = self.input_value.len();
                self.suggestions_visible = false;

                return DialogEventResult::Close(DialogResult::Selected(suggestion.value.clone()));
            }
        }
        DialogEventResult::NotHandled
    }

    /// Trigger autocomplete suggestions
    fn trigger_autocomplete(&mut self) {
        if let Some(_url) = &self.options.autocomplete.suggestions_url {
            // Trigger HTTP request
            self.pending_request = true;
            self.loading = true;
            // Would make actual HTTP request here
        } else {
            // Use static suggestions
            self.update_static_suggestions();
        }
    }

    /// Update suggestions from static list
    fn update_static_suggestions(&mut self) {
        self.suggestions = filtered_suggestions(&self.options.autocomplete, &self.input_value);

        self.suggestions_visible = !self.suggestions.is_empty();
        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Submit the dialog
    fn submit(&mut self) -> DialogEventResult {
        // If a suggestion is selected, use it
        if self.suggestions_visible && self.selected_suggestion.is_some() {
            return self.select_suggestion();
        }

        // Otherwise use current input value
        DialogEventResult::Close(DialogResult::Confirmed(Some(self.input_value.clone())))
    }

    /// Get suggestion index at the given screen position
    fn get_suggestion_at_position(&self, x: u16, y: u16) -> Option<usize> {
        if !self.suggestions_visible || self.suggestions.is_empty() {
            return None;
        }

        // Calculate dialog bounds and suggestion list area
        let dialog_bounds = &self.state.bounds;

        // Check if click is within dialog bounds
        if x < dialog_bounds.origin.x as u16
            || x >= (dialog_bounds.origin.x + dialog_bounds.width()) as u16
            || y < dialog_bounds.origin.y as u16
            || y >= (dialog_bounds.origin.y + dialog_bounds.height()) as u16
        {
            return None;
        }

        // Calculate suggestion list area (assuming it starts after title, prompt, and input)
        // This is a simplified calculation - in a real implementation, you'd want to
        // track the exact layout coordinates during rendering
        let suggestions_start_y = dialog_bounds.origin.y as u16 + 4; // Title + prompt + input + padding
        let suggestion_height = 1; // Each suggestion takes 1 line

        if y < suggestions_start_y {
            return None;
        }

        let suggestion_index = (y - suggestions_start_y) as usize / suggestion_height as usize;

        if suggestion_index < self.suggestions.len() {
            Some(suggestion_index)
        } else {
            None
        }
    }
}

impl DialogComponent for AutocompleteDialog {
    fn id(&self) -> DialogId {
        self.state.id
    }

    fn dialog_type(&self) -> &'static str {
        "autocomplete"
    }

    fn render(&self, bounds: Rect, theme: &DialogTheme) -> Element {
        Element::typed::<live::LiveAutocomplete>(live::LiveProps {
            id: self.state.id,
            options: self.options.clone(),
            value: self.input_value.clone(),
            bounds,
            theme: theme.clone(),
        })
    }

    fn handle_event(&mut self, event: &Event) -> DialogEventResult {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Escape => {
                    if self.suggestions_visible {
                        self.suggestions_visible = false;
                        self.selected_suggestion = None;
                        DialogEventResult::StateChanged
                    } else if self.options.escape_closable {
                        DialogEventResult::Close(DialogResult::Cancelled)
                    } else {
                        DialogEventResult::NotHandled
                    }
                }
                KeyCode::Enter => {
                    if self.suggestions_visible && self.selected_suggestion.is_some() {
                        self.select_suggestion()
                    } else {
                        self.submit()
                    }
                }
                KeyCode::Up => self.move_selection_up(),
                KeyCode::Down => self.move_selection_down(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Backspace => self.handle_backspace(),
                KeyCode::Tab => {
                    if self.suggestions_visible && self.selected_suggestion.is_some() {
                        self.select_suggestion()
                    } else {
                        DialogEventResult::NotHandled
                    }
                }
                KeyCode::Char(c) => self.handle_text_input(&c.to_string()),
                _ => DialogEventResult::NotHandled,
            },
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::Down => {
                        // Check if click is outside dialog (backdrop)
                        if self.options.backdrop_closable
                            && !self.state.bounds.contains_point(Point::new(
                                mouse_event.position.x() as usize,
                                mouse_event.position.y() as usize,
                            ))
                        {
                            return DialogEventResult::Close(DialogResult::Cancelled);
                        }

                        // Check if click is on a suggestion item
                        if let Some(clicked_index) = self.get_suggestion_at_position(
                            mouse_event.position.x() as u16,
                            mouse_event.position.y() as u16,
                        ) {
                            // Select the clicked suggestion
                            self.selected_suggestion = Some(clicked_index);

                            self.select_suggestion()
                        } else {
                            DialogEventResult::NotHandled
                        }
                    }
                    _ => DialogEventResult::NotHandled,
                }
            }
            _ => DialogEventResult::NotHandled,
        }
    }

    fn update(&mut self, _delta_time: Duration) -> bool {
        // Handle debounced autocomplete
        if let Some(last_input) = self.last_input_time {
            if last_input.elapsed() >= self.options.autocomplete.debounce_delay
                && !self.pending_request
            {
                if self.input_value.graphemes(true).count() >= self.options.autocomplete.min_chars {
                    self.trigger_autocomplete();
                }
                self.last_input_time = None;
            }
        }

        // Handle animations
        if self.state.is_animating() {
            if let Some(start_time) = self.state.animation_start {
                let elapsed = start_time.elapsed();
                if elapsed >= Duration::from_millis(200) {
                    match self.state.animation_state {
                        super::DialogAnimationState::ShowingIn => {
                            self.state.animation_state = super::DialogAnimationState::Visible;
                        }
                        super::DialogAnimationState::HidingOut => {
                            self.state.animation_state = super::DialogAnimationState::Hidden;
                            self.state.visible = false;
                        }
                        _ => {}
                    }
                    self.state.animation_start = None;
                    return true;
                }
            }
        }

        false
    }

    fn get_bounds(&self) -> DialogBounds {
        self.bounds.clone()
    }

    fn is_modal(&self) -> bool {
        self.options.modal
    }

    fn backdrop_closable(&self) -> bool {
        self.options.backdrop_closable
    }

    fn escape_closable(&self) -> bool {
        self.options.escape_closable
    }

    fn z_index(&self) -> u16 {
        1000
    }

    fn animation(&self) -> Option<super::DialogAnimationConfig> {
        Some(super::DialogAnimationConfig {
            animation_type: super::DialogAnimationType::Fade,
            duration: Duration::from_millis(200),
            easing: super::DialogEasing::EaseInOut,
            animate_in: true,
            animate_out: true,
        })
    }

    fn get_focusable_elements(&self) -> Vec<FocusableElementInfo> {
        let mut elements = vec![super::dialog_component::FocusableElementInfo {
            id: "input".to_string(),
            element_type: super::dialog_component::FocusableElementType::Input,
            tab_index: 0,
            enabled: true,
            bounds: Rect::default(),
            properties: std::collections::HashMap::new(),
        }];

        // Add suggestion elements
        for (index, _) in self.suggestions.iter().enumerate() {
            elements.push(super::dialog_component::FocusableElementInfo {
                id: format!("suggestion-{}", index),
                element_type: super::dialog_component::FocusableElementType::Custom(
                    "suggestion".to_string(),
                ),
                tab_index: index as i32 + 1,
                enabled: true,
                bounds: Rect::default(),
                properties: std::collections::HashMap::new(),
            });
        }

        elements.extend(vec![
            super::dialog_component::FocusableElementInfo {
                id: "cancel".to_string(),
                element_type: super::dialog_component::FocusableElementType::Button,
                tab_index: 100,
                enabled: true,
                bounds: Rect::default(),
                properties: std::collections::HashMap::new(),
            },
            super::dialog_component::FocusableElementInfo {
                id: "ok".to_string(),
                element_type: super::dialog_component::FocusableElementType::Button,
                tab_index: 101,
                enabled: true,
                bounds: Rect::default(),
                properties: std::collections::HashMap::new(),
            },
        ]);

        elements
    }

    fn set_focus(&mut self, element_id: &str) -> bool {
        if element_id == "input" {
            self.input_focused = true;
            true
        } else if element_id.starts_with("suggestion-") {
            if let Some(index_str) = element_id.strip_prefix("suggestion-") {
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < self.suggestions.len() {
                        self.selected_suggestion = Some(index);
                        self.input_focused = false;
                        return true;
                    }
                }
            }
            false
        } else {
            self.input_focused = false;
            true
        }
    }

    fn get_focused_element(&self) -> Option<String> {
        if self.input_focused {
            Some("input".to_string())
        } else {
            self.selected_suggestion
                .map(|index| format!("suggestion-{}", index))
        }
    }

    fn validate(&self) -> super::dialog_component::ValidationResult {
        super::dialog_component::ValidationResult::default() // Autocomplete dialogs don't need validation
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for AutocompleteDialogOptions {
    fn default() -> Self {
        Self {
            title: "Search".to_string(),
            prompt: "Type to search:".to_string(),
            autocomplete: AutocompleteConfig::default(),
            size: None,
            position: DialogPosition::Center,
            modal: true,
            backdrop_closable: true,
            escape_closable: true,
            css_classes: HashMap::new(),
            on_select: None,
            on_change: None,
            on_close: None,
        }
    }
}

impl Default for AutocompleteConfig {
    fn default() -> Self {
        Self {
            placeholder: Some("Type to search...".to_string()),
            default_value: None,
            min_chars: 1,
            debounce_delay: Duration::from_millis(300),
            max_suggestions: 10,
            suggestions_url: None,
            headers: None,
            static_suggestions: Vec::new(),
            suggestion_renderer: None,
            show_descriptions: true,
            highlight_matches: true,
            filter_function: None,
        }
    }
}

fn filtered_suggestions(config: &AutocompleteConfig, query: &str) -> Vec<AutocompleteSuggestion> {
    if query.graphemes(true).count() < config.min_chars {
        return Vec::new();
    }
    let values = if let Some(filter) = &config.filter_function {
        filter(query, &config.static_suggestions)
    } else {
        let query = query.to_lowercase();
        config
            .static_suggestions
            .iter()
            .filter(|value| value.to_lowercase().contains(&query))
            .cloned()
            .collect()
    };
    values
        .into_iter()
        .take(config.max_suggestions)
        .map(|value| AutocompleteSuggestion::simple(&value))
        .collect()
}

impl AutocompleteSuggestion {
    /// Create a simple suggestion with just a value
    pub fn simple(value: &str) -> Self {
        Self {
            value: value.to_string(),
            display: None,
            description: None,
            icon: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a suggestion with value and display text
    pub fn with_display(value: &str, display: &str) -> Self {
        Self {
            value: value.to_string(),
            display: Some(display.to_string()),
            description: None,
            icon: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a suggestion with value, display, and description
    pub fn with_description(value: &str, display: &str, description: &str) -> Self {
        Self {
            value: value.to_string(),
            display: Some(display.to_string()),
            description: Some(description.to_string()),
            icon: None,
            metadata: HashMap::new(),
        }
    }

    /// Add an icon to the suggestion
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Add metadata to the suggestion
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn native_editing_preserves_whole_graphemes() {
        let mut dialog = AutocompleteDialog::new(DialogId::from_u32(1), Default::default());
        dialog.handle_text_input("Ae\u{301}界👩‍💻Z");
        dialog.move_cursor_left();
        dialog.handle_backspace();
        assert_eq!(dialog.input_value, "Ae\u{301}界Z");
        dialog.move_cursor_left();
        dialog.handle_backspace();
        assert_eq!(dialog.input_value, "A界Z");
        dialog.move_cursor_right();
        dialog.handle_text_input("X");
        assert_eq!(dialog.input_value, "A界XZ");
    }

    #[test]
    fn native_minimum_counts_displayed_characters() {
        let mut options = AutocompleteDialogOptions::default();
        options.autocomplete.min_chars = 2;
        options.autocomplete.static_suggestions = vec!["界面".into()];
        let mut dialog = AutocompleteDialog::new(DialogId::from_u32(1), options);
        dialog.handle_text_input("界");
        assert!(dialog.suggestions.is_empty());
        dialog.handle_text_input("面");
        assert_eq!(dialog.suggestions.len(), 1);
    }

    #[test]
    fn native_selection_without_callback_closes() {
        let mut dialog = AutocompleteDialog::new(DialogId::from_u32(1), Default::default());
        dialog.suggestions = vec![AutocompleteSuggestion::simple("chosen")];
        dialog.selected_suggestion = Some(0);
        dialog.suggestions_visible = true;
        assert!(
            matches!(dialog.select_suggestion(), DialogEventResult::Close(DialogResult::Selected(value)) if value == "chosen")
        );
    }

    #[test]
    fn native_submit_honors_selection_veto_once_without_mutation() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let options = AutocompleteDialogOptions {
            on_select: Some(Arc::new(move |_| {
                observed.fetch_add(1, Ordering::SeqCst);
                false
            })),
            ..Default::default()
        };
        let mut dialog = AutocompleteDialog::new(DialogId::from_u32(1), options);
        dialog.handle_text_input("draft");
        dialog.suggestions = vec![AutocompleteSuggestion::simple("chosen")];
        dialog.selected_suggestion = Some(0);
        dialog.suggestions_visible = true;
        assert!(!matches!(dialog.submit(), DialogEventResult::Close(_)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(dialog.input_value, "draft");
        assert!(dialog.suggestions_visible);
    }
}
