//! Platform-specific terminal handling
//!
//! This module provides direct TTY access for advanced terminal features
//! that crossterm cannot provide, such as:
//! - Image rendering (Kitty graphics, Sixel)
//! - Hyperlink support (OSC 8)
//! - Pixel mouse coordinates
//! - Synchronized output
//! - Enhanced keyboard protocols

#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
mod input_receiver;
#[cfg(unix)]
pub use input_receiver::{InputIntoIter, InputIter, InputReceiver};

#[cfg(windows)]
pub mod windows;

pub mod capabilities;
pub mod ctlseqs;
pub mod image;
pub mod r#loop;
pub mod parser;
pub mod unicode;

#[cfg(test)]
mod tests;

use crate::error::Result;

/// General image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    /// Portable Network Graphics format
    Png,
    /// JPEG image format
    Jpeg,
    /// Graphics Interchange Format
    Gif,
    /// WebP image format
    WebP,
    /// Sixel graphics format for terminals
    Sixel,
    /// Raw pixel data
    Raw,
}

/// Kitty graphics image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KittyImageFormat {
    /// Raw RGB data (24-bit)
    Rgb,
    /// Raw RGBA data (32-bit)
    Rgba,
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
}

/// Enhanced keyboard protocol features
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnhancedKeyboardFeatures {
    /// Disambiguate escape codes
    pub disambiguate_escape_codes: bool,
    /// Report all keys as escape codes
    pub report_all_keys: bool,
    /// Report alternate keys
    pub report_alternate_keys: bool,
    /// Report all key events (press, repeat, release)
    pub report_all_events: bool,
    /// Report text as codepoints
    pub report_text_as_codepoints: bool,
}

impl Default for EnhancedKeyboardFeatures {
    fn default() -> Self {
        Self {
            disambiguate_escape_codes: true,
            report_all_keys: true,
            report_alternate_keys: false,
            report_all_events: false,
            report_text_as_codepoints: false,
        }
    }
}

/// Platform-agnostic TTY interface
pub trait PlatformTty: Send + Sync {
    /// Write raw bytes to the terminal
    fn write(&self, data: &[u8]) -> Result<usize>;

    /// Get terminal size in characters
    fn size(&self) -> Result<(u16, u16)>;

    /// Restore original terminal settings
    fn restore(&self) -> Result<()>;
}

/// Terminal capabilities detected through direct queries
#[derive(Debug, Clone, Default)]
pub struct TerminalCapabilities {
    /// Supports true color (24-bit RGB)
    pub true_color: bool,

    /// Supports Kitty graphics protocol
    pub kitty_graphics: bool,

    /// Supports Sixel graphics
    pub sixel_graphics: bool,

    /// Supports iTerm2 inline images
    pub iterm2_images: bool,

    /// Supports hyperlinks (OSC 8)
    pub hyperlinks: bool,

    /// Supports pixel mouse coordinates
    pub pixel_mouse: bool,

    /// Supports synchronized output (flicker-free updates)
    pub synchronized_output: bool,

    /// Supports enhanced keyboard protocol (Kitty)
    pub enhanced_keyboard: bool,

    /// Supports bracketed paste mode
    pub bracketed_paste: bool,

    /// Supports focus in/out events
    pub focus_events: bool,

    /// Terminal name/version if detected
    pub terminal_name: Option<String>,
}

/// Events that can be parsed from terminal input
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalEvent {
    /// Key press with enhanced information
    Key {
        /// The key that was pressed
        code: KeyCode,
        /// Modifier keys held during the press (Ctrl, Alt, Shift, etc.)
        modifiers: KeyModifiers,
        /// Type of key event (press, release, repeat)
        kind: KeyEventKind,
    },

    /// Mouse event with pixel coordinates if available
    Mouse {
        /// Type of mouse event (click, move, scroll, etc.)
        kind: MouseEventKind,
        /// Physical button carried by button and drag events.
        button: Option<MouseButton>,
        /// Column position in terminal cells
        column: u16,
        /// Row position in terminal cells
        row: u16,
        /// Pixel-level X coordinate (if supported by terminal)
        pixel_x: Option<u16>,
        /// Pixel-level Y coordinate (if supported by terminal)
        pixel_y: Option<u16>,
        /// Modifier keys held during the mouse event
        modifiers: KeyModifiers,
    },

    /// Terminal resize
    Resize {
        /// New terminal width in columns
        width: u16,
        /// New terminal height in rows
        height: u16,
    },

    /// Focus gained
    FocusGained,

    /// Focus lost
    FocusLost,

    /// Paste start (bracketed paste)
    PasteStart,

    /// Paste end (bracketed paste)
    PasteEnd,

    /// Paste content
    Paste(String),

    /// Terminal capability response
    CapabilityResponse(String),

    /// Color scheme change (dark/light)
    ColorScheme(ColorScheme),

    /// Error event for async event loops
    Error(String),
}

