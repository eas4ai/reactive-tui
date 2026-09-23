#!/usr/bin/env python3
"""charts-goldens: CHT-012, CHT-013, CHT-014, CHT-023, CHT-024, CHT-025, CHT-026, CHT-027,
CHT-028 and BAR-004 through the tests/charts_goldens.rs binary plus file and doc probes.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys
import unicodedata

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules
from catalog_manual import chart_docs_problems

SNAP = ROOT / "tests/snapshots/charts"
TYPES = ["line", "area", "scatter", "bar", "candlestick"]
SIZES = {"mini": (20, 5), "medium": (80, 24), "large": (600, 160)}
# The image widget, reworked to draw its block fallback through the
# blitters: a medium golden and a wide one of at least 400 columns.
IMAGE_GOLDENS = ("image_medium", "image_wide")


def golden_problems() -> list[str]:
    problems = []
    for kind in TYPES:
        for cls in SIZES:
            if not (SNAP / f"{kind}_{cls}.ansi").is_file():
                problems.append(f"missing golden {kind}_{cls}.ansi")
    src = (ROOT / "tests/charts_goldens.rs")
    if src.is_file() and re.search(r'REGENERATE.*==\s*Ok\("1"\)', src.read_text()) is None:
        problems.append("golden test lacks the REGENERATE=1 opt-in guard")
    return problems


def display_width(line: str) -> int:
    """Terminal columns of one golden row: wide characters take two, marks none."""
    return sum(0 if unicodedata.combining(ch) else 2 if unicodedata.east_asian_width(ch) in "WF" else 1
               for ch in line)


def wide_problems() -> list[str]:
    """BAR-004: each chart type's and the image widget's wide golden, measured
    from its text grid, is at least 400 columns, and every golden test renders
    on the debug backend and regenerates only under REGENERATE=1."""
    problems = []
    for kind in TYPES:
        path = SNAP / f"{kind}_large.ansi"
        if not path.is_file():
            problems.append(f"no wide golden for {kind}")
            continue
        grid = path.read_text(errors="replace").rsplit("\ncolors: ", 1)[0]
        columns = max((display_width(row) for row in grid.split("\n")), default=0)
        if columns < 400:
            problems.append(f"{kind}_large.ansi is {columns} columns wide, under 400")
    for name in IMAGE_GOLDENS:
        path = ROOT / "tests/snapshots/image" / f"{name}.ansi"
        if not path.is_file():
            problems.append(f"missing image golden {name}.ansi")
    wide = ROOT / "tests/snapshots/image/image_wide.ansi"
    if wide.is_file():
        grid = wide.read_text(errors="replace").rsplit("\ncolors: ", 1)[0]
        columns = max((display_width(row) for row in grid.split("\n")), default=0)
        if columns < 400:
            problems.append(f"image_wide.ansi is {columns} columns wide, under 400")
    for src, test in (("tests/charts_goldens.rs", "cht_023_"),
                      ("tests/api_widget_behavior/image.rs", "bar_004_")):
        path = ROOT / src
        text = path.read_text(errors="replace") if path.is_file() else ""
        body = re.search(rf"fn {test}\w*\(\)\s*\{{(.*?)\n\}}", text, re.S)
        if body is None:
            problems.append(f"no {test} golden test in {src}")
        elif "on_debug(" not in body.group(1):
            problems.append(f"the {test} golden test does not render on the debug backend")
        elif not re.search(r'REGENERATE.*==\s*Ok\("1"\)', body.group(1) + text):
            problems.append(f"the {test} golden test lacks the REGENERATE=1 opt-in guard")
    return problems


def main() -> int:
    results = {}
    for req, sub in (("CHT-012", "cht_012_"), ("CHT-013", "cht_013_"), ("CHT-024", "cht_024_"), ("CHT-025", "cht_025_"), ("CHT-026", "cht_026_")):
        results[req] = cargo_test_filtered("charts_goldens", sub)
    # The decimation cost ratio is a property of the optimized build.
    results["CHT-027"] = cargo_test_filtered("charts_goldens", "cht_027_", release=True)
    chart_src = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts"))
    if not re.search(r"enum ChartType\s*\{[^}]*\bCandlestick\b", chart_src, re.S):
        results["CHT-014"] = (False, "no candlestick chart type in chart code")
    else:
        results["CHT-014"] = cargo_test_filtered("charts_goldens", "cht_014_")
    if re.search(r"\b(GlyphSet|ascii_fallback|Ascii)\b", chart_src):
        results["CHT-028"] = cargo_test_filtered("charts_goldens", "cht_028_")
    else:
        results["CHT-028"] = (False, "no ASCII glyph fallback in the chart code")
    g = golden_problems()
    d = chart_docs_problems()
    ok_023, why_023 = cargo_test_filtered("charts_goldens", "cht_023_")
    results["CHT-023"] = (ok_023 and not g and not d, "; ".join((g + d)[:6]) or why_023)
    # BAR-004 holds only when every golden compares equal (the cht_023_ test,
    # which regenerates only under REGENERATE=1), none is missing, and each
    # type's wide golden is at least 400 columns on the debug backend.
    w = wide_problems()
    ok_image, why_image = cargo_test_filtered("api_widget_behavior", "bar_004_")
    problems_004 = g + w + ([] if ok_023 else [f"golden comparison failed: {why_023}"]) + (
        [] if ok_image else [f"image golden comparison failed: {why_image}"])
    results["BAR-004"] = (not problems_004, "; ".join(problems_004[:4]) or "chart and image goldens on the debug backend compare equal; wide ones 400+ columns")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
