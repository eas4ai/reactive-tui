//! UNIX related logic for terminal manipulation.

#[cfg(feature = "events")]
use crate::event::KeyboardEnhancementFlags;
use crate::terminal::{
    sys::file_descriptor::{tty_fd, FileDesc},
    WindowSize,
};
#[cfg(feature = "libc")]
use libc::{
    cfmakeraw, ioctl, tcgetattr, tcsetattr, termios as Termios, winsize, STDOUT_FILENO, TCSANOW,
    TIOCGWINSZ,
};
use parking_lot::Mutex;
#[cfg(not(feature = "libc"))]
use rustix::{
    fd::AsFd,
    termios::{Termios, Winsize},
};

use std::{fs::File, io, process};
#[cfg(feature = "libc")]
use std::{
    mem,
    os::unix::io::{IntoRawFd, RawFd},
};

// Some(Termios) -> we're in the raw mode and this is the previous mode
// None -> we're not in the raw mode
static TERMINAL_MODE_PRIOR_RAW_MODE: Mutex<Option<Termios>> = parking_lot::const_mutex(None);

pub(crate) fn is_raw_mode_enabled() -> bool {
    TERMINAL_MODE_PRIOR_RAW_MODE.lock().is_some()
}

#[cfg(feature = "libc")]
impl From<winsize> for WindowSize {
    fn from(size: winsize) -> WindowSize {
        WindowSize {
            columns: size.ws_col,
            rows: size.ws_row,
            width: size.ws_xpixel,
            height: size.ws_ypixel,
        }
    }
}
#[cfg(not(feature = "libc"))]
impl From<Winsize> for WindowSize {
    fn from(size: Winsize) -> WindowSize {
        WindowSize {
            columns: size.ws_col,
            rows: size.ws_row,
            width: size.ws_xpixel,
            height: size.ws_ypixel,
        }
    }
}

#[allow(clippy::useless_conversion)]
#[cfg(feature = "libc")]
pub(crate) fn window_size() -> io::Result<WindowSize> {
    // http://rosettacode.org/wiki/Terminal_control/Dimensions#Library:_BSD_libc
    let mut size = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let file = File::open("/dev/tty").map(|file| (FileDesc::new(file.into_raw_fd(), true)));
    let fd = if let Ok(file) = &file {
        file.raw_fd()
    } else {
        // Fallback to libc::STDOUT_FILENO if /dev/tty is missing
        STDOUT_FILENO
    };

    if wrap_with_result(unsafe { ioctl(fd, TIOCGWINSZ.into(), &mut size) }).is_ok() {
        return Ok(size.into());
    }

    Err(std::io::Error::last_os_error().into())
}

#[cfg(not(feature = "libc"))]
pub(crate) fn window_size() -> io::Result<WindowSize> {
    let file = File::open("/dev/tty").map(|file| FileDesc::Owned(file.into()));
    let fd = if let Ok(file) = &file {
        file.as_fd()
    } else {
        // Fallback to libc::STDOUT_FILENO if /dev/tty is missing
        rustix::stdio::stdout()
    };
    let size = rustix::termios::tcgetwinsize(fd)?;
    Ok(size.into())
}

#[allow(clippy::useless_conversion)]
pub(crate) fn size() -> io::Result<(u16, u16)> {
    if let Ok(window_size) = window_size() {
        return Ok((window_size.columns, window_size.rows));
    }

    tput_size().ok_or_else(|| std::io::Error::last_os_error().into())
}

#[cfg(feature = "libc")]
pub(crate) fn enable_raw_mode() -> io::Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock();
    if original_mode.is_some() {
        return Ok(());
    }

    let tty = tty_fd()?;
    let fd = tty.raw_fd();
    let mut ios = get_terminal_attr(fd)?;
    let original_mode_ios = ios;
    raw_terminal_attr(&mut ios);
    set_terminal_attr(fd, &ios)?;
    // Keep it last - set the original mode only if we were able to switch to the raw mode
    *original_mode = Some(original_mode_ios);
    Ok(())
}

