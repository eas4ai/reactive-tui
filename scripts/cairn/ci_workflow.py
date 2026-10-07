#!/usr/bin/env python3
"""BAR-012 (ci-workflow): the push workflow is green at the commit under check.

Static part, read from .github/workflows/ci.yml, .gitattributes and
Cargo.toml: the platform job runs on an Ubuntu, a macOS and a Windows runner;
its matrix pins a toolchain per job, a stable release named in full
(1.95.0, or 1.95.0-x86_64-pc-windows-msvc) at or above Cargo.toml's
rust-version, the MSVC toolchain on Windows and never a GNU one; every cargo
call names a toolchain and no step runs `rustup default` or `rustup set
default-host`; its steps run the five commands (build, test, fmt, clippy,
cargo deny); the Linux job builds and tests at rust-version (its own pin
when that is the floor, or a floor step); a step installs the ConPTY runtime
on Windows; and tracked text files get LF line endings. A static violation
is the verdict, and no run is consulted.

Run part: the workflow's run for HEAD, read through gh. A run tests a
commit, so the working tree must match HEAD for this mechanism's inputs;
otherwise no verdict (unverified). Only a run that could have run the
platform matrix counts: the workflow skips that matrix on schedule, so a
scheduled run is no run here. A completed run decides: it passes when each
of the three platform jobs succeeded and no job failed; otherwise it fails
with its jobs. A run still going is waited for. With no run, the commit is
pushed as the temporary branch ci/check-<sha>, the workflow is started on it
(workflow_dispatch), waited for up to 55 minutes, and the branch is deleted.
A gh or git call that fails for want of the network prints no result line,
so Sudus records the run as unverified. The mechanism's own tests
(test_ci_workflow.py) run first; when they fail there is no verdict.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time

import yaml

from _common import ROOT, finish

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
RUNNERS = ("ubuntu-", "macos-", "windows-")
# The job of each runner, as the workflow names it: `platform (<runner>)`.
PLATFORM_JOBS = tuple(f"platform ({prefix}" for prefix in RUNNERS)
TESTS = ROOT / "scripts/cairn/test_ci_workflow.py"
INPUTS = [".gitattributes", ".github", "Cargo.lock", "Cargo.toml", "benches", "bindings", "build.rs", "crates",
          "deny.toml", "examples", "include", "scripts", "src", "tests"]
CEILING = 55 * 60
POLL = 30


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


PINNED = re.compile(r"^(\d+)\.(\d+)\.(\d+)(-[A-Za-z0-9_.-]+)?$")
# Steps spell a pinned call as `cargo +1.95.0 ...` or `cargo "+$TOOLCHAIN" ...`.
CARGO_CALL = re.compile(r"(?<![\w/.-])cargo\s+(\S+)")
PIN_ARG = re.compile(r'^"?\+')


def matrix_entries(platform: dict) -> list[dict]:
    """The platform job's matrix as one dict per job: `include` entries, or
    the `os` list without toolchains when the matrix has no include."""
    matrix = (platform.get("strategy") or {}).get("matrix") or {}
    include = matrix.get("include")
    if isinstance(include, list):
        return [dict(entry) for entry in include if isinstance(entry, dict)]
    return [{"os": os} for os in (matrix.get("os") or [])]


def version_pair(text: str) -> tuple[int, int] | None:
    match = re.match(r"^(\d+)\.(\d+)", text or "")
    return (int(match.group(1)), int(match.group(2))) if match else None


def runs_of(job: dict) -> list[tuple[str, str]]:
    """Each step's `if` condition and `run` text."""
    return [(str(step.get("if", "")), str(step.get("run", ""))) for step in (job.get("steps") or [])]


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
    platform = jobs.get("platform") or {}
    entries = matrix_entries(platform)
    for prefix in RUNNERS:
        if not any(str(entry.get("os", "")).startswith(prefix) for entry in entries):
            found.append(f"the platform job has no {prefix[:-1]} runner")
    linux_pin = None
    for entry in entries:
        os_name = str(entry.get("os", ""))
        toolchain = str(entry.get("toolchain", "") or "")
        if not PINNED.match(toolchain):
            found.append(f"the {os_name} job's toolchain is not pinned in full (found {toolchain!r})")
            continue
        pair = version_pair(toolchain)
        if floor is not None and pair is not None and pair < floor:
            found.append(f"the {os_name} job pins {toolchain}, below Cargo.toml's rust-version {version}")
        if "gnu" in toolchain:
            found.append(f"the {os_name} job names a GNU toolchain ({toolchain})")
        if os_name.startswith("windows-") and not toolchain.endswith("-pc-windows-msvc"):
            found.append(f"the Windows job's toolchain {toolchain} does not name the MSVC toolchain in full")
        if os_name.startswith("ubuntu-"):
            linux_pin = pair
    all_runs = [run for job in jobs.values() if isinstance(job, dict) for _, run in runs_of(job)]
    joined_all = "\n".join(all_runs)
    if re.search(r"rustup\s+default\b", joined_all):
        found.append("a step runs `rustup default`, which changes the runner's default toolchain")
    if re.search(r"rustup\s+set\s+default-host\b", joined_all):
        found.append("a step runs `rustup set default-host`, which changes the runner's default host")
    if re.search(r"windows-gnu", joined_all):
        found.append("a step names a GNU Windows toolchain")
    for match in CARGO_CALL.finditer(joined_all):
        if not PIN_ARG.match(match.group(1)):
            found.append(f"a cargo call names no toolchain: `cargo {match.group(1)}`")
            break
    platform_runs_text = "\n".join(run for _, run in runs_of(platform))
    normalized = re.sub(r'cargo\s+"?\+\S+"?\s+', "cargo ", platform_runs_text)
    for command in COMMANDS:
        if command not in normalized:
            found.append(f"the workflow does not run `{command}`")
    if not re.search(r"rustup\s+toolchain\s+install\b", platform_runs_text):
        found.append("the platform job installs no toolchain")
    if floor is not None and linux_pin is not None and linux_pin != floor:
        floor_steps = "\n".join(run for cond, run in runs_of(platform) if "Linux" in cond)
        floor_text = re.sub(r"cargo\s+\+" + re.escape(version) + r"(\.\d+)?\s+", "cargo ", floor_steps)
        if "cargo build --locked --all-targets" not in floor_text or "cargo test --locked --no-fail-fast" not in floor_text:
            found.append(f"the Linux job does not build and test at rust-version {version}")
    conpty = [step for step in (platform.get("steps") or []) if "install-conpty-runtime.py" in str(step.get("run", ""))]
    if not any("Windows" in str(step.get("if", "")) for step in conpty):
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


