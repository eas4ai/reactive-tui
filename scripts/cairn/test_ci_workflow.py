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


def workflow(toolchains=("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc"), extra_runs=(), extra_steps=(),
             pin='"+$TOOLCHAIN"'):
    """A workflow in memory in the shape the static checks read: a platform
    job whose matrix pins a toolchain per runner and whose steps run the five
    commands through that pin."""
    include = [{"os": os, "toolchain": tc} for os, tc in zip(("ubuntu-24.04", "macos-14", "windows-2022"), toolchains)]
    runs = ["rustup toolchain install \"$TOOLCHAIN\" --profile minimal --component rustfmt --component clippy",
            f"cargo {pin} install cargo-deny --version 0.20.2 --locked",
            f"cargo {pin} build --locked --all-targets",
            f"cargo {pin} test --locked --no-fail-fast",
            f"cargo {pin} fmt --all -- --check",
            f"cargo {pin} clippy --locked --all-targets -- -D warnings",
            f"cargo {pin} deny --locked check advisories bans licenses sources", *extra_runs]
    steps = [{"run": run} for run in runs]
    steps.append({"if": "runner.os == 'Windows'", "run": "python -B scripts/install-conpty-runtime.py target/debug/deps --arch x64"})
    steps.extend(extra_steps)
    return {"jobs": {"platform": {"strategy": {"matrix": {"include": include}}, "steps": steps}}}


ATTRIBUTES = "* text=auto eol=lf\n"


class StaticChecks(unittest.TestCase):
    def violations(self, wf, version="1.95"):
        return ci.static_violations(wf, ATTRIBUTES, version)

    def test_a_workflow_pinned_per_job_at_the_floor_passes(self):
        self.assertEqual(self.violations(workflow()), [])

    def test_an_unpinned_matrix_fails_per_job(self):
        wf = {"jobs": {"platform": {"strategy": {"matrix": {"os": ["ubuntu-24.04", "macos-14", "windows-2022"]}},
                                    "steps": workflow()["jobs"]["platform"]["steps"]}}}
        found = self.violations(wf)
        self.assertEqual(sum("not pinned in full" in v for v in found), 3, found)

    def test_a_pin_without_a_patch_number_is_not_in_full(self):
        found = self.violations(workflow(toolchains=("1.95", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertTrue(any("ubuntu-24.04 job's toolchain is not pinned in full" in v for v in found), found)

    def test_a_pin_below_the_floor_fails(self):
        found = self.violations(workflow(toolchains=("1.91.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertTrue(any("below Cargo.toml's rust-version 1.95" in v for v in found), found)

    def test_a_gnu_windows_toolchain_fails(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-gnu")))
        self.assertTrue(any("names a GNU toolchain" in v for v in found), found)
        self.assertTrue(any("does not name the MSVC toolchain" in v for v in found), found)

    def test_a_windows_pin_without_the_triple_is_not_msvc_in_full(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0")))
        self.assertTrue(any("does not name the MSVC toolchain in full" in v for v in found), found)

    def test_rustup_default_fails(self):
        found = self.violations(workflow(extra_runs=("rustup default 1.95.0",)))
        self.assertTrue(any("`rustup default`" in v for v in found), found)

    def test_rustup_set_default_host_fails(self):
        found = self.violations(workflow(extra_runs=("rustup set default-host x86_64-pc-windows-msvc",)))
        self.assertTrue(any("`rustup set default-host`" in v for v in found), found)

    def test_a_default_change_in_another_job_fails_too(self):
        wf = workflow()
        wf["jobs"]["advisories"] = {"steps": [{"run": "rustup toolchain install 1.95.0\nrustup default 1.95.0"}]}
        found = self.violations(wf)
        self.assertTrue(any("`rustup default`" in v for v in found), found)

    def test_a_bare_cargo_call_fails(self):
        found = self.violations(workflow(extra_runs=("cargo doc --no-deps",)))
        self.assertTrue(any("names no toolchain: `cargo doc`" in v for v in found), found)

    def test_a_literal_pin_counts(self):
        self.assertEqual(self.violations(workflow(pin="+1.95.0")), [])

    def test_a_linux_pin_above_the_floor_needs_a_floor_step(self):
        above = ("1.99.0", "1.99.0", "1.99.0-x86_64-pc-windows-msvc")
        found = self.violations(workflow(toolchains=above))
        self.assertTrue(any("does not build and test at rust-version 1.95" in v for v in found), found)
        floor = {"if": "runner.os == 'Linux'",
                 "run": "rustup toolchain install 1.95.0 --profile minimal\n"
                        "cargo +1.95.0 build --locked --all-targets\ncargo +1.95.0 test --locked --no-fail-fast"}
        self.assertEqual(self.violations(workflow(toolchains=above, extra_steps=(floor,))), [])

    def test_a_missing_command_fails(self):
        wf = workflow()
        wf["jobs"]["platform"]["steps"] = [s for s in wf["jobs"]["platform"]["steps"] if "fmt" not in str(s.get("run"))]
        found = self.violations(wf)
        self.assertTrue(any("does not run `cargo fmt --all -- --check`" in v for v in found), found)


if __name__ == "__main__":
    unittest.main()
