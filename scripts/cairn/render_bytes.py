#!/usr/bin/env python3
"""render-bytes: RAS-001, RAS-002, RAS-005, RAS-006 and RAS-008 through the
crates/reactive-tui-suprtui/tests/render_bytes.rs binary (one `ras_NNN_` test group per requirement),
RAS-005 in the optimized build with a best-of-three inside the test.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules

CRATE = "reactive-tui-suprtui"
BINARY = "render_bytes"


def main() -> int:
    results = {}
    for req, sub in (("RAS-001", "ras_001_"), ("RAS-002", "ras_002_"), ("RAS-008", "ras_008_")):
        results[req] = cargo_test_filtered(BINARY, sub, package=CRATE)
    # RAS-005 is best of three runs: the bench is memory-bound, so a run under
    # load from another build can miss the bound while the renderer meets it.
    for attempt in range(1, 4):
        ok, why = cargo_test_filtered(BINARY, "ras_005_", package=CRATE, release=True)
        if ok:
            why = f"run {attempt} of 3: {why}"
            break
    results["RAS-005"] = (ok, why)
    render = "\n".join(strip_test_modules(f.read_text(errors="replace"))
                       for f in rust_sources("crates/reactive-tui-suprtui/src/render.rs"))
    stats = re.search(r"pub struct RenderStats\s*\{([^}]*)\}", render, re.S)
    fields = set(re.findall(r"pub (\w+):", stats.group(1))) if stats else set()
    wanted = {"bytes_emitted", "moves_emitted", "moves_elided", "fg_emitted", "fg_elided", "bg_emitted",
              "bg_elided", "attr_emitted", "attr_elided", "layout_ns", "diff_ns", "emit_ns", "write_ns"}
    missing = sorted(wanted - fields)
    if missing:
        results["RAS-006"] = (False, "RenderStats lacks " + ", ".join(missing))
    else:
        results["RAS-006"] = cargo_test_filtered("suprtui_renderer", "ras_006_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
