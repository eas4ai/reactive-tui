#!/usr/bin/env python3
"""display-pieces: DIS-001 to DIS-006 through the tests/display_pieces_contract.rs
binary and the library's `dis_004_` and `dis_005_` unit tests: the colors of
the icon catalog, spinner, separator, badge and tag, key hint, empty state,
skeleton and shimmer, status bar, description list, inline alert, link,
pagination bar and stepper by role under a theme whose roles all differ, from
their props and their builders alike, with focus, hover and disabled shown by
color alone; the width each fills or takes; their frames, stillness under
reduced motion, counts, page math, keys, the link's OSC 8 and the key hint's
binding; what each tells the screen reader; the one icon catalog every named
widget draws from; and the widgets that draw the shared pieces in place of
their own copies.
Prints one `cairn: <REQ>: pass|fail` line per requirement. A requirement no
test of its name has exercised does not pass.
"""
import sys

from _common import cargo_test_filtered, finish


def both(*runs: tuple[bool, str]) -> tuple[bool, str]:
    """One result from several test runs: pass when every run passed."""
    return all(ok for ok, _ in runs), "; ".join(why for _, why in runs)


def main() -> int:
    results = {f"DIS-00{n}": cargo_test_filtered("display_pieces_contract", f"dis_00{n}_") for n in (1, 2, 3, 6)}
    # DIS-004 and DIS-005 also have unit tests beside the widgets: what a
    # widget tells the screen reader, and the glyph an icon paints, are read
    # from its node and its catalog, which only the crate sees.
    for n in (4, 5):
        results[f"DIS-00{n}"] = both(cargo_test_filtered(None, f"dis_00{n}_"),
                                     cargo_test_filtered("display_pieces_contract", f"dis_00{n}_"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
