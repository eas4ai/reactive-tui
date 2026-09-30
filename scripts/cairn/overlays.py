#!/usr/bin/env python3
"""overlays: OVL-001 to OVL-004 through the tests/overlays_contract.rs
binary and the library's `ovl_004_` unit tests: the colors of the modal,
the popover, the toast and the five dialogs by role under a theme whose
roles all differ, their size in cells and the half-viewport limit, where
each is placed and what it is painted over, and what each tells the screen
reader and where the focus goes.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def both(*runs: tuple[bool, str]) -> tuple[bool, str]:
    """One result from several test runs: pass when every run passed."""
    return all(ok for ok, _ in runs), "; ".join(why for _, why in runs)


def main() -> int:
    results = {f"OVL-00{n}": cargo_test_filtered("overlays_contract", f"ovl_00{n}_") for n in (1, 2, 3)}
    results["OVL-004"] = both(cargo_test_filtered(None, "ovl_004_"),
                              cargo_test_filtered("overlays_contract", "ovl_004_"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
