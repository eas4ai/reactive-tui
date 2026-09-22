//! Terminal capability detection, mode control, environment inference,
//! OSC 52 clipboard framing, and image-protocol resolution.
//!
//! Ports `terminal.zig` (capability/response handling, mode setters,
//! environment overrides, clipboard writer) and the `resolveImageProtocol`
//! helper from `renderer.zig`. The crate stays I/O-free: callers pass a
//! `&mut Vec<u8>` byte sink that they flush to their own TTY writer.
//!
//! Requirement map: TRM-001 capabilities, TRM-002 mode bytes, TRM-003
//! restore/shutdown, TRM-004 kitty keyboard, TRM-005 mouse encoding,
//! TRM-008 environment inference, TRM-009 clipboard budget, TRM-010
//! image-protocol resolution.

fn push_kitty_push(sink: &mut Vec<u8>, flags: u8) {
    sink.extend_from_slice(b"\x1b[>");
    let mut digits = [0u8; 3];
    let mut len = 0;
    let mut v = flags;
    loop {
        digits[len] = b'0' + v % 10;
        len += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    for d in digits[..len].iter().rev() {
        sink.push(*d);
    }
    sink.push(b'u');
}

// ---------------------------------------------------------------------------
// Sequence table (mirrors ansi.zig; byte-exact).
// ---------------------------------------------------------------------------

/// Enter alternate screen (`DECSET 1049`).
pub const ALT_ENTER: &str = "\x1b[?1049h";
/// Leave alternate screen (`DECRST 1049`).
pub const ALT_EXIT: &str = "\x1b[?1049l";
const ENABLE_MOUSE_TRACKING: &str = "\x1b[?1000h";
const DISABLE_MOUSE_TRACKING: &str = "\x1b[?1000l";
const ENABLE_BUTTON_EVENT_TRACKING: &str = "\x1b[?1002h";
const DISABLE_BUTTON_EVENT_TRACKING: &str = "\x1b[?1002l";
const ENABLE_ANY_EVENT_TRACKING: &str = "\x1b[?1003h";
const DISABLE_ANY_EVENT_TRACKING: &str = "\x1b[?1003l";
const ENABLE_SGR_MOUSE_MODE: &str = "\x1b[?1006h";
const DISABLE_SGR_MOUSE_MODE: &str = "\x1b[?1006l";
const FOCUS_SET: &str = "\x1b[?1004h";
const FOCUS_RESET: &str = "\x1b[?1004l";
const BRACKETED_PASTE_SET: &str = "\x1b[?2004h";
const BRACKETED_PASTE_RESET: &str = "\x1b[?2004l";
const KITTY_KEYBOARD_POP: &str = "\x1b[<u";
const MODIFY_OTHER_KEYS_SET: &str = "\x1b[>4;1m";
const MODIFY_OTHER_KEYS_RESET: &str = "\x1b[>4;0m";
const COLOR_SCHEME_SET: &str = "\x1b[?2031h";
const COLOR_SCHEME_RESET: &str = "\x1b[?2031l";

/// Default progressive-enhancement flags pushed for kitty keyboard
/// (`0b00101`: disambiguate keys + report event types).
pub const DEFAULT_KITTY_KEYBOARD_FLAGS: u8 = 0b00101;

/// Chunk size for GNU screen DCS passthrough envelopes.
pub const SCREEN_PASSTHROUGH_CHUNK_SIZE: usize = 252;

// ---------------------------------------------------------------------------
// TRM-001: capability model.
// ---------------------------------------------------------------------------

/// Unicode width strategy negotiated with the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WidthMethod {
    /// Full `unicode` table path (default).
    #[default]
    Unicode,
    /// Conservative `wcwidth` path (tmux, Apple Terminal, forced).
    Wcwidth,
    /// Unicode tables without zero-width-joiner sequences (forced).
    NoZwj,
}

