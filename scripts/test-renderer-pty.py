#!/usr/bin/env python3
"""Check probe predicates against partial escapes, sparse cells and stale frames."""
import importlib.util
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("renderer_probe", Path(__file__).with_name("check-renderer-pty.py"))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
END = probe.SYNC_END
INITIAL = b"\x1b[?2026h\x1b[1;1HCount: 0" + END


class FramePredicateTests(unittest.TestCase):
    def test_partial_cursor_digits_are_not_counter_updates(self):
        self.assertTrue(probe.counter_frame(INITIAL, 0))
        # The old byte predicate accepted the 1 in this unfinished cursor move.
        data = INITIAL + END + b"\x1b[?2026h\x1b[1"
        self.assertFalse(probe.counter_frame(data, 1, len(INITIAL)))

    def test_sparse_update_requires_its_own_completed_frame(self):
        update = INITIAL + b"\x1b[?2026h\x1b[1;8H1"
        self.assertFalse(probe.counter_frame(update, 1, len(INITIAL)))
        self.assertTrue(probe.counter_frame(update + END, 1, len(INITIAL)))

    def test_resize_requires_new_dimensions_and_current_screen(self):
        resized = INITIAL + b"\x1b[?2026h\x1b[12;34H " + END
        updated = resized + b"\x1b[?2026h\x1b[1;8H1" + END
        self.assertFalse(probe.counter_frame(resized, 1, len(INITIAL), (34, 12)))
        self.assertTrue(probe.counter_frame(updated, 1, len(INITIAL), (34, 12)))
        self.assertFalse(probe.counter_frame(updated, 1, len(INITIAL), (60, 18)))
        self.assertFalse(probe.counter_frame(updated, 1, len(updated), (34, 12)))

    def test_cleared_screen_cannot_reuse_historical_counter(self):
        old = b"\x1b[1;1HCount: 1" + END
        cleared = old + b"\x1b[?2026h\x1b[2J\x1b[12;34H " + END
        self.assertFalse(probe.counter_frame(cleared, 1, len(old), (34, 12)))


if __name__ == "__main__":
    unittest.main()
