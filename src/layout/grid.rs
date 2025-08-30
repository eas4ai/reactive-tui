//! Declarative Grid Layout System
//!
//! A clean, intuitive API for creating grid layouts with CSS integration
//! and dynamic manipulation capabilities.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Grid area definition with name and positioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridArea {
    pub name: String,
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub css_class: Option<String>,
    pub z_index: i32,
}

impl GridArea {
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
    
    pub fn at(mut self, row: usize, col: usize) -> Self {
        self.row = row;
        self.col = col;
        self
    }
    
    pub fn span(mut self, row_span: usize, col_span: usize) -> Self {
        self.row_span = row_span;
        self.col_span = col_span;
        self
    }
    
    pub fn class(mut self, css_class: impl Into<String>) -> Self {
        self.css_class = Some(css_class.into());
        self
    }

    pub fn z(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

/// Declarative grid layout builder
#[derive(Debug, Clone)]
pub struct DeclarativeGrid {
    pub cols: usize,
    pub rows: usize,
    pub gap: usize,
    pub areas: Vec<GridArea>,
    pub area_map: HashMap<String, usize>, // name -> index in areas
    pub css_class: Option<String>,
}

impl DeclarativeGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            gap: 0,
            areas: Vec::new(),
            area_map: HashMap::new(),
            css_class: None,
        }
    }
    
    pub fn gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }
    
    pub fn class(mut self, css_class: impl Into<String>) -> Self {
        self.css_class = Some(css_class.into());
        self
    }
    
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
        let target_index = self.area_map.get(target_area)
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
        let target_index = self.area_map.get(target_area)
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
        let index = self.area_map.remove(area_name)
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
        self.area_map.get(name)
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
        let area = self.get_area_mut(name)
            .ok_or_else(|| format!("Area '{}' not found", name))?;
        
        area.row = row;
        area.col = col;
        Ok(())
    }
    
    /// Update area span
    pub fn resize_area(&mut self, name: &str, row_span: usize, col_span: usize) -> Result<(), String> {
        let area = self.get_area_mut(name)
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
                        grid[r][c] = &area.name;
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
        let cols = format!("repeat({}, 1fr)", self.cols);
        let rows = format!("repeat({}, 1fr)", self.rows);
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
                        if let Some(existing) = grid[r][c] {
                            errors.push(format!("Areas '{}' and '{}' overlap at ({}, {})", 
                                area.name, existing, r, c));
                        } else {
                            grid[r][c] = Some(&area.name);
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

/// Macro for declarative grid layout
#[macro_export]
macro_rules! layout {
    (grid(cols: $cols:expr, rows: $rows:expr, gap: $gap:expr) {
        $($name:literal at ($row:expr, $col:expr) $(span ($row_span:expr, $col_span:expr))? $(class $css_class:literal)? $(z $z_index:expr)?,)*
    }) => {{
        let mut grid = $crate::layout::grid::DeclarativeGrid::new($cols, $rows).gap($gap);
        $(
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
    use super::*;
    
    #[test]
    fn test_declarative_grid_creation() {
        let grid = layout! {
            grid(cols: 3, rows: 3, gap: 1) {
                "Header" at (0, 0) span (1, 3) class "header",
                "Sidebar" at (1, 0) class "sidebar",
                "Main" at (1, 1) class "main",
                "Notifications" at (1, 2) class "notifications",
                "Footer" at (2, 0) span (1, 3) class "footer",
            }
        };
        
        assert_eq!(grid.cols, 3);
        assert_eq!(grid.rows, 3);
        assert_eq!(grid.gap, 1);
        assert_eq!(grid.areas.len(), 5);
        
        let header = grid.get_area("Header").unwrap();
        assert_eq!(header.row, 0);
        assert_eq!(header.col, 0);
        assert_eq!(header.row_span, 1);
        assert_eq!(header.col_span, 3);
        assert_eq!(header.css_class, Some("header".to_string()));
    }
    
    #[test]
    fn test_grid_manipulation() {
        let mut grid = layout! {
            grid(cols: 2, rows: 2, gap: 1) {
                "Header" at (0, 0) span (1, 2),
                "Footer" at (1, 0) span (1, 2),
            }
        };
        
        // Insert row before Footer
        grid.insert_row_before("Footer").unwrap();
        
        assert_eq!(grid.rows, 3);
        assert_eq!(grid.get_area("Footer").unwrap().row, 2);
    }
    
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
        assert_eq!(rows, "repeat(2, 1fr)");
    }
}
