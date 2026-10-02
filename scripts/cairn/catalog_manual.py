#!/usr/bin/env python3
"""BAR-006: a delivered widget has a catalog page and a manual section whose
builder methods exist in the code. Scope: the chart family (line, area,
scatter, bar, candlestick), the image widget, reworked to draw its block
fallback through the renderer's blitters, the graphics canvas
(src/graphics, manual/wgpu-graphics.md), and the menu family (the menu bar,
the context menu, the popup menu and the dialog menu: src/widgets/menu,
src/builder/widgets/menu.rs, manual/menus.md).

Both files are read by their structure, not by substring. A catalog page is
a CatalogPage listed in ALL that selected_page maps to a function; a chart
type is on the Charts page when that function builds its card and its typed
builder. A manual heading is a Markdown heading outside fenced code, and a
section runs to the next heading of its level or the end of the file. Code
spans pair backtick runs of equal length within a paragraph, as a Markdown
renderer pairs them, and a call written outside any span is checked too, so
a stray backtick hides no citation.

Also exposes `chart_docs_problems()` for the charts-goldens mechanism, which
owns CHT-023.
"""

import re
import sys

from _common import ROOT, mask, matching, rust_sources, strip_test_modules
from charts_builders import builder_methods

CHART_TYPES = ["line", "area", "scatter", "bar", "candlestick", "pie", "donut", "radar", "sankey"]
CATALOG = ROOT / "examples/widget_catalog/catalog.rs"
MANUAL = ROOT / "manual/display-widgets.md"


def chart_text() -> str:
    return "\n".join(strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(
        "src/widgets/display/charts.rs", "src/widgets/display/charts", "src/builder/widgets/chart.rs"))


def code_methods() -> set[str]:
    return set(re.findall(r"\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)", chart_text()))


FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})")
HEADING = re.compile(r"^ {0,3}(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$")
TICKS = re.compile(r"`+")
TYPED_CALL = re.compile(r"\b([A-Z][A-Za-z0-9]*)::([a-z_][a-z0-9_]*)\(")
METHOD_CALL = re.compile(r"\.([a-z_][a-z0-9_]*)\(")


def blocks(markdown: str) -> list[tuple[str, str]]:
    """The markdown as ("fence", code) and ("text", paragraph) blocks."""
    out, paragraph, fence, code = [], [], None, []
    for line in markdown.splitlines():
        opened = FENCE.match(line)
        if fence is not None:
            if opened and opened.group(1)[0] == fence[0] and len(opened.group(1)) >= len(fence):
                out.append(("fence", "\n".join(code)))
                fence, code = None, []
            else:
                code.append(line)
            continue
        if opened:
            if paragraph:
                out.append(("text", "\n".join(paragraph)))
                paragraph = []
            fence = opened.group(1)
            continue
        if line.strip():
            paragraph.append(line)
        elif paragraph:
            out.append(("text", "\n".join(paragraph)))
            paragraph = []
    if paragraph:
        out.append(("text", "\n".join(paragraph)))
    if fence is not None:
        out.append(("fence", "\n".join(code)))
    return out


def headings(markdown: str) -> list[tuple[int, str]]:
    """(level, title) of every heading outside fenced code."""
    found = []
    for kind, block in blocks(markdown):
        if kind == "text":
            for line in block.splitlines():
                m = HEADING.match(line)
                if m:
                    found.append((len(m.group(1)), m.group(2)))
    return found


def section(markdown: str, title: str) -> str | None:
    """The text under the heading `title`, up to the next heading of the same
    or a higher level, or the end of the file; None without that heading."""
    lines, fence, start, level = markdown.splitlines(), None, None, 0
    for index, line in enumerate(lines):
        opened = FENCE.match(line)
        if fence is not None:
            if opened and opened.group(1)[0] == fence[0] and len(opened.group(1)) >= len(fence):
                fence = None
            continue
        if opened:
            fence = opened.group(1)
            continue
        m = HEADING.match(line)
        if not m:
            continue
        if start is not None and len(m.group(1)) <= level:
            return "\n".join(lines[start:index])
        if start is None and m.group(2) == title:
            start, level = index + 1, len(m.group(1))
    return None if start is None else "\n".join(lines[start:])


