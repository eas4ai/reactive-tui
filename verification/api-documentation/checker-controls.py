"""Safe negative and positive controls for the API documentation mechanism."""
from pathlib import Path
import hashlib
import inspect
import io
import json
import runpy
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
CHECK = runpy.run_path(str(ROOT / "scripts/check-api-documentation.py"))
# Fixture paths inside a temporary tree, not files of this repository.
NESTED_MANUAL = "/".join(["manual", "nested"])
NESTED_EXAMPLE = NESTED_MANUAL + "/example.md"


class ConsumerCaptureControls(unittest.TestCase):
    def test_consumer_library_captures_keep_separate_matching_hashes(self):
        self.assertIn("name", inspect.signature(CHECK["Check"].library).parameters)
        with tempfile.TemporaryDirectory() as temporary:
            check = CHECK["Check"].__new__(CHECK["Check"])
            check.output, check.steps = Path(temporary), []
            outputs = iter(json.dumps({"reason": "compiler-artifact",
                "target": {"name": "reactive_tui"}, "filenames": [name + ".rlib"]})
                for name in ("props", "examples"))
            def execute(command, log, timeout):
                output = next(outputs)
                log.write_text(output)
                return output
            check.execute = execute
            with patch("sys.stdout", new=io.StringIO()), \
                    patch.dict(CHECK["Check"].run.__globals__, ROOT=check.output):
                self.assertEqual(check.library("props-consumer-library"), "props.rlib")
                self.assertEqual(check.library("examples-consumer-library"), "examples.rlib")
            self.assertNotEqual(check.steps[0]["log"], check.steps[1]["log"])
            for step in check.steps:
                self.assertEqual(step["sha256"], hashlib.sha256(
                    (check.output / step["log"]).read_bytes()).hexdigest())


class RetainedDocumentationControls(unittest.TestCase):
    def test_matrix_uses_the_retained_manual_path(self):
        self.assertEqual(CHECK["MATRIX"], ROOT / "manual/supported-api.md")

    def test_matrix_accepts_graphics_behavior_requirements_but_not_unknown_names(self):
        row = "| `graphics` | Offscreen canvas | GPU-001 | Opt-in | Covered by named checks |\n"
        self.assertEqual(CHECK["matrix_modules"](row), {"graphics"})
        with self.assertRaises(AssertionError):
            CHECK["matrix_modules"](row.replace("GPU-001", "UNKNOWN-001"))

    def test_nested_manual_examples_enter_the_compilation_inventory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for directory in (NESTED_MANUAL, "include", "bindings/typescript", "src", "crates/reactive-tui-macros/src", "output"):
                (root / directory).mkdir(parents=True, exist_ok=True)
            (root / "README.md").write_text("```rust,no_run\nlet value = 1;\n```\n")
            (root / "include/README.md").write_text("```c\nint main(void) { return 0; }\n```\n")
            (root / "bindings/typescript/README.md").write_text("```typescript\nconst value = 1;\n```\n")
            (root / "crates/reactive-tui-macros/src/lib.rs").write_text("")
            manual = root / NESTED_EXAMPLE
            manual.write_text("# Example\n```rust,no_run\nlet nested = 2;\n```\n\n```python\nassert True\n```\n")
            check = CHECK["Check"].__new__(CHECK["Check"])
            check.output = root / "output"
            with patch.dict(CHECK["Check"].collect_examples.__globals__, ROOT=root):
                rust, _, _, python = check.collect_examples()
            self.assertIn(NESTED_EXAMPLE, [name for name, _ in rust])
            self.assertIn(NESTED_EXAMPLE, [name for name, _, _ in python])
            inventory = json.loads((check.output / "examples.json").read_text())
            self.assertEqual([entry["language"] for entry in inventory
                              if entry["file"] == NESTED_EXAMPLE], ["rust", "python"])

    def test_inventory_rejects_missing_modules_and_missing_linked_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "src").mkdir()
            (root / "manual").mkdir()
            (root / "src/lib.rs").write_text("pub mod markdown;\n")
            matrix = root / "manual/supported-api.md"
            limits = "Orca GNOME Terminal iTerm2 3.7 Kitty patch validate Option<T>\n"
            rows = "| `markdown` | Conversion | API-018 | Host limits | Covered by named checks |\n"
            rows += "| `macros` | Props | API-018 | Explicit validation | Covered by named checks |\n"
            matrix.write_text(limits + rows)
            check = CHECK["Check"].__new__(CHECK["Check"])
            with patch.dict(CHECK["Check"].inventory.__globals__, ROOT=root, MATRIX=matrix):
                check.inventory()
                matrix.write_text(limits + rows.splitlines()[0] + "\n")
                with self.assertRaisesRegex(AssertionError, "missing"):
                    check.inventory()
                matrix.write_text(limits + rows + "[Evidence](../missing.rs)\n")
                with self.assertRaisesRegex(AssertionError, "Broken.*link"):
                    check.inventory()


class DocumentationControls(unittest.TestCase):
    def test_fences_preserve_separate_examples_and_flags(self):
        text = "prose\n```rust, ignore\nlet x = 1;\n```\n\n```c\nint x;\n```\n"
        self.assertEqual(list(CHECK["fences"](text)), [
            (2, ["rust", "ignore"], "let x = 1;\n"), (6, ["c"], "int x;\n")])

    def test_matrix_rejects_missing_fields_duplicate_modules_and_unclear_status(self):
        row = "| `markdown` | Conversion | API-018 | Explicit limits | Verified with limits |\n"
        self.assertEqual(CHECK["matrix_modules"](row), {"markdown"})
        for bad in (row + row, row.replace("Explicit limits", ""), row.replace("API-018", ""),
                    row.replace("Verified with limits", "Ready")):
            with self.subTest(row=bad), self.assertRaises(AssertionError):
                CHECK["matrix_modules"](bad)

    def test_markdown_requires_index_entry_and_all_public_submodules(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            index = '<a href="markdown/index.html">Markdown</a>'
            with self.assertRaises(AssertionError):
                CHECK["require_markdown"](index, root)
            for page in ("index.html", "renderer/index.html", "converter/index.html", "ast_walker/index.html"):
                path = root / "markdown" / page
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("public API")
            CHECK["require_markdown"](index, root)
            with self.assertRaises(AssertionError):
                CHECK["require_markdown"]("hidden module", root)
            (root / "markdown/converter/index.html").unlink()
            with self.assertRaises(AssertionError):
                CHECK["require_markdown"](index, root)

    def test_zero_executed_tests_and_failed_nested_step_cannot_pass(self):
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as temporary:
            check = CHECK["Check"].__new__(CHECK["Check"])
            check.output, check.steps = Path(temporary), []
            check.execute = lambda command, log, timeout: "test result: ok. 0 passed; 0 failed; 1 ignored"
            self.assertIsNone(check.run("zero-cases", ["simulated-output"], expected_tests=1))
            self.assertEqual(check.steps[-1]["result"], "fail")
            check.execute = lambda command, log, timeout: "test result: ok. 1 passed; 0 failed; 0 ignored"
            self.assertIsNotNone(check.run("one-case", ["simulated-output"], expected_tests=1))
            check.inspect("parent", lambda: check.steps.append({"name": "nested", "result": "fail"}))
            self.assertEqual(check.steps[-1]["result"], "fail")


if __name__ == "__main__":
    unittest.main(verbosity=2)
