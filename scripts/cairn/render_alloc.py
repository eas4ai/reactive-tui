#!/usr/bin/env python3
"""render-alloc: RAS-003 and RAS-007 through crates/reactive-tui-suprtui/tests/render_alloc.rs
binary (a counting global allocator), PNT-003 through a probe of the backend:
the frame is held and sent as a shared handle, never cloned per present.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, strip_test_modules

CRATE = "reactive-tui-suprtui"


def main() -> int:
    results = {}
    for req, sub in (("RAS-003", "ras_003_"), ("RAS-007", "ras_007_")):
        results[req] = cargo_test_filtered("render_alloc", sub, package=CRATE)
    backend = strip_test_modules((ROOT / "src/backend/suprtui.rs").read_text(errors="replace"))
    # The frame is held as a shared handle and present sends that handle; the
    # one copy is made when the frame is staged from a borrowed element.
    resent = re.search(r"self\.frame\.clone\(\)", backend) is not None
    shared = re.search(r"frame:\s*Arc<Element>", backend) is not None
    if resent or not shared:
        why = "present clones the frame element" if resent else "frame is not held as Arc<Element>"
        results["PNT-003"] = (False, why)
    else:
        results["PNT-003"] = cargo_test_filtered("suprtui_renderer", "pnt_003_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
