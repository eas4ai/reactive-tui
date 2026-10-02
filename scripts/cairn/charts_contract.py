#!/usr/bin/env python3
"""Runs one requirement group of tests/charts_contract.rs and prints
per-requirement lines. Usage: charts_contract.py <group>

groups: interaction (CHT-018, CHT-019, CHT-031, CHT-032, CHT-036), motion (CHT-022),
        frame-budget (CHT-021, BAR-005), widget-bar (BAR-003)

frame-budget runs on the performance cores only, where the kernel names
them (/sys/devices/cpu_core/cpus on a hybrid CPU), so another program's
build pushing the test onto the slower efficiency cores does not decide the
16.6 ms budget; on a CPU with one core type nothing changes (developer's
answer to escalation 5c777a6a).

widget-bar covers the widgets this work delivered or reworked: the chart
family (tests/charts_contract.rs), the image widget, whose block fallback
now draws through the renderer's blitters (tests/api_widget_behavior/image.rs
and its screen-reader unit test in src/widgets/display/image/live.rs), the
graphics canvas (tests/canvas_widget.rs, built with wgpu-graphics), the
menu family (tests/menus_contract.rs), the overlay family: the modal,
the popover, the toast and the five dialogs (tests/overlays_contract.rs),
the input family: the text input, checkbox, radio button, select,
slider and button (tests/input_widgets_contract.rs), and the layout
family: the tabs, accordion, breadcrumb, scroll view and stack
(tests/layout_widgets_contract.rs), and the data family: the table, data
table, tree, file explorer and progress bar (tests/data_widgets_contract.rs).
frame-budget also measures an animating canvas (tests/canvas_widget.rs), a
dialog fading in and a progress dialog whose bar moves
(tests/overlays_contract.rs), an accordion whose section opens
(tests/layout_widgets_contract.rs) and an indeterminate progress bar
(tests/data_widgets_contract.rs).

Its color check reads the production code of both widgets and of their
builders, with comments and test items removed, and reports every color
literal: a hex string with or without `#` (so `u32::from_str_radix("ff0000",
16)` is one), an rgb() or named-color string, a tuple or array of three or
four channel values (a literal pixel such as `[255, 255, 255, 255]`), a six-
or eight-digit hex integer, a color type built from numbers (Color::Rgb(..),
Rgba { a: .., r: .. } in any field order, ColorDefinition::rgb(..),
fg_rgba(..)), a named constructor (Color::Red, Rgba::white()) and a color
of the layout's palette, in a class (`bg-gray-800`, `text-white`) or as a
string of its own (`"blue-500"`). The
builder code is found by type, wherever it is under src/builder: every
struct, impl and function whose header names ChartBuilder or ImageBuilder,
and the consts, statics and free functions of the same file those items
name, followed through the helpers they name in turn. A channel may be
written in decimal or hex (`[0xff, 0x00, 0x00, 0xff]`), and a pixel as three
or four 0-1 floats in an array as in a tuple. A pixel computed from image
data, or the all-zero pixel `[0; 4]`, is not a literal.
"""

import os
import re
import sys
from pathlib import Path

from _common import ROOT, cargo_test_filtered, finish, mask, matching, rust_sources, strip_test_modules

