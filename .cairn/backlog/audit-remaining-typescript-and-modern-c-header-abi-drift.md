# Audit remaining TypeScript and modern C header ABI drift

Surfaced from: MNT-003
Captured: 2026-09-08T15:26:45.236Z

The current maintenance commitment repairs Rust FFI target compilation and selected terminal/renderer lifecycle paths. Source inspection also found that bindings/typescript/src/ffi.ts declares rtui_terminal_create with width and height whereas include/reactive_tui/terminal.h declares only the output pointer. The TypeScript loader names init/shutdown/get_size exports absent from the implementation. The modern C RTuiCapabilities field layout also differs from Rust Capabilities, and the older builder header declares aliases absent from the Rust exports. Audit and reconcile the complete binding/header ABI in a separate commitment before claiming those consumers supported; do not execute the mismatched calls.
