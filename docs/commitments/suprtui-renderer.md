# Commitment: suprtui-renderer

Status: Agreed 2026-09-07
Requirements: RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

A pinned SuprTUI dependency and an additive backend usable through App, plus
an interactive counter example. Layout keeps the current Taffy/CSS pipeline;
the SuprTUI path paints grapheme cells directly instead of passing through the
legacy single-char Surface. App hands complete frames to this backend and
provides a default root input callback. Existing Backend implementations keep
their contract and behavior.

## Boundaries

Product edits belong to the backend, App integration, layout paint adapter,
example, dependency manifests, and their tests. The complete src tree is a
mechanism input because CSS/layout and shared error/event types affect output;
that input footprint does not authorize unrelated legacy repairs. Initial
project-wide failures remain reported in recon. No changes to the sibling
SuprTUI checkout and no embedded-terminal migration are needed for this unit.

## Records and proof

- Requirements: `docs/spec/rendering.md`.
- Mechanism: `.cairn/mechanisms/suprtui-renderer.md`.
- Tests: `tests/suprtui_renderer.rs`, `scripts/check-renderer-pty.py`.
- Runner: `scripts/check-renderer.sh`.
- Review: `.cairn/reviews/suprtui-renderer.md`.
- Evidence receipts and logs: `.cairn/evidence/`.

Done-when: all six requirements pass the mechanism, controlled failure examples
establish that it rejects violations, and a review covers what the checks miss.
The host compatibility proof is a pseudo-terminal exercising the actual ANSI
stream and termios. Real Kitty, GNOME Terminal, and Ghostty visual checks remain
explicitly unverified unless those hosts are available for testing.
