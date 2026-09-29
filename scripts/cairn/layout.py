#!/usr/bin/env python3
"""layout: LAY-001 to LAY-004 through the tests/layout_contract.rs binary:
gaps, equal tracks and spans in whole cells on the debug and the terminal
backend at every width from 40 to 512, the spacing classes' unit for every
whole number from 0 to 512, the gap and grid classes, and layouts that
settle (the data table with its panels open, and three pages of the widget
catalog).

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def main() -> int:
    return finish({f"LAY-00{n}": cargo_test_filtered("layout_contract", f"lay_00{n}_") for n in (1, 2, 3, 4)})


if __name__ == "__main__":
    sys.exit(main())
