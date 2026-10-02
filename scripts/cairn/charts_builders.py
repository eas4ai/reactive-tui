#!/usr/bin/env python3
"""CHT-020, CHT-029 and CHT-035: the chart builders mirror the gpui-kit method
names per chart type, the cartesian types against gpui-kit 0.7.0 (CHT-020)
and the pie, donut, radar and sankey types each commitment delivers
(CHT-029); and every chart type is buildable through `ChartsBuilder`,
`builder::widgets::chart` (ChartBuilder), its typed builder and the `chart!`
macro, with every chart-level option of a typed builder on the two generic
builders too (CHT-035). Data accessors (x, y, band, value, open, high, low,
close) and per-series or per-point settings (name, stroke, fill, label,
colors, tooltip closures) are carried by DataSeries and DataPoint, which both
generic routes take, so they are not counted as chart-level options.

Prints `cairn: CHT-020: pass|fail`, `cairn: CHT-029: pass|fail` and
`cairn: CHT-035: pass|fail`.
"""

import re
import sys

from _common import ROOT, finish, rust_sources, strip_test_modules

CARTESIAN = ["line", "area", "scatter", "bar", "candlestick"]
LINE_AXES = ["y_domain", "y_axis", "y_axis_label_placement", "y_tick_count", "y_tick_format", "x_tick_count",
             "grid_columns", "reference_line", "y_padding"]
# method -> chart types that must expose it (the gpui-kit 0.7.0 chart module,
# CHT-020's list)
REQUIRED = {
    **{m: list(CARTESIAN) for m in ("name", "interactive", "tooltip_title", "tooltip_value", "tooltip_value_color",
                                    "tooltip_content", "tick_margin", "grid", "grid_dashed", "x_axis")},
    "x": ["line", "area", "scatter", "candlestick"], "y": ["line", "area", "scatter"],
    "stroke": ["line", "area", "scatter"], "fill": ["area", "bar"],
    "natural": ["line", "area"], "linear": ["line", "area"], "step_after": ["line", "area"], "dot": ["line", "area"],
    "point_count": ["line", "area"], "stacked": ["area", "bar"],
    **{m: ["line", "area", "scatter"] for m in LINE_AXES},
    "band": ["bar"], "value": ["bar"], "fill_gradient": ["bar"], "label": ["bar"], "label_color": ["bar"],
    "label_axis": ["bar"], "value_axis": ["bar"], "value_tick_count": ["bar"], "value_axis_label_placement": ["bar"],
    "value_tick_format": ["bar"], "band_count": ["bar"], "band_tick_count": ["bar"], "alignment": ["bar"],
    "padding_inner": ["bar"], "padding_outer": ["bar"], "max_band_width": ["bar", "candlestick"], "min_length": ["bar"],
    "open": ["candlestick"], "high": ["candlestick"], "low": ["candlestick"], "close": ["candlestick"],
    "body_width_ratio": ["candlestick"], "bullish": ["candlestick"], "bearish": ["candlestick"],
}
FIRST_CUT = set(CARTESIAN)
# CHT-035: ChartType variant -> the `chart!` keyword and the ChartBuilder
# constructor names accepted for it.
ROUTES = {
    "BarVertical": ("bar", ["bar_chart", "bar"]),
    "BarHorizontal": ("horizontal_bar", ["horizontal_bar_chart", "bar_horizontal_chart", "horizontal_bar", "bar_horizontal"]),
    "Line": ("line", ["line_chart", "line"]),
    "Area": ("area", ["area_chart", "area"]),
    "Scatter": ("scatter", ["scatter_chart", "scatter_plot", "scatter"]),
    "Candlestick": ("candlestick", ["candlestick_chart", "candlestick"]),
    "Pie": ("pie", ["pie_chart", "pie"]),
    "Donut": ("donut", ["donut_chart", "donut"]),
    "Radar": ("radar", ["radar_chart", "radar"]),
    "Sankey": ("sankey", ["sankey_chart", "sankey"]),
}
# Typed-builder methods that read the datum into the points (the generic
# routes take finished DataPoints instead), plus the constructor and the
# builders' own build/render. Every other typed method is an option CHT-035
# wants on both generic builders, per-point ones (tooltip_title, label_color,
# fill_gradient, ...) over `&DataPoint`.
NOT_OPTIONS = {"new", "build", "render", "x", "y", "band", "value", "open", "high", "low", "close", "label"}
# A typed option whose generic-builder method has another name.
ALIASES = {"alignment": ["growth", "alignment"], "dot": ["dots", "dot"], "natural": ["curve"], "linear": ["curve"],
           "step_after": ["curve"], "interactive": ["show_tooltips", "interactive"]}
