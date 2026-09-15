#!/usr/bin/env python3
import sys


def main() -> int:
    if sys.argv[1:] != ["DQC-005"]:
        print("usage: check-pre-release-public-api.py DQC-005", file=sys.stderr)
        return 2
    print("DQC-005 failed: public-API mechanism is not implemented", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
