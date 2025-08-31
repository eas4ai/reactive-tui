use super::{Alignment, Border, DisplaySize, ScrollState};
use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use std::collections::HashMap;
use std::sync::Arc;

type RowActionCallback = dyn Fn(usize, &str) + Send + Sync;

/// Props for the Table component
#[derive(Clone)]
pub struct TableProps {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub selected_row: Option<usize>,
    pub sortable: bool,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub selectable: bool,
    pub multi_select: bool,
    pub border: Border,
    pub header_style: Option<String>,
    pub row_style: Option<String>,
    pub selected_style: Option<String>,
    pub alternate_row_style: Option<String>,
    pub scrollable: bool,
    pub max_height: Option<u16>,
    pub resizable_columns: bool,
    pub show_header: bool,
    pub zebra_striping: bool,
    pub on_select: Option<Arc<dyn Fn(Option<usize>) + Send + Sync>>,
    pub on_multi_select: Option<Arc<dyn Fn(Vec<usize>) + Send + Sync>>,
    pub on_sort: Option<Arc<dyn Fn(usize, bool) + Send + Sync>>,
    pub on_row_action: Option<Arc<RowActionCallback>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub title: String,
    pub key: String,
    pub width: DisplaySize,
    pub alignment: Alignment,
    pub sortable: bool,
    pub resizable: bool,
    pub min_width: u16,
    pub max_width: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub id: String,
    pub cells: HashMap<String, TableCell>,
    pub selectable: bool,
    pub style: Option<String>,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub content: String,
    pub style: Option<String>,
    pub alignment: Option<Alignment>,
    pub clickable: bool,
    pub action: Option<String>,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            selected_row: None,
            sortable: false,
            sort_column: None,
            sort_ascending: true,
            selectable: true,
            multi_select: false,
            border: Border::default(),
            header_style: None,
            row_style: None,
            selected_style: Some("bg-blue fg-white".to_string()),
            alternate_row_style: None,
            scrollable: true,
            max_height: None,
            resizable_columns: false,
            show_header: true,
            zebra_striping: false,
            on_select: None,
            on_multi_select: None,
            on_sort: None,
            on_row_action: None,
        }
    }
}

impl PartialEq for TableProps {
    fn eq(&self, other: &Self) -> bool {
        self.columns == other.columns
            && self.rows == other.rows
            && self.selected_row == other.selected_row
            && self.sortable == other.sortable
            && self.sort_column == other.sort_column
            && self.sort_ascending == other.sort_ascending
            && self.selectable == other.selectable
            && self.multi_select == other.multi_select
            && self.border == other.border
            && self.header_style == other.header_style
            && self.row_style == other.row_style
            && self.selected_style == other.selected_style
            && self.alternate_row_style == other.alternate_row_style
            && self.scrollable == other.scrollable
            && self.max_height == other.max_height
            && self.resizable_columns == other.resizable_columns
            && self.show_header == other.show_header
            && self.zebra_striping == other.zebra_striping
        // Skip callback comparisons as they can't be compared
    }
}

impl Props for TableProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State for the Table component
#[derive(Debug, Clone)]
pub struct TableState {
    pub selected_rows: Vec<usize>,
    pub scroll_state: ScrollState,
    pub column_widths: Vec<u16>,
    pub resizing_column: Option<usize>,
    pub resize_start_x: u16,
    pub focused: bool,
    pub hover_row: Option<usize>,
    pub visible_rows: Vec<usize>,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            selected_rows: Vec::new(),
            scroll_state: ScrollState::new(),
            column_widths: Vec::new(),
            resizing_column: None,
            resize_start_x: 0,
            focused: false,
            hover_row: None,
            visible_rows: Vec::new(),
            sort_column: None,
            sort_ascending: true,
        }
    }
}

// TableState implements Default + Send + Sync automatically

/// Table display component for structured data
pub struct Table;

impl Table {
    /// Create a new Table with default props
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Element {
        Element::component_with_props("Table", TableProps::default())
    }

    /// Create a Table with custom props
    pub fn with_props(props: TableProps) -> Element {
        Element::component_with_props("Table", props)
    }

    /// Builder method for columns
    pub fn with_columns(columns: Vec<TableColumn>) -> TableProps {
        TableProps {
            columns,
            ..Default::default()
        }
    }

