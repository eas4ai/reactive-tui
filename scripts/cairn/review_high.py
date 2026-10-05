#!/usr/bin/env python3
"""review-high: the code review of 2026-10-04's six high-priority findings
(docs/spec/roadmap.md, review-high-findings), each through its falsifier in
tests/review_high.rs, run on this host:

- SIG-002 (docs/spec/reactive.md): an animation's `on_update` and
  `on_complete` callbacks that read their animation and stop it return,
  updated directly and by an `AnimationManager`;
- FFI-001 (docs/spec/ffi.md): an audit of src/ffi finds no safe C export
  that uses its caller's pointer beyond a null check, and no unsafe export
  that takes a pointer without a `# Safety` section; the audit first shows it
  catches a violating fixture;
- THM-004 (docs/spec/theme.md): a theme whose variables name each other
  resolves, in a child process, to nothing for a name and to the fallback
  for a role, within a second;
- CHT-040 (docs/spec/charts.md): charts asking for ten million slots or
  grid columns stay within 64 MiB and 2 seconds, and a count of usize::MAX
  draws its one value.

INP-012 (docs/spec/input.md) runs tests/review_high_windows.rs on the
Windows test tablet, through the test host file host_builds.py reads: a
pseudo console runs an App on the direct TTY backend and a key is typed into
it. When the tablet cannot be reached INP-012 gets no result line, so Sudus
records it unverified.

Prints one `cairn: <REQ>: pass|fail` line per requirement it reached.
"""

import json
import re
import sys
import tempfile
from pathlib import Path

from _common import cargo_test_filtered, report
from host_builds import CONFIG, Unreachable, run_on, snapshot

PACKAGE = "reactive-tui"
LINUX = {
    "SIG-002": "sig_002_",
    "FFI-001": "ffi_001_",
    "THM-004": "thm_004_",
    "CHT-040": "cht_040_",
}


def windows_result(output: str) -> tuple[bool, str]:
    """INP-012's verdict from the tablet's test run."""
    ran = re.search(r"test result: \w+\. (\d+) passed; (\d+) failed", output)
    if not ran:
        lines = [line.strip() for line in output.splitlines() if line.strip()]
        raise Unreachable("the test did not run: " + "; ".join(lines[-3:])[:300])
    passed, failed = int(ran.group(1)), int(ran.group(2))
    if failed == 0 and passed > 0:
        return True, f"{passed} passed on the Windows tablet"
    lines = output.splitlines()
    messages = [lines[i + 1].strip() for i, line in enumerate(lines) if "panicked at" in line and i + 1 < len(lines)]
    return False, "; ".join(message for message in messages if message)[:300] or f"{failed} failed"


def main() -> int:
    results = {}
    for req, prefix in LINUX.items():
        results[req] = cargo_test_filtered("review_high", prefix, package=PACKAGE)
    try:
        config = json.loads(CONFIG.read_text())
        host = config["windows"]
    except (OSError, ValueError, KeyError) as error:
        print(f"INP-012 unverified: no Windows test host in the test host file ({error})")
        host = None
    if host is not None:
        try:
            with tempfile.TemporaryDirectory(prefix="review-high-") as scratch:
                commit, bundle = snapshot(Path(scratch))
                _, output = run_on("windows", host, commit, bundle,
                                   f"test --locked -p {PACKAGE} --test review_high_windows -- --nocapture",
                                   timeout=3600)
            print(output[-3000:])
            results["INP-012"] = windows_result(output)
        except Unreachable as error:
            print(f"INP-012 unverified: the Windows tablet was not reached: {error}")
    ok_all = True
    for req, (ok, why) in results.items():
        ok_all &= report(req, ok, why)
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
