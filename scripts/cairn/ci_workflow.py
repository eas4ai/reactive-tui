#!/usr/bin/env python3
"""BAR-012 (ci-workflow): the push workflow is green at the commit under check.

Static part, read from .github/workflows/ci.yml, .gitattributes and
Cargo.toml. Where each job runs and when: a push or dispatch run goes to the
project's three machines, addressed by their labels (`self-hosted`,
`rust-ci`, an OS label and an architecture label), one Linux, one macOS and
one Windows; a pull request's run goes to GitHub's hosted runners and never
to a self-hosted label; the weekly advisories job runs on the Linux machine;
a job's `if` must be one the reader can evaluate for each event. What each
job runs at: its matrix pins a toolchain per entry, a stable release named
in full (1.95.0, or 1.95.0-x86_64-pc-windows-msvc) at or above Cargo.toml's
rust-version, the MSVC toolchain on Windows and never a GNU one; what a step
runs at is read the way rustup decides it, the `+<toolchain>` argument of
the call resolved through the environment the step sees (workflow, job and
step `env`, the matrix entry substituted), so every cargo call must name a
toolchain and that toolchain must be the job's pin; no step may override
TOOLCHAIN or RUSTUP_TOOLCHAIN; an expression the reader cannot resolve is a
violation. Each job that runs cargo installs its pin with `rustup toolchain
install`, and only when `rustup toolchain list` lacks it, as every `cargo
install` of a tool runs only after the tool's `--version` was found wanting
and every `uv pip install` tries `--offline` first, so a runner that holds
the pinned tools reaches no network for them and a network blip fails no
job; no step runs `rustup default` or `rustup set default-host`; every
platform job's steps run the five commands (build, test, fmt, clippy, cargo
deny) at the pin; the Linux job builds and tests at rust-version (its own
pin when that is the floor, otherwise a Linux step whose calls select the
floor release, installed too); the advisories job is held to the same pin
rules through its job-level TOOLCHAIN; a step installs the ConPTY runtime on
Windows; and tracked text files get LF line endings. A static violation is
the verdict, and no run is consulted.

Run part: the workflow's run for HEAD, read through gh. A run tests a
commit, so the working tree must match HEAD for this mechanism's inputs;
otherwise no verdict (unverified). Only a run that could have run the
platform matrix counts: the workflow skips that matrix on schedule, so a
scheduled run is no run here. A completed run decides: it passes when each
of the three platform jobs, named as the workflow names them, succeeded and
no job failed; otherwise it fails with its jobs. A run still going is waited
for. With no run, the commit is pushed as the temporary branch
ci/check-<sha>, the workflow is started on it (workflow_dispatch), waited
for, and the branch is deleted. The wait reads the run's jobs and the
repository's runners: a job queued because every runner that could take it
is offline (a machine asleep or off) is announced once per machine, on the
check's output and as a desktop notification, and its waiting time does not
count against the 55-minute ceiling, which counts only while the run is
executing; such a wait ends after a day with no verdict, and the machine's
return is announced too. A gh or git call that fails for want of the network
prints no result line, so Sudus records the run as unverified. The
mechanism's own tests (test_ci_workflow.py) run first; when they fail there
is no verdict.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time

import yaml

from _common import ROOT, finish

sys.path.insert(0, str(ROOT / "scripts"))
import workflow_toolchains as wt  # noqa: E402

REQUIREMENT = "BAR-012"
WORKFLOW = ROOT / ".github/workflows/ci.yml"
ATTRIBUTES = ROOT / ".gitattributes"
COMMANDS = (
    "cargo build --locked --all-targets",
    "cargo test --locked --no-fail-fast",
    "cargo fmt --all -- --check",
    "cargo clippy --locked --all-targets -- -D warnings",
    "cargo deny --locked check advisories bans licenses sources",
)
# The two the Linux job also runs at the floor when its pin is above it.
FLOOR_COMMANDS = COMMANDS[:2]
MACHINES = ("Linux", "macOS", "Windows")
TESTS = ROOT / "scripts/cairn/test_ci_workflow.py"
INPUTS = [".gitattributes", ".github", "Cargo.lock", "Cargo.toml", "benches", "bindings", "build.rs", "crates",
          "deny.toml", "examples", "include", "scripts", "src", "tests"]
CEILING = 55 * 60
DAY = 24 * 3600
POLL = 30
PINNED = wt.RELEASE
version_pair = wt.release_pair


class NoVerdict(Exception):
    """The run could not be read or started: no result line."""


def sh(args: list[str], timeout: int = 120) -> str:
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
    if result.returncode != 0:
        raise NoVerdict(f"`{' '.join(args[:3])}` exited {result.returncode}: {result.stderr.strip()[:200]}")
    return result.stdout


def rust_version() -> str:
    text = (ROOT / "Cargo.toml").read_text()
    match = re.search(r'^rust-version\s*=\s*"([^"]+)"', text, re.M)
    return match.group(1) if match else ""


def matrix_entries(job: dict) -> list[dict]:
    """A job's matrix as one dict per entry: `include` entries, or the `os`
    list without toolchains when the matrix has no include."""
    matrix = (job.get("strategy") or {}).get("matrix") or {}
    include = matrix.get("include")
    if isinstance(include, list):
        return [dict(entry) for entry in include if isinstance(entry, dict)]
    return [{"os": os} for os in (matrix.get("os") or [])]


def runs_of(job: dict) -> list[tuple[str, str]]:
    """Each step's `if` condition and `run` text."""
    return [(str(step.get("if", "")), str(step.get("run", ""))) for step in (job.get("steps") or [])]


