//! Advanced Data Table Widget
//!
//! Extends the basic table widget with advanced features:
//! - Column filtering with multiple filter types
//! - Pagination for large datasets
//! - Virtual scrolling for performance
//! - Advanced sorting with multiple columns
//! - Export capabilities
//! - Column visibility controls

use super::table::{TableProps, TableColumn, TableRow, TableCell};
use crate::component::{Component, Element, Props};
use crate::prelude::LayoutType;
use std::collections::HashMap;
use std::sync::Arc;

/// Filter types for table columns
#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    /// Text contains filter
    Contains(String),
    /// Exact text match
    Equals(String),
    /// Numeric range filter
    Range(f64, f64),
    /// Date range filter
    DateRange(String, String), // ISO date strings
    /// Boolean filter
    Boolean(bool),
}

/// Column filter configuration
#[derive(Debug, Clone)]
pub struct ColumnFilter {
    /// Column key to filter on
    pub column_key: String,
    /// Filter type and value
    pub filter_type: FilterType,
    /// Whether filter is active
    pub active: bool,
}

/// Pagination configuration
#[derive(Debug, Clone, PartialEq)]
pub struct PaginationConfig {
    /// Current page (0-based)
    pub current_page: usize,
    /// Number of rows per page
    pub page_size: usize,
    /// Total number of rows
    pub total_rows: usize,
    /// Whether pagination is enabled
    pub enabled: bool,
}

impl PaginationConfig {
    /// Calculate total number of pages
    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            1
        } else {
            (self.total_rows + self.page_size - 1) / self.page_size
        }
    }

    /// Get the start index for current page
    pub fn start_index(&self) -> usize {
        self.current_page * self.page_size
    }

    /// Get the end index for current page
    pub fn end_index(&self) -> usize {
        std::cmp::min(self.start_index() + self.page_size, self.total_rows)
    }

    /// Check if there's a next page
    pub fn has_next_page(&self) -> bool {
        self.current_page < self.total_pages().saturating_sub(1)
    }

    /// Check if there's a previous page
    pub fn has_previous_page(&self) -> bool {
        self.current_page > 0
    }
}

/// Virtual scrolling configuration
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualScrollConfig {
    /// Height of each row in pixels
    pub row_height: u16,
    /// Number of rows to render outside visible area (buffer)
    pub overscan: usize,
    /// Total number of rows in dataset
    pub total_rows: usize,
    /// Current scroll offset
    pub scroll_offset: usize,
    /// Visible area height
    pub viewport_height: u16,
    /// Whether virtual scrolling is enabled
    pub enabled: bool,
}

impl VirtualScrollConfig {
    /// Calculate which rows should be rendered
    pub fn visible_range(&self) -> (usize, usize) {
        if !self.enabled {
            return (0, self.total_rows);
        }

        let visible_rows = (self.viewport_height / self.row_height) as usize;
        let start = self.scroll_offset.saturating_sub(self.overscan);
        let end = std::cmp::min(
            self.scroll_offset + visible_rows + self.overscan,
            self.total_rows,
        );
        (start, end)
    }
}

/// Props for the Advanced Data Table component
#[derive(Clone)]
pub struct DataTableProps {
    /// Base table properties
    pub table_props: TableProps,
    
    /// Column filters
    pub filters: Vec<ColumnFilter>,
    
    /// Pagination configuration
    pub pagination: PaginationConfig,
    
    /// Virtual scrolling configuration
    pub virtual_scroll: VirtualScrollConfig,
    
    /// Whether to show filter controls
    pub show_filters: bool,
    
    /// Whether to show pagination controls
    pub show_pagination: bool,
    
    /// Whether columns can be hidden/shown
    pub column_visibility_control: bool,
    
    /// Hidden column keys
    pub hidden_columns: Vec<String>,
    
    /// Search query for global search
    pub search_query: Option<String>,
    
    /// Whether to enable global search
    pub searchable: bool,
    
