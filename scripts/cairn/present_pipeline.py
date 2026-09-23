#!/usr/bin/env python3
"""present-pipeline: PIP-001 and PIP-002 through the tests/present_pipeline.rs
binary (a slow writer measures present's wall time and the frames in flight; a
failing writer checks the report and the forced repaint). PIP-002 also requires
the manual sentence to be updated.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import ROOT, cargo_test_filtered, finish

MANUAL = ROOT / "manual/rendering-and-backends.md"
# The sentences before this commitment: the first predates the pipeline, the
# second named only present and shutdown and said the App kept the
# acknowledged geometry before the failure was reported.
STALE = ("until a frame is presented successfully",
         "until the next present reports the previous frame's failure")
# What the manual must say: every call that reports a failure, the geometry
# before and after the report, and that a failed frame is never the fallback.
NEW = ("reported by the next `present`, by `sync` or by `shutdown`",
       "Until the failure is reported, the backend reports the newest presented frame's geometry",
       "the geometry of the last frame whose flush was acknowledged",
       "the failed frame is never used as the fallback")


def main() -> int:
    results = {}
    results["PIP-001"] = cargo_test_filtered("present_pipeline", "pip_001_")
    text = " ".join(MANUAL.read_text(errors="replace").split()) if MANUAL.is_file() else ""
    stale = [s for s in STALE if s in text]
    missing = [s for s in NEW if s not in text]
    if stale:
        results["PIP-002"] = (False, "manual still says: " + stale[0])
    elif missing:
        results["PIP-002"] = (False, "manual lacks: " + missing[0])
    else:
        results["PIP-002"] = cargo_test_filtered("present_pipeline", "pip_002_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
