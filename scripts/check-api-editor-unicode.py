#!/usr/bin/env python3
"""Exercise both editors with Unicode edits and independent display expectations."""
from pathlib import Path
import subprocess

for arguments in [["--test", "api_editor_unicode"], ["--lib", "editor::"]]:
    subprocess.run(
        ["cargo", "test", "--locked", *arguments, "--", "--test-threads=1"],
        cwd=Path(__file__).resolve().parents[1], check=True, timeout=120,
    )
