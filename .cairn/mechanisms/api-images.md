# Mechanism: api-images

command: python3 -B scripts/check-api-images.py
inputs:
  - .cairn/mechanisms/api-images.md
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - tests
  - scripts/check-api-images.py
  - scripts/check-widget-platforms.py
  - scripts/check-dialog-http.py
  - scripts/check-iterm-host.py
  - scripts/install-conpty-runtime.py
  - .github/workflows/clipboard-platforms.yml
  - docs/analysis/widget-platforms
  - docs/analysis/api-image-hosts
  - docs/spec/rust-api-remediation.md
  - docs/widget-acceptance.md
  - docs/image-acceptance.md
requirements:
  - API-014
