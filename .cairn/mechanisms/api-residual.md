# Mechanism: api-residual

command: python3 -B scripts/check-api-residual.py
inputs:
  - .cairn/mechanisms
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - tests
  - examples
  - benches
  - README.md
  - docs
  - include
  - bindings/typescript
  - scripts
  - verification
  - .github/workflows/clipboard-platforms.yml
requirements:
  - API-019

The inventory preserves each concern's contract and falsifier before repair.
Acceptance requires an executed behavioral check for every API-019 concern;
missing coverage is a failure. Individual development selections cannot pass
the complete requirement. API-020 owns image-capture timeout cleanup.

The declaration begins with reference persistence/reentry and actual Rust test
discovery. The remaining focused checks must be built and demonstrated before
this mechanism can pass. Source compilation, inventory presence and historical
defect-confirming tests do not establish runtime correctness.
