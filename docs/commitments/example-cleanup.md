# Commitment: example-cleanup

Status: Agreed 2026-09-15
Requirements: EXC-001

## Deliverable

Remove the six examples the developer found broken in Kitty, retain the working
gradient example, and remove stale tracked references to the deleted programs.

## Boundaries

This commitment changes `examples/`, tracked documentation or manifest
references to the removed examples, and the Cairn mechanism for EXC-001. It
does not repair the deleted programs or build the widget catalog.

Done-when: EXC-001 passes and final review finds no tracked invitation to run a
removed example.
