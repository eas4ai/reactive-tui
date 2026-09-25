#!/usr/bin/env python3
"""Check image runner orchestration without launching a terminal host."""
from contextlib import ExitStack
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("api_images", ROOT / "scripts/check-api-images.py")
CHECK = importlib.util.module_from_spec(spec)
spec.loader.exec_module(CHECK)
DRIVER = ["/usr/bin/python3", "-B", "tests/api_widget_behavior/image_host_capture.py"]


class ImageProbeLaunchTests(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.probe = str(self.root / "private target" / "image_host_probe")
        self.build_probe = Mock(return_value=self.probe)
        self.ensure_host = Mock(return_value=(self.root / "host/kitty", {}))
        self.captures = []
        self.negative_count = 0
        guards = ExitStack()
        self.addCleanup(guards.close)
        guards.enter_context(patch.object(CHECK, "ROOT", self.root))
        guards.enter_context(patch.object(CHECK.os, "chdir"))
        guards.enter_context(patch.dict(CHECK.os.environ))
        guards.enter_context(patch.object(CHECK.shutil, "which", side_effect=lambda name: "/usr/bin/" + name))
        guards.enter_context(patch.object(CHECK.runpy, "run_path", side_effect=self.module))
        guards.enter_context(patch("sys.stdout", new=io.StringIO()))

    def module(self, path):
        if str(path).endswith("scripts/check-widget-platforms.py"):
            return {"execute": self.execute, "build_probe": self.bounded_probe}
        if str(path).endswith("scripts/kitty-host/build.py"):
            return {"ensure_host": self.ensure_host, "digest": lambda path: "a" * 64}
        raise AssertionError("Unexpected runner dependency: " + str(path))

    def bounded_probe(self, output):
        return self.build_probe()

    def execute(self, command, output, timeout):
        if command[:3] == DRIVER:
            self.captures.append(command)
            directory = Path(command[4])
            directory.mkdir(parents=True, exist_ok=True)
            pixels = [{color: {"count": self.negative_count} for color in
                       ("red", "blue", "green", "yellow")} for _ in range(3)]
            (directory / "pixels.json").write_text(json.dumps(pixels))
            if len(command) > 5 and command[5] == "app-ascii":
                output.write_text("AssertionError: controlled missing pixels")
                raise RuntimeError("Controlled pixel rejection")
        return "test result: ok. 1 passed; 0 failed;"

    def test_all_rust_routes_forward_selected_probe_as_one_argument(self):
        CHECK.main()
        rust = [command for command in self.captures if command[4].split("/")[-1] != "kitty-independent"]
        self.assertEqual(len(rust), len(CHECK.HOSTS) + 1)
        self.assertEqual(rust[0][3:4] + rust[0][5:], ["kitty", "app-ascii", self.probe])
        self.assertEqual([(command[3], command[5]) for command in rust[1:]],
                         [(host, mode or "app-auto") for host, mode in CHECK.HOSTS])
        self.assertTrue(all(command[6:] == [self.probe] for command in rust))
        self.build_probe.assert_called_once_with()

    def test_missing_cargo_artifact_stops_before_pixel_capture(self):
        self.build_probe.side_effect = RuntimeError("Cargo did not report the example executable")
        with self.assertRaisesRegex(RuntimeError, "Cargo.*executable"):
            CHECK.main()
        self.assertEqual(self.captures, [])

    def test_cargo_failure_is_not_masked(self):
        self.build_probe.side_effect = subprocess.CalledProcessError(101, ["cargo", "build"])
        with self.assertRaises(subprocess.CalledProcessError):
            CHECK.main()
        self.assertEqual(self.captures, [])

    def test_independent_c_reference_is_not_replaced_with_the_rust_probe(self):
        CHECK.main()
        reference = [command for command in self.captures if command[4].split("/")[-1] == "kitty-independent"]
        self.assertEqual(len(reference), 1)
        self.assertEqual(reference[0][3:4] + reference[0][5:],
                         ["kitty", "kitty", str(self.root / "target/kitty-image-host/reference")])

    def test_ascii_negative_with_image_colors_is_not_accepted(self):
        self.negative_count = 1
        with self.assertRaisesRegex(RuntimeError, "Forced ASCII.*rejection"):
            CHECK.main()


if __name__ == "__main__":
    unittest.main()
