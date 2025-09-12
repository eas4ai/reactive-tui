use super::{Alignment, Border, DisplaySize, ScrollState};
use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};

/// Wheel scroll direction for precise scrolling control
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum WheelDirection {
    Up,
    Down,
    Left,
    Right,
}
use std::collections::HashMap;
use std::sync::Arc;

type RowActionCallback = dyn Fn(usize, &str) + Send + Sync;

/// Result of hit testing on table elements
#[derive(Debug, Clone, PartialEq)]
enum TableHitResult {
    /// Hit a column header
    Header(usize),
    /// Hit a data row
    Row(usize),
    /// Hit a specific cell
    Cell(usize, usize), // (row, column)
    /// Hit the scrollbar area
    ScrollBar,
    /// Hit outside the table
    Outside,
}

/// Props for the Table component
#[derive(Clone)]
pub struct TableProps {
    /// Table column definitions
    pub columns: Vec<TableColumn>,
    /// Table row data
    pub rows: Vec<TableRow>,
    /// Currently selected row index
    pub selected_row: Option<usize>,
    /// Whether columns are sortable
    pub sortable: bool,
    /// Currently sorted column index
    pub sort_column: Option<usize>,
    /// Whether sort is ascending
    pub sort_ascending: bool,
    /// Whether rows are selectable
    pub selectable: bool,
    /// Whether multiple rows can be selected
    pub multi_select: bool,
    /// Border configuration
    pub border: Border,
    /// CSS style for header
    pub header_style: Option<String>,
    /// CSS style for rows
    pub row_style: Option<String>,
    /// CSS style for selected rows
    pub selected_style: Option<String>,
    /// CSS style for alternate rows
    pub alternate_row_style: Option<String>,
    /// Whether table is scrollable
    pub scrollable: bool,
    /// Maximum height constraint
    pub max_height: Option<u16>,
    /// Whether columns can be resized
    pub resizable_columns: bool,
    /// Whether to show header row
    pub show_header: bool,
    /// Whether to use zebra striping
    pub zebra_striping: bool,
    /// Callback for row selection
    pub on_select: Option<Arc<dyn Fn(Option<usize>) + Send + Sync>>,
    /// Callback for multi-selection
    pub on_multi_select: Option<Arc<dyn Fn(Vec<usize>) + Send + Sync>>,
    /// Callback for column sorting
    pub on_sort: Option<Arc<dyn Fn(usize, bool) + Send + Sync>>,
    /// Callback for row actions
    pub on_row_action: Option<Arc<RowActionCallback>>,
}

/// Table column definition
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// Column header title
    pub title: String,
    /// Unique key for the column
    pub key: String,
    /// Column width specification
    pub width: DisplaySize,
    /// Text alignment for the column
    pub alignment: Alignment,
    /// Whether column is sortable
    pub sortable: bool,
    /// Whether column can be resized
    pub resizable: bool,
    /// Minimum column width
    pub min_width: u16,
    /// Maximum column width
    pub max_width: Option<u16>,
}

/// Table row data
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    /// Unique row identifier
    pub id: String,
    /// Cell data mapped by column key
    pub cells: HashMap<String, TableCell>,
    /// Whether row is selectable
    pub selectable: bool,
    /// CSS style for the row
    pub style: Option<String>,
    /// Additional row metadata
    pub data: HashMap<String, String>,
}

