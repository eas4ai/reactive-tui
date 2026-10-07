#!/usr/bin/env python3
"""Inspect the maintained CI workflow and exercise its event selection."""
from pathlib import Path
import re
import subprocess
import sys

import yaml

import workflow_toolchains as wt


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/ci.yml"
MANIFEST = ROOT / "Cargo.toml"
PLATFORMS = ["ubuntu-24.04", "macos-14", "windows-2022"]
# Each job pins the toolchain its matrix entry names (BAR-012); the steps
# spell the pin as the TOOLCHAIN variable the job sets from that entry.
PIN = '"+$TOOLCHAIN"'
COMMANDS = (
    f"cargo {PIN} build --locked --all-targets",
    f"cargo {PIN} test --locked --no-fail-fast",
    f"cargo {PIN} fmt --all -- --check",
    f"cargo {PIN} clippy --locked --all-targets -- -D warnings",
    f"cargo {PIN} deny --locked check advisories bans licenses sources",
)
# The two the Linux job also runs at rust-version when its pin is above it.
FLOOR_COMMANDS = ("cargo build --locked --all-targets", "cargo test --locked --no-fail-fast")
# A stable release named in full, with the target triple on Windows.
PINNED_TOOLCHAIN = wt.RELEASE
DEFAULT_CHANGES = re.compile(r"rustup\s+(default\b|set\s+default-host\b)")
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
            result.extend(f"{name} ({os_name})" for os_name in matrix_platforms(job))
        elif condition == CONDITIONS["advisories"] and event == "schedule":
            result.append(name)
    return result


def matrix_entries(job: dict) -> list[dict]:
    """The platform job's matrix entries: `include` items, each an os and
    its pinned toolchain."""
    include = job.get("strategy", {}).get("matrix", {}).get("include", [])
    return [entry for entry in include if isinstance(entry, dict)] if isinstance(include, list) else []


def matrix_platforms(job: dict) -> list[str]:
    return [str(entry.get("os", "")) for entry in matrix_entries(job)]


def validate_matrix(job: dict) -> list[str]:
    """Three entries, one per supported operating system in order, each with
    a toolchain named in full, MSVC on Windows; the job takes TOOLCHAIN from
    the entry and no step changes a runner's default toolchain or host."""
    errors = []
    strategy = job.get("strategy", {})
    entries = matrix_entries(job)
    if (job.get("runs-on") != "${{ matrix.os }}" or strategy.get("fail-fast") != "false"
            or set(strategy.get("matrix", {})) != {"include"} or matrix_platforms(job) != PLATFORMS):
        errors.append("platform job must run all three supported operating systems")
    for entry in entries:
        toolchain = str(entry.get("toolchain", ""))
        if not PINNED_TOOLCHAIN.match(toolchain):
            errors.append(f"platform ({entry.get('os')}): toolchain is not pinned in full ({toolchain!r})")
        elif str(entry.get("os", "")).startswith("windows") and not toolchain.endswith("-pc-windows-msvc"):
            errors.append(f"platform ({entry.get('os')}): toolchain {toolchain} is not the MSVC toolchain named in full")
    if job.get("env", {}).get("TOOLCHAIN") != "${{ matrix.toolchain }}":
        errors.append("platform: TOOLCHAIN must come from the matrix entry")
    return errors


def validate_job_toolchain(label: str, job: dict, entry: dict | None, pin: str, workflow_env: dict,
                           floor: str | None, floor_steps: bool) -> list[str]:
    """What the job does at a toolchain other than its pin, read the way
    rustup decides it: the pin at or above rust-version; RUSTUP_TOOLCHAIN,
    when set, naming the pin; no step overriding a toolchain variable; every
    cargo call selecting the pin through the environment the step sees; a
    step installing the pin. With `floor_steps`, a Linux-only step whose
    build and test calls select the rust-version release is the floor
    coverage BAR-012 asks of a pin above the floor, and it must install that
    release too."""
    errors = []
    floor_pair = wt.release_pair(floor)
    pin_pair = wt.release_pair(pin)
    if floor_pair is not None and pin_pair is not None and pin_pair < floor_pair:
        errors.append(f"{label}: toolchain {pin} is below rust-version {floor}")
    job_env = wt.effective_env(workflow_env, job.get("env", {}), entry=entry)
    if "RUSTUP_TOOLCHAIN" in job_env and wt.resolve(job_env["RUSTUP_TOOLCHAIN"], job_env) != pin:
        errors.append(f"{label}: RUSTUP_TOOLCHAIN does not name the pin {pin}")
    os_label = wt.runner_os(str(entry.get("os", "")) if entry else str(job.get("runs-on", "")))
    installed: list[str | None] = []
    floor_release = None
    floor_seen = {command: False for command in FLOOR_COMMANDS}
    for step in job.get("steps", []):
        applies = wt.step_runs_on(step, os_label)
        if applies is False:
            continue
        step_env = step.get("env", {})
        for variable in wt.TOOLCHAIN_VARIABLES:
            if variable in step_env:
                errors.append(f"{label}: a step overrides {variable}")
        env = wt.effective_env(workflow_env, job.get("env", {}), step_env, entry=entry)
        run = wt.substitute(str(step.get("run", "")), entry)
        installed.extend(wt.installs(run, env))
        for call in wt.cargo_calls(run, env):
            if not call["explicit"]:
                errors.append(f"{label}: `{call['command']}` names no toolchain")
                continue
            selected = call["selected"]
            if selected == pin:
                continue
            at_floor = (floor_steps and floor_pair is not None and applies is True and selected is not None
                        and PINNED_TOOLCHAIN.match(selected) and wt.release_pair(selected) == floor_pair
                        and any(command in call["command"] for command in FLOOR_COMMANDS))
            if at_floor:
                floor_release = selected
                for command in FLOOR_COMMANDS:
                    if command in call["command"]:
                        floor_seen[command] = True
                continue
            errors.append(f"{label}: `{call['command']}` runs at {selected or 'the runner default'}, not the pin {pin}")
    if pin not in installed:
        errors.append(f"{label}: no step installs the pinned toolchain {pin}")
    if floor_steps and floor_pair is not None and pin_pair != floor_pair:
        if not all(floor_seen.values()):
            errors.append(f"{label}: the pin is above rust-version {floor} and no Linux step builds and tests at it")
        elif floor_release not in installed:
            errors.append(f"{label}: no step installs the rust-version toolchain {floor_release}")
    return errors


