//! term-core commitment tests (TRM-001..005, TRM-008..010).

use suprtui::term::*;

// ---------------------------------------------------------------------------
// TRM-001: capability defaults and query-response detection.
// ---------------------------------------------------------------------------

#[test]
fn req_001_capability_defaults() {
    let caps = Capabilities::default();
    assert!(!caps.kitty_keyboard);
    assert!(!caps.kitty_graphics);
    assert!(!caps.rgb);
    assert!(!caps.ansi256);
    assert_eq!(caps.unicode, WidthMethod::Unicode);
    assert!(!caps.sgr_pixels);
    assert!(!caps.color_scheme_updates);
    assert!(!caps.explicit_width);
    assert!(!caps.scaled_text);
    assert!(!caps.sixel);
    assert!(!caps.focus_tracking);
    assert!(!caps.sync);
    assert!(!caps.bracketed_paste);
    assert!(!caps.hyperlinks);
    assert!(!caps.osc52);
    assert!(!caps.notifications);
    assert!(!caps.explicit_cursor_positioning);
    assert!(!caps.remote);
    // Optimistic tri-state: unknown must not block clipboard emission.
    assert_eq!(Osc52Support::default(), Osc52Support::Unknown);
    assert!(Osc52Support::Unknown.may_write());
    assert!(Osc52Support::Supported.may_write());
    assert!(!Osc52Support::Unsupported.may_write());
}

#[test]
fn req_001_query_responses() {
    // Kitty keyboard: `CSI ? <flags> u` for ghostty/alacritty-style `?0u`.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1b[?0u");
    assert!(caps.kitty_keyboard);

    // Multi-digit flags also prove the protocol.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1b[?31u");
    assert!(caps.kitty_keyboard);

    // A bare `u` or other CSI reply proves nothing.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1b[?62;22c");
    assert!(!caps.kitty_keyboard);

    // tmux identity: conservative widths plus explicit cursor positioning.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1bP>|tmux 3.5a\x1b\\\x1b[?1;2;4c");
    assert_eq!(caps.unicode, WidthMethod::Wcwidth);
    assert!(caps.explicit_cursor_positioning);

    // alacritty identity: explicit cursor positioning.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1b[?6c");
    assert!(!caps.explicit_cursor_positioning);
    caps.apply_query_response(b"alacritty");
    assert!(caps.explicit_cursor_positioning);

    // Known terminal names prove OSC 52 and hyperlinks (case-insensitive).
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1bP>|WezTerm 2024\x1b\\");
    assert!(caps.osc52);
    assert!(caps.hyperlinks);

    // Unknown terminals prove neither.
    let mut caps = Capabilities::default();
    caps.apply_query_response(b"\x1bP>|vt100\x1b\\");
    assert!(!caps.osc52);
    assert!(!caps.hyperlinks);
}

#[test]
fn req_001_xtgettcap_ms() {
    let mut caps = Capabilities::default();
    let mut support = Osc52Support::Unknown;
    // DCS `1+r4d73=<hex>` reply.
    let proven = caps.note_xtgettcap_ms(&mut support, b"\x1bP1+r4d73=6578616374\x1b\\");
    assert!(proven);
    assert_eq!(support, Osc52Support::Supported);
    assert!(caps.osc52);

    // Odd-length hex is not proof.
    let mut caps = Capabilities::default();
    let mut support = Osc52Support::Unknown;
    assert!(!caps.note_xtgettcap_ms(&mut support, b"\x1bP1+r4d73=abc\x1b\\"));
    assert_eq!(support, Osc52Support::Unknown);

    // Wrong capability key is not proof.
    let mut caps = Capabilities::default();
    let mut support = Osc52Support::Unknown;
    assert!(!caps.note_xtgettcap_ms(&mut support, b"\x1bP1+r546d=00\x1b\\"));
    assert_eq!(support, Osc52Support::Unknown);
}

// ---------------------------------------------------------------------------
// TRM-002/TRM-005: mode bytes and mouse encoding order.
// ---------------------------------------------------------------------------

