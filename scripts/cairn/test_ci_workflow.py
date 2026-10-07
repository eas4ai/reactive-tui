#!/usr/bin/env python3
"""The ci-workflow mechanism's own tests: which runs and jobs may satisfy
BAR-012 (no scheduled run, every platform job successful), and what the
static reading of the workflow accepts and refuses, without gh."""
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


MATRIX_PIN = '"+$TOOLCHAIN"'
INSTALL = 'rustup toolchain install "$TOOLCHAIN" --profile minimal --component rustfmt --component clippy'
FLOOR_STEP = {"if": "runner.os == 'Linux'",
              "run": "rustup toolchain install 1.95.0 --profile minimal\n"
                     "cargo +1.95.0 build --locked --all-targets\ncargo +1.95.0 test --locked --no-fail-fast"}
ABOVE = ("1.99.0", "1.99.0", "1.99.0-x86_64-pc-windows-msvc")


def workflow(toolchains=("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc"), extra_runs=(), extra_steps=(),
             pin=MATRIX_PIN, env={"TOOLCHAIN": "${{ matrix.toolchain }}"}, install=INSTALL):
    """A workflow in memory in the shape the static checks read: a platform
    job whose matrix pins a toolchain per runner, whose environment takes
    TOOLCHAIN from the entry, and whose steps install that toolchain and run
    the five commands through the pin."""
    include = [{"os": os, "toolchain": tc} for os, tc in zip(("ubuntu-24.04", "macos-14", "windows-2022"), toolchains)]
    runs = [install,
            f"cargo {pin} install cargo-deny --version 0.20.2 --locked",
            f"cargo {pin} build --locked --all-targets",
            f"cargo {pin} test --locked --no-fail-fast",
            f"cargo {pin} fmt --all -- --check",
            f"cargo {pin} clippy --locked --all-targets -- -D warnings",
            f"cargo {pin} deny --locked check advisories bans licenses sources", *extra_runs]
    steps = [{"run": run} for run in runs]
    steps.append({"if": "runner.os == 'Windows'", "run": "python -B scripts/install-conpty-runtime.py target/debug/deps --arch x64"})
    steps.extend(extra_steps)
    platform = {"strategy": {"matrix": {"include": include}}, "steps": steps}
    if env is not None:
        platform["env"] = dict(env)
    return {"jobs": {"platform": platform}}