def citations(markdown: str) -> list[str]:
    """Every piece of text that can cite a builder call: each code span and
    fenced block on its own, then the prose outside them."""
    pieces, prose = [], []
    for kind, block in blocks(markdown):
        if kind == "fence":
            pieces.append(block)
            continue
        i, outside = 0, []
        while (run := TICKS.search(block, i)) is not None:
            closing = next((m for m in TICKS.finditer(block, run.end()) if len(m.group()) == len(run.group())), None)
            if closing is None:
                # An unmatched run is literal text; pairing continues after it.
                outside.append(block[i:run.end()])
                i = run.end()
                continue
            outside.append(block[i:run.start()])
            pieces.append(block[run.end():closing.start()])
            i = closing.end()
        outside.append(block[i:])
        prose.append("".join(outside))
    return pieces + ["\n".join(prose)]


def catalog_pages() -> tuple[str, str, dict[str, tuple[int, int]]]:
    """The catalog's code and prose (comments removed, strings kept) and the
    body span of each page function that a listed CatalogPage maps to."""
    text = CATALOG.read_text(errors="replace") if CATALOG.exists() else ""
    code, prose = mask(text)
    listed_at = re.search(r"\bconst ALL\s*:\s*\[Self\s*;\s*\d+\]\s*=\s*\[", code)
    listed = set(re.findall(r"Self::(\w+)", code[listed_at.end():code.find("]", listed_at.end())])) if listed_at else set()
    pages = {}
    for variant, function in re.findall(r"CatalogPage::(\w+)\s*=>\s*self\.(\w+)\(\)", code):
        if variant not in listed:
            continue
        m = re.search(rf"\bfn\s+{function}\s*\(", code)
        opening = code.find("{", m.end()) if m else -1
        if opening >= 0:
            pages[variant] = (opening, matching(code, opening))
    return code, prose, pages


def chart_docs_problems() -> list[str]:
    problems = []
    code, prose, pages = catalog_pages()
    charts = pages.get("Charts")
    if charts is None:
        problems.append("catalog lists no Charts page")
    for kind in CHART_TYPES:
        name = kind.capitalize()
        if charts is not None and not (
                re.search(rf'Self::card\(\s*"{name} chart\b', prose[charts[0]:charts[1]])
                and re.search(rf"\b{name}ChartBuilder::new\(", code[charts[0]:charts[1]])):
            problems.append(f"catalog's Charts page has no {kind} chart card built with {name}ChartBuilder")
    manual = MANUAL.read_text(errors="replace") if MANUAL.exists() else ""
    titles = [title for _, title in headings(manual)]
    for kind in CHART_TYPES:
        if not any(re.match(rf"{kind} chart\b", title, re.I) for title in titles):
            problems.append(f"manual has no heading for the {kind} chart")
    if not any(re.search(r"size class", title, re.I) for title in titles):
        problems.append("manual has no size-class heading")
    charts_text = section(manual, "Charts")
    if charts_text is None:
        return problems + ["manual has no Charts section"]
    for cls in ("mini", "medium", "large"):
        if not re.search(rf"\b{cls}\b", charts_text, re.I):
            problems.append(f"manual does not describe the {cls} size class")
    methods = code_methods()
    # A call on `Type::method(` is checked on that type; a chain that starts
    # with one is checked on its type; any other `.method(` against every
    # chart pub fn.
    by_type = {k.lower(): v for k, v in builder_methods(chart_text()).items()}
    for piece in citations(charts_text):
        typed = TYPED_CALL.findall(piece)
        for type_name, method in typed:
            owned = by_type.get(type_name.lower())
            if owned is None:
                problems.append(f"manual cites {type_name}::{method}() but {type_name} is not a chart type")
            elif method not in owned:
                problems.append(f"manual cites {type_name}::{method}() which is not a pub fn on {type_name}")
        chain_owner = by_type.get(typed[0][0].lower()) if typed else None
        for name in METHOD_CALL.findall(piece):
            if chain_owner is not None:
                if name not in chain_owner:
                    problems.append(f"manual cites .{name}() on {typed[0][0]} which has no such pub fn")
            elif name not in methods:
                problems.append(f"manual cites .{name}() which is not a pub fn in chart code")
    return problems


