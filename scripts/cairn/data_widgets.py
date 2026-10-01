#!/usr/bin/env python3
"""data-widgets: DAT-001 to DAT-004 through the tests/data_widgets_contract.rs
binary and the library's `dat_004_` unit tests: the colors of the table,
the data table, the tree, the file explorer and the progress bar by role
under a theme whose roles all differ, from their props and their builders
alike, with focus, hover and selection shown by color alone; the width and
height each fills, columns that share the width by weight, and no cap on
a data table's rows; numeric sorting, a revealed selection and a cursor
that stays near a collapse; and what each widget tells the screen reader
and the key behind every pointer action.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def both(*runs: tuple[bool, str]) -> tuple[bool, str]:
    """One result from several test runs: pass when every run passed."""
    return all(ok for ok, _ in runs), "; ".join(why for _, why in runs)


def main() -> int:
    results = {f"DAT-00{n}": cargo_test_filtered("data_widgets_contract", f"dat_00{n}_") for n in (1, 2, 3)}
    # DAT-004 also has unit tests beside the widgets: what a widget tells
    # the screen reader is read from its node, which only the crate sees.
    results["DAT-004"] = both(cargo_test_filtered(None, "dat_004_"),
                              cargo_test_filtered("data_widgets_contract", "dat_004_"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
