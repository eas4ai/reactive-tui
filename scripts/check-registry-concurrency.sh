#!/bin/sh
set -eu
cargo test --locked --lib component::registry -- --test-threads=1
python3 scripts/check-registry-concurrency.py
cargo test --locked --test component_tests --test component_macro_test --test component_macro_integration
