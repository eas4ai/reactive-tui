//! CLP-001: a copy reaches the terminal's clipboard through OSC 52.

use crate::common::app_input::{self, FramePredicate, Snapshot};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode, KeyEvent, KeyModifiers},
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// `ESC ] 52 ; c ; <"hello" in base64> ESC \`.
const HELLO: &[u8] = b"\x1b]52;c;aGVsbG8=\x1b\\";

fn ctrl(c: char) -> Option<Event> {
    Some(Event::Key(
        KeyEvent::new(KeyCode::Char(c)).with_modifiers(KeyModifiers::ctrl()),
    ))
}

fn shown(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| frame.text.contains(needle))
}

fn hidden(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| !frame.text.contains(needle))
}

/// Types `c`, so that the App paints a frame after the step before it.
fn typed(c: char) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(KeyCode::Char(c))))
}

/// Whether `needle` occurs in the bytes some frame of `frames` wrote.
fn written(frames: &[Snapshot], needle: &[u8]) -> bool {
    frames.iter().any(|frame| {
        frame
            .output
            .windows(needle.len())
            .any(|window| window == needle)
    })
}

/// A focused text input holding `hello`.
fn input() -> Element {
    builder::text_input().value("hello").build().auto_focus()
}

#[test]
#[serial_test::serial(clipboard)]
fn clp_001_a_text_inputs_copy_writes_the_selection_as_osc_52() {
    let frames = app_input::run_when_frame(
        Root(input()),
        (40, 5),
        vec![
            (shown("hello"), ctrl('a')),
            (shown("hello"), ctrl('c')),
            // The typed character replaces the selection, so the next frame
            // is one the App painted after the copy.
            (shown("hello"), typed('!')),
            (hidden("hello"), None),
        ],
    );
    assert!(
        written(&frames, HELLO),
        "Ctrl+C on the selected text writes the OSC 52 sequence for it:\n{:?}",
        frames
            .iter()
            .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
            .collect::<Vec<_>>()
    );
}

/// CLP-001: text handed to the App from elsewhere, as the clipboard's
/// thread hands it after a local paste, reaches the focused input as a
/// paste on the App's next turn.
#[test]
#[serial_test::serial(clipboard)]
fn clp_001_text_pasted_from_elsewhere_reaches_the_focused_input() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let handed = std::sync::Arc::new(AtomicBool::new(false));
    let hand_over = {
        let handed = handed.clone();
        Box::new(move |frame: &Snapshot| {
            if frame.text.contains("hello") && !handed.swap(true, Ordering::SeqCst) {
                reactive_tui::clipboard::paste_text(" from elsewhere");
                return true;
            }
            false
        }) as FramePredicate
    };
    let frames = app_input::run_when_frame(
        Root(input()),
        (40, 5),
        vec![
            (
                shown("hello"),
                Some(Event::Key(KeyEvent::new(KeyCode::End))),
            ),
            // The step's key is a no-op at the end of the text; a step
            // without an event ends the run.
            (hand_over, Some(Event::Key(KeyEvent::new(KeyCode::End)))),
            (shown("hello from elsewhere"), None),
        ],
    );
    assert!(
        frames
            .last()
            .is_some_and(|frame| frame.text.contains("hello from elsewhere")),
        "the text handed over was pasted at the cursor:\n{}",
        frames
            .last()
            .map(|frame| frame.text.clone())
            .unwrap_or_default()
    );
}

#[test]
#[serial_test::serial(clipboard)]
fn clp_001_a_frame_with_no_copy_carries_no_osc_52() {
    let frames = app_input::run_when_frame(
        Root(input()),
        (40, 5),
        vec![
            (
                shown("hello"),
                Some(Event::Key(KeyEvent::new(KeyCode::End))),
            ),
            (shown("hello"), typed('!')),
            (shown("hello!"), None),
        ],
    );
    assert!(
        !written(&frames, b"\x1b]52"),
        "no copy was requested, so no frame carries an OSC 52 sequence"
    );
}

#[test]
#[serial_test::serial(clipboard)]
fn clp_001_a_cut_goes_to_the_terminal_and_pastes_back() {
    let frames = app_input::run_when_frame(
        Root(input()),
        (40, 5),
        vec![
            (shown("hello"), ctrl('a')),
            (shown("hello"), ctrl('x')),
            (hidden("hello"), ctrl('v')),
            (shown("hello"), None),
        ],
    );
    assert!(
        written(&frames, HELLO),
        "Ctrl+X on the selected text writes the OSC 52 sequence for it"
    );
    assert!(
        frames
            .last()
            .is_some_and(|frame| frame.text.contains("hello")),
        "Ctrl+V after the cut puts the text back:\n{}",
        frames
            .last()
            .map(|frame| frame.text.clone())
            .unwrap_or_default()
    );
}
