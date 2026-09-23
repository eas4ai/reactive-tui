#!/usr/bin/env python3
"""Regression tests for the closure runner's exact Cargo executable selection."""
import importlib.util
import inspect
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("api_closure", ROOT / ".cairn/api-closure/check.py")
CHECK = importlib.util.module_from_spec(spec)
spec.loader.exec_module(CHECK)


class ProbeExecutableTests(unittest.TestCase):
    def setUp(self):
        output = patch("sys.stdout", new=io.StringIO())
        output.start()
        self.addCleanup(output.stop)
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        control = self.root / "control.json"
        control.write_text(json.dumps({"result": "not an acceptance pass",
                                       "surviving_capture_processes": [123],
                                       "running_after_manual_cleanup": []}))
        review = self.root / "review.md"
        review.write_text(CHECK.REVIEW_HEADING + "\nStatus: Complete\nKnown open findings: none\n"
                          "Historical defect controls are not acceptance evidence\n"
                          + " ".join(f"RAPI-{n:02d}" for n in range(1, 16)) + "\n"
                          + " ".join(f"API-{n:03d}" for n in range(1, 21)))
        todo = self.root / "todo.md"
        todo.write_text("- Done: API-020\n")
        for name, value in (("ROOT", self.root), ("CONTROL", control),
                            ("REVIEW", review), ("TODO", todo)):
            guard = patch.object(CHECK, name, value)
            guard.start()
            self.addCleanup(guard.stop)

    @staticmethod
    def artifact(name, kind, executable):
        return json.dumps({"reason": "compiler-artifact",
                           "target": {"name": name, "kind": [kind]},
                           "executable": executable})

    def test_both_captures_use_exact_cargo_example_executable(self):
        executable = str(self.root / "private target" / "image_host_probe")
        output = "\n".join(("Cargo status line",
                            self.artifact("image_host_probe", "lib", "/wrong/library"),
                            self.artifact("different_probe", "example", "/wrong/example"),
                            self.artifact("image_host_probe", "example", executable)))
        result = subprocess.CompletedProcess([], 0, stdout=output)
        with patch.object(CHECK.subprocess, "run", return_value=result), \
                patch.object(CHECK, "run_capture", return_value={}) as capture:
            CHECK.main()
        calls = capture.call_args_list
        self.assertEqual(len(calls), 2)
        self.assertEqual([call.args[2] for call in calls], [False, True])
        self.assertEqual([call.args[3:] for call in calls], [(executable,), (executable,)])

    def test_missing_cargo_example_executable_is_rejected_before_capture(self):
        output = self.artifact("image_host_probe", "example", None)
        result = subprocess.CompletedProcess([], 0, stdout=output)
        with patch.object(CHECK.subprocess, "run", return_value=result), \
                patch.object(CHECK, "run_capture", return_value={}) as capture:
            with self.assertRaisesRegex(RuntimeError, "Cargo.*executable"):
                CHECK.main()
        capture.assert_not_called()

    def test_cargo_failure_is_not_masked(self):
        failure = subprocess.CalledProcessError(101, ["cargo", "build"])
        with patch.object(CHECK.subprocess, "run", side_effect=failure), \
                patch.object(CHECK, "run_capture") as capture:
            with self.assertRaises(subprocess.CalledProcessError):
                CHECK.main()
        capture.assert_not_called()

    def test_capture_forwards_explicit_executable_as_one_argument(self):
        self.assertIn("executable", inspect.signature(CHECK.run_capture).parameters)
        executable = str(self.root / "private target" / "image_host_probe")
        with patch.object(CHECK.subprocess, "Popen", side_effect=OSError("stop before spawn")) as spawn:
            with self.assertRaisesRegex(OSError, "stop before spawn"):
                CHECK.run_capture("normal", self.root, False, executable)
        self.assertEqual(spawn.call_args.args[0], [CHECK.CAPTURE_PYTHON, "-B",
                         str(CHECK.CAPTURE), "gnome", str(self.root / "normal"),
                         "app-auto", executable])


if __name__ == "__main__":
    unittest.main()
