#!/usr/bin/env python3
"""review-native: the code review of 2026-10-04's native animation, editor and markdown findings
(docs/spec/roadmap.md, review-native-findings), each through its falsifier.

A requirement's tests are named after it (`ani_001_...`) and live in
tests/review_native.rs and its modules under tests/review_native, or, where they
need a module's private parts, among the library's unit tests. This runs
both binaries filtered by the requirement's prefix: the requirement passes
when no run fails and at least one test of its name ran, and fails when a
run fails or none ran.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish

PACKAGE = "reactive-tui"
REQUIREMENTS = [
    "ANI-001", "ANI-002", "ANI-003", "ANI-004", "ANI-005", "ANI-006", "ANI-007", "ANI-008", "ANI-009",
    "TXT-001", "TXT-002", "TXT-003",
]


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("review_native", None):
        ok, reason = cargo_test_filtered(binary, prefix, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/review_native.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
