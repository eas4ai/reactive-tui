#!/usr/bin/env python3
"""BAR-006: a delivered widget has a catalog page and a manual section whose
builder methods exist in the code. Scope for this commitment: the chart
family (line, area, bar, candlestick).

Also exposes `chart_docs_problems()` for the charts-goldens mechanism, which
owns CHT-023.
"""

import re
import sys

from _common import ROOT, rust_sources, strip_test_modules

CHART_TYPES = ["line", "area", "scatter", "bar", "candlestick"]
CATALOG = ROOT / "examples/widget_catalog/catalog.rs"
MANUAL = ROOT / "manual/display-widgets.md"


def code_methods() -> set[str]:
    text = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"))
    return set(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)", text))


def chart_docs_problems() -> list[str]:
    problems = []
    catalog = CATALOG.read_text(errors="replace").lower() if CATALOG.exists() else ""
    manual = MANUAL.read_text(errors="replace") if MANUAL.exists() else ""
    for kind in CHART_TYPES:
        if f"{kind} chart" not in catalog and f"{kind}_chart" not in catalog:
            problems.append(f"catalog has no {kind} chart page")
        if not re.search(rf"^#+ .*\b{kind}\b", manual, re.I | re.M):
            problems.append(f"manual has no heading for the {kind} chart")
    if not re.search(r"^#+ .*size class", manual, re.I | re.M):
        problems.append("manual has no size-class heading")
    for cls in ("mini", "medium", "large"):
        if not re.search(rf"\b{cls}\b", manual, re.I):
            problems.append(f"manual does not describe the {cls} size class")
    methods = code_methods()
    cited = set(re.findall(r"`\.?([a-z_][a-z0-9_]*)\(", manual))
    for name in sorted(cited - methods):
        problems.append(f"manual cites .{name}() which is not a pub fn in chart code")
    return problems


def main() -> int:
    problems = chart_docs_problems()
    if problems:
        print("BAR-006 violated:")
        for p in problems:
            print("  " + p)
        return 1
    print("BAR-006 holds for the chart family")
    return 0


if __name__ == "__main__":
    sys.exit(main())
