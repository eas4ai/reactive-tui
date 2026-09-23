#!/usr/bin/env python3
"""painter-goldens: PNT-001, PNT-002 and PNT-004 through the tests/painter_hit.rs
binary: fast path against general path on every golden, per-cell hit query under
masks and z-order, layout not recomputed for an unchanged spec. PNT-002 also probes
that hit_bounds no longer scans cells.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, strip_test_modules


def main() -> int:
    results = {}
    results["PNT-001"] = cargo_test_filtered("painter_hit", "pnt_001_")
    results["PNT-004"] = cargo_test_filtered("painter_hit", "pnt_004_")
    painter = strip_test_modules((ROOT / "src/layout/paint_tree/suprtui.rs").read_text(errors="replace"))
    hb = re.search(r"fn hit_bounds\([^)]*\)[^{]*\{(.*?)\n\}", painter, re.S)
    scans = hb is not None and re.search(r"inside_masks\(", hb.group(1)) is not None
    trait_src = (ROOT / "src/backend/mod.rs").read_text(errors="replace")
    query = re.search(r"fn hit_at\(&self,", trait_src) is not None
    if scans or not query:
        why = "hit_bounds still scans cells through inside_masks" if scans else "Backend has no per-cell hit query hit_at"
        results["PNT-002"] = (False, why)
    else:
        results["PNT-002"] = cargo_test_filtered("painter_hit", "pnt_002_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
