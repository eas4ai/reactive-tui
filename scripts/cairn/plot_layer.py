#!/usr/bin/env python3
"""CHT-010 and CHT-011: a public plot layer exists and every chart maps data
through it; band padding, zero inclusion and zero baseline hold.

Prints one `cairn: CHT-01x: pass|fail` line per requirement.
"""

import re
import sys

from _common import ROOT, cargo_test_filtered, finish, rust_sources, strip_test_modules

PLOT_DIRS = ["src/widgets/display/charts/plot", "src/plot"]
REQUIRED_ITEMS = ["ScaleLinear", "ScaleBand", "ScalePoint", "ScaleOrdinal", "Tick", "Axis", "Grid", "Legend", "Tooltip"]
RENDERERS = ["src/widgets/display/charts/live"]
LOCAL_SCALE = re.compile(r"\(\s*max\w*\s*-\s*min\w*\s*\)|/\s*range\b|\*\s*\(?\s*\w*height\w*\s*-\s*1\s*\)?\s*/")


def main() -> int:
    plot = next((ROOT / d for d in PLOT_DIRS if (ROOT / d).is_dir()), None)
    problems_010, problems_011 = [], []
    if plot is None:
        problems_010.append("no plot layer directory (" + " or ".join(PLOT_DIRS) + ")")
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
        for m in LOCAL_SCALE.finditer(text):
            line = text[: m.start()].count("\n") + 1
            problems_010.append(f"local scale arithmetic at {f.relative_to(ROOT)}:{line}")
    ok, why = cargo_test_filtered("plot_layer_contract", "cht_011_")
    if not ok:
        problems_011.append(why.splitlines()[-1] if why else "scale unit tests failed")
    return finish({
        "CHT-010": (not problems_010, "; ".join(problems_010[:5]) if problems_010 else "plot layer present and used by every renderer"),
        "CHT-011": (not problems_011, "; ".join(problems_011[:5]) if problems_011 else "band padding, zero inclusion and zero baseline hold"),
    })


if __name__ == "__main__":
    sys.exit(main())
