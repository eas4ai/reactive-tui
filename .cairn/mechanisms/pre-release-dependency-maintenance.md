# Mechanism: pre-release-dependency-maintenance

command: python3 -B scripts/check-pre-release-dependency-code-quality.py DQC-002
inputs:
  - .cairn/mechanisms/pre-release-dependency-maintenance.md
  - Cargo.toml
  - Cargo.lock
  - crates
  - src
  - deny.toml
  - scripts/check-pre-release-dependency-code-quality.py
  - scripts/test-pre-release-dependency-code-quality.py
  - docs/spec/pre-release-dependency-code-quality.md
  - docs/commitments/pre-release-dependency-code-quality.md
  - docs/decisions
requirements:
  - DQC-002

The check MUST first prove its result validator rejects successful-looking
fixtures with an unmaintained production package, an unapproved duplicate
`taffy` or `vte` line, an ambiguous maintained crossterm package identity, or
`onig_sys` in the default dependency graph.

The check MUST then inspect locked default, minimal, FFI, and Markdown feature
graphs with the supported Rust toolchain. It MUST fail on command failure,
incomplete graph output, any unexplained maintenance advisory, a duplicate
`taffy` or `vte` line without an active decision naming both versions and the
reason, any package named `crossterm` that resolves to the maintained fork, or
`onig_sys` in the default graph. The report MUST identify every inspected
feature graph and the audited lockfile.
