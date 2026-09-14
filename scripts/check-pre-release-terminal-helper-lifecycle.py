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
    command = [
        "cargo",
        "test",
        "--locked",
        "--test",
        "pre_release_terminal_helper_lifecycle",
        "--jobs",
        JOBS,
    ]
    subprocess.run(
        command
        + [
            "validator_rejects_safe_violating_fixtures",
            "--",
            "--exact",
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    subprocess.run(
        command
        + [
            "--",
            "--test-threads=1",
            "--skip",
            "validator_rejects_safe_violating_fixtures",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    print("TRL-004 terminal helper lifecycle safety passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
