# Use Koffi for the audited TypeScript native boundary

Level: Judged
Decided by: Codex
Rests on: ABI-001, ABI-003
Would be wrong if: The dependency cannot load the audited ABI on the supported runtime, changes working native behavior, or adds more maintenance than repairing the current loader.

## Decision

Replace ffi-napi and ref-napi with a pinned Koffi dependency. The current ffi-napi native addon fails to compile on the installed Node 24 runtime because its N-API callback types disagree with Node headers. Koffi provides typed struct, pointer, callback and output-parameter declarations needed by the existing Rust ABI. Verify every loaded signature before invoking native code and compare its layouts with compiler probes. Keep lifecycle operations synchronous and explicit. Do not patch away native compiler errors or pin an obsolete Node release to hide the failure. Official API references: https://koffi.dev/load, https://koffi.dev/output, https://koffi.dev/composites.

## Realized by

- d485469ce6722e9efb9068673a68e8ce4115c13c Align C and TypeScript bindings with the compiled Rust ABI
