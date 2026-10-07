//! This module provides platform related functions.

#[cfg(unix)]
pub(crate) use self::unix::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, size, window_size,
};
#[cfg(unix)]
#[cfg(feature = "events")]
pub use self::unix::{query_startup, supports_keyboard_enhancement};
#[cfg(windows)]
#[cfg(feature = "events")]
pub use self::windows::{query_startup, supports_keyboard_enhancement};

/// What the terminal answered to the startup queries. On Unix every field
/// can be set; on Windows only the background color is asked (INP-013), so
/// the others keep their defaults.
#[cfg(feature = "events")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StartupReplies {
    /// The keyboard enhancement flags, when the terminal reports the Kitty
    /// keyboard protocol.
    pub keyboard: Option<crate::event::KeyboardEnhancementFlags>,
    /// The background color, as 16-bit red, green and blue.
    pub background: Option<(u16, u16, u16)>,
    /// Whether the terminal accepts Kitty graphics sent directly (`t=d`).
    pub kitty_graphics: bool,
    /// Whether it also read Kitty graphics from shared memory (`t=s`), which
    /// only a terminal on the same machine can.
    pub kitty_shared_memory: bool,
    /// Whether its device attributes list Sixel graphics (attribute 4).
    pub sixel: bool,
}
#[cfg(all(windows, test))]
pub(crate) use self::windows::temp_screen_buffer;
#[cfg(windows)]
pub(crate) use self::windows::{
    clear, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, scroll_down, scroll_up,
    set_size, set_window_title, size, window_size,
};

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub mod file_descriptor;
#[cfg(unix)]
mod unix;
