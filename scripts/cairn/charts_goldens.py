#!/usr/bin/env python3
"""charts-goldens: CHT-012, CHT-013, CHT-014, CHT-015, CHT-016, CHT-023, CHT-024, CHT-025,
CHT-026, CHT-027, CHT-028, CHT-030 and BAR-004 through the tests/charts_goldens.rs binary plus file
and doc probes.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import re
import shutil
import sys
import tempfile
import unicodedata
from pathlib import Path

from _common import (ROOT, cargo_test_filtered, enclosing_block, finish, mask, matching, rust_sources,
                     statement_start, strip_test_modules)
from catalog_manual import chart_docs_problems

SNAP = ROOT / "tests/snapshots/charts"
TYPES = ["line", "area", "scatter", "bar", "candlestick", "pie", "donut", "radar", "sankey"]
SIZES = {"mini": (20, 5), "medium": (80, 24), "large": (600, 160)}
# The image widget, reworked to draw its block fallback through the
# blitters: a medium golden and a wide one of at least 400 columns.
IMAGE_GOLDENS = ("image_medium", "image_wide")


GOLDEN_TESTS = ("tests/charts_goldens.rs", "tests/api_widget_behavior/image.rs")
# A write of a golden file, and the one condition allowed to guard it.
WRITE = re.compile(r"\b(?:fs::(?:write|copy|rename|hard_link)|File::(?:create|create_new|options)|OpenOptions::new)\b"
                   r"|\.persist(?:_noclobber)?\b")
GUARD = re.compile(r'^\s*if\s+(?:std::)?env::var\(\s*"REGENERATE"\s*\)\s*\.as_deref\(\)\s*==\s*Ok\(\s*"1"\s*\)\s*$')
ASSERTS = re.compile(r"\bassert(?:_eq|_ne)?!|\bpanic!")
FN = re.compile(r"\bfn\s+\w+")


def regeneration_problems(src: str) -> list[str]:
    """BAR-004: the golden test in `src` compares unless REGENERATE=1 is
    set. Read as code, with comments removed, every write of a file sits in
    a block guarded by exactly `if std::env::var("REGENERATE").as_deref() ==
    Ok("1")`, and the function that writes also asserts outside that block.
    A guard quoted in a comment, or a condition that adds `|| true`, does
    not count."""
    path = ROOT / src
    if not path.is_file():
        return [f"no golden test file {src}"]
    text = path.read_text(errors="replace")
    code, prose = mask(text)
    problems = []
    writes = list(WRITE.finditer(code))
    if not writes:
        return [f"{src} writes no golden, so REGENERATE=1 cannot refresh one"]
    for write in writes:
        line = code.count("\n", 0, write.start()) + 1
        block = enclosing_block(code, write.start())
        start = statement_start(code, block)
        if not GUARD.match(prose[start:block]):
            problems.append(f"{src}:{line} writes a golden outside a REGENERATE == \"1\" guard")
            continue
        fn = [m for m in FN.finditer(code, 0, start)]
        opening = code.find("{", fn[-1].end()) if fn else -1
        if opening < 0:
            problems.append(f"{src}:{line} writes a golden outside a function")
            continue
        outside = code[opening:start] + code[matching(code, block):matching(code, opening)]
        if not ASSERTS.search(outside):
            problems.append(f"{src}:{line} regenerates in a function that compares nothing")
    return problems


SNAPSHOTS = ROOT / "tests/snapshots"
# The variable both golden tests read their snapshot directory from.
SNAPSHOTS_ENV = "REACTIVE_TUI_SNAPSHOTS"
# (test binary, test name prefix, snapshot family): the test that compares
# every golden of the family.
GOLDEN_RUNS = (("charts_goldens", "cht_023_", "charts"), ("api_widget_behavior", "bar_004_", "image"))


def checked_in() -> dict[Path, bytes]:
    """Every file of the golden families as it is now."""
    return {p: p.read_bytes() for family in ("charts", "image") for p in (SNAPSHOTS / family).rglob("*") if p.is_file()}


def alter(golden: bytes) -> bytes:
    """`golden` with the last digit of its color digest changed."""
    at = golden.rstrip(b"\n").rfind(b"colors: ")
    if at < 0:
        raise ValueError("golden has no color digest")
    end = len(golden.rstrip(b"\n")) - 1
    flipped = b"0" if golden[end:end + 1] != b"0" else b"1"
    return golden[:end] + flipped + golden[end + 1:]


def altered_golden_problems(before: dict[Path, bytes]) -> list[str]:
    """BAR-004, by behavior: each golden test compares its frame with every
    golden and never rewrites one, whatever call it writes with. For each
    golden of a family in turn, the family as it was before any test ran is
    copied to a scratch directory and that one golden is changed there; the
    test, pointed at the copy, must fail naming it and leave every file of
    the copy as it was."""
    problems = []
    for binary, test, family in GOLDEN_RUNS:
        files = {p.relative_to(SNAPSHOTS / family): data for p, data in before.items()
                 if p.is_relative_to(SNAPSHOTS / family)}
        names = sorted(str(r)[:-len(".ansi")] for r in files if r.suffix == ".ansi")
        if not names:
            problems.append(f"no {family} goldens to compare")
        for name in names:
            with tempfile.TemporaryDirectory(prefix="goldens-") as scratch:
                copy = Path(scratch) / family
                for rel, data in files.items():
                    (copy / rel).parent.mkdir(parents=True, exist_ok=True)
                    (copy / rel).write_bytes(data)
                golden = copy / f"{name}.ansi"
                golden.write_bytes(alter(golden.read_bytes()))
                expected = {p: p.read_bytes() for p in copy.rglob("*") if p.is_file()}
                ok, why = cargo_test_filtered(binary, test, env={SNAPSHOTS_ENV: scratch})
                if ok:
                    problems.append(f"{binary} {test} passed with {family}/{name}.ansi changed, so it does not compare it")
                elif f"golden mismatch for {name}" not in why:
                    problems.append(f"{binary} {test} failed for another reason than the changed {name}.ansi: {why[:200]}")
                now = {p: p.read_bytes() for p in copy.rglob("*") if p.is_file()}
                rewritten = sorted(str(p.relative_to(copy)) for p in expected.keys() | now.keys()
                                   if expected.get(p) != now.get(p))
                if rewritten:
                    problems.append(f"{binary} {test} rewrote {family}/{', '.join(rewritten[:4])} in the copy "
                                    f"instead of comparing (changed: {name}.ansi)")
            if problems:
                return problems
    return problems


def golden_problems() -> list[str]:
    problems = []
    for kind in TYPES:
        for cls in SIZES:
            if not (SNAP / f"{kind}_{cls}.ansi").is_file():
                problems.append(f"missing golden {kind}_{cls}.ansi")
    problems.extend(regeneration_problems("tests/charts_goldens.rs"))
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
    problems.extend(regeneration_problems("tests/api_widget_behavior/image.rs"))
    return problems


def main() -> int:
    # The goldens as checked in, before any test runs: a test that rewrites
    # them must not leave them looking unchanged to a later snapshot.
    before = checked_in()
    results = {}
    chart_src = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts"))
    for req, sub in (("CHT-012", "cht_012_"), ("CHT-013", "cht_013_"), ("CHT-024", "cht_024_"), ("CHT-026", "cht_026_")):
        results[req] = cargo_test_filtered("charts_goldens", sub)
    # CHT-025: fill cells resolve through the renderer's blitters, so the
    # chart code must call the blitter's two-color split.
    if not re.search(r"\bblit_block\b", chart_src):
        results["CHT-025"] = (False, "the chart canvas resolves no fill cell through the renderer's blitters "
                                     "(no blit_block call in chart code)")
    else:
        results["CHT-025"] = cargo_test_filtered("charts_goldens", "cht_025_")
    # CHT-015: the pie and donut expose inner and outer radius and a pad angle.
    absent = [name for name in ("inner_radius", "outer_radius", "pad_angle") if not re.search(rf"\b{name}\b", chart_src)]
    if absent:
        results["CHT-015"] = (False, "the pie and donut code exposes no " + ", ".join(absent))
    else:
        results["CHT-015"] = cargo_test_filtered("charts_goldens", "cht_015_")
    if not re.search(r"enum ChartType\s*\{[^}]*\bRadar\b", chart_src, re.S):
        results["CHT-016"] = (False, "no radar chart type in chart code")
    else:
        results["CHT-016"] = cargo_test_filtered("charts_goldens", "cht_016_")
    if not re.search(r"enum ChartType\s*\{[^}]*\bSankey\b", chart_src, re.S):
        results["CHT-030"] = (False, "no sankey chart type in chart code")
    else:
        results["CHT-030"] = cargo_test_filtered("charts_goldens", "cht_030_")
    # The decimation cost ratio is a property of the optimized build.
    results["CHT-027"] = cargo_test_filtered("charts_goldens", "cht_027_", release=True)
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
    if ok_023 and ok_image:
        problems_004 += altered_golden_problems(before)
    after = checked_in()
    rewritten = sorted(str(p.relative_to(ROOT)) for p in before.keys() | after.keys() if before.get(p) != after.get(p))
    if rewritten:
        problems_004.append(f"a test run changed checked-in goldens: {', '.join(rewritten[:4])}")
    results["BAR-004"] = (not problems_004, "; ".join(problems_004[:4]) or "chart and image goldens on the debug backend compare equal; wide ones 400+ columns")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
