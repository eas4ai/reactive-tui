#!/usr/bin/env python3
"""The ci-workflow mechanism's own tests: which runs and jobs may satisfy
BAR-012 (no scheduled run, every platform job successful), and what the
static reading of the workflow accepts and refuses, without gh."""
import copy
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


EXPECTED = ["platform (linux-x64)", "platform (macos-arm64)", "platform (windows-x64)"]
PLATFORM = [job(name) for name in EXPECTED]
HOSTED_SKIPPED = [job(f"platform-hosted ({os})", "skipped") for os in ("ubuntu-24.04", "macos-14", "windows-2022")]
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
        ok, why = ci.verdict(run(), PLATFORM + HOSTED_SKIPPED + [ADVISORIES_SKIPPED], EXPECTED)
        self.assertTrue(ok, why)
        self.assertIn("green on every job", why)

    def test_a_skipped_platform_matrix_fails_even_when_the_run_succeeded(self):
        jobs = [job(j["name"], "skipped") for j in PLATFORM] + [job("scheduled security advisories")]
        ok, why = ci.verdict(run(event="schedule"), jobs, EXPECTED)
        self.assertFalse(ok)
        for name in EXPECTED:
            self.assertIn(name, why)

    def test_a_missing_platform_job_fails(self):
        ok, why = ci.verdict(run(), PLATFORM[:2] + [ADVISORIES_SKIPPED], EXPECTED)
        self.assertFalse(ok)
        self.assertIn("platform (windows-x64)", why)
        self.assertNotIn("platform (linux-x64)", why)

    def test_a_failed_platform_job_fails_with_its_name(self):
        jobs = PLATFORM[:2] + [job("platform (windows-x64)", "failure")]
        ok, why = ci.verdict(run(conclusion="failure"), jobs, EXPECTED)
        self.assertFalse(ok)
        self.assertIn("platform (windows-x64) failure", why)

    def test_an_unfinished_run_fails_on_the_ceiling(self):
        ok, why = ci.verdict(run(status="in_progress", conclusion=""), [], EXPECTED)
        self.assertFalse(ok)
        self.assertIn("did not finish", why)

    def test_the_expected_names_come_from_the_push_jobs_name_template(self):
        self.assertEqual(ci.platform_job_names(workflow()), EXPECTED)


LINUX = ["self-hosted", "rust-ci", "Linux", "X64"]
MAC = ["self-hosted", "rust-ci", "macOS", "ARM64"]


def runner(name, labels, status="online", busy=False):
    return {"name": name, "status": status, "busy": busy, "labels": list(labels)}


def queued(name, labels, status="queued"):
    return {"name": name, "status": status, "labels": list(labels)}


RUNNERS = [runner("rust-linux-x64", LINUX), runner("rust-macos-arm64", MAC, status="offline")]


