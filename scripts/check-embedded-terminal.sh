#!/bin/sh
set -eu
printf '%s\n' 'Embedded terminal worker and keyboard tests'
cargo test --locked --features embedded-terminal --lib embedded:: -- --test-threads=1
printf '%s\n' 'Embedded terminal integration tests'
cargo test --locked --features embedded-terminal --test embedded_terminal -- --test-threads=1
printf '%s\n' 'Embedded terminal acceptance probe'
cargo build --locked --features embedded-terminal --example embedded_terminal_probe
python3 scripts/check-embedded-terminal-pty.py
printf '%s\n' 'Inherited renderer gate'
sh scripts/check-renderer.sh
