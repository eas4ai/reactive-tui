#!/usr/bin/env python3
"""Attack repository housekeeping with real temporary Git repositories."""
from pathlib import Path
import subprocess
import tempfile
import unittest

from dependency_check_test_support import load_checker


class HousekeepingTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(Path(__file__).with_name("check-pre-release-housekeeping.py"),
                                    "rid_housekeeping", "RID housekeeping checker")
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "--quiet")
        self.write("src/lib.rs", "pub fn example() {}\n")
        self.write("Cargo.toml", '[package]\nname="fixture"\nversion="0.1.0"\ninclude=["/src/**"]\n')
        self.write(".gitignore", "target/\n*.log\n")
        self.git("add", ".")

    def git(self, *arguments):
        return subprocess.run(["git", *arguments], cwd=self.root, check=True,
                              capture_output=True, text=True, timeout=10)

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def test_valid_repository_with_future_build_noise_passes(self):
        self.assertEqual(self.checker.inspect_repository(self.root), [])

    def test_os_noise_remains_local(self):
        self.write(".gitignore", ".DS_Store\nThumbs.db\n")
        self.assertEqual(self.checker.inspect_repository(self.root), [])

    def test_tracked_ignored_file_is_rejected(self):
        self.write(".gitignore", "src/\n")
        self.assertTrue(any("tracked file is ignored" in error
                            for error in self.checker.inspect_repository(self.root)))

    def test_negative_ignore_exception_is_respected(self):
        self.write(".gitignore", "src/\n!src/\n!src/lib.rs\n")
        self.assertEqual(self.checker.inspect_repository(self.root), [])

    def test_removed_literal_ignore_and_exception_are_rejected(self):
        for pattern in ("removed-example/", "!removed-guide.md", "/removed/**"):
            with self.subTest(pattern=pattern):
                self.write(".gitignore", pattern + "\n")
                self.assertTrue(any("stale ignore" in error
                                    for error in self.checker.inspect_repository(self.root)))

    def test_removed_package_include_and_exclusion_are_rejected(self):
        for key, pattern in (("include", "/removed/**"), ("exclude", "removed/**"),
                             ("include", "!/removed/**")):
            with self.subTest(key=key, pattern=pattern):
                self.write("Cargo.toml", f'[package]\nname="fixture"\nversion="0.1.0"\n{key}=["{pattern}"]\n')
                self.assertTrue(any("stale package" in error
                                    for error in self.checker.inspect_repository(self.root)))

    def test_nested_package_rules_use_the_package_directory(self):
        self.write("nested/src/lib.rs", "pub fn nested() {}\n")
        self.write("nested/Cargo.toml", '[package]\nname="nested"\nversion="0.1.0"\ninclude=["/src/**"]\n')
        self.git("add", "nested")
        self.assertEqual(self.checker.inspect_repository(self.root), [])

    def test_malformed_package_document_is_reported(self):
        self.write("Cargo.toml", "[package\n")
        self.assertTrue(any("invalid package" in error
                            for error in self.checker.inspect_repository(self.root)))


if __name__ == "__main__":
    unittest.main()
