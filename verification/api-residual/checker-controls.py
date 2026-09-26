#!/usr/bin/env python3
"""Safe positive and violating inputs for the residual acceptance checker."""
from pathlib import Path
import json
import runpy
import unittest

ROOT = Path(__file__).resolve().parents[2]
CHECK = runpy.run_path(str(ROOT / "scripts/check-api-residual.py"))


class CheckerControls(unittest.TestCase):
    def test_input_lifecycle_requires_every_case_and_cleanup(self):
        rows = [{"case": case, "exit": 0, "timeout": False, "reaped": True} for case in CHECK["INPUT_CASES"]]
        def verify(value):
            CHECK["require_input_cases"]("INPUT_LIFECYCLE " + json.dumps(value))
        verify(rows)
        for violating in (rows[:-1], rows + rows[:1],
                          [{**rows[0], "exit": 101}] + rows[1:],
                          [{**rows[0], "timeout": True}] + rows[1:],
                          [{**rows[0], "reaped": False}] + rows[1:]):
            with self.assertRaises(AssertionError):
                verify(violating)
        with self.assertRaises(AssertionError):
            CHECK["require_input_cases"]("No actual lifecycle execution")

    def test_discovery_requires_exact_registered_names(self):
        name = "backend::tests::map_paste_event"
        CHECK["require_registered"](name + ": test\n", [name])
        for violating in ("0 tests, 0 benchmarks", "some_other_test: test", name):
            with self.assertRaises(AssertionError):
                CHECK["require_registered"](violating, [name])

    def test_execution_rejects_zero_ignored_and_failed_tests(self):
        name = "map_paste_event"
        passing = f"test {name} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;"
        CHECK["require_executed"](passing, [name])
        for violating in ("test result: ok. 0 passed; 0 failed;", passing.replace("... ok", "... ignored"),
                          passing + "\ntest result: FAILED. 1 passed; 1 failed;"):
            with self.assertRaises(AssertionError):
                CHECK["require_executed"](violating, [name])

    def test_inventory_cannot_substitute_for_focused_checks(self):
        concerns = CHECK["CONCERNS"]
        CHECK["require_coverage"](concerns)
        for violating in ((), concerns[1:]):
            with self.assertRaises(AssertionError):
                CHECK["require_coverage"](violating)

    def test_consumer_requires_completion_marker(self):
        CHECK["require_marker"]("PERFORMANCE_OWNER_OK\n", "PERFORMANCE_OWNER_OK")
        for text in ("", "not PERFORMANCE_OWNER_OK", "PERFORMANCE_OWNER_OK missing"):
            with self.assertRaises(AssertionError):
                CHECK["require_marker"](text, "PERFORMANCE_OWNER_OK")

    def test_failed_command_propagates_through_its_group(self):
        check = CHECK["Check"].__new__(CHECK["Check"])
        check.steps = []
        check.output = ROOT / "target/api-residual-checker-controls"

        def reject(*_args):
            raise RuntimeError("controlled subprocess failure")

        check.execute = reject
        check.inspect("refs")
        self.assertTrue(check.steps)
        self.assertTrue(all(step["result"] == "fail" for step in check.steps))


if __name__ == "__main__":
    unittest.main()
