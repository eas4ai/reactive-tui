#!/usr/bin/env python3
"""charts-goldens: CHT-012, CHT-013, CHT-014, CHT-023, CHT-024, CHT-025, CHT-026, CHT-027,
CHT-028 and BAR-004 through the tests/charts_goldens.rs binary plus file and doc probes.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules
from catalog_manual import chart_docs_problems

SNAP = ROOT / "tests/snapshots/charts"
TYPES = ["line", "area", "scatter", "bar", "candlestick"]
SIZES = {"mini": (20, 5), "medium": (80, 24), "large": (600, 160)}


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


def main() -> int:
    results = {}
    for req, sub in (("CHT-012", "cht_012_"), ("CHT-013", "cht_013_"), ("CHT-024", "cht_024_"), ("CHT-025", "cht_025_"), ("CHT-026", "cht_026_"), ("CHT-027", "cht_027_")):
        results[req] = cargo_test_filtered("charts_goldens", sub)
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
    wide = [p for p in SNAP.glob("*_large.ansi")] if SNAP.is_dir() else []
    results["BAR-004"] = (bool(wide) and not g, "no checked-in wide golden" if not wide else ("; ".join(g[:4]) or "goldens at two sizes, wide one 600 columns"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