def entry_label(name: str, entry: dict) -> str:
    """How a violation names a matrix entry's job: its `name` or `os` value."""
    return f"{entry.get('name') or entry.get('os') or name} job"


def job_name_for(name: str, job: dict, entry: dict) -> str | None:
    """The job's name on GitHub for the entry: its `name` template with the
    matrix substituted, or None when an expression remains."""
    template = job.get("name")
    if template is None:
        return f"{name} ({', '.join(str(v) for v in entry.values())})"
    text = wt.substitute(str(template), entry)
    return None if wt.EXPRESSION.search(text) else text


def pin_violations(label: str, pin: str, floor: tuple[int, int] | None, version: str, windows: bool) -> list[str]:
    """What a job's pin lacks: named in full, at or above the floor, never
    GNU, the MSVC triple on Windows."""
    if not PINNED.match(pin):
        return [f"the {label}'s toolchain is not pinned in full (found {pin!r})"]
    found = []
    pair = version_pair(pin)
    if floor is not None and pair is not None and pair < floor:
        found.append(f"the {label} pins {pin}, below Cargo.toml's rust-version {version}")
    if "gnu" in pin:
        found.append(f"the {label} names a GNU toolchain ({pin})")
    if windows and not pin.endswith("-pc-windows-msvc"):
        found.append(f"the {label}'s toolchain {pin} does not name the MSVC toolchain in full")
    return found