    /// Export options
    pub exportable: bool,
    
    /// Callback for filter changes
    pub on_filter_change: Option<Arc<dyn Fn(Vec<ColumnFilter>) + Send + Sync>>,
    
    /// Callback for pagination changes
    pub on_page_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    
    /// Callback for search changes
    pub on_search_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    
    /// Callback for column visibility changes
    pub on_column_visibility_change: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    
    /// Callback for export requests
    pub on_export: Option<Arc<dyn Fn(&str) + Send + Sync>>, // format: "csv", "json", etc.
}

impl Props for DataTableProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PartialEq for DataTableProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare all fields except callbacks
        self.table_props == other.table_props
            && self.pagination == other.pagination
            && self.virtual_scroll == other.virtual_scroll
            && self.show_filters == other.show_filters
            && self.show_pagination == other.show_pagination
            && self.column_visibility_control == other.column_visibility_control
            && self.hidden_columns == other.hidden_columns
            && self.search_query == other.search_query
            && self.searchable == other.searchable
            && self.exportable == other.exportable
    }
}

impl Default for DataTableProps {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

/// State for the Advanced Data Table component
#[derive(Debug, Clone, PartialEq)]
pub struct DataTableState {
    /// Filtered and paginated rows
    pub filtered_rows: Vec<TableRow>,
    
    /// Current filter input values
    pub filter_inputs: HashMap<String, String>,
    
    /// Whether filter panel is open
    pub filter_panel_open: bool,
    
    /// Whether column visibility panel is open
    pub column_panel_open: bool,
    
    /// Current search input
    pub search_input: String,
    
    /// Virtual scroll state
    pub scroll_position: usize,
    
    /// Loading state for async operations
    pub loading: bool,
    
    /// Error message if any
    pub error_message: Option<String>,
}

impl Default for DataTableState {
    fn default() -> Self {
        Self {
            filtered_rows: Vec::new(),
            filter_inputs: HashMap::new(),
            filter_panel_open: false,
            column_panel_open: false,
            search_input: String::new(),
            scroll_position: 0,
            loading: false,
            error_message: None,
        }
    }
}

/// Advanced Data Table component
pub struct DataTable;

impl Component for DataTable {
    type Props = DataTableProps;
    type State = DataTableState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        // For now, just render the main table with basic controls
        // In a full implementation, we would create a complex layout with all features

        // Apply filters and pagination to get the rows to display
        let display_rows = self.get_display_rows(props, state);

        // Create modified table props with filtered rows
        let mut table_props = props.table_props.clone();
        table_props.rows = display_rows;

        // Filter out hidden columns
        table_props.columns = table_props.columns
            .into_iter()
            .filter(|col| !props.hidden_columns.contains(&col.key))
            .collect();

        // Create a simple layout with the table and basic info
        let info_text = if props.pagination.enabled {
            format!(
                "Advanced Table: Page {} of {} ({} total rows)",
                props.pagination.current_page + 1,
                props.pagination.total_pages(),
                props.pagination.total_rows
            )
        } else {
            format!("Advanced Table: {} rows", table_props.rows.len())
        };

        // Return a layout with info and table
        Element::layout(LayoutType::Flex)
            .child(Element::text(info_text))
            .child(Element::component_with_props("Table", table_props))
    }
}

impl DataTable {



    /// Get the rows to display after applying filters and pagination
    fn get_display_rows(&self, props: &DataTableProps, _state: &DataTableState) -> Vec<TableRow> {
        let mut rows = props.table_props.rows.clone();

        // Apply global search
        if let Some(search_query) = &props.search_query {
            if !search_query.is_empty() {
                rows = self.apply_global_search(rows, search_query);
            }
        }

        // Apply column filters
        rows = self.apply_filters(rows, &props.filters);

        // Apply pagination
        if props.pagination.enabled {
            let start = props.pagination.start_index();
            let end = props.pagination.end_index();
            rows = rows.into_iter().skip(start).take(end - start).collect();
        }

        rows
    }

