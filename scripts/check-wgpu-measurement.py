#!/usr/bin/env python3
"""GPU-005: report falsifiers plus actual bounded native-host measurements."""
import importlib.util
import json
import math
import os
from pathlib import Path
import struct
import subprocess
import unittest
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
STAGES = ["render", "readback", "conversion", "presentation", "total"]


def validate(report, mode, size):
    assert report["schema"] == 1
    assert report["mode"] == mode
    assert report["viewport"] == {"columns": size[0], "rows": size[1]}
    assert report["terminal"]["term"] == "xterm-kitty"
    assert report["terminal"]["kitty_pid_present"] is True
    reference = report["comparison_adapter"]
    assert reference["hardware"] is True and reference["name"] and reference["backend"]
    assert report["adapter"] and (report["adapter"] == reference["name"] if mode == "GPU" else report["adapter"].startswith("CPU"))
    for value in [report["requested_seconds"], report["sampled_seconds"], report["achieved_fps"], *report["mean_ms"].values()]:
        assert isinstance(value, (int, float)) and math.isfinite(value) and value >= 0
    assert 0.25 <= report["requested_seconds"] <= 30
    assert report["sampled_seconds"] >= report["requested_seconds"]
    assert isinstance(report["frames"], int) and report["frames"] > 0
    assert math.isclose(report["achieved_fps"], report["frames"] / report["sampled_seconds"], rel_tol=1e-6)
    costs = report["mean_ms"]
    assert all(costs[key] > 0 for key in ["render", "conversion", "presentation", "total"])
    assert costs["readback"] > 0 if mode == "GPU" else costs["readback"] == 0
    assert costs["total"] + 0.001 >= sum(costs[key] for key in STAGES[:-1])
    quality = report["quality"]
    assert quality["pixels"] == size[0] * size[1] * 2
    assert 0 <= quality["differing_pixels"] <= quality["pixels"]
    assert 0 <= quality["max_channel_error"] <= 255
    assert math.isfinite(quality["mean_absolute_rgb_error"]) and 0 <= quality["mean_absolute_rgb_error"] <= 255
    assert quality["description"] and quality["elapsed_seconds"] == 1
    assert "Excludes initialization, App reconciliation, terminal display scanout" in report["timing_scope"]


def fixture():
    return {"schema": 1, "mode": "GPU", "adapter": "hardware", "comparison_adapter": {"hardware": True, "name": "hardware", "backend": "Vulkan"},
            "terminal": {"term": "xterm-kitty", "kitty_pid_present": True}, "viewport": {"columns": 60, "rows": 24},
            "requested_seconds": 1, "sampled_seconds": 1, "frames": 10, "achieved_fps": 10,
            "mean_ms": dict(zip(STAGES, [1, 2, 3, 4, 10])),
            "quality": {"pixels": 2880, "differing_pixels": 2, "max_channel_error": 1, "mean_absolute_rgb_error": 0.01, "description": "quantization", "elapsed_seconds": 1},
            "timing_scope": "Excludes initialization, App reconciliation, terminal display scanout"}


class ValidatorTests(unittest.TestCase):
    def test_corrected_fixture(self):
        validate(fixture(), "GPU", (60, 24))
        cpu = fixture()
        cpu.update(mode="CPU", adapter="CPU (explicit selection)")
        cpu["mean_ms"]["readback"] = 0
        validate(cpu, "CPU", (60, 24))

    def test_every_missing_field_fails(self):
        for key in fixture():
            broken = fixture()
            del broken[key]
            with self.assertRaises((AssertionError, KeyError)):
                validate(broken, "GPU", (60, 24))
        for key in STAGES:
            broken = fixture()
            del broken["mean_ms"][key]
            with self.assertRaises((AssertionError, KeyError)):
                validate(broken, "GPU", (60, 24))

    def test_software_and_mislabeled_fallback_fail(self):
        broken = fixture()
        broken["comparison_adapter"]["hardware"] = False
        with self.assertRaises(AssertionError):
            validate(broken, "GPU", (60, 24))
        broken = fixture()
        broken["adapter"] = "CPU fallback"
        with self.assertRaises(AssertionError):
            validate(broken, "GPU", (60, 24))

    def test_nonfinite_hidden_cost_and_false_fps_fail(self):
        for key, value in [("readback", 0), ("render", float("nan")), ("total", 1)]:
            broken = fixture()
            broken["mean_ms"][key] = value
            with self.assertRaises(AssertionError):
                validate(broken, "GPU", (60, 24))
        broken = fixture()
        broken["achieved_fps"] = 60
        with self.assertRaises(AssertionError):
            validate(broken, "GPU", (60, 24))


def main():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(ValidatorTests))
    assert result.wasSuccessful(), "validator self-tests failed"
    manual = (ROOT / "manual/wgpu-graphics.md").read_text()
    for text in ["--features wgpu-graphics", "--cpu", "--graphics-fault", "wgpu_benchmark", "800x600", "Xvfb", "scanout", "unverified"]:
        assert text in manual, f"documentation missing {text}"
    for command in [
        ["python3", "-B", "scripts/test-wgpu-host.py"],
        ["cargo", "+1.91.0", "test", "--locked", "-p", "reactive-tui", "--features", "wgpu-graphics", "--test", "wgpu_measurement"],
        ["cargo", "+1.91.0", "build", "--locked", "-p", "reactive-tui", "--features", "wgpu-graphics", "--example", "wgpu_benchmark", "--example", "widget_catalog"],
    ]:
        print("RUN", " ".join(command), flush=True)
        subprocess.run(command, cwd=ROOT, check=True, timeout=1200)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    destination = ROOT / "target/evidence/captures/wgpu" / f"{stamp}-{os.getpid()}"
    spec = importlib.util.spec_from_file_location("wgpu_host", ROOT / "scripts/check-wgpu-host.py")
    host = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(host)
    host.run(destination)
    for size in host.SIZES:
        for mode in ["GPU", "CPU"]:
            report = json.loads((destination / f"benchmark-{mode.lower()}-{size[0]}x{size[1]}.json").read_text())
            validate(report, mode, size)
        png = (destination / f"kitty-gpu-{size[0]}x{size[1]}.png").read_bytes()
        assert png[:8] == b"\x89PNG\r\n\x1a\n"
        width, height = struct.unpack(">II", png[16:24])
        assert width >= size[0] * 5 and height >= size[1] * 10, "not a host-sized capture"
    print("PASS GPU-005: six hardware/CPU reports, separate costs, quality differences, three resized Kitty captures", flush=True)
    print("REVIEW: inspect these host captures before claiming GPU-002 visual acceptance", flush=True)


if __name__ == "__main__":
    main()
