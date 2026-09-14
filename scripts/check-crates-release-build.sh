#!/usr/bin/env bash
set -euo pipefail

jobs="${CARGO_BUILD_JOBS:-8}"
case "$jobs" in
    ''|*[!0-9]*) echo "CARGO_BUILD_JOBS must be an integer from 1 through 8" >&2; exit 1 ;;
esac
if (( jobs < 1 || jobs > 8 )); then
    echo "CARGO_BUILD_JOBS must be an integer from 1 through 8" >&2
    exit 1
fi

export CARGO_BUILD_JOBS="$jobs"
export CARGO_TERM_COLOR=never

run() {
    echo "+ $*"
    "$@"
}

run cargo package --manifest-path reactive-tui-macros/Cargo.toml
run cargo package --manifest-path src/backend/crossterm/Cargo.toml
run cargo package --manifest-path src/backend/engine/Cargo.toml
run cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features
run cargo +1.91.0 check --locked --jobs "$jobs"
run cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features --features ffi
run cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features --features embedded-terminal
run cargo +nightly check --locked --jobs "$jobs" --all-features
run cargo package --no-verify