#[test]
fn req_005_mouse_encoding() {
    // Click/drag: reset any-event tracking first so the family does not
    // collapse to the last-writer-wins mode on some terminals.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_mouse_mode(&mut sink, &mut modes, true, false);
    assert_eq!(sink, b"\x1b[?1003l\x1b[?1000h\x1b[?1002h\x1b[?1006h");
    assert!(modes.mouse);
    assert!(!modes.mouse_movement);

    // Idempotent: a repeated enable with the same shape emits nothing.
    sink.clear();
    set_mouse_mode(&mut sink, &mut modes, true, false);
    assert!(sink.is_empty());

    // Full motion: no reset prefix, any-event tracking enabled.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_mouse_mode(&mut sink, &mut modes, true, true);
    assert_eq!(sink, b"\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h");

    // Disable emits the full four-sequence reset in order.
    sink.clear();
    set_mouse_mode(&mut sink, &mut modes, false, false);
    assert_eq!(sink, b"\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1006l");
    assert!(!modes.mouse);

    // Disable when already off emits nothing.
    sink.clear();
    set_mouse_mode(&mut sink, &mut modes, false, false);
    assert!(sink.is_empty());

    // Switching shapes re-emits the full enable set.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_mouse_mode(&mut sink, &mut modes, true, false);
    sink.clear();
    set_mouse_mode(&mut sink, &mut modes, true, true);
    assert_eq!(sink, b"\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h");
}

#[test]
fn req_004_key_encoding() {
    // Kitty keyboard push carries the requested flags; pop is bare.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_kitty_keyboard(&mut sink, &mut modes, true, DEFAULT_KITTY_KEYBOARD_FLAGS);
    assert_eq!(sink, b"\x1b[>5u");
    assert!(modes.kitty_keyboard);
    assert_eq!(modes.kitty_keyboard_flags, 5);

    // Idempotent while active.
    sink.clear();
    set_kitty_keyboard(&mut sink, &mut modes, true, DEFAULT_KITTY_KEYBOARD_FLAGS);
    assert!(sink.is_empty());

    // Disable pops once and clears flags.
    set_kitty_keyboard(&mut sink, &mut modes, false, 0);
    assert_eq!(sink, b"\x1b[<u");
    assert!(!modes.kitty_keyboard);
    assert_eq!(modes.kitty_keyboard_flags, 0);

    // Disable when inactive emits nothing.
    sink.clear();
    set_kitty_keyboard(&mut sink, &mut modes, false, 0);
    assert!(sink.is_empty());
}

#[test]
fn req_002_mode_bytes() {
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_bracketed_paste(&mut sink, &mut modes, true);
    set_focus_tracking(&mut sink, &mut modes, true);
    set_modify_other_keys(&mut sink, &mut modes, true);
    set_color_scheme_updates(&mut sink, &mut modes, true);
    assert_eq!(sink, b"\x1b[?2004h\x1b[?1004h\x1b[>4;1m\x1b[?2031h");

    sink.clear();
    set_bracketed_paste(&mut sink, &mut modes, false);
    set_focus_tracking(&mut sink, &mut modes, false);
    set_modify_other_keys(&mut sink, &mut modes, false);
    set_color_scheme_updates(&mut sink, &mut modes, false);
    assert_eq!(sink, b"\x1b[?2004l\x1b[?1004l\x1b[>4;0m\x1b[?2031l");
}

// ---------------------------------------------------------------------------
// TRM-003: restore and balanced shutdown.
// ---------------------------------------------------------------------------

#[test]
fn req_002_mode_restore() {
    // Empty state restores to silence.
    let mut sink = Vec::new();
    restore_modes(&mut sink, &TermModes::default());
    assert!(sink.is_empty());

    // Every active mode is re-emitted unconditionally, including a repeat
    // of the enables that a focus-losing terminal may have stripped.
    let mut modes = TermModes::default();
    let mut setup = Vec::new();
    set_mouse_mode(&mut setup, &mut modes, true, false);
    set_bracketed_paste(&mut setup, &mut modes, true);
    set_focus_tracking(&mut setup, &mut modes, true);
    set_kitty_keyboard(&mut setup, &mut modes, true, 5);
    set_modify_other_keys(&mut setup, &mut modes, true);

    let mut sink = Vec::new();
    restore_modes(&mut sink, &modes);
    assert_eq!(
        sink,
        b"\x1b[?1003l\x1b[?1000h\x1b[?1002h\x1b[?1006h\
          \x1b[?1004h\x1b[?2004h\x1b[<u\x1b[>5u\x1b[>4;1m"
    );
}

