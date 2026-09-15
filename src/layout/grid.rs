//! Declarative Grid Layout System
//!
//! A clean, intuitive API for creating grid layouts with CSS integration
//! and dynamic manipulation capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grid scalar value for sizing, similar to CSS units
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridScalar {
    /// Fixed size in terminal cells
    Cells(u16),
    /// Fraction units (like CSS fr) - proportional sizing
    Fr(f32),
    /// Percentage of available space
    Percent(f32),
    /// Auto-size based on content
    Auto,
}

impl GridScalar {
    /// Parse a scalar from string (e.g., "1fr", "50%", "10", "auto")
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        if input == "auto" {
            return Some(GridScalar::Auto);
        }

        if input.ends_with("fr") {
            if let Ok(value) = input.trim_end_matches("fr").parse::<f32>() {
                return Some(GridScalar::Fr(value));
            }
        }

        if input.ends_with('%') {
            if let Ok(value) = input.trim_end_matches('%').parse::<f32>() {
                return Some(GridScalar::Percent(value));
            }
        }

        if let Ok(value) = input.parse::<u16>() {
            return Some(GridScalar::Cells(value));
        }

        None
    }

    /// Convert to CSS grid template value
    pub fn to_css(&self) -> String {
        match self {
            GridScalar::Cells(n) => format!("{}ch", n),
            GridScalar::Fr(f) => format!("{}fr", f),
            GridScalar::Percent(p) => format!("{}%", p),
            GridScalar::Auto => "auto".to_string(),
        }
    }
}

/// Grid area definition with name and positioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridArea {
    /// Name identifier for the grid area
    pub name: String,
    /// Starting row position (0-based)
    pub row: usize,
    /// Starting column position (0-based)
    pub col: usize,
    /// Number of rows this area spans
    pub row_span: usize,
    /// Number of columns this area spans
    pub col_span: usize,
    /// Optional CSS class for styling
    pub css_class: Option<String>,
    /// Z-index for layering (higher values appear on top)
    pub z_index: i32,
}

/// Grid child with Element content (for component-style usage)
#[derive(Debug, Clone)]
pub struct GridChild {
    /// The element to render in this grid cell
    pub element: crate::component::Element,
    /// Starting row position (0-based)
    pub row: usize,
    /// Starting column position (0-based)
    pub column: usize,
    /// Number of rows this child spans
    pub row_span: usize,
    /// Number of columns this child spans
    pub column_span: usize,
}

impl GridArea {
    /// Create a new grid area with the specified name
    ///
    /// # Arguments
    /// * `name` - Name identifier for this grid area
    ///
    /// # Returns
    /// A new `GridArea` positioned at (0,0) with 1x1 span
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            css_class: None,
            z_index: 0,
        }
    }

    /// Set the position of this grid area
    ///
    /// # Arguments
    /// * `row` - Starting row position (0-based)
    /// * `col` - Starting column position (0-based)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn at(mut self, row: usize, col: usize) -> Self {
        self.row = row;
        self.col = col;
        self
    }

    /// Set the span (size) of this grid area
    ///
    /// # Arguments
    /// * `row_span` - Number of rows to span
    /// * `col_span` - Number of columns to span
    ///
    /// # Returns
    /// Self for method chaining
    pub fn span(mut self, row_span: usize, col_span: usize) -> Self {
        self.row_span = row_span;
        self.col_span = col_span;
        self
    }

    /// Set the CSS class for styling this grid area
    ///
    /// # Arguments
    /// * `css_class` - CSS class name to apply
    ///
    /// # Returns
    /// Self for method chaining
    pub fn class(mut self, css_class: impl Into<String>) -> Self {
        self.css_class = Some(css_class.into());
        self
    }

    /// Set the z-index for layering
    ///
    /// # Arguments
    /// * `z_index` - Z-index value (higher values appear on top)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn z(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

/// Declarative grid layout builder
#[derive(Debug, Clone)]
pub struct DeclarativeGrid {
    /// Number of columns in the grid
    pub cols: usize,
    /// Number of rows in the grid
    pub rows: usize,
    /// Gap between grid cells
    pub gap: usize,
    /// Named grid areas for positioning
    pub areas: Vec<GridArea>,
    /// Mapping from area names to indices in areas vector
    pub area_map: HashMap<String, usize>,
    /// Optional CSS class for styling
    pub css_class: Option<String>,
    /// Column sizing (if None, uses equal fr units)
    pub column_sizes: Option<Vec<GridScalar>>,
    /// Row sizing (if None, uses auto sizing)
    pub row_sizes: Option<Vec<GridScalar>>,
    /// Column gap in cells
    pub column_gap: u16,
    /// Row gap in cells
    pub row_gap: u16,
    /// Child elements (alternative to areas for component-style usage)
    pub children: Vec<GridChild>,
}

