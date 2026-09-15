#!/usr/bin/env python3
import sys


def main() -> int:
    if sys.argv[1:] != ["DQC-004"]:
        print("usage: check-pre-release-test-integrity.py DQC-004", file=sys.stderr)
        return 2
    print("DQC-004 failed: test-integrity mechanism is not implemented", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
