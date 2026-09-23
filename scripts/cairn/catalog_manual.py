#!/usr/bin/env python3
"""BAR-006: a delivered widget has a catalog page and a manual section whose
builder methods exist in the code. Scope: the chart family (line, area,
scatter, bar, candlestick) and the image widget, reworked to draw its block
fallback through the renderer's blitters.

Also exposes `chart_docs_problems()` for the charts-goldens mechanism, which
owns CHT-023.
"""

import re
import sys

from _common import ROOT, rust_sources, strip_test_modules
from charts_builders import builder_methods

CHART_TYPES = ["line", "area", "scatter", "bar", "candlestick"]
CATALOG = ROOT / "examples/widget_catalog/catalog.rs"
MANUAL = ROOT / "manual/display-widgets.md"


def chart_text() -> str:
    return "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"))


def code_methods() -> set[str]:
    return set(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)", chart_text()))


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
    # Every call in a code span: `Type::method(` is checked on that type,
    # a chain that starts with `Type::new(` or `Type::method(` is checked on
    # that type, and a bare `.method(` span against every chart pub fn.
    by_type = {k.lower(): v for k, v in builder_methods(chart_text()).items()}
    for span in re.findall(r"`([^`]*)`", manual):
        typed = re.findall(r"\b([A-Z][A-Za-z0-9]*)::([a-z_][a-z0-9_]*)\(", span)
        for type_name, method in typed:
            owned = by_type.get(type_name.lower())
            if owned is None:
                problems.append(f"manual cites {type_name}::{method}() but {type_name} is not a chart type")
            elif method not in owned:
                problems.append(f"manual cites {type_name}::{method}() which is not a pub fn on {type_name}")
        chain_owner = by_type.get(typed[0][0].lower()) if typed else None
        for name in re.findall(r"\.([a-z_][a-z0-9_]*)\(", span):
            if chain_owner is not None:
                if name not in chain_owner:
                    problems.append(f"manual cites .{name}() on {typed[0][0]} which has no such pub fn")
            elif name not in methods:
                problems.append(f"manual cites .{name}() which is not a pub fn in chart code")
    return problems


IMAGE_MANUAL = ROOT / "manual/images-and-clipboard.md"
IMAGE_BUILDER = ROOT / "src/builder/specialized.rs"


def image_docs_problems() -> list[str]:
    """The image widget: a catalog page built with `image()`, a manual section
    headed "Image widget", and every builder method that section cites is a
    pub fn on ImageBuilder."""
    problems = []
    catalog = CATALOG.read_text(errors="replace") if CATALOG.exists() else ""
    if not re.search(r"\bimage\(\)", catalog) or "Image" not in catalog:
        problems.append("catalog has no image page")
    manual = IMAGE_MANUAL.read_text(errors="replace") if IMAGE_MANUAL.exists() else ""
    section = re.search(r"^## Image widget\n(.*?)(?=^## )", manual, re.M | re.S)
    if section is None:
        problems.append("manual has no Image widget heading")
        return problems
    builder = IMAGE_BUILDER.read_text(errors="replace") if IMAGE_BUILDER.exists() else ""
    body = re.search(r"impl ImageBuilder\s*\{(.*?)\n\}", builder, re.S)
    methods = set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", body.group(1))) if body else set()
    if not methods:
        problems.append("no ImageBuilder impl found")
    for span in re.findall(r"`([^`]*)`", section.group(1)):
        for name in re.findall(r"\.([a-z_][a-z0-9_]*)\(", span):
            if name not in methods:
                problems.append(f"manual cites .{name}() which is not a pub fn on ImageBuilder")
    return problems


def main() -> int:
    problems = chart_docs_problems() + image_docs_problems()
    if problems:
        print("BAR-006 violated:")
        for p in problems:
            print("  " + p)
        return 1
    print("BAR-006 holds for the chart family and the image widget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
