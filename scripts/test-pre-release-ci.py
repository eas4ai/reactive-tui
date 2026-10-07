#!/usr/bin/env python3
"""Attack workflow selection and mandatory commands without running CI."""
import copy
from pathlib import Path
import unittest

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts/check-pre-release-ci.py"
PIN = '"+$TOOLCHAIN"'
COMMANDS = (
    f"cargo {PIN} build --locked --all-targets",
    f"cargo {PIN} test --locked --no-fail-fast",
    f"cargo {PIN} fmt --all -- --check",
    f"cargo {PIN} clippy --locked --all-targets -- -D warnings",
    f"cargo {PIN} deny --locked check advisories bans licenses sources",
)
MATRIX = [{"os": "ubuntu-24.04", "toolchain": "1.95.0"},
          {"os": "macos-14", "toolchain": "1.95.0"},
          {"os": "windows-2022", "toolchain": "1.95.0-x86_64-pc-windows-msvc"}]
PLATFORM_INSTALL = ('rustup toolchain install "$TOOLCHAIN" --profile minimal --component rustfmt --component clippy\n'
                    'cargo "+$TOOLCHAIN" install cargo-deny --version 0.20.2 --locked')
ADVISORY_INSTALL = ('rustup toolchain install "$TOOLCHAIN" --profile minimal\n'
                    'cargo "+$TOOLCHAIN" install cargo-audit --version 0.22.1 --locked\n'
                    'cargo "+$TOOLCHAIN" install cargo-deny --version 0.20.2 --locked')
FLOOR_STEP = {"if": "runner.os == 'Linux'",
              "run": "rustup toolchain install 1.95.0 --profile minimal\n"
                     "cargo +1.95.0 build --locked --all-targets\ncargo +1.95.0 test --locked --no-fail-fast"}


