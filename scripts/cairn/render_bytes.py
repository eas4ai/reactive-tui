#!/usr/bin/env python3
"""render-bytes: RAS-001, RAS-002, RAS-005, RAS-006 and RAS-008 through the
crates/reactive-tui-suprtui/tests/render_bytes.rs binary (one `ras_NNN_` test group per requirement),
RAS-005 once in the optimized build, with the best of three runs inside the test.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules

CRATE = "reactive-tui-suprtui"
BINARY = "render_bytes"


def main() -> int:
    results = {}
    results["RAS-001"] = cargo_test_filtered(BINARY, "ras_001_", package=CRATE)
    # RAS-002: the renderer's byte stream, and every frame the App's backend
    # writes, which adds its own frame boundary around the renderer's bytes.
    stream = cargo_test_filtered(BINARY, "ras_002_", package=CRATE)
    written = cargo_test_filtered("suprtui_renderer", "ras_002_")
    results["RAS-002"] = (stream[0] and written[0], "; ".join(
        f"{name}: {why}" for name, (ok, why) in (("stream", stream), ("backend", written)) if not ok)
        or f"stream {stream[1]}; backend {written[1]}")
    # RAS-008: the byte stream shows a change in any array is found, and the
    # crate's unit test counts the cells the compare builds (none), which
    # only a build with cfg(test) can observe.
    stream = cargo_test_filtered(BINARY, "ras_008_", package=CRATE)
    compare = cargo_test_filtered(None, "ras_008_", package=CRATE)
    results["RAS-008"] = (stream[0] and compare[0], "; ".join(
        f"{name}: {why}" for name, (ok, why) in (("stream", stream), ("compare", compare)) if not ok)
        or f"stream {stream[1]}; compare {compare[1]}")
    # RAS-005 is the best of three runs, and the test times three runs of
    # each bound and keeps the fastest, so it runs once here: a retry would
    # make it the best of nine.
    results["RAS-005"] = cargo_test_filtered(BINARY, "ras_005_", package=CRATE, release=True)
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