def step_violations(label: str, job: dict, entry: dict | None, workflow_env: dict, pin: str,
                    os_label: str | None, floor: tuple[int, int] | None, version: str,
                    required: tuple[str, ...], linux: bool) -> list[str]:
    """What the job's steps do at a toolchain other than its pin, read
    through the environment each step sees for this matrix entry: the
    toolchain variables on the job, overrides on a step, every cargo call's
    selection, the installs, the required commands at the pin, and the
    Linux floor coverage when the pin is above rust-version."""
    found = []
    job_env = wt.effective_env(workflow_env, job.get("env"), entry=entry)
    for variable in wt.TOOLCHAIN_VARIABLES:
        if variable not in job_env:
            continue
        value = wt.resolve(job_env[variable], job_env)
        if value is None:
            found.append(f"the {label}'s {variable} is an expression the gate cannot read ({job_env[variable]!r})")
        elif variable == "RUSTUP_TOOLCHAIN" and value != pin:
            found.append(f"the {label} sets RUSTUP_TOOLCHAIN to {value}, not its pin {pin}")
    installed: list[str | None] = []
    seen = {command: False for command in required}
    floor_seen = {command: False for command in FLOOR_COMMANDS}
    floor_release = None
    floor_wanted = linux and floor is not None and version_pair(pin) != floor
    for step in job.get("steps") or []:
        applies = wt.step_runs_on(step, os_label)
        if applies is False:
            continue
        step_env = step.get("env") or {}
        for variable in wt.TOOLCHAIN_VARIABLES:
            if variable in step_env:
                found.append(f"a step of the {label} overrides {variable}, so it may not run at the job's pin {pin}")
        env = wt.effective_env(workflow_env, job.get("env"), step_env, entry=entry)
        run = wt.substitute(str(step.get("run", "") or ""), entry)
        installed.extend(wt.installs(run, env))
        for call in wt.cargo_calls(run, env):
            if not call["explicit"]:
                found.append(f"a cargo call names no toolchain: `cargo {call['first']}`")
                continue
            selected = call["selected"]
            if selected is None:
                found.append(f"the {label} runs `{call['command']}` at a toolchain the gate cannot read ({call['first']})")
            elif selected == pin:
                for command in required:
                    if command in call["command"]:
                        seen[command] = True
            elif (floor_wanted and applies is True and PINNED.match(selected) and version_pair(selected) == floor
                  and any(command in call["command"] for command in FLOOR_COMMANDS)):
                for command in FLOOR_COMMANDS:
                    if command in call["command"]:
                        floor_seen[command] = True
                floor_release = selected
            else:
                found.append(f"the {label} runs `{call['command']}` at {selected}, not its pin {pin}")
    for command in required:
        if not seen[command]:
            found.append(f"the {label} does not run `{command}` at its pin {pin}")
    if None in installed:
        found.append(f"the {label} installs a toolchain the gate cannot read")
    names = [name for name in installed if name]
    if pin not in names:
        found.append(f"the {label} installs {', '.join(names)}, not its pin {pin}" if names
                     else f"the {label} installs no toolchain")
    if floor_wanted:
        if not all(floor_seen.values()):
            found.append(f"the Linux job does not build and test at rust-version {version}")
        elif floor_release not in names:
            found.append(f"the Linux job runs at {floor_release} but installs it nowhere")
    return found


def entry_violations(name: str, job: dict, entry: dict, workflow_env: dict, floor: tuple[int, int] | None,
                     version: str) -> list[str]:
    """One platform job of a matrix: its pin, then its steps against that
    pin, with the runner OS read from its labels."""
    label = entry_label(name, entry)
    labels = wt.labels_of(job, entry)
    os_label = wt.os_of_labels(labels)
    pin = str(entry.get("toolchain", "") or "")
    found = pin_violations(label, pin, floor, version, windows=os_label == "Windows")
    if found:
        return found
    return step_violations(label, job, entry, workflow_env, pin, os_label, floor, version,
                           required=COMMANDS, linux=os_label == "Linux")


def job_violations(name: str, job: dict, workflow_env: dict, floor: tuple[int, int] | None,
                   version: str) -> list[str]:
    """A job without a matrix (the weekly advisories job): when it runs
    cargo, its job-level TOOLCHAIN is its pin and the same rules hold."""
    label = f"{name} job"
    job_env = wt.effective_env(workflow_env, job.get("env"))
    runs_cargo = any(wt.cargo_calls(run, {}) for _, run in runs_of(job))
    if "TOOLCHAIN" not in job_env:
        return [f"the {label} runs cargo but sets no TOOLCHAIN pin"] if runs_cargo else []
    pin = wt.resolve(job_env["TOOLCHAIN"], job_env)
    if pin is None:
        return [f"the {label}'s TOOLCHAIN is an expression the gate cannot read ({job_env['TOOLCHAIN']!r})"]
    found = pin_violations(label, pin, floor, version, windows=False)
    if found:
        return found
    return step_violations(label, job, None, workflow_env, pin, wt.os_of_labels(wt.labels_of(job, None)),
                           floor, version, required=(), linux=False)


