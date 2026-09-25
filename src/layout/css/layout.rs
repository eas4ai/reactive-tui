//! Layout utilities: flexbox, grid, display, position

use super::parsers::{parse_grid_value, parse_z_index};
use crate::layout::style::{AlignItems, Direction, GridAutoFlow, JustifyContent, StyleBuilder};

/// Apply display utilities
pub fn apply_display(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "block" => Some(sb.display_flex().direction(Direction::Column)),
        "inline" => Some(sb.display_flex().direction(Direction::Row)),
        "inline-block" => Some(sb.display_flex().direction(Direction::Row)),
        "flex" => Some(sb.display_flex()),
        "inline-flex" => Some(sb.display_flex()),
        "grid" => Some(sb.display_grid()),
        "inline-grid" => Some(sb.display_grid()),
        "hidden" => Some(sb.size_px(Some(0.0), Some(0.0))), // Hide by setting size to 0
        _ => None,
    }
}

/// Apply flexbox utilities
pub fn apply_flexbox(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Flex direction
        "flex-row" => Some(sb.display_flex().direction(Direction::Row)),
        "flex-row-reverse" => Some(sb.display_flex().direction(Direction::RowReverse)),
        "flex-col" => Some(sb.display_flex().direction(Direction::Column)),
        "flex-col-reverse" => Some(sb.display_flex().direction(Direction::ColumnReverse)),

        // Flex wrap
        "flex-wrap" => Some(sb.display_flex().flex_wrap(true)),
        "flex-nowrap" => Some(sb.display_flex().flex_wrap(false)),

        // Flex grow/shrink
        "flex-1" => Some(sb.display_flex().flex_grow(1.0)),
        "flex-auto" => Some(sb.display_flex().flex_grow(1.0).flex_shrink(1.0)),
        "flex-initial" => Some(sb.display_flex().flex_grow(0.0).flex_shrink(1.0)),
        "flex-none" => Some(sb.display_flex().flex_grow(0.0).flex_shrink(0.0)),
        "grow" | "grow-1" => Some(sb.flex_grow(1.0)),
        "grow-0" => Some(sb.flex_grow(0.0)),
        "shrink" | "shrink-1" => Some(sb.flex_shrink(1.0)),
        "shrink-0" => Some(sb.flex_shrink(0.0)),

        _ => None,
    }
}

/// Apply justify-content utilities
pub fn apply_justify_content(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "justify-start" => Some(sb.justify_content(JustifyContent::Start)),
        "justify-end" => Some(sb.justify_content(JustifyContent::End)),
        "justify-center" => Some(sb.justify_content(JustifyContent::Center)),
        "justify-between" => Some(sb.justify_content(JustifyContent::SpaceBetween)),
        "justify-around" => Some(sb.justify_content(JustifyContent::SpaceAround)),
        "justify-evenly" => Some(sb.justify_content(JustifyContent::SpaceEvenly)),
        _ => None,
    }
}

/// Apply align-items utilities
pub fn apply_align_items(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "items-start" => Some(sb.align_items(AlignItems::Start)),
        "items-end" => Some(sb.align_items(AlignItems::End)),
        "items-center" => Some(sb.align_items(AlignItems::Center)),
        "items-baseline" => Some(sb.align_items(AlignItems::Baseline)),
        "items-stretch" => Some(sb.align_items(AlignItems::Stretch)),
        _ => None,
    }
}

/// Apply align-self utilities (for individual item alignment override)
pub fn apply_align_self(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    use crate::layout::style::AlignSelf;
    match token {
        "self-auto" => Some(sb.align_self(AlignSelf::Auto)),
        "self-start" => Some(sb.align_self(AlignSelf::Start)),
        "self-end" => Some(sb.align_self(AlignSelf::End)),
        "self-center" => Some(sb.align_self(AlignSelf::Center)),
        "self-stretch" => Some(sb.align_self(AlignSelf::Stretch)),
        _ => None,
    }
}

/// Apply place-items utilities (shorthand for align-items + justify-items)
/// In flexbox context, this sets both align-items and justify-content
pub fn apply_place_items(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "place-items-start" => Some(
            sb.align_items(AlignItems::Start)
                .justify_content(JustifyContent::Start),
        ),
        "place-items-end" => Some(
            sb.align_items(AlignItems::End)
                .justify_content(JustifyContent::End),
        ),
        "place-items-center" => Some(
            sb.align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
        ),
        "place-items-stretch" => Some(
            sb.align_items(AlignItems::Stretch)
                .justify_content(JustifyContent::Start),
        ),
        _ => None,
    }
}

