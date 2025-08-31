//! Working Grid Layout Component
//!
//! A simple, functional grid system based on the reference implementation
//! that actually works and produces clean, readable output.

use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::event::Event;
use std::any::Any;

/// Grid scalar value for sizing, similar to CSS units
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

/// Properties for the Grid layout component
#[derive(Clone, Debug, PartialEq)]
pub struct GridProps {
    pub columns: Vec<GridScalar>,
    pub rows: Vec<GridScalar>,
    pub column_gap: u16,
    pub row_gap: u16,
    pub show_borders: bool,
    pub children: Vec<GridChild>,
}

impl Default for GridProps {
    fn default() -> Self {
        Self {
            columns: vec![GridScalar::Fr(1.0), GridScalar::Fr(1.0)],
            rows: vec![GridScalar::Auto],
            column_gap: 1,
            row_gap: 1,
            show_borders: true,
            children: Vec::new(),
        }
    }
}

impl Props for GridProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridChild {
    pub element: Element,
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

impl GridChild {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            row: 0,
            column: 0,
            row_span: 1,
            column_span: 1,
        }
    }

    pub fn at(mut self, row: usize, column: usize) -> Self {
        self.row = row;
        self.column = column;
        self
    }

    pub fn span(mut self, row_span: usize, column_span: usize) -> Self {
        self.row_span = row_span;
        self.column_span = column_span;
        self
    }
}

/// Resolved grid track (column or row) with offset and size
#[derive(Debug, Clone, Copy)]
pub struct GridTrack {
    pub offset: u16,
    pub size: u16,
}

/// State for Grid component
#[derive(Clone, Debug, Default)]
pub struct GridState {
    pub viewport_width: usize,
    pub viewport_height: usize,
}

/// Working Grid layout component
pub struct Grid {
    state: GridState,
}

impl Grid {
    /// Create a simple grid with automatic item placement
    pub fn auto_grid(columns: usize, rows: usize, items: Vec<Element>) -> GridProps {
        let mut children = Vec::new();

        for (i, item) in items.iter().enumerate() {
            let row = i / columns;
            let col = i % columns;
            if row < rows {
                children.push(GridChild {
                    element: item.clone(),
                    row,
                    column: col,
                    row_span: 1,
                    column_span: 1,
                });
            }
        }

        GridProps {
            columns: (0..columns).map(|_| GridScalar::Fr(1.0)).collect(),
            rows: (0..rows).map(|_| GridScalar::Auto).collect(),
            children,
            ..Default::default()
        }
    }

    /// Resolve grid tracks (columns or rows) into concrete positions and sizes
    fn resolve_tracks(
        &self,
        scalars: &[GridScalar],
        available_space: u16,
        gap: u16,
    ) -> Vec<GridTrack> {
        let count = scalars.len();
        if count == 0 {
            return Vec::new();
        }

        let total_gap = gap * (count.saturating_sub(1)) as u16;
        let content_space = available_space.saturating_sub(total_gap) as f32;

        // First pass: resolve fixed sizes and calculate remaining space
        let mut resolved_sizes = vec![0.0; count];
        let mut total_fractions = 0.0;
        let mut used_space = 0.0;

        for (i, scalar) in scalars.iter().enumerate() {
            match scalar {
                GridScalar::Cells(cells) => {
                    resolved_sizes[i] = *cells as f32;
                    used_space += *cells as f32;
                }
                GridScalar::Percent(pct) => {
                    resolved_sizes[i] = content_space * (pct / 100.0);
                    used_space += resolved_sizes[i];
                }
                GridScalar::Fr(fr) => {
                    total_fractions += fr;
                }
                GridScalar::Auto => {
                    // Auto sizing: reasonable default
                    resolved_sizes[i] = 20.0;
                    used_space += 20.0;
                }
            }
        }

        // Second pass: resolve fraction units
        let remaining_space = (content_space - used_space).max(0.0);
        let fraction_unit = if total_fractions > 0.0 {
            remaining_space / total_fractions
        } else {
            0.0
        };

        for (i, scalar) in scalars.iter().enumerate() {
            if let GridScalar::Fr(fr) = scalar {
                resolved_sizes[i] = fraction_unit * fr;
            }
        }

        // Build tracks with offsets
        let mut tracks = Vec::new();
        let mut current_offset = 0;

        for (i, size) in resolved_sizes.iter().enumerate() {
            tracks.push(GridTrack {
                offset: current_offset,
                size: size.round().max(1.0) as u16,
            });

            current_offset += size.round().max(1.0) as u16;
            if i < count - 1 {
                current_offset += gap;
            }
        }

        tracks
    }

