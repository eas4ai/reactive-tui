//! Part of tests/review_facade.rs: the C renderer and terminal functions
//! enable raw mode and enter the alternate screen, which the pipes of
//! `cargo test` refuse, so their tests run in a copy of this binary on a
//! pseudo-terminal.

/// Set in the copy of this binary that a test runs on a pseudo-terminal.
pub const ON_TERMINAL: &str = "REACTIVE_TUI_REVIEW_FACADE_ON_TERMINAL";

/// Runs `body` on a pseudo-terminal: in a copy of this binary that runs
/// `test` (its path as libtest prints it), or directly when this process
/// already is that copy. In the copy it returns `None`; otherwise it asserts
/// that the copy passed and returns everything the copy wrote to the
/// terminal, for the test to look at.
pub fn on_terminal(test: &str, body: impl FnOnce()) -> Option<String> {
    if std::env::var_os(ON_TERMINAL).is_some() {
        body();
        return None;
    }
    let run = crate::pty_support::run_test(test, &[(ON_TERMINAL, "1")]);
    let text = run.text();
    assert!(
        run.status.success() && text.contains("test result: ok. 1 passed"),
        "{test} failed on the pseudo-terminal (exit {:?}); it wrote:\n{text}",
        run.status.code()
    );
    Some(text)
}

/// `text` without its SGR sequences (`ESC [ ... m`), so cursor movements
/// and text can be read together.
pub fn without_sgr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("\x1b[") {
        let (head, tail) = rest.split_at(start);
        out.push_str(head);
        let body = &tail[2..];
        match body.find(|c: char| c.is_ascii_alphabetic()) {
            Some(end) if body.as_bytes()[end] == b'm' => rest = &body[end + 1..],
            Some(end) => {
                out.push_str(&tail[..2 + end + 1]);
                rest = &body[end + 1..];
            }
            None => {
                out.push_str(tail);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}