/// Apply grid utilities
pub fn apply_grid(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Grid template columns
    if let Some(cols) = parse_grid_value(token, "grid-cols-") {
        return Some(sb.display_grid().grid_template_columns(cols));
    }

    // Grid template rows
    if let Some(rows) = parse_grid_value(token, "grid-rows-") {
        return Some(sb.display_grid().grid_template_rows(rows));
    }

    // Grid column utilities
    match token {
        "col-auto" => return Some(sb.grid_column_auto()),
        "col-span-full" => return Some(sb.grid_column_span(12)), // Full width
        _ => {}
    }

    if let Some(span) = parse_grid_value(token, "col-span-") {
        return Some(sb.grid_column_span(span));
    }

    if let Some(start) = parse_grid_value(token, "col-start-") {
        return Some(sb.grid_column_start(start as i16));
    }

    if let Some(end) = parse_grid_value(token, "col-end-") {
        return Some(sb.grid_column_end(end as i16));
    }

    // Grid row utilities
    match token {
        "row-auto" => return Some(sb.grid_row_auto()),
        "row-span-full" => return Some(sb.grid_row_span(12)), // Full height
        _ => {}
    }

    if let Some(span) = parse_grid_value(token, "row-span-") {
        return Some(sb.grid_row_span(span));
    }

    if let Some(start) = parse_grid_value(token, "row-start-") {
        return Some(sb.grid_row_start(start as i16));
    }

    if let Some(end) = parse_grid_value(token, "row-end-") {
        return Some(sb.grid_row_end(end as i16));
    }

    // Grid auto flow
    match token {
        "grid-flow-row" => Some(sb.display_grid().grid_auto_flow(GridAutoFlow::Row)),
        "grid-flow-col" => Some(sb.display_grid().grid_auto_flow(GridAutoFlow::Column)),
        "grid-flow-dense" => Some(sb.display_grid().grid_auto_flow(GridAutoFlow::RowDense)),
        "grid-flow-row-dense" => Some(sb.display_grid().grid_auto_flow(GridAutoFlow::RowDense)),
        "grid-flow-col-dense" => Some(sb.display_grid().grid_auto_flow(GridAutoFlow::ColumnDense)),
        _ => {
            // Grid auto-fit and auto-fill
            if token.starts_with("grid-cols-") {
                if token.contains("auto-fit") {
                    // Extract minimum size from token like "grid-cols-auto-fit-20"
                    if let Some(min_size) = token
                        .strip_prefix("grid-cols-auto-fit-")
                        .and_then(|s| s.parse::<u16>().ok())
                    {
                        return Some(sb.display_grid().grid_auto_fit_columns(min_size));
                    }
                    // Default auto-fit
                    return Some(sb.display_grid().grid_auto_fit_columns(20)); // 20 chars default
                }
                if token.contains("auto-fill") {
                    // Extract minimum size from token like "grid-cols-auto-fill-15"
                    if let Some(min_size) = token
                        .strip_prefix("grid-cols-auto-fill-")
                        .and_then(|s| s.parse::<u16>().ok())
                    {
                        return Some(sb.display_grid().grid_auto_fill_columns(min_size));
                    }
                    // Default auto-fill
                    return Some(sb.display_grid().grid_auto_fill_columns(15)); // 15 chars default
                }
            }

            // Grid rows auto-fit and auto-fill
            if token.starts_with("grid-rows-") {
                if token.contains("auto-fit") {
                    if let Some(min_size) = token
                        .strip_prefix("grid-rows-auto-fit-")
                        .and_then(|s| s.parse::<u16>().ok())
                    {
                        return Some(sb.display_grid().grid_auto_fit_rows(min_size));
                    }
                    return Some(sb.display_grid().grid_auto_fit_rows(3)); // 3 rows default
                }
                if token.contains("auto-fill") {
                    if let Some(min_size) = token
                        .strip_prefix("grid-rows-auto-fill-")
                        .and_then(|s| s.parse::<u16>().ok())
                    {
                        return Some(sb.display_grid().grid_auto_fill_rows(min_size));
                    }
                    return Some(sb.display_grid().grid_auto_fill_rows(2)); // 2 rows default
                }
            }

            // Grid gap utilities (more specific than general gap)
            if let Some(gap) = parse_grid_value(token, "grid-gap-") {
                return Some(sb.display_grid().gap_px(gap as f32, gap as f32));
            }
            if let Some(gap_x) = parse_grid_value(token, "grid-gap-x-") {
                return Some(sb.display_grid().gap_px(gap_x as f32, 0.0));
            }
            if let Some(gap_y) = parse_grid_value(token, "grid-gap-y-") {
                return Some(sb.display_grid().gap_px(0.0, gap_y as f32));
            }

            // Grid area utilities
            if token.starts_with("grid-area-") {
                let area_name = token.strip_prefix("grid-area-")?;
                return Some(sb.display_grid().grid_area(area_name));
            }

            // Grid template areas (simplified)
            match token {
                "grid-template-areas-sidebar" => {
                    // Common sidebar layout
                    Some(
                        sb.display_grid()
                            .grid_template_areas(&["sidebar main", "sidebar main"]),
                    )
                }
                "grid-template-areas-header" => {
                    // Common header layout
                    Some(sb.display_grid().grid_template_areas(&[
                        "header header",
                        "main main",
                        "footer footer",
                    ]))
                }
                _ => None,
            }
        }
    }
}

