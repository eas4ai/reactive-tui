#!/usr/bin/env python3
"""Acceptance check for the capture-oriented widget catalog example."""

from __future__ import annotations

from pathlib import Path
import argparse
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
PAGES = (
    "input_page",
    "layout_page",
    "data_page",
    "menus_dialogs_page",
    "media_page",
    "motion_page",
    "system_page",
)
WIDGET_FAMILIES = (
    "Checkbox",
    "RadioButton",
    "Select",
    "Slider",
    "TextInput",
    "Accordion",
    "Breadcrumb",
    "ScrollView",
    "Stack",
    "Tabs",
    "Chart",
    "Table",
    "DataTable",
    "Tree",
    "FileExplorer",
    "ProgressBar",
    "Modal",
    "Popover",
    "Image",
    "MenuBar",
    "ContextMenu",
    "PopupMenu",
    "DialogMenu",
    "ConfirmationDialog",
    "InputDialog",
    "AutocompleteDialog",
    "ProgressDialog",
    "Toast",
    "WizardDialog",
    "TerminalWidget",
)
WIDGET_MOUNTS = dict(zip(WIDGET_FAMILIES, (
    "checkbox(", "radio_button(", "select(", "slider(", "text_input(",
    "simple_accordion(", "path_breadcrumb(", "scroll_view(", "stack(", "tabs(",
    "builder::chart(", "Table::with_props(", "data_table(", "tree(", "file_explorer(", "progress_bar(",
    "builder::modal(", "popover(", "image(", "menubar(", "context_menu(", "builder::popup_menu(",
    "Element::typed::<DialogMenu>", "confirmation_dialog(", "InputDialog::new(", "AutocompleteDialog::new(",
    "progress_dialog(", "toast(", "wizard(", "Element::typed::<TerminalWidget>",
), strict=True))


def validate(
    *,
    requirement: str = "all",
    source: str,
    readme: str,
    manual: str,
    test_output: str,
    compile_output: str,
    pty_output: str,
) -> list[str]:
    errors: list[str] = []
    for page in PAGES:
        if f"fn {page}" not in source:
            errors.append(f"missing catalog page: {page}")
    for family in WIDGET_FAMILIES:
        if family not in source:
            errors.append(f"missing widget family: {family}")
        if WIDGET_MOUNTS[family] not in source:
            errors.append(f"missing live widget mount: {family}")
    if "manual/assets/logo.jpg" not in source or "http://" in source or "https://" in source:
        errors.append("catalog must use manual/assets/logo.jpg")
    if "cargo run --locked --example widget_catalog" not in readme:
        errors.append("README omits the widget_catalog command")
    if "widget_catalog" not in manual or not ({"screenshot", "screenshots", "video"} & set(manual.lower().split())):
        errors.append("manual omits the catalog capture purpose")
    if "test result: ok." not in test_output or "0 failed" not in test_output:
        errors.append("catalog behavior tests did not pass")
    if "Finished `dev` profile" not in compile_output:
        errors.append("locked widget_catalog compile did not pass")
    frames = {line for line in pty_output.splitlines() if line.startswith("CATALOG FRAME ")}
    if len(frames) < 2:
        errors.append("PTY check did not observe two distinct cube frames")
    for key in ("CTRL_Q", "CTRL_C", "ESCAPE"):
        if f"{key} EXIT 0" not in pty_output:
            errors.append(f"PTY check did not prove {key} exit")
    if requirement == "all":
        return errors
    prefixes = {
        "CAT-001": ("missing catalog page", "missing widget family", "missing live widget mount"),
        "CAT-002": ("PTY check",),
        "CAT-003": ("catalog must", "README omits", "manual omits"),
    }[requirement] + ("catalog behavior tests", "locked widget_catalog compile")
    return [error for error in errors if error.startswith(prefixes)]


def run(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    return completed.returncode, completed.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("requirement", nargs="?", default="all", choices=("all", "CAT-001", "CAT-002", "CAT-003"))
    requirement = parser.parse_args().requirement
    validator_code, validator_output = run([sys.executable, "-B", "scripts/test-widget-catalog.py"])
    print(validator_output, end="")
    if validator_code != 0:
        return validator_code
    host_test_code, host_test_output = run([sys.executable, "-B", "scripts/test-widget-catalog-host.py"])
    print(host_test_output, end="")
    if host_test_code:
        return host_test_code

    source_dir = ROOT / "examples" / "widget_catalog"
    source = "\n".join(path.read_text() for path in sorted(source_dir.glob("*.rs"))) if source_dir.is_dir() else ""
    readme = (ROOT / "README.md").read_text()
    manual = (ROOT / "manual" / "README.md").read_text()

    test_code, test_output = run(
        ["cargo", "+1.91.0", "test", "--locked", "--test", "widget_catalog_behavior", "--jobs", "8"]
    )
    print(test_output, end="")
    compile_code, compile_output = run(
        ["cargo", "+1.91.0", "build", "--locked", "--example", "widget_catalog", "--jobs", "8"]
    )
    print(compile_output, end="")
    pty_code, pty_output = (0, "")
    if requirement in ("all", "CAT-002"):
        pty_code, pty_output = run([sys.executable, "-B", "scripts/check-widget-catalog-pty.py"])
    print(pty_output, end="")

    errors = validate(
        requirement=requirement,
        source=source,
        readme=readme,
        manual=manual,
        test_output=test_output,
        compile_output=compile_output,
        pty_output=pty_output,
    )
    if test_code != 0 and "catalog behavior tests did not pass" not in errors:
        errors.append("catalog behavior test command failed")
    if compile_code != 0 and "locked widget_catalog compile did not pass" not in errors:
        errors.append("locked widget_catalog compile command failed")
    if pty_code != 0 and not any(error.startswith("PTY check") for error in errors):
        errors.append("PTY check command failed")

    if errors:
        for error in errors:
            print(f"FAIL CAT: {error}", file=sys.stderr)
        return 1
    if requirement in ("all", "CAT-001", "CAT-002"):
        host_code, host_output = run([sys.executable, "-B", "scripts/check-widget-catalog-host.py", "--requirement", requirement])
        print(host_output, end="")
        if host_code:
            return host_code
    print(f"PASS {requirement}: catalog requirement checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
