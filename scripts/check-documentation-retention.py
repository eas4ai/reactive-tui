#!/usr/bin/env python3
"""Verify that only Cairn-managed documentation remains tracked."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ALLOWED_DOC_PREFIXES = (
    "docs/spec/",
    "docs/commitments/",
    "docs/decisions/",
)
SPEC, COMMITMENTS, DECISIONS = ALLOWED_DOC_PREFIXES
RETENTION_PROBE = "__retention_probe__.md"
REQUIRED_PATHS = (
    ".cairn/mechanisms/documentation-retention",
    "docs/spec/overview.md",
    "docs/spec/glossary.md",
    "docs/spec/roadmap.md",
    SPEC + "documentation-retention.md",
    COMMITMENTS + "documentation-retention.md",
    DECISIONS + "retain-only-cairn-managed-documentation.md",
)


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def main() -> int:
    errors: list[str] = []
    tracked_result = git("ls-files", "docs")
    if tracked_result.returncode != 0:
        print(tracked_result.stderr, file=sys.stderr)
        return 1

    tracked = set(tracked_result.stdout.splitlines())
    legacy = sorted(
        path
        for path in tracked
        if path.startswith("docs/")
        and not path.startswith(ALLOWED_DOC_PREFIXES)
    )
    if legacy:
        errors.append("tracked documentation exists outside Cairn directories:")
        errors.extend(f"  {path}" for path in legacy)

    for required in REQUIRED_PATHS:
        if required.startswith(".cairn/"):
            result = git("ls-files", "--error-unmatch", required)
            is_tracked = result.returncode == 0
        else:
            is_tracked = required in tracked
        if not is_tracked:
            errors.append(f"required Cairn artifact is not tracked: {required}")

    legacy_probe = git("check-ignore", "--no-index", f"docs/{RETENTION_PROBE}")
    if legacy_probe.returncode != 0:
        errors.append(f"legacy docs path is not ignored: docs/{RETENTION_PROBE}")

    for directory in ("spec", "commitments", "decisions"):
        probe = f"docs/{directory}/{RETENTION_PROBE}"
        result = git("check-ignore", "--no-index", probe)
        if result.returncode == 0:
            errors.append(f"Cairn docs path is ignored: {probe}")
        elif result.returncode != 1:
            errors.append(f"could not evaluate ignore boundary for {probe}: {result.stderr.strip()}")

    if errors:
        print("documentation retention check failed", file=sys.stderr)
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(
        "documentation retention check passed: "
        f"{sum(path.startswith('docs/') for path in tracked)} Cairn documents tracked"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
