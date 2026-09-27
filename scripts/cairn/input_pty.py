#!/usr/bin/env python3
"""INP-001 to INP-011: terminal input on the default backend, observed on a
pseudo-terminal.

Runs the library tests in src/backend/suprtui/input_pty.rs, one requirement
at a time. Each test starts a copy of the library test binary on a new
pseudo-terminal, where an App runs on SuprTuiBackend::new; the test writes
real SGR mouse reports, bracketed pastes, Kitty keys, focus reports and
Ctrl+Z to the terminal, answers the App's startup queries, and reads back
what the App wrote and which events its elements received. Unix only:
the pseudo-terminal comes from openpty.
"""

import sys

from _common import cargo_test_filtered, finish

REQUIREMENTS = [
    ("INP-001", "backend::suprtui::input_pty::inp_001_"),
    ("INP-002", "backend::suprtui::input_pty::inp_002_"),
    ("INP-003", "backend::suprtui::input_pty::inp_003_"),
    ("INP-004", "backend::suprtui::input_pty::inp_004_"),
    ("INP-005", "backend::suprtui::input_pty::inp_005_"),
    ("INP-006", "backend::suprtui::input_pty::inp_006_"),
    ("INP-007", "backend::suprtui::input_pty::inp_007_"),
    ("INP-008", "backend::suprtui::input_pty::inp_008_"),
    ("INP-009", "backend::suprtui::input_pty::inp_009_"),
    ("INP-010", "backend::suprtui::input_pty::inp_010_"),
    ("INP-011", "backend::suprtui::input_pty::inp_011_"),
]


def main() -> int:
    # One test at a time: the click timings are 100 ms against a 500 ms window,
    # and each test runs its own App on its own pseudo-terminal.
    results = {req: cargo_test_filtered(None, name, package="reactive-tui", env={"RUST_TEST_THREADS": "1"})
               for req, name in REQUIREMENTS}
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
