#!/usr/bin/env python3
"""render-alloc: RAS-003 and RAS-007 through crates/reactive-tui-suprtui/tests/render_alloc.rs
binary (a counting global allocator), RAS-007 also through the painter's
ras_007_ unit tests and tests/suprtui_renderer.rs, PNT-003 through the pnt_003_ tests in
tests/suprtui_renderer.rs: a per-thread counting allocator measures that
staging a frame copies the element once and presenting copies it not at all,
and the backend must hold the frame as a shared handle.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, strip_test_modules

CRATE = "reactive-tui-suprtui"


def main() -> int:
    results = {}
    results["RAS-003"] = cargo_test_filtered("render_alloc", "ras_003_", package=CRATE)
    # RAS-007: the renderer's lazy hit grid and that it never clears the next
    # buffer (the crate), that painting a frame clears it once (the
    # application's painter unit tests), and that a smaller frame after a
    # larger one leaves no stale cell through the App's backend.
    parts = (("renderer", cargo_test_filtered("render_alloc", "ras_007_", package=CRATE)),
             ("painter", cargo_test_filtered(None, "ras_007_", package="reactive-tui")),
             ("backend", cargo_test_filtered("suprtui_renderer", "ras_007_")))
    results["RAS-007"] = (all(ok for _, (ok, _) in parts),
                          "; ".join(f"{name}: {why}" for name, (_, why) in parts))
    backend = strip_test_modules((ROOT / "src/backend/suprtui.rs").read_text(errors="replace"))
    # render_frame stores a shared handle; whether present sends it without
    # a copy is measured by the tests, not read from the source (cloning the
    # Arc is the correct way to send it).
    if re.search(r"frame:\s*Arc<Element>", backend) is None:
        results["PNT-003"] = (False, "frame is not held as Arc<Element>")
    else:
        results["PNT-003"] = cargo_test_filtered("suprtui_renderer", "pnt_003_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
