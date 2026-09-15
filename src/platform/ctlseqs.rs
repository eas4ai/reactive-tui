//! Terminal control sequences
//!
//! Comprehensive escape sequence library for terminal control

/// Device capability queries
pub mod queries {
    /// Primary device attributes query
    pub const PRIMARY_DEVICE_ATTRS: &str = "\x1b[c";

    /// Tertiary device attributes query  
    pub const TERTIARY_DEVICE_ATTRS: &str = "\x1b[=c";

    /// Device status report
    pub const DEVICE_STATUS_REPORT: &str = "\x1b[5n";

    /// XTerm version query
    pub const XTVERSION: &str = "\x1b[>0q";

    /// DECRQM queries for mode reporting
    pub const DECRQM_FOCUS: &str = "\x1b[?1004$p";
    /// Query SGR pixel mode support
    pub const DECRQM_SGR_PIXELS: &str = "\x1b[?1016$p";
    /// Query synchronized output support
    pub const DECRQM_SYNC: &str = "\x1b[?2026$p";
    /// Query Unicode support
    pub const DECRQM_UNICODE: &str = "\x1b[?2027$p";
    /// Query color scheme support
    pub const DECRQM_COLOR_SCHEME: &str = "\x1b[?2031$p";

    /// Kitty keyboard protocol query
    pub const CSI_U_QUERY: &str = "\x1b[?u";

    /// Kitty graphics capability query
    pub const KITTY_GRAPHICS_QUERY: &str = "\x1b_Gi=1,a=q\x1b\\";

    /// Sixel geometry query
    pub const SIXEL_GEOMETRY_QUERY: &str = "\x1b[?2;1;0S";
}

/// Mouse protocol control sequences
pub mod mouse {
    /// Enable mouse reporting (button motion, any motion, focus, SGR)
    pub const MOUSE_SET: &str = "\x1b[?1002;1003;1004;1006h";

    /// Enable mouse reporting with pixel coordinates
    pub const MOUSE_SET_PIXELS: &str = "\x1b[?1002;1003;1004;1016h";

    /// Disable mouse reporting
    pub const MOUSE_RESET: &str = "\x1b[?1002;1003;1004;1006;1016l";
}

/// Window and display control
pub mod window {
    /// Enable in-band window size reports
    pub const IN_BAND_RESIZE_SET: &str = "\x1b[?2048h";

    /// Disable in-band window size reports
    pub const IN_BAND_RESIZE_RESET: &str = "\x1b[?2048l";
}

/// Synchronized output control
pub mod sync {
    /// Enable synchronized output
    pub const SYNC_SET: &str = "\x1b[?2026h";

    /// Disable synchronized output
    pub const SYNC_RESET: &str = "\x1b[?2026l";
}

/// Unicode mode control
pub mod unicode {
    /// Enable Unicode mode
    pub const UNICODE_SET: &str = "\x1b[?2027h";

    /// Disable Unicode mode
    pub const UNICODE_RESET: &str = "\x1b[?2027l";
}

/// Bracketed paste mode
pub mod paste {
    /// Enable bracketed paste
    pub const BP_SET: &str = "\x1b[?2004h";

    /// Disable bracketed paste
    pub const BP_RESET: &str = "\x1b[?2004l";
}

/// Color scheme detection and updates
pub mod color_scheme {
    /// Request current color scheme
    pub const COLOR_SCHEME_REQUEST: &str = "\x1b[?996n";

    /// Enable color scheme update notifications
    pub const COLOR_SCHEME_SET: &str = "\x1b[?2031h";

    /// Disable color scheme update notifications
    pub const COLOR_SCHEME_RESET: &str = "\x1b[?2031l";
}

/// Enhanced keyboard protocol (Kitty)
pub mod keyboard {
    /// Push keyboard encoding mode
    pub fn csi_u_push(flags: u32) -> String {
        format!("\x1b[>{flags}u")
    }

    /// Pop keyboard encoding mode
    pub const CSI_U_POP: &str = "\x1b[<u";
}

/// Cursor control sequences
pub mod cursor {
    /// Move cursor to home position
    pub const HOME: &str = "\x1b[H";

    /// Move cursor to specific position
    pub fn cup(row: u16, col: u16) -> String {
        format!("\x1b[{row};{col}H")
    }

    /// Hide cursor
    pub const HIDE_CURSOR: &str = "\x1b[?25l";

    /// Show cursor
    pub const SHOW_CURSOR: &str = "\x1b[?25h";

    /// Set cursor shape
    pub fn cursor_shape(shape: u8) -> String {
        format!("\x1b[{shape} q")
    }

    /// Reverse index (move up)
    pub const RI: &str = "\x1bM";

    /// Index (move down)
    pub const IND: &str = "\n";

