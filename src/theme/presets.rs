use super::variables::ThemeVariables;
use super::Theme;

/// Dark theme preset - optimized for low-light environments
pub fn dark_theme() -> Theme {
    Theme::new("dark")
        .with_variables(
            ThemeVariables::new()
                .colors()
                    // Brand colors
                    .primary(59, 130, 246)      // Blue-500
                    .secondary(168, 85, 247)     // Purple-500
                    
                    // Background hierarchy
                    .background(17, 24, 39)      // Gray-900
                    .background_secondary(31, 41, 55)  // Gray-800
                    .background_tertiary(55, 65, 81)   // Gray-700
                    
                    // Text hierarchy
                    .text(243, 244, 246)         // Gray-100
                    .text_secondary(209, 213, 219) // Gray-300
                    .text_muted(156, 163, 175)   // Gray-400
                    
                    // Borders
                    .border(75, 85, 99)          // Gray-600
                    .border_focus(59, 130, 246)  // Blue-500
                    
                    // Semantic colors
                    .success(34, 197, 94)        // Green-500
                    .warning(245, 158, 11)       // Amber-500
                    .error(239, 68, 68)          // Red-500
                    .info(59, 130, 246)          // Blue-500
                    .build()
                    
                .spacing()
                    .xs(1)
                    .sm(2)
                    .md(4)
                    .lg(6)
                    .xl(8)
                    .xxl(12)
                    .build()
                    
                .typography()
                    .size_xs(10)
                    .size_sm(12)
                    .size_md(14)
                    .size_lg(18)
                    .size_xl(24)
                    .weight_normal()
                    .weight_bold()
                    .build()
                    
                .borders()
                    .width(1)
                    .radius_sm(2)
                    .radius_md(4)
                    .radius_lg(8)
                    .style("solid")
                    .build()
                    
                .shadows()
                    .none()
                    .sm()
                    .md()
                    .lg()
                    .xl()
                    .build()
        )
}

/// Light theme preset - optimized for bright environments
pub fn light_theme() -> Theme {
    Theme::new("light")
        .with_variables(
            ThemeVariables::new()
                .colors()
                    // Brand colors
                    .primary(37, 99, 235)        // Blue-600
                    .secondary(147, 51, 234)      // Purple-600
                    
                    // Background hierarchy
                    .background(255, 255, 255)   // White
                    .background_secondary(249, 250, 251) // Gray-50
                    .background_tertiary(243, 244, 246)  // Gray-100
                    
                    // Text hierarchy
                    .text(17, 24, 39)            // Gray-900
                    .text_secondary(55, 65, 81)  // Gray-700
                    .text_muted(107, 114, 128)   // Gray-500
                    
                    // Borders
                    .border(229, 231, 235)       // Gray-200
                    .border_focus(37, 99, 235)   // Blue-600
                    
                    // Semantic colors
                    .success(22, 163, 74)        // Green-600
                    .warning(217, 119, 6)        // Amber-600
                    .error(220, 38, 38)          // Red-600
                    .info(37, 99, 235)           // Blue-600
                    .build()
                    
                .spacing()
                    .xs(1)
                    .sm(2)
                    .md(4)
                    .lg(6)
                    .xl(8)
                    .xxl(12)
                    .build()
                    
                .typography()
                    .size_xs(10)
                    .size_sm(12)
                    .size_md(14)
                    .size_lg(18)
                    .size_xl(24)
                    .weight_normal()
                    .weight_bold()
                    .build()
                    
                .borders()
                    .width(1)
                    .radius_sm(2)
                    .radius_md(4)
                    .radius_lg(8)
                    .style("solid")
                    .build()
                    
                .shadows()
                    .none()
                    .sm()
                    .md()
                    .lg()
                    .xl()
                    .build()
        )
}

