#!/usr/bin/env python3
import subprocess
import sys


REQUIRED_PATHS = ("parser", "terminal", "reconciliation", "focus", "window")


def validate_observation(observation: dict) -> list[str]:
    errors = []
    if observation.get("source_audit") is not True:
        errors.append("source audit failed")
    if not observation.get("audited_files"):
        errors.append("source audit reported no library files")
    for finding in observation.get("raw_macros", []):
        errors.append(
            f"{finding['path']}:{finding['line']} reaches {finding['macro']}!"
        )
    if observation.get("capture") is not True:
        errors.append("captured-output probe failed")
    for stream in ("stdout", "stderr"):
        if observation.get(stream):
            errors.append(f"captured internal {stream} output")
    paths = observation.get("exercised_paths", {})
    for path in REQUIRED_PATHS:
        if paths.get(path) is not True:
            errors.append(f"diagnostic probe skipped {path}")
    return errors


def capture_command() -> list[str]:
    return [
        "cargo",
        "+1.91.0",
        "test",
        "--locked",
        "--quiet",
        "--test",
        "dqc_003_captured_diagnostics",
    ]


def check_dqc_003() -> int:
    unit = subprocess.run(
        [sys.executable, "-B", "scripts/test-pre-release-library-diagnostics.py"],
        check=False,
    )
    if unit.returncode != 0:
        return 1
    print("DQC-003 failed: source audit and captured-output probe are not implemented", file=sys.stderr)
    return 1


def main() -> int:
    if sys.argv[1:] != ["DQC-003"]:
        print("usage: check-pre-release-library-diagnostics.py DQC-003", file=sys.stderr)
        return 2
    return check_dqc_003()


if __name__ == "__main__":
    raise SystemExit(main())
