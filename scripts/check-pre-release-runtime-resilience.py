#!/usr/bin/env python3
"""Run focused checks for pre-release runtime-resilience requirements."""
import os
import pty
from pathlib import Path
import select
import signal
import subprocess
import sys
import time


root = Path(__file__).resolve().parents[1]
requirement = sys.argv[1] if len(sys.argv) == 2 else ""
environment = os.environ.copy()
environment["CARGO_BUILD_JOBS"] = "8"
environment["CARGO_INCREMENTAL"] = "0"
environment["PYTHONDONTWRITEBYTECODE"] = "1"
target_directory = Path(environment.get("CARGO_TARGET_DIR", root / "target"))


def check_rtr_001():
    subprocess.run(
        ["cargo", "build", "--locked", "--jobs", "8", "--example", "accessibility_probe"],
        cwd=root, env=environment, check=True, timeout=300,
    )
    subprocess.run(
        ["python3", "-B", "tests/api_widget_behavior/transport_failures.py", "--binary",
         str(target_directory / "debug/examples/accessibility_probe")],
        cwd=root, env=environment, check=True, timeout=40,
    )


def validate_threaded_probe(mode, returncode, output, timed_out):
    marker = f"RTR002 PASS {mode}".encode()
    assert not timed_out, f"{mode} probe exceeded its shutdown/progress deadline"
    assert returncode == 0, output.decode(errors="replace")
    assert marker in output, f"{mode} probe did not report completion"


def prove_threaded_validator_rejects_violations():
    marker = b"RTR002 PASS saturation"
    violations = [
        ("saturation", 0, b"", False),
        ("saturation", 1, marker, False),
        ("saturation", 0, marker, True),
    ]
    for observation in violations:
        try:
            validate_threaded_probe(*observation)
        except AssertionError:
            continue
        raise AssertionError(f"validator accepted violating observation: {observation!r}")
    validate_threaded_probe("saturation", 0, marker, False)


def run_threaded_probe(mode):
    master, slave = pty.openpty()
    child = None
    output = bytearray()
    timed_out = False
    try:
        probe_environment = environment.copy()
        probe_environment["RTR002_MODE"] = mode
        child = subprocess.Popen(
            ["cargo", "test", "--locked", "--jobs", "8", "--lib", "rtr_002_threaded_event_loop_probe",
             "--", "--ignored", "--nocapture"],
            cwd=root, env=probe_environment, stdin=slave, stdout=slave, stderr=slave,
            start_new_session=True,
        )
        deadline = time.monotonic() + 8
        sent = False
        while child.poll() is None:
            if time.monotonic() >= deadline:
                timed_out = True
                os.killpg(child.pid, signal.SIGKILL)
                child.wait(timeout=2)
                break
            if select.select([master], [], [], 0.05)[0]:
                try:
                    output.extend(os.read(master, 16384))
                except OSError:
                    break
            if mode == "saturation" and not sent and b"RTR002 READY saturation" in output:
                os.write(master, b"a" * 1024 + b"\n")
                sent = True
        while select.select([master], [], [], 0)[0]:
            try:
                output.extend(os.read(master, 16384))
            except OSError:
                break
        returncode = child.returncode if child.returncode is not None else 1
        print(output.decode(errors="replace"), end="", flush=True)
        validate_threaded_probe(mode, returncode, bytes(output), timed_out)
    finally:
        if child and child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=2)
        os.close(master)
        os.close(slave)


def check_rtr_002():
    prove_threaded_validator_rejects_violations()
    subprocess.run(
        ["cargo", "test", "--locked", "--jobs", "8", "--lib",
         "rtr_002_threaded_event_loop_probe", "--no-run"],
        cwd=root, env=environment, check=True, timeout=300,
    )
    subprocess.run(
        ["cargo", "test", "--locked", "--jobs", "8", "--lib",
         "rtr_002_direct_post_reports_capacity_and_recovers_after_consumption"],
        cwd=root, env=environment, check=True, timeout=60,
    )
    for mode in ("saturation", "idle-stop", "drop"):
        run_threaded_probe(mode)


if requirement == "RTR-001":
    check_rtr_001()
elif requirement == "RTR-002":
    check_rtr_002()
else:
    raise SystemExit("usage: check-pre-release-runtime-resilience.py RTR-001|RTR-002")