impl DeclarativeGrid {
    /// Create a new declarative grid with specified columns and rows
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            gap: 0,
            areas: Vec::new(),
            area_map: HashMap::new(),
            css_class: None,
            column_sizes: None,
            row_sizes: None,
            column_gap: 1,
            row_gap: 1,
            children: Vec::new(),
        }
    }

    /// Set gap between grid cells
    pub fn gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }

    /// Set CSS class for the grid
    pub fn class(mut self, css_class: impl Into<String>) -> Self {
        self.css_class = Some(css_class.into());
        self
    }

    /// Set column sizes using GridScalar values
    pub fn column_sizes(mut self, sizes: Vec<GridScalar>) -> Self {
        self.column_sizes = Some(sizes);
        self
    }

    /// Set row sizes using GridScalar values
    pub fn row_sizes(mut self, sizes: Vec<GridScalar>) -> Self {
        self.row_sizes = Some(sizes);
        self
    }

    /// Set column gap
    pub fn column_gap(mut self, gap: u16) -> Self {
        self.column_gap = gap;
        self
    }

    /// Set row gap
    pub fn row_gap(mut self, gap: u16) -> Self {
        self.row_gap = gap;
        self
    }

    /// Make grid responsive (fill entire terminal) - this is the default behavior
    pub fn responsive(mut self) -> Self {
        // Add responsive classes to ensure grid fills terminal
        let responsive_classes = "w-full h-full min-w-full min-h-full";
        self.css_class = Some(match self.css_class {
            Some(existing) => format!("{} {}", existing, responsive_classes),
            None => responsive_classes.to_string(),
        });
        self
    }

    /// Make grid fixed size (override default responsive behavior)
    pub fn fixed_size(mut self, width: &str, height: &str) -> Self {
        let fixed_classes = format!("w-{} h-{}", width, height);
        self.css_class = Some(match self.css_class {
            Some(existing) => format!("{} {}", existing, fixed_classes),
            None => fixed_classes,
        });
        self
    }

    /// Add a child element
    pub fn child(mut self, child: GridChild) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: Vec<GridChild>) -> Self {
        self.children.extend(children);
        self
    }

    /// Create a simple grid with automatic item placement (responsive by default).
    ///
    /// Zero columns produce an empty grid because no item has a valid column.
    pub fn auto_grid(columns: usize, rows: usize, items: Vec<crate::component::Element>) -> Self {
        let mut grid = Self::new(columns, rows).responsive(); // Responsive by default
        if columns == 0 {
            return grid;
        }

        for (i, item) in items.iter().enumerate() {
            let row = i / columns;
            let col = i % columns;
            if row < rows {
                grid.children.push(GridChild {
                    element: item.clone(),
                    row,
                    column: col,
                    row_span: 1,
                    column_span: 1,
                });
            }
        }

        grid
    }

    /// Add a named area to the grid layout
    ///
    /// # Arguments
    /// * `area` - The grid area to add with its name and position
    ///
    /// # Returns
    /// Self for method chaining
    pub fn area(mut self, area: GridArea) -> Self {
        let index = self.areas.len();
        self.area_map.insert(area.name.clone(), index);
        self.areas.push(area);
        self
    }

    /// Add multiple areas at once
    pub fn areas(mut self, areas: Vec<GridArea>) -> Self {
        for area in areas {
            self = self.area(area);
        }
        self
    }

    /// Insert a new row before the specified area
    pub fn insert_row_before(&mut self, target_area: &str) -> Result<(), String> {
        let target_index = self
            .area_map
            .get(target_area)
            .ok_or_else(|| format!("Area '{}' not found", target_area))?;

        let target_row = self.areas[*target_index].row;

        // Shift all areas at or after target row down by 1
        for area in &mut self.areas {
            if area.row >= target_row {
                area.row += 1;
            }
        }

        self.rows += 1;
        Ok(())
    }

    /// Insert a new column before the specified area
    pub fn insert_col_before(&mut self, target_area: &str) -> Result<(), String> {
        let target_index = self
            .area_map
            .get(target_area)
            .ok_or_else(|| format!("Area '{}' not found", target_area))?;

        let target_col = self.areas[*target_index].col;

        // Shift all areas at or after target column right by 1
        for area in &mut self.areas {
            if area.col >= target_col {
                area.col += 1;
            }
        }

        self.cols += 1;
        Ok(())
    }

    /// Remove an area by name
    pub fn remove_area(&mut self, area_name: &str) -> Result<GridArea, String> {
        let index = self
            .area_map
            .remove(area_name)
            .ok_or_else(|| format!("Area '{}' not found", area_name))?;

        let removed_area = self.areas.remove(index);

        // Update indices in area_map
        for (_, area_index) in self.area_map.iter_mut() {
            if *area_index > index {
                *area_index -= 1;
            }
        }

        Ok(removed_area)
    }

    /// Get area by name
    pub fn get_area(&self, name: &str) -> Option<&GridArea> {
        self.area_map
            .get(name)
            .and_then(|&index| self.areas.get(index))
    }

    /// Get mutable area by name
    pub fn get_area_mut(&mut self, name: &str) -> Option<&mut GridArea> {
        if let Some(&index) = self.area_map.get(name) {
            self.areas.get_mut(index)
        } else {
            None
        }
    }

    /// Update area position
    pub fn move_area(&mut self, name: &str, row: usize, col: usize) -> Result<(), String> {
        let area = self
            .get_area_mut(name)
            .ok_or_else(|| format!("Area '{}' not found", name))?;

        area.row = row;
        area.col = col;
        Ok(())
    }

    /// Update area span
    pub fn resize_area(
        &mut self,
        name: &str,
        row_span: usize,
        col_span: usize,
    ) -> Result<(), String> {
        let area = self
            .get_area_mut(name)
            .ok_or_else(|| format!("Area '{}' not found", name))?;

        area.row_span = row_span;
        area.col_span = col_span;
        Ok(())
    }

    /// Get areas sorted by z-index for proper rendering order
    pub fn areas_by_z_index(&self) -> Vec<&GridArea> {
        let mut sorted_areas: Vec<&GridArea> = self.areas.iter().collect();
        sorted_areas.sort_by_key(|area| area.z_index);
        sorted_areas
    }

    /// Generate CSS grid template areas string
    pub fn to_css_grid_areas(&self) -> String {
        let mut grid = vec![vec!["."; self.cols]; self.rows];

        // Fill grid with area names (sorted by z-index for proper layering)
        for area in self.areas_by_z_index() {
            for r in area.row..(area.row + area.row_span) {
                for c in area.col..(area.col + area.col_span) {
                    if r < self.rows && c < self.cols {
                        if let Some(row) = grid.get_mut(r) {
                            if let Some(cell) = row.get_mut(c) {
                                *cell = &area.name;
                            }
                        }
                    }
                }
            }
        }

        // Convert to CSS format
        grid.iter()
            .map(|row| format!("\"{}\"", row.join(" ")))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate CSS grid template columns/rows
    pub fn to_css_grid_template(&self) -> (String, String) {
        let cols = if let Some(ref sizes) = self.column_sizes {
            sizes
                .iter()
                .map(|s| s.to_css())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            format!("repeat({}, 1fr)", self.cols)
        };

        let rows = if let Some(ref sizes) = self.row_sizes {
            sizes
                .iter()
                .map(|s| s.to_css())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            format!("repeat({}, auto)", self.rows)
        };

        (cols, rows)
    }

    /// Validate grid layout (check for overlaps, out-of-bounds, etc.)
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut grid = vec![vec![None::<&str>; self.cols]; self.rows];

        for area in &self.areas {
            // Check bounds
            if area.row + area.row_span > self.rows {
                errors.push(format!("Area '{}' extends beyond grid rows", area.name));
            }
            if area.col + area.col_span > self.cols {
                errors.push(format!("Area '{}' extends beyond grid columns", area.name));
            }

            // Check overlaps
            for r in area.row..(area.row + area.row_span) {
                for c in area.col..(area.col + area.col_span) {
                    if r < self.rows && c < self.cols {
                        if let Some(row) = grid.get(r) {
                            if let Some(existing) = row.get(c).and_then(|&x| x) {
                                errors.push(format!(
                                    "Areas '{}' and '{}' overlap at ({}, {})",
                                    area.name, existing, r, c
                                ));
                            } else if let Some(row) = grid.get_mut(r) {
                                if let Some(cell) = row.get_mut(c) {
                                    *cell = Some(&area.name);
                                }
                            }
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Macro for declarative grid layout (responsive by default)
#[macro_export]
macro_rules! layout {
    (grid(cols: $cols:expr, rows: $rows:expr, gap: $gap:expr) {
        $($name:literal at ($row:expr, $col:expr) $(span ($row_span:expr, $col_span:expr))? $(class $css_class:literal)? $(z $z_index:expr)?,)*
    }) => {{
        let mut grid = $crate::layout::grid::DeclarativeGrid::new($cols, $rows)
            .gap($gap)
            .responsive(); // Make responsive by default
        $(
            #[allow(unused_mut)]
            let mut area = $crate::layout::grid::GridArea::new($name).at($row, $col);
            $(area = area.span($row_span, $col_span);)?
            $(area = area.class($css_class);)?
            $(area = area.z($z_index);)?
            grid = grid.area(area);
        )*
        grid
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_css_generation() {
        let grid = layout! {
            grid(cols: 2, rows: 2, gap: 1) {
                "A" at (0, 0),
                "B" at (0, 1),
                "C" at (1, 0) span (1, 2),
            }
        };

        let css_areas = grid.to_css_grid_areas();
        assert_eq!(css_areas, "\"A B\" \"C C\"");

        let (cols, rows) = grid.to_css_grid_template();
        assert_eq!(cols, "repeat(2, 1fr)");
        assert_eq!(rows, "repeat(2, auto)");
    }

    #[test]
    fn test_flexible_sizing() {
        use super::{DeclarativeGrid, GridScalar};

        let grid = DeclarativeGrid::new(3, 2)
            .column_sizes(vec![
                GridScalar::Cells(10),
                GridScalar::Fr(1.0),
                GridScalar::Percent(25.0),
            ])
            .row_sizes(vec![GridScalar::Auto, GridScalar::Fr(2.0)])
            .column_gap(2)
            .row_gap(1);

        let (cols, rows) = grid.to_css_grid_template();
        assert_eq!(cols, "10ch 1fr 25%");
        assert_eq!(rows, "auto 2fr");
        assert_eq!(grid.column_gap, 2);
        assert_eq!(grid.row_gap, 1);
    }

    #[test]
    fn test_auto_grid() {
        use super::DeclarativeGrid;
        use crate::component::Element;

        let items = vec![
            Element::text("Item 1"),
            Element::text("Item 2"),
            Element::text("Item 3"),
            Element::text("Item 4"),
        ];

        let grid = DeclarativeGrid::auto_grid(2, 2, items);

        assert_eq!(grid.cols, 2);
        assert_eq!(grid.rows, 2);
        assert_eq!(grid.children.len(), 4);

        // Check positioning
        assert_eq!(grid.children[0].row, 0);
        assert_eq!(grid.children[0].column, 0);
        assert_eq!(grid.children[1].row, 0);
        assert_eq!(grid.children[1].column, 1);
        assert_eq!(grid.children[2].row, 1);
        assert_eq!(grid.children[2].column, 0);
        assert_eq!(grid.children[3].row, 1);
        assert_eq!(grid.children[3].column, 1);
    }

    #[test]
    fn test_responsive_by_default() {
        use super::DeclarativeGrid;
        use crate::component::Element;

        // Auto-grid should be responsive by default
        let items = vec![Element::text("Item 1"), Element::text("Item 2")];
        let auto_grid = DeclarativeGrid::auto_grid(2, 1, items);

        assert!(auto_grid.css_class.is_some());
        let css = auto_grid.css_class.unwrap();
        assert!(css.contains("w-full"));
        assert!(css.contains("h-full"));
        assert!(css.contains("min-w-full"));
        assert!(css.contains("min-h-full"));

        // Manual responsive call
        let manual_grid = DeclarativeGrid::new(2, 2).responsive();
        let css = manual_grid.css_class.unwrap();
        assert!(css.contains("w-full"));
        assert!(css.contains("h-full"));

        // Fixed size override
        let fixed_grid = DeclarativeGrid::new(2, 2).fixed_size("80", "24");
        let css = fixed_grid.css_class.unwrap();
        assert!(css.contains("w-80"));
        assert!(css.contains("h-24"));
    }
}
