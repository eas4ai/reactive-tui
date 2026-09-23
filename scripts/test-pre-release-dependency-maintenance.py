#!/usr/bin/env python3
from pathlib import Path
import unittest

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-pre-release-dependency-maintenance.py"


def clean_observation():
    return {
        "audit": True,
        "graphs": {name: True for name in ("default", "minimal", "ffi", "markdown")},
        "unmaintained": [],
        "versions": {"taffy": ["0.9.2", "0.13.0"], "vte": ["0.15.0"]},
        "duplicate_reviews": {
            "taffy": "retain-isolated-taffy-lines-across-layout-engines",
            "vte": "",
        },
        "crossterm_packages": ["reactive-tui-crossterm"],
        "onig_default": False,
    }


class ObservationValidationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(
            CHECKER,
            "dependency_maintenance_checker",
            "dependency maintenance checker",
        )

    def test_complete_clean_observation_is_accepted(self):
        self.assertEqual(self.checker.validate_observation(clean_observation()), [])

    def test_every_feature_graph_must_finish(self):
        for name in ("default", "minimal", "ffi", "markdown"):
            with self.subTest(name=name):
                observation = clean_observation()
                observation["graphs"][name] = False
                self.assertIn(
                    f"{name} dependency graph failed",
                    self.checker.validate_observation(observation),
                )

    def test_audit_command_must_finish(self):
        observation = dict(clean_observation(), audit=False)
        self.assertIn("cargo audit failed", self.checker.validate_observation(observation))

    def test_unmaintained_production_package_needs_exact_decision(self):
        observation = clean_observation()
        observation["unmaintained"] = [
            {
                "advisory": "RUSTSEC-2099-0001",
                "package": "example",
                "version": "1.0.0",
                "decision": "",
            }
        ]
        self.assertIn(
            "RUSTSEC-2099-0001 example 1.0.0 lacks a reviewed maintenance decision",
            self.checker.validate_observation(observation),
        )

    def test_duplicate_taffy_or_vte_needs_review(self):
        for package in ("taffy", "vte"):
            with self.subTest(package=package):
                observation = clean_observation()
                observation["versions"][package] = ["1.0.0", "2.0.0"]
                observation["duplicate_reviews"][package] = ""
                self.assertIn(
                    f"duplicate {package} versions lack a reviewed decision: 1.0.0, 2.0.0",
                    self.checker.validate_observation(observation),
                )

    def test_missing_expected_dependency_is_incomplete_output(self):
        for package in ("taffy", "vte"):
            with self.subTest(package=package):
                observation = clean_observation()
                observation["versions"][package] = []
                self.assertIn(
                    f"dependency graphs did not report {package}",
                    self.checker.validate_observation(observation),
                )

    def test_crossterm_fork_identity_must_be_unambiguous(self):
        observation = clean_observation()
        observation["crossterm_packages"].append("crossterm")
        self.assertIn(
            "maintained crossterm fork identity is ambiguous",
            self.checker.validate_observation(observation),
        )

    def test_default_graph_rejects_oniguruma(self):
        observation = dict(clean_observation(), onig_default=True)
        self.assertIn(
            "default dependency graph contains onig_sys",
            self.checker.validate_observation(observation),
        )


class GraphParsingTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(
            CHECKER,
            "dependency_maintenance_checker",
            "dependency maintenance checker",
        )

    def test_package_versions_are_distinct_and_sorted(self):
        tree = "taffy v0.13.0\nreactive-tui v0.1.0\ntaffy v0.9.2\ntaffy v0.13.0\n"
        self.assertEqual(self.checker.package_versions(tree, "taffy"), ["0.9.2", "0.13.0"])

    def test_graph_commands_cover_required_feature_sets_and_lockfile(self):
        commands = self.checker.graph_commands()
        self.assertEqual(set(commands), {"default", "minimal", "ffi", "markdown"})
        for command in commands.values():
            self.assertEqual(command[:2], ["cargo", "+1.91.0"])
            self.assertIn("--locked", command)
            self.assertIn("--target", command)
        self.assertIn("--no-default-features", commands["minimal"])
        self.assertIn("ffi", commands["ffi"])
        self.assertEqual(commands["markdown"][-2:], ["-e", "features"])

    def test_duplicate_decision_names_versions_and_reason(self):
        self.assertFalse(
            self.checker.duplicate_decision_is_complete(
                "taffy", ["0.9.2", "0.13.0"], "taffy 0.9.2 and taffy 0.13.0"
            )
        )
        self.assertTrue(
            self.checker.duplicate_decision_is_complete(
                "taffy",
                ["0.9.2", "0.13.0"],
                "taffy 0.9.2\ntaffy 0.13.0\nReason: the APIs require separate migrations",
            )
        )


if __name__ == "__main__":
    unittest.main()
