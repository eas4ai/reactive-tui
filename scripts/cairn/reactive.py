#!/usr/bin/env python3
"""SIG-001 (reactive-signals): the thread-safe signal's read-modify-writes
are applied once and whole.

Runs the sig_001_ tests of tests/reactive_contract.rs (two threads of
reducer dispatches and of update_atomic calls lose no increment; a call back
into the signal from an update_atomic callback fails with its message), then
reads the sources: none of the framework's read-modify-writes that SIG-001
lists may still go through `update`, and the doc comment of
`ThreadSafeSignal::update` must say that concurrent updates may overwrite
each other. Prints `cairn: SIG-001: pass|fail`.
"""

from __future__ import annotations

import re

from _common import ROOT, cargo_test_filtered, finish

PACKAGE = "reactive-tui"
# The framework's own read-modify-writes SIG-001 names, which go through
# update_atomic; a closure-taking `update` call in one of them is a violation.
PUBLISHERS = [
    "src/widgets/dialog/engine.rs",
    "src/widgets/dialog/wizard/live.rs",
    "src/widgets/display/image/live/worker.rs",
    "src/widgets/display/popover/live.rs",
    "src/widgets/display/popover/live/events.rs",
    "src/widgets/terminal/monitor.rs",
    "src/hooks/clipboard.rs",
    "src/hooks/processor.rs",
]
COPY_UPDATE = re.compile(r"\.update\(\s*\|")


def publishers_on_update() -> list[str]:
    hits = []
    for path in PUBLISHERS:
        text = (ROOT / path).read_text()
        for number, line in enumerate(text.splitlines(), 1):
            if COPY_UPDATE.search(line):
                hits.append(f"{path}:{number}")
    return hits


def update_doc_warns() -> bool:
    """Whether the doc comment above `pub fn update` in the thread-safe
    signal's module says concurrent updates may overwrite each other."""
    lines = (ROOT / "src/reactive/hooks.rs").read_text().splitlines()
    for i, line in enumerate(lines):
        if re.match(r"\s*pub fn update\(", line):
            doc = []
            j = i - 1
            while j >= 0 and lines[j].strip().startswith(("///", "#[")):
                doc.append(lines[j].strip())
                j -= 1
            return any("overwrite" in d for d in doc)
    return False


def main() -> int:
    tests_ok, tests_why = cargo_test_filtered("reactive_contract", "sig_001_", package=PACKAGE)
    hits = publishers_on_update()
    doc = update_doc_warns()
    why = [f"tests: {tests_why}"]
    if hits:
        why.append(f"read-modify-writes still on update: {', '.join(hits)}")
    if not doc:
        why.append("the doc comment of ThreadSafeSignal::update does not say concurrent updates may overwrite each other")
    return finish({"SIG-001": (tests_ok and not hits and doc, "; ".join(why))})


if __name__ == "__main__":
    raise SystemExit(main())
