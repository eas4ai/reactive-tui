#!/usr/bin/env python3
"""CHT-020: the chart builder mirrors the gpui-kit method names per chart type.

Prints `cairn: CHT-020: pass|fail`.
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
            owners = [n for n in lower if n.endswith("builder") and n not in ("chartaxis", "chartprops", "chartlegend")]
            if not owners:
                missing.append(f"no chart builder type found")
                continue
            if not any(method in lower[o] for o in owners):
                missing.append(f"{kind}: .{method}() missing")
    closures = re.findall(r"Box<dyn Fn|Arc<dyn Fn|Rc<dyn Fn", "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources("src/widgets/display/charts.rs")))
    if closures:
        missing.append(f"{len(closures)} closure fields stored in chart props (must be evaluated at build)")
    missing = sorted(set(missing))
    return finish({"CHT-020": (not missing, "; ".join(missing[:8]) + (f" (+{len(missing)-8})" if len(missing) > 8 else "") if missing else "all first-cut methods present")})


if __name__ == "__main__":
    sys.exit(main())
