#!/usr/bin/env python3
"""Run focused pre-release reactive concurrency checks."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
JOBS = "8"
VALIDATOR = "hooks::animation::tests::rac_001_validator_rejects_each_controlled_violation"


def run_test(test_filter: str, *test_args: str) -> None:
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            test_filter,
            "--jobs",
            JOBS,
            "--",
            *test_args,
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )


def main() -> int:
    if sys.argv[1:] != ["RAC-001"]:
        print(f"usage: {Path(sys.argv[0]).name} RAC-001", file=sys.stderr)
        return 2
    run_test(VALIDATOR, "--exact")
    run_test("hooks::animation::tests::rac_001_", "--skip", VALIDATOR)
    print(
        "RAC-001 retained owners, scheduler reuse, bounded threads, and unmount cancellation passed"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