/// Key codes for keyboard input
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCode {
    /// Backspace key
    Backspace,
    /// Enter/Return key
    Enter,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// Tab key
    Tab,
    /// Shift+Tab (reverse tab)
    BackTab,
    /// Delete key
    Delete,
    /// Insert key
    Insert,
    /// Escape key
    Escape,
    /// Character key
    Char(char),
    /// Function key (F1-F12)
    F(u8),

    // Media keys
    /// Media play key
    MediaPlay,
    /// Media pause key
    MediaPause,
    /// Media play/pause toggle key
    MediaPlayPause,
    /// Media reverse key
    MediaReverse,
    /// Media stop key
    MediaStop,
    /// Media fast forward key
    MediaFastForward,
    /// Media rewind key
    MediaRewind,
    /// Next track key
    MediaTrackNext,
    /// Previous track key
    MediaTrackPrevious,
    /// Media record key
    MediaRecord,
    /// Lower volume key
    LowerVolume,
    /// Raise volume key
    RaiseVolume,
    /// Mute volume key
    MuteVolume,

    // Modifier keys
    /// Left Shift key
    LeftShift,
    /// Left Ctrl key
    LeftCtrl,
    /// Left Alt key
    LeftAlt,
    /// Left Super/Windows/Command key
    LeftSuper,
    /// Right Shift key
    RightShift,
    /// Right Ctrl key
    RightCtrl,
    /// Right Alt key
    RightAlt,
    /// Right Super/Windows/Command key
    RightSuper,
    /// ISO Level 3 Shift key
    IsoLevel3Shift,
    /// ISO Level 5 Shift key
    IsoLevel5Shift,

    /// Unknown or unmapped key
    Unknown,
}

/// Keyboard modifier keys state
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct KeyModifiers {
    /// Whether Shift key is pressed
    pub shift: bool,
    /// Whether Ctrl key is pressed
    pub ctrl: bool,
    /// Whether Alt key is pressed
    pub alt: bool,
    /// Whether Meta key is pressed (Super/Windows/Command)
    pub meta: bool,
}

impl KeyModifiers {
    /// Create empty key modifiers (no modifiers pressed)
    pub fn empty() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
}

/// Type of key event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyEventKind {
    /// Key was pressed down
    Press,
    /// Key was released
    Release,
    /// Key is being held down (repeat event)
    Repeat,
}

/// Type of mouse event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventKind {
    /// Mouse button pressed down
    Down,
    /// Mouse button released
    Up,
    /// Mouse dragged while button is held
    Drag,
    /// Mouse moved without button pressed
    Move,
    /// Mouse wheel scrolled up
    ScrollUp,
    /// Mouse wheel scrolled down
    ScrollDown,
    /// Mouse wheel scrolled left
    ScrollLeft,
    /// Mouse wheel scrolled right
    ScrollRight,
}

/// Physical mouse button reported by native terminal input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Primary or left button.
    Left,
    /// Middle button, commonly the scroll-wheel press.
    Middle,
    /// Secondary or right button.
    Right,
}

/// Color scheme preference for the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorScheme {
    /// Dark color scheme
    Dark,
    /// Light color scheme
    Light,
}

/// Control sequences for terminal features
pub mod sequences {
    // Device capability queries
    /// Query primary device attributes
    pub const PRIMARY_DEVICE_ATTRS: &[u8] = b"\x1b[c";
    /// Query secondary device attributes
    pub const SECONDARY_DEVICE_ATTRS: &[u8] = b"\x1b[>c";
    /// Query tertiary device attributes
    pub const TERTIARY_DEVICE_ATTRS: &[u8] = b"\x1b[=c";

    // Feature queries
    /// Query Kitty graphics protocol support
    pub const KITTY_GRAPHICS_QUERY: &[u8] = b"\x1b_Gi=1,a=q\x1b\\";
    /// Query Sixel graphics support
    pub const SIXEL_QUERY: &[u8] = b"\x1b[?2;1;0S";
    /// Query synchronized output support
    pub const SYNC_QUERY: &[u8] = b"\x1b[?2026$p";
    /// Query Unicode support
    pub const UNICODE_QUERY: &[u8] = b"\x1b[?2027$p";

    /// The capability queries sent at startup, ten milliseconds apart. The
    /// mode 2027 (unicode) answer is the glyph capability report the charts
    /// follow (CHT-028).
    pub const STARTUP_QUERIES: &[&[u8]] = &[
        PRIMARY_DEVICE_ATTRS,
        SECONDARY_DEVICE_ATTRS,
        KITTY_GRAPHICS_QUERY,
        SYNC_QUERY,
        UNICODE_QUERY,
        PIXEL_MOUSE_QUERY,
        ENHANCED_KEYBOARD_QUERY,
    ];
    /// Query pixel-level mouse support
    pub const PIXEL_MOUSE_QUERY: &[u8] = b"\x1b[?1016$p";
    /// Query enhanced keyboard protocol support
    pub const ENHANCED_KEYBOARD_QUERY: &[u8] = b"\x1b[?u";

