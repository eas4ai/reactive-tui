# Mechanism: example-cleanup

command: python3 -B scripts/check-example-cleanup.py
inputs:
  - .cairn/mechanisms/example-cleanup.md
  - Cargo.toml
  - examples
  - README.md
  - manual
  - docs
  - scripts/check-example-cleanup.py
  - scripts/test-example-cleanup.py
requirements:
  - EXC-001

The check MUST first prove its validator rejects successful-looking fixtures
that retain a removed example, omit `gradient_blocks.rs`, retain a tracked
invitation to run a removed example, skip the locked compile, or report a
failed compile as passing.

The check MUST then require the exact existing Rust example inventory to be
`gradient_blocks.rs`, scan tracked manifests and documentation for every
removed example name, and run
`cargo +1.91.0 check --locked --example gradient_blocks`. It MUST fail on an
incomplete inventory, any stale tracked reference, command failure, or
incomplete output, and MUST report the inventory and compile result.
