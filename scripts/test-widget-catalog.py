#!/usr/bin/env python3
"""Falsifier tests for the widget catalog acceptance checker."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check-widget-catalog.py"
SPEC = importlib.util.spec_from_file_location("widget_catalog_check", CHECKER_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


GOOD_SOURCE = "\n".join(
    [
        'const LOGO: &str = "manual/assets/logo.jpg";',
        "fn input_page() {}",
        "fn layout_page() {}",
        "fn data_page() {}",
        "fn menus_dialogs_page() {}",
        "fn media_page() {}",
        "fn motion_page() {}",
        "fn system_page() {}",
        "Checkbox RadioButton Select Slider TextInput",
        "Accordion Breadcrumb ScrollView Stack Tabs",
        "Chart Table DataTable Tree FileExplorer ProgressBar Modal Popover Image",
        "MenuBar ContextMenu PopupMenu DialogMenu",
        "ConfirmationDialog InputDialog AutocompleteDialog ProgressDialog Toast WizardDialog",
        "TerminalWidget",
    ]
)
GOOD_SOURCE += "\n" + "\n".join(CHECKER.WIDGET_MOUNTS.values())
GOOD_README = "cargo run --locked --example widget_catalog"
GOOD_MANUAL = "widget_catalog screenshots video"
GOOD_TEST = "test result: ok. 8 passed; 0 failed"
GOOD_COMPILE = "Finished `dev` profile"
GOOD_PTY = "CATALOG FRAME 1\nCATALOG FRAME 2\nCTRL_Q EXIT 0\nCTRL_C EXIT 0\nESCAPE EXIT 0"


class WidgetCatalogValidationTests(unittest.TestCase):
    def validate(self, **overrides: str) -> list[str]:
        values = {
            "source": GOOD_SOURCE,
            "readme": GOOD_README,
            "manual": GOOD_MANUAL,
            "test_output": GOOD_TEST,
            "compile_output": GOOD_COMPILE,
            "pty_output": GOOD_PTY,
        }
        values.update(overrides)
        return CHECKER.validate(**values)

    def test_accepts_complete_evidence(self) -> None:
        self.assertEqual(self.validate(), [])

    def test_shell_ignores_unfinished_motion_and_docs(self) -> None:
        self.assertEqual(self.validate(requirement="CAT-001", pty_output="", readme="", manual=""), [])

    def test_motion_ignores_unfinished_docs(self) -> None:
        self.assertEqual(self.validate(requirement="CAT-002", readme="", manual=""), [])

    def test_media_ignores_unfinished_motion(self) -> None:
        self.assertEqual(self.validate(requirement="CAT-003", pty_output=""), [])

    def test_rejects_missing_widget_family_page(self) -> None:
        source = GOOD_SOURCE.replace("fn data_page() {}\n", "")
        self.assertIn("missing catalog page: data_page", self.validate(source=source))

    def test_rejects_inventory_label_without_a_live_mount(self) -> None:
        source = GOOD_SOURCE.replace("InputDialog::new(", "")
        self.assertIn("missing live widget mount: InputDialog", self.validate(requirement="CAT-001", source=source))

    def test_rejects_wrong_logo(self) -> None:
        source = GOOD_SOURCE.replace("manual/assets/logo.jpg", "https://example.com/logo.jpg")
        self.assertIn("catalog must use manual/assets/logo.jpg", self.validate(source=source))

    def test_rejects_compile_only_evidence(self) -> None:
        self.assertIn(
            "catalog behavior tests did not pass",
            self.validate(test_output="not run"),
        )

    def test_rejects_identical_animation_frames(self) -> None:
        pty = GOOD_PTY.replace("CATALOG FRAME 2", "CATALOG FRAME 1")
        self.assertIn("PTY check did not observe two distinct cube frames", self.validate(pty_output=pty))

    def test_rejects_quit_that_does_not_exit(self) -> None:
        pty = GOOD_PTY.replace("CTRL_Q EXIT 0", "CTRL_Q STILL RUNNING")
        self.assertIn("PTY check did not prove CTRL_Q exit", self.validate(pty_output=pty))

    def test_rejects_missing_documentation(self) -> None:
        self.assertIn("README omits the widget_catalog command", self.validate(readme=""))


if __name__ == "__main__":
    unittest.main()