def routing_violations(jobs: dict) -> list[str]:
    """Where each event's jobs run: push and dispatch on the three machines
    and nowhere else, a pull request never on a self-hosted label, the
    schedule on the advisories job on the Linux machine; every condition
    and every `runs-on` readable."""
    found = []
    runs: dict[str, list[tuple[str, dict, dict | None]]] = {event: [] for event in wt.EVENTS}
    for name, job in jobs.items():
        if not isinstance(job, dict):
            continue
        condition = str(job.get("if", "") or "")
        entries = matrix_entries(job) or [None]
        for event in wt.EVENTS:
            decision = wt.runs_for_event(condition, event)
            if decision is None:
                found.append(f"the {name} job's condition is one the gate cannot read ({condition!r})")
                break
            if decision:
                runs[event].extend((name, job, entry) for entry in entries)
    for event in ("push", "workflow_dispatch"):
        machines_seen = set()
        for name, job, entry in runs[event]:
            labels = wt.labels_of(job, entry)
            label = entry_label(name, entry) if entry else f"{name} job"
            if labels is None:
                found.append(f"the {label}'s runs-on is an expression the gate cannot read")
            elif not wt.is_machine(labels):
                found.append(f"a {event} run sends the {label} to labels other than one of the three machines' ({', '.join(labels)})")
            else:
                machines_seen.add(wt.os_of_labels(labels))
        for machine in MACHINES:
            if machine not in machines_seen:
                found.append(f"no {event} job runs on the {machine} machine")
    for name, job, entry in runs["pull_request"]:
        labels = wt.labels_of(job, entry)
        label = entry_label(name, entry) if entry else f"{name} job"
        if labels is None:
            found.append(f"the {label}'s runs-on is an expression the gate cannot read")
        elif "self-hosted" in labels:
            found.append(f"a pull request's run sends the {label} to a self-hosted label")
    scheduled = runs["schedule"]
    if not scheduled:
        found.append("no job runs on the schedule")
    for name, job, entry in scheduled:
        labels = wt.labels_of(job, entry) or []
        label = entry_label(name, entry) if entry else f"{name} job"
        if entry is not None:
            found.append(f"the {label} runs on the schedule, which the platform matrix must not")
        elif not (wt.is_machine(labels) and wt.os_of_labels(labels) == "Linux"):
            found.append(f"the {label} does not run on the Linux machine ({', '.join(labels) or 'no labels'})")
    return found


def platform_job_names(workflow: dict) -> list[str]:
    """The names the push run's platform jobs carry on GitHub, one per
    matrix entry that runs on push."""
    names = []
    for name, job in (workflow.get("jobs") or {}).items():
        if not isinstance(job, dict) or wt.runs_for_event(str(job.get("if", "") or ""), "push") is not True:
            continue
        for entry in matrix_entries(job):
            resolved = job_name_for(name, job, entry)
            if resolved:
                names.append(resolved)
    return names


SEGMENTS = re.compile(r"(\|\||&&|;|\||\n)")
CARGO_TOOL_INSTALL = re.compile(r"^cargo\s+(?:\S+\s+)?install\b(?=.*\bcargo-([a-z-]+)\b)")
UV_INSTALL = re.compile(r"^uv\s+pip\s+install\b")
OFFLINE = re.compile(r"^uv\s+pip\s+install\b.*--offline\b")


def commands(run: str) -> list[tuple[str, str]]:
    """A step's shell text as (joiner, command) pairs in order: the commands
    split on newlines, `;`, `&&`, `||` and `|`, a backslash-newline joined to
    its line, each with the joiner that precedes it."""
    tokens = SEGMENTS.split(re.sub(r"\\\n", " ", run))
    result = []
    joiner = "\n"
    for index, token in enumerate(tokens):
        if index % 2:
            joiner = token
            continue
        command = token.strip()
        if command:
            result.append((joiner, command))
    return result


