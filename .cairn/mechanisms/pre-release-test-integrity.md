# Mechanism: pre-release-test-integrity

command: python3 -B scripts/check-pre-release-test-integrity.py DQC-004
inputs:
  - .cairn/mechanisms/pre-release-test-integrity.md
  - Cargo.toml
  - Cargo.lock
  - src
  - tests
  - scripts/check-pre-release-test-integrity.py
  - scripts/check-pre-release-library-diagnostics.py
  - scripts/dependency_check_test_support.py
  - scripts/test-pre-release-test-integrity.py
  - docs/spec/pre-release-dependency-code-quality.md
  - docs/commitments/pre-release-dependency-code-quality.md
requirements:
  - DQC-004

The check MUST first prove its result validator rejects successful-looking
fixtures containing an unreferenced FFI export module, an unsafe global test
hook, a Rust integration source that is absent from the compiled target graph,
or a readiness test with no observable assertion. The validator tests MUST
also prove that a sampled assertion mutation is rejected.

The check MUST then enumerate Cargo integration targets and compiled test
artifacts on the supported Rust toolchain for the maintained default and FFI
feature sets. It MUST trace every Rust source below `tests/` either to a Cargo
test target or to a module reachable from one, and MUST fail if an intended
test source is disconnected or its target does not compile.

The check MUST inventory Rust modules and exported C functions below `src/ffi`
and prove each shipped export module is reachable from the maintained FFI
module tree. It MUST audit shipped code for mutable global or raw-pointer test
hooks that are not gated from production. For sampled readiness tests, it MUST
run the original assertion and a safe isolated mutant, and MUST require the
original to pass and the mutant to fail. The report MUST name every compiled
test target, reachable FFI export module, audited hook, and mutation result.
