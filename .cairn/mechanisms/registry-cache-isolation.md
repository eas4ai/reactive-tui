# Mechanism: registry-cache-isolation

command: sh scripts/check-registry-cache-isolation.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - reactive-tui-macros
  - src
  - scripts
  - tests
  - examples
  - docs/recon-evidence/cache-isolation
requirements:
  - ABI-004
  - CCH-001
  - CCH-002
