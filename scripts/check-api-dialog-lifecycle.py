#!/usr/bin/env python3
"""Run the dedicated dialog lifecycle acceptance target."""

import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parent.parent
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    command = ["cargo", "test", "--locked", "--test", "api_dialog_lifecycle"]
    print("+ " + " ".join(command), flush=True)
    try:
        return subprocess.run(command, cwd=root, env=env, timeout=600).returncode
    except subprocess.TimeoutExpired:
        print("Dialog lifecycle acceptance exceeded 600 seconds", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
