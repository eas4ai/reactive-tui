#!/usr/bin/env python3
"""theme: THM-001 to THM-003 through the tests/theme_contract.rs binary:
every built-in preset defines every color role and its text contrasts with
what it is drawn on, a theme of three colors still resolves every role, and
the frame after a change of theme holds no color of the old theme.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def main() -> int:
    return finish({f"THM-00{n}": cargo_test_filtered("theme_contract", f"thm_00{n}_") for n in (1, 2, 3)})


if __name__ == "__main__":
    sys.exit(main())