#[cfg(not(feature = "libc"))]
pub(crate) fn enable_raw_mode() -> io::Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock();
    if original_mode.is_some() {
        return Ok(());
    }

    let tty = tty_fd()?;
    let mut ios = get_terminal_attr(&tty)?;
    let original_mode_ios = ios.clone();
    ios.make_raw();
    set_terminal_attr(&tty, &ios)?;
    // Keep it last - set the original mode only if we were able to switch to the raw mode
    *original_mode = Some(original_mode_ios);
    Ok(())
}

/// Reset the raw mode.
///
/// More precisely, reset the whole termios mode to what it was before the first call
/// to [enable_raw_mode]. If you don't mess with termios outside of crossterm, it's
/// effectively disabling the raw mode and doing nothing else.
#[cfg(feature = "libc")]
pub(crate) fn disable_raw_mode() -> io::Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock();
    if let Some(original_mode_ios) = original_mode.as_ref() {
        let tty = tty_fd()?;
        set_terminal_attr(tty.raw_fd(), original_mode_ios)?;
        // Keep it last - remove the original mode only if we were able to switch back
        *original_mode = None;
    }
    Ok(())
}

#[cfg(not(feature = "libc"))]
pub(crate) fn disable_raw_mode() -> io::Result<()> {
    let mut original_mode = TERMINAL_MODE_PRIOR_RAW_MODE.lock();
    if let Some(original_mode_ios) = original_mode.as_ref() {
        let tty = tty_fd()?;
        set_terminal_attr(&tty, original_mode_ios)?;
        // Keep it last - remove the original mode only if we were able to switch back
        *original_mode = None;
    }
    Ok(())
}

#[cfg(not(feature = "libc"))]
fn get_terminal_attr(fd: impl AsFd) -> io::Result<Termios> {
    let result = rustix::termios::tcgetattr(fd)?;
    Ok(result)
}

#[cfg(not(feature = "libc"))]
fn set_terminal_attr(fd: impl AsFd, termios: &Termios) -> io::Result<()> {
    rustix::termios::tcsetattr(fd, rustix::termios::OptionalActions::Now, termios)?;
    Ok(())
}

/// Queries the terminal's support for progressive keyboard enhancement.
///
/// On unix systems, this function will block and possibly time out while
/// [`crossterm::event::read`](crate::event::read) or [`crossterm::event::poll`](crate::event::poll) are being called.
#[cfg(feature = "events")]
pub fn supports_keyboard_enhancement() -> io::Result<bool> {
    query_keyboard_enhancement_flags().map(|flags| flags.is_some())
}

/// Queries the terminal's currently active keyboard enhancement flags.
///
/// On unix systems, this function will block and possibly time out while
/// [`crossterm::event::read`](crate::event::read) or [`crossterm::event::poll`](crate::event::poll) are being called.
#[cfg(feature = "events")]
pub fn query_keyboard_enhancement_flags() -> io::Result<Option<KeyboardEnhancementFlags>> {
    if is_raw_mode_enabled() {
        query_keyboard_enhancement_flags_raw()
    } else {
        query_keyboard_enhancement_flags_nonraw()
    }
}

#[cfg(feature = "events")]
fn query_keyboard_enhancement_flags_nonraw() -> io::Result<Option<KeyboardEnhancementFlags>> {
    enable_raw_mode()?;
    let flags = query_keyboard_enhancement_flags_raw();
    disable_raw_mode()?;
    flags
}

