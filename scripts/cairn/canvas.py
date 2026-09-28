#!/usr/bin/env python3
"""The graphics canvas mechanisms (docs/spec/canvas.md and BAR-010).
Usage: canvas.py <group>

scenes (canvas-scenes): GFX-001, GFX-003, GFX-007 and GFX-008 through
    tests/canvas_scenes.rs, and GFX-008's demo tests in
    tests/widget_catalog_behavior.rs and tests/animation_showcase_behavior.rs.
output (canvas-output): GFX-005 through tests/canvas_output.rs and GFX-006
    through the library's pseudo-terminal tests
    (src/backend/suprtui/input_pty.rs).
hosts (canvas-hosts): GFX-002 through tests/canvas_hosts.rs on this host, the
    macOS host and the Windows tablet, each of which has a hardware adapter,
    and GFX-004 through tests/canvas_speed.rs in a release build on the
    tablet, whose 95th percentiles it prints.
gates (canvas-gates): BAR-010: cargo build, clippy -D warnings, doc and test
    of reactive-tui with the wgpu-graphics feature here, then the host build
    with the feature on both test hosts, failing on any warning.

Every cargo run passes --features wgpu-graphics. The GPU tests take turns
through serial_test, so their binaries run with cargo's usual threads. A test
host that cannot be reached, or a missing host file, prints no line for the
requirements that need it, so Sudus records them unverified (as
host_builds.py does).
"""

from __future__ import annotations

import json
import re
import sys
import tempfile
from pathlib import Path

from _common import JOBS, cargo_test_filtered, finish, report, run
from host_builds import BUILD, CONFIG, HOSTS, Unreachable, build_on, run_on, snapshot

FEATURE = ["wgpu-graphics"]
PACKAGE = "reactive-tui"


def test_group(binaries: list[str], prefix: str) -> tuple[bool, str]:
    """The tests named `prefix*` in each binary, as one result."""
    results = [cargo_test_filtered(binary, prefix, features=FEATURE, package=PACKAGE) for binary in binaries]
    return all(ok for ok, _ in results), "; ".join(f"{b}: {why}" for b, (_, why) in zip(binaries, results))


def scenes() -> int:
    return finish({
        "GFX-001": test_group(["canvas_scenes"], "gfx_001_"),
        "GFX-003": test_group(["canvas_scenes"], "gfx_003_"),
        "GFX-007": test_group(["canvas_scenes"], "gfx_007_"),
        "GFX-008": test_group(["canvas_scenes", "widget_catalog_behavior", "animation_showcase_behavior"],
                              "gfx_008_"),
    })


def output() -> int:
    return finish({
        "GFX-005": test_group(["canvas_output"], "gfx_005_"),
        "GFX-006": cargo_test_filtered(None, "backend::suprtui::input_pty::gfx_006_", package=PACKAGE),
    })


def test_summary(output: str) -> tuple[bool, str]:
    """Whether one remote cargo test run passed, with its failures."""
    ran = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed", output)
    if not ran:
        errors = [line.strip() for line in output.splitlines() if line.startswith("error")]
        return False, f"did not run: {'; '.join(errors[:3]) or 'no test summary'}"
    passed = sum(int(p) for p, _ in ran)
    failed = sum(int(f) for _, f in ran)
    if failed == 0 and passed > 0:
        return True, f"{passed} passed"
    lines = output.splitlines()
    messages = [lines[i + 1].strip() for i, l in enumerate(lines) if "panicked at" in l and i + 1 < len(lines)]
    return False, "; ".join(messages)[:300] or f"{failed} failed"


def hosts() -> int:
    local = test_group(["canvas_hosts"], "gfx_002_")
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"GFX-002 and GFX-004 unverified: no readable test host file at {CONFIG} ({error})")
        return 1
    tests = f"test --locked -p {PACKAGE} --features wgpu-graphics --jobs {JOBS}"
    results = {"linux": local}
    speed = None
    with tempfile.TemporaryDirectory(prefix="canvas-hosts-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        try:
            for name in HOSTS:
                _, out = run_on(name, config[name], commit, bundle, f"{tests} --test canvas_hosts")
                results[name] = test_summary(out)
            _, out = run_on("windows", config["windows"], commit, bundle,
                            f"{tests} --release --test canvas_speed -- --ignored --nocapture")
            speed = test_summary(out)
            timing = re.search(r"GFX-004 .*", out)
            if timing:
                speed = (speed[0], f"{timing.group(0)}; {speed[1]}")
        except (Unreachable, KeyError) as error:
            print(f"GFX-002 and GFX-004 unverified: {error}")
            return 1
    ok = all(passed for passed, _ in results.values())
    report("GFX-002", ok, "; ".join(f"{name}: {why}" for name, (_, why) in results.items()))
    report("GFX-004", speed[0], f"windows: {speed[1]}")
    return 0 if ok and speed[0] else 1


def gates() -> int:
    feature = ["-p", PACKAGE, "--features", "wgpu-graphics", "--jobs", JOBS]
    commands = [
        ["cargo", "build", "--locked", *feature],
        ["cargo", "clippy", "--locked", "--all-targets", *feature, "--", "-D", "warnings"],
        ["cargo", "doc", "--locked", "--no-deps", *feature],
        ["cargo", "test", "--locked", "--no-fail-fast", *feature],
    ]
    failing = []
    for command in commands:
        result = run(command, timeout=3600, interleave=True)
        print(result.stdout[-6000:])
        if result.returncode != 0:
            failing.append(" ".join(command[:2]))
    if failing:
        return finish({"BAR-010": (False, f"failing on Linux: {', '.join(failing)}")})
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"BAR-010 unverified: no readable test host file at {CONFIG} ({error})")
        return 1
    with tempfile.TemporaryDirectory(prefix="canvas-gates-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        try:
            built = [build_on(name, config[name], commit, bundle, f"{BUILD} --features wgpu-graphics")
                     for name in HOSTS]
        except (Unreachable, KeyError) as error:
            print(f"BAR-010 unverified: {error}")
            return 1
    ok = all(passed for passed, _ in built)
    return finish({"BAR-010": (ok, "Linux: build, clippy, doc and test pass; " + "; ".join(why for _, why in built))})


GROUPS = {"scenes": scenes, "output": output, "hosts": hosts, "gates": gates}


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in GROUPS:
        sys.exit(f"usage: canvas.py {'|'.join(GROUPS)}")
    sys.exit(GROUPS[sys.argv[1]]())