# CHT-023: the cards the Charts page must show, by a phrase in the card's
# title, besides one card per type.
VARIANT_CARDS = [
    ("a line with several series", r'Self::card\(\s*"Line chart[^"]*\b(?:two|three|several|multi)'),
    ("a stacked area", r'Self::card\(\s*"Area chart[^"]*\bstacked\b'),
    ("a scatter with several series", r'Self::card\(\s*"Scatter chart[^"]*\b(?:two|three|several|multi)'),
    ("horizontal bars", r'Self::card\(\s*"Bar chart[^"]*\bhorizontal\b'),
    ("grouped bars", r'Self::card\(\s*"Bar chart[^"]*\bgrouped\b'),
    ("stacked bars", r'Self::card\(\s*"Bar chart[^"]*\bstacked\b'),
]
# A forced large class in a rectangle under the class's own range (CHT-024:
# 200 columns by 40 rows).
FORCED_LARGE = re.compile(r"\(\s*SizeClass::Large\s*,\s*(\d+)\w*\s*,\s*(\d+)\w*\s*\)")


def chart_variant_problems() -> list[str]:
    """CHT-023: the Charts page has a card for every cartesian variant and
    does not force the large class into a rectangle under 200 by 40."""
    problems = []
    code, prose, pages = catalog_pages()
    charts = pages.get("Charts")
    if charts is None:
        return ["catalog lists no Charts page"]
    page_prose, page_code = prose[charts[0]:charts[1]], code[charts[0]:charts[1]]
    for what, pattern in VARIANT_CARDS:
        if not re.search(pattern, page_prose, re.I):
            problems.append(f"catalog's Charts page has no card for {what}")
    for m in FORCED_LARGE.finditer(page_code):
        width, height = int(m.group(1)), int(m.group(2))
        if width < 200 or height < 40:
            problems.append(f"catalog's Charts page forces the large class into {width} by {height}, under its 200 by 40 range")
    return problems


IMAGE_MANUAL = ROOT / "manual/images-and-clipboard.md"
IMAGE_BUILDER = ROOT / "src/builder/specialized.rs"


def image_docs_problems() -> list[str]:
    """The image widget: a listed catalog page built with `image()` that
    shows an Image card, a manual section headed "Image widget", and every
    builder method that section cites is a pub fn on ImageBuilder."""
    problems = []
    code, prose, pages = catalog_pages()
    if not any(re.search(r"(?<![\w.])image\(\)", code[a:b]) and re.search(r'Self::card\(\s*"Image\b', prose[a:b])
               for a, b in pages.values()):
        problems.append("catalog has no page with an Image card built with image()")
    manual = IMAGE_MANUAL.read_text(errors="replace") if IMAGE_MANUAL.exists() else ""
    text = section(manual, "Image widget")
    if text is None:
        problems.append("manual has no Image widget heading")
        return problems
    builder = IMAGE_BUILDER.read_text(errors="replace") if IMAGE_BUILDER.exists() else ""
    body = re.search(r"impl ImageBuilder\s*\{(.*?)\n\}", builder, re.S)
    methods = set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", body.group(1))) if body else set()
    if not methods:
        problems.append("no ImageBuilder impl found")
    for piece in citations(text):
        for name in METHOD_CALL.findall(piece):
            if name not in methods:
                problems.append(f"manual cites .{name}() which is not a pub fn on ImageBuilder")
    return problems


CANVAS_MANUAL = ROOT / "manual/wgpu-graphics.md"


