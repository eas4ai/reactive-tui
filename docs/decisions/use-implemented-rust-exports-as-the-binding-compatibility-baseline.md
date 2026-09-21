# Use implemented Rust exports as the binding compatibility baseline

Level: Judged
Decided by: Codex
Rests on: ABI-001, ABI-002, ABI-003
Would be wrong if: A working Rust export is broken, a missing declaration with an existing behavioral mapping is needlessly retired, or unimplemented APIs are hidden without an explicit migration inventory.

## Decision

Keep the compiled Rust C ABI authoritative. Correct incompatible consumer declarations and preserve aliases that can delegate to implemented behavior. Record each unimplemented declaration and its replacement or unsupported status in migration guidance rather than adding successful no-op native stubs. TypeScript must load only audited exports; supported terminal, surface, renderer and element-builder paths get real interoperability tests. Static signature and layout checks must compare independent Rust and C compiler results, not just matching names. This follows the confirmed binding-repair scope and the recommended policy presented to the developer; a different developer choice supersedes it.

## Realized by

- d485469ce6722e9efb9068673a68e8ce4115c13c Align C and TypeScript bindings with the compiled Rust ABI
