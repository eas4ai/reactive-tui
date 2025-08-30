//! Modern terminal capability detection system
//!
//! This module provides real-time probing of terminal features by sending
//! escape sequences and parsing responses. It detects graphics protocols,
//! mouse support levels, Unicode handling, and advanced terminal features
//! without relying on hardcoded terminal databases.

use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

/// Comprehensive terminal capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCapabilities {
    /// Basic terminal information
    pub terminal_info: TerminalInfo,
    /// Color and graphics capabilities
    pub graphics: GraphicsCapabilities,
    /// Input and interaction capabilities
    pub input: InputCapabilities,
    /// Text and Unicode handling
    pub text: TextCapabilities,
    /// Advanced protocol support
    pub protocols: ProtocolCapabilities,
    /// Performance characteristics
    pub performance: PerformanceCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    /// Terminal program name
    pub program: Option<String>,
    /// Terminal version
    pub version: Option<String>,
    /// Terminal type (TERM environment variable)
    pub term_type: String,
    /// Connection type (local, SSH, etc.)
    pub connection_type: ConnectionType,
    /// Terminal size in characters
    pub size: (u16, u16),
    /// Terminal size in pixels (if available)
    pub pixel_size: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsCapabilities {
    /// Color depth support
    pub color_depth: ColorDepth,
    /// True color (24-bit RGB) support
    pub truecolor: bool,
    /// Kitty graphics protocol support
    pub kitty_graphics: bool,
    /// Sixel graphics support
    pub sixel: bool,
    /// iTerm2 inline images support
    pub iterm2_images: bool,
    /// ReGIS graphics support
    pub regis: bool,
    /// Maximum image dimensions (if known)
    pub max_image_size: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCapabilities {
    /// Mouse support levels
    pub mouse: MouseCapabilities,
    /// Keyboard enhancement support
    pub keyboard: KeyboardCapabilities,
    /// Focus tracking support
    pub focus_tracking: bool,
    /// Bracketed paste support
    pub bracketed_paste: bool,
    /// Clipboard integration
    pub clipboard: ClipboardCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseCapabilities {
    /// Basic mouse support (clicks)
    pub basic: bool,
    /// Drag tracking support
    pub drag: bool,
    /// Motion tracking support
    pub motion: bool,
    /// Pixel-level mouse coordinates
    pub pixels: bool,
    /// SGR mouse mode support
    pub sgr_mode: bool,
    /// Mouse wheel support
    pub wheel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardCapabilities {
    /// Kitty keyboard protocol support
    pub kitty_keyboard: bool,
    /// Enhanced key reporting
    pub enhanced_keys: bool,
    /// Function key support level
    pub function_keys: u8,
    /// Modifier key combinations
    pub modifiers: ModifierSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierSupport {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub hyper: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardCapabilities {
    /// OSC 52 clipboard support
    pub osc52: bool,
    /// System clipboard integration
    pub system: bool,
    /// Primary selection support (X11)
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCapabilities {
    /// Unicode version supported
    pub unicode_version: f32,
    /// Unicode width calculation method
    pub width_method: UnicodeWidthMethod,
    /// Grapheme cluster support
    pub grapheme_clusters: bool,
    /// Emoji support level
    pub emoji_support: EmojiSupport,
    /// Font fallback capabilities
    pub font_fallback: bool,
    /// Combining character support
    pub combining_chars: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    /// Synchronized output support
    pub synchronized_output: bool,
    /// Hyperlink support (OSC 8)
    pub hyperlinks: bool,
    /// Notification support (OSC 9)
    pub notifications: bool,
    /// Title setting support
    pub title_setting: bool,
    /// Cursor shape control
    pub cursor_shapes: bool,
    /// Alternative screen buffer
    pub alt_screen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCapabilities {
    /// Measured input latency
    pub input_latency_ms: f32,
    /// Measured output latency
    pub output_latency_ms: f32,
    /// Maximum sustainable refresh rate
    pub max_refresh_rate: u32,
    /// Buffer size recommendations
    pub optimal_buffer_size: usize,
    /// Supports high-frequency updates
    pub high_frequency_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Local,
    SSH,
    Tmux,
    Screen,
    Web,
    Container,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorDepth {
    Monochrome,
    Color16,
    Color256,
    TrueColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnicodeWidthMethod {
    Wcwidth,
    Unicode,
    Wcswidth,
    Terminal, // Let terminal handle width calculation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmojiSupport {
    None,
    Basic,    // Simple emoji
    Extended, // Emoji with modifiers
    Full,     // Full emoji 15.0+ support
}

/// Capability detection errors
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("IO error during capability detection: {0}")]
    Io(#[from] io::Error),
    #[error("Timeout waiting for terminal response")]
    Timeout,
    #[error("Invalid terminal response: {0}")]
    InvalidResponse(String),
    #[error("Capability detection not supported in test mode")]
    TestMode,
}

pub type Result<T> = std::result::Result<T, CapabilityError>;

impl TerminalCapabilities {
    /// Detect all terminal capabilities with comprehensive probing
    pub fn detect() -> Result<Self> {
        #[cfg(test)]
        return Err(CapabilityError::TestMode);

        #[cfg(not(test))]
        {
            use crate::core::terminal_probe::TerminalProbe;
            let mut prober = TerminalProbe::new();
            prober.probe_all().map_err(CapabilityError::Io)
        }
    }

    /// Detect capabilities with custom timeout
    pub fn detect_with_timeout(timeout: Duration) -> Result<Self> {
        #[cfg(test)]
        return Err(CapabilityError::TestMode);

        #[cfg(not(test))]
        {
            use crate::core::terminal_probe::TerminalProbe;
            let mut prober = TerminalProbe::with_timeout(timeout);
            prober.probe_all().map_err(CapabilityError::Io)
        }
    }

    /// Get a capability score (0-100) indicating terminal modernity
    pub fn capability_score(&self) -> u8 {
        let mut score = 0u8;

        // Graphics capabilities (30 points)
        if self.graphics.truecolor {
            score += 10;
        }
        if self.graphics.kitty_graphics {
            score += 8;
        }
        if self.graphics.sixel {
            score += 6;
        }
        if self.graphics.iterm2_images {
            score += 6;
        }

        // Input capabilities (25 points)
        if self.input.mouse.pixels {
            score += 8;
        }
        if self.input.keyboard.kitty_keyboard {
            score += 7;
        }
        if self.input.bracketed_paste {
            score += 5;
        }
        if self.input.focus_tracking {
            score += 5;
        }

        // Protocol capabilities (25 points)
        if self.protocols.synchronized_output {
            score += 10;
        }
        if self.protocols.hyperlinks {
            score += 8;
        }
        if self.protocols.cursor_shapes {
            score += 4;
        }
        if self.protocols.alt_screen {
            score += 3;
        }

        // Text capabilities (20 points)
        if self.text.unicode_version >= 13.0 {
            score += 8;
        }
        if self.text.grapheme_clusters {
            score += 6;
        }
        if matches!(self.text.emoji_support, EmojiSupport::Full) {
            score += 6;
        }

        score.min(100)
    }

    /// Check if terminal is suitable for high-performance applications
    pub fn is_high_performance(&self) -> bool {
        self.graphics.truecolor
            && self.protocols.synchronized_output
            && self.performance.max_refresh_rate >= 60
            && self.performance.input_latency_ms < 50.0
    }

    /// Get recommended settings based on capabilities
    pub fn recommended_settings(&self) -> RecommendedSettings {
        RecommendedSettings {
            max_fps: self.performance.max_refresh_rate.min(144),
            buffer_size: self.performance.optimal_buffer_size,
            use_synchronized_output: self.protocols.synchronized_output,
            enable_mouse: self.input.mouse.basic,
            mouse_level: self.get_best_mouse_level(),
            color_mode: self.get_best_color_mode(),
            unicode_method: self.text.width_method.clone(),
        }
    }

    fn get_best_mouse_level(&self) -> MouseLevel {
        if self.input.mouse.pixels {
            MouseLevel::Pixels
        } else if self.input.mouse.motion {
            MouseLevel::Motion
        } else if self.input.mouse.drag {
            MouseLevel::Drag
        } else if self.input.mouse.basic {
            MouseLevel::Basic
        } else {
            MouseLevel::None
        }
    }

    fn get_best_color_mode(&self) -> ColorMode {
        if self.graphics.truecolor {
            ColorMode::TrueColor
        } else if matches!(self.graphics.color_depth, ColorDepth::Color256) {
            ColorMode::Color256
        } else {
            ColorMode::Color16
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecommendedSettings {
    pub max_fps: u32,
    pub buffer_size: usize,
    pub use_synchronized_output: bool,
    pub enable_mouse: bool,
    pub mouse_level: MouseLevel,
    pub color_mode: ColorMode,
    pub unicode_method: UnicodeWidthMethod,
}

#[derive(Debug, Clone)]
pub enum MouseLevel {
    None,
    Basic,
    Drag,
    Motion,
    Pixels,
}

#[derive(Debug, Clone)]
pub enum ColorMode {
    Color16,
    Color256,
    TrueColor,
}
