use crate::component::{Component, Element, Props};
use crate::event::Event;
use crate::event::router::EventResult;
use std::any::Any;

/// Properties for Grid layout component
#[derive(Clone, Debug, PartialEq)]
pub struct GridProps {
    pub columns: GridTrackDefinition,
    pub rows: GridTrackDefinition,
    pub gap: GridGap,
    pub padding: GridPadding,
    pub alignment: GridAlignment,
    pub justify: GridJustify,
    pub auto_flow: GridAutoFlow,
    pub children: Vec<GridChild>,
}

impl Default for GridProps {
    fn default() -> Self {
        Self {
            columns: GridTrackDefinition::repeat(1, GridTrackSize::Fr(1.0)),
            rows: GridTrackDefinition::auto(),
            gap: GridGap::default(),
            padding: GridPadding::default(),
            alignment: GridAlignment::default(),
            justify: GridJustify::default(),
            auto_flow: GridAutoFlow::Row,
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
    pub area: Option<GridArea>,
    pub span: GridSpan,
}

impl GridChild {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            area: None,
            span: GridSpan::default(),
        }
    }

    pub fn with_area(mut self, area: GridArea) -> Self {
        self.area = Some(area);
        self
    }

    pub fn with_span(mut self, column_span: usize, row_span: usize) -> Self {
        self.span = GridSpan {
            column: column_span,
            row: row_span,
        };
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridArea {
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl GridArea {
    pub fn new(row_start: usize, column_start: usize, row_end: usize, column_end: usize) -> Self {
        Self {
            row_start,
            column_start,
            row_end,
            column_end,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridSpan {
    pub column: usize,
    pub row: usize,
}

impl Default for GridSpan {
    fn default() -> Self {
        Self { column: 1, row: 1 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrackDefinition {
    Fixed(Vec<GridTrackSize>),
    Repeat {
        count: usize,
        sizes: Vec<GridTrackSize>,
    },
    Auto,
}

impl GridTrackDefinition {
    pub fn repeat(count: usize, size: GridTrackSize) -> Self {
        Self::Repeat {
            count,
            sizes: vec![size],
        }
    }

    pub fn fixed(sizes: Vec<GridTrackSize>) -> Self {
        Self::Fixed(sizes)
    }

    pub fn auto() -> Self {
        Self::Auto
    }

    pub fn resolve_tracks(&self, available_size: usize, item_count: usize) -> Vec<usize> {
        match self {
            Self::Fixed(sizes) => self.calculate_track_sizes(sizes, available_size),
            Self::Repeat { count, sizes } => {
                let mut expanded = Vec::new();
                for _ in 0..*count {
                    expanded.extend(sizes.clone());
                }
                self.calculate_track_sizes(&expanded, available_size)
            }
            Self::Auto => {
                // Auto-generate tracks based on item count
                let track_count = (item_count as f64).sqrt().ceil() as usize;
                let track_size = available_size / track_count.max(1);
                vec![track_size; track_count]
            }
        }
    }

    fn calculate_track_sizes(&self, sizes: &[GridTrackSize], available_size: usize) -> Vec<usize> {
        let mut resolved = vec![0; sizes.len()];
        let mut remaining_size = available_size;
        let mut fr_total = 0.0;

        // First pass: resolve fixed and auto sizes
        for (i, size) in sizes.iter().enumerate() {
            match size {
                GridTrackSize::Fixed(px) => {
                    resolved[i] = *px;
                    remaining_size = remaining_size.saturating_sub(*px);
                }
                GridTrackSize::Auto => {
                    // Auto size - estimate based on content (simplified)
                    let auto_size = available_size / sizes.len().max(1);
                    resolved[i] = auto_size;
                    remaining_size = remaining_size.saturating_sub(auto_size);
                }
                GridTrackSize::Fr(fr) => {
                    fr_total += fr;
                }
                GridTrackSize::MinContent => {
                    // Min content size (simplified)
                    resolved[i] = 5;
                    remaining_size = remaining_size.saturating_sub(5);
                }
                GridTrackSize::MaxContent => {
                    // Max content size (simplified)
                    resolved[i] = available_size / 4;
                    remaining_size = remaining_size.saturating_sub(available_size / 4);
                }
            }
        }

        // Second pass: distribute remaining space to fr units
        if fr_total > 0.0 && remaining_size > 0 {
            for (i, size) in sizes.iter().enumerate() {
                if let GridTrackSize::Fr(fr) = size {
                    resolved[i] = ((remaining_size as f64) * (fr / fr_total)) as usize;
                }
            }
        }

        resolved
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrackSize {
    Fixed(usize), // Fixed pixel size
    Fr(f64),      // Fractional unit
    Auto,         // Auto-size based on content
    MinContent,   // Minimum content size
    MaxContent,   // Maximum content size
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GridGap {
    pub row: usize,
    pub column: usize,
}

impl GridGap {
    pub fn all(gap: usize) -> Self {
        Self {
            row: gap,
            column: gap,
        }
    }

    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GridPadding {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl GridPadding {
    pub fn all(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridAlignment {
    pub items: GridAlignItems,     // Align items within their grid area
    pub content: GridAlignContent, // Align the grid within the container
}

impl Default for GridAlignment {
    fn default() -> Self {
        Self {
            items: GridAlignItems::Stretch,
            content: GridAlignContent::Start,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridAlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridAlignContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridJustify {
    pub items: GridJustifyItems,     // Justify items within their grid area
    pub content: GridJustifyContent, // Justify the grid within the container
}

impl Default for GridJustify {
    fn default() -> Self {
        Self {
            items: GridJustifyItems::Stretch,
            content: GridJustifyContent::Start,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridJustifyItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridJustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridAutoFlow {
    Row,         // Fill row by row
    Column,      // Fill column by column
    RowDense,    // Fill row by row, dense packing
    ColumnDense, // Fill column by column, dense packing
}

/// State for Grid component
#[derive(Clone, Debug, Default)]
pub struct GridState {
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub computed_columns: Vec<usize>,
    pub computed_rows: Vec<usize>,
}

/// Grid layout component - 2D grid layout system
pub struct Grid {
    state: GridState,
}

impl Grid {
    fn compute_grid_tracks(
        &self,
        props: &GridProps,
        available_width: usize,
        available_height: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let content_width =
            available_width.saturating_sub(props.padding.left + props.padding.right);
        let content_height =
            available_height.saturating_sub(props.padding.top + props.padding.bottom);

        let item_count = props.children.len();

        let columns = props.columns.resolve_tracks(content_width, item_count);
        let rows = props.rows.resolve_tracks(content_height, item_count);

        // Adjust for gaps
        let adjusted_columns =
            self.adjust_tracks_for_gaps(&columns, props.gap.column, content_width);
        let adjusted_rows = self.adjust_tracks_for_gaps(&rows, props.gap.row, content_height);

        (adjusted_columns, adjusted_rows)
    }

    fn adjust_tracks_for_gaps(&self, tracks: &[usize], gap: usize, available: usize) -> Vec<usize> {
        if tracks.is_empty() {
            return Vec::new();
        }

        let total_gap = gap * (tracks.len().saturating_sub(1));
        let available_for_tracks = available.saturating_sub(total_gap);
        let total_requested: usize = tracks.iter().sum();

        if total_requested <= available_for_tracks {
            tracks.to_vec()
        } else {
            // Scale down proportionally
            tracks
                .iter()
                .map(|&size| (size * available_for_tracks) / total_requested.max(1))
                .collect()
        }
    }

    fn place_grid_items(
        &self,
        props: &GridProps,
        columns: &[usize],
        rows: &[usize],
    ) -> Vec<GridItemPlacement> {
        let mut placements = Vec::new();
        let mut grid_matrix = vec![vec![false; columns.len()]; rows.len()];

        // First pass: place items with explicit grid areas
        for (item_index, child) in props.children.iter().enumerate() {
            if let Some(area) = &child.area {
                let placement = GridItemPlacement {
                    item_index,
                    row: area.row_start,
                    column: area.column_start,
                    row_span: area.row_end.saturating_sub(area.row_start).max(1),
                    column_span: area.column_end.saturating_sub(area.column_start).max(1),
                };

                // Mark cells as occupied
                self.mark_cells_occupied(&mut grid_matrix, &placement);
                placements.push(placement);
            }
        }

        // Second pass: auto-place remaining items
        for (item_index, child) in props.children.iter().enumerate() {
            if child.area.is_none()
                && let Some(placement) = self.find_auto_placement(
                    &grid_matrix,
                    item_index,
                    &child.span,
                    &props.auto_flow,
                    columns.len(),
                    rows.len(),
                )
            {
                self.mark_cells_occupied(&mut grid_matrix, &placement);
                placements.push(placement);
            }
        }

        placements
    }

    #[allow(clippy::ptr_arg)]
    fn mark_cells_occupied(&self, grid_matrix: &mut Vec<Vec<bool>>, placement: &GridItemPlacement) {
        for row in placement.row..placement.row + placement.row_span {
            for col in placement.column..placement.column + placement.column_span {
                if row < grid_matrix.len() && col < grid_matrix[0].len() {
                    grid_matrix[row][col] = true;
                }
            }
        }
    }

    #[allow(clippy::unnecessary_mut_passed, clippy::ptr_arg)]
    fn find_auto_placement(
        &self,
        grid_matrix: &Vec<Vec<bool>>,
        item_index: usize,
        span: &GridSpan,
        auto_flow: &GridAutoFlow,
        columns_count: usize,
        rows_count: usize,
    ) -> Option<GridItemPlacement> {
        match auto_flow {
            GridAutoFlow::Row | GridAutoFlow::RowDense => {
                self.find_row_placement(grid_matrix, item_index, span, columns_count, rows_count)
            }
            GridAutoFlow::Column | GridAutoFlow::ColumnDense => {
                self.find_column_placement(grid_matrix, item_index, span, columns_count, rows_count)
            }
        }
    }

    #[allow(clippy::ptr_arg)]
    fn find_row_placement(
        &self,
        grid_matrix: &Vec<Vec<bool>>,
        item_index: usize,
        span: &GridSpan,
        columns_count: usize,
        rows_count: usize,
    ) -> Option<GridItemPlacement> {
        for row in 0..rows_count {
            for col in 0..columns_count {
                if self.can_place_item(grid_matrix, row, col, span, columns_count, rows_count) {
                    return Some(GridItemPlacement {
                        item_index,
                        row,
                        column: col,
                        row_span: span.row,
                        column_span: span.column,
                    });
                }
            }
        }
        None
    }

    #[allow(clippy::ptr_arg)]
    fn find_column_placement(
        &self,
        grid_matrix: &Vec<Vec<bool>>,
        item_index: usize,
        span: &GridSpan,
        columns_count: usize,
        rows_count: usize,
    ) -> Option<GridItemPlacement> {
        for col in 0..columns_count {
            for row in 0..rows_count {
                if self.can_place_item(grid_matrix, row, col, span, columns_count, rows_count) {
                    return Some(GridItemPlacement {
                        item_index,
                        row,
                        column: col,
                        row_span: span.row,
                        column_span: span.column,
                    });
                }
            }
        }
        None
    }

    #[allow(clippy::ptr_arg)]
    fn can_place_item(
        &self,
        grid_matrix: &Vec<Vec<bool>>,
        row: usize,
        col: usize,
        span: &GridSpan,
        columns_count: usize,
        rows_count: usize,
    ) -> bool {
        // Check if item fits within grid bounds
        if row + span.row > rows_count || col + span.column > columns_count {
            return false;
        }

        // Check if all cells in the span are available
        for r in row..row + span.row {
            for c in col..col + span.column {
                if r < grid_matrix.len() && c < grid_matrix[0].len() && grid_matrix[r][c] {
                    return false;
                }
            }
        }

        true
    }

    fn calculate_item_positions(
        &self,
        props: &GridProps,
        placements: &[GridItemPlacement],
        columns: &[usize],
        rows: &[usize],
    ) -> Vec<(usize, usize, usize, usize)> {
        let mut positions = Vec::new();

        for placement in placements {
            // Calculate position and size
            let x = props.padding.left
                + columns.iter().take(placement.column).sum::<usize>()
                + props.gap.column * placement.column;

            let y = props.padding.top
                + rows.iter().take(placement.row).sum::<usize>()
                + props.gap.row * placement.row;

            let width = columns
                .iter()
                .skip(placement.column)
                .take(placement.column_span)
                .sum::<usize>()
                + props.gap.column * (placement.column_span.saturating_sub(1));

            let height = rows
                .iter()
                .skip(placement.row)
                .take(placement.row_span)
                .sum::<usize>()
                + props.gap.row * (placement.row_span.saturating_sub(1));

            positions.push((x, y, width, height));
        }

        positions
    }

    fn render_item(
        &self,
        element: &Element,
        _x: usize,
        _y: usize,
        width: usize,
        height: usize,
    ) -> Vec<String> {
        let content = match &element.element_type {
            crate::component::ElementType::Text(text) => text.clone(),
            _ => {
                // Render container children recursively
                element
                    .children
                    .iter()
                    .map(|c| match &c.element_type {
                        crate::component::ElementType::Text(text) => text.clone(),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        let mut lines = Vec::new();
        let content_lines: Vec<&str> = content.lines().collect();

        // Process content lines (no positioning here - that's handled by canvas placement)
        for (i, line) in content_lines.iter().enumerate() {
            if i >= height {
                break;
            }

            let trimmed_line = if line.len() > width {
                &line[..width]
            } else {
                line
            };
            lines.push(trimmed_line.to_string());
        }

        // Fill remaining height with empty lines
        while lines.len() < height {
            lines.push(String::new());
        }

        lines
    }
}

#[derive(Clone, Debug, PartialEq)]
struct GridItemPlacement {
    item_index: usize,
    row: usize,
    column: usize,
    row_span: usize,
    column_span: usize,
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
        if props.children.is_empty() {
            return Element::text("");
        }

        let available_width = if state.viewport_width > 0 {
            state.viewport_width
        } else {
            80
        };
        let available_height = if state.viewport_height > 0 {
            state.viewport_height
        } else {
            24
        };

        let (columns, rows) = self.compute_grid_tracks(props, available_width, available_height);
        let placements = self.place_grid_items(props, &columns, &rows);
        let positions = self.calculate_item_positions(props, &placements, &columns, &rows);

        // Create a canvas to render all items
        let mut canvas = vec![vec![' '; available_width]; available_height];

        // Render each item to the canvas
        for (placement, &(x, y, width, height)) in placements.iter().zip(positions.iter()) {
            if let Some(child) = props.children.get(placement.item_index) {
                let item_lines = self.render_item(&child.element, 0, 0, width, height);

                // Draw item lines onto canvas
                for (line_idx, line) in item_lines.iter().enumerate() {
                    let canvas_y = y + line_idx;
                    if canvas_y >= available_height {
                        break;
                    }

                    for (char_idx, ch) in line.chars().enumerate() {
                        let canvas_x = x + char_idx;
                        if canvas_x < available_width && ch != ' ' {
                            canvas[canvas_y][canvas_x] = ch;
                        }
                    }
                }
            }
        }

        // Convert canvas to string
        let result = canvas
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        Element::text(result)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Resize(resize_event) => {
                state.viewport_width = resize_event.width as usize;
                state.viewport_height = resize_event.height as usize;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;

    #[test]
    fn test_grid_basic_layout() {
        let grid = Grid::new(GridProps::default());
        let props = GridProps {
            columns: GridTrackDefinition::repeat(2, GridTrackSize::Fr(1.0)),
            rows: GridTrackDefinition::repeat(2, GridTrackSize::Fr(1.0)),
            children: vec![
                GridChild::new(Element::text("A")),
                GridChild::new(Element::text("B")),
                GridChild::new(Element::text("C")),
                GridChild::new(Element::text("D")),
            ],
            ..Default::default()
        };
        let state = GridState {
            viewport_width: 80,
            viewport_height: 24,
            ..Default::default()
        };

        let element = grid.render(&props, &state);
        if let ElementType::Text(content) = &element.element_type {
            assert!(content.contains("A"));
            assert!(content.contains("B"));
            assert!(content.contains("C"));
            assert!(content.contains("D"));
        }
    }

    #[test]
    fn test_grid_track_resolution() {
        let _grid = Grid::new(GridProps::default());
        let definition = GridTrackDefinition::Fixed(vec![
            GridTrackSize::Fixed(20),
            GridTrackSize::Fr(1.0),
            GridTrackSize::Fixed(15),
        ]);

        let tracks = definition.resolve_tracks(100, 3);
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0], 20); // Fixed size
        assert_eq!(tracks[2], 15); // Fixed size
        assert!(tracks[1] > 0); // Fr unit gets remaining space
    }

    #[test]
    fn test_grid_item_placement() {
        let grid = Grid::new(GridProps::default());
        let props = GridProps {
            columns: GridTrackDefinition::repeat(3, GridTrackSize::Fr(1.0)),
            rows: GridTrackDefinition::repeat(2, GridTrackSize::Fr(1.0)),
            children: vec![
                GridChild::new(Element::text("A")).with_area(GridArea::new(0, 0, 1, 2)),
                GridChild::new(Element::text("B")),
                GridChild::new(Element::text("C")),
            ],
            ..Default::default()
        };

        let columns = vec![25, 25, 25];
        let rows = vec![10, 10];
        let placements = grid.place_grid_items(&props, &columns, &rows);

        assert_eq!(placements.len(), 3);

        // First item has explicit placement
        assert_eq!(placements[0].row, 0);
        assert_eq!(placements[0].column, 0);
        assert_eq!(placements[0].column_span, 2);
    }

    #[test]
    fn test_grid_gaps() {
        let grid = Grid::new(GridProps::default());
        let tracks = vec![20, 20, 20];
        let adjusted = grid.adjust_tracks_for_gaps(&tracks, 5, 100);

        // With gaps, tracks should be adjusted to fit
        assert_eq!(adjusted.len(), 3);
        let total_with_gaps: usize = adjusted.iter().sum::<usize>() + 2 * 5; // 2 gaps
        assert!(total_with_gaps <= 100);
    }

    #[test]
    fn test_grid_span_placement() {
        let grid = Grid::new(GridProps::default());
        let span = GridSpan { column: 2, row: 1 };
        let grid_matrix = vec![vec![false, false, false], vec![false, false, false]];

        assert!(grid.can_place_item(&grid_matrix, 0, 0, &span, 3, 2));
        assert!(grid.can_place_item(&grid_matrix, 0, 1, &span, 3, 2));
        assert!(!grid.can_place_item(&grid_matrix, 0, 2, &span, 3, 2)); // Would exceed bounds
    }

    #[test]
    fn test_grid_auto_flow() {
        let grid = Grid::new(GridProps::default());
        let props = GridProps {
            columns: GridTrackDefinition::repeat(2, GridTrackSize::Fr(1.0)),
            rows: GridTrackDefinition::auto(),
            auto_flow: GridAutoFlow::Row,
            children: vec![
                GridChild::new(Element::text("1")),
                GridChild::new(Element::text("2")),
                GridChild::new(Element::text("3")),
            ],
            ..Default::default()
        };

        let columns = vec![40, 40];
        let rows = vec![8, 8];
        let placements = grid.place_grid_items(&props, &columns, &rows);

        // Items should be placed row by row
        assert_eq!(placements.len(), 3);
        assert_eq!(placements[0].row, 0);
        assert_eq!(placements[0].column, 0);
        assert_eq!(placements[1].row, 0);
        assert_eq!(placements[1].column, 1);
        assert_eq!(placements[2].row, 1);
        assert_eq!(placements[2].column, 0);
    }
}