#[test]
fn req_003_alt_screen_balance() {
    // Enter/exit pair on transition only; repeats stay silent.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_alt_screen(&mut sink, &mut modes, true);
    assert_eq!(sink, ALT_ENTER.as_bytes());
    sink.clear();
    set_alt_screen(&mut sink, &mut modes, true);
    assert!(sink.is_empty());
    set_alt_screen(&mut sink, &mut modes, false);
    assert_eq!(sink, ALT_EXIT.as_bytes());

    // Shutdown is balanced: alt screen exits and every tracked enable has
    // its disable even when state already drifted to off.
    let mut modes = TermModes::default();
    let mut sink = Vec::new();
    set_alt_screen(&mut sink, &mut modes, true);
    set_mouse_mode(&mut sink, &mut modes, true, true);
    set_bracketed_paste(&mut sink, &mut modes, true);
    sink.clear();
    // Simulate drift from a failed write.
    modes.mouse = false;
    shutdown(&mut sink, &mut modes);
    let out = String::from_utf8(sink).unwrap();
    assert!(out.starts_with("\x1b[?1049l"));
    assert!(out.contains("\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1006l"));
    assert!(out.contains("\x1b[?1004l"));
    assert!(out.contains("\x1b[?2004l"));
    assert!(out.contains("\x1b[>4;0m"));
    assert!(out.contains("\x1b[?2031l"));
    // All enables are cleared; the last motion shape is retained for
    // diagnostics exactly as the reference retains it.
    assert!(!modes.mouse);
    assert!(modes.mouse_movement);
    assert!(!modes.bracketed_paste);
    assert!(!modes.focus_tracking);
    assert!(!modes.kitty_keyboard);
    assert!(!modes.alt_screen);
}

// ---------------------------------------------------------------------------
// TRM-008: environment inference.
// ---------------------------------------------------------------------------

fn facts<'a>(vars: &'a [(&'a str, &'a str)]) -> EnvFacts<'a> {
    EnvFacts {
        vars,
        from_xtversion: false,
        xtversion_name: "",
        multiplexer: Multiplexer::None,
        is_foot: false,
        is_windows: false,
        osc52: false,
        hyperlinks: false,
        unicode: WidthMethod::Unicode,
    }
}