def guarded(parts: list[tuple[str, str]], index: int, check: re.Pattern) -> bool:
    """Whether the command at `index` runs only after `check` found its tool
    wanting: it is the right side of `check || install`, the check being the
    pipeline before the `||`, or it stands inside an `if ...; then ... fi`
    whose condition runs the check."""
    joiner, _ = parts[index]
    if joiner == "||":
        at = index - 1
        while at >= 0:
            if check.search(parts[at][1]):
                return True
            if parts[at][0] != "|":
                break
            at -= 1
    opened = None
    for at in range(index - 1, -1, -1):
        command = parts[at][1]
        if command == "fi":
            return False
        if command == "then":
            opened = at
            continue
        if opened is not None and re.match(r"^(if|elif)\b", command):
            return any(check.search(parts[k][1]) for k in range(at, opened))
    return False


def guard_violations(label: str, run: str) -> list[str]:
    """The installs a step runs without first finding the tool missing: a
    `rustup toolchain install` not guarded by `rustup toolchain list`, a
    `cargo install cargo-<tool>` not guarded by `cargo <tool> --version`, or
    a `uv pip install` not guarded by an `--offline` try. A check that only
    appears somewhere in the step does not guard the install; it has to be
    the condition the install runs under. Such a step reaches the network on
    every job, and a blip on the runner fails the job."""
    found = []
    parts = commands(run)
    for index, (_, command) in enumerate(parts):
        if wt.INSTALL.search(command) and not guarded(parts, index, re.compile(r"rustup\s+toolchain\s+list\b")):
            found.append(f"the {label} installs its toolchain without checking `rustup toolchain list` first, "
                         "so every job reaches the network for it")
        match = CARGO_TOOL_INSTALL.match(command)
        if match:
            tool = match.group(1)
            if not guarded(parts, index, re.compile(rf"\b{re.escape(tool)}\s+--version\b")):
                found.append(f"the {label} installs cargo-{tool} without checking `cargo {tool} --version` first, "
                             "so every job reaches the network for it")
        if UV_INSTALL.match(command) and not OFFLINE.match(command) and not guarded(parts, index, OFFLINE):
            found.append(f"the {label} installs its Python tools without an offline try first, "
                         "so every job reaches the network for them")
    return found

def static_violations(workflow: dict | None = None, attributes: str | None = None,
                      version: str | None = None) -> list[str]:
    """What the workflow file and the attributes lack, as the falsifier lists it.
    The arguments let the mechanism's own tests pass a workflow in memory."""
    found = []
    if workflow is None:
        if not WORKFLOW.exists():
            return [f"{WORKFLOW.relative_to(ROOT)} is missing"]
        workflow = yaml.safe_load(WORKFLOW.read_text())
    if version is None:
        version = rust_version()
    floor = version_pair(version)
    if not version or floor is None:
        found.append("Cargo.toml declares no rust-version")
    jobs = workflow.get("jobs") or {}
    workflow_env = workflow.get("env") or {}
    found.extend(routing_violations(jobs))
    all_runs = [run for job in jobs.values() if isinstance(job, dict) for _, run in runs_of(job)]
    joined_all = "\n".join(all_runs)
    if re.search(r"rustup\s+default\b", joined_all):
        found.append("a step runs `rustup default`, which changes the runner's default toolchain")
    if re.search(r"rustup\s+set\s+default-host\b", joined_all):
        found.append("a step runs `rustup set default-host`, which changes the runner's default host")
    if re.search(r"windows-gnu", joined_all):
        found.append("a step names a GNU Windows toolchain")
    conpty_seen = False
    for name, job in jobs.items():
        if not isinstance(job, dict):
            continue
        for _, run in runs_of(job):
            found.extend(guard_violations(f"{name} job", run))
        entries = matrix_entries(job)
        if entries:
            for entry in entries:
                found.extend(entry_violations(name, job, entry, workflow_env, floor, version))
                if job_name_for(name, job, entry) is None:
                    found.append(f"the {name} job's name is an expression the gate cannot read")
            conpty = [step for step in (job.get("steps") or []) if "install-conpty-runtime.py" in str(step.get("run", ""))]
            if any("Windows" in str(step.get("if", "")) for step in conpty):
                conpty_seen = True
            else:
                found.append(f"no Windows step of the {name} job installs the ConPTY runtime")
        else:
            found.extend(job_violations(name, job, workflow_env, floor, version))
    if not conpty_seen and not any("ConPTY" in v for v in found):
        found.append("no Windows step installs the ConPTY runtime")
    if attributes is None:
        if not ATTRIBUTES.exists():
            found.append(".gitattributes is missing")
            return found
        attributes = ATTRIBUTES.read_text()
    if not re.search(r"^\*\s+text=auto\s+eol=lf\b", attributes, re.M):
        found.append(".gitattributes does not give tracked text files LF line endings (`* text=auto eol=lf`)")
    return found