#[cfg(feature = "events")]
fn query_keyboard_enhancement_flags_raw() -> io::Result<Option<KeyboardEnhancementFlags>> {
    use crate::event::{
        filter::{KeyboardEnhancementFlagsFilter, PrimaryDeviceAttributesFilter},
        poll_internal, read_internal, InternalEvent,
    };
    use std::io::Write;
    use std::time::Duration;

    // This is the recommended method for testing support for the keyboard enhancement protocol.
    // We send a query for the flags supported by the terminal and then the primary device attributes
    // query. If we receive the primary device attributes response but not the keyboard enhancement
    // flags, none of the flags are supported.
    //
    // See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/#detection-of-support-for-this-protocol>

    // ESC [ ? u        Query progressive keyboard enhancement flags (kitty protocol).
    // ESC [ c          Query primary device attributes.
    const QUERY: &[u8] = b"\x1B[?u\x1B[c";

    let result = File::open("/dev/tty").and_then(|mut file| {
        file.write_all(QUERY)?;
        file.flush()
    });
    if result.is_err() {
        let mut stdout = io::stdout();
        stdout.write_all(QUERY)?;
        stdout.flush()?;
    }

    loop {
        match poll_internal(
            Some(Duration::from_millis(2000)),
            &KeyboardEnhancementFlagsFilter,
        ) {
            Ok(true) => {
                match read_internal(&KeyboardEnhancementFlagsFilter) {
                    Ok(InternalEvent::KeyboardEnhancementFlags(current_flags)) => {
                        // Flush the PrimaryDeviceAttributes out of the event queue.
                        read_internal(&PrimaryDeviceAttributesFilter).ok();
                        return Ok(Some(current_flags));
                    }
                    _ => return Ok(None),
                }
            }
            Ok(false) => {
                return Err(io::Error::other(
                    "The keyboard enhancement status could not be read within a normal duration",
                ));
            }
            Err(_) => {}
        }
    }
}

/// What the terminal answered to the startup queries.
#[cfg(feature = "events")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StartupReplies {
    /// The keyboard enhancement flags, when the terminal reports the Kitty
    /// keyboard protocol.
    pub keyboard: Option<KeyboardEnhancementFlags>,
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

/// The ids of the Kitty graphics queries sent directly and through shared
/// memory.
#[cfg(feature = "events")]
const KITTY_DIRECT_QUERY: u32 = 31;
#[cfg(feature = "events")]
const KITTY_SHARED_QUERY: u32 = 32;

/// One black RGB pixel in POSIX shared memory, for the Kitty shared-memory
/// query. A terminal that reads it removes it; dropping it removes it
/// otherwise.
#[cfg(feature = "events")]
struct SharedPixel {
    name: String,
}

#[cfg(feature = "events")]
impl SharedPixel {
    fn create() -> Option<Self> {
        use rustix::fs::{ftruncate, Mode};
        use rustix::shm;
        let name = format!("/rtui-probe-{}", std::process::id());
        // A leftover from an earlier process with this id is not ours to keep.
        let _ = shm::unlink(name.as_str());
        let flags = shm::OFlags::CREATE | shm::OFlags::EXCL | shm::OFlags::RDWR;
        let fd = shm::open(name.as_str(), flags, Mode::RUSR | Mode::WUSR).ok()?;
        let pixel = SharedPixel { name };
        // Three zero bytes: one black pixel.
        ftruncate(&fd, 3).ok()?;
        Some(pixel)
    }
}

#[cfg(feature = "events")]
impl Drop for SharedPixel {
    fn drop(&mut self) {
        let _ = rustix::shm::unlink(self.name.as_str());
    }
}

/// Standard base64 with padding, for the shared-memory object's name.
#[cfg(feature = "events")]
fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, byte)| n | u32::from(*byte) << (16 - 8 * i));
        for i in 0..4 {
            out.push(if i <= chunk.len() {
                TABLE[(n >> (18 - 6 * i) & 63) as usize] as char
            } else {
                '='
            });
        }
    }
    out
}

