"""Safe negative and positive controls for the API documentation mechanism."""
from pathlib import Path
import runpy
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
CHECK = runpy.run_path(str(ROOT / "scripts/check-api-documentation.py"))


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