def canvas_docs_problems() -> list[str]:
    """The graphics canvas: a listed catalog page that builds a Canvas, a
    manual section headed "Canvas widget", and every method that section
    cites is a pub fn in src/graphics."""
    problems = []
    code, _, pages = catalog_pages()
    if not any(re.search(r"\bCanvas\b", code[a:b]) for a, b in pages.values()):
        problems.append("catalog has no page that builds a Canvas")
    manual = CANVAS_MANUAL.read_text(errors="replace") if CANVAS_MANUAL.exists() else ""
    text = section(manual, "Canvas widget")
    if text is None:
        problems.append("manual has no Canvas widget heading")
        return problems
    methods = set()
    for f in rust_sources("src/graphics"):
        methods |= set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", strip_test_modules(f.read_text(errors="replace"))))
    for piece in citations(text):
        for name in METHOD_CALL.findall(piece):
            if name not in methods:
                problems.append(f"manual cites .{name}() which is not a pub fn in src/graphics")
    return problems


MENUS_MANUAL = ROOT / "manual/menus.md"
# Each menu: its card's title on the catalog's Menus & dialogs page, what
# builds it there, and its heading in the manual.
MENUS = (
    ("MenuBar", r"(?<![\w.])menubar\(\)", "Menu bar"),
    ("ContextMenu", r"(?<![\w.])context_menu\(\)", "Context menu"),
    ("PopupMenu", r"\bpopup_menu\(\)", "Popup menu"),
    ("DialogMenu", r"\bDialogMenuBuilder::\w+\(\)", "Dialog menu"),
)


def menu_docs_problems() -> list[str]:
    """The menu family: the catalog's Menus & dialogs page builds each of the
    four menus in a card of its name, the manual has a heading for each, and
    every method the manual's menu sections cite is a pub fn in the menu
    code or its builder."""
    problems = []
    code, prose, pages = catalog_pages()
    page = pages.get("MenusDialogs")
    if page is None:
        problems.append("catalog lists no Menus & dialogs page")
    for card, built, _ in MENUS:
        if page is not None and not (re.search(rf'"{card}"', prose[page[0]:page[1]])
                                     and re.search(built, code[page[0]:page[1]])):
            problems.append(f"catalog's Menus & dialogs page has no {card} card built with {built}")
    manual = MENUS_MANUAL.read_text(errors="replace") if MENUS_MANUAL.exists() else ""
    titles = [title for _, title in headings(manual)]
    methods = set()
    for f in rust_sources("src/widgets/menu", "src/builder/widgets/menu.rs"):
        methods |= set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", strip_test_modules(f.read_text(errors="replace"))))
    for _, _, heading in MENUS:
        if heading not in titles:
            problems.append(f"manual has no {heading} heading")
            continue
        for piece in citations(section(manual, heading) or ""):
            for name in METHOD_CALL.findall(piece):
                if name not in methods:
                    problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn in menu code")
    for heading in ("Colors", "Panels", "Keyboard"):
        text = section(manual, heading)
        if text is None:
            problems.append(f"manual has no {heading} heading under Menus")
            continue
        for piece in citations(text):
            for name in METHOD_CALL.findall(piece):
                if name not in methods:
                    problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn in menu code")
    return problems


DIALOGS_MANUAL = ROOT / "manual/dialogs.md"
DISPLAY_MANUAL = ROOT / "manual/display-widgets.md"
# Each overlay: its card's title on the catalog's Menus & dialogs page, what
# builds it there, the manual page and the heading of its section.
OVERLAYS = (
    ("Modal", r"(?<![\w.])modal\(\)", DISPLAY_MANUAL, "Modal"),
    ("Popover", r"(?<![\w.])popover\(\)", DISPLAY_MANUAL, "Popover"),
    ("ConfirmationDialog", r"\bconfirmation_dialog\(\)", DIALOGS_MANUAL, "Confirmation dialog"),
    ("InputDialog", r"\bInputDialog::new\(", DIALOGS_MANUAL, "Input dialog"),
    ("AutocompleteDialog", r"\bAutocompleteDialog::new\(", DIALOGS_MANUAL, "Autocomplete dialog"),
    ("ProgressDialog", r"\bprogress_dialog\(\)", DIALOGS_MANUAL, "Progress dialog"),
    ("Toast", r"(?<![\w.])toast\(\)", DIALOGS_MANUAL, "Toast"),
    ("WizardDialog", r"(?<![\w.])wizard\(\)", DIALOGS_MANUAL, "Wizard dialog"),
)
OVERLAY_CODE = ("src/widgets/dialog", "src/widgets/display/modal.rs", "src/widgets/display/modal",
                "src/widgets/display/popover.rs", "src/widgets/display/popover",
                "src/builder/widgets/dialog.rs", "src/builder/dialog_builders.rs",
                "src/builder/specialized.rs", "src/builder/widgets/display.rs")