def platform_runs(runs: list[dict]) -> list[dict]:
    """The runs that could have run the platform matrix: not cancelled, and
    not scheduled, since the workflow skips the matrix on schedule and runs
    only the advisories job then."""
    return [r for r in runs if r["conclusion"] != "cancelled" and r["event"] != "schedule"]


def missing_platform_jobs(jobs: list[dict]) -> list[str]:
    """The platform jobs without a successful instance among `jobs`."""
    return [prefix for prefix in PLATFORM_JOBS
            if not any(str(j["name"]).startswith(prefix) and j["conclusion"] == "success" for j in jobs)]


def jobs_of(run_id: int) -> list[dict]:
    out = sh(["gh", "run", "view", str(run_id), "--json", "jobs"])
    return json.loads(out)["jobs"]


def wait_for(run_id: int, deadline: float) -> dict:
    while True:
        out = sh(["gh", "run", "view", str(run_id), "--json", "status,conclusion,url"])
        run = json.loads(out)
        if run["status"] == "completed":
            return run
        if time.monotonic() > deadline:
            return run
        time.sleep(POLL)


def dispatch(sha: str, deadline: float) -> dict:
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
        final = wait_for(run["databaseId"], deadline)
        final["databaseId"] = run["databaseId"]
        return final
    finally:
        subprocess.run(["git", "push", "-q", "origin", "--delete", branch], cwd=ROOT, capture_output=True, text=True, timeout=300)


def verdict(run: dict, jobs: list[dict]) -> tuple[bool, str]:
    """Pass when the run completed, each of the three platform jobs succeeded
    and no job failed. A skipped platform job is a job that never started,
    which the falsifier names."""
    run_id = run["databaseId"]
    if run["status"] != "completed":
        return False, f"run {run_id} did not finish within {CEILING // 60} minutes ({run['url']})"
    for job in jobs:
        print(f"  {job['name']}: {job['conclusion'] or job['status']}", flush=True)
    bad = [f"{j['name']} {j['conclusion'] or j['status']}" for j in jobs if j["conclusion"] not in ("success", "skipped")]
    bad += [f"no successful {prefix}...) job" for prefix in missing_platform_jobs(jobs)]
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
    print("static: the workflow runs the five commands on the three runners at a toolchain pinned per job "
          "at or above Cargo.toml's rust-version, MSVC on Windows, changing no default, the Linux job at the floor, "
          "installs the ConPTY runtime on Windows, and .gitattributes pins LF", flush=True)
    try:
        sha = head()
        dirty = dirty_inputs()
        if dirty:
            print(f"{REQUIREMENT} unverified: a run tests a commit, and these inputs differ from HEAD: {', '.join(dirty[:6])}")
            return 1
        deadline = time.monotonic() + CEILING
        runs = platform_runs(runs_for(sha))
        completed = [r for r in runs if r["status"] == "completed"]
        going = [r for r in runs if r["status"] != "completed"]
        if completed:
            run = max(completed, key=lambda r: r["createdAt"])
            print(f"run {run['databaseId']} for {sha[:12]} ({run['event']}): {run['url']}", flush=True)
        elif going:
            run = max(going, key=lambda r: r["createdAt"])
            print(f"waiting for run {run['databaseId']} for {sha[:12]} ({run['event']}): {run['url']}", flush=True)
            run = {**wait_for(run["databaseId"], deadline), "databaseId": run["databaseId"]}
        else:
            print(f"no run for {sha[:12]}: starting one", flush=True)
            run = dispatch(sha, deadline)
        ok, why = verdict(run, jobs_of(run["databaseId"]))
    except (NoVerdict, subprocess.TimeoutExpired, json.JSONDecodeError, KeyError) as error:
        print(f"{REQUIREMENT} unverified: {error}")
        return 1
    return finish({REQUIREMENT: (ok, why)})


if __name__ == "__main__":
    if len(sys.argv) != 1:
        sys.exit("usage: ci_workflow.py")
    sys.exit(gate())