GROUPS = {
    "interaction": [("CHT-018", "cht_018_"), ("CHT-019", "cht_019_"), ("CHT-031", "cht_031_"), ("CHT-032", "cht_032_"),
                    ("CHT-036", "cht_036_")],
    "motion": [("CHT-022", "cht_022_")],
    "frame-budget": [("CHT-021", "cht_021_"), ("BAR-005", "bar_005_")],
    "widget-bar": [("BAR-003", "bar_003_")],
}
NAMED = "red|green|blue|black|white|yellow|cyan|magenta|gray|grey|orange|purple|pink|brown|navy|teal|lime|maroon|olive|silver"
CHANNEL = r"(?:0x[0-9a-fA-F]{1,2}|25[0-5]|2[0-4]\d|1?\d?\d)(?:_?u8)?"
UNIT = r"(?:0?\.\d+|1\.0*|0\.0*)(?:_?f32|_?f64)?"
NUM = r"-?\d+(?:\.\d+)?(?:_?(?:f32|f64|u8|u16|u32))?"
COLOR_TYPE = r"(?:Rgba?|Colou?r|ColorDefinition|Rgba8|Hsla?)"
# The families of the layout's palette (src/layout/colors.rs): `blue-500`.
PALETTE = "slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose"
SHADE = rf"(?:white|black|(?:{PALETTE})-\d{{2,3}})"
# (pattern, whether it reads string contents)
COLOR_LITERALS = {
    "hex string": (re.compile(r'"(?:#[0-9a-fA-F]{3,8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})"'), True),
    "rgb() string": (re.compile(r'"\s*(?:rgba?|hsla?)\s*\('), True),
    "named color string": (re.compile(rf'"(?i:{NAMED})"'), True),
    # A palette color in a class or as a token: `bg-gray-800`, `text-white`,
    # `border-blue-500`, or the whole string `blue-500`.
    "palette class": (re.compile(rf"(?<![\w-])(?:bg|text|fg|border|ring|accent|caret|placeholder|from|via|to)-{SHADE}(?![\w-])"), True),
    "palette color string": (re.compile(rf'"(?:{PALETTE})-\d{{2,3}}"'), True),
    "channel tuple": (re.compile(rf"(?<![\w\]])\(\s*{CHANNEL}\s*,\s*{CHANNEL}\s*,\s*{CHANNEL}\s*(?:,\s*{CHANNEL}\s*)?\)"), False),
    "channel array": (re.compile(rf"(?<![\w\])])\[\s*{CHANNEL}\s*,\s*{CHANNEL}\s*,\s*{CHANNEL}\s*(?:,\s*{CHANNEL}\s*)?,?\s*\]"), False),
    "unit tuple": (re.compile(rf"(?<![\w\]])\(\s*{UNIT}\s*,\s*{UNIT}\s*,\s*{UNIT}\s*(?:,\s*{UNIT}\s*)?\)"), False),
    "unit array": (re.compile(rf"(?<![\w\])])\[\s*{UNIT}\s*,\s*{UNIT}\s*,\s*{UNIT}\s*(?:,\s*{UNIT}\s*)?,?\s*\]"), False),
    # One value for every channel, as in [1.0; 4] or vec![255; 3]. A value
    # of zero is no color: it is transparent, or a buffer still to be filled.
    "repeat array": (re.compile(rf"(?<![\w\])])\[\s*(?!0(?:\.0*)?(?:_?[a-z]\w*)?\s*;)(?:{UNIT}|{CHANNEL})\s*;\s*[34]\s*\]"), False),
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
    "canvas": (("src/graphics",), r"CanvasBuilder"),
    "menu": (("src/widgets/menu", "src/builder/widgets/menu.rs"), r"(?:MenuBar|ContextMenu|PopupMenu|MenuItem)Builder"),
    "overlay": (("src/widgets/dialog", "src/widgets/display/modal.rs", "src/widgets/display/modal",
                 "src/widgets/display/popover.rs", "src/widgets/display/popover",
                 "src/builder/widgets/dialog.rs", "src/builder/dialog_builders.rs"),
                r"(?:Modal|Toast|Dialog|ConfirmationDialog|ProgressDialog|Wizard|Popover)Builder"),
    # The input family: the text input, checkbox, radio button, select and
    # slider, their builders, and the two button builders (input-widgets.md).
    "input": (("src/widgets/input", "src/builder/widgets/input.rs", "src/builder/specialized.rs",
               "src/builder/core.rs"),
              r"(?:TextInput|Checkbox|RadioButton|Select|Slider)Builder"),
    # The layout family: the tabs, the accordion, the breadcrumb, the scroll
    # view and the stack, and their builders (layout-widgets.md).
    "layout": (("src/widgets/layout", "src/builder/widgets/layout.rs", "src/builder/widgets/accordion.rs",
                "src/builder/widgets/breadcrumb.rs", "src/builder/specialized.rs"),
               r"(?:Tabs|Accordion|Breadcrumb|ScrollView|Stack)Builder"),
    # The data family: the table, the data table, the tree, the file
    # explorer and the progress bar, and their builders (data-widgets.md).
    "data": (("src/widgets/display/look.rs",
              "src/widgets/display/table.rs", "src/widgets/display/table",
              "src/widgets/display/data_table.rs", "src/widgets/display/data_table",
              "src/widgets/display/tree.rs", "src/widgets/display/tree",
              "src/widgets/display/file_explorer.rs", "src/widgets/display/file_explorer",
              "src/widgets/display/progress_bar.rs", "src/widgets/display/progress_bar",
              "src/builder/widgets/table.rs", "src/builder/widgets/display.rs",
              "src/builder/widgets/file_explorer.rs", "src/builder/specialized.rs"),
             r"(?:Table|DataTable|Tree|FileExplorer|ProgressBar)Builder"),
}
BUILDERS = "src/builder"
# The kernel's list of performance cores on a hybrid CPU.
PERFORMANCE_CORES = Path("/sys/devices/cpu_core/cpus")


