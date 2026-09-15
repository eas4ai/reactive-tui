# Mechanism: pre-release-library-diagnostics

command: python3 -B scripts/check-pre-release-library-diagnostics.py DQC-003
inputs:
  - .cairn/mechanisms/pre-release-library-diagnostics.md
  - Cargo.toml
  - Cargo.lock
  - src
  - tests
  - scripts/check-pre-release-library-diagnostics.py
  - scripts/test-pre-release-library-diagnostics.py
  - docs/spec/pre-release-dependency-code-quality.md
  - docs/commitments/pre-release-dependency-code-quality.md
requirements:
  - DQC-003

The check MUST first prove its result validator rejects a successful-looking
fixture with a raw stdout or stderr macro in non-test library code, a failed or
empty captured-output probe, or a missing parser, terminal, reconciliation,
focus, or window diagnostic path.

The check MUST then audit non-test library syntax for `print!`, `println!`,
`eprint!`, and `eprintln!` and run a captured-output integration probe with the
supported Rust toolchain. The probe MUST exercise parser, terminal,
reconciliation, focus, and window diagnostics, MUST fail if any command fails
or any required path is skipped, and MUST fail if an internal diagnostic is
written to process stdout or stderr. The report MUST identify every audited
source file and exercised diagnostic path.
