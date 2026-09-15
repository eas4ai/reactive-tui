#!/usr/bin/env python3
"""Run focused checks for pre-release runtime-resilience requirements."""
import os
from pathlib import Path
import subprocess
import sys


root = Path(__file__).resolve().parents[1]
requirement = sys.argv[1] if len(sys.argv) == 2 else ""
if requirement != "RTR-001":
    raise SystemExit("usage: check-pre-release-runtime-resilience.py RTR-001")

environment = os.environ.copy()
environment["CARGO_BUILD_JOBS"] = "8"
environment["CARGO_INCREMENTAL"] = "0"
environment["PYTHONDONTWRITEBYTECODE"] = "1"
target_directory = Path(environment.get("CARGO_TARGET_DIR", root / "target"))

subprocess.run(
    ["cargo", "build", "--locked", "--jobs", "8", "--example", "accessibility_probe"],
    cwd=root,
    env=environment,
    check=True,
    timeout=300,
)
subprocess.run(
    [
        "python3",
        "-B",
        "tests/api_widget_behavior/transport_failures.py",
        "--binary",
        str(target_directory / "debug/examples/accessibility_probe"),
    ],
    cwd=root,
    env=environment,
    check=True,
    timeout=40,
)
