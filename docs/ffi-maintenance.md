# FFI maintenance verification

The maintenance gate compiles all ffi-feature test targets, links a C fixture
against the shared library using the shipped terminal/event headers, and runs
six Rust FFI integration targets plus FFI unit tests in an 80x24 PTY.
Each PTY run must exit successfully and restore its original terminal settings.

The restored terminal functions are create, destroy, get_dimensions, sync and
poll_event with their existing C declarations. Creation does not enter raw mode.
Size errors are reported rather than inventing fallback dimensions. Poll timeout
zero is nonblocking. The event header documents the representable key, mouse and
resize payloads; unsupported events return NotSupported without writing output.

Renderer shutdown restores terminal state while retaining the handle allocation
until destroy. Renderer-owned surfaces are registered as borrowed and cannot be
freed by the caller-owned surface destructor. Borrowed access ends at renderer
shutdown or destruction. Callers must serialize handle access and destruction;
these raw-pointer APIs do not promise safety for concurrent use or stale pointer
reuse after a different allocation takes the same address.

App-builder setters keep their allocation alive while replacing the stored Rust
builder value. Root callbacks transfer an FFIElement wrapper to the application.

This does not certify the complete TypeScript or modern C header API. Their
existing signature and layout drift is captured in the Cairn backlog. The C
smoke fixture calls only the audited terminal/event declarations.
