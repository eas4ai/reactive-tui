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
OLD = "until a frame is presented successfully"
NEW = "until the next present reports the previous frame's failure"


def main() -> int:
    results = {}
    results["PIP-001"] = cargo_test_filtered("present_pipeline", "pip_001_")
    text = " ".join(MANUAL.read_text(errors="replace").split()) if MANUAL.is_file() else ""
    if OLD in text or NEW not in text:
        results["PIP-002"] = (False, "manual still says the App keeps geometry " + OLD if OLD in text else "manual lacks the sentence: " + NEW)
    else:
        results["PIP-002"] = cargo_test_filtered("present_pipeline", "pip_002_")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