def performance_cpus() -> set[int] | None:
    """The CPUs the kernel names as performance cores, or None when it names
    none (one core type, or no hybrid topology to read)."""
    try:
        text = PERFORMANCE_CORES.read_text().strip()
    except OSError:
        return None
    cpus = set()
    for part in filter(None, text.split(",")):
        low, _, high = part.partition("-")
        cpus.update(range(int(low), int(high or low) + 1))
    return cpus or None


def builder_spans(code: str, types: str) -> list[tuple[int, int]]:
    """The struct, enum, impl and fn items of `code` whose header names one of
    `types` (a struct's name, an impl's self type or trait argument, a
    function's parameter or return type)."""
    head = re.compile(rf"\b(?:(?:struct|enum)\s+(?:{types})\b|(?:impl|fn)\b[^{{;]*\b(?:{types})\b)[^{{;]*\{{")
    return [(m.start(), matching(code, m.end() - 1)) for m in head.finditer(code)]


ITEM = re.compile(r"^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:(?:const|static)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:"
                  r"|(?:(?:const|async|unsafe)\s+|extern\s+\"[^\"]*\"\s+)*fn\s+([A-Za-z_]\w*))", re.M)


def item_end(code: str, m: re.Match) -> int:
    """Offset just past the const, static or fn item that `m` starts."""
    if m.group(1):
        depth, k = 0, m.end()
        while k < len(code):
            depth += code[k] in "([{"
            depth -= code[k] in ")]}"
            if code[k] == ";" and depth == 0:
                return k + 1
            k += 1
        return k
    brace, semi = code.find("{", m.end()), code.find(";", m.end())
    return matching(code, brace) if brace >= 0 and (semi < 0 or brace < semi) else semi + 1


def used_items(code: str, spans: list[tuple[int, int]]) -> list[tuple[int, int]]:
    """The consts, statics and free functions of `code` that the items in
    `spans` name, and the ones those name in turn: the colors a builder
    keeps beside itself are the builder's."""
    items = {}
    for m in ITEM.finditer(code):
        span = (m.start(), item_end(code, m))
        if not any(a <= span[0] < b for a, b in spans):
            items.setdefault(m.group(1) or m.group(2), []).append(span)
    found, queue = [], list(spans)
    while queue:
        a, b = queue.pop()
        for name in set(re.findall(r"\b[A-Za-z_]\w*\b", code[a:b])):
            for span in items.pop(name, []):
                found.append(span)
                queue.append(span)
    return found


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


