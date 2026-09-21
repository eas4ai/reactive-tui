# Commitment: documentation-retention

Status: Agreed 2026-09-14
Requirements: DOC-001

## Deliverable

Remove tracked documentation outside the Cairn-managed specification,
commitment, and decision directories. Keep the broader `docs` path ignored and
preserve `.cairn` records.

## Boundaries

This commitment changes repository retention only. It does not change Rust,
native bindings, examples, or runtime behavior. Removed documents remain in Git
history at `5c069365`.

Done-when: DOC-001 passes, its mechanism rejects a tracked legacy-document
fixture, and final review confirms no Cairn artifact or production source was
removed.
