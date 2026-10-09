#!/usr/bin/env python3
"""Bounded state/memo behavior checks for generated and manual hook contexts."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_hook_state"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)

subprocess.run(
    ["cargo", "test", "--locked", "--lib", "reactive::hooks::tests"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