class OfflineWait(unittest.TestCase):
    def test_a_queued_job_whose_only_runner_is_offline_is_an_offline_wait(self):
        jobs = [queued("platform (linux-x64)", LINUX, status="in_progress"), queued("platform (macos-arm64)", MAC)]
        self.assertEqual(ci.offline_waits(jobs, RUNNERS), {"platform (macos-arm64)": "rust-macos-arm64"})

    def test_a_job_queued_on_an_online_runner_is_not_an_offline_wait(self):
        jobs = [queued("platform (linux-x64)", LINUX)]
        self.assertEqual(ci.offline_waits(jobs, RUNNERS), {})

    def test_a_hosted_or_unregistered_label_is_not_an_offline_wait(self):
        jobs = [queued("platform-hosted (ubuntu-24.04)", ["ubuntu-24.04"]),
                queued("platform (windows-x64)", ["self-hosted", "rust-ci", "Windows", "X64"])]
        self.assertEqual(ci.offline_waits(jobs, RUNNERS), {})

    def test_offline_time_is_announced_once_and_does_not_count_against_the_ceiling(self):
        waiter = ci.Waiter(ceiling=100, day=1000)
        jobs = [queued("platform (macos-arm64)", MAC)]
        first = waiter.step(jobs, RUNNERS, 60)
        self.assertEqual(len(first), 1)
        self.assertIn("waiting for the rust-macos-arm64 runner, which is offline, to run platform (macos-arm64)", first[0])
        self.assertEqual(waiter.step(jobs, RUNNERS, 60), [])
        self.assertEqual((waiter.executing, waiter.offline), (0.0, 120.0))
        self.assertFalse(waiter.over_ceiling())
        back = waiter.step([queued("platform (macos-arm64)", MAC, status="in_progress")], [], 60)
        self.assertEqual(back, ["the rust-macos-arm64 runner is back online; the run goes on"])
        self.assertEqual(waiter.executing, 60.0)
        self.assertEqual(waiter.step([], [], 50), [])
        self.assertTrue(waiter.over_ceiling())

    def test_an_offline_wait_ends_after_a_day(self):
        waiter = ci.Waiter(ceiling=100, day=200)
        jobs = [queued("platform (macos-arm64)", MAC)]
        waiter.step(jobs, RUNNERS, 150)
        self.assertFalse(waiter.over_day())
        waiter.step(jobs, RUNNERS, 100)
        self.assertTrue(waiter.over_day())
        self.assertFalse(waiter.over_ceiling())

    def test_an_unfinished_run_names_execution_time(self):
        ok, why = ci.verdict(run(status="in_progress", conclusion=""), [], EXPECTED)
        self.assertFalse(ok)
        self.assertIn("did not finish within 55 minutes of execution", why)


MATRIX_PIN = '"+$TOOLCHAIN"'
INSTALL = 'rustup toolchain install "$TOOLCHAIN" --profile minimal --component rustfmt --component clippy'
FLOOR_STEP = {"if": "runner.os == 'Linux'",
              "run": "rustup toolchain install 1.95.0 --profile minimal\n"
                     "cargo +1.95.0 build --locked --all-targets\ncargo +1.95.0 test --locked --no-fail-fast"}
PINS = ("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")
ABOVE = ("1.99.0", "1.99.0", "1.99.0-x86_64-pc-windows-msvc")
MACHINES = (("linux-x64", ["self-hosted", "rust-ci", "Linux", "X64"]),
            ("macos-arm64", ["self-hosted", "rust-ci", "macOS", "ARM64"]),
            ("windows-x64", ["self-hosted", "rust-ci", "Windows", "X64"]))
HOSTED = ("ubuntu-24.04", "macos-14", "windows-2022")
LOCAL_CONDITION = "github.event_name == 'push' || github.event_name == 'workflow_dispatch'"
HOSTED_CONDITION = "github.event_name == 'pull_request'"


def steps(pin=MATRIX_PIN, install=INSTALL, extra_runs=(), extra_steps=()):
    runs = [install,
            f"cargo {pin} install cargo-deny --version 0.20.2 --locked",
            f"cargo {pin} build --locked --all-targets",
            f"cargo {pin} test --locked --no-fail-fast",
            f"cargo {pin} fmt --all -- --check",
            f"cargo {pin} clippy --locked --all-targets -- -D warnings",
            f"cargo {pin} deny --locked check advisories bans licenses sources", *extra_runs]
    result = [{"run": run} for run in runs]
    result.append({"if": "runner.os == 'Windows'", "run": "python -B scripts/install-conpty-runtime.py target/debug/deps --arch x64"})
    result.extend(copy.deepcopy(s) for s in extra_steps)
    return result


