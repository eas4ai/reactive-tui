#!/usr/bin/env python3
"""CHT-020 and CHT-029: the chart builders mirror the gpui-kit method names per
chart type, the cartesian types (CHT-020) and the pie, donut, radar and sankey
types each commitment delivers (CHT-029).

Prints `cairn: CHT-020: pass|fail` and `cairn: CHT-029: pass|fail`.
"""

import re
import sys

from _common import finish, rust_sources, strip_test_modules

# method -> chart types that must expose it (the gpui-kit 0.6.6 chart module)
REQUIRED = {
    "x": ["line", "area", "scatter"], "y": ["line", "area", "scatter"], "stroke": ["line", "area"], "fill": ["area", "bar"],
    "natural": ["line", "area"], "linear": ["line", "area"], "step_after": ["line", "area"], "dot": ["line"],
    "tick_margin": ["line", "area", "bar", "candlestick", "scatter"], "grid": ["line", "area", "bar", "candlestick", "scatter"],
    "band": ["bar"], "value": ["bar"], "alignment": ["bar"], "label": ["bar"],
    "open": ["candlestick"], "high": ["candlestick"], "low": ["candlestick"], "close": ["candlestick"],
}
FIRST_CUT = {"line", "area", "bar", "candlestick", "scatter"}
# CHT-029: chart type -> the ChartType variant that marks it delivered, and
# the methods its builder must expose.
PIE = ["value", "label", "color", "inner_radius", "outer_radius", "pad_angle", "label_gap"]
RADIAL = {
    "pie": ("Pie", PIE),
    "donut": ("Donut", PIE),
    "radar": ("Radar", ["value", "label", "stroke", "fill", "dot", "grid", "grid_levels", "max_value", "outer_radius"]),
    "sankey": ("Sankey", ["new", "value_scale", "node_align", "iterations", "node_width", "node_padding",
                          "node_color", "node_label", "value_label", "labels", "link_opacity", "min_link_width",
                          "label_gap"]),
}


def builder_methods(text: str) -> dict[str, set[str]]:
    """Map builder type name -> set of pub method names."""
    found: dict[str, set[str]] = {}
    for m in re.finditer(r"impl(?:<[^>]*>)?\s+([A-Za-z_][A-Za-z0-9_]*)(?:<[^>]*>)?\s*\{", text):
        name = m.group(1)
        depth, j = 1, m.end()
        while j < len(text) and depth:
            depth += text[j] == "{"
            depth -= text[j] == "}"
            j += 1
        body = text[m.end(): j]
        found.setdefault(name, set()).update(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]", body))
    return found


def main() -> int:
    text = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"))
    methods = builder_methods(text)
    lower = {k.lower(): v for k, v in methods.items()}
    missing = []
    for method, kinds in REQUIRED.items():
        for kind in kinds:
            if kind not in FIRST_CUT:
                continue
            # The method must exist on that chart type's own builder
            # (LineChartBuilder for line, ...), not on any builder.
            owner = f"{kind}chartbuilder"
            if owner not in lower:
                missing.append(f"{kind}: no {kind.capitalize()}ChartBuilder type found")
                continue
            if method not in lower[owner]:
                missing.append(f"{kind}: .{method}() missing on {kind.capitalize()}ChartBuilder")
    # CHT-029: a type counts as delivered once ChartType has its variant.
    variants = re.search(r"enum ChartType\s*\{([^}]*)\}", text, re.S)
    delivered = set(re.findall(r"\b([A-Z][A-Za-z]*)\b", variants.group(1))) if variants else set()
    radial_missing = []
    for kind, (variant, required) in RADIAL.items():
        if variant not in delivered:
            continue
        owner = f"{kind}chartbuilder"
        if owner not in lower:
            radial_missing.append(f"{kind}: no {kind.capitalize()}ChartBuilder type found")
            continue
        radial_missing.extend(f"{kind}: .{method}() missing on {kind.capitalize()}ChartBuilder"
                              for method in required if method not in lower[owner])
        # A sankey's links are built with SankeyLink::new(source, target, value).
        if kind == "sankey" and "new" not in lower.get("sankeylink", set()):
            radial_missing.append("sankey: no SankeyLink::new(source, target, value)")
    closures = re.findall(r"Box<dyn Fn|Arc<dyn Fn|Rc<dyn Fn", "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources("src/widgets/display/charts.rs")))
    stored = [f"{len(closures)} closure fields stored in chart props (must be evaluated at build)"] if closures else []
    missing = sorted(set(missing)) + stored
    radial_missing = sorted(set(radial_missing)) + stored

    def reason(problems: list[str], ok: str) -> str:
        return "; ".join(problems[:8]) + (f" (+{len(problems)-8})" if len(problems) > 8 else "") if problems else ok

    return finish({
        "CHT-020": (not missing, reason(missing, "all first-cut methods present")),
        "CHT-029": (not radial_missing, reason(radial_missing, "every delivered radial and flow type has its methods")),
    })


if __name__ == "__main__":
    sys.exit(main())
