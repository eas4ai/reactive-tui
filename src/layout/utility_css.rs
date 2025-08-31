use super::style::{AlignItems, Direction, GridAutoFlow, JustifyContent, StyleBuilder};
// Utility: align-self / place-items/content mapping

use super::colors::parse_color_token;

fn parse_px(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        let trimmed = n.strip_suffix("px").unwrap_or(n);
        trimmed.parse::<f32>().ok()
    })
}

fn parse_pct(token: &str, prefix: &str) -> Option<f32> {
    token
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix('%'))
        .and_then(|n| n.parse::<f32>().ok())
}

pub fn apply_utility_classes(class: &str, mut sb: StyleBuilder) -> StyleBuilder {
    for t in class.split_whitespace() {
        match t {
            "flex" => {
                sb = sb.display_flex();
            }
            "flex-1" => {
                sb = sb.display_flex().flex_grow(1.0);
            }
            "grid" => {
                sb = sb.display_grid();
            }
            "flex-row" => {
                sb = sb.direction(Direction::Row);
            }
            "flex-col" => {
                sb = sb.direction(Direction::Column);
            }
            "justify-start" => {
                sb = sb.justify(JustifyContent::Start);
            }
            "justify-center" => {
                sb = sb.justify(JustifyContent::Center);
            }
            "justify-end" => {
                sb = sb.justify(JustifyContent::End);
            }
            "justify-between" => {
                sb = sb.justify(JustifyContent::SpaceBetween);
            }
            "justify-around" => {
                sb = sb.justify(JustifyContent::SpaceAround);
            }
            "justify-evenly" => {
                sb = sb.justify(JustifyContent::SpaceEvenly);
            }
            "items-start" => {
                sb = sb.align(AlignItems::Start);
            }
            "items-center" => {
                sb = sb.align(AlignItems::Center);
            }
            "items-end" => {
                sb = sb.align(AlignItems::End);
            }

            // Screen dimensions
            "h-screen" => {
                sb = sb.height_pct(100.0);
            }
            "w-screen" => {
                sb = sb.width_pct(100.0);
            }

            // Full dimensions
            "h-full" => {
                sb = sb.height_pct(100.0);
            }
            "w-full" => {
                sb = sb.width_pct(100.0);
            }

            // Container utilities
            "container" => {
                sb = sb.width_pct(100.0);
            }

            // Display utilities
            "block" => {
                sb = sb.display_flex().direction(Direction::Column);
            }
            "inline" => {
                sb = sb.display_flex().direction(Direction::Row);
            }

            // Overflow utilities (placeholders for now)
            "overflow-hidden" | "overflow-auto" => {
                // These would need special handling in the layout system
                continue;
            }

            // Spacing utilities (space-x, space-y)
            // These create gaps between child elements
            "space-x-1" => {
                sb = sb.gap_px(4.0, 0.0);
            }
            "space-x-2" => {
                sb = sb.gap_px(8.0, 0.0);
            }
            "space-x-3" => {
                sb = sb.gap_px(12.0, 0.0);
            }
            "space-x-4" => {
                sb = sb.gap_px(16.0, 0.0);
            }
            "space-y-1" => {
                sb = sb.gap_px(0.0, 4.0);
            }
            "space-y-2" => {
                sb = sb.gap_px(0.0, 8.0);
            }
            "space-y-3" => {
                sb = sb.gap_px(0.0, 12.0);
            }
            "space-y-4" => {
                sb = sb.gap_px(0.0, 16.0);
            }
            "space-y-6" => {
                sb = sb.gap_px(0.0, 24.0);
            }
            "items-stretch" => {
                sb = sb.align(AlignItems::Stretch);
            }
            "font-bold" => {
                sb = sb.bold(true);
            }
            "italic" => {
                sb = sb.italic(true);
            }
            "underline" => {
                sb = sb.underline(true);
            }
            "reverse" => {
                sb = sb.reverse(true);
            }
            "line-through" => {
                sb = sb.strike(true);
            }
            _ => {
                match parse_px(t, "w-") {
                    Some(px) => {
                        sb = sb.size_px(Some(px), None);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "h-") {
                    Some(px) => {
                        sb = sb.size_px(None, Some(px));
                        continue;
                    }
                    None => {}
                }
                match parse_pct(t, "w-") {
                    Some(p) => {
                        sb = sb.width_pct(p);
                        continue;
                    }
                    None => {}
                }
                match parse_pct(t, "h-") {
                    Some(p) => {
                        sb = sb.height_pct(p);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "min-w-") {
                    Some(px) => {
                        sb = sb.min_size_px(Some(px), None);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "min-h-") {
                    Some(px) => {
                        sb = sb.min_size_px(None, Some(px));
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "max-w-") {
                    Some(px) => {
                        sb = sb.max_size_px(Some(px), None);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "max-h-") {
                    Some(px) => {
                        sb = sb.max_size_px(None, Some(px));
                        continue;
                    }
                    None => {}
                }
                // Padding/Margin shorthands
                match parse_px(t, "p-") {
                    Some(px) => {
                        sb = sb.padding_all_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "px-") {
                    Some(px) => {
                        sb = sb.padding_x_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "py-") {
                    Some(px) => {
                        sb = sb.padding_y_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "pl-") {
                    Some(px) => {
                        sb = sb.padding_l_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "pr-") {
                    Some(px) => {
                        sb = sb.padding_r_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "pt-") {
                    Some(px) => {
                        sb = sb.padding_t_px(px);
                        continue;
                    }
                    None => {}
                }
                match parse_px(t, "pb-") {
                    Some(px) => {
                        sb = sb.padding_b_px(px);
                        continue;
                    }
                    None => {}
                }

                if let Some(px) = parse_px(t, "m-") {
                    sb = sb.margin_all_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "mx-") {
                    sb = sb.margin_x_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "my-") {
                    sb = sb.margin_y_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "ml-") {
                    sb = sb.margin_l_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "mr-") {
                    sb = sb.margin_r_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "mt-") {
                    sb = sb.margin_t_px(px);
                    continue;
                }
                if let Some(px) = parse_px(t, "mb-") {
                    sb = sb.margin_b_px(px);
                    continue;
                }

                // Gap
                if let Some(px) = parse_px(t, "gap-") {
                    sb = sb.gap_px(px, px);
                    continue;
                }
                if let Some(px) = parse_px(t, "gap-x-") {
                    sb = sb.gap_px(px, 0.0);
                    continue;
                }
                if let Some(px) = parse_px(t, "gap-y-") {
                    sb = sb.gap_px(0.0, px);
                    continue;
                }

                // Grid
                if let Some(n) = t
                    .strip_prefix("grid-cols-")
                    .and_then(|n| n.parse::<u16>().ok())
                {
                    sb = sb.grid_cols(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("grid-rows-")
                    .and_then(|n| n.parse::<u16>().ok())
                {
                    sb = sb.grid_rows(n);
                    continue;
                }
                if t == "grid-flow-row" {
                    sb = sb.grid_auto_flow(GridAutoFlow::Row);
                    continue;
                }
                if t == "grid-flow-col" {
                    sb = sb.grid_auto_flow(GridAutoFlow::Column);
                    continue;
                }
                if t == "grid-flow-row-dense" {
                    sb = sb.grid_auto_flow(GridAutoFlow::RowDense);
                    continue;
                }
                if t == "grid-flow-col-dense" {
                    sb = sb.grid_auto_flow(GridAutoFlow::ColumnDense);
                    continue;
                }

                if t == "overflow-clip" {
                    sb = sb.overflow_clip();
                    continue;
                }
                // Placement
                if let Some(n) = t
                    .strip_prefix("col-span-")
                    .and_then(|n| n.parse::<u16>().ok())
                {
                    sb = sb.col_span(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("row-span-")
                    .and_then(|n| n.parse::<u16>().ok())
                {
                    sb = sb.row_span(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("col-start-")
                    .and_then(|n| n.parse::<i16>().ok())
                {
                    sb = sb.col_start(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("col-end-")
                    .and_then(|n| n.parse::<i16>().ok())
                {
                    sb = sb.col_end(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("row-start-")
                    .and_then(|n| n.parse::<i16>().ok())
                {
                    sb = sb.row_start(n);
                    continue;
                }
                if let Some(n) = t
                    .strip_prefix("row-end-")
                    .and_then(|n| n.parse::<i16>().ok())
                {
                    sb = sb.row_end(n);
                    continue;
                }

                // Colors & text attributes
                if let Some((r, g, b, a)) = t.strip_prefix("text-").and_then(parse_color_token) {
                    sb = sb.text_rgba(r, g, b, a);
                    continue;
                }
                if let Some((r, g, b, a)) = t.strip_prefix("bg-").and_then(parse_color_token) {
                    sb = sb.bg_rgba(r, g, b, a);
                    continue;
                }
                // Place items/content (subset)
                if t == "place-items-start" {
                    sb = sb.align(AlignItems::Start);
                    continue;
                }
                if t == "place-items-center" {
                    sb = sb.align(AlignItems::Center);
                    continue;
                }
                if t == "place-items-end" {
                    sb = sb.align(AlignItems::End);
                    continue;
                }
                if t == "place-items-stretch" {
                    sb = sb.align(AlignItems::Stretch);
                    continue;
                }
                if t == "place-content-center" {
                    sb = sb.justify(JustifyContent::Center);
                    continue;
                }
                if t == "place-content-start" {
                    sb = sb.justify(JustifyContent::Start);
                    continue;
                }
                if t == "place-content-end" {
                    sb = sb.justify(JustifyContent::End);
                    continue;
                }

                if t == "place-content-between" {
                    sb = sb.align_content(JustifyContent::SpaceBetween);
                    continue;
                }
                if t == "place-content-around" {
                    sb = sb.align_content(JustifyContent::SpaceAround);
                    continue;
                }
                if t == "place-content-evenly" {
                    sb = sb.align_content(JustifyContent::SpaceEvenly);
                    continue;
                }

                // Alignment utilities (subset)
                if t == "self-start" {
                    sb = sb.align_self(super::style::AlignSelf::Start);
                    continue;
                }
                if t == "self-center" {
                    sb = sb.align_self(super::style::AlignSelf::Center);
                    continue;
                }
                if t == "self-end" {
                    sb = sb.align_self(super::style::AlignSelf::End);
                    continue;
                }
                if t == "self-stretch" {
                    sb = sb.align_self(super::style::AlignSelf::Stretch);
                    continue;
                }
            }
        }

        // Image-specific utility classes
        if t.starts_with("image-") || t.starts_with("aspect-ratio-") {
            match t {
                "image-fit-cover" => {
                    // Equivalent to object-fit: cover - scale to fill container while preserving aspect ratio
                    // This would be handled by the image widget itself
                    continue;
                }
                "image-fit-contain" => {
                    // Equivalent to object-fit: contain - scale to fit within container while preserving aspect ratio
                    continue;
                }
                "image-fit-fill" => {
                    // Equivalent to object-fit: fill - stretch to fill container (may distort aspect ratio)
                    continue;
                }
                "image-quality-fast" => {
                    // Fast rendering with lower quality
                    continue;
                }
                "image-quality-balanced" => {
                    // Balanced quality and performance
                    continue;
                }
                "image-quality-high" => {
                    // High quality rendering (slower)
                    continue;
                }
                "aspect-ratio-16-9" => {
                    // 16:9 aspect ratio constraint
                    continue;
                }
                "aspect-ratio-4-3" => {
                    // 4:3 aspect ratio constraint
                    continue;
                }
                "aspect-ratio-1-1" => {
                    // Square aspect ratio
                    continue;
                }
                "image-rendering-pixelated" => {
                    // Pixelated/nearest-neighbor rendering
                    continue;
                }
                "image-rendering-smooth" => {
                    // Smooth/anti-aliased rendering
                    continue;
                }
                _ => {}
            }
        }
    }
    sb
}
