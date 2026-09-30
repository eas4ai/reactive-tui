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
                + overlay_docs_problems() + input_docs_problems())
    if problems:
        print("BAR-006 violated:")
        for p in problems:
            print("  " + p)
        return 1
    print("BAR-006 holds for the chart family, the image widget, the graphics canvas, the menus, the overlays and the input widgets")
    return 0


if __name__ == "__main__":
    sys.exit(main())