    /// Builder method for rows
    pub fn with_rows(mut props: TableProps, rows: Vec<TableRow>) -> TableProps {
        props.rows = rows;
        props
    }

    /// Builder method for selection
    pub fn selectable(mut props: TableProps, selectable: bool) -> TableProps {
        props.selectable = selectable;
        props
    }

    /// Builder method for sorting
    pub fn sortable(mut props: TableProps, sortable: bool) -> TableProps {
        props.sortable = sortable;
        props
    }

    fn calculate_column_widths(&self, props: &TableProps, available_width: u16) -> Vec<u16> {
        let column_count = props.columns.len();
        if column_count == 0 {
            return Vec::new();
        }

        let mut widths = vec![0u16; column_count];
        let mut remaining_width = available_width;
        let mut flex_columns = Vec::new();
        let mut flex_total = 0.0;

        // First pass: calculate fixed and percent widths
        for (i, column) in props.columns.iter().enumerate() {
            match column.width {
                DisplaySize::Fixed(w) => {
                    widths[i] = w.min(remaining_width);
                    remaining_width = remaining_width.saturating_sub(widths[i]);
                }
                DisplaySize::Percent(p) => {
                    let w = (available_width as f32 * p / 100.0) as u16;
                    widths[i] = w.min(remaining_width);
                    remaining_width = remaining_width.saturating_sub(widths[i]);
                }
                DisplaySize::Flex(f) => {
                    flex_columns.push(i);
                    flex_total += f;
                }
                DisplaySize::Auto => {
                    // Auto width based on content - simplified to equal distribution
                    flex_columns.push(i);
                    flex_total += 1.0;
                }
            }
        }

        // Second pass: distribute remaining width among flex columns
        if !flex_columns.is_empty() && remaining_width > 0 {
            for &i in &flex_columns {
                let flex_value = match props.columns[i].width {
                    DisplaySize::Flex(f) => f,
                    DisplaySize::Auto => 1.0,
                    _ => unreachable!(),
                };

                let width = (remaining_width as f32 * flex_value / flex_total) as u16;
                widths[i] = width.max(props.columns[i].min_width);

                if let Some(max) = props.columns[i].max_width {
                    widths[i] = widths[i].min(max);
                }
            }
        }

        widths
    }

