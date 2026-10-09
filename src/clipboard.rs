//! The terminal's clipboard (docs/spec/clipboard.md, CLP-001).
//!
//! A copy made inside an application reaches the user's clipboard through
//! the terminal: the App writes `ESC ] 52 ; c ; <base64> ESC \`, the OSC 52
//! sequence, to its terminal before its next frame, so a copy over SSH lands
//! on the user's machine. The clipboard hook's writer and a text input's copy
//! and cut ask for it here; the App drains the requests on each turn of its
//! loop and before each frame, and hands them to its backend. A request made
//! on the thread that runs an App is that App's; one made on any other
//! thread waits for whichever App drains next. When a local clipboard
//! command is available (wl-copy, xsel, xclip, pbcopy or PowerShell) the
//! copy runs it as well, so it keeps working in a terminal that ignores
//! OSC 52, and a paste reads it ([`crate::clipboard::paste_local`]); the terminal's bracketed
//! paste arrives as an event either way.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use base64::Engine as _;

static LOCAL_COMMANDS: AtomicBool = AtomicBool::new(true);

/// Whether a copy runs the local clipboard command beside the terminal's
/// sequence. On by default (CLP-001); an application turns it off with
/// [`set_local_commands`], for example a test that must not touch the
/// developer's clipboard.
pub fn local_commands() -> bool {
    LOCAL_COMMANDS.load(Ordering::Relaxed)
}

/// Turns the local clipboard command on or off for every copy in this
/// process; the terminal's sequence is written either way.
pub fn set_local_commands(enabled: bool) {
    LOCAL_COMMANDS.store(enabled, Ordering::Relaxed);
}

thread_local! {
    // The queue of the thread that runs an App: a copy requested there, by
    // its components, hooks and callbacks, is that App's alone, so two Apps
    // on two threads, as a test binary runs them, never take each other's.
    static PENDING: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
    // Whether an App's loop runs on this thread ([`enter_app_thread`]).
    static RUNS_APP: Cell<bool> = const { Cell::new(false) };
}

// The queue of every thread that runs no App: a copy requested from a
// background thread, a timer or a task elsewhere waits here for whichever
// App drains next, the one App a terminal application has.
static ELSEWHERE: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

/// Marks the current thread as one that runs an App's loop, until the
/// returned guard drops: the App's own requests queue on this thread and
/// the App drains them with the process-wide ones.
pub(crate) fn enter_app_thread() -> AppThread {
    RUNS_APP.with(|runs| runs.set(true));
    AppThread(())
}

/// The guard of [`enter_app_thread`].
pub(crate) struct AppThread(());

impl Drop for AppThread {
    fn drop(&mut self) {
        RUNS_APP.with(|runs| runs.set(false));
    }
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
/// sequence is written to the terminal before the App's next frame. On the
/// thread that runs the App, where components, hooks and callbacks run, the
/// request is that App's; from any other thread, a background task or a
/// timer, it waits for whichever App drains next.
pub fn copy_to_terminal(text: &str) {
    let sequence = osc52(text);
    if RUNS_APP.with(Cell::get) {
        PENDING.with(|pending| pending.borrow_mut().push(sequence));
    } else {
        ELSEWHERE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(sequence);
    }
}

/// Puts `text` on the clipboard every way this process can: through the
/// terminal (OSC 52), asked for on this thread, and, when a local clipboard
/// command is available, through that command as well, run on a thread of
/// its own so the App's thread never waits for it (a clipboard tool that
/// hands the text to a daemon can hold its pipes for seconds). A failure of
/// the command is logged, not returned: the terminal has the text.
pub fn copy(text: &str) {
    copy_to_terminal(text);
    if !local_commands() {
        return;
    }
    let text = text.to_owned();
    let spawned = std::thread::Builder::new()
        .name("reactive-tui clipboard".into())
        .spawn(move || {
            if let Err(error) = crate::hooks::clipboard::local_copy(&text, || false) {
                log::debug!("Local clipboard copy failed: {error}");
            }
        });
    if let Err(error) = spawned {
        log::debug!("Local clipboard copy could not start: {error}");
    }
}

/// The clipboard's text through the local paste command, when the local
/// commands are on and one is available (CLP-001); `None` otherwise, so the
/// caller keeps its own copy. The command is given two seconds, and a failure
/// is logged, not returned.
pub fn paste_local() -> Option<String> {
    if !local_commands() {
        return None;
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    match crate::hooks::clipboard::local_paste(|| std::time::Instant::now() > deadline) {
        Ok(text) => text,
        Err(error) => {
            log::debug!("Local clipboard paste failed: {error}");
            None
        }
    }
}

/// The sequences requested since the last take, this thread's first and
/// then every other thread's, in order; the App writes them to its terminal.
pub(crate) fn take_pending() -> Vec<Vec<u8>> {
    let mut taken = PENDING.with(|pending| std::mem::take(&mut *pending.borrow_mut()));
    taken.append(&mut ELSEWHERE.lock().unwrap_or_else(|e| e.into_inner()));
    taken
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
    /// without a copy. A request from a thread that runs no App reaches the
    /// App too; a request on an App's thread stays that App's.
    #[test]
    #[serial_test::serial(clipboard_requests)]
    fn clp_001_a_requested_copy_is_pending_until_taken() {
        // Other tests of this binary may copy at the same time into the
        // process-wide part of the queue, so the requests made here are
        // looked for by content, and their absence after a take is what is
        // checked, not an empty queue.
        let _app = enter_app_thread();
        let _ = take_pending();
        copy_to_terminal("hello");
        std::thread::spawn(|| copy_to_terminal("world"))
            .join()
            .unwrap();
        let taken = take_pending();
        let hello = taken.iter().position(|s| *s == osc52("hello"));
        let world = taken.iter().position(|s| *s == osc52("world"));
        assert!(
            hello.is_some() && world.is_some() && hello < world,
            "this App's request first, then the background thread's: {taken:?}"
        );
        let again = take_pending();
        assert!(
            !again.contains(&osc52("hello")) && !again.contains(&osc52("world")),
            "a take empties both queues of what was taken"
        );
        let other = std::thread::spawn(|| {
            let _app = enter_app_thread();
            copy_to_terminal("mine");
            take_pending()
        })
        .join()
        .unwrap();
        assert!(
            other.contains(&osc52("mine")),
            "another App's thread drains its own"
        );
        assert!(
            !take_pending().contains(&osc52("mine")),
            "and leaves nothing of its own for this one"
        );
    }
}