def overlay_docs_problems() -> list[str]:
    """The overlay family: the catalog's Menus & dialogs page builds each of
    the eight overlays in a card of its name, the manual has a heading for
    each, and every method the manual's overlay sections cite is a pub fn
    in the overlay code or its builders."""
    problems = []
    code, prose, pages = catalog_pages()
    page = pages.get("MenusDialogs")
    if page is None:
        problems.append("catalog lists no Menus & dialogs page")
    for card, built, _, _ in OVERLAYS:
        if page is not None and not (re.search(rf'"{card}"', prose[page[0]:page[1]])
                                     and re.search(built, code[page[0]:page[1]])):
            problems.append(f"catalog's Menus & dialogs page has no {card} card built with {built}")
    methods = set()
    for f in rust_sources(*OVERLAY_CODE):
        methods |= set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", strip_test_modules(f.read_text(errors="replace"))))
    for _, _, page_path, heading in OVERLAYS:
        manual = page_path.read_text(errors="replace") if page_path.exists() else ""
        if heading not in [title for _, title in headings(manual)]:
            problems.append(f"{page_path.relative_to(ROOT)} has no {heading} heading")
            continue
        for piece in citations(section(manual, heading) or ""):
            for name in METHOD_CALL.findall(piece):
                if name not in methods:
                    problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn in overlay code")
    return problems


INPUT_MANUAL = ROOT / "manual/input-widgets.md"
# Each input widget: its card's title on the catalog's Input widgets page,
# what builds it there, and the heading of its section in the manual.
INPUT_WIDGETS = (
    ("TextInput", r"\btext_input\(\)", "Text input"),
    ("Checkbox", r"(?<![\w.])checkbox\(\)", "Checkbox"),
    ("RadioButton", r"\bradio_button\(\)", "Radio button"),
    ("Select", r"(?<![\w.])select\(\)", "Select"),
    ("Slider", r"(?<![\w.])slider\(\)", "Slider"),
    ("Button", r"\bprimary_button\(", "Button"),
)
INPUT_CODE = ("src/widgets/input", "src/builder/widgets/input.rs", "src/builder/specialized.rs",
              "src/builder/core.rs")


LAYOUT_MANUAL = ROOT / "manual/layout-widgets.md"
# Each layout widget: its card's title on the catalog's Layout widgets
# page, what builds it there, and the heading of its section in the manual.
LAYOUT_WIDGETS = (
    ("Breadcrumb", r"\bpath_breadcrumb\(", "Breadcrumb"),
    ("Accordion", r"\bsimple_accordion\(", "Accordion"),
    ("Tabs", r"(?<![\w.])tabs\(\)", "Tabs"),
    ("ScrollView", r"\bscroll_view\(\)", "Scroll view"),
    ("Stack", r"(?<![\w.])stack\(\)", "Stack"),
)
LAYOUT_CODE = ("src/widgets/layout", "src/builder/widgets/layout.rs", "src/builder/widgets/accordion.rs",
               "src/builder/widgets/breadcrumb.rs", "src/builder/specialized.rs")


ENTRY_POINT = re.compile(r"\bbuilder::([a-z_][a-z0-9_]*)\(\)")
TYPE_PATH = re.compile(r"\b(?:widgets::[a-z_:]+::)?([A-Z][A-Za-z0-9]*)(?:::new|\b)")


