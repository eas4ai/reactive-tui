#!/usr/bin/env python3
"""BAR-001: the five workspace gates pass on Linux. Exit code is the result."""

import sys

from _common import JOBS, run

GATES = [
    ["cargo", "build", "--locked", "--workspace", "--jobs", JOBS],
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--locked", "--workspace", "--all-targets", "--jobs", JOBS, "--", "-D", "warnings"],
    ["cargo", "doc", "--locked", "--workspace", "--no-deps", "--jobs", JOBS],
    ["cargo", "test", "--locked", "--workspace", "--no-fail-fast", "--jobs", JOBS],
]


def main() -> int:
    failed = []
    for cmd in GATES:
        r = run(cmd, timeout=3600)
        if r.returncode != 0:
            failed.append(" ".join(cmd[:3]))
            print("\n".join((r.stdout + r.stderr).splitlines()[-40:]))
    if failed:
        print(f"BAR-001 violated: gates failing: {', '.join(failed)}")
        return 1
    print("BAR-001 holds: all five gates pass")
    return 0


if __name__ == "__main__":
    sys.exit(main())
