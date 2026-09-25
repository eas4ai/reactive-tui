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
    if [[ "$(uname -s)" == "Linux" ]] && command -v taskset >/dev/null 2>&1; then
        local cpu_list="0-$((jobs - 1))"
        echo "+ taskset --cpu-list $cpu_list $*"
        taskset --cpu-list "$cpu_list" "$@"
    else
        echo "+ $*"
        "$@"
    fi
}

zig_0_16_0() {
    if command -v zig >/dev/null 2>&1 && [[ "$(zig version)" == "0.16.0" ]]; then
        run "$@"
    elif command -v mise >/dev/null 2>&1; then
        run mise exec zig@0.16.0 -- "$@"
    else
        echo "Zig 0.16.0 is required for embedded-terminal release checks" >&2
        return 1
    fi
}

run cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features
run cargo +1.91.0 check --locked --jobs "$jobs"
run cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features --features ffi
zig_0_16_0 cargo +1.91.0 check --locked --jobs "$jobs" --no-default-features --features embedded-terminal
zig_0_16_0 cargo +nightly check --locked --jobs "$jobs" --all-features
run cargo +1.91.0 package --workspace --no-verify
