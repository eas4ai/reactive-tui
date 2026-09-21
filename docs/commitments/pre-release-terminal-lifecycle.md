# Commitment: pre-release-terminal-lifecycle

Status: Agreed 2026-09-14
Requirements: TRL-001, TRL-002, TRL-003, TRL-004

## Deliverable

Restore the host terminal after panics, termination signals, and helper-process
failures. Filter control data before it reaches the host and accept clipboard
responses only for requests the framework made.

## Boundaries

This commitment changes application and backend shutdown, Unix signal and panic
handling, terminal string writers, input parsing, helper-process execution, and
focused lifecycle checks. It does not redesign widget behavior or package the
release.

Done-when: TRL-001 through TRL-004 pass; fault injection demonstrates terminal
restoration after worker and main-thread panics and process signals; final
review finds no unbounded named helper or unsolicited host control path.