#[test]
fn req_008_environment() {
    // Empty environment: conservative defaults.
    let caps = detect_from_env(&facts(&[]));
    assert!(!caps.osc52);
    assert!(!caps.hyperlinks);
    assert!(!caps.remote);
    assert_eq!(caps.multiplexer, Multiplexer::None);
    assert_eq!(caps.unicode, WidthMethod::Unicode);
    assert!(caps.graphics_enabled);
    assert_eq!(caps.image_protocol, ImageProtocol::Auto);
    assert_eq!(caps.explicit_width, None);

    // Windows Terminal session: OSC 52 on.
    let caps = detect_from_env(&facts(&[("WT_SESSION", "abc")]));
    assert!(caps.osc52);

    // tmux multiplexer: OSC 52 on, conservative widths, explicit cursor.
    let caps = detect_from_env(&facts(&[("TERM_PROGRAM", "tmux")]));
    assert_eq!(caps.multiplexer, Multiplexer::Tmux);
    assert!(caps.osc52);
    assert_eq!(caps.unicode, WidthMethod::Wcwidth);
    assert!(caps.explicit_cursor_positioning);

    // TERM fallbacks: 256color/xterm-likes get OSC 52.
    let caps = detect_from_env(&facts(&[("TERM", "xterm-256color")]));
    assert!(caps.osc52);
    let caps = detect_from_env(&facts(&[("TERM_PROGRAM", "WezTerm")]));
    assert!(caps.osc52);

    // SSH presence marks the session remote.
    for var in ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY", "MOSH_CONNECTION"] {
        let caps = detect_from_env(&facts(&[(var, "1")]));
        assert!(caps.remote, "{var} should mark remote");
    }

    // Alacritty socket proves explicit cursor positioning.
    let caps = detect_from_env(&facts(&[("ALACRITTY_SOCKET", "/tmp/x")]));
    assert!(caps.explicit_cursor_positioning);
    let caps = detect_from_env(&facts(&[("TERM_PROGRAM", "Apple_Terminal")]));
    assert_eq!(caps.unicode, WidthMethod::Wcwidth);

    // Width and graphics overrides.
    let caps = detect_from_env(&facts(&[("OPENTUI_FORCE_WCWIDTH", "1")]));
    assert_eq!(caps.unicode, WidthMethod::Wcwidth);
    let caps = detect_from_env(&facts(&[("OPENTUI_FORCE_NOZWJ", "1")]));
    assert_eq!(caps.unicode, WidthMethod::NoZwj);
    let caps = detect_from_env(&facts(&[("OPENTUI_GRAPHICS", "false")]));
    assert!(!caps.graphics_enabled);
    let caps = detect_from_env(&facts(&[("OPENTUI_IMAGE_PROTOCOL", "SIXEL")]));
    assert_eq!(caps.image_protocol, ImageProtocol::Sixel);
    let caps = detect_from_env(&facts(&[("OPENTUI_FORCE_EXPLICIT_WIDTH", "0")]));
    assert_eq!(caps.explicit_width, Some(false));

    // Hyperlinks: known TERM, WSL + Windows Terminal, xtversion identity.
    let caps = detect_from_env(&facts(&[("TERM", "xterm-kitty")]));
    assert!(caps.hyperlinks);
    let caps = detect_from_env(&facts(&[
        ("WSL_DISTRO_NAME", "Ubuntu"),
        ("WT_SESSION", "x"),
        ("TERM", "xterm-256color"),
    ]));
    assert!(caps.hyperlinks);
    let xt = EnvFacts {
        from_xtversion: true,
        xtversion_name: "ghostty",
        ..facts(&[])
    };
    assert!(detect_from_env(&xt).hyperlinks);

    // foot inside a multiplexer clears response-proven hyperlinks, but the
    // xtversion identity immediately re-proves them (reference order).
    let foot_mux = EnvFacts {
        from_xtversion: true,
        xtversion_name: "foot",
        multiplexer: Multiplexer::Tmux,
        is_foot: true,
        hyperlinks: true,
        ..facts(&[])
    };
    assert!(detect_from_env(&foot_mux).hyperlinks);
    // With an unknown xtversion identity the foot clear sticks.
    let foot_unknown = EnvFacts {
        from_xtversion: true,
        xtversion_name: "vt100",
        multiplexer: Multiplexer::Tmux,
        is_foot: true,
        hyperlinks: true,
        ..facts(&[])
    };
    assert!(!detect_from_env(&foot_unknown).hyperlinks);
    // Response seeds survive the environment stage untouched otherwise.
    let seeded = EnvFacts {
        osc52: true,
        hyperlinks: true,
        unicode: WidthMethod::Wcwidth,
        ..facts(&[])
    };
    let caps = detect_from_env(&seeded);
    assert!(caps.osc52 && caps.hyperlinks);
    assert_eq!(caps.unicode, WidthMethod::Wcwidth);
}

// ---------------------------------------------------------------------------
// TRM-009: clipboard budget and framing.
// ---------------------------------------------------------------------------

#[test]
fn req_009_clipboard_budget() {
    // Direct: base64(3 -> 4) + 9 framing bytes (`ESC ] 52 ; c ;` + `ESC \`).
    assert_eq!(
        clipboard_sequence_size(3, ClipboardPassthrough::Direct),
        Ok(13)
    );
    // Empty payload still frames the header: 0 + 9.
    assert_eq!(
        clipboard_sequence_size(0, ClipboardPassthrough::Direct),
        Ok(9)
    );
    // Tmux: inner + 2 wrap bytes + 7 DCS start + 2 DCS end.
    assert_eq!(
        clipboard_sequence_size(3, ClipboardPassthrough::Tmux),
        Ok(13 + 2 + 7 + 2)
    );
    // Screen with a tiny payload: one 252-byte envelope of 4 bytes.
    // inner = 4 + 8 (BEL framing); one chunk -> +4 envelope.
    assert_eq!(
        clipboard_sequence_size(3, ClipboardPassthrough::Screen),
        Ok(12 + 4)
    );
    // Screen chunk boundary: payload whose framed length crosses 252
    // buys a second envelope.
    let one_chunk = clipboard_sequence_size(180, ClipboardPassthrough::Screen).unwrap();
    let two_chunk = clipboard_sequence_size(184, ClipboardPassthrough::Screen).unwrap();
    assert_eq!(one_chunk, 240 + 8 + 4);
    assert_eq!(two_chunk, 248 + 8 + 8);
    // Overflow maps to PayloadTooLarge, never wraps.
    assert_eq!(
        clipboard_sequence_size(usize::MAX, ClipboardPassthrough::Direct),
        Err(ClipboardError::PayloadTooLarge)
    );
}

