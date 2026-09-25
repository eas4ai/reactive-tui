#!/bin/sh
set -eu
cargo test --locked --lib reactive::wake -- --test-threads=1
cargo test --locked --lib reactive::scheduler -- --test-threads=1
cargo test --locked --test app_wakeups -- --test-threads=1
cargo build --locked --example app_wakeup_probe
python3 scripts/check-app-wakeups-pty.py
sh scripts/check-embedded-terminal.sh
