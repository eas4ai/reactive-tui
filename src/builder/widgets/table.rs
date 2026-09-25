//! DataTable widget builder
//!
//! This module provides the DataTableBuilder for creating data tables with
//! features like pagination, virtual scrolling, filtering, and sorting.

use crate::component::Element;
use crate::widgets::display::table::{TableCell, TableColumn, TableRow};
use crate::widgets::display::{Alignment, ColumnFilter, DataTableProps, DisplaySize, FilterType};
use std::collections::HashMap;

/// Create a DataTable with fluent configuration
pub fn data_table() -> DataTableBuilder {
    DataTableBuilder::new()
}

/// Builder for DataTable components with fluent API
pub struct DataTableBuilder {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    pagination_enabled: bool,
    page_size: usize,
    virtual_scroll_enabled: bool,
    row_height: u16,
    viewport_height: u16,
    searchable: bool,
    filterable: bool,
    exportable: bool,
    hidden_columns: Vec<String>,
    filters: Vec<ColumnFilter>,
    class: Option<String>,
}

impl DataTableBuilder {
    fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            pagination_enabled: true,
            page_size: 25,
            virtual_scroll_enabled: false,
            row_height: 32,
            viewport_height: 400,
            searchable: true,
            filterable: true,
            exportable: true,
            hidden_columns: Vec::new(),
            filters: Vec::new(),
            class: None,
        }
    }

    /// Add a column to the table
    pub fn column(mut self, title: &str, key: &str) -> Self {
        self.columns.push(TableColumn {
            title: title.to_string(),
            key: key.to_string(),
            width: DisplaySize::Flex(1.0),
            alignment: Alignment::Start,
            sortable: true,
            resizable: true,
            min_width: 100,
            max_width: None,
        });
        self
    }

    /// Add a column with custom configuration
    pub fn column_with_config(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Add multiple columns at once
    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Add a row to the table
    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Add multiple rows at once
    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// Add a simple row from key-value pairs
    pub fn simple_row(mut self, data: Vec<(&str, &str)>) -> Self {
        let mut cells = HashMap::new();
        for (key, value) in data {
            cells.insert(
                key.to_string(),
                TableCell {
                    content: value.to_string(),
                    style: None,
                    alignment: None,
                    clickable: false,
                    action: None,
                },
            );
        }

        self.rows.push(TableRow {
            id: format!("row_{}", self.rows.len()),
            cells,
            selectable: true,
            style: None,
            data: HashMap::new(),
        });
        self
    }

    /// Configure pagination
    pub fn pagination(mut self, enabled: bool, page_size: usize) -> Self {
        self.pagination_enabled = enabled;
        self.page_size = page_size;
        self
    }

    /// Configure virtual scrolling
    pub fn virtual_scroll(mut self, enabled: bool, row_height: u16, viewport_height: u16) -> Self {
        self.virtual_scroll_enabled = enabled;
        self.row_height = row_height;
        self.viewport_height = viewport_height;
        self
    }

    /// Configure features
    pub fn features(mut self, searchable: bool, filterable: bool, exportable: bool) -> Self {
        self.searchable = searchable;
        self.filterable = filterable;
        self.exportable = exportable;
        self
    }

    /// Hide specific columns
    pub fn hide_columns(mut self, column_keys: Vec<String>) -> Self {
        self.hidden_columns = column_keys;
        self
    }

    /// Add a filter
    pub fn filter(mut self, column_key: &str, filter_type: FilterType) -> Self {
        self.filters.push(ColumnFilter {
            column_key: column_key.to_string(),
            filter_type,
            active: true,
        });
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the DataTable element
    pub fn build(self) -> Element {
        self.build_with_name("DataTable")
    }

    /// Build the DataTable element with a custom component name
    pub fn build_with_name(self, component_name: &str) -> Element {
        let mut props = DataTableProps::new(self.columns, self.rows)
            .with_pagination(self.pagination_enabled, self.page_size)
            .with_virtual_scroll(
                self.virtual_scroll_enabled,
                self.row_height,
                self.viewport_height,
            )
            .with_features(self.searchable, self.filterable, self.exportable);
        props.hidden_columns = self.hidden_columns;
        props.filters = self.filters;
        props.table_props.sortable = true;

        let mut element = Element::component_with_props(component_name, props);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<DataTableBuilder> for Element {
    fn from(builder: DataTableBuilder) -> Self {
        builder.build()
    }
}
