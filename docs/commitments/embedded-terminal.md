# Commitment: embedded-terminal

Status: Agreed 2026-09-07
Requirements: EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

An optional libghostty-backed embedded session, a complete-cell frame adapter
for SuprTUI, additive App support for background updates and a configurable
quit chord, and an interactive shell example. A real PTY replaces the pipe
approach for this new path. The existing terminal implementation stays
available until its callers are migrated in a later commitment.

## Boundaries

This milestone covers one shell screen composed into an application frame,
keyboard input, resizing, output, and lifetime. It does not claim arbitrary
widget-tree embedding, native image protocols, mouse interaction, clipboard,
or complete browser CSS semantics. These remain later work. Existing legacy
test failures are recorded rather than silently included as completed fixes.

The reported counter-input failure remains open: it was not reproduced in
normal or controlling-PTY probes, and the developer deferred host-specific
investigation. The shell probes must exercise App input directly; their passes
must not be described as resolving that unobserved host failure.

Done-when: all named requirements have current passing evidence, controlled
violations establish that the new checks reject failures, and a review covers
process lifetime, error paths, buffering, and the native dependency boundary.