    // Feature enable/disable
    /// Enable mouse tracking
    pub const ENABLE_MOUSE: &[u8] = b"\x1b[?1002;1003;1004;1006h";
    /// Enable pixel-level mouse tracking
    pub const ENABLE_PIXEL_MOUSE: &[u8] = b"\x1b[?1002;1003;1004;1016h";
    /// Disable mouse tracking
    pub const DISABLE_MOUSE: &[u8] = b"\x1b[?1002;1003;1004;1006;1016l";

    /// Enable bracketed paste mode
    pub const ENABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004h";
    /// Disable bracketed paste mode
    pub const DISABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004l";

    /// Enable focus events
    pub const ENABLE_FOCUS_EVENTS: &[u8] = b"\x1b[?1004h";
    /// Disable focus events
    pub const DISABLE_FOCUS_EVENTS: &[u8] = b"\x1b[?1004l";

    /// Enable synchronized output for flicker-free updates
    pub const ENABLE_SYNC_OUTPUT: &[u8] = b"\x1b[?2026h";
    /// Disable synchronized output
    pub const DISABLE_SYNC_OUTPUT: &[u8] = b"\x1b[?2026l";

    /// Enable alternate screen buffer
    pub const ENABLE_ALT_SCREEN: &[u8] = b"\x1b[?1049h";
    /// Disable alternate screen buffer
    pub const DISABLE_ALT_SCREEN: &[u8] = b"\x1b[?1049l";

    // Cursor control
    /// Hide the cursor
    pub const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
    /// Show the cursor
    pub const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
    /// Save cursor position
    pub const SAVE_CURSOR: &[u8] = b"\x1b7";
    /// Restore cursor position
    pub const RESTORE_CURSOR: &[u8] = b"\x1b8";

    // Clear operations
    /// Clear entire screen
    pub const CLEAR_SCREEN: &[u8] = b"\x1b[2J";
    /// Clear current line
    pub const CLEAR_LINE: &[u8] = b"\x1b[2K";
    /// Clear from cursor to end of screen
    pub const CLEAR_TO_END: &[u8] = b"\x1b[J";

    // Hyperlink support (OSC 8)
    /// Generate hyperlink start sequence
    ///
    /// # Arguments
    /// * `uri` - The URI to link to
    ///
    /// # Returns
    /// Terminal escape sequence to start a hyperlink
    pub fn hyperlink_start(uri: &str) -> String {
        format!("\x1b]8;;{}\x1b\\", uri)
    }

    /// End hyperlink sequence
    pub const HYPERLINK_END: &[u8] = b"\x1b]8;;\x1b\\";

    /// Generate Kitty graphics protocol image transmission sequence
    ///
    /// # Arguments
    /// * `id` - Unique image identifier
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `data` - Raw image data
    ///
    /// # Returns
    /// Complete terminal escape sequence for image transmission
    pub fn kitty_image_transmit(id: u32, width: u32, height: u32, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(
            format!("\x1b_Gf=32,i={},s={},v={},m=1;", id, width, height).as_bytes(),
        );
        result.extend_from_slice(data);
        result.extend_from_slice(b"\x1b\\");
        result
    }
}

// Platform-specific implementation selection
/// Default TTY implementation for Unix platforms
#[cfg(unix)]
pub type DefaultTty = unix::UnixTty;

/// Default TTY implementation for Windows platforms
#[cfg(windows)]
pub type DefaultTty = windows::WindowsTty;

/// High-level TTY wrapper with capability detection
pub struct DirectTty {
    inner: DefaultTty,
    capabilities: TerminalCapabilities,
    parser: parser::EscapeSequenceParser,
}

impl DirectTty {
    /// Initialize direct TTY with capability detection
    pub fn init() -> Result<Self> {
        let inner = DefaultTty::init()?;
        let mut tty = Self {
            inner,
            capabilities: TerminalCapabilities::default(),
            parser: parser::EscapeSequenceParser::new(),
        };

        // Detect terminal capabilities
        tty.detect_capabilities()?;

        Ok(tty)
    }

    /// Poll for terminal events with timeout
    pub fn poll_events(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<TerminalEvent>> {
        #[cfg(unix)]
        {
            let mut buffer = [0u8; 1024];
            match self.inner.read(&mut buffer, timeout) {
                Ok(n) if n > 0 => {
                    let events = self.parser.parse(&buffer[..n]);
                    Ok(events)
                }
                Ok(_) => Ok(vec![]), // No data
                Err(e) => Err(e),
            }
        }

        #[cfg(windows)]
        {
            self.inner.read_input_events(timeout)
        }
    }

    /// Start a bounded async event stream (Unix only).
    /// Dropping the receiver or this terminal joins its input worker.
    #[cfg(unix)]
    pub fn start_async_events(&self) -> Result<InputReceiver<TerminalEvent>> {
        let mut parser = parser::EscapeSequenceParser::new();
        self.inner.spawn_input(move |data| parser.parse(data))
    }

    /// Write raw bytes to terminal
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        self.inner.write(data)
    }

    /// Restore the native terminal mode before releasing this session.
    pub fn restore(&self) -> Result<()> {
        self.inner.restore()
    }

    /// Write a string to terminal
    pub fn write_str(&self, s: &str) -> Result<usize> {
        self.write(s.as_bytes())
    }

