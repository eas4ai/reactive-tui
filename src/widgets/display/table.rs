use super::{Alignment, Border, DisplaySize, ScrollState};
use crate::component::{Component, Element, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers};

use std::collections::HashMap;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

pub(in crate::widgets) mod border;
mod live;
type RowActionCallback = dyn Fn(usize, &str) + Send + Sync;

pub(super) fn validation_error(props: &TableProps) -> Option<String> {
    let mut columns = std::collections::HashSet::new();
    for column in &props.columns {
        if !columns.insert(&column.key) {
            return Some(format!("Duplicate table column key: {}", column.key));
        }
        if column
            .max_width
            .is_some_and(|maximum| maximum < column.min_width)
        {
            return Some(format!(
                "Column {} minimum width exceeds its maximum",
                column.key
            ));
        }
        if matches!(column.width,DisplaySize::Percent(value)|DisplaySize::Flex(value) if !value.is_finite() || value<0.0)
        {
            return Some(format!(
                "Column {} width must be finite and nonnegative",
                column.key
            ));
        }
    }
    if props
        .sort_column
        .is_some_and(|index| index >= props.columns.len())
    {
        return Some("Table sort column is out of range".into());
    }
    let mut rows = std::collections::HashSet::new();
    for row in &props.rows {
        if !rows.insert(&row.id) {
            return Some(format!("Duplicate table row ID: {}", row.id));
        }
    }
    None
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
        let mut widths = Vec::with_capacity(props.columns.len());
        let mut flexible = Vec::new();
        let mut weight = 0.0f64;
        for (index, column) in props.columns.iter().enumerate() {
            let requested = match column.width {
                DisplaySize::Fixed(width) => width,
                DisplaySize::Percent(percent) => {
                    (available_width as f32 * percent.max(0.0) / 100.0) as u16
                }
                DisplaySize::Auto => Self::calculate_column_content_width(props, index),
                DisplaySize::Flex(factor) => {
                    if factor.is_finite() && factor > 0.0 {
                        flexible.push((index, factor as f64));
                        weight += factor as f64;
                    }
                    column.min_width
                }
            };
            widths.push(
                requested
                    .max(column.min_width)
                    .min(column.max_width.unwrap_or(u16::MAX).max(column.min_width)),
            );
        }
        let occupied: usize = widths.iter().map(|&width| width as usize).sum();
        let mut extra = (available_width as usize).saturating_sub(occupied);
        for (index, factor) in flexible {
            let share = (extra as f64 * factor / weight).round() as usize;
            let cap = props.columns[index]
                .max_width
                .unwrap_or(u16::MAX)
                .max(props.columns[index].min_width);
            let added = share.min(cap.saturating_sub(widths[index]) as usize);
            widths[index] = widths[index].saturating_add(added as u16);
            extra -= added;
            weight -= factor;
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
        if !props.selectable || props.rows.is_empty() || props.columns.is_empty() {
            return EventResult::Ignored;
        }

        let order: Vec<_> = self
            .sort_rows(props, state)
            .into_iter()
            .filter(|&index| props.rows[index].selectable)
            .collect();
        if order.is_empty() {
            return EventResult::Ignored;
        }
        let current = state
            .selected_row
            .or_else(|| state.selected_rows.first().copied());
        let position = current.and_then(|row| order.iter().position(|&index| index == row));
        let last = order.len() - 1;
        let page = state.scroll_state.viewport_height.max(1) as usize;
        let target = match key {
            KeyCode::Down => Some(position.map_or(0, |index| (index + 1) % order.len())),
            KeyCode::Up => {
                Some(position.map_or(last, |index| if index == 0 { last } else { index - 1 }))
            }
            KeyCode::Home => Some(0),
            KeyCode::End => Some(last),
            KeyCode::PageDown => {
                Some(position.map_or(0, |index| index.saturating_add(page).min(last)))
            }
            KeyCode::PageUp => Some(position.map_or(last, |index| index.saturating_sub(page))),
            KeyCode::Enter => {
                let Some(row) = position.map(|index| order[index]) else {
                    return EventResult::Ignored;
                };
                if let Some(callback) = &props.on_row_action {
                    callback(row, "select");
                }
                return EventResult::Consumed;
            }
            KeyCode::Char(' ') if props.multi_select => {
                let Some(row) = position.map(|index| order[index]) else {
                    return EventResult::Ignored;
                };
                self.toggle_row_selection(props, state, row);
                return EventResult::Consumed;
            }
            KeyCode::Char('a') if modifiers.ctrl && props.multi_select => {
                let previous = state.selected_rows.clone();
                state.selected_rows = order;
                if previous != state.selected_rows {
                    if let Some(callback) = &props.on_multi_select {
                        callback(state.selected_rows.clone());
                    }
                }
                return EventResult::Consumed;
            }
            _ => None,
        };
        if let Some(target) = target {
            self.select_row(props, state, order[target], modifiers.shift);
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn select_row(&self, props: &TableProps, state: &mut TableState, row: usize, extend: bool) {
        if !props.selectable || !props.rows.get(row).is_some_and(|row| row.selectable) {
            return;
        }
        let previous = state.selected_rows.clone();
        let previous_row = state.selected_row;
        state.selected_row = Some(row);
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
            if previous != state.selected_rows {
                if let Some(callback) = &props.on_multi_select {
                    callback(state.selected_rows.clone());
                }
            }
        } else if previous_row != state.selected_row {
            if let Some(callback) = &props.on_select {
                callback(Some(row));
            }
        }
        if props.scrollable {
            if let Some(position) = self
                .sort_rows(props, state)
                .iter()
                .position(|&index| index == row)
            {
                self.scroll_to_row(state, position);
            }
        }
    }

    fn toggle_row_selection(&self, props: &TableProps, state: &mut TableState, row: usize) {
        if !props.selectable
            || !props.multi_select
            || !props.rows.get(row).is_some_and(|row| row.selectable)
        {
            return;
        }
        state.selected_row = Some(row);
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
        let row_y = u16::try_from(row).unwrap_or(u16::MAX);
        let viewport_top = state.scroll_state.offset_y;
        let viewport_bottom = viewport_top.saturating_add(state.scroll_state.viewport_height);

        if row_y < viewport_top {
            state.scroll_state.offset_y = row_y;
        } else if row_y >= viewport_bottom {
            state.scroll_state.offset_y =
                row_y.saturating_sub(state.scroll_state.viewport_height.saturating_sub(1));
        }
    }

    /// Content widths are terminal display columns, including two cells of spacing.
    fn calculate_column_content_width(props: &TableProps, index: usize) -> u16 {
        let Some(column) = props.columns.get(index) else {
            return 0;
        };
        let width = std::iter::once(if props.show_header {
            column.title.as_str()
        } else {
            ""
        })
        .chain(
            props
                .rows
                .iter()
                .filter_map(|row| row.cells.get(&column.key).map(|cell| cell.content.as_str())),
        )
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
        u16::try_from(width.saturating_add(2)).unwrap_or(u16::MAX)
    }
}

impl Component for Table {
    type Props = TableProps;
    type State = TableState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = TableState {
            selected_row: props.selected_row.filter(|&i| {
                props.selectable && props.rows.get(i).is_some_and(|row| row.selectable)
            }),
            sort_column: props.sort_column.filter(|&i| i < props.columns.len()),
            sort_ascending: props.sort_ascending,
            ..Default::default()
        };
        state.selected_rows = state.selected_row.into_iter().collect();
        self.update(props, &mut state);
        state
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveTable>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
            sort_request: None,
            cursor_change: None,
            controlled_selection: false,
            window: None,
        })
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if validation_error(props).is_some() {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(key) if key.kind != crate::event::types::KeyEventKind::Release => {
                self.handle_key_navigation(key.code.clone(), key.modifiers, props, state)
            }
            Event::Focus(focus) => {
                state.focused = focus.kind == crate::event::types::FocusEventKind::Gained;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let previous = (
            state.selected_row,
            state.selected_rows.clone(),
            state.visible_rows.clone(),
            state.column_widths.clone(),
        );
        state
            .selected_rows
            .retain(|&i| props.selectable && props.rows.get(i).is_some_and(|row| row.selectable));
        state.selected_row = state
            .selected_row
            .filter(|&i| props.selectable && props.rows.get(i).is_some_and(|row| row.selectable));
        if state.column_widths.len() != props.columns.len() {
            state.column_widths =
                self.calculate_column_widths(props, state.scroll_state.viewport_width);
        }
        state.visible_rows = self.sort_rows(props, state);
        previous
            != (
                state.selected_row,
                state.selected_rows.clone(),
                state.visible_rows.clone(),
                state.column_widths.clone(),
            )
    }
}

