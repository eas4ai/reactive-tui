# Commitment: rust-api-remediation

Status: Agreed 2026-09-08
Requirements: API-001, API-002, API-003, API-004, API-005, API-006, API-007, API-008, API-009, API-010, API-011, API-012, API-013, API-014, API-015, API-016, API-017, API-018, API-019, API-020, ABI-001, ABI-002, ABI-003, ABI-004, MNT-001, MNT-002, MNT-003, MNT-004, DFT-001, DFT-002, DFT-003, DFT-004, CCH-001, CCH-002, REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Remediate the complete Rust API audit in docs/api-audit.md. Deliver functional
native component, input and feature paths, then audited foreign-language access.
All 15 findings and the audit coverage-table concerns are mapped in
`docs/spec/rust-api-remediation.md`; all 34 prior requirements remain inherited.

## Execution order

1. Signal ownership (API-001).
2. Component expansion, hooks and runtime (API-002 through API-004).
3. Event routing and focus (API-005 and API-006).
4. Unicode editor, clipboard and feature configurations (API-007, API-008, API-015).
5. Styling, widgets, dialogs, animation, screens and images (API-009 through API-014).
6. Entry-point integration and native consumers (API-016 and API-017).
7. Documentation, residual concerns and complete regression review (API-018 through API-020).

Inventory work begins before implementation and is maintained throughout. Each
requirement gets its own meaningful mechanism and failure demonstration; do not
use one broad success-only command as proof of the whole catalog.

## Boundaries

Preserve the SuprTUI renderer, embedded-terminal direction, working Rust APIs and
C ABI. Existing decision records remain valid within their original scopes; this
commitment now includes behavior previously deferred by the ABI-only commitment.
Record new architecture and ownership decisions before building them. Escalate
contradictions with existing contracts rather than silently superseding them.

No feature removal, silent migration, successful no-op replacement, or documentation
relabeling may substitute for repair. Select image URL behavior and protocol/platform
claims explicitly. Completing an advertised behavior is in scope; importing the
old editor-rs application, adding its unrelated IDE/AI/Git features, or rewriting
the renderer is not. The editor recovery observations may inform implementation
choices but do not independently expand this commitment.

The developer approved the narrow Kitty image-layer ordering repair on
2026-09-12 (escalation api-014-api-020). API-014 includes its isolated pinned
host build, independent failure demonstration and unchanged image acceptance.

Done-when: all 54 requirements have current passing evidence and the final review
reconciles every finding and subcase, tests failures and corrected behavior, checks
ownership and integration, and records the production self-audit.
