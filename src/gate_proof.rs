//! Proof for the workspace gates (BAR-001) that `cargo test` ran this
//! crate's unit tests. The gate sets `REACTIVE_TUI_GATE_PROOF` to a file
//! path and `REACTIVE_TUI_GATE_NONCE` to a value it chose, then reads the
//! file back after the run. A runner that prints test results without
//! running the test binary leaves the file absent.

#[test]
fn bar_001_the_workspace_gates_see_this_test_run() {
    let path = std::env::var_os("REACTIVE_TUI_GATE_PROOF");
    let nonce = std::env::var("REACTIVE_TUI_GATE_NONCE").ok();
    assert_eq!(
        path.is_some(),
        nonce.is_some(),
        "the gate sets both variables or neither"
    );
    if let (Some(path), Some(nonce)) = (path, nonce) {
        std::fs::write(&path, &nonce).expect("the gate's proof file is writable");
        let written = std::fs::read_to_string(&path).expect("the proof file reads back");
        assert_eq!(written, nonce);
    }
}
