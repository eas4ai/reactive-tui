#!/usr/bin/env python3
"""Runs one requirement group of tests/charts_contract.rs and prints
per-requirement lines. Usage: charts_contract.py <group>

groups: interaction (CHT-018, CHT-019), motion (CHT-022),
        frame-budget (CHT-021, BAR-005), widget-bar (BAR-003)

widget-bar covers the widgets this work delivered or reworked: the chart
family (tests/charts_contract.rs) and the image widget, whose block fallback
now draws through the renderer's blitters (tests/api_widget_behavior/image.rs
and its screen-reader unit test in src/widgets/display/image/live.rs).
"""

import re
import sys

from _common import cargo_test_filtered, finish, rust_sources, strip_test_modules

GROUPS = {
    "interaction": [("CHT-018", "cht_018_"), ("CHT-019", "cht_019_")],
    "motion": [("CHT-022", "cht_022_")],
    "frame-budget": [("CHT-021", "cht_021_"), ("BAR-005", "bar_005_")],
    "widget-bar": [("BAR-003", "bar_003_")],
}
NUM = r"\d+(?:\.\d+)?(?:f32|f64)?"
HEX = re.compile(rf'"#[0-9a-fA-F]{{3,8}}"|\(\s*{NUM}\s*,\s*{NUM}\s*,\s*{NUM}\s*,\s*{NUM}\s*\)')


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
        for family, dirs in (("chart", ("src/widgets/display/charts.rs", "src/widgets/display/charts")),
                             ("image", ("src/widgets/display/image",))):
            text = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(*dirs))
            hexes = len(HEX.findall(text))
            if hexes:
                problems.append(f"{hexes} hard-coded colors in {family} code")
        results["BAR-003"] = (not problems, "; ".join(problems) or f"charts and image: {why}")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
