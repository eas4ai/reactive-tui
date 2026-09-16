# Mechanism: pre-release-terminal-helper-lifecycle

command: python3 -B scripts/check-pre-release-terminal-helper-lifecycle.py TRL-004
inputs:
  - Cargo.toml
  - Cargo.lock
  - src
  - tests/pre_release_terminal_helper_lifecycle.rs
  - scripts/check-pre-release-terminal-helper-lifecycle.py
requirements:
  - TRL-004

The check MUST use `syn` to inventory terminal discovery and configuration
helpers and fail when a helper is added, removed, or returns to an unbounded
`Command::output`, `Command::status`, or `Command::spawn` path. The inventory
MUST include the `which` probes used for image helpers and the `stty size`
probe used for character dimensions.

On Unix, subprocess tests MUST prepend controlled `which` and `stty`
executables to `PATH`. Each executable MUST fork a descendant, record both
process identifiers, and then block. The public framework path that invokes
the helper MUST return its documented fallback or bounded error before a
fixed outer deadline. After it returns, the test MUST prove that both the
direct helper and its descendant are gone and that the direct helper was
reaped. The validator MUST first reject safe fixtures for an unbounded helper,
direct-child-only termination, and a skipped helper path. Static source
inspection alone cannot pass.
