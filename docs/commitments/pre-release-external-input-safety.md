# Commitment: pre-release-external-input-safety

Status: Agreed 2026-09-14
Requirements: XIS-001, XIS-002, XIS-003

## Deliverable

Make dialog network behavior explicit and bounded, cap complete image decode
memory, pass external-renderer paths safely, and close file-operation identity
races inside the explorer root.

## Boundaries

This commitment changes dialog HTTP helpers, image decoding and external
renderers, file-explorer mutations, user-facing manual pages, focused tests, and
their Cairn mechanisms. It does not broaden supported network protocols or
filesystem authority.

Done-when: XIS-001 through XIS-003 pass; fake-process, allocation-boundary, and
entry-replacement tests reject the audited cases; final review confirms all
network, memory, argument, and file-identity limits are documented.