#[test]
fn req_009_clipboard_writes() {
    // Direct write: `OSC 52 ; c ; <base64("Hi")> ST`.
    let mut sink = Vec::new();
    write_clipboard(
        &mut sink,
        ClipboardTarget::Clipboard,
        b"Hi",
        ClipboardPassthrough::Direct,
        true,
    )
    .unwrap();
    assert_eq!(sink, b"\x1b]52;c;SGk=\x1b\\");

    // Primary target selects `p`.
    let mut sink = Vec::new();
    write_clipboard(
        &mut sink,
        ClipboardTarget::Primary,
        b"A",
        ClipboardPassthrough::Direct,
        true,
    )
    .unwrap();
    assert_eq!(sink, b"\x1b]52;p;QQ==\x1b\\");

    // Refused terminals fail before emitting a byte.
    let mut sink = Vec::new();
    assert_eq!(
        write_clipboard(
            &mut sink,
            ClipboardTarget::Clipboard,
            b"Hi",
            ClipboardPassthrough::Direct,
            false,
        ),
        Err(ClipboardError::NotSupported)
    );
    assert!(sink.is_empty());

    // Tmux wraps in DCS; inner ESC bytes are doubled.
    let mut sink = Vec::new();
    write_clipboard(
        &mut sink,
        ClipboardTarget::Clipboard,
        b"Hi",
        ClipboardPassthrough::Tmux,
        true,
    )
    .unwrap();
    assert_eq!(sink, b"\x1bPtmux;\x1b\x1b]52;c;SGk=\x1b\x1b\\\x1b\\");

    // Screen chunks into BEL-terminated DCS envelopes.
    let mut sink = Vec::new();
    write_clipboard(
        &mut sink,
        ClipboardTarget::Clipboard,
        b"Hi",
        ClipboardPassthrough::Screen,
        true,
    )
    .unwrap();
    assert_eq!(sink, b"\x1bP\x1b]52;c;SGk=\x07\x1b\\");
}

// ---------------------------------------------------------------------------
// TRM-010: image-protocol resolution.
// ---------------------------------------------------------------------------

#[test]
fn req_010_image_protocol() {
    // Explicit requests win.
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Kitty,
            ImageProtocol::Auto,
            false,
            false,
            false,
            false
        ),
        ResolvedProtocol::Kitty
    );
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Sixel,
            ImageProtocol::Auto,
            false,
            false,
            true,
            false
        ),
        ResolvedProtocol::Sixel
    );
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Blocks,
            ImageProtocol::Kitty,
            false,
            true,
            true,
            false
        ),
        ResolvedProtocol::Fallback
    );
    // Refused forced sixel falls back.
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Sixel,
            ImageProtocol::Auto,
            false,
            false,
            false,
            true
        ),
        ResolvedProtocol::Fallback
    );
    // Auto: tmux forces fallback even with probed graphics.
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Auto,
            ImageProtocol::Auto,
            true,
            true,
            true,
            false
        ),
        ResolvedProtocol::Fallback
    );
    // Auto: kitty graphics preferred over sixel, then sixel, then blocks.
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Auto,
            ImageProtocol::Auto,
            false,
            true,
            true,
            false
        ),
        ResolvedProtocol::Kitty
    );
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Auto,
            ImageProtocol::Auto,
            false,
            false,
            true,
            false
        ),
        ResolvedProtocol::Sixel
    );
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Auto,
            ImageProtocol::Auto,
            false,
            false,
            false,
            false
        ),
        ResolvedProtocol::Fallback
    );
    // Auto with a configured override behaves like the explicit request.
    assert_eq!(
        resolve_image_protocol(
            ImageProtocol::Auto,
            ImageProtocol::Kitty,
            false,
            false,
            false,
            false
        ),
        ResolvedProtocol::Kitty
    );

    // Forced-sixel refusal: probed sixel is always honored.
    assert!(!refuses_forced_sixel(
        true,
        true,
        Multiplexer::Tmux,
        "kitty"
    ));
    assert!(refuses_forced_sixel(false, true, Multiplexer::Tmux, "foot"));
    assert!(refuses_forced_sixel(
        false,
        true,
        Multiplexer::None,
        "kitty"
    ));
    assert!(!refuses_forced_sixel(
        false,
        true,
        Multiplexer::None,
        "wezterm"
    ));
    assert!(refuses_forced_sixel(
        false,
        false,
        Multiplexer::None,
        "Apple_Terminal"
    ));
    assert!(!refuses_forced_sixel(
        false,
        false,
        Multiplexer::None,
        "xterm"
    ));
}
