#!/bin/sh
set -eu
printf '%s\n' 'Internal renderer tests'
cargo test --locked --manifest-path src/backend/engine/Cargo.toml --target-dir target/renderer
printf '%s\n' 'Renderer integration tests'
cargo test --locked --test suprtui_renderer
printf '%s\n' 'Renderer example build'
cargo build --locked --example suprtui_counter
printf '%s\n' 'Renderer pseudo-terminal checks'
python3 scripts/check-renderer-pty.py
printf '%s\n' 'Specification lint'
node scripts/spec-lint.mjs docs/spec
