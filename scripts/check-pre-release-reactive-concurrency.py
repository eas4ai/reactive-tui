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
RAC_003_TESTS = 2


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


def validate_fallback_timer(
    *, idle_wakeups: int, callback_fired: bool, worker_stopped: bool
) -> list[str]:
    errors: list[str] = []
    if idle_wakeups:
        errors.append("fallback worker woke without work or shutdown")
    if not callback_fired:
        errors.append("scheduled fallback timer did not fire")
    if not worker_stopped:
        errors.append("fallback worker remained alive without an owner")
    return errors


def prove_fallback_validator() -> None:
    valid = {"idle_wakeups": 0, "callback_fired": True, "worker_stopped": True}
    assert not validate_fallback_timer(**valid)
    for changed in (
        {"idle_wakeups": 1},
        {"callback_fired": False},
        {"worker_stopped": False},
    ):
        assert validate_fallback_timer(**(valid | changed))


def reject_permanent_fallback_polling() -> None:
    source = (ROOT / "src/hooks/timer.rs").read_text(encoding="utf-8")
    compact = "".join(source.split())
    forbidden = "std::thread::sleep(Duration::from_millis(1));"
    if forbidden in compact:
        raise AssertionError("fallback timer still uses a permanent one-millisecond poll loop")


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


def run_integration_test(test_target: str, test_filter: str) -> None:
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--test",
            test_target,
            test_filter,
            "--jobs",
            JOBS,
            "--",
            "--exact",
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )


def run_doc_test(test_filter: str) -> None:
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--doc",
            test_filter,
            "--jobs",
            JOBS,
        ],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )


def require_tests(test_filter: str, expected: int) -> None:
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--lib",
            test_filter,
            "--jobs",
            JOBS,
            "--",
            "--list",
        ],
        cwd=ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
        timeout=600,
    )
    discovered = sum(line.rstrip().endswith(": test") for line in result.stdout.splitlines())
    if discovered != expected:
        raise AssertionError(
            f"expected {expected} {test_filter} tests, discovered {discovered}"
        )


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {"RAC-001", "RAC-002", "RAC-003"}:
        print(f"usage: {Path(sys.argv[0]).name} RAC-001|RAC-002|RAC-003", file=sys.stderr)
        return 2
    if sys.argv[1] == "RAC-001":
        run_test(VALIDATOR, "--exact")
        run_test("hooks::animation::tests::rac_001_", "--skip", VALIDATOR)
        print(
            "RAC-001 retained owners, scheduler reuse, bounded threads, and unmount cancellation passed"
        )
    elif sys.argv[1] == "RAC-002":
        prove_reentrancy_validator()
        run_test("rac_002_")
        run_integration_test(
            "api_residual_refs", "reference_hooks_preserve_existing_type_bounds"
        )
        run_doc_test("hooks::refs::Ref")
        print("RAC-002 callbacks completed re-entry and later progress without held guards")
    else:
        prove_fallback_validator()
        reject_permanent_fallback_polling()
        require_tests("rac_003_", RAC_003_TESTS)
        run_test("rac_003_")
        print("RAC-003 fallback worker slept when idle, ran work, and stopped with its owner")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
