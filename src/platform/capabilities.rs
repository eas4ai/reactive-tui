//! Terminal capability detection for the platform layer
//!
//! Re-exports and integrates with the existing core capabilities module

pub use crate::core::capabilities::{
    ColorDepth, TerminalCapabilities as CoreCapabilities, TerminalQuery,
};

use super::TerminalCapabilities;

/// Convert from core capabilities to platform capabilities
impl From<CoreCapabilities> for TerminalCapabilities {
    fn from(core: CoreCapabilities) -> Self {
        Self {
            true_color: core.color_depth == ColorDepth::TrueColor,
            kitty_graphics: core.kitty_graphics,
            sixel_graphics: core.sixel,
            iterm2_images: core.iterm2_graphics,
            hyperlinks: true, // Assume OSC 8 support for modern terminals
            pixel_mouse: core.pixel_mouse,
            synchronized_output: core.synchronized_output,
            enhanced_keyboard: core.enhanced_keyboard,
            bracketed_paste: true, // Most terminals support this
            focus_events: true,    // Most terminals support this
            terminal_name: None,
        }
    }
}

/// Convert from platform capabilities to core capabilities
impl From<TerminalCapabilities> for CoreCapabilities {
    fn from(platform: TerminalCapabilities) -> Self {
        Self {
            color_depth: if platform.true_color {
                ColorDepth::TrueColor
            } else {
                ColorDepth::Colors256
            },
            unicode: true, // Assume Unicode support
            sixel: platform.sixel_graphics,
            kitty_graphics: platform.kitty_graphics,
            iterm2_graphics: platform.iterm2_images,
            enhanced_keyboard: platform.enhanced_keyboard,
            pixel_mouse: platform.pixel_mouse,
            synchronized_output: platform.synchronized_output,
        }
    }
}

/// Detect terminal capabilities using the existing core detection
pub fn detect_capabilities() -> crate::error::Result<TerminalCapabilities> {
    // Use environment-based detection as fallback
    let core_caps = TerminalQuery::detect_from_env();
    Ok(core_caps.into())
}

/// Detect capabilities with direct terminal queries
pub fn detect_capabilities_with_queries() -> crate::error::Result<TerminalCapabilities> {
    use std::io::{stdin, stdout};

    let query = TerminalQuery::new();

    // Send queries to terminal
    query.send_queries(&mut stdout())?;

    // Try to read responses
    let core_caps = query.parse_responses(&mut stdin())?;

    Ok(core_caps.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_conversion() {
        let core = CoreCapabilities {
            color_depth: ColorDepth::TrueColor,
            unicode: true,
            sixel: true,
            kitty_graphics: false,
            iterm2_graphics: false,
            enhanced_keyboard: true,
            pixel_mouse: false,
            synchronized_output: true,
        };

        let platform: TerminalCapabilities = core.into();
        assert!(platform.true_color);
        assert!(platform.sixel_graphics);
        assert!(platform.enhanced_keyboard);
        assert!(platform.synchronized_output);
        assert!(!platform.kitty_graphics);
        assert!(!platform.pixel_mouse);

        let back_to_core: CoreCapabilities = platform.into();
        assert_eq!(back_to_core.color_depth, ColorDepth::TrueColor);
        assert!(back_to_core.sixel);
        assert!(back_to_core.enhanced_keyboard);
        assert!(back_to_core.synchronized_output);
    }
}
