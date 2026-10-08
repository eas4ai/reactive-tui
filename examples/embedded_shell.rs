//! Run with: cargo run --locked --features embedded-terminal --example embedded_shell
//! Ctrl+Q exits the host; Ctrl+C and Escape are forwarded to the child shell.

include!("../tests/runtime_probes/embedded_terminal.rs");
