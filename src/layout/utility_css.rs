use super::style::{AlignItems, Direction, GridAutoFlow, JustifyContent, StyleBuilder};
// Utility: align-self / place-items/content mapping

use super::colors::parse_color_token;

fn parse_px(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        let trimmed = n.strip_suffix("px").unwrap_or(n);
        trimmed.parse::<f32>().ok()
    })
}

/// Parse Tailwind spacing scale (0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 72, 80, 96)
fn parse_spacing(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        match n {
            "0" => Some(0.0),
            "0.5" => Some(2.0),
            "1" => Some(4.0),
            "1.5" => Some(6.0),
            "2" => Some(8.0),
            "2.5" => Some(10.0),
            "3" => Some(12.0),
            "3.5" => Some(14.0),
            "4" => Some(16.0),
            "5" => Some(20.0),
            "6" => Some(24.0),
            "7" => Some(28.0),
            "8" => Some(32.0),
            "9" => Some(36.0),
            "10" => Some(40.0),
            "11" => Some(44.0),
            "12" => Some(48.0),
            "14" => Some(56.0),
            "16" => Some(64.0),
            "20" => Some(80.0),
            "24" => Some(96.0),
            "28" => Some(112.0),
            "32" => Some(128.0),
            "36" => Some(144.0),
            "40" => Some(160.0),
            "44" => Some(176.0),
            "48" => Some(192.0),
            "52" => Some(208.0),
            "56" => Some(224.0),
            "60" => Some(240.0),
            "64" => Some(256.0),
            "72" => Some(288.0),
            "80" => Some(320.0),
            "96" => Some(384.0),
            _ => n.parse::<f32>().ok().map(|v| v * 4.0), // Fallback: treat as rem * 16px
        }
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
            "flex-auto" => {
                sb = sb.display_flex().flex_grow(1.0).flex_shrink(1.0);
            }
            "flex-initial" => {
                sb = sb.display_flex().flex_grow(0.0).flex_shrink(1.0);
            }
            "flex-none" => {
                sb = sb.display_flex().flex_grow(0.0).flex_shrink(0.0);
            }
            "flex-wrap" => {
                sb = sb.display_flex().flex_wrap(true);
            }
            "flex-nowrap" => {
                sb = sb.display_flex().flex_wrap(false);
            }
            "grow" | "grow-1" => {
                sb = sb.flex_grow(1.0);
            }
            "grow-0" => {
                sb = sb.flex_grow(0.0);
            }
            "shrink" | "shrink-1" => {
                sb = sb.flex_shrink(1.0);
            }
            "shrink-0" => {
                sb = sb.flex_shrink(0.0);
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
            "inline-block" => {
                sb = sb.display_flex().direction(Direction::Row);
            }
            "hidden" => {
                // In TUI context, we can use zero size to hide
                sb = sb.size_px(Some(0.0), Some(0.0));
            }

            // Position utilities for overlays and modals
            "static" => {
                // Default positioning - normal document flow
                sb = sb.z_index(0);
            }
            "relative" => {
                // Relative positioning - can be positioned relative to normal position
                // Useful for dropdowns and tooltips
                sb = sb.z_index(1);
            }
            "absolute" => {
                // Absolute positioning - positioned relative to nearest positioned ancestor
                // Essential for modals, popovers, overlays
                sb = sb.z_index(10);
            }
            "fixed" => {
                // Fixed positioning - positioned relative to viewport
                // Critical for modals, notifications, sticky headers
                sb = sb.z_index(50);
            }
            "sticky" => {
                // Sticky positioning - toggles between relative and fixed
                // Useful for sticky headers, sidebars
                sb = sb.z_index(20);
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
            // Typography utilities
            "font-thin" => {
                // TUI doesn't have font weights, but we can mark it
            }
            "font-extralight" => {
                // TUI doesn't have font weights, but we can mark it
            }
            "font-light" => {
                // TUI doesn't have font weights, but we can mark it
            }
            "font-normal" => {
                sb = sb.bold(false);
            }
            "font-medium" => {
                // TUI doesn't have font weights, but we can mark it
            }
            "font-semibold" => {
                sb = sb.bold(true);
            }
            "font-bold" => {
                sb = sb.bold(true);
            }
            "font-extrabold" => {
                sb = sb.bold(true);
            }
            "font-black" => {
                sb = sb.bold(true);
            }
            "italic" => {
                sb = sb.italic(true);
            }
            "not-italic" => {
                sb = sb.italic(false);
            }
            "underline" => {
                sb = sb.underline(true);
            }
            "no-underline" => {
                sb = sb.underline(false);
            }
            "line-through" => {
                sb = sb.strike(true);
            }
            "no-line-through" => {
                sb = sb.strike(false);
            }
            "reverse" => {
                sb = sb.reverse(true);
            }
            "no-reverse" => {
                sb = sb.reverse(false);
            }

            // Text size utilities (TUI-adapted - these would affect character scaling if supported)
            "text-xs" => {
                // Extra small text - 0.75rem
            }
            "text-sm" => {
                // Small text - 0.875rem
            }
            "text-base" => {
                // Base text - 1rem (default)
            }
            "text-lg" => {
                // Large text - 1.125rem
            }
            "text-xl" => {
                // Extra large text - 1.25rem
            }
            "text-2xl" => {
                // 2x large text - 1.5rem
            }
            "text-3xl" => {
                // 3x large text - 1.875rem
            }
            "text-4xl" => {
                // 4x large text - 2.25rem
            }
            "text-5xl" => {
                // 5x large text - 3rem
            }
            "text-6xl" => {
                // 6x large text - 3.75rem
            }
            "text-7xl" => {
                // 7x large text - 4.5rem
            }
            "text-8xl" => {
                // 8x large text - 6rem
            }
            "text-9xl" => {
                // 9x large text - 8rem
            }

            // Text alignment
            "text-left" => {
                // Left align text - would need text layout support
            }
            "text-center" => {
                // Center align text - would need text layout support
            }
            "text-right" => {
                // Right align text - would need text layout support
            }
            "text-justify" => {
                // Justify text - would need text layout support
            }
            _ => {
                if let Some(px) = parse_px(t, "w-") {
                    sb = sb.size_px(Some(px), None);
                    continue;
                }
                if let Some(px) = parse_px(t, "h-") {
                    sb = sb.size_px(None, Some(px));
                    continue;
                }
                if let Some(p) = parse_pct(t, "w-") {
                    sb = sb.width_pct(p);
                    continue;
                }
                if let Some(p) = parse_pct(t, "h-") {
                    sb = sb.height_pct(p);
                    continue;
                }
                if let Some(px) = parse_px(t, "min-w-") {
                    sb = sb.min_size_px(Some(px), None);
                    continue;
                }
                if let Some(px) = parse_px(t, "min-h-") {
                    sb = sb.min_size_px(None, Some(px));
                    continue;
                }
                if let Some(px) = parse_px(t, "max-w-") {
                    sb = sb.max_size_px(Some(px), None);
                    continue;
                }
                if let Some(px) = parse_px(t, "max-h-") {
                    sb = sb.max_size_px(None, Some(px));
                    continue;
                }
                // Padding/Margin shorthands - use Tailwind spacing scale
                if let Some(px) = parse_spacing(t, "p-") {
                    sb = sb.padding_all_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "px-") {
                    sb = sb.padding_x_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "py-") {
                    sb = sb.padding_y_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "pl-") {
                    sb = sb.padding_l_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "pr-") {
                    sb = sb.padding_r_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "pt-") {
                    sb = sb.padding_t_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "pb-") {
                    sb = sb.padding_b_px(px);
                    continue;
                }

                // Margin utilities with Tailwind spacing scale
                if let Some(px) = parse_spacing(t, "m-") {
                    sb = sb.margin_all_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "mx-") {
                    sb = sb.margin_x_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "my-") {
                    sb = sb.margin_y_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "ml-") {
                    sb = sb.margin_l_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "mr-") {
                    sb = sb.margin_r_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "mt-") {
                    sb = sb.margin_t_px(px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "mb-") {
                    sb = sb.margin_b_px(px);
                    continue;
                }

                // Gap utilities with Tailwind spacing scale
                if let Some(px) = parse_spacing(t, "gap-") {
                    sb = sb.gap_px(px, px);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "gap-x-") {
                    sb = sb.gap_px(px, 0.0);
                    continue;
                }
                if let Some(px) = parse_spacing(t, "gap-y-") {
                    sb = sb.gap_px(0.0, px);
                    continue;
                }

                // Border utilities (TUI-adapted)
                if t == "border" {
                    // Default border - could be implemented as outline characters or bg color
                    // For now, we could add a subtle background to simulate border
                    if !sb.has_bg_color() {
                        sb = sb.bg_rgba(0.2, 0.2, 0.2, 1.0); // Dark gray border effect
                    }
                    continue;
                }
                if t.starts_with("border-")
                    && (t.contains("gray") || t.contains("black") || t.contains("white"))
                {
                    // Border colors - apply as background color for border effect
                    // This is a TUI approximation of borders
                    continue;
                }
                if t.starts_with("rounded") {
                    // Border radius - not applicable in TUI, but we acknowledge it
                    continue;
                }

                // Opacity utilities
                if let Some(opacity_str) = t.strip_prefix("opacity-") {
                    if let Ok(opacity) = opacity_str.parse::<u8>() {
                        let alpha = (opacity as f32 / 100.0).clamp(0.0, 1.0);
                        sb = sb.opacity(alpha);
                        continue;
                    }
                }

                // Z-index utilities for layering (modals, popovers, dropdowns)
                match t {
                    "z-auto" => {
                        sb = sb.z_index(0);
                        continue;
                    }
                    "z-0" => {
                        sb = sb.z_index(0);
                        continue;
                    }
                    "z-10" => {
                        sb = sb.z_index(10);
                        continue;
                    }
                    "z-20" => {
                        sb = sb.z_index(20);
                        continue;
                    }
                    "z-30" => {
                        sb = sb.z_index(30);
                        continue;
                    }
                    "z-40" => {
                        sb = sb.z_index(40);
                        continue;
                    }
                    "z-50" => {
                        sb = sb.z_index(50);
                        continue;
                    }
                    _ => {}
                }

                // Custom z-index values
                if let Some(z_str) = t.strip_prefix("z-") {
                    if let Ok(z_index) = z_str.parse::<i32>() {
                        sb = sb.z_index(z_index);
                        continue;
                    }
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
