# C interface

Prefix: FFI

The library exports about 240 functions to C (src/ffi, built with the `ffi`
feature); the C headers under include/reactive_tui and the TypeScript
bindings under bindings/typescript call them. Read on 2026-10-04 from the
developer's code review of 65e618ec (finding N02): 118 of those functions
are safe `extern "C" fn` on the Rust side although they read or write
through a pointer their caller passes, for example `rtui_signal_int_set`
(src/ffi/reactive.rs) turns its handle into a reference after a null check
alone, and `rtui_signal_string_get` writes into a caller's buffer. Safe Rust
can call them with any address. The other 115 are already `unsafe`. Clippy's
`not_unsafe_ptr_arg_deref` lint does not see these, because each
dereference sits inside the closure the function hands to `catch_panic`.

## Observed

(none yet)

## Draft

[FFI-001] Every function the library exports to C that takes a pointer from its caller and uses it for anything but a null check MUST be an `unsafe extern "C" fn` on the Rust side whose documentation says in a `# Safety` section what the caller must guarantee (a live handle the library returned and has not destroyed, a buffer of at least the length given, a NUL-terminated string), so safe Rust cannot call it with an invalid pointer.
Falsifier: In src/ffi, a `pub extern "C" fn` that is not `unsafe` uses a raw-pointer parameter for anything but comparing it with null, in its body or in a closure in its body; or an `unsafe extern "C" fn` with a raw-pointer parameter has no `# Safety` section in its documentation.
Mechanism: review-high
Rationale: A safe Rust signature promises that any argument is sound, and a pointer-taking export cannot keep that promise (the developer's code review of 2026-10-04, N02).
Status: Agreed 2026-10-04
