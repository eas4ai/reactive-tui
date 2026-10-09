//! The terminal's clipboard (docs/spec/clipboard.md, CLP-001).
//!
//! A copy made inside an application reaches the user's clipboard through
//! the terminal: the App writes `ESC ] 52 ; c ; <base64> ESC \`, the OSC 52
//! sequence, to its terminal before its next frame, so a copy over SSH lands
//! on the user's machine. The clipboard hook's writer and a text input's copy
//! and cut ask for it here, on the App's thread; the App drains the requests
//! on each turn of its loop and before each frame, and hands them to its
//! backend. When a local clipboard command is available (wl-copy, xsel, xclip,
//! pbcopy or PowerShell) the copy runs it as well, so it keeps working in a
//! terminal that ignores OSC 52.

use std::cell::RefCell;

use base64::Engine as _;

thread_local! {
    // One queue per thread: a copy is requested on the thread that runs the
    // App, where its components, hooks and callbacks run, and that App drains
    // it. Two Apps on two threads, as a test binary runs them, never take each
    // other's requests.
    static PENDING: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

/// The OSC 52 sequence that asks the terminal to put `text` on the clipboard:
/// `ESC ] 52 ; c ; <the text's UTF-8 bytes in base64> ESC \`.
pub fn osc52(text: &str) -> Vec<u8> {
    let mut bytes = b"\x1b]52;c;".to_vec();
    bytes.extend_from_slice(
        base64::engine::general_purpose::STANDARD
            .encode(text.as_bytes())
            .as_bytes(),
    );
    bytes.extend_from_slice(b"\x1b\\");
    bytes
}

/// Asks the running App to put `text` on the terminal's clipboard: the OSC 52
/// sequence is written to the terminal before the App's next frame. Call it
/// on the thread that runs the App, where components, hooks and callbacks
/// run; a request made on another thread reaches no App.
pub fn copy_to_terminal(text: &str) {
    PENDING.with(|pending| pending.borrow_mut().push(osc52(text)));
}

/// Puts `text` on the clipboard every way this process can: through the
/// terminal (OSC 52) and, when a local clipboard command is available,
/// through that command as well. The result is the local command's; a
/// machine with no such command copies through the terminal alone and
/// reports no error.
pub fn copy(text: &str) -> Result<(), String> {
    copy_to_terminal(text);
    crate::hooks::clipboard::local_copy(text, || false)
}

/// The sequences requested since the last take, in order; the App writes
/// them to its terminal.
pub(crate) fn take_pending() -> Vec<Vec<u8>> {
    PENDING.with(|pending| std::mem::take(&mut *pending.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CLP-001: the sequence names the clipboard, carries the text in
    /// base64 and ends with ST.
    #[test]
    fn clp_001_the_osc_52_sequence_carries_the_text_in_base64() {
        assert_eq!(osc52("hello"), b"\x1b]52;c;aGVsbG8=\x1b\\".to_vec());
        assert_eq!(osc52(""), b"\x1b]52;c;\x1b\\".to_vec());
        assert_eq!(
            String::from_utf8(osc52("clipboard '界'\nsecond line")).unwrap(),
            "\x1b]52;c;Y2xpcGJvYXJkICfnlYwnCnNlY29uZCBsaW5l\x1b\\"
        );
    }

    /// CLP-001: a request waits for the App to take it; nothing is written
    /// without a copy.
    #[test]
    fn clp_001_a_requested_copy_is_pending_until_taken() {
        let _ = take_pending();
        assert!(take_pending().is_empty(), "nothing waits without a copy");
        copy_to_terminal("hello");
        copy_to_terminal("world");
        assert_eq!(take_pending(), vec![osc52("hello"), osc52("world")]);
        assert!(take_pending().is_empty(), "a take empties the queue");
    }
}
