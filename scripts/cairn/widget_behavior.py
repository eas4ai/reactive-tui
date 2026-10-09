#!/usr/bin/env python3
"""widget-behavior: the behavior the widgets share, from the widget study's
adoption changes (docs/spec/roadmap.md, widget-behavior), each through its
falsifier: docs/spec/components.md CMP-009, docs/spec/input-widgets.md
CTL-005 and CTL-006, docs/spec/theme.md THM-006, docs/spec/layout-widgets.md
NAV-006, docs/spec/clipboard.md CLP-001 and docs/spec/keymap.md KEY-001 and
KEY-002.

A requirement's tests are named after it (`key_001_...`) and live in
tests/widget_behavior.rs and its modules under tests/widget_behavior, or,
where they need a module's private parts, among the library's unit tests.
This runs both binaries, filtered by the requirement's prefix: the
requirement passes when no run fails and at least one test of its name ran,
and fails when a run fails or none ran. Prints one `cairn: <REQ>: pass|fail`
line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish

PACKAGE = "reactive-tui"
REQUIREMENTS = ["CMP-009", "CTL-005", "CTL-006", "THM-006", "NAV-006", "CLP-001", "KEY-001", "KEY-002"]


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("widget_behavior", None):
        ok, reason = cargo_test_filtered(binary, prefix, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/widget_behavior.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