/// Probed terminal capabilities. All flags start `false` (TRM-001); query
/// responses and environment facts only ever turn them on, except the
/// explicit vscode downgrade handled by [`EnvCaps`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub kitty_keyboard: bool,
    pub kitty_graphics: bool,
    pub rgb: bool,
    pub ansi256: bool,
    pub unicode: WidthMethod,
    pub sgr_pixels: bool,
    pub color_scheme_updates: bool,
    pub explicit_width: bool,
    pub scaled_text: bool,
    pub sixel: bool,
    pub focus_tracking: bool,
    pub sync: bool,
    pub bracketed_paste: bool,
    pub hyperlinks: bool,
    pub osc52: bool,
    pub notifications: bool,
    pub explicit_cursor_positioning: bool,
    pub remote: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            kitty_keyboard: false,
            kitty_graphics: false,
            rgb: false,
            ansi256: false,
            unicode: WidthMethod::Unicode,
            sgr_pixels: false,
            color_scheme_updates: false,
            explicit_width: false,
            scaled_text: false,
            sixel: false,
            focus_tracking: false,
            sync: false,
            bracketed_paste: false,
            hyperlinks: false,
            osc52: false,
            notifications: false,
            explicit_cursor_positioning: false,
            remote: false,
        }
    }
}

/// Tri-state OSC 52 answer. `Unknown` stays optimistic: a missing or
/// inconclusive capability response must not block clipboard emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Osc52Support {
    #[default]
    Unknown,
    Supported,
    Unsupported,
}

impl Osc52Support {
    /// Mirrors `canWriteClipboard`: only an explicit refusal blocks writes.
    pub fn may_write(self) -> bool {
        self != Osc52Support::Unsupported
    }
}

fn contains_case_insensitive(haystack: &[u8], needle: &str) -> bool {
    let n = needle.as_bytes();
    if n.is_empty() || haystack.len() < n.len() {
        return false;
    }
    haystack.windows(n.len()).any(|w| w.eq_ignore_ascii_case(n))
}

const OSC52_TERMS: &[&str] = &[
    "iterm",
    "kitty",
    "alacritty",
    "wezterm",
    "contour",
    "foot",
    "rio",
    "ghostty",
    "tmux",
    "screen",
];

const HYPERLINK_TERMS: &[&str] = &["ghostty", "kitty", "wezterm", "alacritty", "foot", "iterm"];

impl Capabilities {
    /// Fold one raw query-response blob into the capability set.
    ///
    /// Mirrors the response half of `terminal.zig`: a `CSI ? <flags> u`
    /// reply proves kitty keyboard; a `tmux`/`alacritty` token proves the
    /// corresponding fallbacks; known terminal names prove OSC 52 and
    /// hyperlink support.
    pub fn apply_query_response(&mut self, response: &[u8]) {
        // Kitty keyboard protocol: `CSI ? <0-31> u`.
        let mut i = 0;
        while i + 4 < response.len() {
            if response[i] == 0x1b
                && response[i + 1] == b'['
                && response[i + 2] == b'?'
                && response[i + 3].is_ascii_digit()
            {
                let mut end = i + 3;
                while end < response.len() && response[end].is_ascii_digit() {
                    end += 1;
                }
                if end < response.len() && response[end] == b'u' {
                    self.kitty_keyboard = true;
                    break;
                }
            }
            i += 1;
        }

        if response.windows(4).any(|w| w == b"tmux") {
            self.unicode = WidthMethod::Wcwidth;
            self.explicit_cursor_positioning = true;
        }
        if contains_case_insensitive(response, "alacritty") {
            self.explicit_cursor_positioning = true;
        }
        if !self.osc52
            && OSC52_TERMS
                .iter()
                .any(|t| contains_case_insensitive(response, t))
        {
            self.osc52 = true;
        }
        if !self.hyperlinks
            && HYPERLINK_TERMS
                .iter()
                .any(|t| contains_case_insensitive(response, t))
        {
            self.hyperlinks = true;
        }
    }

