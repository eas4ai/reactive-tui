#!/usr/bin/env python3
"""input-widgets: CTL-001 to CTL-004 through the tests/input_widgets_contract.rs
binary and the library's `ctl_004_` unit tests: the colors of the text
input, the checkbox, the radio button, the select, the slider and the
button by role under a theme whose roles all differ, from their props and
their builders alike, with focus, hover and disabled shown by color alone;
the width a field, a select's row and a slider's track fill and the rows a
select's list shows; where the list opens and what it is painted over; and
what each control tells the screen reader and the key behind every pointer
action. CTL-007 to CTL-010 through the tests/input_pieces_contract.rs binary
and the library's `ctl_009_` and `ctl_010_` unit tests: the number input,
the code input, the rating, the switch and the form, their colors, widths,
keys, clicks, paste and callbacks, and what each tells the screen reader.
A requirement no test of its name has exercised does not pass.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def both(*runs: tuple[bool, str]) -> tuple[bool, str]:
    """One result from several test runs: pass when every run passed."""
    return all(ok for ok, _ in runs), "; ".join(why for _, why in runs)


def main() -> int:
    results = {f"CTL-00{n}": cargo_test_filtered("input_widgets_contract", f"ctl_00{n}_") for n in (1, 2, 3)}
    # CTL-004 also has unit tests beside the widgets: what a control tells
    # the screen reader is read from its node, which only the crate sees.
    results["CTL-004"] = both(cargo_test_filtered(None, "ctl_004_"),
                              cargo_test_filtered("input_widgets_contract", "ctl_004_"))
    # The input pieces: CTL-007 and CTL-008 through the contract binary;
    # CTL-009 and CTL-010 also have unit tests beside the widgets (the data
    # table builder's callback, and what a piece tells the screen reader).
    for n in (7, 8):
        results[f"CTL-00{n}"] = cargo_test_filtered("input_pieces_contract", f"ctl_00{n}_")
    for n in (9, 10):
        results[f"CTL-0{n:02d}"] = both(cargo_test_filtered(None, f"ctl_0{n:02d}_"),
                                        cargo_test_filtered("input_pieces_contract", f"ctl_0{n:02d}_"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
