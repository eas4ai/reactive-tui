//! Direct terminal capability detection using escape sequence queries
//!
//! Detects terminal capabilities through both direct queries and environment variables.

use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

/// Terminal capabilities we actually care about
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerminalCapabilities {
    // Core rendering
    /// Color depth support level
    pub color_depth: ColorDepth,
    /// Unicode character support
    pub unicode: bool,

    // Graphics protocols
    /// Sixel graphics protocol support
    pub sixel: bool,
    /// Kitty graphics protocol support
    pub kitty_graphics: bool,
    /// iTerm2 graphics protocol support
    pub iterm2_graphics: bool,

    // Input enhancements
    /// Enhanced keyboard protocol support
    pub enhanced_keyboard: bool,
    /// Pixel-level mouse tracking support
    pub pixel_mouse: bool,

    // Performance features
    /// Synchronized output support for flicker-free updates
    pub synchronized_output: bool,
}

/// The first of `TERM_PROGRAM` and `TERM` that holds more than space.
fn identity_from(term_program: Option<String>, term: Option<String>) -> Option<String> {
    [term_program, term]
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

/// Terminal color depth capabilities
///
/// Represents the color support level of the terminal, from monochrome
/// to full 24-bit true color support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorDepth {
    /// Color depth is unknown - defaults to 16 colors as safe fallback
    #[default]
    Unknown,
    /// Monochrome terminal - supports only 2 colors (black and white)
    Monochrome,
    /// Basic 16-color support (8 colors + bright variants)
    Colors16,
    /// Extended 256-color palette support
    Colors256,
    /// Full 24-bit RGB true color support (16.7 million colors)
    TrueColor,
}

impl ColorDepth {
    /// Convert to the number of colors supported
    pub fn color_count(&self) -> u32 {
        match self {
            ColorDepth::Unknown => 16, // Safe fallback
            ColorDepth::Monochrome => 2,
            ColorDepth::Colors16 => 16,
            ColorDepth::Colors256 => 256,
            ColorDepth::TrueColor => 16_777_216,
        }
    }

    /// Check if this depth supports at least the given number of colors
    pub fn supports(&self, colors: u32) -> bool {
        self.color_count() >= colors
    }
}

// Standard terminal query sequences
const PRIMARY_DEVICE_ATTRS: &[u8] = b"\x1b[c"; // DA1 - Basic terminal ID
const XT_VERSION: &[u8] = b"\x1b[>0q"; // Terminal version
const DECRQM_UNICODE: &[u8] = b"\x1b[?2027$p"; // Unicode support
const DECRQM_SGR_PIXELS: &[u8] = b"\x1b[?1016$p"; // Pixel mouse
const DECRQM_SYNC: &[u8] = b"\x1b[?2026$p"; // Synchronized output
const CSI_U_QUERY: &[u8] = b"\x1b[?u"; // Enhanced keyboard
const KITTY_GRAPHICS_QUERY: &[u8] = b"\x1b_Gi=1,a=q\x1b\\"; // Kitty graphics
const SIXEL_GEOMETRY_QUERY: &[u8] = b"\x1b[?2;1;0S"; // Sixel support

/// Query terminal capabilities directly
pub struct TerminalQuery {
    timeout: Duration,
}

