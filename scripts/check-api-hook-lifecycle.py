#!/usr/bin/env python3
"""Exercise component-owned hook resources through App rendering."""
from pathlib import Path
import subprocess

subprocess.run(
    ["cargo", "test", "--locked", "--test", "api_hook_lifecycle", "--", "--test-threads=1"],
    cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
)

for target in ["reactive::hooks::tests", "reactive::component_scope::tests", "hooks::timer::tests"]:
    subprocess.run(
        ["cargo", "test", "--locked", "--lib", target, "--", "--test-threads=1"],
        cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
    )
