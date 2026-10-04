#!/usr/bin/env python3
"""The ci-workflow mechanism's own tests: which runs and jobs may satisfy
BAR-012 (no scheduled run, every platform job successful), without gh."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ci_workflow as ci  # noqa: E402


def run(**fields):
    base = {"databaseId": 1, "status": "completed", "conclusion": "success", "event": "push",
            "createdAt": "2026-10-04T00:00:00Z", "url": "https://example.invalid/run/1"}
    return {**base, **fields}


def job(name, conclusion="success"):
    return {"name": name, "conclusion": conclusion, "status": "completed"}


PLATFORM = [job("platform (ubuntu-24.04)"), job("platform (macos-14)"), job("platform (windows-2022)")]
ADVISORIES_SKIPPED = job("scheduled security advisories", "skipped")


class RunSelection(unittest.TestCase):
    def test_a_scheduled_run_cannot_stand_for_the_platform_matrix(self):
        runs = [run(databaseId=7, event="schedule"), run(databaseId=8, event="push", conclusion="failure")]
        self.assertEqual([r["databaseId"] for r in ci.platform_runs(runs)], [8])

    def test_cancelled_runs_are_not_runs(self):
        runs = [run(databaseId=9, conclusion="cancelled"), run(databaseId=10, event="workflow_dispatch")]
        self.assertEqual([r["databaseId"] for r in ci.platform_runs(runs)], [10])

    def test_only_a_scheduled_run_means_no_run(self):
        self.assertEqual(ci.platform_runs([run(event="schedule")]), [])


class JobVerdict(unittest.TestCase):
    def test_three_successful_platform_jobs_pass(self):
        ok, why = ci.verdict(run(), PLATFORM + [ADVISORIES_SKIPPED])
        self.assertTrue(ok, why)
        self.assertIn("green on every job", why)

    def test_a_skipped_platform_matrix_fails_even_when_the_run_succeeded(self):
        jobs = [job(j["name"], "skipped") for j in PLATFORM] + [job("scheduled security advisories")]
        ok, why = ci.verdict(run(event="schedule"), jobs)
        self.assertFalse(ok)
        for prefix in ("platform (ubuntu-", "platform (macos-", "platform (windows-"):
            self.assertIn(prefix, why)

    def test_a_missing_platform_job_fails(self):
        ok, why = ci.verdict(run(), PLATFORM[:2] + [ADVISORIES_SKIPPED])
        self.assertFalse(ok)
        self.assertIn("platform (windows-", why)
        self.assertNotIn("platform (ubuntu-", why)

    def test_a_failed_platform_job_fails_with_its_name(self):
        jobs = PLATFORM[:2] + [job("platform (windows-2022)", "failure")]
        ok, why = ci.verdict(run(conclusion="failure"), jobs)
        self.assertFalse(ok)
        self.assertIn("platform (windows-2022) failure", why)

    def test_an_unfinished_run_fails_on_the_ceiling(self):
        ok, why = ci.verdict(run(status="in_progress", conclusion=""), [])
        self.assertFalse(ok)
        self.assertIn("did not finish", why)


if __name__ == "__main__":
    unittest.main()