def fixture():
    return {
        "on": {"push": {"branches": ["main"]},
               "pull_request": {"branches": ["main"]},
               "schedule": [{"cron": "23 4 * * 1"}], "workflow_dispatch": {}},
        "permissions": {"contents": "read"},
        "jobs": {
            "platform": {
                "if": "github.event_name != 'schedule'",
                "runs-on": "${{ matrix.os }}", "timeout-minutes": "45",
                "strategy": {"fail-fast": "false", "matrix": {"include": copy.deepcopy(MATRIX)}},
                "defaults": {"run": {"shell": "bash"}},
                "env": {"TOOLCHAIN": "${{ matrix.toolchain }}", "RUSTUP_TOOLCHAIN": "${{ matrix.toolchain }}"},
                "steps": [{"uses": "actions/checkout@" + "1" * 40,
                           "with": {"persist-credentials": "false"}},
                          {"run": PLATFORM_INSTALL},
                          *({"run": command} for command in COMMANDS)],
            },
            "advisories": {
                "if": "github.event_name == 'schedule'", "runs-on": "ubuntu-24.04",
                "timeout-minutes": "30",
                "env": {"TOOLCHAIN": "1.95.0", "RUSTUP_TOOLCHAIN": "1.95.0"},
                "steps": [{"uses": "actions/checkout@" + "1" * 40,
                           "with": {"persist-credentials": "false"}},
                          {"run": ADVISORY_INSTALL},
                          {"run": "python -B scripts/check-pre-release-dependency-code-quality.py DQC-001"}],
            },
        },
    }


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        self.checker = load_checker(CHECKER, "rid_ci", "RID workflow checker")
        self.workflow = fixture()

    def test_complete_workflow_is_accepted(self):
        self.assertEqual(self.checker.validate_workflow(self.workflow), [])
        self.assertEqual(self.checker.validate_workflow(self.workflow, floor="1.95"), [])

    def test_main_events_select_every_platform(self):
        for event in ("push", "pull_request", "workflow_dispatch"):
            self.assertEqual(self.checker.selected_jobs(self.workflow, event, "main"),
                             ["platform (ubuntu-24.04)", "platform (macos-14)",
                              "platform (windows-2022)"])
        self.assertEqual(self.checker.selected_jobs(self.workflow, "push", "other"), [])
        self.assertEqual(self.checker.selected_jobs(self.workflow, "pull_request", "other"), [])
        self.assertEqual(self.checker.selected_jobs(self.workflow, "schedule", "main"),
                         ["advisories"])

    def test_each_missing_platform_is_rejected(self):
        for index, entry in enumerate(MATRIX):
            changed = copy.deepcopy(self.workflow)
            del changed["jobs"]["platform"]["strategy"]["matrix"]["include"][index]
            self.assertTrue(self.checker.validate_workflow(changed), entry["os"])

    def test_an_os_list_without_toolchains_is_rejected(self):
        changed = copy.deepcopy(self.workflow)
        changed["jobs"]["platform"]["strategy"]["matrix"] = {"os": [e["os"] for e in MATRIX]}
        self.assertTrue(self.checker.validate_workflow(changed))

    def test_unpinned_gnu_and_detached_toolchains_are_rejected(self):
        for index, toolchain in ((0, "1.95"), (0, "stable"), (2, "1.95.0-x86_64-pc-windows-gnu"), (2, "1.95.0")):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"]["platform"]["strategy"]["matrix"]["include"][index]["toolchain"] = toolchain
            self.assertTrue(self.checker.validate_workflow(changed), toolchain)
        changed = copy.deepcopy(self.workflow)
        changed["jobs"]["platform"]["env"]["TOOLCHAIN"] = "1.95.0"
        self.assertTrue(self.checker.validate_workflow(changed), "TOOLCHAIN not from the matrix")

    def test_a_default_toolchain_or_host_change_is_rejected(self):
        for command in ("rustup default 1.95.0", "rustup set default-host x86_64-pc-windows-msvc"):
            for job in ("platform", "advisories"):
                changed = copy.deepcopy(self.workflow)
                changed["jobs"][job]["steps"].insert(1, {"run": command})
                self.assertTrue(self.checker.validate_workflow(changed), f"{job}: {command}")

    # Finding 1 of the ci-rust-version-bump review: what a step runs at is the
    # environment it sees, so an override or another literal is rejected.

    def test_a_step_level_toolchain_override_is_rejected(self):
        for job in ("platform", "advisories"):
            for variable in ("TOOLCHAIN", "RUSTUP_TOOLCHAIN"):
                changed = copy.deepcopy(self.workflow)
                changed["jobs"][job]["steps"][1]["env"] = {variable: "1.96.0"}
                errors = self.checker.validate_workflow(changed)
                self.assertTrue(any(f"a step overrides {variable}" in e for e in errors), (job, variable, errors))

    def test_a_cargo_call_at_another_toolchain_or_none_is_rejected(self):
        for job, command, expected in (("platform", "cargo +1.96.0 doc --no-deps", "not the pin"),
                                       ("advisories", "cargo +stable install cargo-outdated --locked", "not the pin"),
                                       ("advisories", "cargo install cargo-outdated --locked", "names no toolchain")):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"][job]["steps"].insert(2, {"run": command})
            errors = self.checker.validate_workflow(changed)
            self.assertTrue(any(e.startswith(job) and expected in e for e in errors), (job, command, errors))

    def test_rustup_toolchain_must_name_the_pin(self):
        for job in ("platform", "advisories"):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"][job]["env"]["RUSTUP_TOOLCHAIN"] = "stable"
            errors = self.checker.validate_workflow(changed)
            self.assertTrue(any("RUSTUP_TOOLCHAIN" in e for e in errors), (job, errors))

    def test_a_missing_install_is_rejected(self):
        for job in ("platform", "advisories"):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"][job]["steps"][1]["run"] = "\n".join(
                line for line in changed["jobs"][job]["steps"][1]["run"].splitlines() if "rustup" not in line)
            errors = self.checker.validate_workflow(changed)
            self.assertTrue(any("installs the pinned toolchain" in e for e in errors), (job, errors))

    def test_a_pin_below_the_floor_is_rejected(self):
        errors = self.checker.validate_workflow(self.workflow, floor="1.96")
        for label in ("platform (ubuntu-24.04)", "platform (macos-14)", "platform (windows-2022)", "advisories"):
            self.assertTrue(any(e.startswith(label) and "below rust-version 1.96" in e for e in errors), (label, errors))
        # A floor under the pin is not a defect of the pin, but the Linux job
        # then owes a build and test at that floor.
        errors = self.checker.validate_workflow(self.workflow, floor="1.90")
        self.assertEqual(errors, ["platform (ubuntu-24.04): the pin is above rust-version 1.90 and no Linux step builds and tests at it"])

    def test_a_linux_floor_step_is_accepted_only_at_the_floor(self):
        above = copy.deepcopy(self.workflow)
        for entry in above["jobs"]["platform"]["strategy"]["matrix"]["include"]:
            entry["toolchain"] = entry["toolchain"].replace("1.95.0", "1.99.0")
        above["jobs"]["advisories"]["env"] = {"TOOLCHAIN": "1.99.0", "RUSTUP_TOOLCHAIN": "1.99.0"}
        self.assertTrue(self.checker.validate_workflow(above, floor="1.95"), "no floor step")
        with_floor = copy.deepcopy(above)
        with_floor["jobs"]["platform"]["steps"].append(copy.deepcopy(FLOOR_STEP))
        self.assertEqual(self.checker.validate_workflow(with_floor, floor="1.95"), [])
        elsewhere = copy.deepcopy(above)
        elsewhere["jobs"]["platform"]["steps"].append({**FLOOR_STEP, "run": FLOOR_STEP["run"].replace("1.95.0", "1.96.0")})
        self.assertTrue(self.checker.validate_workflow(elsewhere, floor="1.95"), "a step at 1.96.0 is neither pin nor floor")

    # Finding 2: the weekly advisories job is held to the same pin rules.

    def test_an_unpinned_advisories_toolchain_is_rejected(self):
        for env in ({"TOOLCHAIN": "stable", "RUSTUP_TOOLCHAIN": "stable"},
                    {"TOOLCHAIN": "1.95", "RUSTUP_TOOLCHAIN": "1.95"},
                    {"TOOLCHAIN": "${{ env.RUST }}"},
                    {}):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"]["advisories"]["env"] = env
            errors = self.checker.validate_workflow(changed)
            self.assertTrue(any(e.startswith("advisories") and "not pinned in full" in e for e in errors), (env, errors))

    def test_each_missing_command_is_rejected(self):
        for command in COMMANDS:
            changed = copy.deepcopy(self.workflow)
            steps = changed["jobs"]["platform"]["steps"]
            changed["jobs"]["platform"]["steps"] = [s for s in steps if s.get("run") != command]
            self.assertTrue(self.checker.validate_workflow(changed), command)

    def test_skip_paths_disabled_jobs_and_allowed_failures_are_rejected(self):
        cases = []
        for event in ("push", "pull_request"):
            for key, value in (("branches", ["other"]), ("paths", ["src/**"]),
                               ("paths-ignore", ["docs/**"]), ("types", ["opened"])):
                changed = copy.deepcopy(self.workflow)
                changed["on"][event][key] = value
                cases.append(changed)
        for field, value in (("if", "false"), ("continue-on-error", "true"),
                             ("needs", ["optional"]), ("env", {"RUSTFLAGS": "-A warnings"})):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"]["platform"][field] = value
            cases.append(changed)
        for field, value in (("if", "runner.os == 'Linux'"),
                             ("continue-on-error", "true"), ("working-directory", "other")):
            changed = copy.deepcopy(self.workflow)
            changed["jobs"]["platform"]["steps"][2][field] = value
            cases.append(changed)
        for changed in cases:
            self.assertTrue(self.checker.validate_workflow(changed), changed)

    def test_unlocked_and_success_masked_commands_are_rejected(self):
        for command in COMMANDS:
            for replacement in (command.replace("--locked", ""), command + " || true",
                                "echo '" + command + "'"):
                if replacement == command:
                    continue
                changed = copy.deepcopy(self.workflow)
                for step in changed["jobs"]["platform"]["steps"]:
                    if step.get("run") == command:
                        step["run"] = replacement
                self.assertTrue(self.checker.validate_workflow(changed), replacement)

    def test_missing_schedule_or_advisory_command_is_rejected(self):
        for target in ("schedule", "advisories", "command"):
            changed = copy.deepcopy(self.workflow)
            if target == "schedule":
                del changed["on"]["schedule"]
            elif target == "advisories":
                del changed["jobs"]["advisories"]
            else:
                changed["jobs"]["advisories"]["steps"][-1]["run"] += " || true"
            self.assertTrue(self.checker.validate_workflow(changed), target)

    def test_floating_actions_and_write_permissions_are_rejected(self):
        self.workflow["jobs"]["platform"]["steps"][0]["uses"] = "actions/checkout@v4"
        self.assertTrue(self.checker.validate_workflow(self.workflow))
        self.workflow = fixture()
        self.workflow["permissions"]["contents"] = "write"
        self.assertTrue(self.checker.validate_workflow(self.workflow))


if __name__ == "__main__":
    unittest.main()