    /// Fold an XTGETTCAP `Ms` (`4d73`) reply into OSC 52 state. Returns
    /// `true` when the reply proves support.
    pub fn note_xtgettcap_ms(&mut self, osc52: &mut Osc52Support, response: &[u8]) -> bool {
        // DCS `1+r4d73=<even-length hex>` terminated by `ESC \`.
        let mut scan = 0;
        while scan < response.len() {
            let start = match response[scan..].windows(2).position(|w| w == b"\x1bP") {
                Some(p) => scan + p,
                None => return false,
            };
            let body_start = start + 2;
            let end = match response[body_start..]
                .windows(2)
                .position(|w| w == b"\x1b\\")
            {
                Some(p) => body_start + p,
                None => return false,
            };
            scan = end + 2;
            let body = &response[body_start..end];
            if body.len() < 6 || body[0] != b'1' || &body[1..3] != b"+r" {
                continue;
            }
            let result = &body[3..];
            let sep = match result.iter().position(|b| *b == b'=') {
                Some(p) => p,
                None => continue,
            };
            if result[..sep].eq_ignore_ascii_case(b"4d73") {
                let value = &result[sep + 1..];
                if !value.is_empty()
                    && value.len().is_multiple_of(2)
                    && value.iter().all(|b| b.is_ascii_hexdigit())
                {
                    *osc52 = Osc52Support::Supported;
                    self.osc52 = true;
                    return true;
                }
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// TRM-002..TRM-005: tracked output modes.
// ---------------------------------------------------------------------------

/// Live terminal output modes tracked by the session. The struct only
/// records what was successfully emitted; every setter is idempotent so a
/// repeated call emits nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TermModes {
    pub mouse: bool,
    /// Retained across disable (mirrors `terminal.zig`, which never clears
    /// it): records the last active motion shape for diagnostics.
    pub mouse_movement: bool,
    pub bracketed_paste: bool,
    pub focus_tracking: bool,
    pub kitty_keyboard: bool,
    pub kitty_keyboard_flags: u8,
    pub modify_other_keys: bool,
    pub color_scheme_updates: bool,
    pub alt_screen: bool,
}

/// Enable click/drag (`movement = false`) or full-motion (`true`) mouse
/// tracking with SGR extended coordinates.
///
/// Some terminals treat `?1000/?1002/?1003` as one family where the last
/// sequence wins, so any-event tracking is reset first when motion is off.
pub fn set_mouse_mode(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool, movement: bool) {
    if enable {
        if modes.mouse && modes.mouse_movement == movement {
            return;
        }
        modes.mouse = true;
        modes.mouse_movement = movement;
        if !movement {
            sink.extend_from_slice(DISABLE_ANY_EVENT_TRACKING.as_bytes());
        }
        sink.extend_from_slice(ENABLE_MOUSE_TRACKING.as_bytes());
        sink.extend_from_slice(ENABLE_BUTTON_EVENT_TRACKING.as_bytes());
        if movement {
            sink.extend_from_slice(ENABLE_ANY_EVENT_TRACKING.as_bytes());
        }
        sink.extend_from_slice(ENABLE_SGR_MOUSE_MODE.as_bytes());
    } else {
        if !modes.mouse {
            return;
        }
        modes.mouse = false;
        write_mouse_disable_sequences(sink);
    }
}

fn write_mouse_disable_sequences(sink: &mut Vec<u8>) {
    sink.extend_from_slice(DISABLE_ANY_EVENT_TRACKING.as_bytes());
    sink.extend_from_slice(DISABLE_BUTTON_EVENT_TRACKING.as_bytes());
    sink.extend_from_slice(DISABLE_MOUSE_TRACKING.as_bytes());
    sink.extend_from_slice(DISABLE_SGR_MOUSE_MODE.as_bytes());
}

/// Best-effort shutdown path: emit the mouse disables even if tracked state
/// already drifted to `false` because an earlier write failed.
pub fn force_disable_mouse_mode(sink: &mut Vec<u8>, modes: &mut TermModes) {
    modes.mouse = false;
    write_mouse_disable_sequences(sink);
}

/// Bracketed paste (`?2004`) and focus tracking (`?1004`) setters.
pub fn set_bracketed_paste(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool) {
    sink.extend_from_slice(
        if enable {
            BRACKETED_PASTE_SET
        } else {
            BRACKETED_PASTE_RESET
        }
        .as_bytes(),
    );
    modes.bracketed_paste = enable;
}

/// Focus event tracking (`?1004`, reports `ESC [ I` / `ESC [ O`).
pub fn set_focus_tracking(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool) {
    sink.extend_from_slice(if enable { FOCUS_SET } else { FOCUS_RESET }.as_bytes());
    modes.focus_tracking = enable;
}

/// Kitty keyboard progressive enhancement: push `CSI > flags u` once,
/// pop with `CSI < u` on disable. Re-enabling after a change pushes again.
pub fn set_kitty_keyboard(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool, flags: u8) {
    if enable {
        if !modes.kitty_keyboard {
            push_kitty_push(sink, flags);
            modes.kitty_keyboard = true;
            modes.kitty_keyboard_flags = flags;
        }
    } else if modes.kitty_keyboard {
        sink.extend_from_slice(KITTY_KEYBOARD_POP.as_bytes());
        modes.kitty_keyboard = false;
        modes.kitty_keyboard_flags = 0;
    }
}

/// xterm `modifyOtherKeys` (`CSI > 4 ; 1 m`) and color-scheme updates
/// (`?2031`) setters.
pub fn set_modify_other_keys(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool) {
    sink.extend_from_slice(
        if enable {
            MODIFY_OTHER_KEYS_SET
        } else {
            MODIFY_OTHER_KEYS_RESET
        }
        .as_bytes(),
    );
    modes.modify_other_keys = enable;
}

/// Color-scheme update notifications (`?2031`).
pub fn set_color_scheme_updates(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool) {
    sink.extend_from_slice(
        if enable {
            COLOR_SCHEME_SET
        } else {
            COLOR_SCHEME_RESET
        }
        .as_bytes(),
    );
    modes.color_scheme_updates = enable;
}

/// Alternate-screen enter/exit. Emits only on transition so enter/exit stay
/// balanced across repeated calls.
pub fn set_alt_screen(sink: &mut Vec<u8>, modes: &mut TermModes, enable: bool) {
    if modes.alt_screen == enable {
        return;
    }
    modes.alt_screen = enable;
    sink.extend_from_slice(if enable { ALT_ENTER } else { ALT_EXIT }.as_bytes());
}

/// Re-send every currently-active mode unconditionally (TRM-003).
///
/// Called on focus-in: some terminals (notably Windows Terminal / ConPTY)
/// silently strip DEC private modes on focus loss. Kitty keyboard pops the
/// stale stack entry before re-pushing so the stack cannot grow; both
/// sequences share the sink so the terminal processes them atomically.
pub fn restore_modes(sink: &mut Vec<u8>, modes: &TermModes) {
    if modes.mouse {
        if !modes.mouse_movement {
            sink.extend_from_slice(DISABLE_ANY_EVENT_TRACKING.as_bytes());
        }
        sink.extend_from_slice(ENABLE_MOUSE_TRACKING.as_bytes());
        sink.extend_from_slice(ENABLE_BUTTON_EVENT_TRACKING.as_bytes());
        if modes.mouse_movement {
            sink.extend_from_slice(ENABLE_ANY_EVENT_TRACKING.as_bytes());
        }
        sink.extend_from_slice(ENABLE_SGR_MOUSE_MODE.as_bytes());
    }
    if modes.focus_tracking {
        sink.extend_from_slice(FOCUS_SET.as_bytes());
    }
    if modes.bracketed_paste {
        sink.extend_from_slice(BRACKETED_PASTE_SET.as_bytes());
    }
    if modes.kitty_keyboard {
        sink.extend_from_slice(KITTY_KEYBOARD_POP.as_bytes());
        push_kitty_push(sink, modes.kitty_keyboard_flags);
    }
    if modes.modify_other_keys {
        sink.extend_from_slice(MODIFY_OTHER_KEYS_SET.as_bytes());
    }
    if modes.color_scheme_updates {
        sink.extend_from_slice(COLOR_SCHEME_SET.as_bytes());
    }
    if modes.alt_screen {
        sink.extend_from_slice(ALT_ENTER.as_bytes());
    }
}

/// Balanced shutdown: leave the alternate screen when active, then emit the
/// full reset set unconditionally so every enable has its disable even when
/// tracked state drifted (TRM-003).
pub fn shutdown(sink: &mut Vec<u8>, modes: &mut TermModes) {
    if modes.alt_screen {
        sink.extend_from_slice(ALT_EXIT.as_bytes());
        modes.alt_screen = false;
    }
    force_disable_mouse_mode(sink, modes);
    sink.extend_from_slice(FOCUS_RESET.as_bytes());
    modes.focus_tracking = false;
    sink.extend_from_slice(BRACKETED_PASTE_RESET.as_bytes());
    modes.bracketed_paste = false;
    if modes.kitty_keyboard {
        sink.extend_from_slice(KITTY_KEYBOARD_POP.as_bytes());
        modes.kitty_keyboard = false;
        modes.kitty_keyboard_flags = 0;
    }
    sink.extend_from_slice(MODIFY_OTHER_KEYS_RESET.as_bytes());
    modes.modify_other_keys = false;
    sink.extend_from_slice(COLOR_SCHEME_RESET.as_bytes());
    modes.color_scheme_updates = false;
}

// ---------------------------------------------------------------------------
// TRM-008: environment inference (pure; no process access).
// ---------------------------------------------------------------------------

/// Detected multiplexer from xtversion or `TERM_PROGRAM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Multiplexer {
    #[default]
    None,
    Tmux,
    Zellij,
}

/// Configured image protocol (`OPENTUI_IMAGE_PROTOCOL` / explicit request).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageProtocol {
    #[default]
    Auto,
    Kitty,
    Sixel,
    Blocks,
}

/// Facts the caller resolves before calling [`detect_from_env`]: raw
/// `KEY=value` pairs plus identity facts that come from query responses
/// rather than the environment.
pub struct EnvFacts<'a> {
    /// Environment variables as `(name, value)` pairs.
    pub vars: &'a [(&'a str, &'a str)],
    /// Whether an xtversion response identified the terminal.
    pub from_xtversion: bool,
    /// Terminal name from xtversion (empty when unknown).
    pub xtversion_name: &'a str,
    /// Multiplexer already resolved from xtversion.
    pub multiplexer: Multiplexer,
    /// Response-derived seeds the environment stage folds into: OSC 52
    /// proof, hyperlink proof, and width strategy from query replies.
    pub osc52: bool,
    pub hyperlinks: bool,
    pub unicode: WidthMethod,
    /// Whether the terminal is foot (from xtversion).
    pub is_foot: bool,
    /// Windows platform build (mirrors `builtin.os.tag == .windows`).
    pub is_windows: bool,
}

