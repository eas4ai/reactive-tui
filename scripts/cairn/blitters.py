#!/usr/bin/env python3
"""blitters: BLT-001 and BLT-002 through the crate's tests/blitters.rs binary
(exhaustive two-partition check over fixed blocks, transparency, tier choice by
capability, table and override). Borrowed algorithms must be credited in the
crate's UPSTREAM.md.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import ROOT, cargo_test_filtered, finish

CRATE = "reactive-tui-suprtui"
UPSTREAM = ROOT / "crates/reactive-tui-suprtui/UPSTREAM.md"


def main() -> int:
    results = {}
    credited = UPSTREAM.is_file() and "notcurses" in UPSTREAM.read_text(errors="replace").lower()
    ok, why = cargo_test_filtered("blitters", "blt_001_", package=CRATE)
    results["BLT-001"] = (ok and credited, why if not ok else ("" if credited else "UPSTREAM.md does not credit notcurses"))
    results["BLT-002"] = cargo_test_filtered("blitters", "blt_002_", package=CRATE)
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
