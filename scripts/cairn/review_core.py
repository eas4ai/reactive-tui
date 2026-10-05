#!/usr/bin/env python3
"""review-core: the code review of 2026-10-04's core findings
(docs/spec/roadmap.md, review-core-findings), each through its falsifier.

A requirement's tests are named after it (`cmp_001_...`) and live in
tests/review_core.rs and its modules under tests/review_core, or, where they
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
    "CMP-001", "CMP-002", "CMP-003", "CMP-004", "CMP-005", "CMP-006", "CMP-007",
    "STY-001", "STY-002", "STY-003", "STY-004",
    "PNT-006",
    "SIG-003", "SIG-004", "SIG-005", "SIG-006", "SIG-007",
]


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("review_core", None):
        ok, reason = cargo_test_filtered(binary, prefix, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/review_core.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
