#!/usr/bin/env python3
"""Attack offscreen-rendering wiring, then execute real hardware/frame tests.

Source fixtures are a guard, not GPU evidence. The Rust tests must execute on
hardware and inspect actual readback plus the existing terminal frame path.
Host-terminal visual acceptance is recorded separately; PNGs alone are not it.
"""

import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "create_texture", "RENDER_ATTACHMENT", "COPY_SRC", "begin_render_pass",
    "pass.draw", "copy_texture_to_buffer", "map_async", "get_mapped_range",
    "padded_row_bytes", "from_rgba", "to_half_block_element", "fg_rgba", "bg_rgba",
    "width as f32 / height as f32", "rows.checked_mul(2)",
)


def validate_renderer(source, shader):
    compact = re.sub(r"\s+", "", source)
    for token in REQUIRED:
        if re.sub(r"\s+", "", token) not in compact:
            raise ValueError(f"missing renderer operation: {token}")
    for token in ("create_surface", "winit", "SurfaceConfiguration"):
        if token in source:
            raise ValueError(f"separate window/surface path: {token}")
    for token in ("@fragment", "scene.aspect", "diffuse", "specular"):
        if token not in shader:
            raise ValueError(f"missing shaded/aspect shader: {token}")


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        self.source = " ".join(REQUIRED)
        self.shader = "@fragment scene.aspect diffuse specular"

    def test_corrected_fixture(self):
        validate_renderer(self.source, self.shader)

    def test_each_missing_operation_fails(self):
        for token in REQUIRED:
            with self.subTest(token=token), self.assertRaisesRegex(ValueError, "missing"):
                validate_renderer(self.source.replace(token, ""), self.shader)

    def test_placeholder_fails(self):
        with self.assertRaisesRegex(ValueError, "missing"):
            validate_renderer('div().text("cube")', self.shader)

    def test_window_paths_fail(self):
        for token in ("create_surface", "winit", "SurfaceConfiguration"):
            with self.subTest(token=token), self.assertRaisesRegex(ValueError, "window"):
                validate_renderer(self.source + token, self.shader)

    def test_shading_and_aspect_are_required(self):
        for token in self.shader.split():
            with self.subTest(token=token), self.assertRaisesRegex(ValueError, "shader"):
                validate_renderer(self.source, self.shader.replace(token, ""))


def main():
    tests = unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests)
    if not unittest.TextTestRunner(verbosity=2).run(tests).wasSuccessful():
        return 1
    source = (ROOT / "src/graphics/mod.rs").read_text()
    shader = (ROOT / "src/graphics/cube.wgsl").read_text()
    validate_renderer(source, shader)
    minimum = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["rust-version"]
    command = ["cargo", f"+{minimum}.0", "test", "--locked", "-p", "reactive-tui",
               "--features", "wgpu-graphics", "--test", "wgpu_graphics", "--", "--nocapture"]
    print("RUN " + " ".join(command), flush=True)
    result = subprocess.run(command, cwd=ROOT, env={**os.environ, "CARGO_TERM_COLOR": "never"},
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=1200)
    print(result.stdout, flush=True)
    if result.returncode:
        return result.returncode
    if not re.search(r"test result: ok\. [3-9][0-9]* passed; 0 failed;", result.stdout):
        raise ValueError("real adapter and frame tests did not all execute")
    if "GPU ADAPTER " not in result.stdout:
        raise ValueError("missing actual adapter identity")
    for viewport in ("60x24", "144x50", "200x60"):
        if f"GPU VIEWPORT {viewport} " not in result.stdout:
            raise ValueError(f"missing composition evidence for {viewport}")
    print("PASS GPU-002 automated: hardware pixels, resizing, aspect, terminal composition", flush=True)
    print("LIMIT: host-terminal visual capture remains separate from this automated check", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