    /// Get terminal size
    pub fn size(&self) -> Result<(u16, u16)> {
        self.inner.size()
    }

    /// Get detected capabilities
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Enable mouse reporting
    pub fn enable_mouse(&self, pixel_mode: bool) -> Result<()> {
        if pixel_mode && self.capabilities.pixel_mouse {
            self.write(sequences::ENABLE_PIXEL_MOUSE)?;
        } else {
            self.write(sequences::ENABLE_MOUSE)?;
        }
        Ok(())
    }

    /// Enable basic mouse reporting
    pub fn enable_basic_mouse(&self) -> Result<()> {
        self.write(sequences::ENABLE_MOUSE)?;
        Ok(())
    }

    /// Enable pixel mouse reporting
    pub fn enable_pixel_mouse(&self) -> Result<()> {
        if self.capabilities.pixel_mouse {
            self.write(sequences::ENABLE_PIXEL_MOUSE)?;
        } else {
            self.enable_basic_mouse()?;
        }
        Ok(())
    }

    /// Disable mouse reporting
    pub fn disable_mouse(&self) -> Result<()> {
        self.write(sequences::DISABLE_MOUSE)?;
        Ok(())
    }

    /// Enable focus events
    pub fn enable_focus_events(&self) -> Result<()> {
        if self.capabilities.focus_events {
            self.write(b"\x1b[?1004h")?;
        }
        Ok(())
    }

    /// Disable focus events
    pub fn disable_focus_events(&self) -> Result<()> {
        if self.capabilities.focus_events {
            self.write(b"\x1b[?1004l")?;
        }
        Ok(())
    }

    /// Enable bracketed paste mode
    pub fn enable_bracketed_paste(&self) -> Result<()> {
        if self.capabilities.bracketed_paste {
            self.write(b"\x1b[?2004h")?;
        }
        Ok(())
    }

    /// Disable bracketed paste mode
    pub fn disable_bracketed_paste(&self) -> Result<()> {
        if self.capabilities.bracketed_paste {
            self.write(b"\x1b[?2004l")?;
        }
        Ok(())
    }

    /// Enable synchronized output for flicker-free updates
    pub fn enable_sync_output(&self) -> Result<()> {
        if self.capabilities.synchronized_output {
            self.write(sequences::ENABLE_SYNC_OUTPUT)?;
        } else {
            // Fallback: use cursor save/restore for basic synchronization
            self.write(b"\x1b7")?; // Save cursor position
        }
        Ok(())
    }

    /// Disable synchronized output
    pub fn disable_sync_output(&self) -> Result<()> {
        if self.capabilities.synchronized_output {
            self.write(sequences::DISABLE_SYNC_OUTPUT)?;
        } else {
            // Fallback: restore cursor position
            self.write(b"\x1b8")?; // Restore cursor position
        }
        Ok(())
    }