    /// Cursor forward
    pub fn cuf(n: u16) -> String {
        format!("\x1b[{n}C")
    }

    /// Cursor backward
    pub fn cub(n: u16) -> String {
        format!("\x1b[{n}D")
    }
}

/// Screen clearing and erasing
pub mod erase {
    /// Erase from cursor to end of screen
    pub const ERASE_BELOW_CURSOR: &str = "\x1b[J";

    /// Erase entire screen
    pub const ERASE_SCREEN: &str = "\x1b[2J";

    /// Erase from cursor to end of line
    pub const ERASE_TO_EOL: &str = "\x1b[K";
}

/// Alternate screen buffer
pub mod screen {
    /// Switch to alternate screen
    pub const SMCUP: &str = "\x1b[?1049h";

    /// Switch to main screen
    pub const RMCUP: &str = "\x1b[?1049l";
}

/// SGR (Select Graphic Rendition) sequences
pub mod sgr {
    /// Reset all attributes
    pub const SGR_RESET: &str = "\x1b[m";

    /// Basic colors (30-37, 90-97)
    pub fn fg_base(color: u8) -> String {
        format!("\x1b[3{color}m")
    }

    /// Set bright foreground color (90-97)
    pub fn fg_bright(color: u8) -> String {
        format!("\x1b[9{color}m")
    }

    /// Set basic background color (40-47)
    pub fn bg_base(color: u8) -> String {
        format!("\x1b[4{color}m")
    }

    /// Set bright background color (90-97)
    pub fn bg_bright(color: u8) -> String {
        format!("\x1b[10{color}m")
    }

    /// Reset colors
    pub const FG_RESET: &str = "\x1b[39m";
    /// Reset background color
    pub const BG_RESET: &str = "\x1b[49m";
    /// Reset underline color
    pub const UL_RESET: &str = "\x1b[59m";

    /// Indexed colors (256-color palette)
    pub fn fg_indexed(index: u8) -> String {
        format!("\x1b[38:5:{index}m")
    }

    /// Set background color using 256-color index
    pub fn bg_indexed(index: u8) -> String {
        format!("\x1b[48:5:{index}m")
    }

    /// Set underline color using 256-color index
    pub fn ul_indexed(index: u8) -> String {
        format!("\x1b[58:5:{index}m")
    }

    /// RGB colors (true color)
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38:2:{r}:{g}:{b}m")
    }

    /// Set background color using RGB values
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48:2:{r}:{g}:{b}m")
    }

    /// Set underline color using RGB values
    pub fn ul_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[58:2:{r}:{g}:{b}m")
    }
}

/// Underline styles
pub mod underline {
    /// Turn off underline
    pub const UL_OFF: &str = "\x1b[24m";

    /// Single underline
    pub const UL_SINGLE: &str = "\x1b[4m";

    /// Double underline
    pub const UL_DOUBLE: &str = "\x1b[4:2m";

    /// Curly underline
    pub const UL_CURLY: &str = "\x1b[4:3m";

    /// Dotted underline
    pub const UL_DOTTED: &str = "\x1b[4:4m";

    /// Dashed underline
    pub const UL_DASHED: &str = "\x1b[4:5m";
}

/// Text attributes
pub mod attributes {
    /// Bold/bright
    pub const BOLD_SET: &str = "\x1b[1m";
    /// Reset bold/bright
    pub const BOLD_RESET: &str = "\x1b[22m";

    /// Dim
    pub const DIM_SET: &str = "\x1b[2m";

    /// Italic
    pub const ITALIC_SET: &str = "\x1b[3m";
    /// Reset italic
    pub const ITALIC_RESET: &str = "\x1b[23m";

    /// Blink
    pub const BLINK_SET: &str = "\x1b[5m";
    /// Reset blink
    pub const BLINK_RESET: &str = "\x1b[25m";

    /// Reverse video
    pub const REVERSE_SET: &str = "\x1b[7m";
    /// Reset reverse video
    pub const REVERSE_RESET: &str = "\x1b[27m";

    /// Invisible/hidden
    pub const INVISIBLE_SET: &str = "\x1b[8m";
    /// Reset invisible/hidden
    pub const INVISIBLE_RESET: &str = "\x1b[28m";

    /// Strikethrough
    pub const STRIKETHROUGH_SET: &str = "\x1b[9m";
    /// Reset strikethrough
    pub const STRIKETHROUGH_RESET: &str = "\x1b[29m";
}

/// Hyperlink support (OSC 8)
pub mod hyperlink {
    /// Create hyperlink
    pub fn create(url: &str, text: &str, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("\x1b]8;id={id};{url}\x07{text}\x1b]8;;\x07"),
            None => format!("\x1b]8;;{url}\x07{text}\x1b]8;;\x07"),
        }
    }

    /// End hyperlink
    pub const END: &str = "\x1b]8;;\x07";
}
