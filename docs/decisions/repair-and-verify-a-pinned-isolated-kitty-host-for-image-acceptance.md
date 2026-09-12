# Repair and verify a pinned isolated Kitty host for image acceptance

Level: Judged
Decided by: Shawn and Codex
Rests on: API-014 API-020
Would be wrong if: The prepared placement counts still do not select the correct image paint path, or a patched host result is represented as repairing users of an unmodified Kitty installation.
History: The developer approved escalation api-014-api-020 on 2026-09-12. This extends the commitment to a narrow host repair; previous image API, ownership, protocol and iTerm2 decisions remain in force.

## Decision

Pin Kitty 0.45.0 source by archive digest and retain a small patch that evaluates a normal window's image-layer requirement after preparing its graphics data. Build it in an isolated directory with at most 12 compiler jobs and retain build provenance and runtime hashes. Use that verified build for the existing Linux image acceptance routes, without repaint requests, retransmission, relaxed pixel checks or longer timeouts. Preserve the system host and the independent failing controls. Add a real independent-sender regression to the host validation and retain corrected source-update, placement and removal captures. Document the required fixed host build and make clear that this does not update users' stock Kitty installations.

## Realized by

`scripts/kitty-host` pins, patches, builds and verifies the isolated host.
`scripts/check-api-images.py` selects it and runs the independent sender through
the existing capture assertions. Build records and stock/repaired captures are
retained in `.cairn/reviews/api-014-kitty-repair`. Formal API-014 acceptance still
requires a fresh committed-tree receipt, including current native evidence.
