#!/usr/bin/env python3
"""blitters: BLT-001 and BLT-002 through the `blitters` test binary in
crates/reactive-tui-suprtui/tests (exhaustive two-partition check over fixed
blocks, transparency, glyph ranges, tier choice by capability, table and
override) and the application's `blt_00N_` library tests (the image widget's
block fallback, grid backgrounds through the painter, the host identity and
the application override). Borrowed algorithms must be credited in the
crate's UPSTREAM.md, and the manual must name the environment override.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import ROOT, cargo_test_filtered, finish

CRATE = "reactive-tui-suprtui"
UPSTREAM = ROOT / "crates/reactive-tui-suprtui/UPSTREAM.md"
MANUAL = ROOT / "manual/images-and-clipboard.md"


def both(prefix: str) -> tuple[bool, str]:
    """The crate's and the application's tests for one requirement."""
    ok, why = cargo_test_filtered("blitters", prefix, package=CRATE)
    if not ok:
        return ok, "crate: " + why
    ok, app = cargo_test_filtered(None, prefix, package="reactive-tui")
    return ok, f"crate: {why}; application: {app}"


def main() -> int:
    results = {}
    ok, why = both("blt_001_")
    credited = UPSTREAM.is_file() and "notcurses" in UPSTREAM.read_text(errors="replace").lower()
    results["BLT-001"] = (ok and credited, why if not ok or credited else "UPSTREAM.md does not credit notcurses")
    ok, why = both("blt_002_")
    documented = MANUAL.is_file() and "REACTIVE_TUI_BLITTER" in MANUAL.read_text(errors="replace")
    results["BLT-002"] = (ok and documented, why if not ok or documented else "the manual does not name REACTIVE_TUI_BLITTER")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
