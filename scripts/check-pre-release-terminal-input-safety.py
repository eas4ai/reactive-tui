#!/usr/bin/env python3
"""Run the focused pre-release terminal input safety check."""

from pathlib import Path
import sys


def main() -> int:
    if sys.argv[1:] != ["TRL-003"]:
        print(f"usage: {Path(sys.argv[0]).name} TRL-003", file=sys.stderr)
        return 2
    print("TRL-003 mechanism has not been implemented", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
