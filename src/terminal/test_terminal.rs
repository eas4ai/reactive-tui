//! Runs a unit test with a controlling terminal of its own.
//!
//! A test that needs a terminal runs again in a copy of the test binary on a
//! new pseudo-terminal. It then runs the same way in a developer's shell, in
//! CI and in an agent shell that has no terminal, and it never writes escape
//! sequences into the terminal that started `cargo test`.

/// The terminal the copy runs on is this many columns wide.
pub(crate) const COLUMNS: u16 = 80;
/// The terminal the copy runs on is this many rows high.
pub(crate) const ROWS: u16 = 24;
/// What the pseudo-terminal answers to a cursor position query (`CSI 6 n`).
pub(crate) const CURSOR_REPORT: &[u8] = b"\x1b[1;1R";

/// Runs `body` with a controlling terminal. `test` is the test's path as
/// libtest prints it, such as `core::renderer::tests::test_clear_surface`.
///
/// On Unix the test binary runs `test` alone on a new pseudo-terminal of
/// [`COLUMNS`] by [`ROWS`] that is the copy's controlling terminal, and this
/// call panics with the copy's output unless it passed. The pseudo-terminal
/// answers each cursor position query with [`CURSOR_REPORT`]. Elsewhere
/// `body` runs in this process.
#[cfg(unix)]
pub(crate) fn on_terminal(test: &str, body: impl FnOnce()) {
    if std::env::var_os(unix::ON_PTY).is_some_and(|name| name == test) {
        body()
    } else {
        unix::run_on_pty(test)
    }
}

#[cfg(not(unix))]
pub(crate) fn on_terminal(_test: &str, body: impl FnOnce()) {
    body()
}

#[cfg(unix)]
mod unix {
    use super::{COLUMNS, CURSOR_REPORT, ROWS};
    use crate::terminal::owned_pty::PtyChild;
    use std::io::{ErrorKind, Read, Write};
    use std::process::Command;
    use std::time::{Duration, Instant};

    /// Names the test whose body the copy on the pseudo-terminal runs.
    pub(super) const ON_PTY: &str = "REACTIVE_TUI_TEST_ON_PTY";
    const CURSOR_QUERY: &[u8] = b"\x1b[6n";
    // A copy starts in milliseconds; the deadline only bounds a hung one,
    // with room for a host that is running the whole workspace's tests.
    const DEADLINE: Duration = Duration::from_secs(120);

    pub(super) fn run_on_pty(test: &str) {
        let mut command = Command::new(std::env::current_exe().expect("the test binary's path"));
        command
            .args([
                test,
                "--exact",
                "--nocapture",
                "--test-threads=1",
                "--color=never",
            ])
            .env(ON_PTY, test);
        let mut child = PtyChild::spawn(command, COLUMNS, ROWS).expect("a pseudo-terminal");
        let deadline = Instant::now() + DEADLINE;
        let mut output = Vec::new();
        let mut answered = 0;
        let status = loop {
            let exited = child.try_wait().expect("the copy's status");
            // Read after the status: output the copy wrote before it exited
            // is still in the pseudo-terminal's buffer.
            drain(&mut child, &mut output);
            let queries = output
                .windows(CURSOR_QUERY.len())
                .filter(|window| *window == CURSOR_QUERY)
                .count();
            while answered < queries {
                child
                    .master
                    .write_all(CURSOR_REPORT)
                    .expect("an answer to the cursor query");
                answered += 1;
            }
            if let Some(status) = exited {
                break status;
            }
            if Instant::now() > deadline {
                let status = child.stop();
                panic!(
                    "{test} did not finish on a pseudo-terminal in {DEADLINE:?} ({status:?}):\n{}",
                    String::from_utf8_lossy(&output)
                );
            }
            child.wait_ready(false).expect("the pseudo-terminal");
        };
        let text = String::from_utf8_lossy(&output);
        assert!(
            status.success() && text.contains("test result: ok. 1 passed"),
            "{test} failed on a pseudo-terminal ({status}):\n{text}"
        );
    }

    fn drain(child: &mut PtyChild, output: &mut Vec<u8>) {
        let mut buffer = [0; 4096];
        loop {
            match child.master.read(&mut buffer) {
                Ok(0) => return,
                Ok(read) => output.extend_from_slice(&buffer[..read]),
                // Nothing buffered yet; or, on Linux, every slave descriptor
                // is closed because the copy exited.
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock
                        || error.raw_os_error() == Some(libc::EIO) =>
                {
                    return
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("reading the pseudo-terminal failed: {error}"),
            }
        }
    }
}