    fn sort_rows(&self, props: &TableProps, state: &TableState) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..props.rows.len()).collect();

        if let Some(sort_col) = state.sort_column {
            if sort_col < props.columns.len() {
                let column = &props.columns[sort_col];
                indices.sort_by(|&a, &b| {
                    let cell_a = props.rows[a].cells.get(&column.key);
                    let cell_b = props.rows[b].cells.get(&column.key);

                    let content_a = cell_a.map(|c| c.content.as_str()).unwrap_or("");
                    let content_b = cell_b.map(|c| c.content.as_str()).unwrap_or("");

                    let cmp = content_a.cmp(content_b);
                    if state.sort_ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
        }

        indices
    }

    fn handle_key_navigation(
        &self,
        key: KeyCode,
        modifiers: KeyModifiers,
        props: &TableProps,
        state: &mut TableState,
    ) -> EventResult {
        if !props.selectable || props.rows.is_empty() {
            return EventResult::Ignored;
        }

        let current_selected = state.selected_rows.first().copied().unwrap_or(0);
        let row_count = props.rows.len();

        match key {
            KeyCode::Up => {
                let new_selected = if current_selected > 0 {
                    current_selected - 1
                } else {
                    row_count - 1
                };
                self.select_row(props, state, new_selected, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::Down => {
                let new_selected = (current_selected + 1) % row_count;
                self.select_row(props, state, new_selected, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.select_row(props, state, 0, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.select_row(props, state, row_count - 1, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                let page_size = state.scroll_state.viewport_height.max(1);
                let new_selected = current_selected.saturating_sub(page_size as usize);
                self.select_row(props, state, new_selected, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let page_size = state.scroll_state.viewport_height.max(1);
                let new_selected = (current_selected + page_size as usize).min(row_count - 1);
                self.select_row(props, state, new_selected, modifiers.shift);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(callback) = &props.on_row_action {
                    callback(current_selected, "select");
                }
                EventResult::Consumed
            }
            KeyCode::Char(' ') => {
                if props.multi_select {
                    self.toggle_row_selection(props, state, current_selected);
                }
                EventResult::Consumed
            }
            KeyCode::Char('a') if modifiers.ctrl => {
                if props.multi_select {
                    state.selected_rows = (0..props.rows.len()).collect();
                    if let Some(callback) = &props.on_multi_select {
                        callback(state.selected_rows.clone());
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn select_row(&self, props: &TableProps, state: &mut TableState, row: usize, extend: bool) {
        if row >= props.rows.len() {
            return;
        }

        if props.multi_select && extend {
            if !state.selected_rows.contains(&row) {
                state.selected_rows.push(row);
            }
        } else {
            state.selected_rows.clear();
            state.selected_rows.push(row);
        }

        // Trigger callbacks
        if props.multi_select {
            if let Some(callback) = &props.on_multi_select {
                callback(state.selected_rows.clone());
            }
        } else if let Some(callback) = &props.on_select {
            callback(Some(row));
        }

        // Scroll to selected row if needed
        self.scroll_to_row(state, row);
    }

    fn toggle_row_selection(&self, props: &TableProps, state: &mut TableState, row: usize) {
        if let Some(pos) = state.selected_rows.iter().position(|&r| r == row) {
            state.selected_rows.remove(pos);
        } else {
            state.selected_rows.push(row);
        }

        if let Some(callback) = &props.on_multi_select {
            callback(state.selected_rows.clone());
        }
    }

    fn scroll_to_row(&self, state: &mut TableState, row: usize) {
        let row_y = row as u16;
        let viewport_top = state.scroll_state.offset_y;
        let viewport_bottom = viewport_top + state.scroll_state.viewport_height;

        if row_y < viewport_top {
            state.scroll_state.offset_y = row_y;
        } else if row_y >= viewport_bottom {
            state.scroll_state.offset_y =
                row_y.saturating_sub(state.scroll_state.viewport_height.saturating_sub(1));
        }
    }
}

impl Component for Table {
    type Props = TableProps;
    type State = TableState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        // Hook functionality would be handled by runtime

        // Calculate layout
        let available_width = 80; // Would be provided by layout system
        let column_widths = self.calculate_column_widths(props, available_width);

        // Sort rows
        let sorted_indices = self.sort_rows(props, state);

        let mut children = Vec::new();

        // Header row
        if props.show_header {
            let mut header_cells = Vec::new();

            for (column_index, (column, _width)) in
                props.columns.iter().zip(column_widths.iter()).enumerate()
            {
                let mut header_text = column.title.clone();

                // Add sort indicators
                if let Some(sort_col) = state.sort_column {
                    if sort_col == column_index {
                        header_text.push_str(if state.sort_ascending { " ↑" } else { " ↓" });
                    }
                }

                let cell_class = props
                    .header_style
                    .clone()
                    .unwrap_or_else(|| "font-bold".to_string());
                header_cells.push(
                    Element::text(&header_text)
                        .with_class(&cell_class)
                        .with_key(format!("header-{}", column.key)),
                );
            }

            children.push(
                Element::layout(LayoutType::Flex)
                    .with_class("flex-row")
                    .with_children(header_cells)
                    .with_key("header"),
            );
        }

        // Data rows
        for (visible_index, &row_index) in sorted_indices.iter().enumerate() {
            if let Some(row) = props.rows.get(row_index) {
                let is_selected = state.selected_rows.contains(&row_index);
                let _is_hovered = state.hover_row == Some(row_index);

                let mut row_class = String::new();

                // Apply row styling
                if is_selected {
                    if let Some(style) = &props.selected_style {
                        row_class.push_str(style);
                    }
                } else if props.zebra_striping && visible_index % 2 == 1 {
                    if let Some(style) = &props.alternate_row_style {
                        row_class.push(' ');
                        row_class.push_str(style);
                    }
                } else if let Some(style) = &props.row_style {
                    row_class.push(' ');
                    row_class.push_str(style);
                }

                // Custom row style
                if let Some(style) = &row.style {
                    row_class.push(' ');
                    row_class.push_str(style);
                }

                let mut row_cells = Vec::new();

                for (column, _width) in props.columns.iter().zip(column_widths.iter()) {
                    let cell = row.cells.get(&column.key);
                    let content = cell.map(|c| c.content.as_str()).unwrap_or("").to_string();

                    let mut cell_class = String::new();
                    if let Some(cell) = cell {
                        if let Some(style) = &cell.style {
                            cell_class.push_str(style);
                        }
                    }

                    row_cells.push(
                        Element::text(&content)
                            .with_class(&cell_class)
                            .with_key(format!("cell-{}-{}", row_index, column.key)),
                    );
                }

                children.push(
                    Element::layout(LayoutType::Flex)
                        .with_class(format!("flex-row {row_class}"))
                        .with_children(row_cells)
                        .with_key(format!("row-{row_index}")),
                );
            }
        }

        let container_class = if props.border.enabled { "border" } else { "" };

        Element::layout(LayoutType::Flex)
            .with_class(format!("flex-col {container_class}"))
            .with_children(children)
            .with_key("table-container")
    }

    fn handle_event(
        &mut self,
        event: &crate::event::types::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key_navigation(
                key_event.code.clone(),
                key_event.modifiers,
                props,
                state,
            ),
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    MouseEvent {
                        kind: MouseEventKind::Click,
                        ..
                    } => {
                        // Handle row selection and column header clicks
                        // Simplified - would need proper hit testing
                        EventResult::Consumed
                    }
                    MouseEvent {
                        kind: MouseEventKind::Wheel,
                        ..
                    } => {
                        if props.scrollable {
                            // Simplified wheel handling
                            state.scroll_state.scroll_down(3);
                            EventResult::Consumed
                        } else {
                            EventResult::Ignored
                        }
                    }
                    _ => EventResult::Ignored,
                }
            }
            Event::Focus(focus_event) => {
                match focus_event.kind {
                    crate::event::types::FocusEventKind::Gained => state.focused = true,
                    crate::event::types::FocusEventKind::Lost => state.focused = false,
                    _ => {}
                };
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // Update visible rows based on scroll position
        let start_row = state.scroll_state.offset_y as usize;
        // Use a default viewport height if not set (e.g., during testing)
        let viewport_height = if state.scroll_state.viewport_height > 0 {
            state.scroll_state.viewport_height as usize
        } else {
            10 // Default height for testing/initialization
        };
        let end_row = (start_row + viewport_height).min(props.rows.len());
        state.visible_rows = (start_row..end_row).collect();

        // Update column widths if they changed
        if state.column_widths.len() != props.columns.len() {
            state.column_widths = vec![100; props.columns.len()]; // Default width
        }

        // Sync sort state with props
        if let Some(sort_col) = props.sort_column {
            state.sort_column = Some(sort_col);
            state.sort_ascending = props.sort_ascending;
        }

        true // Always re-render for now
    }
}

impl Default for Table {
    fn default() -> Self {
        Self
    }
}

// Helper implementations
impl TableColumn {
    pub fn new(title: &str, key: &str) -> Self {
        Self {
            title: title.to_string(),
            key: key.to_string(),
            width: DisplaySize::Auto,
            alignment: Alignment::Start,
            sortable: true,
            resizable: true,
            min_width: 50,
            max_width: None,
        }
    }

    pub fn with_width(mut self, width: DisplaySize) -> Self {
        self.width = width;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

impl TableRow {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            cells: HashMap::new(),
            selectable: true,
            style: None,
            data: HashMap::new(),
        }
    }

    pub fn with_cell(mut self, key: &str, content: &str) -> Self {
        self.cells.insert(key.to_string(), TableCell::new(content));
        self
    }

    pub fn with_styled_cell(mut self, key: &str, content: &str, style: &str) -> Self {
        self.cells
            .insert(key.to_string(), TableCell::new(content).with_style(style));
        self
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl TableCell {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            style: None,
            alignment: None,
            clickable: false,
            action: None,
        }
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn clickable(mut self, action: &str) -> Self {
        self.clickable = true;
        self.action = Some(action.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_props() -> TableProps {
        let columns = vec![
            TableColumn::new("Name", "name").with_width(DisplaySize::Flex(2.0)),
            TableColumn::new("Age", "age").with_width(DisplaySize::Fixed(50)),
            TableColumn::new("City", "city").with_width(DisplaySize::Percent(30.0)),
        ];

        let rows = vec![
            TableRow::new("1")
                .with_cell("name", "John Doe")
                .with_cell("age", "30")
                .with_cell("city", "New York"),
            TableRow::new("2")
                .with_cell("name", "Jane Smith")
                .with_cell("age", "25")
                .with_cell("city", "Los Angeles"),
            TableRow::new("3")
                .with_cell("name", "Bob Johnson")
                .with_cell("age", "35")
                .with_cell("city", "Chicago"),
        ];

        TableProps {
            columns,
            rows,
            selectable: true,
            sortable: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_table_creation() {
        let table = Table::default();
        let props = create_test_props();
        let state = TableState::default();
        let element = table.render(&props, &state);
        // Table renders as a flex layout container
        assert_eq!(
            element.element_type,
            crate::component::ElementType::Layout(crate::component::LayoutType::Flex)
        );
    }

    #[test]
    fn test_column_width_calculation() {
        let table = Table::default();
        let props = create_test_props();
        let widths = table.calculate_column_widths(&props, 200);

        assert_eq!(widths.len(), 3);
        assert!(widths.iter().sum::<u16>() <= 200);
        assert!(widths[1] == 50); // Fixed width column
    }

    #[test]
    fn test_row_selection() {
        let table = Table::default();
        let props = create_test_props();
        let mut state = TableState::default();

        table.select_row(&props, &mut state, 1, false);
        assert_eq!(state.selected_rows, vec![1]);

        // Multi-select
        let mut multi_props = props.clone();
        multi_props.multi_select = true;

        table.select_row(&multi_props, &mut state, 2, true);
        assert_eq!(state.selected_rows, vec![1, 2]);
    }

    #[test]
    fn test_sorting() {
        let table = Table::default();
        let props = create_test_props();
        let mut state = TableState::default();

        // Sort by first column (name)
        state.sort_column = Some(0);
        state.sort_ascending = true;

        let sorted_indices = table.sort_rows(&props, &state);
        assert_eq!(sorted_indices.len(), 3);

        // Check that Bob Johnson (index 2) comes first alphabetically
        assert_eq!(sorted_indices[0], 2);
    }

    #[test]
    fn test_keyboard_navigation() {
        let table = Table::default();
        let props = create_test_props();
        let mut state = TableState::default();

        // Test down arrow
        let result =
            table.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(state.selected_rows, vec![1]);

        // Test up arrow (should wrap to last)
        let result =
            table.handle_key_navigation(KeyCode::Up, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(state.selected_rows, vec![0]);
    }

    #[test]
    fn test_scrolling() {
        let mut state = TableState::default();
        state.scroll_state.viewport_height = 10;
        state.scroll_state.content_height = 50;

        // Test scroll down
        state.scroll_state.scroll_down(5);
        assert_eq!(state.scroll_state.offset_y, 5);

        // Test scroll up
        state.scroll_state.scroll_up(2);
        assert_eq!(state.scroll_state.offset_y, 3);

        // Test scroll to bounds
        state.scroll_state.scroll_to_bottom();
        assert_eq!(state.scroll_state.offset_y, 40); // content_height - viewport_height

        state.scroll_state.scroll_to_top();
        assert_eq!(state.scroll_state.offset_y, 0);
    }

    #[test]
    fn test_table_builder() {
        let columns = vec![TableColumn::new("Test", "test").with_width(DisplaySize::Fixed(100))];

        let rows = vec![TableRow::new("1").with_cell("test", "Value")];

        let props = Table::with_columns(columns);
        let props = Table::with_rows(props, rows);
        let props = Table::selectable(props, true);
        let props = Table::sortable(props, true);

        assert!(props.selectable);
        assert!(props.sortable);
        assert_eq!(props.columns.len(), 1);
        assert_eq!(props.rows.len(), 1);
    }

    #[test]
    fn test_event_handling() {
        let mut table = Table::default();
        let mut props = create_test_props();
        let mut state = TableState::default();

        // Test key event
        let key_event = Event::Key(crate::event::types::KeyEvent::new(KeyCode::Down));
        let result = table.handle_event(&key_event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);

        // Test focus event
        let focus_event = Event::Focus(crate::event::types::FocusEvent {
            kind: crate::event::types::FocusEventKind::Gained,
            timestamp: std::time::Instant::now(),
        });
        let result = table.handle_event(&focus_event, &mut props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert!(state.focused);
    }

    #[test]
    fn test_component_update() {
        let mut table = Table::default();
        let props = create_test_props();
        let mut state = TableState::default();

        table.update(&props, &mut state);

        // Should have column widths set
        assert_eq!(state.column_widths.len(), props.columns.len());

        // Should have visible rows calculated
        assert!(!state.visible_rows.is_empty());
    }
}
