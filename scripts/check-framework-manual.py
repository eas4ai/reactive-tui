#!/usr/bin/env python3
"""Validate the framework manual's navigation and source coverage."""

from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
MANUAL = ROOT / "manual"
OVERVIEW = MANUAL / "README.md"
REQUIRED_HEADINGS = {
    "Purpose",
    "Main API",
    "Behavior",
    "Limits",
    "Source map",
    "Related chapters",
}
LINK = re.compile(r"(?<!!)\[[^]]+\]\(([^)]+)\)")
MODULES = re.compile(r"^Crate modules:\s*(.+)$", re.MULTILINE)
WIDGET = re.compile(r"^Widget module:\s*`([a-z_]+)`\s*$", re.MULTILINE)


def fail(message: str) -> None:
    raise AssertionError(message)


def headings(text: str) -> list[str]:
    return [match.group(2).strip() for match in re.finditer(r"^(#{1,6})\s+(.+)$", text, re.MULTILINE)]


def anchor(heading: str) -> str:
    value = heading.strip().lower()
    value = re.sub(r"[^a-z0-9 _-]", "", value)
    return re.sub(r"[ _]+", "-", value)


def link_target(page: Path, raw: str) -> tuple[Path, str | None] | None:
    if raw.startswith(("http://", "https://", "mailto:")):
        return None
    path_text, marker, fragment = unquote(raw).partition("#")
    target = (page.parent / path_text).resolve() if path_text else page.resolve()
    return target, fragment if marker else None


def tracked_files() -> set[Path]:
    output = subprocess.check_output(
        ["git", "ls-files", "-z"], cwd=ROOT
    ).decode().split("\0")
    return {(ROOT / item).resolve() for item in output if item}


def exported_modules(path: Path) -> set[str]:
    return set(re.findall(r"^pub mod ([a-z_]+);", path.read_text(), re.MULTILINE))


def cargo_features() -> set[str]:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        return set(tomllib.load(handle).get("features", {}))


def quoted_names(line: str) -> set[str]:
    return set(re.findall(r"`([a-z][a-z0-9_-]*)`", line))


def main() -> int:
    if not OVERVIEW.is_file():
        fail("manual/README.md is missing")

    pages = sorted(path for path in MANUAL.glob("*.md") if path.name != "README.md")
    if not pages:
        fail("the manual has no focused pages")

    tracked = tracked_files()
    overview_text = OVERVIEW.read_text()
    overview_headings = {anchor(item) for item in headings(overview_text)}
    linked_pages: set[Path] = set()

    for raw in LINK.findall(overview_text):
        resolved = link_target(OVERVIEW, raw)
        if resolved is None:
            continue
        target, fragment = resolved
        if not target.exists():
            fail(f"broken overview link: {raw}")
        if fragment:
            available = overview_headings if target == OVERVIEW.resolve() else {
                anchor(item) for item in headings(target.read_text())
            }
            if fragment not in available:
                fail(f"broken heading link: {raw}")
        if target.parent == MANUAL.resolve() and target.name != "README.md":
            linked_pages.add(target)

    expected_pages = {page.resolve() for page in pages}
    if linked_pages != expected_pages:
        missing = sorted(str(path.relative_to(ROOT)) for path in expected_pages - linked_pages)
        extra = sorted(str(path.relative_to(ROOT)) for path in linked_pages - expected_pages)
        fail(f"manual page reachability differs; missing={missing}, extra={extra}")

    covered_modules: set[str] = set()
    covered_widgets: set[str] = set()
    for page in pages:
        text = page.read_text()
        page_headings = set(headings(text))
        missing_headings = REQUIRED_HEADINGS - page_headings
        if missing_headings:
            fail(f"{page.relative_to(ROOT)} lacks headings: {sorted(missing_headings)}")

        module_match = MODULES.search(text)
        if not module_match:
            fail(f"{page.relative_to(ROOT)} lacks a Crate modules line")
        covered_modules.update(quoted_names(module_match.group(1)))
        widget_match = WIDGET.search(text)
        if widget_match:
            covered_widgets.add(widget_match.group(1))

        source_links = 0
        test_links = 0
        for raw in LINK.findall(text):
            resolved = link_target(page, raw)
            if resolved is None:
                continue
            target, fragment = resolved
            if not target.exists():
                fail(f"broken link in {page.relative_to(ROOT)}: {raw}")
            if target not in tracked:
                fail(f"untracked link target in {page.relative_to(ROOT)}: {raw}")
            if fragment:
                available = {anchor(item) for item in headings(target.read_text())}
                if fragment not in available:
                    fail(f"broken heading link in {page.relative_to(ROOT)}: {raw}")
            relative = target.relative_to(ROOT).as_posix()
            source_links += relative.startswith(("src/", "crates/", "bindings/"))
            test_links += relative.startswith(("tests/", "examples/", "verification/"))
        if source_links == 0 or test_links == 0:
            fail(f"{page.relative_to(ROOT)} needs tracked source and confirming test links")

    public_modules = exported_modules(ROOT / "src/lib.rs")
    if covered_modules != public_modules:
        fail(
            "public module coverage differs; "
            f"missing={sorted(public_modules - covered_modules)}, "
            f"unknown={sorted(covered_modules - public_modules)}"
        )

    public_widgets = exported_modules(ROOT / "src/widgets/mod.rs")
    if covered_widgets != public_widgets:
        fail(
            "widget coverage differs; "
            f"missing={sorted(public_widgets - covered_widgets)}, "
            f"unknown={sorted(covered_widgets - public_widgets)}"
        )

    getting_started = (MANUAL / "getting-started.md").read_text()
    named_features = quoted_names(
        next(
            line for line in getting_started.splitlines()
            if line.startswith("Cargo features:")
        )
    )
    features = cargo_features()
    if named_features != features:
        fail(
            "Cargo feature coverage differs; "
            f"missing={sorted(features - named_features)}, "
            f"unknown={sorted(named_features - features)}"
        )

    print(
        f"framework manual: {len(pages)} pages, "
        f"{len(public_modules)} modules, {len(public_widgets)} widget families, "
        f"{len(features)} Cargo features"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, StopIteration, ValueError) as error:
        print(f"framework manual check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
