#!/usr/bin/env python3
"""TRM-001 (pty-stop): stopping a pseudo-terminal child ends it, reaps it and
returns within 1 s on Linux and on macOS, also when the child printed faster
than it was read and nothing reads it any more.

Runs the trm_001_ library tests of reactive-tui here, one at a time, then
on the macOS test host: the working tree is snapshotted as a commit, sent
as a bundle over SSH and checked out in the host's clone, where the same
tests run (host_builds.run_on). Each test stops its child on a thread of
its own and fails after 10 s if the stop has not returned, so a hang is a
failed test, not a stuck gate; the host run has a 30-minute ceiling behind
that guard for a build that will not finish.

A Linux failure is the verdict and the host is not tried. A host that cannot
be reached, or a missing host file, prints no result line, so Sudus records
the run as unverified (as host_builds.py does).
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import JOBS, cargo_test_filtered, finish
from host_builds import CONFIG, Unreachable, run_on, snapshot

REQUIREMENT = "TRM-001"
FILTER = "trm_001_"
HOST = "macos"
HOST_TIMEOUT = 1800


def host_verdict(output: str, code: str) -> tuple[bool, str]:
    """Read cargo test's summary and panic messages out of the host's output."""
    ran = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", output)
    if not ran:
        lines = [line.strip() for line in output.splitlines() if line.strip()]
        detail = "; ".join(line for line in lines if line.startswith(("error", "Caused by")))[:300]
        return False, f"{HOST}: the tests did not run (exit {code}): {detail or 'no summary'}"
    passed, failed = int(ran.group(1)), int(ran.group(2))
    if passed + failed == 0:
        return False, f"{HOST}: no test matched {FILTER!r}"
    if failed == 0 and code == "0":
        return True, f"{HOST}: {passed} passed"
    lines = output.splitlines()
    messages = [lines[i + 1].strip() for i, line in enumerate(lines) if "panicked at" in line and i + 1 < len(lines)]
    return False, f"{HOST}: " + ("; ".join(m for m in messages if m)[:300] or f"{failed} failed")


def gate() -> int:
    ok, why = cargo_test_filtered(None, FILTER, package="reactive-tui", env={"RUST_TEST_THREADS": "1"})
    if not ok:
        return finish({REQUIREMENT: (False, f"Linux: {why}")})
    try:
        config = json.loads(CONFIG.read_text())
        host = config[HOST]
    except (OSError, ValueError, KeyError) as error:
        print(f"{REQUIREMENT} unverified: no usable {HOST} entry in the test host file at {CONFIG} ({error})")
        return 1
    with tempfile.TemporaryDirectory(prefix="pty-stop-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        try:
            code, output = run_on(HOST, host, commit, bundle,
                                  f"test --locked -p reactive-tui --lib --jobs {JOBS} -- {FILTER} --test-threads=1",
                                  timeout=HOST_TIMEOUT)
        except Unreachable as error:
            print(f"{REQUIREMENT} unverified: {error}")
            return 1
        except subprocess.TimeoutExpired:
            return finish({REQUIREMENT: (False, f"Linux: {why}; {HOST}: the tests did not finish within {HOST_TIMEOUT // 60} minutes")})
    host_ok, host_why = host_verdict(output, code)
    return finish({REQUIREMENT: (host_ok, f"Linux: {why}; {host_why}")})


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit("usage: pty_stop.py")
    sys.exit(gate())
