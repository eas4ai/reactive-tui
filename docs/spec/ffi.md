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

Read on 2026-10-04 from the same review (findings N14, N15, N17, N19, N20
and N22) and checked against the code at 0aade996 on 2026-10-06: the C
text-buffer renderers paint one Unicode scalar per cell and advance one
column each (src/ffi/text.rs), so the letter after a wide glyph lands in
the glyph's continuation cell and a combining mark is split from its base;
the widget constructors of src/ffi/widgets.rs return a styled text element
where the Rust builders return the native text input, checkbox and progress
bar, so a C text input takes no typing and a C checkbox keeps no state; the
hit-grid and statistics functions of src/ffi/stats.rs record nothing
(`checkHit` answers 1 anywhere inside the renderer, `dumpBuffers` writes no
file); `renderSurfaceToTerminal` ignores the terminal it is given, builds a
second renderer with its own terminal session and restores the host
terminal when that renderer drops, and copies cells without their graphemes
and pictures; `rtui_effect_create` boxes a callback that only
`rtui_effect_run` ever calls, while the module calls itself a complete
reactive system; and `rtui_version` says 0.1.0 where Cargo.toml says 1.0.0,
README names a 0.1.0 candidate and Syntect, and the TypeScript package says
0.1.0.

## Observed

(none yet)

## Draft

