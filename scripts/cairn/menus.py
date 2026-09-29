#!/usr/bin/env python3
"""menus: MNU-001 to MNU-004 through the tests/menus_contract.rs binary:
the colors of the menu bar, the context menu, the popup menu and the dialog
menu by role under a theme whose roles all differ, a panel as wide and as
tall as its rows up to the viewport, a panel opened beside what opened it,
and a panel painted whole inside a modal, a popover and a box that clips.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def main() -> int:
    return finish({f"MNU-00{n}": cargo_test_filtered("menus_contract", f"mnu_00{n}_") for n in (1, 2, 3, 4)})


if __name__ == "__main__":
    sys.exit(main())
