#!/usr/bin/env python3
"""Check host diagnostic backpressure and cube-only pixel selection."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


HOST = load("host", "check-wgpu-host.py")
PTY = load("catalog_pty", "check-widget-catalog-pty.py")


class CatalogHostTests(unittest.TestCase):
    def test_host_diagnostics_cannot_fill_an_undrained_pipe(self):
        real_popen = subprocess.Popen
        with tempfile.TemporaryDirectory(prefix="catalog-log-test-") as private:
            host = HOST.Host.__new__(HOST.Host)
            host.directory, host.counter, host.env = private, 0, dict(os.environ)
            # Replace only the external GUI command. A real child writes more
            # than a pipe buffer, using the launcher's actual diagnostic sink.
            def writer(_command, **kwargs):
                return real_popen([sys.executable, "-c", "import os; os.write(2, b'diagnostic\\n' * 10000)"], **kwargs)

            with patch.object(HOST.subprocess, "Popen", side_effect=writer):
                host.launch(60, 24, [])
            try:
                try:
                    self.assertEqual(host.process.wait(timeout=2), 0)
                except subprocess.TimeoutExpired:
                    self.fail("owned host blocked while writing diagnostics")
                self.assertIn("diagnostic", host.log_output())
            finally:
                HOST.stop(host.process)
                for stream in (host.process.stdout, host.process.stderr):
                    if stream is not None:
                        stream.close()

    def test_cube_hash_ignores_labels_and_blank_braille(self):
        screen = PTY.Screen(12, 2)
        screen.feed("Motion ·◆⠀⠁⠂".encode())
        first = screen.cube()
        screen.feed(b"\x1b[1;1HLABELS")
        self.assertEqual(first, screen.cube())
        screen.feed("\x1b[1;11H⠃".encode())
        self.assertNotEqual(first, screen.cube())


if __name__ == "__main__":
    unittest.main()
