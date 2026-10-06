#!/usr/bin/env python3
"""review-facade: the code review of 2026-10-04's C facade findings
(docs/spec/roadmap.md, review-facade-findings), each through its falsifier:
docs/spec/ffi.md FFI-002 to FFI-007, docs/spec/text.md TXT-004 and
docs/spec/animation.md ANI-010.

A requirement's tests are named after it (`ffi_002_...`) and live in
tests/review_facade.rs and its modules under tests/review_facade, or, where
they need a module's private parts, among the library's unit tests. This runs
both binaries with the `ffi` feature, filtered by the requirement's prefix:
the requirement passes when no run fails and at least one test of its name
ran, and fails when a run fails or none ran. Three of the tests run in a copy
of the binary on a pseudo-terminal, because the C renderer takes raw mode.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish

PACKAGE = "reactive-tui"
REQUIREMENTS = [f"FFI-{n:03d}" for n in range(2, 8)] + ["TXT-004", "ANI-010"]
FEATURES = ["ffi"]


def check(req: str) -> tuple[bool, str]:
    prefix = req.lower().replace("-", "_") + "_"
    ran, why = [], []
    for binary in ("review_facade", None):
        ok, reason = cargo_test_filtered(binary, prefix, features=FEATURES, package=PACKAGE)
        if reason.startswith("no test matched"):
            continue
        ran.append(ok)
        why.append(f"{binary or 'library'}: {reason}")
    if not ran:
        return False, f"no test named {prefix}* ran in tests/review_facade.rs or the library"
    return all(ran), "; ".join(why)


def main() -> int:
    return finish({req: check(req) for req in REQUIREMENTS})


if __name__ == "__main__":
    sys.exit(main())
