# Mechanism: pre-release-dependency-security

command: python3 -B scripts/check-pre-release-dependency-code-quality.py DQC-001
inputs:
  - .cairn/mechanisms/pre-release-dependency-security.md
  - Cargo.toml
  - Cargo.lock
  - crates
  - deny.toml
  - scripts/check-pre-release-dependency-code-quality.py
  - docs/spec/pre-release-dependency-code-quality.md
  - docs/commitments/pre-release-dependency-code-quality.md
  - docs/decisions
requirements:
  - DQC-001

The check MUST first prove its result validator rejects a successful-looking
fixture that omits an advisory, license, source, or duplicate-policy result;
reports a vulnerable or unsound package without an exact reviewed exception;
leaves `atty` reachable; or accepts an exception without a matching decision
that names the exposure and mitigation.

The check MUST then run locked cargo-audit and cargo-deny checks against the
production feature graph. The checked-in deny policy MUST enforce advisories,
licenses, sources, and allowed duplicates. Every advisory exception MUST match
an active decision record and the resolved package version exactly. The check
MUST fail on command failure, incomplete output, an undeclared exception, or a
reachable `atty` package, and MUST report the audited lockfile and policy.