fn env_get<'a>(vars: &'a [(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    vars.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn is_true_value(v: &str) -> bool {
    v == "true" || v == "1"
}

fn is_false_value(v: &str) -> bool {
    v == "false" || v == "0"
}

/// Capability facts inferred from the environment, mirroring
/// `checkEnvironmentOverrides` (TRM-008).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvCaps {
    pub osc52: bool,
    pub hyperlinks: bool,
    pub multiplexer: Multiplexer,
    pub remote: bool,
    pub unicode: WidthMethod,
    pub explicit_cursor_positioning: bool,
    pub graphics_enabled: bool,
    pub image_protocol: ImageProtocol,
    /// `OPENTUI_FORCE_EXPLICIT_WIDTH` override when present.
    pub explicit_width: Option<bool>,
}

pub fn detect_from_env(facts: &EnvFacts<'_>) -> EnvCaps {
    let vars = facts.vars;
    let mut caps = EnvCaps {
        osc52: facts.osc52,
        hyperlinks: facts.hyperlinks,
        multiplexer: facts.multiplexer,
        remote: env_get(vars, "SSH_CONNECTION").is_some()
            || env_get(vars, "SSH_CLIENT").is_some()
            || env_get(vars, "SSH_TTY").is_some()
            || env_get(vars, "MOSH_CONNECTION").is_some(),
        unicode: facts.unicode,
        explicit_cursor_positioning: false,
        graphics_enabled: true,
        image_protocol: ImageProtocol::Auto,
        explicit_width: None,
    };

    if env_get(vars, "OPENTUI_FORCE_WCWIDTH").is_some() {
        caps.unicode = WidthMethod::Wcwidth;
    }
    if env_get(vars, "OPENTUI_FORCE_UNICODE").is_some() {
        caps.unicode = WidthMethod::Unicode;
    }
    if env_get(vars, "OPENTUI_FORCE_NOZWJ").is_some() {
        caps.unicode = WidthMethod::NoZwj;
    }
    if let Some(val) = env_get(vars, "OPENTUI_FORCE_EXPLICIT_WIDTH") {
        if is_true_value(val) {
            caps.explicit_width = Some(true);
        } else if is_false_value(val) {
            caps.explicit_width = Some(false);
        }
    }

    if !facts.from_xtversion {
        if env_get(vars, "WT_SESSION").is_some() {
            caps.osc52 = true;
        }
        if !caps.osc52
            && (is_in_tmux(vars, caps.multiplexer)
                || is_in_screen(vars)
                || env_get(vars, "STY").is_some())
        {
            caps.osc52 = true;
        }
        if !caps.osc52
            && let Some(prog) = env_get(vars, "TERM_PROGRAM")
            && OSC52_TERMS.iter().any(|t| prog.eq_ignore_ascii_case(t))
        {
            caps.osc52 = true;
        }
        if !caps.osc52
            && let Some(term) = env_get(vars, "TERM")
            && (OSC52_TERMS
                .iter()
                .any(|t| contains_case_insensitive(term.as_bytes(), t))
                || term.contains("256color")
                || term.contains("xterm"))
        {
            caps.osc52 = true;
        }
    }

    if let Some(val) = env_get(vars, "OPENTUI_GRAPHICS") {
        if is_false_value(val) {
            caps.graphics_enabled = false;
        } else if is_true_value(val) {
            caps.graphics_enabled = true;
        }
    }

    if let Some(value) = env_get(vars, "OPENTUI_IMAGE_PROTOCOL") {
        if value.eq_ignore_ascii_case("auto") {
            caps.image_protocol = ImageProtocol::Auto;
        } else if value.eq_ignore_ascii_case("kitty") {
            caps.image_protocol = ImageProtocol::Kitty;
        } else if value.eq_ignore_ascii_case("sixel") {
            caps.image_protocol = ImageProtocol::Sixel;
        } else if value.eq_ignore_ascii_case("blocks") {
            caps.image_protocol = ImageProtocol::Blocks;
        }
    }

    if !facts.from_xtversion {
        if let Some(prog) = env_get(vars, "TERM_PROGRAM") {
            if caps.multiplexer != Multiplexer::Zellij && prog == "tmux" {
                caps.multiplexer = Multiplexer::Tmux;
                caps.unicode = WidthMethod::Wcwidth;
                caps.explicit_cursor_positioning = true;
            }
            // The vscode branch only clears kitty flags, which live in
            // `Capabilities` (response stage), not in `EnvCaps`.
            // Apple_Terminal/Alacritty set width/cursor facts directly.
            if prog == "Apple_Terminal" {
                caps.unicode = WidthMethod::Wcwidth;
            } else if prog == "Alacritty" {
                caps.explicit_cursor_positioning = true;
            }
        }
        if env_get(vars, "ALACRITTY_SOCKET").is_some() || env_get(vars, "ALACRITTY_LOG").is_some() {
            caps.explicit_cursor_positioning = true;
        }
    }

    // Hyperlinks: foot inside a multiplexer loses them; xtversion names and
    // TERM fallbacks (plus WSL + Windows Terminal) prove them.
    if facts.is_foot && caps.multiplexer != Multiplexer::None {
        caps.hyperlinks = false;
    }
    if !caps.hyperlinks
        && facts.from_xtversion
        && HYPERLINK_TERMS
            .iter()
            .any(|t| facts.xtversion_name.eq_ignore_ascii_case(t))
    {
        caps.hyperlinks = true;
    }
    if !caps.hyperlinks
        && !facts.from_xtversion
        && let Some(term) = env_get(vars, "TERM")
        && HYPERLINK_TERMS
            .iter()
            .any(|t| contains_case_insensitive(term.as_bytes(), t))
        && (!facts.is_foot || caps.multiplexer == Multiplexer::None)
    {
        caps.hyperlinks = true;
    }
    let is_wsl =
        env_get(vars, "WSL_DISTRO_NAME").is_some() || env_get(vars, "WSL_INTEROP").is_some();
    if !caps.hyperlinks
        && !facts.from_xtversion
        && is_wsl
        && env_get(vars, "WT_SESSION").is_some()
        && let Some(term) = env_get(vars, "TERM")
        && term.starts_with("xterm")
    {
        caps.hyperlinks = true;
    }
    if facts.is_windows && !facts.from_xtversion && caps.multiplexer == Multiplexer::None {
        caps.hyperlinks = true;
    }

    caps
}

fn is_in_tmux(vars: &[(&str, &str)], multiplexer: Multiplexer) -> bool {
    multiplexer == Multiplexer::Tmux || env_get(vars, "TMUX").is_some()
}

fn is_in_screen(vars: &[(&str, &str)]) -> bool {
    env_get(vars, "TERM").is_some_and(|t| t.starts_with("screen")) || env_get(vars, "STY").is_some()
}

// ---------------------------------------------------------------------------
// TRM-009: OSC 52 clipboard budget and framing.
// ---------------------------------------------------------------------------

/// OSC 52 selection target (`52;<c>;<base64>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardTarget {
    #[default]
    Clipboard,
    Primary,
    Select,
    Secondary,
}