/// Apply position utilities
pub fn apply_position(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Basic positioning
    match token {
        "static" => return Some(sb.position_static()),
        "relative" => return Some(sb.position_relative()),
        "absolute" => return Some(sb.position_absolute()),
        "fixed" => return Some(sb.position_fixed()),
        "sticky" => return Some(sb.position_sticky()),
        _ => {}
    }

    // Try positioning utilities
    if let Some(result) = apply_positioning_utilities(token, sb.clone()) {
        return Some(result);
    }

    // Try z-index utilities
    if let Some(z) = parse_z_index(token) {
        return Some(sb.z_index(z));
    }

    None
}

/// Apply advanced positioning utilities
pub fn apply_positioning_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Inset utilities - all sides
    match token {
        "inset-0" => return Some(sb.inset_all(0.0)),
        "inset-auto" => return Some(sb.inset_all_auto()),
        "inset-full" => return Some(sb.inset_all(100.0)),
        "inset-1/2" => return Some(sb.inset_all(50.0)),
        _ => {}
    }

    // Try parsing inset-[value]
    if let Some(value_str) = token.strip_prefix("inset-") {
        if let Ok(value) = value_str.parse::<f32>() {
            return Some(sb.inset_all(value));
        }
        // Try percentage values
        if value_str.ends_with('%') {
            if let Ok(pct) = value_str.trim_end_matches('%').parse::<f32>() {
                return Some(sb.inset_all(pct));
            }
        }
    }

    // Individual side insets
    if let Some(result) = apply_inset_side(token, sb.clone()) {
        return Some(result);
    }

    None
}

/// Apply individual side inset utilities
pub fn apply_inset_side(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Top insets
    if let Some(value_str) = token.strip_prefix("top-") {
        return parse_inset_value(value_str).map(|v| sb.inset_top(v));
    }

    // Right insets
    if let Some(value_str) = token.strip_prefix("right-") {
        return parse_inset_value(value_str).map(|v| sb.inset_right(v));
    }

    // Bottom insets
    if let Some(value_str) = token.strip_prefix("bottom-") {
        return parse_inset_value(value_str).map(|v| sb.inset_bottom(v));
    }

    // Left insets
    if let Some(value_str) = token.strip_prefix("left-") {
        return parse_inset_value(value_str).map(|v| sb.inset_left(v));
    }

    None
}

/// Parse inset value from string
fn parse_inset_value(value_str: &str) -> Option<f32> {
    match value_str {
        "0" => Some(0.0),
        "auto" => Some(0.0), // Auto treated as 0 in TUI
        "full" => Some(100.0),
        "1/2" => Some(50.0),
        "1/3" => Some(33.33),
        "2/3" => Some(66.67),
        "1/4" => Some(25.0),
        "3/4" => Some(75.0),
        _ => {
            // Try parsing as number
            if let Ok(value) = value_str.parse::<f32>() {
                Some(value)
            } else if value_str.ends_with('%') {
                // Try percentage
                value_str.trim_end_matches('%').parse::<f32>().ok()
            } else {
                None
            }
        }
    }
}

