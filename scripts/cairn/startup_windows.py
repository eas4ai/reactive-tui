#!/usr/bin/env python3
"""startup-windows: INP-013 (docs/spec/input.md), the default backend's
startup exchange on Windows, through its falsifier in
tests/startup_windows.rs, run on the Windows test tablet.

The tablet is read from the test host file host_builds.py reads
(~/.config/reactive-tui/test-hosts.json, outside the repository). The working
tree goes there as a git bundle and `cargo test --test startup_windows` runs
in the tablet's clone, one test at a time: each test runs the binary again as
a pseudo console's child, where an App on the default backend starts, and
answers its questions as a light, dark or silent terminal.

When the tablet cannot be reached, or the host file names none, no result
line is printed, so Sudus records the run as unverified. Otherwise prints
`cairn: INP-013: pass|fail`.
"""

import json
import re
import sys
import tempfile
from pathlib import Path

from _common import report
from host_builds import CONFIG, Unreachable, run_on, snapshot

PACKAGE = "reactive-tui"
TEST = "startup_windows"
REQUIREMENT = "INP-013"


def verdict(output: str) -> tuple[bool, str]:
    """INP-013's verdict from the tablet's test run."""
    ran = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", output)
    if not ran:
        lines = [line.strip() for line in output.splitlines() if line.strip()]
        raise Unreachable("the test did not run: " + "; ".join(lines[-3:])[:300])
    passed, failed = int(ran.group(1)), int(ran.group(2))
    # The child-runner test passes on its own; the terminal-side tests must
    # have run too.
    if failed == 0 and passed > 1:
        return True, f"{passed} passed on the Windows tablet"
    if failed == 0:
        return False, f"only {passed} test ran on the Windows tablet"
    lines = output.splitlines()
    messages = [lines[i + 1].strip() for i, line in enumerate(lines) if "panicked at" in line and i + 1 < len(lines)]
    return False, "; ".join(message for message in messages if message)[:300] or f"{failed} failed"


def main() -> int:
    try:
        config = json.loads(CONFIG.read_text())
        host = config["windows"]
    except (OSError, ValueError, KeyError) as error:
        print(f"{REQUIREMENT} unverified: no Windows test host in the test host file ({error})")
        return 0
    try:
        with tempfile.TemporaryDirectory(prefix="startup-windows-") as scratch:
            commit, bundle = snapshot(Path(scratch))
            _, output = run_on("windows", host, commit, bundle,
                               f"test --locked -p {PACKAGE} --test {TEST} -- --nocapture --test-threads=1",
                               timeout=3600)
        print(output[-3000:])
        ok, why = verdict(output)
    except Unreachable as error:
        print(f"{REQUIREMENT} unverified: the Windows tablet was not reached: {error}")
        return 0
    return 0 if report(REQUIREMENT, ok, why) else 1


if __name__ == "__main__":
    sys.exit(main())
