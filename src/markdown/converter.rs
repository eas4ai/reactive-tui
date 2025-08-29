use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba};

/// Create an Rgba color from 0-255 values
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

/// Utility functions for converting between different styled text representations
/// and markdown-specific styling operations
///
/// Convert a plain text string to a single StyledRun with default style
pub fn plain_text_to_styled_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        Rgba::white(),       // White text
        Rgba::transparent(), // Transparent background
        Attr::empty(),       // No attributes
    )
}

/// Convert multiple plain text lines to StyledLine vector
pub fn plain_text_to_styled_lines(text: &str) -> Vec<StyledLine> {
    text.lines()
        .map(|line| {
            let mut styled_line = StyledLine::new();
            styled_line.push(plain_text_to_styled_run(line));
            styled_line
        })
        .collect()
}

/// Merge adjacent styled runs with the same style to optimize rendering
pub fn optimize_styled_runs(runs: Vec<StyledRun>) -> Vec<StyledRun> {
    if runs.is_empty() {
        return runs;
    }

    let mut optimized = Vec::new();
    let mut current_run = runs[0].clone();

    for run in runs.into_iter().skip(1) {
        if run.fg == current_run.fg && run.bg == current_run.bg && run.attr == current_run.attr {
            current_run.text.push_str(&run.text);
        } else {
            optimized.push(current_run);
            current_run = run;
        }
    }

    optimized.push(current_run);
    optimized
}

/// Optimize an entire StyledLine by merging adjacent runs
pub fn optimize_styled_line(line: StyledLine) -> StyledLine {
    let runs = line.runs;
    let optimized_runs = optimize_styled_runs(runs);

    let mut optimized_line = StyledLine::new();
    for run in optimized_runs {
        optimized_line.push(run);
    }

    optimized_line
}

/// Optimize multiple StyledLines
pub fn optimize_styled_lines(lines: Vec<StyledLine>) -> Vec<StyledLine> {
    lines.into_iter().map(optimize_styled_line).collect()
}

/// Create a styled run with specific markdown styling
pub fn create_heading_run(text: &str, level: u32) -> StyledRun {
    let fg = match level {
        1 => Rgba::white(),            // White
        2 => rgba(200, 200, 200, 255), // Light gray
        3 => rgba(150, 150, 150, 255), // Medium gray
        _ => rgba(128, 128, 128, 255), // Default gray
    };

    StyledRun::new(text.to_string(), fg, Rgba::transparent(), Attr::BOLD)
}

/// Create a styled run for code blocks
pub fn create_code_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        rgba(200, 200, 200, 255), // Light text
        rgba(40, 40, 40, 255),    // Dark background
        Attr::empty(),
    )
}

/// Create a styled run for inline code  
pub fn create_inline_code_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        rgba(220, 220, 220, 255), // Light text
        rgba(60, 60, 60, 255),    // Slightly lighter dark background
        Attr::empty(),
    )
}

/// Create a styled run for links
pub fn create_link_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        rgba(100, 150, 255, 255), // Blue
        Rgba::transparent(),
        Attr::UNDERLINE,
    )
}

/// Create a styled run for quotes
pub fn create_quote_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        rgba(128, 128, 128, 255), // Gray
        Rgba::transparent(),
        Attr::ITALIC,
    )
}

/// Create a styled run for emphasis (italic)
pub fn create_emphasis_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        Rgba::white(), // White
        Rgba::transparent(),
        Attr::ITALIC,
    )
}

/// Create a styled run for strong (bold)
pub fn create_strong_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        Rgba::white(), // White
        Rgba::transparent(),
        Attr::BOLD,
    )
}

/// Create a styled run for strikethrough
pub fn create_strikethrough_run(text: &str) -> StyledRun {
    StyledRun::new(
        text.to_string(),
        Rgba::white(), // White
        Rgba::transparent(),
        Attr::STRIKE,
    )
}

/// Create a styled line from a vector of styled runs
pub fn create_styled_line(runs: Vec<StyledRun>) -> StyledLine {
    let optimized_runs = optimize_styled_runs(runs);
    let mut line = StyledLine::new();
    for run in optimized_runs {
        line.push(run);
    }
    line
}

/*
TODO: Implement remaining utility functions when needed:
- extract_plain_text
- extract_plain_text_lines
- count_visible_chars
- word_wrap_styled_runs
*/