def head() -> str:
    return sh(["git", "rev-parse", "HEAD"]).strip()


def dirty_inputs() -> list[str]:
    out = sh(["git", "status", "--porcelain", "--", *[i for i in INPUTS if (ROOT / i).exists()]])
    return [line[3:] for line in out.splitlines() if line.strip()]


def runs_for(sha: str) -> list[dict]:
    out = sh(["gh", "run", "list", "--workflow", "ci.yml", "--commit", sha, "--limit", "20",
              "--json", "databaseId,status,conclusion,event,createdAt,url"])
    return json.loads(out or "[]")


def runs_on_branch(branch: str) -> list[dict]:
    out = sh(["gh", "run", "list", "--workflow", "ci.yml", "--branch", branch, "--limit", "5",
              "--json", "databaseId,status,conclusion,event,createdAt,url"])
    return json.loads(out or "[]")


# The events on which the workflow runs the platform matrix on the machines.
# A pull request runs the hosted matrix instead, with the machines' jobs
# skipped, and a schedule runs only the advisories job.
LOCAL_EVENTS = ("push", "workflow_dispatch")


def platform_runs(runs: list[dict]) -> list[dict]:
    """The runs that could have run the platform matrix on the machines: a
    push or dispatch run that was not cancelled. A pull request's or a
    schedule's run never stands for it, however new or green."""
    return [r for r in runs if r["conclusion"] != "cancelled" and r["event"] in LOCAL_EVENTS]


def missing_platform_jobs(jobs: list[dict], expected: list[str]) -> list[str]:
    """The expected platform jobs without a successful instance among `jobs`."""
    return [name for name in expected
            if not any(str(j["name"]) == name and j["conclusion"] == "success" for j in jobs)]


def jobs_of(run_id: int) -> list[dict]:
    out = sh(["gh", "run", "view", str(run_id), "--json", "jobs"])
    return json.loads(out)["jobs"]


