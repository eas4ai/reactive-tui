#!/usr/bin/env python3
"""Use real temporary Git repositories to attack link validation."""
from pathlib import Path
import subprocess
import tempfile
import unittest

from dependency_check_test_support import load_checker


CHECKER = Path(__file__).with_name("check-pre-release-documentation.py")
# Fixture paths inside the temporary repository, not files of this repository.
GUIDE = "/".join(["manual", "guide.md"])


class DocumentationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(CHECKER, "rid_docs", "RID documentation checker")
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "--quiet")
        self.write("README.md", f"# Start\n[Manual]({GUIDE}#behavior)\n")
        self.write(GUIDE, "# Guide\n## Behavior\nWorks.\n")
        self.git("add", ".")

    def git(self, *arguments):
        return subprocess.run(["git", *arguments], cwd=self.root, check=True,
                              capture_output=True, text=True, timeout=10)

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)

    def test_tracked_links_pass(self):
        self.assertEqual(self.checker.inspect_repository(self.root), [])

    def test_missing_and_untracked_targets_fail(self):
        for target in ("missing.md", "local.md"):
            with self.subTest(target=target):
                self.write("README.md", f"[Target]({target})\n")
                self.write("local.md", "# Local\n")
                self.assertTrue(self.checker.inspect_repository(self.root))

    def test_missing_heading_and_encoded_escape_fail(self):
        for link in (GUIDE + "#absent", "../outside.md", "%2e%2e/outside.md"):
            with self.subTest(link=link):
                self.write("README.md", f"[Target]({link})\n")
                self.assertTrue(self.checker.inspect_repository(self.root))

    def test_images_reference_links_and_html_targets_are_checked(self):
        for text in ("![Logo](missing.png)", "[Guide][ref]\n[ref]: missing.md",
                     '<img src="missing.png">', '<a href="missing.md">Guide</a>'):
            with self.subTest(text=text):
                self.write("README.md", text)
                self.assertTrue(self.checker.inspect_repository(self.root))

    def test_external_links_and_code_examples_are_not_local_targets(self):
        self.write("README.md", "[Web](https://example.com/a)\n```md\n[Example](missing.md)\n```\n")
        self.assertEqual(self.checker.inspect_repository(self.root), [])


if __name__ == "__main__":
    unittest.main()