impl Default for Table {
    fn default() -> Self {
        Self
    }
}

/// The advanced table owns filtering, page selection and sort priorities; its
/// child owns measured cell layout and input within the current row view.
pub(super) fn data_view(
    config: TableProps,
    seed: TableState,
    sort_request: Arc<dyn Fn(usize, bool) + Send + Sync>,
    cursor_change: Arc<dyn Fn(Option<usize>) + Send + Sync>,
    window: Option<(usize, usize, u64)>,
) -> Element {
    Element::typed::<live::LiveTable>(live::LiveProps {
        config,
        seed,
        sort_request: Some(sort_request),
        cursor_change: Some(cursor_change),
        controlled_selection: true,
        window,
    })
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
        let table = Table;
        let props = create_test_props();
        let state = TableState::default();
        let element = table.render(&props, &state);
        assert!(matches!(
            element.element_type,
            crate::component::ElementType::Component(_)
        ));
    }

    #[test]
    fn test_column_width_calculation() {
        let table = Table;
        let props = create_test_props();
        let widths = table.calculate_column_widths(&props, 200);

        assert_eq!(widths.len(), 3);
        assert!(widths.iter().sum::<u16>() <= 200);
        assert!(widths[1] == 50); // Fixed width column
    }

    #[test]
    fn test_row_selection() {
        let table = Table;
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
        let table = Table;
        let props = create_test_props();
        // Sort by first column (name).
        let state = TableState {
            sort_column: Some(0),
            sort_ascending: true,
            ..TableState::default()
        };

        let sorted_indices = table.sort_rows(&props, &state);
        assert_eq!(sorted_indices.len(), 3);

        // Check that Bob Johnson (index 2) comes first alphabetically
        assert_eq!(sorted_indices[0], 2);
    }

    #[test]
    fn test_keyboard_navigation() {
        let table = Table;
        let props = create_test_props();
        let mut state = TableState::default();

        // Test down arrow
        let result =
            table.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(state.selected_rows, vec![0]);

        // Test up arrow (should wrap to last)
        let result =
            table.handle_key_navigation(KeyCode::Up, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(state.selected_rows, vec![2]);
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
    fn sorted_navigation_skips_disabled_rows_and_reports_source_indices_once() {
        use std::sync::Mutex;
        let selections = Arc::new(Mutex::new(Vec::new()));
        let sink = selections.clone();
        let mut props = create_test_props();
        props.rows[1].selectable = false;
        props.on_select = Some(Arc::new(move |row| sink.lock().unwrap().push(row)));
        let mut state = TableState {
            sort_column: Some(0),
            sort_ascending: true,
            ..Default::default()
        };
        let table = Table;
        table.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(state.selected_row, Some(2)); // Bob is first after sorting.
        table.handle_key_navigation(KeyCode::Home, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(*selections.lock().unwrap(), vec![Some(2)]);
        table.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state);
        assert_eq!(state.selected_row, Some(0)); // Skip disabled Jane.
        assert_eq!(state.selected_rows, vec![0]);
        table.select_row(&props, &mut state, 1, false);
        assert_eq!(*selections.lock().unwrap(), vec![Some(2), Some(0)]);
        props.multi_select = true;
        table.handle_key_navigation(
            KeyCode::Char('a'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::empty()
            },
            &props,
            &mut state,
        );
        assert_eq!(state.selected_rows, vec![2, 0]);
        props.selectable = false;
        assert_eq!(
            table.handle_key_navigation(KeyCode::Down, KeyModifiers::empty(), &props, &mut state),
            EventResult::Ignored
        );
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
        let mut table = Table;
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
        let mut table = Table;
        let props = create_test_props();
        let mut state = TableState::default();

        table.update(&props, &mut state);

        // Should have column widths set
        assert_eq!(state.column_widths.len(), props.columns.len());

        // Should have visible rows calculated
        assert!(!state.visible_rows.is_empty());
    }
}
