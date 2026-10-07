//! Part of tests/review_terminal.rs: the default backend's startup exchange,
//! run on a pseudo-terminal whose terminal side never answers the device
//! attributes query.

use crate::pty_support;
use std::time::{Duration, Instant};

const PROBE: &str = "REACTIVE_TUI_PLT_004_PROBE";

/// PLT-004: `ESC ]` typed during the startup exchange is held as a possible
/// reply; when the exchange ends by its timeout the held bytes come out as
/// Alt+], and a device attributes reply arriving later is consumed, not
/// delivered as keys.
#[test]
fn plt_004_a_key_held_as_a_possible_reply_is_released_when_the_exchange_ends() {
    if std::env::var_os(PROBE).is_some() {
        return probe();
    }
    let run = pty_support::run_test_with_input(
        "startup::plt_004_a_key_held_as_a_possible_reply_is_released_when_the_exchange_ends",
        &[(PROBE, "1")],
        &[
            // Alt+] typed while the exchange waits for replies.
            (Duration::from_millis(50), b"\x1b]"),
            // A device attributes reply long after the exchange timed out, and
            // after the copy has given up waiting for the held key: a later
            // byte would release the key, which is not the release required.
            (Duration::from_millis(2500), b"\x1b[?62;4c"),
        ],
    );
    let text = run.text();
    assert!(
        run.status.success() && text.contains("PLT004_OK"),
        "PLT-004: the copy on the pseudo-terminal failed ({}):\n{text}",
        run.status
    );
}

/// The copy on the pseudo-terminal.
fn probe() {
    use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

    crossterm::terminal::enable_raw_mode().expect("raw mode on the pseudo-terminal");
    // The terminal side answers nothing, so the exchange ends by its timeout.
    let _replies = crossterm::terminal::query_startup(Duration::from_millis(200));

    // The exchange ended by its 200 ms timeout; the held key is due at once.
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut first = None;
    while first.is_none() && Instant::now() < deadline {
        if poll(Duration::from_millis(50)).expect("poll") {
            first = Some(read().expect("read"));
        }
    }
    match &first {
        Some(Event::Key(key))
            if key.code == KeyCode::Char(']') && key.modifiers.contains(KeyModifiers::ALT) => {}
        other => {
            crossterm::terminal::disable_raw_mode().ok();
            println!("PLT004_FAIL: the ESC ] held during the exchange came out as {other:?}");
            std::process::exit(1);
        }
    }

    // The late reply must be recognized and consumed: no key may come of it.
    let deadline = Instant::now() + Duration::from_millis(3000);
    while Instant::now() < deadline {
        if poll(Duration::from_millis(50)).expect("poll") {
            let event = read().expect("read");
            crossterm::terminal::disable_raw_mode().ok();
            println!("PLT004_FAIL: the late device attributes reply came out as {event:?}");
            std::process::exit(1);
        }
    }
    crossterm::terminal::disable_raw_mode().expect("cooked mode again");
    println!("PLT004_OK");
}