def workflow(toolchains=PINS, extra_runs=(), extra_steps=(), pin=MATRIX_PIN,
             env={"TOOLCHAIN": "${{ matrix.toolchain }}"}, install=INSTALL, hosted=True):
    """A workflow in memory in the shape the static checks read: a platform
    job on the three machines by their labels for push and dispatch, the same
    steps on hosted runners for pull requests, each matrix entry pinning a
    toolchain the job's environment takes up, each step installing it and
    running the five commands through the pin."""
    local = {"name": "platform (${{ matrix.name }})", "if": LOCAL_CONDITION, "runs-on": "${{ matrix.runner }}",
             "strategy": {"matrix": {"include": [{"name": name, "runner": list(labels), "toolchain": tc}
                                                 for (name, labels), tc in zip(MACHINES, toolchains)]}},
             "steps": steps(pin, install, extra_runs, extra_steps)}
    jobs = {"platform": local}
    if hosted:
        jobs["platform-hosted"] = {"name": "platform-hosted (${{ matrix.os }})", "if": HOSTED_CONDITION,
                                   "runs-on": "${{ matrix.os }}",
                                   "strategy": {"matrix": {"include": [{"os": os, "toolchain": tc}
                                                                       for os, tc in zip(HOSTED, toolchains)]}},
                                   "steps": steps(pin, install, extra_runs, extra_steps)}
    for j in jobs.values():
        if env is not None:
            j["env"] = dict(env)
    jobs["advisories"] = advisories()
    return {"jobs": jobs}


def advisories(toolchain: "str | None" = "1.95.0", install='rustup toolchain install "$TOOLCHAIN" --profile minimal',
               cargo='cargo "+$TOOLCHAIN" install cargo-audit --version 0.22.1 --locked',
               runs_on=("self-hosted", "rust-ci", "Linux", "X64"), condition="github.event_name == 'schedule'"):
    """The weekly job in the same shape: the Linux machine, TOOLCHAIN set on
    the job, a step that installs it and cargo calls through the pin."""
    job = {"if": condition, "runs-on": list(runs_on) if isinstance(runs_on, tuple) else runs_on,
           "steps": [{"run": "\n".join(part for part in (install, cargo) if part)},
                     {"run": "python -B scripts/check-pre-release-dependency-code-quality.py DQC-001"}]}
    if toolchain is not None:
        job["env"] = {"TOOLCHAIN": toolchain, "RUSTUP_TOOLCHAIN": toolchain}
    return job


ATTRIBUTES = "* text=auto eol=lf\n"


