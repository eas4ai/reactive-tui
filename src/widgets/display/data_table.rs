//! Advanced Data Table Widget
//!
//! Extends the basic table widget with advanced features:
//! - Column filtering with multiple filter types
//! - Pagination for large datasets
//! - Virtual scrolling for performance
//! - Advanced sorting with multiple columns
//! - Export capabilities
//! - Column visibility controls

use super::table::{TableCell, TableColumn, TableProps, TableRow};
use crate::component::{Component, Element, Props};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for export callback to reduce complexity
type ExportCallback = Arc<dyn Fn(&str) + Send + Sync>;

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
#[derive(Debug, Clone, PartialEq)]
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
            self.total_rows.div_ceil(self.page_size)
        }
    }

    /// Get the start index for current page
    pub fn start_index(&self) -> usize {
        self.current_page
            .saturating_mul(self.page_size)
            .min(self.total_rows)
    }

    /// Get the end index for current page
    pub fn end_index(&self) -> usize {
        self.start_index()
            .saturating_add(self.page_size)
            .min(self.total_rows)
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

        // A zero row height cannot describe a viewport. Return an empty range
        // instead of dividing by zero or inventing a row size.
        if self.row_height == 0 {
            return (0, 0);
        }
        let visible_rows = (self.viewport_height as usize).div_ceil(self.row_height as usize);
        let offset = self.scroll_offset.min(self.total_rows);
        let start = offset.saturating_sub(self.overscan);
        let end = offset
            .saturating_add(visible_rows)
            .saturating_add(self.overscan)
            .min(self.total_rows);
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
    pub on_export: Option<ExportCallback>, // format: "csv", "json", etc.
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
            && self.filters == other.filters
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
#[derive(Debug, Clone, PartialEq, Default)]
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

/// Advanced Data Table component
pub struct DataTable;

mod live;

impl Component for DataTable {
    type Props = DataTableProps;
    type State = DataTableState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveDataTable>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }
}

impl DataTable {
    /// Apply a specific filter to a cell
    fn apply_filter_to_cell(&self, cell: &TableCell, filter_type: &FilterType) -> bool {
        match filter_type {
            FilterType::Contains(text) => {
                cell.content.to_lowercase().contains(&text.to_lowercase())
            }
            FilterType::Equals(text) => cell.content.eq_ignore_ascii_case(text),
            FilterType::Range(min, max) => {
                if let Ok(value) = cell.content.parse::<f64>() {
                    value >= *min && value <= *max
                } else {
                    false
                }
            }
            FilterType::DateRange(start, end) => {
                valid_date(start)
                    && valid_date(end)
                    && valid_date(&cell.content)
                    && cell.content >= *start
                    && cell.content <= *end
            }
            FilterType::Boolean(expected) => {
                // Parse the cell content to a boolean value
                let cell_value = match cell.content.to_lowercase().as_str() {
                    "true" | "yes" | "1" => true,
                    "false" | "no" | "0" => false,
                    _ => return false, // Invalid boolean format, exclude from results
                };
                // Return whether the parsed value matches the expected value
                cell_value == *expected
            }
        }
    }
}

/// Validate the ISO calendar-date form accepted by column date filters.
fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, byte)| i != 4 && i != 7 && !byte.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = value[..4].parse().unwrap();
    let month: usize = value[5..7].parse().unwrap();
    let day: u32 = value[8..].parse().unwrap();
    let leap = year.is_multiple_of(400) || year.is_multiple_of(4) && !year.is_multiple_of(100);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    month > 0 && month <= 12 && day > 0 && day <= days[month - 1]
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
    pub fn with_virtual_scroll(
        mut self,
        enabled: bool,
        row_height: u16,
        viewport_height: u16,
    ) -> Self {
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

#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn page_ranges_remain_sliceable_after_data_shrinks_or_indices_overflow() {
        let mut page = PaginationConfig {
            current_page: usize::MAX,
            page_size: 25,
            total_rows: 3,
            enabled: true,
        };
        assert_eq!((page.start_index(), page.end_index()), (3, 3));
        page.current_page = 1;
        page.page_size = usize::MAX;
        assert_eq!((page.start_index(), page.end_index()), (3, 3));
        page.current_page = 0;
        assert_eq!((page.start_index(), page.end_index()), (0, 3));
        page.page_size = 0;
        assert_eq!((page.start_index(), page.end_index()), (0, 0));
    }

    #[test]
    fn virtual_ranges_include_partial_rows_and_handle_invalid_geometry() {
        let mut viewport = VirtualScrollConfig {
            row_height: 3,
            overscan: 0,
            total_rows: 10,
            scroll_offset: 2,
            viewport_height: 4,
            enabled: true,
        };
        assert_eq!(viewport.visible_range(), (2, 4));
        viewport.scroll_offset = usize::MAX;
        viewport.overscan = usize::MAX;
        assert_eq!(viewport.visible_range(), (0, 10));
        viewport.row_height = 0;
        assert_eq!(viewport.visible_range(), (0, 0));
        viewport.enabled = false;
        assert_eq!(viewport.visible_range(), (0, 10));
    }
}