def test_only(path: Path) -> bool:
    """Whether `path` is an out-of-line module its parent file declares under
    #[cfg(test)] (`#[cfg(test)] mod tests;`, with or without a #[path]
    attribute), so the scan reads production code only, as it does for an
    inline test module: the declaration is there in the parent's code and
    gone once its test items are blanked."""
    declaration = re.compile(rf"\bmod\s+{re.escape(path.stem)}\s*;")
    for parent in (path.parent / "mod.rs", path.parent.with_suffix(".rs")):
        if parent == path or not parent.is_file():
            continue
        text = parent.read_text(errors="replace")
        if declaration.search(mask(text)[0]) and not declaration.search(mask(strip_test_modules(text))[0]):
            return True
    return False


def color_literals(family: tuple[tuple[str, ...], str]) -> list[str]:
    """`path:line kind` for every color literal in a family's production code:
    its files, read whole, and its builder items under src/builder. A file
    that is a test module is not production code."""
    dirs, types = family
    whole = rust_sources(*dirs)
    found = []
    for f in whole:
        if test_only(f):
            continue
        found += literals_in(f, strip_test_modules(f.read_text(errors="replace")))
    for f in rust_sources(BUILDERS):
        if f in whole or test_only(f):
            continue
        text = strip_test_modules(f.read_text(errors="replace"))
        code = mask(text)[0]
        spans = builder_spans(code, types)
        if spans:
            found += literals_in(f, text, spans + used_items(code, spans))
    return found


def main() -> int:
    group = sys.argv[1] if len(sys.argv) > 1 else ""
    if group not in GROUPS:
        print(__doc__)
        return 2
    # The frame budget is a property of the optimized build; the other
    # groups observe behavior and run the default profile.
    release = group == "frame-budget"
    if release:
        cpus = (performance_cpus() or set()) & os.sched_getaffinity(0)
        if cpus:
            # Children inherit the affinity: the build and the test run there.
            os.sched_setaffinity(0, cpus)
        print("frame budget measured on CPUs", ",".join(map(str, sorted(os.sched_getaffinity(0)))),
              "(performance cores)" if cpus else "(no performance cores named: any core)", flush=True)
    results = {req: cargo_test_filtered("charts_contract", sub, release=release) for req, sub in GROUPS[group]}
    if group == "frame-budget":
        ok, why = results["BAR-005"]
        canvas = cargo_test_filtered("canvas_widget", "bar_005_", features=["wgpu-graphics"], release=True)
        overlays = cargo_test_filtered("overlays_contract", "bar_005_", release=True)
        layout = cargo_test_filtered("layout_widgets_contract", "bar_005_", release=True)
        data = cargo_test_filtered("data_widgets_contract", "bar_005_", release=True)
        results["BAR-005"] = (ok and canvas[0] and overlays[0] and layout[0] and data[0],
                              f"charts and image: {why}; canvas: {canvas[1]}; overlays: {overlays[1]}; "
                              f"layout widgets: {layout[1]}; data widgets: {data[1]}")
    if group == "widget-bar":
        ok, why = results["BAR-003"]
        problems = [] if ok else [f"charts: {why}"]
        for name, (passed, reason) in (
            ("image", cargo_test_filtered("api_widget_behavior", "bar_003_")),
            ("image screen reader", cargo_test_filtered(None, "bar_003_", package="reactive-tui")),
            ("canvas", cargo_test_filtered("canvas_widget", "bar_003_", features=["wgpu-graphics"])),
            ("menus", cargo_test_filtered("menus_contract", "bar_003_")),
            ("overlays", cargo_test_filtered("overlays_contract", "bar_003_")),
            ("layout", cargo_test_filtered("layout_widgets_contract", "bar_003_")),
            ("data", cargo_test_filtered("data_widgets_contract", "bar_003_")),
            ("input widgets", cargo_test_filtered("input_widgets_contract", "bar_003_")),
        ):
            if not passed:
                problems.append(f"{name}: {reason}")
        for family, code in WIDGET_CODE.items():
            literals = color_literals(code)
            if literals:
                problems.append(f"{len(literals)} hard-coded colors in {family} code: {', '.join(literals[:4])}")
        results["BAR-003"] = (not problems, "; ".join(problems) or f"charts, image, canvas, menus, overlays, input, layout and data widgets: {why}")
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
