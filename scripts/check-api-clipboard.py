#!/usr/bin/env python3
"""Check isolated clipboard subprocess behavior before platform integration."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_clipboard", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
subprocess.run(
    ["cargo", "test", "--locked", "--lib", "hooks::clipboard_process::tests", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
subprocess.run(
    ["python3", "-B", "scripts/check-clipboard-platforms.py", "--verify"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=30,
)
