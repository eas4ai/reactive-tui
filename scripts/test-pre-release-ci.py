#!/usr/bin/env python3
"""Attack workflow selection and mandatory commands without running CI."""
import copy
from pathlib import Path
import unittest

from dependency_check_test_support import load_checker


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts/check-pre-release-ci.py"
COMMANDS = (
    "cargo +1.91.0 build --locked --all-targets",
    "cargo +1.91.0 test --locked --no-fail-fast",
    "cargo +1.91.0 fmt --all -- --check",
    "cargo +1.91.0 clippy --locked --all-targets -- -D warnings",
    "python -B scripts/check-pre-release-dependency-maintenance.py DQC-002",
)


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
                "strategy": {"fail-fast": "false", "matrix": {
                    "os": ["ubuntu-24.04", "macos-14", "windows-2022"]}},
                "defaults": {"run": {"shell": "bash"}},
                "steps": [{"uses": "actions/checkout@" + "1" * 40,
                           "with": {"persist-credentials": "false"}},
                          *({"run": command} for command in COMMANDS)],
            },
            "advisories": {
                "if": "github.event_name == 'schedule'", "runs-on": "ubuntu-24.04",
                "timeout-minutes": "30",
                "steps": [{"uses": "actions/checkout@" + "1" * 40,
                           "with": {"persist-credentials": "false"}},
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
        for os_name in fixture()["jobs"]["platform"]["strategy"]["matrix"]["os"]:
            changed = copy.deepcopy(self.workflow)
            changed["jobs"]["platform"]["strategy"]["matrix"]["os"].remove(os_name)
            self.assertTrue(self.checker.validate_workflow(changed), os_name)

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
            changed["jobs"]["platform"]["steps"][1][field] = value
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
