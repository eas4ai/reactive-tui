# Disable automatic Windows GNU DLL exports while retaining explicit C exports

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-015 ABI-001
Would be wrong if: A Windows build still overflows its export table, or the FFI feature loses a declared C export.
History: Native platform recovery retains existing C ABI guarantees; no platform or feature is being removed.

## Decision

The native Windows default build fails with export ordinal too large: 134953. With FFI disabled the library has no intended C exports, and the GNU linker automatically exports internal dependency symbols. Add a target-specific cdylib linker argument disabling automatic export, retaining Rust-generated explicit exports with FFI enabled. Declare the build script as a check input and verify both native behavior and the FFI export table. GNU ld documents explicit exports and auto-export at https://sourceware.org/binutils/docs/ld/WIN32.html; Cargo scopes rustc-link-arg-cdylib to DLL output.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `build.rs`.

Behavior checks: `scripts/check-widget-platforms.py`, `docs/binding-abi-baseline.json`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
