#!/usr/bin/env python3
"""Require the widget matrix and real App input/frame acceptance workflows."""
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
result = subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_widget_behavior", "--", "--test-threads=1"],
    cwd=root, timeout=180,
)
matrix = (root / "docs/widget-acceptance.md").read_text()
pending = [line for line in matrix.splitlines() if line.startswith("|") and "PENDING" in line]
for line in pending:
    print("Missing App acceptance coverage: " + line, flush=True)
if result.returncode or pending:
    raise SystemExit(1)
