"""Deterministic host acceptance regressions; no desktop or GPU required."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("wgpu_host", Path(__file__).with_name("check-wgpu-host.py"))
host = importlib.util.module_from_spec(spec)
spec.loader.exec_module(host)


class FrameSynchronizationTests(unittest.TestCase):
    def test_incomplete_frame_after_capture_is_rejected(self):
        value = host.Host.__new__(host.Host)
        value.env = {"DISPLAY": ":owned-test"}
        value.wait_frame = unittest.mock.Mock()
        value.frame_ready = unittest.mock.Mock(return_value=False)
        with patch.object(host.subprocess, "run") as capture:
            with self.assertRaisesRegex(RuntimeError, "not accepted"):
                value.capture(60, 24, Path("owned-new-capture.png"))
        capture.assert_called_once()
        value.frame_ready.assert_called_once_with(60, 24, "GPU ·")

    def test_stale_valid_frame_before_resize_loading_cannot_pass(self):
        screen = "Reactive TUI · Widget Catalog\nGPU · hardware\n" + "▀" * 900 + "\nCtrl+Q quit"
        loading = "Reactive TUI · Widget Catalog\nStarting graphics\nPreparing viewport\nCtrl+Q quit"
        value = host.Host.__new__(host.Host)
        value.process = unittest.mock.Mock()
        value.process.poll.return_value = None
        value.remote = unittest.mock.Mock(side_effect=[
            item for text in [screen, loading, screen, screen, screen, screen]
            for item in ['[{"tabs":[{"windows":[{"columns":60,"lines":24}]}]}]', text]
        ])
        with patch.object(host.time, "sleep"), patch.object(host.time, "monotonic", side_effect=range(20)):
            value.wait_frame(60, 24, "GPU ·")
        self.assertGreaterEqual(value.remote.call_count, 10, "accepted the pre-resize stale frame")


if __name__ == "__main__":
    unittest.main(verbosity=2)
