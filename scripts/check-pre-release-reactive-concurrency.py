#!/usr/bin/env python3
"""Run focused pre-release reactive concurrency checks."""

from pathlib import Path
import sys


def main() -> int:
    if sys.argv[1:] != ["RAC-001"]:
        print(f"usage: {Path(sys.argv[0]).name} RAC-001", file=sys.stderr)
        return 2
    print("RAC-001 animation ownership mechanism is not implemented", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