/// Apply order utilities
pub fn apply_order(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        "order-first" => {
            // First order gets very low z-index
            Some(sb.z_index(-9999))
        }
        "order-last" => {
            // Last order gets very high z-index
            Some(sb.z_index(9999))
        }
        _ => {
            // Try parsing order-N
            if let Some(order_str) = token.strip_prefix("order-") {
                if let Ok(order) = order_str.parse::<i32>() {
                    // Map order to z-index for TUI
                    Some(sb.z_index(order))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/// Apply overflow utilities (critical for TUI)
pub fn apply_overflow(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    match token {
        // Basic overflow
        "overflow-hidden" => Some(sb.overflow_hidden()),
        "overflow-scroll" => Some(sb.overflow_scroll()),
        "overflow-auto" => Some(sb.overflow_auto()),
        "overflow-visible" => Some(sb.overflow_visible()),

        // X-axis overflow
        "overflow-x-hidden" => Some(sb.overflow_x_hidden()),
        "overflow-x-scroll" => Some(sb.overflow_x_scroll()),
        "overflow-x-auto" => Some(sb.overflow_x_auto()),
        "overflow-x-visible" => Some(sb.overflow_x_visible()),

        // Y-axis overflow
        "overflow-y-hidden" => Some(sb.overflow_y_hidden()),
        "overflow-y-scroll" => Some(sb.overflow_y_scroll()),
        "overflow-y-auto" => Some(sb.overflow_y_auto()),
        "overflow-y-visible" => Some(sb.overflow_y_visible()),

        _ => None,
    }
}

/// Apply all layout utilities
pub fn apply_layout_utilities(token: &str, sb: StyleBuilder) -> Option<StyleBuilder> {
    // Try display utilities
    if let Some(result) = apply_display(token, sb.clone()) {
        return Some(result);
    }

    // Try flexbox utilities
    if let Some(result) = apply_flexbox(token, sb.clone()) {
        return Some(result);
    }

    // Try justify-content
    if let Some(result) = apply_justify_content(token, sb.clone()) {
        return Some(result);
    }

    // Try align-items
    if let Some(result) = apply_align_items(token, sb.clone()) {
        return Some(result);
    }

    // Try align-self utilities
    if let Some(result) = apply_align_self(token, sb.clone()) {
        return Some(result);
    }

    // Try place-items utilities
    if let Some(result) = apply_place_items(token, sb.clone()) {
        return Some(result);
    }

    // Try grid utilities
    if let Some(result) = apply_grid(token, sb.clone()) {
        return Some(result);
    }

    // Try order utilities
    if let Some(result) = apply_order(token, sb.clone()) {
        return Some(result);
    }

    // Try overflow utilities
    if let Some(result) = apply_overflow(token, sb.clone()) {
        return Some(result);
    }

    // Try position utilities
    if let Some(result) = apply_position(token, sb) {
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_display("flex", sb).expect("Should apply flex display");
        let style = result.build();
        assert_eq!(style.display, taffy::style::Display::Flex);

        let sb = StyleBuilder::new();
        let result = apply_display("grid", sb).expect("Should apply grid display");
        let style = result.build();
        assert_eq!(style.display, taffy::style::Display::Grid);
    }

    #[test]
    fn test_flexbox_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_flexbox("flex-col", sb).expect("Should apply flex-col");
        let style = result.build();
        assert_eq!(style.flex_direction, taffy::style::FlexDirection::Column);

        let sb = StyleBuilder::new();
        let result = apply_flexbox("flex-1", sb).expect("Should apply flex-1");
        let style = result.build();
        assert_eq!(style.flex_grow, 1.0);
    }

    #[test]
    fn test_justify_content_utilities() {
        let sb = StyleBuilder::new();

        let result =
            apply_justify_content("justify-center", sb).expect("Should apply justify-center");
        let style = result.build();
        assert_eq!(
            style.justify_content,
            Some(taffy::style::JustifyContent::Center)
        );
    }

    #[test]
    fn test_align_items_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_align_items("items-center", sb).expect("Should apply items-center");
        let style = result.build();
        assert_eq!(style.align_items, Some(taffy::style::AlignItems::Center));
    }

    #[test]
    fn test_position_utilities() {
        let sb = StyleBuilder::new();

        let result = apply_position("absolute", sb).expect("Should apply absolute position");
        assert_eq!(result.get_z_index(), Some(10));

        let sb = StyleBuilder::new();
        let result = apply_position("z-50", sb).expect("Should apply z-index 50");
        assert_eq!(result.get_z_index(), Some(50));
    }
}