impl ClipboardTarget {
    pub fn to_char(self) -> u8 {
        match self {
            ClipboardTarget::Clipboard => b'c',
            ClipboardTarget::Primary => b'p',
            ClipboardTarget::Select => b's',
            ClipboardTarget::Secondary => b'q',
        }
    }
}

/// Passthrough envelope around the OSC 52 sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardPassthrough {
    /// Raw `OSC 52 ... ST`.
    #[default]
    Direct,
    /// Wrapped in tmux DCS with `ESC` doubling.
    Tmux,
    /// Chunked into screen DCS envelopes with `BEL` terminator.
    Screen,
}

const OSC52_DIRECT_FRAMING: usize = "\x1b]52;c;".len() + "\x1b\\".len();
const OSC52_SCREEN_FRAMING: usize = "\x1b]52;c;".len() + 1; // BEL terminator
const TMUX_DCS_START: &str = "\x1bPtmux;";
const TMUX_DCS_END: &str = "\x1b\\";
const SCREEN_DCS_START: &str = "\x1bP";
const SCREEN_DCS_END: &str = "\x1b\\";

/// Clipboard write failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    /// Terminal refused OSC 52 (`Osc52Support::Unsupported`).
    NotSupported,
    /// Payload or framing arithmetic overflowed.
    PayloadTooLarge,
}

