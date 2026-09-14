#!/usr/bin/env python3
"""Run the focused pre-release terminal helper lifecycle check."""

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
JOBS = "8"


def main() -> int:
    if sys.argv[1:] != ["TRL-004"]:
        print(f"usage: {Path(sys.argv[0]).name} TRL-004", file=sys.stderr)
        return 2
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--test",
            "pre_release_terminal_helper_lifecycle",
            "--jobs",
            JOBS,
            "--",
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
