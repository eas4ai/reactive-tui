#!/usr/bin/env python3
"""render-replay: RAS-004. Every renderer test frame and charts golden, rendered
by the current rasterizer, replayed through the vt100 and ghostty parser models,
must match the screen recorded in tests/snapshots/renderer before the change.
A frame with no recording is a failure.

Prints one `cairn: RAS-004: pass|fail` line.
"""

import sys

from _common import ROOT, cargo_test_filtered, finish

SNAP = ROOT / "tests/snapshots/renderer"


def main() -> int:
    recordings = sorted(SNAP.glob("*.screen")) if SNAP.is_dir() else []
    if not recordings:
        return finish({"RAS-004": (False, "no recorded screens under tests/snapshots/renderer")})
    ok, why = cargo_test_filtered("renderer_replay", "ras_004_", features=["embedded-terminal"])
    return finish({"RAS-004": (ok, why if not ok else f"{len(recordings)} recordings; {why}")})


if __name__ == "__main__":
    sys.exit(main())
