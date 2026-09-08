#!/bin/sh
set -eu
cargo test --locked --test registry_concurrency
cargo test --locked --test simple_performance_test -- --test-threads=2
