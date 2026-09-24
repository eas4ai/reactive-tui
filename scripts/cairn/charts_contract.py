#!/usr/bin/env python3
"""Runs one requirement group of tests/charts_contract.rs and prints
per-requirement lines. Usage: charts_contract.py <group>

groups: interaction (CHT-018, CHT-019), motion (CHT-022),
        frame-budget (CHT-021, BAR-005), widget-bar (BAR-003)

widget-bar covers the widgets this work delivered or reworked: the chart
family (tests/charts_contract.rs) and the image widget, whose block fallback
now draws through the renderer's blitters (tests/api_widget_behavior/image.rs
and its screen-reader unit test in src/widgets/display/image/live.rs).

Its color check reads the production code of both widgets and of their
builders, with comments and test items removed, and reports every color
literal: a hex string with or without `#` (so `u32::from_str_radix("ff0000",
16)` is one), an rgb() or named-color string, a tuple or array of three or
four channel values (a literal pixel such as `[255, 255, 255, 255]`), a six-
or eight-digit hex integer, a color type built from numbers (Color::Rgb(..),
Rgba { a: .., r: .. } in any field order, ColorDefinition::rgb(..),
fg_rgba(..)) and a named constructor (Color::Red, Rgba::white()). The
builder code is found by type, wherever it is under src/builder: every
struct, impl and function whose header names ChartBuilder or ImageBuilder.
A pixel computed from image data, or the all-zero pixel `[0; 4]`, is not a
literal.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, mask, matching, rust_sources, strip_test_modules

GROUPS = {
    "interaction": [("CHT-018", "cht_018_"), ("CHT-019", "cht_019_")],
    "motion": [("CHT-022", "cht_022_")],
    "frame-budget": [("CHT-021", "cht_021_"), ("BAR-005", "bar_005_")],
    "widget-bar": [("BAR-003", "bar_003_")],
}
NAMED = "red|green|blue|black|white|yellow|cyan|magenta|gray|grey|orange|purple|pink|brown|navy|teal|lime|maroon|olive|silver"
CHANNEL = r"(?:25[0-5]|2[0-4]\d|1?\d?\d)(?:_?u8)?"
UNIT = r"(?:0?\.\d+|1\.0*|0\.0*)(?:_?f32|_?f64)?"
NUM = r"-?\d+(?:\.\d+)?(?:_?(?:f32|f64|u8|u16|u32))?"
COLOR_TYPE = r"(?:Rgba?|Colou?r|ColorDefinition|Rgba8|Hsla?)"
# (pattern, whether it reads string contents)
COLOR_LITERALS = {
    "hex string": (re.compile(r'"(?:#[0-9a-fA-F]{3,8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})"'), True),
    "rgb() string": (re.compile(r'"\s*(?:rgba?|hsla?)\s*\('), True),
    "named color string": (re.compile(rf'"(?i:{NAMED})"'), True),
    "channel tuple": (re.compile(rf"(?<![\w\]])\(\s*{CHANNEL}\s*,\s*{CHANNEL}\s*,\s*{CHANNEL}\s*(?:,\s*{CHANNEL}\s*)?\)"), False),
    "channel array": (re.compile(rf"(?<![\w\])])\[\s*{CHANNEL}\s*,\s*{CHANNEL}\s*,\s*{CHANNEL}\s*(?:,\s*{CHANNEL}\s*)?,?\s*\]"), False),
    "unit tuple": (re.compile(rf"(?<![\w\]])\(\s*{UNIT}\s*,\s*{UNIT}\s*,\s*{UNIT}\s*(?:,\s*{UNIT}\s*)?\)"), False),
    "hex integer": (re.compile(r"\b0x(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b"), False),
    "color built from numbers": (re.compile(rf"\b{COLOR_TYPE}\s*(?:::\s*\w+\s*)?(?:\(\s*\[?|\{{\s*(?:r|g|b|a|red|green|blue|alpha)\s*:)\s*{NUM}(?!\s*;)"), False),
    "rgb setter": (re.compile(rf"\b(?:\w+_)?rgba?\s*\(\s*{NUM}"), False),
    "named constructor": (re.compile(rf"\b{COLOR_TYPE}\s*::\s*(?i:{NAMED}|dark\w*|light\w*)\b"), False),
}
# Each family: the directories and files of its widget code, read whole, and
# the builder types whose items are read wherever they are under BUILDERS.
WIDGET_CODE = {
    "chart": (("src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"),
              r"\w*ChartBuilder"),
    "image": (("src/widgets/display/image", "src/builder/widgets/display.rs"), r"ImageBuilder"),
}
BUILDERS = "src/builder"


def builder_spans(code: str, types: str) -> list[tuple[int, int]]:
    """The struct, enum, impl and fn items of `code` whose header names one of
    `types` (a struct's name, an impl's self type or trait argument, a
    function's parameter or return type)."""
    head = re.compile(rf"\b(?:(?:struct|enum)\s+(?:{types})\b|(?:impl|fn)\b[^{{;]*\b(?:{types})\b)[^{{;]*\{{")
    return [(m.start(), matching(code, m.end() - 1)) for m in head.finditer(code)]


def literals_in(path, text: str, spans: list[tuple[int, int]] | None = None) -> list[str]:
    code, prose = mask(text)
    found = []
    for kind, (pattern, strings) in COLOR_LITERALS.items():
        for m in pattern.finditer(prose if strings else code):
            if spans is not None and not any(a <= m.start() < b for a, b in spans):
                continue
            line = text.count("\n", 0, m.start()) + 1
            found.append(f"{path.relative_to(ROOT)}:{line} {kind} {' '.join(m.group(0).split())}")
    return found


def color_literals(family: tuple[tuple[str, ...], str]) -> list[str]:
    """`path:line kind` for every color literal in a family's production code:
    its files, read whole, and its builder items under src/builder."""
    dirs, types = family
    whole = rust_sources(*dirs)
    found = []
    for f in whole:
        found += literals_in(f, strip_test_modules(f.read_text(errors="replace")))
    for f in rust_sources(BUILDERS):
        if f in whole:
            continue
        text = strip_test_modules(f.read_text(errors="replace"))
        spans = builder_spans(mask(text)[0], types)
        if spans:
            found += literals_in(f, text, spans)
    return found


def main() -> int:
    group = sys.argv[1] if len(sys.argv) > 1 else ""
    if group not in GROUPS:
        print(__doc__)
        return 2
    # The frame budget is a property of the optimized build; the other
    # groups observe behavior and run the default profile.
    release = group == "frame-budget"
    results = {req: cargo_test_filtered("charts_contract", sub, release=release) for req, sub in GROUPS[group]}
    if group == "widget-bar":
        ok, why = results["BAR-003"]
        problems = [] if ok else [f"charts: {why}"]
        for name, (passed, reason) in (
            ("image", cargo_test_filtered("api_widget_behavior", "bar_003_")),
            ("image screen reader", cargo_test_filtered(None, "bar_003_", package="reactive-tui")),
        ):
            if not passed:
                problems.append(f"{name}: {reason}")
        for family, code in WIDGET_CODE.items():
            literals = color_literals(code)
            if literals:
                problems.append(f"{len(literals)} hard-coded colors in {family} code: {', '.join(literals[:4])}")
        results["BAR-003"] = (not problems, "; ".join(problems) or f"charts and image: {why}")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