    /// Apply global search across all columns
    fn apply_global_search(&self, rows: Vec<TableRow>, search_query: &str) -> Vec<TableRow> {
        let query = search_query.to_lowercase();

        rows.into_iter()
            .filter(|row| {
                row.cells.values().any(|cell| {
                    cell.content.to_lowercase().contains(&query)
                })
            })
            .collect()
    }

    /// Apply column-specific filters
    fn apply_filters(&self, rows: Vec<TableRow>, filters: &[ColumnFilter]) -> Vec<TableRow> {
        let active_filters: Vec<&ColumnFilter> = filters.iter().filter(|f| f.active).collect();

        if active_filters.is_empty() {
            return rows;
        }

        rows.into_iter()
            .filter(|row| {
                active_filters.iter().all(|filter| {
                    if let Some(cell) = row.cells.get(&filter.column_key) {
                        self.apply_filter_to_cell(cell, &filter.filter_type)
                    } else {
                        false
                    }
                })
            })
            .collect()
    }

    /// Apply a specific filter to a cell
    fn apply_filter_to_cell(&self, cell: &TableCell, filter_type: &FilterType) -> bool {
        match filter_type {
            FilterType::Contains(text) => {
                cell.content.to_lowercase().contains(&text.to_lowercase())
            }
            FilterType::Equals(text) => {
                cell.content.eq_ignore_ascii_case(text)
            }
            FilterType::Range(min, max) => {
                if let Ok(value) = cell.content.parse::<f64>() {
                    value >= *min && value <= *max
                } else {
                    false
                }
            }
            FilterType::DateRange(start, end) => {
                // Simple date comparison (would need proper date parsing in production)
                cell.content >= *start && cell.content <= *end
            }
            FilterType::Boolean(expected) => {
                match cell.content.to_lowercase().as_str() {
                    "true" | "yes" | "1" => *expected,
                    "false" | "no" | "0" => !*expected,
                    _ => false,
                }
            }

        }
    }


}

/// Helper functions for creating advanced table configurations
impl DataTableProps {
    /// Create a new DataTableProps with sensible defaults
    pub fn new(columns: Vec<TableColumn>, rows: Vec<TableRow>) -> Self {
        let total_rows = rows.len();

        Self {
            table_props: TableProps {
                columns,
                rows,
                ..Default::default()
            },
            filters: Vec::new(),
            pagination: PaginationConfig {
                current_page: 0,
                page_size: 25,
                total_rows,
                enabled: total_rows > 25,
            },
            virtual_scroll: VirtualScrollConfig {
                row_height: 32,
                overscan: 5,
                total_rows,
                scroll_offset: 0,
                viewport_height: 400,
                enabled: total_rows > 100,
            },
            show_filters: true,
            show_pagination: true,
            column_visibility_control: true,
            hidden_columns: Vec::new(),
            search_query: None,
            searchable: true,
            exportable: true,
            on_filter_change: None,
            on_page_change: None,
            on_search_change: None,
            on_column_visibility_change: None,
            on_export: None,
        }
    }

    /// Enable/disable specific features
    pub fn with_pagination(mut self, enabled: bool, page_size: usize) -> Self {
        self.pagination.enabled = enabled;
        self.pagination.page_size = page_size;
        self.show_pagination = enabled;
        self
    }

    /// Configure virtual scrolling
    pub fn with_virtual_scroll(mut self, enabled: bool, row_height: u16, viewport_height: u16) -> Self {
        self.virtual_scroll.enabled = enabled;
        self.virtual_scroll.row_height = row_height;
        self.virtual_scroll.viewport_height = viewport_height;
        self
    }

    /// Set feature visibility
    pub fn with_features(mut self, searchable: bool, filterable: bool, exportable: bool) -> Self {
        self.searchable = searchable;
        self.show_filters = filterable;
        self.exportable = exportable;
        self
    }
}
