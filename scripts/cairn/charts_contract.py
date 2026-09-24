#!/usr/bin/env python3
"""Runs one requirement group of tests/charts_contract.rs and prints
per-requirement lines. Usage: charts_contract.py <group>

groups: interaction (CHT-018, CHT-019), motion (CHT-022),
        frame-budget (CHT-021, BAR-005), widget-bar (BAR-003)

widget-bar covers the widgets this work delivered or reworked: the chart
family (tests/charts_contract.rs) and the image widget, whose block fallback
now draws through the renderer's blitters (tests/api_widget_behavior/image.rs
and its screen-reader unit test in src/widgets/display/image/live.rs).

Its color check reads the production code of both widgets and their
builders, with comments and test items removed, and reports every color
literal: a hex, rgb() or named-color string, a tuple of three or four
channel values, a six- or eight-digit hex integer, a color type built from
numbers (Color::Rgb(..), Rgba { r: .. }, ColorDefinition::rgb(..),
fg_rgba(..)) and a named constructor (Color::Red, Rgba::white()). A pixel
computed from image data, or the all-zero pixel `[0; 4]`, is not a literal.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, mask, rust_sources, strip_test_modules

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
    "hex string": (re.compile(r'"#[0-9a-fA-F]{3,8}"'), True),
    "rgb() string": (re.compile(r'"\s*(?:rgba?|hsla?)\s*\('), True),
    "named color string": (re.compile(rf'"(?i:{NAMED})"'), True),
    "channel tuple": (re.compile(rf"(?<![\w\]])\(\s*{CHANNEL}\s*,\s*{CHANNEL}\s*,\s*{CHANNEL}\s*(?:,\s*{CHANNEL}\s*)?\)"), False),
    "unit tuple": (re.compile(rf"(?<![\w\]])\(\s*{UNIT}\s*,\s*{UNIT}\s*,\s*{UNIT}\s*(?:,\s*{UNIT}\s*)?\)"), False),
    "hex integer": (re.compile(r"\b0x(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b"), False),
    "color built from numbers": (re.compile(rf"\b{COLOR_TYPE}\s*(?:::\s*\w+\s*)?(?:\(\s*\[?|\{{\s*r\s*:)\s*{NUM}(?!\s*;)"), False),
    "rgb setter": (re.compile(rf"\b(?:\w+_)?rgba?\s*\(\s*{NUM}"), False),
    "named constructor": (re.compile(rf"\b{COLOR_TYPE}\s*::\s*(?i:{NAMED}|dark\w*|light\w*)\b"), False),
}
WIDGET_CODE = {
    "chart": ("src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"),
    "image": ("src/widgets/display/image", "src/builder/widgets/display.rs"),
}


def color_literals(dirs: tuple[str, ...]) -> list[str]:
    """`path:line kind` for every color literal in the production code under `dirs`."""
    found = []
    for f in rust_sources(*dirs):
        text = strip_test_modules(f.read_text(errors="replace"))
        code, prose = mask(text)
        for kind, (pattern, strings) in COLOR_LITERALS.items():
            for m in pattern.finditer(prose if strings else code):
                line = text.count("\n", 0, m.start()) + 1
                found.append(f"{f.relative_to(ROOT)}:{line} {kind} {' '.join(m.group(0).split())}")
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
        for family, dirs in WIDGET_CODE.items():
            literals = color_literals(dirs)
            if literals:
                problems.append(f"{len(literals)} hard-coded colors in {family} code: {', '.join(literals[:4])}")
        results["BAR-003"] = (not problems, "; ".join(problems) or f"charts and image: {why}")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
