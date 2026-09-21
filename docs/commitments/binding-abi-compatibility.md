# Commitment: binding-abi-compatibility

Status: Agreed 2026-09-08
Requirements: ABI-001, ABI-002, ABI-003, ABI-004, MNT-001, MNT-002, MNT-003, MNT-004, DFT-001, DFT-002, DFT-003, DFT-004, CCH-001, CCH-002, REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Align shipped C and TypeScript declarations with the Rust FFI and verify actual
consumer interoperability. Cover the known terminal signature/symbol drift,
capabilities layout and legacy builder aliases, with a complete declaration audit.

## Boundaries

Preserve existing Rust exports and supported consumer behavior. Record judged
compatibility decisions before implementation. This is binding compatibility,
not a new widget catalog, renderer redesign or live clipboard repair. Do not run
known mismatched calls. Document changes to broken consumer declarations.

Done-when: all named requirements pass and final review records the full ABI
inventory, meaningful failure demonstrations, ownership and production self-audit.