def impl_methods(code: str) -> dict[str, set[str]]:
    """The pub fns of each type's `impl` blocks in one file's code."""
    methods: dict[str, set[str]] = {}
    for found in re.finditer(r"\bimpl(?:<[^>]*>)?\s+([A-Z][A-Za-z0-9]*)(?:<[^>]*>)?\s*\{", code):
        depth, i = 1, found.end()
        while i < len(code) and depth:
            depth += {"{": 1, "}": -1}.get(code[i], 0)
            i += 1
        methods.setdefault(found.group(1), set()).update(
            re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", code[found.end():i]))
    return methods


def receivers(sources: dict[Path, str]) -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """What a section's citations can be called on. Two builders may share
    a name (`builder::stack()` returns src/builder/specialized.rs's
    StackBuilder, not src/widgets/layout/stack.rs's), so an entry point's
    methods are the `impl` block in the entry point's own file, or in the
    file its `use` line imports the type from; a type cited by name takes
    every `impl` block of that name."""
    impls = {path: impl_methods(code) for path, code in sources.items()}
    by_name: dict[str, set[str]] = {}
    for methods in impls.values():
        for name, fns in methods.items():
            by_name.setdefault(name, set()).update(fns)
    entries: dict[str, set[str]] = {}
    for path, code in sources.items():
        for name, ret in re.findall(r"\bpub fn ([a-z_][a-z0-9_]*)\(\) -> ([A-Z][A-Za-z0-9]*)", code):
            if ret in impls[path]:
                entries.setdefault(name, impls[path][ret])
                continue
            imported = re.search(rf"^\s*use\s+([\w:]+)::(?:\{{[^}}]*\b{ret}\b[^}}]*\}}|{ret})\s*;", code, re.M)
            module = imported.group(1).rsplit("::", 1)[-1] if imported else None
            source = next((other for other in sources if other.stem == module and ret in impls[other]), None)
            entries.setdefault(name, impls[source][ret] if source else by_name.get(ret, set()))
    return entries, by_name


def receiver_problems(section_text: str, heading: str, sources: dict[Path, str], family: str) -> list[str]:
    """Every `.method(` a section cites must be a pub fn of a type the
    section names: the type a cited `builder::name()` returns, or a type
    cited by name (`Type::new(..)`, `widgets::layout::Type`). A section that
    names no type is held to the family's pool of pub fns."""
    entries, by_name = receivers(sources)
    pool = set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", "\n".join(sources.values())))
    allowed, named = set(), []
    for piece in citations(section_text):
        for entry in ENTRY_POINT.findall(piece):
            if entry in entries:
                named.append(f"builder::{entry}()")
                allowed |= entries[entry]
        for type_name in TYPE_PATH.findall(piece):
            if type_name in by_name:
                named.append(type_name)
                allowed |= by_name[type_name]
    problems = []
    for piece in citations(section_text):
        for name in METHOD_CALL.findall(piece):
            if named and name not in allowed:
                problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn of "
                                f"{', '.join(sorted(set(named)))} in {family} code")
            elif not named and name not in pool:
                problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn in {family} code")
    return problems


def layout_docs_problems() -> list[str]:
    """The layout family: the catalog's Layout widgets page builds each of
    the five widgets in a card of its name, the manual has a heading for
    each, and every method the manual's sections cite is a pub fn in the
    layout code or its builders."""
    problems = []
    code, prose, pages = catalog_pages()
    page = pages.get("Layout")
    if page is None:
        problems.append("catalog lists no Layout widgets page")
    for card, built, _ in LAYOUT_WIDGETS:
        if page is not None and not (re.search(rf'"{card}"', prose[page[0]:page[1]])
                                     and re.search(built, code[page[0]:page[1]])):
            problems.append(f"catalog's Layout widgets page has no {card} card built with {built}")
    sources = {f: strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(*LAYOUT_CODE)}
    manual = LAYOUT_MANUAL.read_text(errors="replace") if LAYOUT_MANUAL.exists() else ""
    for _, _, heading in LAYOUT_WIDGETS:
        if heading not in [title for _, title in headings(manual)]:
            problems.append(f"{LAYOUT_MANUAL.relative_to(ROOT)} has no {heading} heading")
            continue
        problems.extend(receiver_problems(section(manual, heading) or "", heading, sources, "layout"))
    return problems