fn base64_encoded_len(payload_len: usize) -> Result<usize, ClipboardError> {
    let padded = payload_len
        .checked_add(2)
        .ok_or(ClipboardError::PayloadTooLarge)?;
    let groups = padded / 3;
    groups.checked_mul(4).ok_or(ClipboardError::PayloadTooLarge)
}

/// Exact byte budget for the framed OSC 52 write (TRM-009), mirroring
/// `clipboardSequenceSize` including tmux/screen envelopes.
pub fn clipboard_sequence_size(
    payload_len: usize,
    via: ClipboardPassthrough,
) -> Result<usize, ClipboardError> {
    let encoded = base64_encoded_len(payload_len)?;
    match via {
        ClipboardPassthrough::Direct => encoded
            .checked_add(OSC52_DIRECT_FRAMING)
            .ok_or(ClipboardError::PayloadTooLarge),
        ClipboardPassthrough::Tmux => {
            let inner = encoded
                .checked_add(OSC52_DIRECT_FRAMING)
                .ok_or(ClipboardError::PayloadTooLarge)?;
            inner
                .checked_add(2)
                .and_then(|v| v.checked_add(TMUX_DCS_START.len()))
                .and_then(|v| v.checked_add(TMUX_DCS_END.len()))
                .ok_or(ClipboardError::PayloadTooLarge)
        }
        ClipboardPassthrough::Screen => {
            let screen_seq = encoded
                .checked_add(OSC52_SCREEN_FRAMING)
                .ok_or(ClipboardError::PayloadTooLarge)?;
            let chunks = screen_seq
                .checked_sub(1)
                .map(|v| v / SCREEN_PASSTHROUGH_CHUNK_SIZE + 1)
                .ok_or(ClipboardError::PayloadTooLarge)?;
            let envelopes = chunks
                .checked_mul(SCREEN_DCS_START.len() + SCREEN_DCS_END.len())
                .ok_or(ClipboardError::PayloadTooLarge)?;
            screen_seq
                .checked_add(envelopes)
                .ok_or(ClipboardError::PayloadTooLarge)
        }
    }
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn push_base64(sink: &mut Vec<u8>, text: &[u8]) {
    for chunk in text.chunks(3) {
        let mut n: u32 = 0;
        for (i, b) in chunk.iter().enumerate() {
            n |= (*b as u32) << (16 - 8 * i);
        }
        let out = match chunk.len() {
            3 => 4,
            2 => 3,
            _ => 2,
        };
        for i in 0..out {
            sink.push(BASE64_ALPHABET[((n >> (18 - 6 * i)) & 0x3f) as usize]);
        }
        for _ in out..4 {
            sink.push(b'=');
        }
    }
}

/// Frame one OSC 52 clipboard write into the sink (TRM-009).
///
/// `may_write` is `Osc52Support::may_write()` from the session; a refusal
/// returns [`ClipboardError::NotSupported`] before any byte is emitted.
pub fn write_clipboard(
    sink: &mut Vec<u8>,
    target: ClipboardTarget,
    text: &[u8],
    via: ClipboardPassthrough,
    may_write: bool,
) -> Result<(), ClipboardError> {
    if !may_write {
        return Err(ClipboardError::NotSupported);
    }
    // Budget first: oversized payloads fail without partial output.
    clipboard_sequence_size(text.len(), via)?;

    let mut inner = Vec::new();
    inner.extend_from_slice(b"\x1b]52;");
    inner.push(target.to_char());
    inner.push(b';');
    push_base64(&mut inner, text);

    match via {
        ClipboardPassthrough::Direct => {
            sink.extend_from_slice(&inner);
            sink.extend_from_slice(b"\x1b\\");
        }
        ClipboardPassthrough::Tmux => {
            sink.extend_from_slice(TMUX_DCS_START.as_bytes());
            for b in inner.iter().chain(b"\x1b\\".iter()) {
                // Tmux passthrough doubles every ESC inside DCS.
                if *b == 0x1b {
                    sink.push(0x1b);
                }
                sink.push(*b);
            }
            sink.extend_from_slice(TMUX_DCS_END.as_bytes());
        }
        ClipboardPassthrough::Screen => {
            inner.push(0x07); // BEL terminator for screen envelopes
            for chunk in inner.chunks(SCREEN_PASSTHROUGH_CHUNK_SIZE) {
                sink.extend_from_slice(SCREEN_DCS_START.as_bytes());
                sink.extend_from_slice(chunk);
                sink.extend_from_slice(SCREEN_DCS_END.as_bytes());
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TRM-010: image-protocol resolution.
// ---------------------------------------------------------------------------

/// Rendering backend selected for image placements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolvedProtocol {
    Kitty,
    Sixel,
    /// Block-cell fallback rendering.
    #[default]
    Fallback,
}

/// Whether a forced sixel request must be refused (TRM-010).
///
/// Sixel that the terminal actually probed is always honored; otherwise a
/// kitty xtversion identity, any multiplexer, or Apple Terminal refuses.
pub fn refuses_forced_sixel(
    sixel_cap: bool,
    from_xtversion: bool,
    multiplexer: Multiplexer,
    term_name: &str,
) -> bool {
    if sixel_cap {
        return false;
    }
    if from_xtversion {
        if multiplexer != Multiplexer::None {
            return true;
        }
        return term_name.eq_ignore_ascii_case("kitty");
    }
    term_name.eq_ignore_ascii_case("Apple_Terminal")
}

/// Resolve the effective image protocol (TRM-010), mirroring
/// `resolveImageProtocol`: an explicit request wins unless it is a refused
/// forced sixel; `Auto` falls back inside tmux and otherwise follows
/// probed graphics capabilities down to block fallback.
pub fn resolve_image_protocol(
    requested: ImageProtocol,
    configured: ImageProtocol,
    in_tmux: bool,
    kitty_graphics: bool,
    sixel: bool,
    forced_sixel_refused: bool,
) -> ResolvedProtocol {
    let effective = match requested {
        ImageProtocol::Auto => configured,
        other => other,
    };
    if effective == ImageProtocol::Sixel && forced_sixel_refused {
        return ResolvedProtocol::Fallback;
    }
    match effective {
        ImageProtocol::Kitty => return ResolvedProtocol::Kitty,
        ImageProtocol::Sixel => return ResolvedProtocol::Sixel,
        ImageProtocol::Blocks => return ResolvedProtocol::Fallback,
        ImageProtocol::Auto => {}
    }
    if in_tmux {
        return ResolvedProtocol::Fallback;
    }
    if kitty_graphics {
        return ResolvedProtocol::Kitty;
    }
    if sixel {
        return ResolvedProtocol::Sixel;
    }
    ResolvedProtocol::Fallback
}