impl Default for TerminalQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalQuery {
    /// Create a new terminal query with default timeout
    ///
    /// # Returns
    /// A new `TerminalQuery` with 100ms timeout
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_millis(100),
        }
    }

    /// Set timeout for query responses
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send all queries to the terminal
    pub fn send_queries(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(
            &[
                PRIMARY_DEVICE_ATTRS,
                XT_VERSION,
                DECRQM_UNICODE,
                DECRQM_SGR_PIXELS,
                DECRQM_SYNC,
                CSI_U_QUERY,
                KITTY_GRAPHICS_QUERY,
                SIXEL_GEOMETRY_QUERY,
            ]
            .concat(),
        )?;
        writer.flush()
    }

    /// Parse responses from the terminal
    pub fn parse_responses(&self, reader: &mut impl Read) -> io::Result<TerminalCapabilities> {
        let mut caps = TerminalCapabilities::default();
        let mut buffer = Vec::with_capacity(1024);

        // Try to read responses with actual timeout
        match self.read_with_timeout(reader, &mut buffer) {
            Ok(n) if n > 0 => {
                // We got some responses, parse them
                self.parse_response_buffer(&buffer, &mut caps);
            }
            _ => {
                // Timeout, error, or no data - not fatal, we have fallbacks
            }
        }

        // Always apply environment fallbacks to fill in gaps
        self.apply_env_fallbacks(&mut caps);

        Ok(caps)
    }

    /// Detect capabilities without terminal I/O (env vars only)
    pub fn detect_from_env() -> TerminalCapabilities {
        let mut caps = TerminalCapabilities::default();
        let query = Self::new();
        query.apply_env_fallbacks(&mut caps);
        caps
    }

    /// The host terminal's identity from the environment: `TERM_PROGRAM`,
    /// or `TERM` when that is unset or empty. No terminal reports which
    /// glyphs its font draws, so image fallback looks this identity up in
    /// a per-terminal table (docs/spec/blitters.md, BLT-002).
    pub fn host_identity() -> Option<String> {
        identity_from(
            std::env::var("TERM_PROGRAM").ok(),
            std::env::var("TERM").ok(),
        )
    }

    #[cfg(unix)]
    fn read_with_timeout(&self, reader: &mut impl Read, buffer: &mut Vec<u8>) -> io::Result<usize> {
        use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

        // Get the file descriptor
        let fd = io::stdin().as_raw_fd();

        // Save original flags and set non-blocking
        let original_flags = unsafe { fcntl(fd, F_GETFL, 0) };
        if original_flags == -1 {
            return Err(io::Error::last_os_error());
        }

        unsafe {
            if fcntl(fd, F_SETFL, original_flags | O_NONBLOCK) == -1 {
                return Err(io::Error::last_os_error());
            }
        }

        let mut temp_buf = [0u8; 256];
        let mut total_read = 0;
        let start = Instant::now();

        // Read with actual timeout
        let result = loop {
            if start.elapsed() > self.timeout {
                break Ok(total_read); // Timeout reached
            }

            match reader.read(&mut temp_buf) {
                Ok(0) => break Ok(total_read), // EOF
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    total_read += n;

                    // Check for response terminators
                    if buffer.ends_with(b"c") || buffer.ends_with(b"\\") {
                        break Ok(total_read);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly and retry
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => break Err(e),
            }

            // Safety limit
            if total_read > 4096 {
                break Ok(total_read);
            }
        };

        // Restore original flags
        unsafe {
            if fcntl(fd, F_SETFL, original_flags) == -1 {
                log::warn!(
                    "Failed to restore original fcntl flags: {}",
                    io::Error::last_os_error()
                );
            }
        }

        result
    }

    #[cfg(windows)]
    fn read_with_timeout(&self, reader: &mut impl Read, buffer: &mut Vec<u8>) -> io::Result<usize> {
        use winapi::um::commapi::SetCommTimeouts;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::{COMMTIMEOUTS, STD_INPUT_HANDLE};

        // Get handle to stdin
        let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        // Set up timeouts
        let mut timeouts = COMMTIMEOUTS {
            ReadIntervalTimeout: 0,
            ReadTotalTimeoutMultiplier: 0,
            ReadTotalTimeoutConstant: self.timeout.as_millis() as u32,
            WriteTotalTimeoutMultiplier: 0,
            WriteTotalTimeoutConstant: 0,
        };

        unsafe {
            if SetCommTimeouts(handle, &mut timeouts) == 0 {
                // Note: SetCommTimeouts might not work on console handles
                // Fall back to polling approach
                return self.read_with_polling_windows(reader, buffer);
            }
        }

        // Read normally with timeout set
        let mut temp_buf = [0u8; 256];
        let mut total_read = 0;

        loop {
            match reader.read(&mut temp_buf) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    total_read += n;

                    if buffer.ends_with(b"c") || buffer.ends_with(b"\\") {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }

            if total_read > 4096 {
                break;
            }
        }

        Ok(total_read)
    }

    #[cfg(windows)]
    fn read_with_polling_windows(
        &self,
        reader: &mut impl Read,
        buffer: &mut Vec<u8>,
    ) -> io::Result<usize> {
        use winapi::um::consoleapi::GetNumberOfConsoleInputEvents;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_INPUT_HANDLE;

        let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        let mut temp_buf = [0u8; 256];
        let mut total_read = 0;
        let start = Instant::now();

        loop {
            if start.elapsed() > self.timeout {
                break;
            }

            // Check if input is available
            let mut events_count = 0u32;
            unsafe {
                if GetNumberOfConsoleInputEvents(handle, &mut events_count) != 0 && events_count > 0
                {
                    // Input available, try to read
                    match reader.read(&mut temp_buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            buffer.extend_from_slice(&temp_buf[..n]);
                            total_read += n;

                            if buffer.ends_with(b"c") || buffer.ends_with(b"\\") {
                                break;
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    // No input, sleep and retry
                    std::thread::sleep(Duration::from_millis(10));
                }
            }

            if total_read > 4096 {
                break;
            }
        }

        Ok(total_read)
    }

    #[cfg(not(any(unix, windows)))]
    fn read_with_timeout(
        &self,
        _reader: &mut impl Read,
        _buffer: &mut Vec<u8>,
    ) -> io::Result<usize> {
        // WASM, embedded systems, or other platforms without terminal I/O
        // These platforms typically don't have terminals to query
        Ok(0)
    }

    /// Parse terminal response buffer to extract capability information
    ///
    /// Processes escape sequence responses from terminal queries including:
    /// - Device Attributes (DA1/DA2)
    /// - Color palette information  
    /// - Cursor position reports
    /// - Terminal identification strings
    pub fn parse_response_buffer(&self, buffer: &[u8], caps: &mut TerminalCapabilities) {
        // Parse Device Attributes (DA1) response
        if let Some(pos) = find_sequence(buffer, b"\x1b[?") {
            if let Some(end) = find_byte(&buffer[pos..], b'c') {
                let response = &buffer[pos..pos + end + 1];
                self.parse_da1_response(response, caps);
            }
        }

        // Parse DECRPM responses (mode reports)
        // Format: CSI ? Pm ; Ps $ y
        // Ps = 0: not recognized, 1: set, 2: reset, 3: permanently set, 4: permanently reset

        // Check for Unicode support response. The terminal's own answer is
        // the capability report charts follow (CHT-028): set (1, 3) keeps
        // braille and block glyphs, reset (2, 4) makes charts draw ASCII.
        if let Some(pos) = find_sequence(buffer, b"\x1b[?2027;") {
            if let Some(end) = find_byte(&buffer[pos + 8..], b'$') {
                let status = &buffer[pos + 8..pos + 8 + end];
                if status.starts_with(b"1") || status.starts_with(b"3") {
                    caps.unicode = true;
                    crate::widgets::display::charts::report_glyph_support(true);
                } else if status.starts_with(b"2") || status.starts_with(b"4") {
                    caps.unicode = false;
                    crate::widgets::display::charts::report_glyph_support(false);
                }
            }
        }

        // Check for pixel mouse response
        if let Some(pos) = find_sequence(buffer, b"\x1b[?1016;") {
            if let Some(end) = find_byte(&buffer[pos + 8..], b'$') {
                let status = &buffer[pos + 8..pos + 8 + end];
                if status.starts_with(b"1") || status.starts_with(b"3") {
                    caps.pixel_mouse = true;
                }
            }
        }

        // Check for synchronized output response
        if let Some(pos) = find_sequence(buffer, b"\x1b[?2026;") {
            if let Some(end) = find_byte(&buffer[pos + 8..], b'$') {
                let status = &buffer[pos + 8..pos + 8 + end];
                if status.starts_with(b"1") || status.starts_with(b"3") {
                    caps.synchronized_output = true;
                }
            }
        }

        // Check for Kitty graphics response
        if find_sequence(buffer, b"\x1b_Gi=31;OK\x1b\\").is_some() {
            caps.kitty_graphics = true;
        }

        // Check for enhanced keyboard (CSI u) response
        if find_sequence(buffer, b"\x1b[?").is_some() && buffer.contains(&b'u') {
            caps.enhanced_keyboard = true;
        }

        // Check for sixel response (graphics geometry report)
        if find_sequence(buffer, b"\x1b[?2;").is_some() && buffer.contains(&b'S') {
            caps.sixel = true;
        }
    }

    fn parse_da1_response(&self, response: &[u8], caps: &mut TerminalCapabilities) {
        // Parse primary device attributes
        // Format: CSI ? Pm ; Pm ; ... Pm c
        // Common values:
        // 1: 132 columns
        // 4: sixel graphics
        // 22: ANSI color
        // 62: VT220 level

        if response.contains(&b'4') {
            caps.sixel = true;
        }

        // Most modern terminals support at least 256 colors
        // Check for ANSI color support (parameter 22)
        if response.windows(2).any(|w| w == b"22") && caps.color_depth == ColorDepth::Unknown {
            caps.color_depth = ColorDepth::Colors256;
        }
    }

    /// Apply environment-based fallback detection for terminal capabilities
    ///
    /// Uses environment variables like TERM, COLORTERM, and terminal-specific
    /// variables to infer capabilities when direct querying fails or is unavailable.
    /// This provides reasonable defaults based on common terminal configurations.
    pub fn apply_env_fallbacks(&self, caps: &mut TerminalCapabilities) {
        // Check COLORTERM for true color support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                caps.color_depth = ColorDepth::TrueColor;
            }
        }

        // Check TERM for color depth hints
        if caps.color_depth == ColorDepth::Unknown {
            if let Ok(term) = std::env::var("TERM") {
                if term.contains("256color") {
                    caps.color_depth = ColorDepth::Colors256;
                } else if term.contains("color") {
                    caps.color_depth = ColorDepth::Colors16;
                } else if term == "xterm" || term == "screen" {
                    // These usually support at least 16 colors
                    caps.color_depth = ColorDepth::Colors16;
                }
            }
        }

        // Check TERM_PROGRAM for specific terminal features
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "kitty" | "ghostty" => {
                    caps.kitty_graphics = true;
                    caps.enhanced_keyboard = true;
                    caps.color_depth = ColorDepth::TrueColor;
                    caps.unicode = true;
                }
                "iTerm.app" => {
                    caps.iterm2_graphics = true;
                    caps.color_depth = ColorDepth::TrueColor;
                    caps.unicode = true;
                }
                "WezTerm" => {
                    caps.sixel = true;
                    caps.iterm2_graphics = true;
                    caps.synchronized_output = true;
                    caps.color_depth = ColorDepth::TrueColor;
                    caps.unicode = true;
                }
                "alacritty" | "Alacritty" => {
                    caps.color_depth = ColorDepth::TrueColor;
                    caps.unicode = true;
                }
                "vscode" => {
                    // VS Code's terminal has some limitations
                    caps.color_depth = ColorDepth::TrueColor;
                    caps.unicode = true;
                }
                _ => {}
            }
        }

        // Check for specific terminal indicators
        if std::env::var("KITTY_WINDOW_ID").is_ok() {
            caps.kitty_graphics = true;
            caps.enhanced_keyboard = true;
        }

        if std::env::var("ITERM_SESSION_ID").is_ok() {
            caps.iterm2_graphics = true;
        }

        // Check for SSH - might have reduced capabilities
        if std::env::var("SSH_CONNECTION").is_ok() || std::env::var("SSH_TTY").is_ok() {
            // Be conservative over SSH unless we know otherwise
            if caps.color_depth == ColorDepth::Unknown {
                caps.color_depth = ColorDepth::Colors256;
            }
        }

        // Default to basic capabilities if nothing detected
        if caps.color_depth == ColorDepth::Unknown {
            caps.color_depth = ColorDepth::Colors16;
        }
    }
}