TYPED_BUILDERS = [f"{kind}chartbuilder" for kind in CARTESIAN]
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


def block_end(text: str, start: int) -> int:
    """The index just past the brace block that opens before `start`."""
    depth, j = 1, start
    while j < len(text) and depth:
        depth += text[j] == "{"
        depth -= text[j] == "}"
        j += 1
    return j


def macro_methods(text: str) -> dict[str, set[str]]:
    """Map `macro_rules!` name -> the pub method names its body defines, for
    the method-set macros the typed builders share (`common_methods!()`)."""
    found: dict[str, set[str]] = {}
    for m in re.finditer(r"macro_rules!\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", text):
        body = text[m.end(): block_end(text, m.end())]
        found[m.group(1)] = set(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]", body))
    return found


def builder_methods(text: str) -> dict[str, set[str]]:
    """Map builder type name -> set of pub method names, counting the methods
    a macro invocation in the impl body generates: a shared method-set macro
    (`common_methods!();`) by its definition, and a forwarding list
    (`forward! { name(args); ... }`) by the names it lists."""
    macros = macro_methods(text)
    found: dict[str, set[str]] = {}
    for m in re.finditer(r"impl(?:<[^>]*>)?\s+([A-Za-z_][A-Za-z0-9_]*)(?:<[^>]*>)?\s*\{", text):
        name = m.group(1)
        body = text[m.end(): block_end(text, m.end())]
        methods = found.setdefault(name, set())
        methods.update(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]", body))
        for call in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)!\s*(\(\s*\)\s*;|\{)", body):
            macro = call.group(1)
            if call.group(2) == "{":
                listed = body[call.end(): block_end(body, call.end())]
                methods.update(re.findall(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(", listed, re.M))
            else:
                methods.update(macros.get(macro, set()))
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
    # The props and their parts hold evaluated data, never a closure; a
    # builder may hold one until `build()` runs it.
    props_text = "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources("src/widgets/display/charts.rs"))
    closures = []
    for name in ("ChartProps", "ChartAxis", "ChartLegend", "DataPoint", "DataSeries", "PointTooltip", "RadialOptions", "SankeyOptions"):
        m = re.search(rf"pub struct {name}\s*\{{(.*?)\n\}}", props_text, re.S)
        if m:
            closures += re.findall(r"Box<dyn Fn|Arc<dyn Fn|Rc<dyn Fn", m.group(1))
    stored = [f"{len(closures)} closure fields stored in chart props (must be evaluated at build)"] if closures else []
    missing = sorted(set(missing)) + stored
    radial_missing = sorted(set(radial_missing)) + stored
    # CHT-035: every delivered type through every route, every chart-level
    # typed option on both generic builders.
    routes = []
    macros = strip_test_modules((ROOT / "src/builder/macros.rs").read_text(errors="replace"))
    chart_macro = re.search(r"macro_rules!\s+chart\s*\{(.*?)\n\}", macros, re.S)
    macro_body = chart_macro.group(1) if chart_macro else ""
    generic = {"ChartsBuilder": lower.get("chartsbuilder", set()), "builder::widgets::chart": lower.get("chartbuilder", set())}
    for variant, (keyword, constructors) in ROUTES.items():
        if variant not in delivered:
            continue
        if not re.search(rf"\[\s*{keyword}\s*:", macro_body):
            routes.append(f"chart! has no [{keyword}: ...] form")
        if not any(c in generic["ChartsBuilder"] for c in constructors) and "chart_type" not in generic["ChartsBuilder"]:
            routes.append(f"ChartsBuilder has no {keyword} constructor")
        if not any(c in generic["builder::widgets::chart"] for c in constructors):
            routes.append(f"builder::widgets::chart has no {keyword} constructor")
    options = set()
    for owner in TYPED_BUILDERS:
        options |= lower.get(owner, set())
    for option in sorted(options - NOT_OPTIONS):
        names = ALIASES.get(option, [option])
        for route, methods in generic.items():
            if not any(n in methods for n in names):
                routes.append(f"{route} has no .{option}()")

    def reason(problems: list[str], ok: str) -> str:
        return "; ".join(problems[:8]) + (f" (+{len(problems)-8})" if len(problems) > 8 else "") if problems else ok

    return finish({
        "CHT-020": (not missing, reason(missing, "every reference method present per type")),
        "CHT-029": (not radial_missing, reason(radial_missing, "every delivered radial and flow type has its methods")),
        "CHT-035": (not routes, reason(routes, "every type through every route, every typed option on both generic builders")),
    })


if __name__ == "__main__":
    sys.exit(main())
