#!/usr/bin/env python3
"""widgets-orca: NAV-011 (docs/spec/layout-widgets.md), what Orca speaks for
the medium widgets on the accessibility probe's medium widgets page, on
this Linux host.

Builds the example accessibility_probe as input-orca does, then runs
tests/api_widget_behavior/orca.py with `--catalog medium`: the probe opens on
its medium widgets page (`--medium`) in GNOME Terminal on a private Xvfb
display and D-Bus session with real Orca, keys are sent by XTest, and
tests/api_widget_behavior/orca_medium.py reads the speech Orca logs after
each step: the split's divider Panes given the focus and Right; the sidebar
Main's first item given the focus, Down onto a closed group and Right; the
list Fruits given the focus and End; and a row appended to the log view
Build log while the focus stays on the list.

The host's needs, the one Orca per user, and what makes a run unverified
rather than failed are input-orca's (scripts/cairn/input_orca.py, whose
helpers this script uses). Prints `cairn: NAV-011: pass|fail` otherwise; a
failure quotes the harness's error, for a speech step the AssertionError
that names every step whose speech broke the requirement.
"""

from __future__ import annotations

import os
import subprocess
import sys

from _common import report, run
from input_orca import (HARNESS, PYTHON, BuildFailed, Unverified, build_probe, library_dir,
                        missing_tools, running_orca, verdict)

REQUIREMENT = "NAV-011"
SUMMARY = "A11Y medium widgets:"


def main() -> int:
    missing = missing_tools()
    if missing:
        print(f"{REQUIREMENT} unverified: missing on this host: {', '.join(missing)}")
        return 0
    others = running_orca()
    if others:
        print(f"{REQUIREMENT} unverified: an Orca already runs for this user, and Orca allows one: "
              f"{'; '.join(others)}")
        return 0
    try:
        try:
            probe = build_probe()
        except BuildFailed as error:
            return 0 if report(REQUIREMENT, False, f"the accessibility probe does not build: {error}") else 1
        library = library_dir()
        env = {"LD_LIBRARY_PATH": library + (":" + os.environ["LD_LIBRARY_PATH"]
                                             if os.environ.get("LD_LIBRARY_PATH") else ""),
               # The harness imports its scenario modules: keep bytecode out of tests/.
               "PYTHONDONTWRITEBYTECODE": "1"}
        try:
            result = run([PYTHON, "-B", HARNESS, "--binary", str(probe), "--catalog", "medium",
                          "--geometry", "120x40"], timeout=240, env=env, interleave=True)
        except subprocess.TimeoutExpired:
            return 0 if report(REQUIREMENT, False, "the Orca run did not finish within 240 s") else 1
        print(result.stdout[-6000:])
        ok, why = verdict(result.stdout, result.returncode, SUMMARY)
    except Unverified as error:
        print(f"{REQUIREMENT} unverified: {error}")
        return 0
    return 0 if report(REQUIREMENT, ok, why) else 1


if __name__ == "__main__":
    sys.exit(main())