// Helper functions
fn find_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_depth_supports() {
        assert!(ColorDepth::TrueColor.supports(256));
        assert!(ColorDepth::Colors256.supports(256));
        assert!(!ColorDepth::Colors256.supports(257));
        assert!(ColorDepth::Colors16.supports(16));
        assert!(!ColorDepth::Colors16.supports(17));
    }

    #[test]
    fn test_parse_da1_with_sixel() {
        let mut caps = TerminalCapabilities::default();
        let query = TerminalQuery::new();
        let response = b"\x1b[?62;4;22c"; // VT220 with sixel and ANSI color
        query.parse_da1_response(response, &mut caps);
        assert!(caps.sixel);
        assert_eq!(caps.color_depth, ColorDepth::Colors256);
    }

    /// CHT-028: a mode 2027 reply of reset reports glyphs unavailable to the
    /// charts, and a reply of set reports them available again.
    #[test]
    fn a_unicode_mode_reset_reply_reports_no_glyph_support_to_charts() {
        use crate::widgets::display::charts::{
            glyph_support, report_glyph_support, GLYPH_REPORT_TEST_LOCK,
        };
        let _serial = GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let query = TerminalQuery::new();
        let mut caps = TerminalCapabilities::default();
        query.parse_response_buffer(b"\x1b[?2027;2$y", &mut caps);
        let after_reset = glyph_support();
        query.parse_response_buffer(b"\x1b[?2027;1$y", &mut caps);
        let after_set = glyph_support();
        report_glyph_support(true);
        assert!(!after_reset, "a reset reply must report glyphs unavailable");
        assert!(
            after_set && caps.unicode,
            "a set reply must report glyphs available"
        );
    }

    #[test]
    fn test_parse_unicode_enabled() {
        let mut caps = TerminalCapabilities::default();
        let query = TerminalQuery::new();
        let buffer = b"\x1b[?2027;1$y"; // Unicode enabled
        query.parse_response_buffer(buffer, &mut caps);
        assert!(caps.unicode);
    }

    #[test]
    fn test_parse_kitty_graphics() {
        let mut caps = TerminalCapabilities::default();
        let query = TerminalQuery::new();
        let buffer = b"\x1b_Gi=31;OK\x1b\\";
        query.parse_response_buffer(buffer, &mut caps);
        assert!(caps.kitty_graphics);
    }

    #[test]
    fn blt_002_host_identity_prefers_term_program_and_skips_empty_values() {
        let some = |value: &str| Some(value.to_string());
        assert_eq!(
            identity_from(some("ghostty"), some("xterm-ghostty")),
            some("ghostty")
        );
        assert_eq!(
            identity_from(some(""), some("xterm-kitty")),
            some("xterm-kitty")
        );
        assert_eq!(identity_from(None, some("linux")), some("linux"));
        assert_eq!(identity_from(some(" "), None), None);
        assert_eq!(identity_from(None, None), None);
    }

    /// Sets an environment variable for one test while holding the lock that
    /// serializes tests of process-wide terminal choices, and restores the
    /// variable's earlier value when dropped, even if the test panics.
    struct EnvGuard {
        name: &'static str,
        previous: Option<std::ffi::OsString>,
        _serial: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let serial = crate::widgets::display::charts::GLYPH_REPORT_TEST_LOCK
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let previous = std::env::var_os(name);
            unsafe {
                std::env::set_var(name, value);
            }
            Self {
                name,
                previous,
                _serial: serial,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    #[test]
    fn test_env_fallback_colorterm() {
        let _env = EnvGuard::set("COLORTERM", "truecolor");
        let caps = TerminalQuery::detect_from_env();
        assert_eq!(caps.color_depth, ColorDepth::TrueColor);
    }

    #[test]
    fn test_env_fallback_term_program() {
        let _env = EnvGuard::set("TERM_PROGRAM", "kitty");
        let caps = TerminalQuery::detect_from_env();
        assert!(caps.kitty_graphics);
        assert!(caps.enhanced_keyboard);
        assert_eq!(caps.color_depth, ColorDepth::TrueColor);
    }
}
