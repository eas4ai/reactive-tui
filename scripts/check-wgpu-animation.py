#!/usr/bin/env python3
"""Validate animation guards, then test the actual clock, worker, and GPU."""

import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "pending: Option<FrameRequest>", "output: Option<WorkerOutput>",
    "ready.wait(mailbox)", "pending.replace(request)", "elapsed < deadline",
    "elapsed.checked_add(FRAME_INTERVAL)", "Duration::from_millis(50)",
    "pixel_count(columns, height)?", "mailbox.pending = None", "thread.join()",
)


def validate_animation(worker, renderer):
    compact = re.sub(r"\s+", "", worker)
    for token in REQUIRED:
        if re.sub(r"\s+", "", token) not in compact:
            raise ValueError(f"missing animation guard: {token}")
    if "VecDeque<FrameRequest>" in compact or "push_back(request)" in compact:
        raise ValueError("accumulating frame queue")
    for token in ("cube_angles(elapsed)", "elapsed.as_secs_f64()"):
        if token not in renderer:
            raise ValueError("rotation must use elapsed time")
    if "frame_count" in renderer:
        raise ValueError("frame-count rotation")


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        self.worker = " ".join(REQUIRED)
        self.renderer = "cube_angles(elapsed) elapsed.as_secs_f64()"

    def test_corrected_fixture(self):
        validate_animation(self.worker, self.renderer)

    def test_each_missing_guard_fails(self):
        for token in REQUIRED:
            with self.subTest(token=token), self.assertRaisesRegex(ValueError, "guard"):
                validate_animation(self.worker.replace(token, ""), self.renderer)

    def test_accumulating_queue_fails(self):
        with self.assertRaisesRegex(ValueError, "queue"):
            validate_animation(self.worker + "VecDeque<FrameRequest>", self.renderer)

    def test_frame_count_fails(self):
        with self.assertRaisesRegex(ValueError, "frame-count"):
            validate_animation(self.worker, self.renderer + " frame_count")
        with self.assertRaisesRegex(ValueError, "elapsed"):
            validate_animation(self.worker, "angle += 0.1")


def main():
    tests = unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests)
    if not unittest.TextTestRunner(verbosity=2).run(tests).wasSuccessful():
        return 1
    validate_animation((ROOT / "src/graphics/animation.rs").read_text(),
                       (ROOT / "src/graphics/mod.rs").read_text())
    minimum = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["rust-version"]
    command = ["cargo", f"+{minimum}.0", "test", "--locked", "-p", "reactive-tui",
               "--features", "wgpu-graphics", "--test", "wgpu_animation", "--", "--nocapture"]
    print("RUN " + " ".join(command), flush=True)
    result = subprocess.run(command, cwd=ROOT, env={**os.environ, "CARGO_TERM_COLOR": "never"},
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=1200)
    print(result.stdout, flush=True)
    if result.returncode:
        return result.returncode
    if not re.search(r"test result: ok\. [4-9][0-9]* passed; 0 failed;", result.stdout):
        raise ValueError("clock, queue, limits, and hardware tests must execute")
    if "GPU ANIMATION hardware=" not in result.stdout:
        raise ValueError("missing real hardware animation evidence")
    print("PASS GPU-003: timed requests, bounded slots, allocation limits, distinct hardware frames", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