DATA_MANUAL = ROOT / "manual/data-widgets.md"
# Each data widget: its card's title on the catalog's Data display page,
# what builds it there, and the heading of its section in the manual.
DATA_WIDGETS = (
    ("Table", r"\bTable::with_props\(", "Table"),
    ("DataTable", r"\bdata_table\(\)", "Data table"),
    ("Tree", r"(?<![\w.])tree\(\)", "Tree"),
    ("FileExplorer", r"\bfile_explorer\(\)", "File explorer"),
    ("ProgressBar", r"\bprogress_bar\(\)", "Progress bar"),
)
DATA_CODE = ("src/widgets/display/look.rs", "src/widgets/display/table.rs", "src/widgets/display/table",
             "src/widgets/display/data_table.rs", "src/widgets/display/data_table",
             "src/widgets/display/tree.rs", "src/widgets/display/tree",
             "src/widgets/display/file_explorer.rs", "src/widgets/display/file_explorer",
             "src/widgets/display/progress_bar.rs", "src/widgets/display/progress_bar",
             "src/builder/widgets/table.rs", "src/builder/widgets/display.rs",
             "src/builder/widgets/file_explorer.rs", "src/builder/specialized.rs")


def data_docs_problems() -> list[str]:
    """The data family: the catalog's Data display page builds each of the
    five widgets in a card of its name, the manual has a heading for each,
    and every method the manual's sections cite is a pub fn of the builder
    or type the section names in the data code."""
    problems = []
    code, prose, pages = catalog_pages()
    page = pages.get("Data")
    if page is None:
        problems.append("catalog lists no Data display page")
    for card, built, _ in DATA_WIDGETS:
        if page is not None and not (re.search(rf'"{card}"', prose[page[0]:page[1]])
                                     and re.search(built, code[page[0]:page[1]])):
            problems.append(f"catalog's Data display page has no {card} card built with {built}")
    sources = {f: strip_test_modules(f.read_text(errors="replace")) for f in rust_sources(*DATA_CODE)}
    manual = DATA_MANUAL.read_text(errors="replace") if DATA_MANUAL.exists() else ""
    for _, _, heading in DATA_WIDGETS:
        if heading not in [title for _, title in headings(manual)]:
            problems.append(f"{DATA_MANUAL.relative_to(ROOT)} has no {heading} heading")
            continue
        problems.extend(receiver_problems(section(manual, heading) or "", heading, sources, "data"))
    return problems


def input_docs_problems() -> list[str]:
    """The input family: the catalog's Input widgets page builds each of the
    six controls in a card of its name, the manual has a heading for each,
    and every method the manual's sections cite is a pub fn in the input
    code or its builders."""
    problems = []
    code, prose, pages = catalog_pages()
    page = pages.get("Input")
    if page is None:
        problems.append("catalog lists no Input widgets page")
    for card, built, _ in INPUT_WIDGETS:
        if page is not None and not (re.search(rf'"{card}"', prose[page[0]:page[1]])
                                     and re.search(built, code[page[0]:page[1]])):
            problems.append(f"catalog's Input widgets page has no {card} card built with {built}")
    methods = set()
    for f in rust_sources(*INPUT_CODE):
        methods |= set(re.findall(r"\bpub fn\s+([a-z_][a-z0-9_]*)", strip_test_modules(f.read_text(errors="replace"))))
    manual = INPUT_MANUAL.read_text(errors="replace") if INPUT_MANUAL.exists() else ""
    for _, _, heading in INPUT_WIDGETS:
        if heading not in [title for _, title in headings(manual)]:
            problems.append(f"{INPUT_MANUAL.relative_to(ROOT)} has no {heading} heading")
            continue
        for piece in citations(section(manual, heading) or ""):
            for name in METHOD_CALL.findall(piece):
                if name not in methods:
                    problems.append(f"manual's {heading} section cites .{name}() which is not a pub fn in input code")
    return problems


def main() -> int:
    problems = (chart_docs_problems() + image_docs_problems() + canvas_docs_problems() + menu_docs_problems()
                + overlay_docs_problems() + input_docs_problems() + layout_docs_problems()
                + data_docs_problems())
    if problems:
        print("BAR-006 violated:")
        for p in problems:
            print("  " + p)
        return 1
    print("BAR-006 holds for the chart family, the image widget, the graphics canvas, the menus, the overlays, "
          "the input, layout and data widgets")
    return 0


if __name__ == "__main__":
    sys.exit(main())
