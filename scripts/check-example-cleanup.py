#!/usr/bin/env python3
"""Validate the supported repository example inventory."""

from __future__ import annotations

from pathlib import Path
import re
import subprocess
import sys


EXPECTED_EXAMPLES = {"gradient_blocks.rs"}
REMOVED_EXAMPLES = (
    "animated_patterns",
    "dialog_engine",
    "embedded_shell",
    "suprtui_counter",
    "visual_effects",
    "wake_counter",
)
REFERENCE_EXCLUSIONS = {
    Path("scripts/check-example-cleanup.py"),
    Path("scripts/test-example-cleanup.py"),
}
REFERENCE_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_])(" + "|".join(map(re.escape, REMOVED_EXAMPLES)) + r")(?:\.rs)?(?![A-Za-z0-9_])"
)


def tracked_files(root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=True,
        stdout=subprocess.PIPE,
    )
    return [root / path.decode() for path in result.stdout.split(b"\0") if path]


def stale_references(root: Path, paths: list[Path]) -> list[str]:
    findings = []
    for path in paths:
        relative = path.relative_to(root)
        if relative in REFERENCE_EXCLUSIONS or relative.parts[:2] in {
            (".cairn", "backlog"),
            (".cairn", "evidence"),
            (".cairn", "reviews"),
        }:
            continue
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeDecodeError):
            continue
        for line_number, line in enumerate(lines, 1):
            for match in REFERENCE_PATTERN.finditer(line):
                findings.append(
                    f"stale removed-example reference: {relative}:{line_number}: {match.group(1)}"
                )
    return findings


def validate(root: Path, *, tracked_paths=None, compile_runner=subprocess.run) -> list[str]:
    errors = []
    examples = {path.name for path in (root / "examples").glob("*.rs")}
    missing = sorted(EXPECTED_EXAMPLES - examples)
    unexpected = sorted(examples - EXPECTED_EXAMPLES)
    if missing:
        errors.append("missing public examples: " + ", ".join(missing))
    if unexpected:
        errors.append("unexpected public examples: " + ", ".join(unexpected))

    paths = list(tracked_paths) if tracked_paths is not None else tracked_files(root)
    errors.extend(stale_references(root, paths))

    compile_result = compile_runner(
        ["cargo", "+1.91.0", "check", "--locked", "--example", "gradient_blocks"],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if compile_result.returncode:
        errors.append("gradient_blocks failed to compile")
        if compile_result.stdout:
            print(compile_result.stdout, end="")
    return errors


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tests = subprocess.run(
        [sys.executable, "-B", "scripts/test-example-cleanup.py"],
        cwd=root,
        check=False,
    )
    if tests.returncode:
        return tests.returncode
    errors = validate(root)
    if errors:
        for error in errors:
            print(f"FAIL EXC-001: {error}", file=sys.stderr)
        return 1
    print("PASS EXC-001: gradient_blocks is the only public Rust example and compiles")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