def advisories(toolchain: str | None = "1.95.0", install='rustup toolchain install "$TOOLCHAIN" --profile minimal',
               cargo='cargo "+$TOOLCHAIN" install cargo-audit --version 0.22.1 --locked'):
    """The weekly job in the same shape: a Linux runner, TOOLCHAIN set on the
    job, a step that installs it and cargo calls through the pin."""
    job = {"runs-on": "ubuntu-24.04",
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

    def test_a_workflow_pinned_per_job_at_the_floor_passes(self):
        self.assertEqual(self.violations(workflow()), [])

    def test_an_unpinned_matrix_fails_per_job(self):
        wf = {"jobs": {"platform": {"strategy": {"matrix": {"os": ["ubuntu-24.04", "macos-14", "windows-2022"]}},
                                    "steps": workflow()["jobs"]["platform"]["steps"]}}}
        found = self.violations(wf)
        self.assertEqual(sum("not pinned in full" in v for v in found), 3, found)

    def test_a_pin_without_a_patch_number_is_not_in_full(self):
        found = self.violations(workflow(toolchains=("1.95", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertViolation(found, "ubuntu-24.04 job's toolchain is not pinned in full")

    def test_a_pin_below_the_floor_fails(self):
        found = self.violations(workflow(toolchains=("1.91.0", "1.95.0", "1.95.0-x86_64-pc-windows-msvc")))
        self.assertViolation(found, "below Cargo.toml's rust-version 1.95")

    def test_a_gnu_windows_toolchain_fails(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0-x86_64-pc-windows-gnu")))
        self.assertViolation(found, "names a GNU toolchain")
        self.assertViolation(found, "does not name the MSVC toolchain")

    def test_a_windows_pin_without_the_triple_is_not_msvc_in_full(self):
        found = self.violations(workflow(toolchains=("1.95.0", "1.95.0", "1.95.0")))
        self.assertViolation(found, "does not name the MSVC toolchain in full")

    def test_rustup_default_fails(self):
        found = self.violations(workflow(extra_runs=("rustup default 1.95.0",)))
        self.assertViolation(found, "`rustup default`")

    def test_rustup_set_default_host_fails(self):
        found = self.violations(workflow(extra_runs=("rustup set default-host x86_64-pc-windows-msvc",)))
        self.assertViolation(found, "`rustup set default-host`")

    def test_a_default_change_in_another_job_fails_too(self):
        wf = workflow()
        wf["jobs"]["advisories"] = {"steps": [{"run": "rustup toolchain install 1.95.0\nrustup default 1.95.0"}]}
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
        self.assertViolation(self.violations(wf), "does not run `cargo fmt --all -- --check`")

    # Finding 1 of the ci-rust-version-bump review: the pin a call names must
    # resolve, through the environment the step sees, to the job's pin.

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
                self.assertViolation(found, "runs `cargo build --locked --all-targets` at 1.96.0, not its pin 1.95.0")

    def test_rustup_toolchain_on_the_job_must_name_the_pin(self):
        env = {"TOOLCHAIN": "${{ matrix.toolchain }}", "RUSTUP_TOOLCHAIN": "stable"}
        self.assertViolation(self.violations(workflow(env=env)), "sets RUSTUP_TOOLCHAIN to stable, not its pin")

    def test_a_call_at_another_release_fails(self):
        found = self.violations(workflow(extra_runs=("cargo +1.96.0 doc --no-deps",)))
        self.assertViolation(found, "runs `cargo doc --no-deps` at 1.96.0, not its pin 1.95.0")

    def test_a_literal_release_on_windows_is_not_its_msvc_pin(self):
        found = self.violations(workflow(pin="+1.95.0"))
        self.assertViolation(found, "not its pin 1.95.0-x86_64-pc-windows-msvc")
        self.assertFalse(any("ubuntu-24.04" in v or "macos-14" in v for v in found), found)

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
        self.assertViolation(found, "runs `cargo build --locked --all-targets` at 1.96.0, not its pin 1.99.0")

    def test_the_floor_step_must_run_on_linux_only_and_install_the_floor(self):
        no_condition = {"run": FLOOR_STEP["run"]}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(no_condition,)))
        self.assertViolation(found, "the macos-14 job runs `cargo build --locked --all-targets` at 1.95.0, not its pin 1.99.0")
        self.assertViolation(found, "the windows-2022 job runs `cargo test --locked --no-fail-fast` at 1.95.0, not its pin")
        self.assertFalse(any("ubuntu-24.04" in v or "Linux" in v for v in found), found)
        other_os = {**FLOOR_STEP, "if": "runner.os == 'macOS'"}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(other_os,)))
        self.assertViolation(found, "does not build and test at rust-version 1.95")
        no_install = {**FLOOR_STEP, "run": "\n".join(FLOOR_STEP["run"].splitlines()[1:])}
        found = self.violations(workflow(toolchains=ABOVE, extra_steps=(no_install,)))
        self.assertViolation(found, "runs at 1.95.0 but installs it nowhere")

    # Finding 2: the weekly advisories job is held to the same rules.

    def test_the_advisories_job_pinned_in_full_passes(self):
        wf = workflow()
        wf["jobs"]["advisories"] = advisories()
        self.assertEqual(self.violations(wf), [])

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
        wf["jobs"]["docs"] = {"runs-on": "ubuntu-24.04", "steps": [{"run": "python -B scripts/build-docs.py"}]}
        self.assertEqual(self.violations(wf), [])


if __name__ == "__main__":
    unittest.main()