    /// Render the grid to a string
    fn render_grid(
        &self,
        props: &GridProps,
        available_width: u16,
        available_height: u16,
    ) -> String {
        if props.children.is_empty() {
            return String::new();
        }

        // Resolve tracks
        let columns = self.resolve_tracks(&props.columns, available_width, props.column_gap);
        let rows = self.resolve_tracks(&props.rows, available_height, props.row_gap);

        if columns.is_empty() || rows.is_empty() {
            return String::new();
        }

        // Create canvas
        let total_width = columns.last().map(|c| c.offset + c.size).unwrap_or(0) as usize;
        let total_height = rows.last().map(|r| r.offset + r.size).unwrap_or(0) as usize;

        let mut canvas = vec![vec![' '; total_width]; total_height];

        // Place children in grid
        for child in &props.children {
            if child.row < rows.len() && child.column < columns.len() {
                let content = self.extract_text_content(&child.element);
                self.render_child_to_canvas(
                    &mut canvas,
                    &content,
                    &columns[child.column],
                    &rows[child.row],
                    child.column_span.min(columns.len() - child.column),
                    child.row_span.min(rows.len() - child.row),
                    &columns,
                    &rows,
                );
            }
        }

        // Convert canvas to string
        canvas
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render a child element to the canvas
    fn render_child_to_canvas(
        &self,
        canvas: &mut Vec<Vec<char>>,
        content: &str,
        start_col: &GridTrack,
        start_row: &GridTrack,
        col_span: usize,
        row_span: usize,
        columns: &[GridTrack],
        rows: &[GridTrack],
    ) {
        // Calculate spanned area
        let end_col_idx = (start_col as *const GridTrack as usize - columns.as_ptr() as usize)
            / std::mem::size_of::<GridTrack>()
            + col_span
            - 1;
        let end_row_idx = (start_row as *const GridTrack as usize - rows.as_ptr() as usize)
            / std::mem::size_of::<GridTrack>()
            + row_span
            - 1;

        let end_col = columns.get(end_col_idx).unwrap_or(start_col);
        let end_row = rows.get(end_row_idx).unwrap_or(start_row);

        let width = (end_col.offset + end_col.size - start_col.offset) as usize;
        let height = (end_row.offset + end_row.size - start_row.offset) as usize;

        let content_lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in content_lines.iter().enumerate() {
            let canvas_y = start_row.offset as usize + line_idx;
            if canvas_y >= canvas.len() || line_idx >= height {
                break;
            }

            let trimmed_line = if line.len() > width {
                &line[..width]
            } else {
                line
            };

            for (char_idx, ch) in trimmed_line.chars().enumerate() {
                let canvas_x = start_col.offset as usize + char_idx;
                if canvas_x < canvas[canvas_y].len() {
                    canvas[canvas_y][canvas_x] = ch;
                }
            }
        }
    }

    /// Extract text content from an element
    fn extract_text_content(&self, element: &Element) -> String {
        match &element.element_type {
            crate::component::ElementType::Text(text) => text.clone(),
            _ => element
                .children
                .iter()
                .map(|c| self.extract_text_content(c))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

impl Component for Grid {
    type Props = GridProps;
    type State = GridState;

    fn new(_props: Self::Props) -> Self {
        Self {
            state: GridState::default(),
        }
    }

    fn update(&mut self, _props: &Self::Props, state: &mut Self::State) -> bool {
        self.state = state.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let available_width = if state.viewport_width > 0 {
            state.viewport_width as u16
        } else {
            80
        };
        let available_height = if state.viewport_height > 0 {
            state.viewport_height as u16
        } else {
            24
        };

        let rendered_content = self.render_grid(props, available_width, available_height);
        Element::text(rendered_content)
    }

    fn handle_event(
        &mut self,
        _event: &Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        EventResult::Ignored
    }
}
