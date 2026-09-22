#!/usr/bin/env python3
"""CHT-017: chart colors resolve through the layout's color resolver and the
Theme; defaults come from --color-chart-* variables; no hex literal in chart
code.

Prints `cairn: CHT-017: pass|fail`.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules

HEX = re.compile(r'"#[0-9a-fA-F]{3,8}"|0x[0-9a-fA-F]{6}\b|Rgba?::new\(\s*\d|Rgba?\(\s*\d')
CHART_DIRS = ("src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs")
THEME_VARS = ["--color-chart-1", "--color-chart-2", "--color-chart-3", "--color-chart-4", "--color-chart-5",
              "--color-chart-bullish", "--color-chart-bearish"]


def main() -> int:
    problems = []
    for f in rust_sources(*CHART_DIRS):
        if f.name.endswith("tests.rs"):
            continue
        text = strip_test_modules(f.read_text(errors="replace"))
        for m in HEX.finditer(text):
            line = text[: m.start()].count("\n") + 1
            problems.append(f"hex literal {m.group(0)} at {f.relative_to(ROOT)}:{line}")
    theme_text = "\n".join(p.read_text(errors="replace") for p in rust_sources("src/theme"))
    presets = len(re.findall(r"pub fn (?:dark|light|high_contrast|solarized_dark|gruvbox_dark)\b", theme_text)) or 5
    for v in THEME_VARS:
        n = theme_text.count(f'"{v}"')
        if n < presets:
            problems.append(f"{v} defined in {n} of {presets} presets")
    ok, why = cargo_test_filtered("charts_contract", "cht_017_")
    if not ok:
        problems.append("token parity test: " + why.splitlines()[-1] if why else "token parity test failed")
    return finish({"CHT-017": (not problems, "; ".join(problems[:6]) if problems else "no hex literals, theme chart variables present, token parity holds")})


if __name__ == "__main__":
    sys.exit(main())