class StaticChecks(unittest.TestCase):
    def violations(self, wf, version="1.95"):
        return ci.static_violations(wf, ATTRIBUTES, version)

    def assertViolation(self, found, text):
        self.assertTrue(any(text in v for v in found), f"{text!r} not among {found}")

    def test_a_workflow_on_the_three_machines_pinned_at_the_floor_passes(self):
        self.assertEqual(self.violations(workflow()), [])

    # Where each event runs.

    def test_a_push_job_on_hosted_runners_fails(self):
        wf = workflow(hosted=False)
        wf["jobs"]["platform"]["runs-on"] = "${{ matrix.os }}"
        for entry, os in zip(wf["jobs"]["platform"]["strategy"]["matrix"]["include"], HOSTED):
            entry["os"] = os
        found = self.violations(wf)
        self.assertViolation(found, "a push run sends the linux-x64 job to labels other than one of the three machines' (ubuntu-24.04)")
        self.assertViolation(found, "no push job runs on the Linux machine")

    def test_a_platform_job_without_an_event_condition_runs_everywhere_and_fails(self):
        wf = workflow(hosted=False)
        del wf["jobs"]["platform"]["if"]
        found = self.violations(wf)
        self.assertViolation(found, "a pull request's run sends the linux-x64 job to a self-hosted label")
        self.assertViolation(found, "runs on the schedule, which the platform matrix must not")

    def test_a_pull_request_on_a_self_hosted_label_fails(self):
        wf = workflow()
        wf["jobs"]["platform-hosted"]["runs-on"] = ["self-hosted", "rust-ci", "Linux", "X64"]
        self.assertViolation(self.violations(wf), "a pull request's run sends the ubuntu-24.04 job to a self-hosted label")

    def test_a_missing_machine_fails(self):
        wf = workflow()
        del wf["jobs"]["platform"]["strategy"]["matrix"]["include"][2]
        found = self.violations(wf)
        self.assertViolation(found, "no push job runs on the Windows machine")
        self.assertViolation(found, "no workflow_dispatch job runs on the Windows machine")

    def test_labels_short_of_a_machines_fail(self):
        for labels in (["self-hosted", "Linux", "X64"], ["self-hosted", "rust-ci", "X64"], ["self-hosted", "rust-ci", "Linux"]):
            wf = workflow()
            wf["jobs"]["platform"]["strategy"]["matrix"]["include"][0]["runner"] = labels
            self.assertViolation(self.violations(wf), "sends the linux-x64 job to labels other than one of the three machines'")

    def test_a_condition_the_gate_cannot_read_fails(self):
        wf = workflow()
        wf["jobs"]["platform"]["if"] = "contains(github.ref, 'main')"
        self.assertViolation(self.violations(wf), "the platform job's condition is one the gate cannot read")

    def test_a_runs_on_expression_the_gate_cannot_read_fails(self):
        wf = workflow()
        wf["jobs"]["platform"]["runs-on"] = "${{ fromJSON(matrix.runner) }}"
        self.assertViolation(self.violations(wf), "the linux-x64 job's runs-on is an expression the gate cannot read")

    def test_a_name_the_gate_cannot_read_fails(self):
        wf = workflow()
        wf["jobs"]["platform"]["name"] = "platform (${{ matrix.name }} on ${{ runner.os }})"
        self.assertViolation(self.violations(wf), "the platform job's name is an expression the gate cannot read")

    def test_the_advisories_job_must_run_on_the_linux_machine(self):
        for runs_on in ("ubuntu-24.04", ["self-hosted", "rust-ci", "macOS", "ARM64"], ["self-hosted", "Linux", "X64"]):
            wf = workflow()
            wf["jobs"]["advisories"] = advisories(runs_on=runs_on)
            self.assertViolation(self.violations(wf), "the advisories job does not run on the Linux machine")

    def test_no_scheduled_job_fails(self):
        wf = workflow()
        del wf["jobs"]["advisories"]
        self.assertViolation(self.violations(wf), "no job runs on the schedule")

    # The pins.

    def test_an_unpinned_matrix_fails_per_job(self):
        wf = workflow()
        for entry in wf["jobs"]["platform"]["strategy"]["matrix"]["include"]:
            del entry["toolchain"]
        found = self.violations(wf)
        self.assertEqual(sum("not pinned in full" in v for v in found), 3, found)

    def test_a_pin_without_a_patch_number_is_not_in_full(self):
        found = self.violations(workflow(toolchains=("1.95", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertViolation(found, "linux-x64 job's toolchain is not pinned in full")

    def test_a_pin_below_the_floor_fails(self):
        found = self.violations(workflow(toolchains=("1.91.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertViolation(found, "below Cargo.toml's rust-version 1.95")

    def test_a_gnu_windows_toolchain_fails(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-gnu")))
        self.assertViolation(found, "names a GNU toolchain")
        self.assertViolation(found, "does not name the MSVC toolchain")

    def test_a_windows_pin_without_the_triple_is_not_msvc_in_full(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0")))
        self.assertViolation(found, "the windows-x64 job's toolchain 1.95.0 does not name the MSVC toolchain in full")
        self.assertViolation(found, "the windows-2022 job's toolchain 1.95.0 does not name the MSVC toolchain in full")

    def test_rustup_default_fails(self):
        found = self.violations(workflow(extra_runs=("rustup default 1.95.0",)))
        self.assertViolation(found, "`rustup default`")

    def test_rustup_set_default_host_fails(self):
        found = self.violations(workflow(extra_runs=("rustup set default-host x86_64-pc-windows-msvc",)))
        self.assertViolation(found, "`rustup set default-host`")

    def test_a_default_change_in_another_job_fails_too(self):
        wf = workflow()
        wf["jobs"]["advisories"]["steps"].append({"run": "rustup default 1.95.0"})
        self.assertViolation(self.violations(wf), "`rustup default`")

    def test_a_bare_cargo_call_fails(self):
        found = self.violations(workflow(extra_runs=("cargo doc --no-deps",)))
        self.assertViolation(found, "names no toolchain: `cargo doc`")

    def test_a_bare_cargo_call_fails_even_under_rustup_toolchain(self):
        env = {"TOOLCHAIN": "${{ matrix.toolchain }}", "RUSTUP_TOOLCHAIN": "${{ matrix.toolchain }}"}
        found = self.violations(workflow(env=env, extra_runs=("cargo doc --no-deps",)))
        self.assertViolation(found, "names no toolchain: `cargo doc`")

    def test_a_matrix_reference_in_the_call_counts_as_the_pin(self):
        self.assertEqual(self.violations(workflow(pin="+${{ matrix.toolchain }}", env=None,
                                                  install="rustup toolchain install ${{ matrix.toolchain }}")), [])

    def test_a_missing_command_fails(self):
        wf = workflow()
        wf["jobs"]["platform"]["steps"] = [s for s in wf["jobs"]["platform"]["steps"] if "fmt" not in str(s.get("run"))]
        self.assertViolation(self.violations(wf), "the linux-x64 job does not run `cargo fmt --all -- --check`")

    def test_a_pin_variable_the_job_does_not_set_fails(self):
        found = self.violations(workflow(env=None))
        self.assertViolation(found, "a toolchain the gate cannot read")

    def test_an_expression_the_gate_cannot_read_fails(self):
        env = {"TOOLCHAIN": "${{ runner.os == 'Linux' && '1.96.0' || matrix.toolchain }}"}
        found = self.violations(workflow(env=env))
        self.assertViolation(found, "TOOLCHAIN is an expression the gate cannot read")

    def test_a_step_that_overrides_the_toolchain_fails(self):
        for variable in ("TOOLCHAIN", "RUSTUP_TOOLCHAIN"):
            wf = workflow()
            build = next(s for s in wf["jobs"]["platform"]["steps"] if "build" in str(s.get("run")))
            build["env"] = {variable: "1.96.0"}
            found = self.violations(wf)
            self.assertViolation(found, f"overrides {variable}")
            if variable == "TOOLCHAIN":
                self.assertViolation(found, "the linux-x64 job runs `cargo build --locked --all-targets` at 1.96.0, not its pin 1.95.0")

    def test_rustup_toolchain_on_the_job_must_name_the_pin(self):
        env = {"TOOLCHAIN": "${{ matrix.toolchain }}", "RUSTUP_TOOLCHAIN": "stable"}
        self.assertViolation(self.violations(workflow(env=env)), "sets RUSTUP_TOOLCHAIN to stable, not its pin")

    def test_a_call_at_another_release_fails(self):
        found = self.violations(workflow(extra_runs=("cargo +1.96.0 doc --no-deps",)))
        self.assertViolation(found, "runs `cargo doc --no-deps` at 1.96.0, not its pin 1.95.0")

    def test_a_literal_release_on_windows_is_not_its_msvc_pin(self):
        found = self.violations(workflow(pin="+1.95.0"))
        self.assertViolation(found, "the windows-x64 job runs `cargo build --locked --all-targets` at 1.95.0, not its pin 1.95.0-x86_64-pc-windows-msvc")
        self.assertFalse(any("linux-x64" in v or "macos-arm64" in v for v in found), found)

    def test_an_install_of_another_release_fails(self):
        found = self.violations(workflow(install="rustup toolchain install 1.96.0 --profile minimal"))
        self.assertViolation(found, "installs 1.96.0, not its pin 1.95.0")

    def test_no_install_fails(self):
        found = self.violations(workflow(install="echo nothing installed"))
        self.assertViolation(found, "installs no toolchain")

    def test_a_linux_pin_above_the_floor_needs_a_floor_step(self):
        found = self.violations(workflow(toolchains=ABOVE))
        self.assertViolation(found, "does not build and test at rust-version 1.95")
        self.assertEqual(self.violations(workflow(toolchains=ABOVE, extra_steps=(FLOOR_STEP,))), [])

    def test_the_floor_step_must_select_the_floor(self):
        at_pin = {**FLOOR_STEP, "run": FLOOR_STEP["run"].replace("+1.95.0", "+1.99.0")}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(at_pin,)))
        self.assertViolation(found, "does not build and test at rust-version 1.95")
        elsewhere = {**FLOOR_STEP, "run": FLOOR_STEP["run"].replace("+1.95.0", "+1.96.0")}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(elsewhere,)))
        self.assertViolation(found, "the linux-x64 job runs `cargo build --locked --all-targets` at 1.96.0, not its pin 1.99.0")

    def test_the_floor_step_must_run_on_linux_only_and_install_the_floor(self):
        no_condition = {"run": FLOOR_STEP["run"]}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(no_condition,)))
        self.assertViolation(found, "the macos-arm64 job runs `cargo build --locked --all-targets` at 1.95.0, not its pin 1.99.0")
        self.assertViolation(found, "the windows-x64 job runs `cargo test --locked --no-fail-fast` at 1.95.0, not its pin")
        self.assertFalse(any("linux-x64" in v or "Linux" in v for v in found), found)
        other_os = {**FLOOR_STEP, "if": "runner.os == 'macOS'"}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(other_os,)))
        self.assertViolation(found, "does not build and test at rust-version 1.95")
        no_install = {**FLOOR_STEP, "run": "\n".join(FLOOR_STEP["run"].splitlines()[1:])}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(no_install,)))
        self.assertViolation(found, "runs at 1.95.0 but installs it nowhere")

    # The weekly advisories job under the same pin rules.

    def test_an_advisories_job_on_stable_fails(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories(toolchain="stable")
        self.assertViolation(self.violations(wf), "advisories job's toolchain is not pinned in full (found 'stable')")

    def test_an_advisories_job_below_the_floor_fails(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories(toolchain="1.91.0")
        self.assertViolation(self.violations(wf), "advisories job pins 1.91.0, below Cargo.toml's rust-version 1.95")

    def test_an_advisories_job_without_a_pin_fails(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories(toolchain=None, install="rustup toolchain install stable",
                                              cargo="cargo +stable install cargo-audit --version 0.22.1 --locked")
        self.assertViolation(self.violations(wf), "advisories job runs cargo but sets no TOOLCHAIN pin")

    def test_an_advisories_call_at_another_toolchain_fails(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories(cargo="cargo +stable install cargo-audit --version 0.22.1 --locked")
        self.assertViolation(self.violations(wf), "runs `cargo install cargo-audit --version 0.22.1 --locked` at stable, not its pin 1.95.0")

    def test_an_advisories_job_that_installs_nothing_fails(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories(install="")
        self.assertViolation(self.violations(wf), "advisories job installs no toolchain")

    def test_a_job_without_cargo_needs_no_pin(self):
        wf = workflow()
        wf["jobs"]["docs"] = {"if": "github.event_name == 'pull_request'", "runs-on": "ubuntu-24.04",
                              "steps": [{"run": "echo documentation only"}]}
        self.assertEqual(self.violations(wf), [])

    def test_the_conpty_step_is_required_on_each_platform_job(self):
        wf = workflow()
        wf["jobs"]["platform-hosted"]["steps"] = [s for s in wf["jobs"]["platform-hosted"]["steps"] if "conpty" not in str(s.get("run"))]
        self.assertViolation(self.violations(wf), "no Windows step of the platform-hosted job installs the ConPTY runtime")


if __name__ == "__main__":
    unittest.main()