/// High contrast theme - optimized for accessibility
pub fn high_contrast_theme() -> Theme {
    Theme::new("high_contrast")
        .with_variables(
            ThemeVariables::new()
                .colors()
                    // High contrast brand colors
                    .primary(0, 127, 255)        // Bright blue
                    .secondary(255, 0, 255)       // Bright magenta
                    
                    // Maximum contrast backgrounds
                    .background(0, 0, 0)         // Pure black
                    .background_secondary(20, 20, 20)   // Near black
                    .background_tertiary(40, 40, 40)    // Dark gray
                    
                    // Maximum contrast text
                    .text(255, 255, 255)         // Pure white
                    .text_secondary(230, 230, 230) // Near white
                    .text_muted(200, 200, 200)   // Light gray
                    
                    // High visibility borders
                    .border(255, 255, 255)       // White borders
                    .border_focus(255, 255, 0)   // Yellow focus
                    
                    // High contrast semantic colors
                    .success(0, 255, 0)          // Bright green
                    .warning(255, 255, 0)        // Bright yellow
                    .error(255, 0, 0)            // Bright red
                    .info(0, 255, 255)           // Bright cyan
                    .build()
                    
                .spacing()
                    .xs(1)
                    .sm(2)
                    .md(4)
                    .lg(6)
                    .xl(8)
                    .xxl(12)
                    .build()
                    
                .typography()
                    .size_xs(12)  // Larger minimum size
                    .size_sm(14)
                    .size_md(16)
                    .size_lg(20)
                    .size_xl(26)
                    .weight_normal()
                    .weight_bold()
                    .build()
                    
                .borders()
                    .width(2)  // Thicker borders for visibility
                    .radius_sm(0)  // Less rounding for clarity
                    .radius_md(2)
                    .radius_lg(4)
                    .style("solid")
                    .build()
                    
                .shadows()
                    .none()
                    .sm()
                    .md()
                    .lg()
                    .xl()
                    .build()
        )
}

/// Solarized Dark theme
pub fn solarized_dark_theme() -> Theme {
    Theme::new("solarized_dark")
        .with_variables(
            ThemeVariables::new()
                .colors()
                    .primary(38, 139, 210)       // Solarized blue
                    .secondary(108, 113, 196)     // Solarized violet
                    .background(0, 43, 54)       // Base03
                    .background_secondary(7, 54, 66)     // Base02
                    .background_tertiary(88, 110, 117)   // Base01
                    .text(253, 246, 227)         // Base3
                    .text_secondary(238, 232, 213) // Base2
                    .text_muted(147, 161, 161)   // Base1
                    .border(101, 123, 131)       // Base00
                    .border_focus(38, 139, 210)  // Blue
                    .success(133, 153, 0)        // Green
                    .warning(181, 137, 0)        // Yellow
                    .error(211, 1, 2)            // Red
                    .info(42, 161, 152)          // Cyan
                    .build()
                    
                .spacing()
                    .xs(1).sm(2).md(4).lg(6).xl(8).xxl(12)
                    .build()
                    
                .typography()
                    .size_xs(10).size_sm(12).size_md(14).size_lg(18).size_xl(24)
                    .weight_normal().weight_bold()
                    .build()
                    
                .borders()
                    .width(1).radius_sm(2).radius_md(4).radius_lg(8).style("solid")
                    .build()
                    
                .shadows()
                    .none().sm().md().lg().xl()
                    .build()
        )
}

/// Gruvbox Dark theme
pub fn gruvbox_dark_theme() -> Theme {
    Theme::new("gruvbox_dark")
        .with_variables(
            ThemeVariables::new()
                .colors()
                    .primary(131, 165, 152)      // Gruvbox blue
                    .secondary(211, 134, 155)     // Gruvbox purple
                    .background(40, 40, 40)      // bg0
                    .background_secondary(60, 56, 54)    // bg1
                    .background_tertiary(80, 73, 69)     // bg2
                    .text(251, 241, 199)         // fg0
                    .text_secondary(235, 219, 178) // fg1
                    .text_muted(189, 174, 147)   // fg3
                    .border(124, 111, 100)       // bg4
                    .border_focus(131, 165, 152) // Blue
                    .success(184, 187, 38)       // Green
                    .warning(250, 189, 47)       // Yellow
                    .error(251, 73, 52)          // Red
                    .info(131, 165, 152)         // Aqua
                    .build()
                    
                .spacing()
                    .xs(1).sm(2).md(4).lg(6).xl(8).xxl(12)
                    .build()
                    
                .typography()
                    .size_xs(10).size_sm(12).size_md(14).size_lg(18).size_xl(24)
                    .weight_normal().weight_bold()
                    .build()
                    
                .borders()
                    .width(1).radius_sm(2).radius_md(4).radius_lg(8).style("solid")
                    .build()
                    
                .shadows()
                    .none().sm().md().lg().xl()
                    .build()
        )
}