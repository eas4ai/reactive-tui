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

The verdict is read from cargo's exit code and the run's own last test
summary (host_builds.tablet_test_verdict), whose tests
(test_host_builds.py) run first; when they fail there is no verdict. When
the tablet cannot be reached, or the host file names none, no result line
is printed, so Sudus records the run as unverified. Otherwise prints
`cairn: INP-013: pass|fail`.
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import ROOT, report
from host_builds import CONFIG, Unreachable, run_on, snapshot, tablet_test_verdict

PACKAGE = "reactive-tui"
TEST = "startup_windows"
REQUIREMENT = "INP-013"
TESTS = ROOT / "scripts/cairn/test_host_builds.py"
# The child-runner test passes on its own; a terminal-side test must have
# run too.
AT_LEAST = 2


def main() -> int:
    own = subprocess.run([sys.executable, "-B", str(TESTS)], cwd=ROOT, capture_output=True, text=True, timeout=120)
    if own.returncode != 0:
        print(f"{REQUIREMENT} unverified: the mechanism's own tests failed:\n{own.stderr[-2000:]}")
        return 0
    try:
        config = json.loads(CONFIG.read_text())
        host = config["windows"]
    except (OSError, ValueError, KeyError) as error:
        print(f"{REQUIREMENT} unverified: no Windows test host in the test host file ({error})")
        return 0
    try:
        with tempfile.TemporaryDirectory(prefix="startup-windows-") as scratch:
            commit, bundle = snapshot(Path(scratch))
            code, output = run_on("windows", host, commit, bundle,
                                  f"test --locked -p {PACKAGE} --test {TEST} -- --nocapture --test-threads=1",
                                  timeout=3600)
        print(output[-3000:])
        ok, why = tablet_test_verdict(output, code, at_least=AT_LEAST)
    except Unreachable as error:
        print(f"{REQUIREMENT} unverified: the Windows tablet was not reached: {error}")
        return 0
    return 0 if report(REQUIREMENT, ok, why) else 1


if __name__ == "__main__":
    sys.exit(main())
