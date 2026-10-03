#!/usr/bin/env python3
"""The graphics canvas mechanisms (docs/spec/canvas.md and BAR-010).
Usage: canvas.py <group>

scenes (canvas-scenes): GFX-001, GFX-003, GFX-007 and GFX-008 through
    tests/canvas_scenes.rs, and GFX-008's demo tests in
    tests/widget_catalog_behavior.rs and tests/animation_showcase_behavior.rs.
output (canvas-output): GFX-005 through tests/canvas_output.rs and the
    picture thread's unit tests, GFX-006 through the library's
    pseudo-terminal tests (src/backend/suprtui/input_pty.rs), and GFX-010,
    pictures one pixel per screen pixel, through tests/canvas_output.rs.
hosts (canvas-hosts): GFX-002 through tests/canvas_hosts.rs on this host, the
    macOS host and the Windows tablet, each of which has a hardware adapter;
    GFX-004 through tests/canvas_speed.rs in a release build on the tablet,
    whose 95th percentiles it prints; and GFX-011, fifteen canvases on one
    drawing thread, through tests/canvas_speed.rs in a release build on this
    host, where its bound binds, and on the two test hosts, whose numbers it
    records. A failure on any host it reached decides GFX-002.
pictures (canvas-pictures): GFX-009 through tests/canvas_pictures.rs, every
    test, the ignored ones too, in a release build on this host, the macOS
    host and the Windows tablet, with the App's wait in present printed for
    each pixel output, and the picture thread's unit test here. A failure on
    any host it reached decides GFX-009.
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
    # The picture thread's own GFX-005 rule, that a picture a newer one of
    # its canvas followed is never written, is in the library's tests.
    binary_ok, binary_why = test_group(["canvas_output"], "gfx_005_")
    lib_ok, lib_why = cargo_test_filtered(
        None, "backend::suprtui::graphics::maker::tests::gfx_005_", features=FEATURE, package=PACKAGE)
    return finish({
        "GFX-005": (binary_ok and lib_ok, f"{binary_why}; picture thread: {lib_why}"),
        "GFX-006": cargo_test_filtered(None, "backend::suprtui::input_pty::gfx_006_", package=PACKAGE),
        "GFX-010": test_group(["canvas_output"], "gfx_010_"),
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


def speed_lines(output: str) -> str:
    """The lines a canvas_speed run prints for GFX-004 and GFX-011."""
    return "; ".join(re.findall(r"GFX-01[14] .*", output))


def hosts() -> int:
    results = {"linux": test_group(["canvas_hosts"], "gfx_002_")}
    missing = []
    speed = None
    recorded = []
    tests = f"test --locked -p {PACKAGE} --features wgpu-graphics --jobs {JOBS}"
    # GFX-011 binds here: fifteen canvases on one drawing thread, in release
    # on the hardware adapter.
    local = run(["cargo", *f"{tests} --release --test canvas_speed -- --ignored gfx_011_ --nocapture".split()],
                timeout=3600, interleave=True)
    print(local.stdout[-3000:])
    many_ok, many_why = test_summary(local.stdout + local.stderr)
    many = (many_ok, f"{speed_lines(local.stdout)}; {many_why}")
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"no readable test host file at {CONFIG} ({error})")
        config = {}
    with tempfile.TemporaryDirectory(prefix="canvas-hosts-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        for name in HOSTS:
            try:
                _, out = run_on(name, config[name], commit, bundle, f"{tests} --test canvas_hosts")
                results[name] = test_summary(out)
                # The tablet's GFX-004 bound, and the GFX-011 numbers of both
                # hosts, recorded and not binding there.
                _, out = run_on(name, config[name], commit, bundle,
                                f"{tests} --release --test canvas_speed -- --ignored --nocapture")
                lines = speed_lines(out)
                recorded.append(f"{name}: {lines or 'no GFX-011 line'}")
                if name == "windows":
                    speed = test_summary(out)
                    timing = re.search(r"GFX-004 .*", out)
                    if timing:
                        speed = (speed[0], f"{timing.group(0)}; {speed[1]}")
            except (Unreachable, KeyError) as error:
                print(f"{name} not reached: {error}")
                missing.append(name)
    failed = [f"{name}: {why}" for name, (ok, why) in results.items() if not ok]
    # A failure on any host it reached decides GFX-002; a host it could not
    # reach leaves GFX-002 unverified only while every reached host passed.
    if failed:
        report("GFX-002", False, "; ".join(failed))
    elif missing:
        print(f"GFX-002 unverified: not reached: {', '.join(missing)}")
    else:
        report("GFX-002", True, "; ".join(f"{name}: {why}" for name, (_, why) in results.items()))
    if speed is None:
        print("GFX-004 unverified: the Windows tablet was not reached")
    else:
        report("GFX-004", speed[0], f"windows: {speed[1]}")
    report("GFX-011", many[0], "; ".join([f"linux: {many[1]}", *recorded]))
    return 0 if not failed and not missing and speed and speed[0] and many[0] else 1


def picture_summary(output: str) -> tuple[bool, str]:
    """Whether one run of tests/canvas_pictures.rs passed, with the App's
    wait in present it measured for each output."""
    ok, why = test_summary(output)
    waits = re.findall(r"GFX-009 the App's wait in present .*? as (.+?): median ([\d.]+) ms, p95 ([\d.]+) ms",
                       output)
    measured = ", ".join(f"{name} p95 {p95} ms" for name, _, p95 in waits)
    return ok, f"{why}; {measured}" if measured else why


def pictures() -> int:
    args = (f"test --locked -p {PACKAGE} --features wgpu-graphics --jobs {JOBS} --release "
            "--test canvas_pictures -- --include-ignored --nocapture --test-threads=1")
    local = run(["cargo", *args.split()], timeout=3600, interleave=True)
    print(local.stdout[-6000:])
    results = {"linux": picture_summary(local.stdout)}
    # The library's GFX-009 tests: a picture handed over while another of
    # its canvas waits replaces it, a thread that cannot start refuses the
    # picture, and sync says so when it gives up on pictures.
    results["linux, picture thread"] = cargo_test_filtered(
        None, "gfx_009_", features=FEATURE, package=PACKAGE)
    missing = []
    try:
        config = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"no readable test host file at {CONFIG} ({error})")
        config = {}
    with tempfile.TemporaryDirectory(prefix="canvas-pictures-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        for name in HOSTS:
            try:
                _, out = run_on(name, config[name], commit, bundle, args)
                results[name] = picture_summary(out)
            except (Unreachable, KeyError) as error:
                print(f"{name} not reached: {error}")
                missing.append(name)
    failed = [f"{name}: {why}" for name, (ok, why) in results.items() if not ok]
    # A failure on any host it reached decides GFX-009; a host it could not
    # reach leaves it unverified only while every reached host passed.
    if failed:
        report("GFX-009", False, "; ".join(failed))
        return 1
    if missing:
        print(f"GFX-009 unverified: not reached: {', '.join(missing)}")
        return 1
    report("GFX-009", True, "; ".join(f"{name}: {why}" for name, (_, why) in results.items()))
    return 0


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


GROUPS = {"scenes": scenes, "output": output, "hosts": hosts, "pictures": pictures, "gates": gates}


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in GROUPS:
        sys.exit(f"usage: canvas.py {'|'.join(GROUPS)}")
    sys.exit(GROUPS[sys.argv[1]]())
