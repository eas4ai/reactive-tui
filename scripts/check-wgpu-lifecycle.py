#!/usr/bin/env python3
"""Attack fallback/ownership wiring, then run actual renderer and PTY checks."""

import os
from pathlib import Path
import re
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUARDS = {
    "hybrid": ("GraphicsMode::CpuFallback", "self.gpu = None", "cpu::render(request, cancellation)",
               "frame.mode = self.mode.clone()", "Err(GraphicsError::Cancelled)"),
    "worker": ("self.cancellation.cancel()", "mailbox.pending = None", "mailbox.stopped", "thread.join()"),
    "catalog": ("GraphicsCanvas::new(wake, self.graphics_options)", "graphics.advance(", "graphics.shutdown()"),
    "renderer": ("set_device_lost_callback", "on_uncaptured_error", "cancellation.is_cancelled()", "Duration::from_millis(10)"),
}


def validate_lifecycle(sources):
    for name, tokens in GUARDS.items():
        compact = re.sub(r"\s+", "", sources[name])
        for token in tokens:
            if re.sub(r"\s+", "", token) not in compact:
                raise ValueError(f"missing fallback/ownership guard: {name}: {token}")


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        self.sources = {name: " ".join(tokens) for name, tokens in GUARDS.items()}

    def test_corrected_fixture(self):
        validate_lifecycle(self.sources)

    def test_each_missing_guard_fails(self):
        for name, tokens in GUARDS.items():
            for token in tokens:
                with self.subTest(name=name, token=token), self.assertRaisesRegex(ValueError, "guard"):
                    candidate = dict(self.sources)
                    candidate[name] = candidate[name].replace(token, "")
                    validate_lifecycle(candidate)

    def test_mislabeled_cpu_fails(self):
        self.sources["hybrid"] = self.sources["hybrid"].replace("frame.mode = self.mode.clone()", "frame.mode = GraphicsMode::Gpu(info)")
        with self.assertRaisesRegex(ValueError, "guard"):
            validate_lifecycle(self.sources)

    def test_detached_thread_fails(self):
        self.sources["worker"] = self.sources["worker"].replace("thread.join()", "drop(thread)")
        with self.assertRaisesRegex(ValueError, "guard"):
            validate_lifecycle(self.sources)


def run(command):
    print("RUN " + " ".join(command), flush=True)
    result = subprocess.run(command, cwd=ROOT, env={**os.environ, "CARGO_TERM_COLOR": "never"},
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=1200)
    print(result.stdout, flush=True)
    if result.returncode:
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.stdout


def main():
    tests = unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests)
    if not unittest.TextTestRunner(verbosity=2).run(tests).wasSuccessful():
        return 1
    files = {"hybrid": "src/graphics/hybrid.rs", "worker": "src/graphics/animation.rs",
             "catalog": "examples/widget_catalog/catalog.rs", "renderer": "src/graphics/mod.rs"}
    validate_lifecycle({name: (ROOT / path).read_text() for name, path in files.items()})
    output = run(["cargo", "+1.91.0", "test", "--locked", "-p", "reactive-tui", "--features", "wgpu-graphics",
                  "--test", "wgpu_lifecycle", "--test", "widget_catalog_behavior", "--", "--nocapture"])
    if len(re.findall(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", output)) != 2:
        raise ValueError("lifecycle and integrated catalog targets must both execute")
    for fault in ("adapter", "device-loss", "readback"):
        if f"GPU FALLBACK fault={fault} mode=CPU fallback" not in output:
            raise ValueError(f"missing production fallback result: {fault}")
    run(["cargo", "+1.91.0", "build", "--locked", "-p", "reactive-tui", "--example", "widget_catalog", "--features", "wgpu-graphics"])
    pty = run([sys.executable, "-B", "scripts/check-wgpu-pty.py"])
    for name in ("CTRL_Q", "CTRL_C", "ESCAPE", "CPU", "adapter", "device-loss", "readback"):
        for proof in ("ANIMATION frames=2", "RESIZE 200x60", "EXIT 0 RESTORED CLEAN"):
            if f"GPU PTY {name} {proof}" not in pty:
                raise ValueError(f"missing PTY evidence: {name} {proof}")
    print("PASS GPU-004: failures select labeled CPU; active/pending work cancels; PTY quit keys restore", flush=True)
    print("LIMIT: PTY is not visual verification in a named desktop host", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
