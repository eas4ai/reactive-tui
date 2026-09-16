# Mechanism: pre-release-ci

command: python3 -B scripts/check-pre-release-ci.py RID-001
inputs:
  - .cairn/mechanisms/pre-release-ci.md
  - .github/workflows
  - scripts/check-pre-release-ci.py
  - scripts/test-pre-release-ci.py
  - scripts/requirements-ci.txt
  - scripts/dependency_check_test_support.py
  - scripts/check-pre-release-dependency-maintenance.py
  - scripts/check-pre-release-dependency-code-quality.py
  - scripts/test-pre-release-dependency-maintenance.py
  - scripts/test-pre-release-dependency-code-quality.py
  - Cargo.toml
  - Cargo.lock
  - dependency-maintenance.toml
  - deny.toml
  - docs/spec/pre-release-ci-documentation.md
  - docs/commitments/pre-release-ci-documentation.md
requirements:
  - RID-001

The check MUST first reject workflow fixtures that omit a platform, required
command, main-branch event, or advisory schedule. It MUST reject path filters,
conditional required steps, unlocked or success-masked commands, allowed
failures, floating action references, and write permissions. It MUST inspect
the tracked workflow and demonstrate main push, pull-request, manual, and
scheduled job selection. Native CI results and required-status enforcement
remain separate release-review obligations; inspection does not prove them.
