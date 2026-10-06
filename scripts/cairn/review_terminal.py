#!/usr/bin/env python3
"""review-terminal: the code review of 2026-10-04's terminal and platform findings
(docs/spec/roadmap.md, review-terminal-findings), each through its falsifier.

A requirement's tests are named after it (`plt_001_...`) and live in
tests/review_terminal.rs and its modules under tests/review_terminal, or, where
they need a module's private parts, among the library's unit tests. This runs
both binaries filtered by the requirement's prefix (PLT-016's unit test lives
in the graphics module, so its library run turns the `wgpu-graphics` feature
on): the requirement passes when no run fails and at least one test of its
name ran, and fails when a run fails or none ran.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish

PACKAGE = "reactive-tui"
REQUIREMENTS = [f"PLT-{n:03d}" for n in range(1, 17)]
# Library runs that need a feature: requirement -> features.
LIBRARY_FEATURES = {"PLT-016": ["wgpu-graphics"]}


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("review_terminal", None):
        features = LIBRARY_FEATURES.get(req) if binary is None else None
        ok, reason = cargo_test_filtered(binary, prefix, features=features, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/review_terminal.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
