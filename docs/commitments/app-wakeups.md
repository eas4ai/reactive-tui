# Commitment: app-wakeups

Status: Agreed 2026-09-07
Requirements: WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

A shared App wake handle, render-observed signal notifications, scheduler
work and deadline integration, wakeable host waiting, and embedded-session
publication notifications. Reuse the existing input decoder where practical;
WezTerm supplies a reference for the wake pattern, not a required dependency.

## Boundaries

Retain the existing renderer, Ghostty interpreter and default quit behavior.
Preserve a polling compatibility path for existing RootComponent and Backend
implementations. Keep timers and frame pacing responsive under wake bursts.
This does not repair the separate registry stall, developer-host counter
report, CSS/animation semantics, FFI, or remaining legacy failures. It does
not adopt portable-pty, replace input protocols, or add terminal pane layout.

Done-when: all named requirements have current passing evidence, controlled
violations fail the new checks, and a review covers lost wake races, idle
behavior, timer reentrancy, subscription ownership and shutdown.
