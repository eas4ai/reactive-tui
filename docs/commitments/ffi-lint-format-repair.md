# Commitment: ffi-lint-format-repair

Status: Agreed 2026-09-08
Requirements: MNT-001, MNT-002, MNT-003, MNT-004, DFT-001, DFT-002, DFT-003, DFT-004, CCH-001, CCH-002, REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Repair the recorded formatting, strict default-feature Clippy and FFI compile/link failures. Preserve published signatures and verify repaired FFI entry points. All earlier acceptance remains inherited.

## Boundaries

No TypeScript redesign, new FFI product API, GPU renderer, host-input repair or blanket lint suppression. Formatting includes workspace Rust packages; Clippy uses the recorded default-feature all-targets command. Behavioral warnings require review and focused regressions.

Done-when: every named requirement passes and final review records failure demonstrations, ABI compatibility and production self-audit.
