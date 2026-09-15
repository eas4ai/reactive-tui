#!/usr/bin/env python3
from pathlib import Path
import unittest

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-pre-release-library-diagnostics.py"
REQUIRED_PATHS = ("parser", "terminal", "reconciliation", "focus", "window")


def clean_observation():
    return {
        "source_audit": True,
        "audited_files": ["src/lib.rs"],
        "raw_macros": [],
        "capture": True,
        "stdout": "",
        "stderr": "",
        "exercised_paths": {name: True for name in REQUIRED_PATHS},
    }


class ObservationValidationTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(
            CHECKER,
            "library_diagnostics_checker",
            "library diagnostics checker",
        )

    def test_complete_silent_observation_is_accepted(self):
        self.assertEqual(self.checker.validate_observation(clean_observation()), [])

    def test_raw_output_macro_is_rejected(self):
        observation = clean_observation()
        observation["raw_macros"] = [
            {"path": "src/lib.rs", "line": 10, "macro": "eprintln"}
        ]
        self.assertIn(
            "src/lib.rs:10 reaches eprintln!",
            self.checker.validate_observation(observation),
        )

    def test_empty_source_audit_is_rejected(self):
        observation = dict(clean_observation(), audited_files=[])
        self.assertIn(
            "source audit reported no library files",
            self.checker.validate_observation(observation),
        )

    def test_failed_capture_is_rejected(self):
        observation = dict(clean_observation(), capture=False)
        self.assertIn(
            "captured-output probe failed",
            self.checker.validate_observation(observation),
        )

    def test_internal_stdout_and_stderr_are_rejected(self):
        for stream in ("stdout", "stderr"):
            with self.subTest(stream=stream):
                observation = dict(clean_observation(), **{stream: "internal diagnostic\n"})
                self.assertIn(
                    f"captured internal {stream} output",
                    self.checker.validate_observation(observation),
                )

    def test_every_required_diagnostic_path_must_run(self):
        for path in REQUIRED_PATHS:
            with self.subTest(path=path):
                observation = clean_observation()
                observation["exercised_paths"][path] = False
                self.assertIn(
                    f"diagnostic probe skipped {path}",
                    self.checker.validate_observation(observation),
                )


class CommandTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(
            CHECKER,
            "library_diagnostics_checker",
            "library diagnostics checker",
        )

    def test_probe_command_uses_supported_locked_toolchain(self):
        command = self.checker.capture_command()
        self.assertEqual(command[:2], ["cargo", "+1.91.0"])
        self.assertIn("--locked", command)
        self.assertIn("dqc_003_captured_diagnostics", command)


if __name__ == "__main__":
    unittest.main()
