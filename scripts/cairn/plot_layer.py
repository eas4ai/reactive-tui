#!/usr/bin/env python3
"""CHT-010 and CHT-011: a public plot layer exists and every chart maps data
through it; band padding, zero inclusion and zero baseline hold.

Prints one `cairn: CHT-01x: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules

PLOT_DIRS = ["src/widgets/display/charts/plot"]
REQUIRED_ITEMS = ["ScaleLinear", "ScaleBand", "ScalePoint", "ScaleOrdinal", "Tick", "Axis", "Grid", "Legend", "Tooltip"]
RENDERERS = ["src/widgets/display/charts/live"]
LOCAL_SCALE = re.compile(
    r"\)\s*/\s*\(|/\s*\(\s*\w+\.1\s*-\s*\w+\.0\s*\)|/\s*range\b|/\s*span\b|\bmapped\s*\("
    r"|\*\s*\w*(?:per_unit|per_cell|per_dot|per_value|step_dots|inv_span|scale_factor)\b|\.mul_add\("
    # A radial renderer dividing the circle by a category count itself.
    r"|\bTAU\s*/")


def main() -> int:
    plot = next((ROOT / d for d in PLOT_DIRS if (ROOT / d).is_dir()), None)
    problems_010, problems_011 = [], []
    if plot is None:
        problems_010.append("no plot layer directory (" + ", ".join(PLOT_DIRS) + ")")
        problems_011.append("no ScaleBand with padding_inner/padding_outer")
    else:
        text = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in plot.rglob("*.rs"))
        missing = [i for i in REQUIRED_ITEMS if not re.search(rf"\bpub\s+(?:struct|enum|trait)\s+{i}\b", text)]
        if missing:
            problems_010.append("plot layer lacks pub " + ", ".join(missing))
        if not ("padding_inner" in text and "padding_outer" in text):
            problems_011.append("ScaleBand lacks padding_inner/padding_outer")
    for f in rust_sources(*RENDERERS):
        text = strip_test_modules(f.read_text(errors="replace"))
        if plot and str(plot) in str(f):
            continue
        if plot and "plot::" not in text and "plot/" not in str(f) and f.name in ("cartesian.rs", "pie.rs", "radar.rs", "sankey.rs", "canvas.rs"):
            problems_010.append(f"{f.relative_to(ROOT)} does not use the plot layer")
        for m in LOCAL_SCALE.finditer(text):
            line = text[: m.start()].count("\n") + 1
            problems_010.append(f"local scale arithmetic at {f.relative_to(ROOT)}:{line}")
    ok, why = cargo_test_filtered("charts_contract", "cht_011_")
    if not ok:
        problems_011.append(why.splitlines()[-1] if why else "scale unit tests failed")
    return finish({
        "CHT-010": (not problems_010, "; ".join(problems_010[:5]) if problems_010 else "plot layer present and used by every renderer"),
        "CHT-011": (not problems_011, "; ".join(problems_011[:5]) if problems_011 else "band padding, zero inclusion and zero baseline hold"),
    })


if __name__ == "__main__":
    sys.exit(main())