[FFI-001] Every function the library exports to C that takes a pointer from its caller and uses it for anything but a null check MUST be an `unsafe extern "C" fn` on the Rust side whose documentation says in a `# Safety` section what the caller must guarantee (a live handle the library returned and has not destroyed, a buffer of at least the length given, a NUL-terminated string), so safe Rust cannot call it with an invalid pointer.
Falsifier: In src/ffi, a `pub extern "C" fn` that is not `unsafe` uses a raw-pointer parameter for anything but comparing it with null, in its body or in a closure in its body; or an `unsafe extern "C" fn` with a raw-pointer parameter has no `# Safety` section in its documentation.
Mechanism: review-high
Rationale: A safe Rust signature promises that any argument is sound, and a pointer-taking export cannot keep that promise (the developer's code review of 2026-10-04, N02).
Status: Agreed 2026-10-04

[FFI-002] Every C function that paints a text buffer onto a surface (`renderTextBufferToSurface`, `renderTextBufferToRenderer`, `renderTextBufferDirect`) MUST place complete grapheme clusters as the editor's painting does: each cluster in its own cells by its display width, a wide glyph keeping its continuation cell, a combining mark staying with its base and a joined sequence painted as one glyph, while the selection stays addressed by scalar index as the C API defines it; the `width_method` argument of `createTextBuffer` and `createOptimizedBuffer` MUST be documented in the header as accepted for compatibility and not changing the one width policy.
Falsifier: With the `ffi` feature, a text buffer holding `界A` painted onto a four-column surface leaves `A` in column 1, the wide glyph's continuation cell, instead of column 2; `e` followed by U+0301 takes two cells, or the surface's grapheme at its column is not the two-scalar cluster; or the header leaves `width_method` undescribed.
Mechanism: review-facade
Rationale: Both renderers wrote one scalar per `Surface::write_str` call and advanced one column each, where the editor paints whole graphemes by their width (the developer's code review of 2026-10-04, N14).
Status: Agreed 2026-10-06

[FFI-003] `rtui_text_input_create` MUST return the native text input the Rust `input` builder produces, with its placeholder and initial value; `rtui_checkbox_create` the native checkbox, with its label and checked state; and `rtui_progress_bar_create` the native progress bar, with its range, value and label, so that an App running them edits, toggles and shows them as it does the Rust-built ones; the header's documentation of these constructors and of `rtui_button_create` MUST say what each returns and that they take no callbacks.
Falsifier: With the `ffi` feature, `rtui_element_get_type` reports `Text` or `Layout` for the element one of the three constructors returns; run as the root of an App on the debug backend, a typed character does not appear in the focused text input, or Space does not toggle the checkbox, or the progress bar's frame shows no bar for its value; or the header has no documentation on one of the four constructors.
Mechanism: review-facade
Rationale: The constructors built a styled text element from a generic builder and never selected a native control, so a C text input accepted no editing and a C checkbox kept no toggle state (N15).
Status: Agreed 2026-10-06

[FFI-004] The C renderer's hit testing and host statistics MUST do what their names say: `addToHitGrid` MUST record its id over its region, clipped to the renderer, in the grid being built; a completed `render` or `renderWithStats` MUST make the built grid the one `checkHit` reads and start an empty one; `checkHit` MUST return the id recorded at (x, y) in that grid and 0 where none is recorded or outside the renderer; `dumpHitGrid` MUST log the recorded regions; `setRenderOffset` MUST shift the rows the renderer writes by the offset; `updateStats` and `updateMemoryStats` MUST keep the values given, which the debug overlay shows and `dumpBuffers` writes; and `dumpBuffers` MUST write the front and back surfaces' text and the kept statistics to a file named `rtui-buffers-<timestamp>.txt` in the current directory and log its path.
Falsifier: On a pseudo-terminal with the `ffi` feature: `checkHit` returns nonzero before any registration; after `addToHitGrid(1, 1, 1, 1, 42)` and a `render`, `checkHit(1, 1)` returns anything but 42 or `checkHit(0, 0)` anything but 0; with the debug overlay on, `updateStats` reporting 42 frames per second followed by a `render` leaves 42 out of the output; `setRenderOffset(2)` followed by a `render` writes the first row at terminal row 1; or `dumpBuffers(7)` leaves no `rtui-buffers-7.txt` holding the front surface's text.
Mechanism: review-facade
Rationale: `addToHitGrid` discarded every argument, `checkHit` answered 1 anywhere inside the renderer, and the offset, statistics and dump functions logged or ignored what they were given (N17).
Status: Agreed 2026-10-06

[FFI-005] `renderSurfaceToTerminal` MUST write the surface, with its graphemes and pictures, through the terminal handle it is given, and MUST NOT create another terminal session or restore the host terminal; `renderWithStats` MUST render the given renderer's frame through the renderer's own terminal, and the header MUST say that its terminal argument is validated and not written to.
Falsifier: On a pseudo-terminal with the `ffi` feature, after `createTerminal` and `setupTerminal`, a `renderSurfaceToTerminal` call writes the alternate-screen exit (`ESC [ ? 1049 l`) or leaves raw mode before the caller's own teardown, or its output lacks a wide glyph's continuation or a cell's full grapheme; or the header does not say what `renderWithStats` does with its terminal argument.
Mechanism: review-facade
Rationale: The function built a second `Renderer`, whose own `Terminal` entered modern mode and restored the host terminal on drop while the caller's handle was live, and copied cells without their grapheme map and pictures (N22).
Status: Agreed 2026-10-06

[FFI-006] An effect made with `rtui_effect_create` MUST run its callback when it is made and again after each change, through the C signal setters, of a C signal its last run read through the C signal getters, running its cleanup callback before each run after the first and at `rtui_effect_destroy`; `rtui_effect_run` MUST run it by hand as well; `rtui_use_signal_int`, `rtui_use_signal_string` and `rtui_use_signal_bool` on one `RTuiHooks` MUST hand out the same signal for the same key; and the module's and the header's documentation MUST describe the effects and the hooks as they are.
Falsifier: With the `ffi` feature, a signal from `rtui_signal_int_create` and an effect whose callback counts its runs and reads that signal: the count is 0 after `rtui_effect_create`, or does not grow after `rtui_signal_int_set` changes the value, or the cleanup has not run before the second run or after `rtui_effect_destroy`; or two `rtui_use_signal_int` calls with one key give signals whose values differ after one is set.
Mechanism: review-facade
Rationale: `rtui_effect_create` boxed its callbacks and subscribed to nothing, only `rtui_effect_run` ever ran the body, and the hooks' effects were never populated, under a header calling it a complete reactive system (N20).
Status: Agreed 2026-10-06

[FFI-007] The library MUST report one version: `rtui_version` MUST return the crate's package version from Cargo.toml with the ABI version as a separate constant, and README.md's release statement, the umbrella header include/reactive_tui.h and the TypeScript package bindings/typescript/package.json MUST name that version; README.md MUST name the syntax highlighter the crate uses.
Falsifier: `rtui_version`'s major, minor and patch differ from `CARGO_PKG_VERSION`; README.md names another version or Syntect as the highlighter; or the umbrella header's version line or the TypeScript package's version differs from Cargo.toml's.
Mechanism: review-facade
Rationale: Cargo.toml said 1.0.0 while `rtui_version`, the umbrella header, README and the TypeScript package said 0.1.0, and README named Syntect two releases after Lumis replaced it (N19).
Status: Agreed 2026-10-06
