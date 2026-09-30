#!/usr/bin/env python3
"""layout-widgets: NAV-001 to NAV-004 through the tests/layout_widgets_contract.rs
binary and the library's `nav_004_` unit tests: the colors of the tabs,
the accordion, the breadcrumb, the scroll view and the stack by role under
a theme whose roles all differ, from their props and their builders alike,
with focus, hover and disabled shown by color alone; the size each fills,
the padding of a tab's label and a segment, and a bar column only while
the content overflows; a tab bar that scrolls to its focused tab, a trail
that keeps its ends whole, and a scroll view that follows the wheel, its
track and its thumb; and what each widget tells the screen reader and the
key behind every pointer action.

Prints one `cairn: <REQ>: pass|fail` line per requirement.
"""

import sys

from _common import cargo_test_filtered, finish


def both(*runs: tuple[bool, str]) -> tuple[bool, str]:
    """One result from several test runs: pass when every run passed."""
    return all(ok for ok, _ in runs), "; ".join(why for _, why in runs)


def main() -> int:
    results = {f"NAV-00{n}": cargo_test_filtered("layout_widgets_contract", f"nav_00{n}_") for n in (1, 2, 3)}
    # NAV-004 also has unit tests beside the widgets: what a widget tells
    # the screen reader is read from its node, which only the crate sees.
    results["NAV-004"] = both(cargo_test_filtered(None, "nav_004_"),
                              cargo_test_filtered("layout_widgets_contract", "nav_004_"))
    return finish(results)


if __name__ == "__main__":
    sys.exit(main())
