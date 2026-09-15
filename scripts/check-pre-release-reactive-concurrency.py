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


def validate_reentrancy(
    *, panicked: bool, timed_out: bool, poisoned: bool, later_progress: bool
) -> list[str]:
    errors: list[str] = []
    if panicked:
        errors.append("re-entrant callback panicked")
    if timed_out:
        errors.append("re-entrant callback exceeded its deadline")
    if poisoned:
        errors.append("re-entrant callback poisoned internal state")
    if not later_progress:
        errors.append("ordinary work did not progress after re-entry")
    return errors


def prove_reentrancy_validator() -> None:
    valid = {
        "panicked": False,
        "timed_out": False,
        "poisoned": False,
        "later_progress": True,
    }
    assert not validate_reentrancy(**valid)
    for changed in (
        {"panicked": True},
        {"timed_out": True},
        {"poisoned": True},
        {"later_progress": False},
    ):
        observation = valid | changed
        assert validate_reentrancy(**observation)


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
    if len(sys.argv) != 2 or sys.argv[1] not in {"RAC-001", "RAC-002"}:
        print(f"usage: {Path(sys.argv[0]).name} RAC-001|RAC-002", file=sys.stderr)
        return 2
    if sys.argv[1] == "RAC-001":
        run_test(VALIDATOR, "--exact")
        run_test("hooks::animation::tests::rac_001_", "--skip", VALIDATOR)
        print(
            "RAC-001 retained owners, scheduler reuse, bounded threads, and unmount cancellation passed"
        )
    else:
        prove_reentrancy_validator()
        run_test("rac_002_")
        print("RAC-002 callbacks completed re-entry and later progress without held guards")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
