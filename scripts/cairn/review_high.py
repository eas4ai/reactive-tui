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
it. Its verdict is read from cargo's exit code and the run's own last test
summary (host_builds.tablet_test_verdict), whose tests (test_host_builds.py)
run first; when they fail, or the tablet cannot be reached, INP-012 gets no
result line, so Sudus records it unverified.

Prints one `cairn: <REQ>: pass|fail` line per requirement it reached.
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import ROOT, cargo_test_filtered, report
from host_builds import CONFIG, Unreachable, run_on, snapshot, tablet_test_verdict

PACKAGE = "reactive-tui"
LINUX = {
    "SIG-002": "sig_002_",
    "FFI-001": "ffi_001_",
    "THM-004": "thm_004_",
    "CHT-040": "cht_040_",
}
TESTS = ROOT / "scripts/cairn/test_host_builds.py"


def main() -> int:
    results = {}
    for req, prefix in LINUX.items():
        results[req] = cargo_test_filtered("review_high", prefix, package=PACKAGE)
    own = subprocess.run([sys.executable, "-B", str(TESTS)], cwd=ROOT, capture_output=True, text=True, timeout=120)
    try:
        config = json.loads(CONFIG.read_text())
        host = config["windows"]
    except (OSError, ValueError, KeyError) as error:
        print(f"INP-012 unverified: no Windows test host in the test host file ({error})")
        host = None
    if own.returncode != 0:
        print(f"INP-012 unverified: the verdict's own tests failed:\n{own.stderr[-2000:]}")
        host = None
    if host is not None:
        try:
            with tempfile.TemporaryDirectory(prefix="review-high-") as scratch:
                commit, bundle = snapshot(Path(scratch))
                code, output = run_on("windows", host, commit, bundle,
                                      f"test --locked -p {PACKAGE} --test review_high_windows -- --nocapture",
                                      timeout=3600)
            print(output[-3000:])
            results["INP-012"] = tablet_test_verdict(output, code)
        except Unreachable as error:
            print(f"INP-012 unverified: the Windows tablet was not reached: {error}")
    ok_all = True
    for req, (ok, why) in results.items():
        ok_all &= report(req, ok, why)
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