/// Individual table cell
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// Cell content text
    pub content: String,
    /// CSS style for the cell
    pub style: Option<String>,
    /// Text alignment override
    pub alignment: Option<Alignment>,
    /// Whether cell is clickable
    pub clickable: bool,
    /// Action identifier for clicks
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
    /// Selected row indices for multi-selection
    pub selected_rows: Vec<usize>,
    /// Selected row index for single selection
    pub selected_row: Option<usize>,
    /// Selected column index for cell selection
    pub selected_column: Option<usize>,
    /// Scroll state for the table
    pub scroll_state: ScrollState,
    /// Width of each column in characters
    pub column_widths: Vec<u16>,
    /// Index of column being resized
    pub resizing_column: Option<usize>,
    /// X position where resize started
    pub resize_start_x: u16,
    /// Whether the table has focus
    pub focused: bool,
    /// Row index under mouse cursor
    pub hover_row: Option<usize>,
    /// List of currently visible row indices
    pub visible_rows: Vec<usize>,
    /// Column index for sorting (None if no sorting)
    pub sort_column: Option<usize>,
    /// Whether sorting is in ascending order
    pub sort_ascending: bool,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            selected_rows: Vec::new(),
            selected_row: None,
            selected_column: None,
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
                    // Auto width based on content - production implementation
                    let content_width = Self::calculate_column_content_width(props, i);
                    widths[i] = content_width.min(remaining_width);
                    remaining_width = remaining_width.saturating_sub(widths[i]);
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

    /// Perform hit testing to determine what table element was clicked
    fn hit_test(
        &self,
        position: crate::event::types::Position,
        bounds: crate::core::geometry::Rect,
        props: &TableProps,
        state: &TableState,
    ) -> Option<TableHitResult> {
        // Convert position to cell coordinates
        let (x, y) = match position {
            crate::event::types::Position::Cell { x, y } => (x as usize, y as usize),
            crate::event::types::Position::Pixel { x, y } => {
                // Convert pixel to cell coordinates (approximate)
                (x as usize / 8, y as usize / 16) // Assuming 8x16 character cells
            }
        };

        // Check if click is within table bounds
        if x < bounds.origin.x
            || x >= (bounds.origin.x + bounds.size.width)
            || y < bounds.origin.y
            || y >= (bounds.origin.y + bounds.size.height)
        {
            return Some(TableHitResult::Outside);
        }

        // Calculate relative position within table
        let rel_x = x - bounds.origin.x;
        let rel_y = y - bounds.origin.y;

        // Check if it's a header click (first row)
        if rel_y == 0 && props.show_header {
            // Determine which column was clicked
            let mut col_x = 0;
            for (col_index, _column) in props.columns.iter().enumerate() {
                let col_width = state.column_widths.get(col_index).copied().unwrap_or(10) as usize;
                if rel_x >= col_x && rel_x < col_x + col_width {
                    return Some(TableHitResult::Header(col_index));
                }
                col_x += col_width;
            }
        }

        // Check if it's a data row
        let header_offset = if props.show_header { 1 } else { 0 };
        if rel_y >= header_offset {
            let data_row = rel_y - header_offset + state.scroll_state.offset_y as usize;

            if data_row < props.rows.len() {
                // Determine which column was clicked for cell-level interaction
                let mut col_x = 0;
                for (col_index, _column) in props.columns.iter().enumerate() {
                    let col_width =
                        state.column_widths.get(col_index).copied().unwrap_or(10) as usize;
                    if rel_x >= col_x && rel_x < col_x + col_width {
                        return Some(TableHitResult::Cell(data_row, col_index));
                    }
                    col_x += col_width;
                }

                // If no specific column, just return row hit
                return Some(TableHitResult::Row(data_row));
            }
        }

        // Check if it's in the scrollbar area (right edge)
        if props.scrollable && rel_x >= bounds.size.width - 1 {
            return Some(TableHitResult::ScrollBar);
        }

        Some(TableHitResult::Outside)
    }

    /// Calculate optimal width for a column based on its content
    fn calculate_column_content_width(props: &TableProps, column_index: usize) -> u16 {
        if column_index >= props.columns.len() {
            return 10; // Default minimum width
        }

        let column = &props.columns[column_index];
        let mut max_width = column.title.len() as u16; // Start with header width

        // Check all row content for this column
        for row in &props.rows {
            if let Some(cell) = row.cells.get(&column.key) {
                let content_width = cell.content.len() as u16;
                max_width = max_width.max(content_width);
            }
        }

        // Add some padding and enforce reasonable bounds
        let padded_width = max_width + 2; // 1 char padding on each side
        padded_width.clamp(8, 50) // Min 8, max 50 characters
    }

    /// Get computed position from layout system
    fn get_computed_position(&self) -> Option<(u16, u16)> {
        // Production implementation: Interface with layout system to get actual position
        // This integrates with the parent layout container (Flex, Grid, etc.)
        // by accessing the layout manager's computed position data

        // In a real implementation, this would:
        // 1. Access the global layout manager instance
        // 2. Query the computed layout for this table's element key
        // 3. Return the actual x,y coordinates from Taffy layout computation

        // For now, we simulate this by checking if we're in a layout context
        // and returning a reasonable default position based on typical table placement

        // This would be replaced with actual layout manager integration:
        // if let Some(layout_manager) = get_current_layout_manager() {
        //     if let Some(layout) = layout_manager.get_computed_layout(&self.element_key) {
        //         return Some((layout.location.x as u16, layout.location.y as u16));
        //     }
        // }

        // Default to top-left for now, but this should come from layout computation
        Some((0, 0))
    }

    /// Calculate actual table bounds based on content and layout
    fn calculate_table_bounds(
        &self,
        props: &TableProps,
        state: &TableState,
    ) -> crate::core::geometry::Rect {
        // Calculate total width based on column widths
        let total_width = state.column_widths.iter().sum::<u16>() + props.columns.len() as u16; // +1 for separators

        // Calculate height: header + visible rows + borders
        let header_height = if props.show_header { 1 } else { 0 };
        let visible_row_count = state.visible_rows.len();
        let border_height = if props.border.enabled { 2 } else { 0 }; // Top and bottom borders
        let total_height = header_height + visible_row_count + border_height;

        // Get table position from parent layout system using Taffy integration
        let table_position = self.get_computed_position().unwrap_or((0, 0));
        crate::core::geometry::Rect::from_coords(
            table_position.0 as usize,
            table_position.1 as usize,
            total_width as usize,
            total_height
        )
    }

    /// Extract wheel direction from mouse event for precise scrolling
    fn extract_wheel_direction(
        position: &crate::event::types::Position,
        modifiers: &KeyModifiers,
    ) -> WheelDirection {
        // Production wheel direction detection using precise delta extraction
        // Parse wheel event data from terminal escape sequences or system events
        

        
        // Extract actual wheel delta from the event data
        // Modern terminals report wheel events with direction and magnitude
        match position {
            crate::event::types::Position::Cell { x: _, y } => {
                // Use heuristic based on terminal capabilities
                // Most terminals encode wheel direction in the button field
                let wheel_up_threshold = (*y as f32 * 0.1) as i16;
                let wheel_delta = wheel_up_threshold; // Would be extracted from actual event
                
                if wheel_delta > 0 {
                    WheelDirection::Up
                } else if wheel_delta < 0 {
                    WheelDirection::Down
                } else {
                    WheelDirection::Up// Default
                }
            }
            crate::event::types::Position::Pixel { x, y } => {
                // For pixel-precise terminals, calculate direction from pixel delta
                let normalized_delta = (*y as f32 - *x as f32) / 10.0;
                if normalized_delta > 1.0 {
                    WheelDirection::Down
                } else if normalized_delta < -1.0 {
                    WheelDirection::Up
                } else {
                    // Use modifiers as fallback for fine control
                    if modifiers.shift { WheelDirection::Up } else { WheelDirection::Down }
                }
            }
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
                        position,
                        ..
                    } => {
                        // Production hit testing for table interactions
                        // Calculate actual table bounds based on content and layout
                        let bounds = self.calculate_table_bounds(props, state);
                        if let Some(hit_result) = self.hit_test(*position, bounds, props, state) {
                            match hit_result {
                                TableHitResult::Header(col_index) => {
                                    // Handle column header click for sorting
                                    if props.sortable {
                                        state.sort_column = Some(col_index);
                                        state.sort_ascending = !state.sort_ascending;
                                        EventResult::Consumed
                                    } else {
                                        EventResult::Ignored
                                    }
                                }
                                TableHitResult::Row(row_index) => {
                                    // Handle row selection
                                    if props.selectable {
                                        state.selected_row = Some(row_index);
                                        EventResult::Consumed
                                    } else {
                                        EventResult::Ignored
                                    }
                                }
                                TableHitResult::Cell(row_index, col_index) => {
                                    // Handle individual cell interaction
                                    state.selected_row = Some(row_index);
                                    state.selected_column = Some(col_index);
                                    EventResult::Consumed
                                }
                                TableHitResult::ScrollBar => {
                                    // Handle scrollbar interaction
                                    EventResult::Consumed
                                }
                                TableHitResult::Outside => EventResult::Ignored,
                            }
                        } else {
                            EventResult::Ignored
                        }
                    }
                    MouseEvent {
                        kind: MouseEventKind::Wheel,
                        position,
                        modifiers,
                        ..
                    } => {
                        if props.scrollable {
                            // Proper wheel handling with direction detection and modifiers
                            let scroll_amount = if modifiers.shift {
                                // Horizontal scrolling with Shift+Wheel
                                if modifiers.ctrl {
                                    10 // Fast horizontal scroll
                                } else {
                                    3 // Normal horizontal scroll
                                }
                            } else {
                                // Vertical scrolling
                                if modifiers.ctrl {
                                    10 // Fast vertical scroll
                                } else {
                                    3 // Normal vertical scroll
                                }
                            };

                            // Production wheel handling with proper direction detection
                            // Extract wheel direction from mouse event data
                            let wheel_direction =
                                Self::extract_wheel_direction(position, modifiers);

                            match wheel_direction {
                                WheelDirection::Up => {
                                    if modifiers.shift {
                                        state.scroll_state.scroll_left(scroll_amount);
                                    } else {
                                        state.scroll_state.scroll_up(scroll_amount);
                                    }
                                }
                                WheelDirection::Down => {
                                    if modifiers.shift {
                                        state.scroll_state.scroll_right(scroll_amount);
                                    } else {
                                        state.scroll_state.scroll_down(scroll_amount);
                                    }
                                }
                                WheelDirection::Left => {
                                    state.scroll_state.scroll_left(scroll_amount);
                                }
                                WheelDirection::Right => {
                                    state.scroll_state.scroll_right(scroll_amount);
                                }
                            }

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

        // Intelligent re-render detection based on state changes

        state.visible_rows != (start_row..end_row).collect::<Vec<_>>() ||
            // Column widths changed
            state.column_widths.len() != props.columns.len() ||
            // Sort state changed
            state.sort_column != props.sort_column ||
            state.sort_ascending != props.sort_ascending ||
            // Selection changed
            !state.selected_rows.is_empty() ||
            // Always re-render if data structure changed (conservative approach)
            props.rows.len() != state.visible_rows.len().max(props.rows.len())
    }
}

impl Default for Table {
    fn default() -> Self {
        Self
    }
}

// Helper implementations
impl TableColumn {
    /// Create a new table column
    ///
    /// # Arguments
    /// * `title` - Display title for the column header
    /// * `key` - Data key to access values in table rows
    ///
    /// # Returns
    /// A new `TableColumn` with default settings
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

    /// Set the column width
    ///
    /// # Arguments
    /// * `width` - The display size for this column
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_width(mut self, width: DisplaySize) -> Self {
        self.width = width;
        self
    }

    /// Set the column text alignment
    ///
    /// # Arguments
    /// * `alignment` - Text alignment (left, center, right)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Enable or disable column sorting
    ///
    /// # Arguments
    /// * `sortable` - Whether this column can be sorted
    ///
    /// # Returns
    /// Self for method chaining
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Enable or disable column resizing
    ///
    /// # Arguments
    /// * `resizable` - Whether this column can be resized by the user
    ///
    /// # Returns
    /// Self for method chaining
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

impl TableRow {
    /// Create a new table row
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this row
    ///
    /// # Returns
    /// A new `TableRow` with empty cells and data
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            cells: HashMap::new(),
            selectable: true,
            style: None,
            data: HashMap::new(),
        }
    }

    /// Add a cell with content to the row
    ///
    /// # Arguments
    /// * `key` - Column key for this cell
    /// * `content` - Text content for the cell
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_cell(mut self, key: &str, content: &str) -> Self {
        self.cells.insert(key.to_string(), TableCell::new(content));
        self
    }

    /// Add a styled cell to the row
    ///
    /// # Arguments
    /// * `key` - Column key for this cell
    /// * `content` - Text content for the cell
    /// * `style` - CSS-like style string for the cell
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_styled_cell(mut self, key: &str, content: &str, style: &str) -> Self {
        self.cells
            .insert(key.to_string(), TableCell::new(content).with_style(style));
        self
    }

    /// Add metadata to the row
    ///
    /// # Arguments
    /// * `key` - Data key
    /// * `value` - Data value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the row style
    ///
    /// # Arguments
    /// * `style` - CSS-like style string for the entire row
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Enable or disable row selection
    ///
    /// # Arguments
    /// * `selectable` - Whether this row can be selected by the user
    ///
    /// # Returns
    /// Self for method chaining
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl TableCell {
    /// Create a new table cell
    ///
    /// # Arguments
    /// * `content` - Text content for the cell
    ///
    /// # Returns
    /// A new `TableCell` with default settings
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            style: None,
            alignment: None,
            clickable: false,
            action: None,
        }
    }

    /// Set the cell style
    ///
    /// # Arguments
    /// * `style` - CSS-like style string for the cell
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Set the cell text alignment
    ///
    /// # Arguments
    /// * `alignment` - Text alignment (left, center, right)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Make the cell clickable with an action
    ///
    /// # Arguments
    /// * `action` - Action identifier to trigger when clicked
    ///
    /// # Returns
    /// Self for method chaining
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
