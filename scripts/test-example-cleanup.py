#!/usr/bin/env python3
"""Validator tests for the EXC-001 example cleanup check."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest


def load_checker():
    path = Path(__file__).with_name("check-example-cleanup.py")
    spec = importlib.util.spec_from_file_location("example_cleanup_checker", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load checker from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CHECKER = load_checker()


class ExampleCleanupTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        examples = self.root / "examples"
        examples.mkdir()
        (examples / "gradient_blocks.rs").write_text("fn main() {}\n")

    def tearDown(self):
        self.directory.cleanup()

    @staticmethod
    def successful_compile(*_args, **_kwargs):
        return subprocess.CompletedProcess([], 0, "", "")

    def validate(self, tracked_paths=(), compile_runner=None):
        return CHECKER.validate(
            self.root,
            tracked_paths=tracked_paths,
            compile_runner=compile_runner or self.successful_compile,
        )

    def test_accepts_gradient_as_the_only_public_example(self):
        self.assertEqual(self.validate(), [])

    def test_rejects_a_removed_example_file(self):
        (self.root / "examples" / "animated_patterns.rs").write_text("fn main() {}\n")
        self.assertIn("unexpected public examples: animated_patterns.rs", self.validate())

    def test_rejects_a_missing_gradient_example(self):
        (self.root / "examples" / "gradient_blocks.rs").unlink()
        self.assertIn("missing public examples: gradient_blocks.rs", self.validate())

    def test_rejects_a_tracked_reference_to_a_removed_example(self):
        readme = self.root / "README.md"
        readme.write_text("cargo run --example embedded_shell\n")
        errors = self.validate(tracked_paths=[readme])
        self.assertEqual(errors, ["stale removed-example reference: README.md:1: embedded_shell"])

    def test_ignores_dialog_engine_api_symbol_names(self):
        source = self.root / "bindings.rs"
        source.write_text("pub fn rtui_dialog_engine_create() {}\n")
        self.assertEqual(self.validate(tracked_paths=[source]), [])

    def test_allows_a_cairn_backlog_to_name_a_removed_example(self):
        backlog = self.root / ".cairn" / "backlog" / "runtime-bug.md"
        backlog.parent.mkdir(parents=True)
        backlog.write_text("embedded_shell crashed in Kitty\n")
        self.assertEqual(self.validate(tracked_paths=[backlog]), [])

    def test_rejects_a_failed_gradient_compile(self):
        def failed_compile(*_args, **_kwargs):
            return subprocess.CompletedProcess([], 101, "", "compile failed")

        self.assertIn("gradient_blocks failed to compile", self.validate(compile_runner=failed_compile))


if __name__ == "__main__":
    unittest.main()
