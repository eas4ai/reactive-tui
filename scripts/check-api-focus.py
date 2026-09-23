#!/usr/bin/env python3
"""Verify stable App focus and nested traps with real rendered input flows."""
from pathlib import Path
import subprocess

for arguments in [["--test", "api_focus"], ["--lib", "event::focus::tests"]]:
    subprocess.run(
        ["cargo", "test", "--locked", *arguments, "--", "--test-threads=1"],
        cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
    )
