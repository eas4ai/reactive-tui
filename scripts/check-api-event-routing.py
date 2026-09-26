#!/usr/bin/env python3
"""Verify retained builder callbacks through the application input loop."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_event_routing", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