/// Asks the terminal for its keyboard enhancement flags, its background
/// color and whether it accepts Kitty graphics sent directly and through
/// shared memory, then its primary device attributes, whose reply ends the
/// exchange and says whether it draws Sixel, and waits at most `timeout`
/// for the replies. Raw mode must be on. Other input read meanwhile, such
/// as keys typed at startup, stays queued for the next read, and the
/// replies never reach it as events.
#[cfg(feature = "events")]
pub fn query_startup(timeout: std::time::Duration) -> io::Result<StartupReplies> {
    use crate::event::{filter::StartupReplyFilter, poll_internal, read_internal, InternalEvent};
    use std::io::Write;
    use std::time::Instant;

    // PLT-004: every exit ends the exchange, including timeout and I/O errors.
    struct PendingReplies;
    impl Drop for PendingReplies {
        fn drop(&mut self) {
            crate::event::sys::unix::parse::STARTUP_REPLIES_PENDING
                .store(false, std::sync::atomic::Ordering::Release);
        }
    }

    // ESC [ ? u      the Kitty keyboard flags
    // ESC ] 11 ; ?   the background color, ended by ST
    // ESC _ G ...    Kitty graphics queries (a=q, so nothing is shown): one
    //                black pixel sent directly, and one in shared memory
    // ESC [ c        the primary device attributes, which every terminal answers
    let shared = SharedPixel::create();
    let mut query = b"\x1B[?u\x1B]11;?\x1B\\".to_vec();
    query.extend_from_slice(
        format!("\x1B_Gi={KITTY_DIRECT_QUERY},s=1,v=1,a=q,t=d,f=24;AAAA\x1B\\").as_bytes(),
    );
    if let Some(shared) = &shared {
        let name = base64(shared.name.as_bytes());
        query.extend_from_slice(
            format!("\x1B_Gi={KITTY_SHARED_QUERY},s=1,v=1,a=q,t=s,f=24;{name}\x1B\\").as_bytes(),
        );
    }
    query.extend_from_slice(b"\x1B[c");
    let query = query.as_slice();

    crate::event::sys::unix::parse::STARTUP_REPLIES_PENDING
        .store(true, std::sync::atomic::Ordering::Release);
    let _pending = PendingReplies;
    let written = File::open("/dev/tty").and_then(|mut file| {
        file.write_all(query)?;
        file.flush()
    });
    if written.is_err() {
        let mut stdout = io::stdout();
        stdout.write_all(query).and_then(|()| stdout.flush())?;
    }
    let deadline = Instant::now() + timeout;
    let mut replies = StartupReplies::default();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() || !poll_internal(Some(remaining), &StartupReplyFilter)? {
            return Ok(replies);
        }
        match read_internal(&StartupReplyFilter)? {
            InternalEvent::KeyboardEnhancementFlags(flags) => replies.keyboard = Some(flags),
            InternalEvent::BackgroundColor(red, green, blue) => {
                replies.background = Some((red, green, blue))
            }
            InternalEvent::KittyGraphicsReply { id, ok } => match id {
                KITTY_DIRECT_QUERY => replies.kitty_graphics = ok,
                KITTY_SHARED_QUERY => replies.kitty_shared_memory = ok,
                _ => {}
            },
            InternalEvent::PrimaryDeviceAttributes { sixel } => {
                replies.sixel = sixel;
                return Ok(replies);
            }
            _ => {}
        }
    }
}

/// execute tput with the given argument and parse
/// the output as a u16.
///
/// The arg should be "cols" or "lines"
fn tput_value(arg: &str) -> Option<u16> {
    let output = process::Command::new("tput").arg(arg).output().ok()?;
    let value = output
        .stdout
        .into_iter()
        .filter_map(|b| char::from(b).to_digit(10))
        .fold(0, |v, n| v * 10 + n as u16);

    if value > 0 {
        Some(value)
    } else {
        None
    }
}

/// Returns the size of the screen as determined by tput.
///
/// This alternate way of computing the size is useful
/// when in a subshell.
fn tput_size() -> Option<(u16, u16)> {
    match (tput_value("cols"), tput_value("lines")) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

#[cfg(feature = "libc")]
// Transform the given mode into an raw mode (non-canonical) mode.
fn raw_terminal_attr(termios: &mut Termios) {
    unsafe { cfmakeraw(termios) }
}

#[cfg(feature = "libc")]
fn get_terminal_attr(fd: RawFd) -> io::Result<Termios> {
    unsafe {
        let mut termios = mem::zeroed();
        wrap_with_result(tcgetattr(fd, &mut termios))?;
        Ok(termios)
    }
}

#[cfg(feature = "libc")]
fn set_terminal_attr(fd: RawFd, termios: &Termios) -> io::Result<()> {
    wrap_with_result(unsafe { tcsetattr(fd, TCSANOW, termios) })
}

#[cfg(feature = "libc")]
fn wrap_with_result(result: i32) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