def repo_slug() -> str:
    return sh(["gh", "repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]).strip()


def runners_of(slug: str) -> list[dict]:
    """The repository's registered runners: name, status (online or
    offline), busy, and their label names."""
    out = sh(["gh", "api", f"repos/{slug}/actions/runners?per_page=100"])
    data = json.loads(out)
    return [{"name": r["name"], "status": r["status"], "busy": r.get("busy", False),
             "labels": [label["name"] for label in r.get("labels", [])]} for r in data.get("runners", [])]


def queued_jobs_of(slug: str, run_id: int) -> list[dict]:
    """The run's jobs with their status and the labels each asked for."""
    out = sh(["gh", "api", f"repos/{slug}/actions/runs/{run_id}/jobs?per_page=100"])
    return [{"name": job["name"], "status": job["status"], "labels": list(job.get("labels") or [])}
            for job in json.loads(out).get("jobs", [])]


def offline_waits(jobs: list[dict], runners: list[dict]) -> dict[str, str]:
    """Job name to the runner it waits for: the queued jobs whose labels only
    registered runners that are offline can take. A job with no registered
    runner at all is not an offline wait; GitHub fails it on its own."""
    waits = {}
    for job in jobs:
        if job.get("status") != "queued":
            continue
        wanted = set(job.get("labels") or [])
        if "self-hosted" not in wanted:
            continue
        matching = [r for r in runners if wanted <= set(r.get("labels", []))]
        if matching and all(r.get("status") != "online" for r in matching):
            waits[job["name"]] = ", ".join(sorted(r["name"] for r in matching))
    return waits


class Waiter:
    """The accounting BAR-012 asks of a wait: the ceiling on run time counts
    only the seconds a job was executing; a job waiting on an offline
    machine is announced once per machine, and that wait, like any wait
    with no job executing (a queued job whose online runner is busy with
    other work, or a run GitHub has given no job yet), ends after a day."""

    def __init__(self, ceiling: float = CEILING, day: float = DAY):
        self.ceiling = ceiling
        self.day = day
        self.executing = 0.0
        self.offline = 0.0
        self.queued = 0.0
        self.announced: set[str] = set()
        self.waiting: set[str] = set()

    def step(self, jobs: list[dict], runners: list[dict], elapsed: float) -> list[str]:
        """Account `elapsed` seconds since the last look and return the
        lines to tell the developer: a machine newly found offline, or back."""
        waits = offline_waits(jobs, runners)
        running = any(job.get("status") == "in_progress" for job in jobs)
        if running:
            self.executing += elapsed
        if waits:
            self.offline += elapsed
        elif not running:
            self.queued += elapsed
        lines = []
        for job, runner in waits.items():
            if runner not in self.announced:
                lines.append(f"waiting for the {runner} runner, which is offline, to run {job}; "
                             "this wait does not count against the run-time ceiling and ends after a day")
                self.announced.add(runner)
        # Each machine is reported back on its own, whether or not another
        # is still offline.
        current = set(waits.values())
        for runner in sorted(self.waiting - current):
            lines.append(f"the {runner} runner is back online; the run goes on")
        self.waiting = current
        return lines

    def over_ceiling(self) -> bool:
        return self.executing > self.ceiling

    def over_day(self) -> bool:
        return self.offline + self.queued > self.day


def tell(line: str) -> None:
    """Print the line for the record and, on the workstation, raise a desktop
    notification; the notification is best effort."""
    print(line, flush=True)
    try:
        subprocess.run(["notify-send", "--app-name=sudus", "--urgency=critical", "CI runner", line],
                       capture_output=True, text=True, timeout=10, check=False)
    except (OSError, subprocess.TimeoutExpired):
        pass


def wait_for(run_id: int, slug: str, waiter: Waiter) -> dict:
    """Poll the run until it completes, the time a job was executing passes
    the ceiling, or the time none was passes a day (no verdict)."""
    last = time.monotonic()
    while True:
        out = sh(["gh", "run", "view", str(run_id), "--json", "status,conclusion,url"])
        run = json.loads(out)
        if run["status"] == "completed":
            return run
        now = time.monotonic()
        jobs = queued_jobs_of(slug, run_id)
        runners = runners_of(slug) if any(job["status"] == "queued" for job in jobs) else []
        for line in waiter.step(jobs, runners, now - last):
            tell(line)
        last = now
        if waiter.over_ceiling():
            return run
        if waiter.over_day():
            who = (f"an offline runner ({', '.join(sorted(waiter.announced))})" if waiter.announced
                   else "a runner to take a queued job")
            raise NoVerdict(f"waited a day for {who}; run {run['url']}")
        time.sleep(POLL)


def dispatch(sha: str, slug: str, waiter: Waiter) -> dict:
    """Push the commit as a temporary branch, start the workflow on it, wait,
    and delete the branch whatever happened."""
    branch = f"ci/check-{sha[:12]}"
    sh(["git", "push", "-q", "origin", f"{sha}:refs/heads/{branch}"], timeout=300)
    try:
        sh(["gh", "workflow", "run", "ci.yml", "--ref", branch])
        started = time.monotonic()
        run = None
        while run is None:
            candidates = [r for r in runs_on_branch(branch) if r["event"] == "workflow_dispatch"]
            if candidates:
                run = max(candidates, key=lambda r: r["createdAt"])
            elif time.monotonic() - started > 180:
                raise NoVerdict(f"the workflow run on {branch} did not appear within 3 minutes")
            else:
                time.sleep(5)
        print(f"started run {run['databaseId']} on {branch}: {run['url']}", flush=True)
        final = wait_for(run["databaseId"], slug, waiter)
        final["databaseId"] = run["databaseId"]
        return final
    finally:
        subprocess.run(["git", "push", "-q", "origin", "--delete", branch], cwd=ROOT, capture_output=True, text=True, timeout=300)


def verdict(run: dict, jobs: list[dict], expected: list[str]) -> tuple[bool, str]:
    """Pass when the run completed, each expected platform job succeeded
    and no job failed. A skipped platform job is a job that never started,
    which the falsifier names."""
    run_id = run["databaseId"]
    if run["status"] != "completed":
        return False, f"run {run_id} did not finish within {CEILING // 60} minutes of execution ({run['url']})"
    for job in jobs:
        print(f"  {job['name']}: {job['conclusion'] or job['status']}", flush=True)
    bad = [f"{j['name']} {j['conclusion'] or j['status']}" for j in jobs if j["conclusion"] not in ("success", "skipped")]
    bad += [f"no successful {name} job" for name in missing_platform_jobs(jobs, expected)]
    if run["conclusion"] == "success" and not bad:
        return True, f"run {run_id} green on every job ({run['url']})"
    return False, f"run {run_id} {run['conclusion']}: {'; '.join(bad) or 'no job ran'} ({run['url']})"


def gate() -> int:
    own = subprocess.run([sys.executable, "-B", str(TESTS)], cwd=ROOT, capture_output=True, text=True, timeout=120)
    if own.returncode != 0:
        print(f"{REQUIREMENT} unverified: the mechanism's own tests fail:\n{own.stderr.strip()[-1500:]}")
        return 1
    violations = static_violations()
    for v in violations:
        print(f"static: {v}", flush=True)
    if violations:
        return finish({REQUIREMENT: (False, "; ".join(violations))})
    workflow = yaml.safe_load(WORKFLOW.read_text())
    expected = platform_job_names(workflow)
    print("static: push and dispatch runs go to the three machines by their labels, pull requests to hosted "
          "runners, the advisories job to the Linux machine; the five commands run on each platform job with "
          "every cargo call resolving to the toolchain pinned per job at or above Cargo.toml's rust-version, "
          "MSVC on Windows, each job installing its pin and changing no default, the Linux job at the floor; "
          "the ConPTY runtime is installed on Windows and .gitattributes pins LF; "
          f"platform jobs {', '.join(expected)}", flush=True)
    try:
        sha = head()
        dirty = dirty_inputs()
        if dirty:
            print(f"{REQUIREMENT} unverified: a run tests a commit, and these inputs differ from HEAD: {', '.join(dirty[:6])}")
            return 1
        slug = repo_slug()
        waiter = Waiter()
        runs = platform_runs(runs_for(sha))
        completed = [r for r in runs if r["status"] == "completed"]
        going = [r for r in runs if r["status"] != "completed"]
        if completed:
            run = max(completed, key=lambda r: r["createdAt"])
            print(f"run {run['databaseId']} for {sha[:12]} ({run['event']}): {run['url']}", flush=True)
        elif going:
            run = max(going, key=lambda r: r["createdAt"])
            print(f"waiting for run {run['databaseId']} for {sha[:12]} ({run['event']}): {run['url']}", flush=True)
            run = {**wait_for(run["databaseId"], slug, waiter), "databaseId": run["databaseId"]}
        else:
            print(f"no run for {sha[:12]}: starting one", flush=True)
            run = dispatch(sha, slug, waiter)
        ok, why = verdict(run, jobs_of(run["databaseId"]), expected)
    except (NoVerdict, subprocess.TimeoutExpired, json.JSONDecodeError, KeyError) as error:
        print(f"{REQUIREMENT} unverified: {error}")
        return 1
    return finish({REQUIREMENT: (ok, why)})


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit("usage: ci_workflow.py")
    sys.exit(gate())
