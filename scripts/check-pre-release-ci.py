#!/usr/bin/env python3
"""Inspect the maintained CI workflow and exercise its event selection."""
from pathlib import Path
import re
import subprocess
import sys

import yaml


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/ci.yml"
PLATFORMS = ["ubuntu-24.04", "macos-14", "windows-2022"]
COMMANDS = (
    "cargo +1.91.0 build --locked --all-targets",
    "cargo +1.91.0 test --locked --no-fail-fast",
    "cargo +1.91.0 fmt --all -- --check",
    "cargo +1.91.0 clippy --locked --all-targets -- -D warnings",
    "python -B scripts/check-pre-release-dependency-maintenance.py DQC-002",
)
ADVISORY = "python -B scripts/check-pre-release-dependency-code-quality.py DQC-001"
CONDITIONS = {"platform": "github.event_name != 'schedule'",
              "advisories": "github.event_name == 'schedule'"}


def selected_jobs(workflow: dict, event: str, branch: str) -> list[str]:
    trigger = workflow.get("on", {}).get(event)
    if trigger is None:
        return []
    if event in ("push", "pull_request") and branch not in trigger.get("branches", []):
        return []
    result = []
    for name, job in workflow.get("jobs", {}).items():
        condition = job.get("if")
        if condition == CONDITIONS["platform"] and event != "schedule":
            matrix = job.get("strategy", {}).get("matrix", {}).get("os", [])
            result.extend(f"{name} ({os_name})" for os_name in matrix)
        elif condition == CONDITIONS["advisories"] and event == "schedule":
            result.append(name)
    return result


def validate_steps(name: str, job: dict, required: tuple[str, ...]) -> list[str]:
    errors = []
    commands = []
    checkout = False
    for step in job.get("steps", []):
        if step.get("continue-on-error", "false") != "false":
            errors.append(f"{name}: a step permits failure")
        if "working-directory" in step:
            errors.append(f"{name}: a step changes the repository root")
        if "uses" in step:
            if not re.fullmatch(r"[\w.-]+/[\w./-]+@[0-9a-f]{40}", step["uses"]):
                errors.append(f"{name}: unpinned action {step['uses']}")
            if step["uses"].startswith("actions/checkout@"):
                checkout = ("if" not in step and
                            step.get("with", {}).get("persist-credentials") == "false" and
                            not set(step.get("with", {})) - {"persist-credentials"})
        command = step.get("run", "")
        if command in required:
            if "if" in step:
                errors.append(f"{name}: conditional required command {command}")
            if step.get("shell", "bash") != "bash":
                errors.append(f"{name}: required command does not use fail-fast bash")
            commands.append(command)
        for flag in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            if flag in step.get("env", {}):
                errors.append(f"{name}: step overrides {flag}")
    if not checkout:
        errors.append(f"{name}: missing unconditional pinned checkout without credentials")
    for command in required:
        if commands.count(command) != 1:
            errors.append(f"{name}: missing or duplicated required command {command}")
    return errors


def validate_workflow(workflow: dict) -> list[str]:
    errors = []
    if not isinstance(workflow, dict):
        return ["workflow must be a mapping"]
    triggers = workflow.get("on", {})
    for event in ("push", "pull_request"):
        if triggers.get(event) != {"branches": ["main"]}:
            errors.append(f"{event} must cover every main-branch change without filters")
    schedule = triggers.get("schedule", [])
    if not (isinstance(schedule, list) and schedule and
            all(isinstance(item, dict) and set(item) == {"cron"} and
                item["cron"] == "23 4 * * 1" for item in schedule)):
        errors.append("missing maintained weekly advisory schedule")
    if "workflow_dispatch" not in triggers:
        errors.append("missing manual trigger")
    if workflow.get("permissions") != {"contents": "read"}:
        errors.append("workflow permissions must be contents: read only")
    if "defaults" in workflow:
        errors.append("workflow-wide command defaults are not supported")
    jobs = workflow.get("jobs", {})
    for name, required in (("platform", COMMANDS), ("advisories", (ADVISORY,))):
        job = jobs.get(name)
        if not isinstance(job, dict):
            errors.append(f"missing {name} job")
            continue
        if job.get("if") != CONDITIONS[name]:
            errors.append(f"{name}: unexpected event condition")
        if job.get("continue-on-error", "false") != "false" or "needs" in job:
            errors.append(f"{name}: failures or dependencies can skip a required result")
        if "permissions" in job or "container" in job or "environment" in job:
            errors.append(f"{name}: unsupported permissions or execution overrides")
        try:
            bounded = 0 < int(job.get("timeout-minutes", "0")) <= 60
        except (ValueError, TypeError):
            bounded = False
        if not bounded:
            errors.append(f"{name}: missing bounded job timeout")
        if job.get("defaults", {"run": {"shell": "bash"}}) != {"run": {"shell": "bash"}}:
            errors.append(f"{name}: commands must run in fail-fast bash at the repository root")
        errors.extend(validate_steps(name, job, required))
        for flag in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            if flag in job.get("env", {}) or flag in workflow.get("env", {}):
                errors.append(f"{name}: workflow or job overrides {flag}")
    platform = jobs.get("platform", {})
    if platform.get("runs-on") != "${{ matrix.os }}" or platform.get("strategy") != {
            "fail-fast": "false", "matrix": {"os": PLATFORMS}}:
        errors.append("platform job must run all three supported operating systems")
    if jobs.get("advisories", {}).get("runs-on") != "ubuntu-24.04":
        errors.append("advisory job must use its maintained Linux runner")
    expected = [f"platform ({os_name})" for os_name in PLATFORMS]
    for event in ("push", "pull_request", "workflow_dispatch"):
        if selected_jobs(workflow, event, "main") != expected:
            errors.append(f"{event} fixture did not select every supported platform")
    if selected_jobs(workflow, "schedule", "main") != ["advisories"]:
        errors.append("schedule fixture did not select advisory checks")
    return errors


def main() -> int:
    if sys.argv[1:] != ["RID-001"]:
        print("usage: check-pre-release-ci.py RID-001", file=sys.stderr)
        return 2
    unit = subprocess.run([sys.executable, "-B", "scripts/test-pre-release-ci.py"],
                          cwd=ROOT, timeout=60, check=False)
    if unit.returncode:
        return unit.returncode
    try:
        # BaseLoader keeps GitHub's `on` key and boolean scalars as strings.
        # No YAML object constructors run on repository content.
        workflow = yaml.load(WORKFLOW.read_text(), Loader=yaml.BaseLoader)
        errors = validate_workflow(workflow)
    except (OSError, yaml.YAMLError, TypeError, AttributeError) as error:
        errors = [f"cannot inspect {WORKFLOW.relative_to(ROOT)}: {error}"]
    if errors:
        print("RID-001 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    for event in ("push", "pull_request", "schedule", "workflow_dispatch"):
        print(f"{event}: {', '.join(selected_jobs(workflow, event, 'main'))}")
    print("RID-001 workflow inspection passed; this is not evidence of native CI execution.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
