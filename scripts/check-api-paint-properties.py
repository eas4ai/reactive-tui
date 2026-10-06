#!/usr/bin/env python3
"""Check gradient cells and scheduled animation frames through App/SuprTUI."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--lib", "app::motion::tests", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_paint_properties", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
