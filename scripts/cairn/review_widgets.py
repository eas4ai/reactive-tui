#!/usr/bin/env python3
"""review-widgets: the widget defects of the code review of 2026-10-04 and
of the widget study (docs/spec/roadmap.md, widget-defects), each through its
falsifier: docs/spec/text.md TXT-005 and TXT-006, docs/spec/charts.md
CHT-041, docs/spec/theme.md THM-005, docs/spec/layout-widgets.md NAV-005,
docs/spec/data-widgets.md DAT-005 and docs/spec/components.md CMP-008.

A requirement's tests are named after it (`txt_005_...`) and live in
tests/review_widgets.rs and its modules under tests/review_widgets, or, where
they need a module's private parts, among the library's unit tests. This runs
both binaries, filtered by the requirement's prefix: the requirement passes
when no run fails and at least one test of its name ran, and fails when a run
fails or none ran. Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish

PACKAGE = "reactive-tui"
REQUIREMENTS = ["TXT-005", "TXT-006", "CHT-041", "THM-005", "NAV-005", "DAT-005", "CMP-008"]


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("review_widgets", None):
        ok, reason = cargo_test_filtered(binary, prefix, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/review_widgets.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
