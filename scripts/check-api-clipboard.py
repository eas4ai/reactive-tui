#!/usr/bin/env python3
"""Check isolated clipboard subprocess behavior before platform integration."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_clipboard", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
