#!/usr/bin/env python3
"""Bounded real App/SuprTUI component lifecycle acceptance."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_component_expansion", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)
