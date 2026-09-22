#!/usr/bin/env python3
"""BAR-002: every #[test] asserts an observable outcome or is named smoke_*.

Scans tests/**/*.rs and src/**/*.rs. A test body counts as asserting when it
contains an assert macro (assert!, assert_eq!, assert_ne!, debug_assert*),
panic! or unreachable!, or the test carries #[should_panic]. A bare
matches! does not count; .expect() and .unwrap() do not count. A #[test]
attribute with no fn after it is itself reported. Files with
`harness = false` in Cargo.toml are skipped.

`--fixture PATH` audits one file only (used to demonstrate the failing case).
"""

import re
import sys
from pathlib import Path

from _common import ROOT, rust_sources

TEST_ATTR = re.compile(r"#\[(?:tokio::)?test[^\]]*\]")
FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
MARKERS = re.compile(r"\bassert(?:_eq|_ne)?!|\bpanic!|\bunreachable!|debug_assert")
HARDWARE = re.compile(r"is_hardware|adapter|GPU|wgpu|Xvfb|DISPLAY")
NO_HARNESS = {"dqc_003_captured_diagnostics"}


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def body_after(text: str, start: int) -> str:
    i = text.find("{", start)
    if i < 0:
        return ""
    depth, j = 1, i + 1
    while j < len(text) and depth:
        depth += text[j] == "{"
        depth -= text[j] == "}"
        j += 1
    return text[i:j]


def audit(path: Path) -> list[str]:
    if path.stem in NO_HARNESS:
        return []
    text = path.read_text(errors="replace")
    bad = []
    for m in TEST_ATTR.finditer(text):
        # An attribute quoted inside a comment line is prose, not a test.
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start:m.start()].lstrip().startswith("//"):
            continue
        # Doc comments and other attributes may sit between #[test] and fn;
        # search past them, comment text excluded, and report a miss.
        raw = text[m.end(): m.end() + 4000]
        window = "\n".join("" if line.lstrip().startswith("//") else line for line in raw.splitlines())
        should_panic = "#[should_panic" in text[max(0, m.start() - 120): m.end() + 120]
        fn = FN.search(window)
        if not fn:
            bad.append(f"{rel(path)}: #[test] at offset {m.start()} with no fn after it")
            continue
        name = fn.group(1)
        if name.startswith("smoke_") or should_panic:
            continue
        body = body_after(text, m.end() + fn.end())
        if not MARKERS.search(body):
            bad.append(f"{rel(path)}::{name}")
        elif HARDWARE.search(body) and re.search(r"\breturn\b", body) and "SKIP" not in body:
            bad.append(f"{rel(path)}::{name} (hardware-gated early return without printing SKIP)")
    return bad


def main() -> int:
    if len(sys.argv) > 2 and sys.argv[1] == "--fixture":
        files = [Path(sys.argv[2]).resolve()]
    else:
        files = rust_sources("tests", "src", "crates/reactive-tui-suprtui/src", "crates/reactive-tui-suprtui/tests")
    bad = [b for f in files for b in audit(f)]
    if bad:
        print("BAR-002 violated: tests without an assertion and not named smoke_:")
        for b in bad:
            print("  " + b)
        return 1
    print(f"BAR-002 holds: {len(files)} files audited")
    return 0


if __name__ == "__main__":
    sys.exit(main())
