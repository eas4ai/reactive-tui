#!/usr/bin/env python3
"""Check adaptive FPS and automatic-grid arithmetic boundaries."""

import os
from pathlib import Path
import signal
import subprocess


ROOT = Path(__file__).resolve().parents[1]
MARKERS = (b"RTR004 PASS adaptive", b"RTR004 PASS grid")


def validate(returncode, output, timed_out):
    assert not timed_out, "arithmetic boundary tests exceeded their deadline"
    assert returncode == 0, output.decode(errors="replace")
    for marker in MARKERS:
        assert marker in output, f"missing boundary completion: {marker!r}"


def prove_validator_rejects_violations():
    complete = b"\n".join(MARKERS)
    for observation in (
        (0, complete, True),
        (1, complete, False),
        (0, MARKERS[0], False),
    ):
        try:
            validate(*observation)
        except AssertionError:
            continue
        raise AssertionError(f"validator accepted violating observation: {observation!r}")
    validate(0, complete, False)


def run_tests():
    environment = os.environ.copy()
    environment["CARGO_BUILD_JOBS"] = "8"
    environment["CARGO_INCREMENTAL"] = "0"
    child = subprocess.Popen(
        [
            "cargo",
            "test",
            "--locked",
            "--jobs",
            "8",
            "--test",
            "arithmetic_boundaries",
            "--",
            "--nocapture",
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    timed_out = False
    try:
        output, _ = child.communicate(timeout=300)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(child.pid, signal.SIGKILL)
        output, _ = child.communicate(timeout=5)
    print(output.decode(errors="replace"), end="", flush=True)
    validate(child.returncode, output, timed_out)


prove_validator_rejects_violations()
run_tests()