def validate_toolchains(workflow: dict, floor: str | None) -> list[str]:
    """The platform matrix per entry, and the advisories job through its
    job-level TOOLCHAIN, under the same pin rules."""
    errors = []
    jobs = workflow.get("jobs", {})
    workflow_env = workflow.get("env", {})
    platform = jobs.get("platform", {})
    for entry in matrix_entries(platform):
        pin = str(entry.get("toolchain", ""))
        if PINNED_TOOLCHAIN.match(pin):
            linux = str(entry.get("os", "")).startswith("ubuntu")
            errors.extend(validate_job_toolchain(f"platform ({entry.get('os')})", platform, entry, pin,
                                                 workflow_env, floor, floor_steps=linux))
    advisories = jobs.get("advisories")
    if isinstance(advisories, dict):
        env = wt.effective_env(workflow_env, advisories.get("env", {}))
        pin = wt.resolve(env.get("TOOLCHAIN"), env) if "TOOLCHAIN" in env else None
        if pin is None or not PINNED_TOOLCHAIN.match(pin):
            errors.append(f"advisories: toolchain is not pinned in full ({env.get('TOOLCHAIN')!r})")
        else:
            errors.extend(validate_job_toolchain("advisories", advisories, None, pin, workflow_env, floor,
                                                 floor_steps=False))
    return errors


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
        if DEFAULT_CHANGES.search(command):
            errors.append(f"{name}: a step changes the runner's default toolchain or host")
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


def validate_workflow(workflow: dict, floor: str | None = None) -> list[str]:
    """Every defect of the workflow; `floor` is Cargo.toml's rust-version,
    which the pins must not fall below (None skips that comparison)."""
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
    errors.extend(validate_matrix(platform))
    errors.extend(validate_toolchains(workflow, floor))
    if jobs.get("advisories", {}).get("runs-on") != "ubuntu-24.04":
        errors.append("advisory job must use its maintained Linux runner")
    expected = [f"platform ({os_name})" for os_name in PLATFORMS]
    for event in ("push", "pull_request", "workflow_dispatch"):
        if selected_jobs(workflow, event, "main") != expected:
            errors.append(f"{event} fixture did not select every supported platform")
    if selected_jobs(workflow, "schedule", "main") != ["advisories"]:
        errors.append("schedule fixture did not select advisory checks")
    return errors


def rust_version() -> str | None:
    """Cargo.toml's rust-version, the floor the pins must not fall below."""
    match = re.search(r'^rust-version\s*=\s*"([^"]+)"', MANIFEST.read_text(), re.M)
    return match.group(1) if match else None


def main() -> int:
    if sys.argv[1:] != ["RID-001"]:
        print("usage: check-pre-release-ci.py RID-001", file=sys.stderr)
        return 2
    unit = subprocess.run([sys.executable, "-B", "scripts/test-pre-release-ci.py"],
                          cwd=ROOT, timeout=60, check=False)
    if unit.returncode:
        return unit.returncode
    workflow, floor = {}, None
    try:
        # BaseLoader keeps GitHub's `on` key and boolean scalars as strings.
        # No YAML object constructors run on repository content.
        workflow = yaml.load(WORKFLOW.read_text(), Loader=yaml.BaseLoader)
        floor = rust_version()
        errors = validate_workflow(workflow, floor)
        if floor is None:
            errors.append("Cargo.toml declares no rust-version")
    except (OSError, yaml.YAMLError, TypeError, AttributeError) as error:
        errors = [f"cannot inspect {WORKFLOW.relative_to(ROOT)}: {error}"]
    if errors:
        print("RID-001 failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    for event in ("push", "pull_request", "schedule", "workflow_dispatch"):
        print(f"{event}: {', '.join(selected_jobs(workflow, event, 'main'))}")
    print(f"RID-001 workflow inspection passed (pins at or above rust-version {floor}); "
          "this is not evidence of native CI execution.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
