#!/usr/bin/env python3
"""The verdict a test run on a test host gives (host_builds.tablet_test_verdict),
without a host: a run is read from cargo's exit status and the last test
summary, never from a summary a failing test's message quotes."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import host_builds as hb  # noqa: E402

ALL_PASS = ("running 5 tests\n"
            "test windows::inp_013_a_light_terminal_gets_the_light_preset ... ok\n"
            "test windows::smoke_inp_013_child ... ok\n"
            "\n"
            "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.95s\n"
            "\n")
# The parent's panic message quotes the child's whole output, whose own
# harness reported five passing tests (the terminal-side tests skip there).
EMBEDDED_CHILD_SUMMARY = ("running 5 tests\n"
                          "thread 'windows::inp_013_a_light_terminal_gets_the_light_preset' (8904) panicked at tests\\startup_windows.rs:209:9:\n"
                          "INP-013 (theme): the App never painted its first frame; output: \"running 5 tests\\r\\n"
                          "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s\\r\\n\"\n"
                          "test windows::inp_013_a_light_terminal_gets_the_light_preset ... FAILED\n"
                          "\n"
                          "failures:\n"
                          "    windows::inp_013_a_light_terminal_gets_the_light_preset\n"
                          "\n"
                          "test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.34s\n"
                          "\n"
                          "error: test failed, to rerun pass `-p reactive-tui --test startup_windows`\n")
COLOURED = ("running 2 tests\n"
            "test a ... \x1b[0;32mok\x1b[0m\n"
            "test b ... \x1b[0;32mok\x1b[0m\n"
            "\n"
            "test result: \x1b[0;32mok\x1b[0m. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n")


class Verdict(unittest.TestCase):
    def test_a_passing_run_passes(self):
        ok, why = hb.tablet_test_verdict(ALL_PASS, "0")
        self.assertTrue(ok, why)
        self.assertIn("5 passed", why)

    def test_a_failing_run_fails_whatever_a_message_quotes(self):
        ok, why = hb.tablet_test_verdict(EMBEDDED_CHILD_SUMMARY, "101")
        self.assertFalse(ok)
        self.assertIn("never painted its first frame", why)

    def test_the_last_summary_counts_even_with_exit_zero(self):
        # A harness that failed but somehow exited 0 still reads as the
        # failure its own last summary reports.
        ok, _ = hb.tablet_test_verdict(EMBEDDED_CHILD_SUMMARY, "0")
        self.assertFalse(ok)

    def test_a_non_zero_exit_fails_a_passing_looking_run(self):
        ok, why = hb.tablet_test_verdict(ALL_PASS, "101")
        self.assertFalse(ok)
        self.assertIn("exited 101", why)

    def test_too_few_tests_fail(self):
        ok, why = hb.tablet_test_verdict(ALL_PASS, "0", at_least=6)
        self.assertFalse(ok)
        self.assertIn("5", why)

    def test_colour_codes_do_not_hide_the_summary(self):
        ok, why = hb.tablet_test_verdict(COLOURED, "0")
        self.assertTrue(ok, why)

    def test_no_summary_is_no_verdict(self):
        with self.assertRaises(hb.Unreachable):
            hb.tablet_test_verdict("error: could not compile `reactive-tui`\nEXIT=101\n", "101")


if __name__ == "__main__":
    unittest.main()
