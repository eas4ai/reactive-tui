#!/bin/sh
set -eu
cargo test --locked --test registry_cache_isolation
sh scripts/check-registry-concurrency.sh