    /// Perform synchronized update with callback
    pub fn synchronized_update<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Self) -> Result<R>,
    {
        self.enable_sync_output()?;
        let result = f(self);
        self.disable_sync_output()?;
        result
    }

    /// Test synchronized output support by sending a query
    pub fn test_sync_output_support(&self) -> Result<bool> {
        // Send sync output query
        self.write(b"\x1b[?2026$p")?;

        // Try to read response with short timeout
        let mut buffer = [0u8; 64];
        match self
            .inner
            .read(&mut buffer, Some(std::time::Duration::from_millis(50)))
        {
            Ok(n) if n > 0 => {
                let response = String::from_utf8_lossy(&buffer[..n]);
                // Look for sync output response
                Ok(response.contains("2026;1$y") || response.contains("2026;2$y"))
            }
            _ => Ok(false),
        }
    }

    /// Force enable synchronized output (even if not detected)
    pub fn force_enable_sync_output(&mut self) -> Result<()> {
        self.capabilities.synchronized_output = true;
        self.enable_sync_output()
    }

    /// Batch multiple operations with synchronization
    pub fn batch_operations<F>(&self, operations: F) -> Result<()>
    where
        F: FnOnce(&Self) -> Result<()>,
    {
        self.synchronized_update(|tty| {
            operations(tty)?;
            Ok(())
        })
    }

    /// Enable enhanced keyboard protocol
    pub fn enable_enhanced_keyboard(&self) -> Result<()> {
        if self.capabilities.enhanced_keyboard {
            // Enable progressive enhancement levels
            self.write(b"\x1b[>1u")?; // Level 1: disambiguate escape codes
            self.write(b"\x1b[>2u")?; // Level 2: report all keys as escape codes
            self.write(b"\x1b[>4u")?; // Level 4: report alternate keys
            self.write(b"\x1b[>8u")?; // Level 8: report all key events
        }
        Ok(())
    }

    /// Disable enhanced keyboard protocol
    pub fn disable_enhanced_keyboard(&self) -> Result<()> {
        if self.capabilities.enhanced_keyboard {
            self.write(b"\x1b[<u")?; // Disable all levels
        }
        Ok(())
    }

    /// Query enhanced keyboard support level
    pub fn query_enhanced_keyboard_support(&self) -> Result<()> {
        self.write(b"\x1b[?u")?; // Query current level
        Ok(())
    }

    /// Enable specific enhanced keyboard features
    pub fn enable_enhanced_keyboard_features(
        &self,
        features: EnhancedKeyboardFeatures,
    ) -> Result<()> {
        if !self.capabilities.enhanced_keyboard {
            return Ok(());
        }

        let mut level = 0u32;

        if features.disambiguate_escape_codes {
            level |= 1;
        }
        if features.report_all_keys {
            level |= 2;
        }
        if features.report_alternate_keys {
            level |= 4;
        }
        if features.report_all_events {
            level |= 8;
        }
        if features.report_text_as_codepoints {
            level |= 16;
        }

        if level > 0 {
            self.write_str(&format!("\x1b[>{}u", level))?;
        }

        Ok(())
    }

    /// Write a hyperlink
    pub fn write_hyperlink(&self, uri: &str, text: &str) -> Result<()> {
        if self.capabilities.hyperlinks {
            self.write_str(&sequences::hyperlink_start(uri))?;
            self.write_str(text)?;
            self.write(sequences::HYPERLINK_END)?;
        } else {
            // Fallback to plain text
            self.write_str(text)?;
        }
        Ok(())
    }

    /// Transmit an image using Kitty graphics protocol
    pub fn transmit_kitty_image(
        &self,
        id: u32,
        width: u32,
        height: u32,
        data: &[u8],
        format: KittyImageFormat,
    ) -> Result<()> {
        if !self.capabilities.kitty_graphics {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Kitty graphics not supported",
            )
            .into());
        }

        // Encode image data as base64
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);

        // Split into chunks (Kitty has a limit on chunk size)
        const CHUNK_SIZE: usize = 4096;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();

        let format_code = match format {
            KittyImageFormat::Rgb => 24,
            KittyImageFormat::Rgba => 32,
            KittyImageFormat::Png => 100,
            KittyImageFormat::Jpeg => 100,
        };

        for (i, chunk) in chunks.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == chunks.len() - 1;

            if is_first {
                // First chunk with metadata
                self.write_str(&format!(
                    "\x1b_Ga=T,f={},i={},s={},v={},m={};{}",
                    format_code,
                    id,
                    width,
                    height,
                    if is_last { 0 } else { 1 }, // m=0 for last chunk, m=1 for more
                    chunk
                ))?;
            } else {
                // Continuation chunk
                self.write_str(&format!(
                    "\x1b_Gm={};{}",
                    if is_last { 0 } else { 1 },
                    chunk
                ))?;
            }

            self.write(b"\x1b\\")?; // End sequence
        }

        Ok(())
    }

    /// Transmit an image file using Kitty graphics protocol
    pub fn transmit_kitty_image_file(&self, id: u32, file_path: &str) -> Result<()> {
        if !self.capabilities.kitty_graphics {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Kitty graphics not supported",
            )
            .into());
        }

        // Use direct file transmission (more efficient)
        use base64::Engine;
        let encoded_path = base64::engine::general_purpose::STANDARD.encode(file_path.as_bytes());

        self.write_str(&format!("\x1b_Ga=T,t=f,i={};{}\x1b\\", id, encoded_path))?;

        Ok(())
    }

    /// Place a Kitty image at specific coordinates
    pub fn place_kitty_image(&self, id: u32, x: u16, y: u16, cols: u16, rows: u16) -> Result<()> {
        if !self.capabilities.kitty_graphics {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Kitty graphics not supported",
            )
            .into());
        }

        self.write_str(&format!(
            "\x1b_Ga=p,i={},C={},R={},c={},r={}\x1b\\",
            id, x, y, cols, rows
        ))?;

        Ok(())
    }

    /// Delete a Kitty image
    pub fn delete_kitty_image(&self, id: u32) -> Result<()> {
        if !self.capabilities.kitty_graphics {
            return Ok(()); // Silently ignore if not supported
        }

        self.write_str(&format!("\x1b_Ga=d,i={}\x1b\\", id))?;
        Ok(())
    }

    /// Transmit Sixel image data
    pub fn transmit_sixel(&self, data: &[u8]) -> Result<()> {
        if !self.capabilities.sixel_graphics {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Sixel graphics not supported",
            )
            .into());
        }

        // Start sixel sequence
        self.write(b"\x1bPq")?;

        // Write sixel data
        self.write(data)?;

        // End sixel sequence
        self.write(b"\x1b\\")?;

        Ok(())
    }

    /// Auto-detect image format and transmit using best available protocol
    pub fn transmit_image_auto(
        &self,
        data: &[u8],
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> Result<()> {
        let format = self.detect_image_format(data)?;

        match format {
            ImageFormat::Png | ImageFormat::Jpeg => {
                if self.capabilities.kitty_graphics {
                    // Use Kitty graphics for PNG/JPEG
                    let (width, height) = self.get_image_dimensions(data, &format)?;
                    let (display_width, display_height) =
                        self.calculate_display_size(width, height, max_width, max_height);

                    let kitty_format = match format {
                        ImageFormat::Png => KittyImageFormat::Png,
                        ImageFormat::Jpeg => KittyImageFormat::Jpeg,
                        _ => KittyImageFormat::Png,
                    };

                    let id = self.generate_image_id();
                    self.transmit_kitty_image(
                        id,
                        display_width,
                        display_height,
                        data,
                        kitty_format,
                    )?;
                    self.place_kitty_image(
                        id,
                        0,
                        0,
                        (display_width / 8) as u16,
                        (display_height / 16) as u16,
                    )?;
                } else if self.capabilities.sixel_graphics {
                    // Convert to Sixel
                    let sixel_data = self.convert_to_sixel(data, &format, max_width, max_height)?;
                    self.transmit_sixel(&sixel_data)?;
                } else if self.capabilities.iterm2_images {
                    // Use iTerm2 protocol
                    self.transmit_iterm2_image(data, max_width, max_height)?;
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "No image protocols supported",
                    )
                    .into());
                }
            }
            ImageFormat::Sixel => {
                if self.capabilities.sixel_graphics {
                    self.transmit_sixel(data)?;
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Sixel graphics not supported",
                    )
                    .into());
                }
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported image format",
                )
                .into());
            }
        }

        Ok(())
    }

    /// Detect image format from data
    fn detect_image_format(&self, data: &[u8]) -> Result<ImageFormat> {
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Data too short to determine format",
            )
            .into());
        }

        // PNG signature
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok(ImageFormat::Png);
        }

        // JPEG signature
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(ImageFormat::Jpeg);
        }

        // Sixel signature
        if data.starts_with(b"\x1bPq") || data.starts_with(b"\"") {
            return Ok(ImageFormat::Sixel);
        }

        // GIF signature
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Ok(ImageFormat::Gif);
        }

        // WebP signature
        if data.len() >= 12
            && data[0..4] == [0x52, 0x49, 0x46, 0x46]
            && data[8..12] == [0x57, 0x45, 0x42, 0x50]
        {
            return Ok(ImageFormat::WebP);
        }

        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown image format").into())
    }

    /// Get image dimensions from data
    fn get_image_dimensions(&self, data: &[u8], format: &ImageFormat) -> Result<(u32, u32)> {
        match format {
            ImageFormat::Png => self.get_png_dimensions(data),
            ImageFormat::Jpeg => self.get_jpeg_dimensions(data),
            _ => Ok((100, 100)), // Default size
        }
    }

    /// Get PNG dimensions
    fn get_png_dimensions(&self, data: &[u8]) -> Result<(u32, u32)> {
        if data.len() < 24 {
            return Ok((100, 100));
        }

        // PNG IHDR chunk starts at byte 16
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

        Ok((width, height))
    }

    /// Get JPEG dimensions (simplified)
    fn get_jpeg_dimensions(&self, data: &[u8]) -> Result<(u32, u32)> {
        // This is a simplified JPEG parser - in production you'd want a proper library
        let mut i = 2; // Skip SOI marker

        while i < data.len() - 8 {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                match marker {
                    0xC0..=0xC3 => {
                        // SOF marker
                        if i + 7 < data.len() {
                            let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                            let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                            return Ok((width, height));
                        }
                    }
                    _ => {
                        // Skip this segment
                        if i + 3 < data.len() {
                            let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                            i += length + 2;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        Ok((100, 100)) // Default if parsing fails
    }

    /// Calculate display size respecting max dimensions
    fn calculate_display_size(
        &self,
        width: u32,
        height: u32,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> (u32, u32) {
        let mut display_width = width;
        let mut display_height = height;

        if let Some(max_w) = max_width {
            if display_width > max_w {
                let ratio = max_w as f32 / display_width as f32;
                display_width = max_w;
                display_height = (display_height as f32 * ratio) as u32;
            }
        }

        if let Some(max_h) = max_height {
            if display_height > max_h {
                let ratio = max_h as f32 / display_height as f32;
                display_height = max_h;
                display_width = (display_width as f32 * ratio) as u32;
            }
        }

        (display_width, display_height)
    }

    /// Generate unique image ID
    fn generate_image_id(&self) -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Convert image to Sixel format (simplified)
    fn convert_to_sixel(
        &self,
        _data: &[u8],
        _format: &ImageFormat,
        _max_width: Option<u32>,
        _max_height: Option<u32>,
    ) -> Result<Vec<u8>> {
        // This would require a full image processing library in production
        // For now, return a simple test pattern
        Ok(b"\"1;1;8;8#0;2;0;0;0#1;2;100;0;0$#1!8~!8~!8~!8~!8~!8~".to_vec())
    }

    /// Transmit image using iTerm2 protocol
    pub fn transmit_iterm2_image(
        &self,
        data: &[u8],
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> Result<()> {
        if !self.capabilities.iterm2_images {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "iTerm2 images not supported",
            )
            .into());
        }

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);

        let mut params = String::new();
        if let Some(width) = max_width {
            params.push_str(&format!("width={}px;", width));
        }
        if let Some(height) = max_height {
            params.push_str(&format!("height={}px;", height));
        }

        self.write_str(&format!(
            "\x1b]1337;File={}inline=1:{}\x07",
            params, encoded
        ))?;

        Ok(())
    }

    /// Detect terminal capabilities by sending queries
    fn detect_capabilities(&mut self) -> Result<()> {
        use std::time::Duration;

        // First, set up for reading responses
        #[cfg(unix)]
        {
            // Temporarily set non-blocking for capability detection
            self.inner.set_nonblocking(true)?;
        }

        // Send every startup capability query, ten milliseconds apart.
        for query in sequences::STARTUP_QUERIES {
            self.write(query)?;
            std::thread::sleep(Duration::from_millis(10));
        }

        // Additional queries for more capabilities
        self.write(b"\x1b[?1;2c")?; // Request terminal ID
        std::thread::sleep(Duration::from_millis(10));
        self.write(b"\x1b]10;?\x1b\\")?; // Query foreground color
        std::thread::sleep(Duration::from_millis(10));
        self.write(b"\x1b]11;?\x1b\\")?; // Query background color
        std::thread::sleep(Duration::from_millis(10));

        // Wait for responses to arrive
        std::thread::sleep(Duration::from_millis(50));

        // Read all available responses
        let mut buffer = [0u8; 8192];
        let mut total_read = 0;
        let timeout = Duration::from_millis(200);

        let start = std::time::Instant::now();
        while start.elapsed() < timeout && total_read < buffer.len() - 1 {
            #[cfg(unix)]
            {
                match self
                    .inner
                    .read(&mut buffer[total_read..], Some(Duration::from_millis(20)))
                {
                    Ok(n) if n > 0 => {
                        total_read += n;
                        // Continue reading if we got data
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Ok(_) => {
                        // No more data available
                        if total_read > 0 {
                            break; // We have some data, process it
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        if total_read > 0 {
                            break; // We have some data, process it
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            #[cfg(windows)]
            {
                // Windows capability detection is limited
                break;
            }
        }

        #[cfg(unix)]
        {
            // Restore blocking mode
            self.inner.set_nonblocking(false)?;
        }

        if total_read > 0 {
            let response = String::from_utf8_lossy(&buffer[..total_read]);
            self.parse_capability_responses(&response);

            // Debug: print what we received
            #[cfg(debug_assertions)]
            {
                log::debug!(
                    "Capability responses received ({} bytes): {:?}",
                    total_read,
                    response
                );
            }
        } else {
            // Fallback to environment-based detection
            self.detect_from_environment();
        }

        Ok(())
    }

    /// Parse terminal capability responses
    fn parse_capability_responses(&mut self, response: &str) {
        // Parse primary device attributes (DA1) - ESC[?...c
        if let Some(da1_start) = response.find("\x1b[?") {
            if let Some(da1_end) = response[da1_start..].find('c') {
                let da1 = &response[da1_start..da1_start + da1_end + 1];
                self.parse_da1_response(da1);
            }
        }

        // Parse secondary device attributes (DA2) - ESC[>...c
        if let Some(da2_start) = response.find("\x1b[>") {
            if let Some(da2_end) = response[da2_start..].find('c') {
                let da2 = &response[da2_start..da2_start + da2_end + 1];
                self.parse_da2_response(da2);
            }
        }

        // Parse Kitty graphics response - ESC_Gi=1;OK ESC\
        if response.contains("\x1b_Gi=1;OK\x1b\\") {
            self.capabilities.kitty_graphics = true;
        }

        // Parse DECRQM responses for various modes
        self.parse_decrqm_responses(response);
    }

    /// Parse DA1 (Primary Device Attributes) response
    fn parse_da1_response(&mut self, da1: &str) {
        // DA1 format: ESC[?<params>c
        // Extract parameters between ? and c
        if let Some(params_str) = da1.strip_prefix("\x1b[?").and_then(|s| s.strip_suffix("c")) {
            let params: Vec<u32> = params_str
                .split(';')
                .filter_map(|s| s.parse().ok())
                .collect();

            // Check for specific capabilities based on DA1 parameters
            for &param in &params {
                match param {
                    1 => {}  // VT100
                    2 => {}  // VT200
                    6 => {}  // VT102
                    7 => {}  // VT131
                    8 => {}  // VT132
                    9 => {}  // VT220
                    15 => {} // VT320
                    18 => {} // VT330
                    19 => {} // VT340
                    24 => {} // VT320
                    41 => {} // VT420
                    61 => {} // VT510
                    64 => {} // VT520
                    65 => {} // VT525
                    _ => {}
                }
            }

            // Modern terminals usually support these
            self.capabilities.true_color = true;
            self.capabilities.hyperlinks = true;
            self.capabilities.bracketed_paste = true;
            self.capabilities.focus_events = true;
        }
    }

    /// Parse DA2 (Secondary Device Attributes) response
    fn parse_da2_response(&mut self, da2: &str) {
        // DA2 format: ESC[><terminal_type>;<version>;<options>c
        if let Some(params_str) = da2.strip_prefix("\x1b[>").and_then(|s| s.strip_suffix("c")) {
            let params: Vec<&str> = params_str.split(';').collect();

            if let Some(&terminal_type) = params.first() {
                match terminal_type.parse::<u32>() {
                    Ok(0) => {
                        // VT100
                        self.capabilities.terminal_name = Some("VT100".to_string());
                    }
                    Ok(1) => {
                        // VT220
                        self.capabilities.terminal_name = Some("VT220".to_string());
                    }
                    Ok(2) => {
                        // VT240
                        self.capabilities.terminal_name = Some("VT240".to_string());
                    }
                    Ok(18) => {
                        // VT330
                        self.capabilities.terminal_name = Some("VT330".to_string());
                    }
                    Ok(19) => {
                        // VT340 - supports Sixel
                        self.capabilities.terminal_name = Some("VT340".to_string());
                        self.capabilities.sixel_graphics = true;
                    }
                    Ok(24) => {
                        // VT320
                        self.capabilities.terminal_name = Some("VT320".to_string());
                    }
                    Ok(41) => {
                        // VT420
                        self.capabilities.terminal_name = Some("VT420".to_string());
                    }
                    Ok(61) => {
                        // VT510
                        self.capabilities.terminal_name = Some("VT510".to_string());
                    }
                    Ok(64) => {
                        // VT520
                        self.capabilities.terminal_name = Some("VT520".to_string());
                    }
                    Ok(65) => {
                        // VT525
                        self.capabilities.terminal_name = Some("VT525".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    /// Parse DECRQM (Request Mode) responses
    fn parse_decrqm_responses(&mut self, response: &str) {
        // DECRQM response format: ESC[?<mode>;<value>$y
        let mut pos = 0;
        while let Some(start) = response[pos..].find("\x1b[?") {
            let abs_start = pos + start;
            if let Some(end) = response[abs_start..].find("$y") {
                let abs_end = abs_start + end + 2;
                let decrqm = &response[abs_start..abs_end];

                if let Some(params_str) = decrqm
                    .strip_prefix("\x1b[?")
                    .and_then(|s| s.strip_suffix("$y"))
                {
                    let params: Vec<u32> = params_str
                        .split(';')
                        .filter_map(|s| s.parse().ok())
                        .collect();

                    if params.len() >= 2 {
                        let mode = params[0];
                        let value = params[1];

                        match mode {
                            2026 => {
                                // Synchronized output
                                self.capabilities.synchronized_output = value == 1 || value == 2;
                            }
                            2027 => {
                                // Unicode core: the terminal's answer is the
                                // glyph capability report the charts follow
                                // (CHT-028); set keeps braille and blocks,
                                // reset switches charts to ASCII.
                                match value {
                                    1 | 3 => {
                                        crate::widgets::display::charts::report_glyph_support(true)
                                    }
                                    2 | 4 => {
                                        crate::widgets::display::charts::report_glyph_support(false)
                                    }
                                    _ => {}
                                }
                            }
                            1016 => {
                                // Pixel mouse
                                self.capabilities.pixel_mouse = value == 1 || value == 2;
                            }
                            2004 => {
                                // Bracketed paste
                                self.capabilities.bracketed_paste = value == 1 || value == 2;
                            }
                            1004 => {
                                // Focus events
                                self.capabilities.focus_events = value == 1 || value == 2;
                            }
                            _ => {}
                        }
                    }
                }

                pos = abs_end;
            } else {
                break;
            }
        }
    }

    /// Fallback environment-based detection
    fn detect_from_environment(&mut self) {
        let term = std::env::var("TERM").unwrap_or_default();
        let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();

        // True color detection
        self.capabilities.true_color = colorterm == "truecolor"
            || colorterm == "24bit"
            || term.contains("256color")
            || term_program == "iTerm.app"
            || term_program == "WezTerm"
            || std::env::var("KITTY_WINDOW_ID").is_ok();

        // Kitty graphics
        self.capabilities.kitty_graphics = std::env::var("KITTY_WINDOW_ID").is_ok();

        // iTerm2 images
        self.capabilities.iterm2_images = term_program == "iTerm.app";

        // Sixel support
        self.capabilities.sixel_graphics =
            term.contains("xterm") || term_program == "WezTerm" || term_program == "foot";

        // Modern terminal features
        self.capabilities.hyperlinks = !term.contains("screen") && !term.contains("tmux");
        self.capabilities.synchronized_output = term_program == "WezTerm"
            || std::env::var("KITTY_WINDOW_ID").is_ok()
            || term_program == "iTerm.app";
        self.capabilities.enhanced_keyboard = std::env::var("KITTY_WINDOW_ID").is_ok();
        self.capabilities.pixel_mouse =
            std::env::var("KITTY_WINDOW_ID").is_ok() || term_program == "WezTerm";
        self.capabilities.bracketed_paste = true;
        self.capabilities.focus_events = true;
    }
}

impl Drop for DirectTty {
    fn drop(&mut self) {
        let _ = self.inner.restore();
    }
}
