#!/usr/bin/env python3
"""BAR-011 (ffi-gates): the library builds, lints, documents and tests with
the `ffi` feature, which the C and TypeScript bindings build it with.

On Linux: cargo build, clippy --all-targets -D warnings, doc --no-deps and
test --no-fail-fast of reactive-tui, each with --locked and --features ffi,
then cargo build with --features ffi,wgpu-graphics. Then the host build
(host_builds.BUILD, every target) with --features ffi,wgpu-graphics on the
macOS host and the Windows tablet, failing on any warning line.

Every Linux command must exit zero before the hosts are tried. A test host
that cannot be reached, or a missing host file, prints no result line, so
Sudus records BAR-011 unverified (as host_builds.py does). So does a Linux
command that failed only because rustc, rustdoc, clippy-driver or the linker
was killed by a signal, read as workspace_gates.py reads it: the crash shows
nothing about the code, and a rerun decides. A command with a failure of its
own (a compile error, a lint, a failed test) fails the gate whatever else
crashed.
"""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

from _common import JOBS, finish, run
from host_builds import BUILD, CONFIG, HOSTS, Unreachable, build_on, snapshot
from workspace_gates import toolchain_crashes

PACKAGE = "reactive-tui"
HOST_FEATURES = "ffi,wgpu-graphics"


def linux() -> tuple[list[str], list[str]]:
    """The Linux commands that failed on the code, as `cargo <subcommand>
    <features>`, and those that failed only because the toolchain crashed."""
    ffi = ["-p", PACKAGE, "--features", "ffi", "--jobs", JOBS]
    both = ["-p", PACKAGE, "--features", HOST_FEATURES, "--jobs", JOBS]
    commands = [
        ["cargo", "build", "--locked", *ffi],
        ["cargo", "clippy", "--locked", "--all-targets", *ffi, "--", "-D", "warnings"],
        ["cargo", "doc", "--locked", "--no-deps", *ffi],
        ["cargo", "test", "--locked", "--no-fail-fast", *ffi],
        ["cargo", "build", "--locked", *both],
    ]
    failing, crashed = [], []
    for command in commands:
        result = run(command, timeout=3600, interleave=True)
        print(result.stdout[-6000:])
        if result.returncode != 0:
            features = command[command.index("--features") + 1]
            name = f"{command[1]} ({features})"
            crashes = toolchain_crashes(result.stdout)
            if crashes:
                crashed.append(f"{name}: {', '.join(crashes)}")
            else:
                failing.append(name)
    return failing, crashed


def gates() -> int:
    failing, crashed = linux()
    if failing:
        return finish({"BAR-011": (False, f"failing on Linux: {', '.join(failing)}")})
    if crashed:
        # No result line: Sudus records the run as unverified, and a rerun decides.
        print(f"BAR-011 unverified: the toolchain crashed, so these commands reached no verdict: {'; '.join(crashed)}")
        return 1
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"BAR-011 unverified: no readable test host file at {CONFIG} ({error})")
        return 1
    with tempfile.TemporaryDirectory(prefix="ffi-gates-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        try:
            built = [build_on(name, config[name], commit, bundle, f"{BUILD} --features {HOST_FEATURES}")
                     for name in HOSTS]
        except (Unreachable, KeyError) as error:
            print(f"BAR-011 unverified: {error}")
            return 1
    ok = all(passed for passed, _ in built)
    return finish({"BAR-011": (ok, "Linux: build, clippy, doc, test and the ffi,wgpu-graphics build pass; "
                               + "; ".join(why for _, why in built))})


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit("usage: ffi_gates.py")
    sys.exit(gates())
