#!/usr/bin/env python3
"""BAR-009: every reactive-tui target builds with no warning on the macOS and
Windows test hosts.

Snapshots the working tree as a commit object (through a scratch index, so
the repository's index and branches are untouched), sends it to each test
host as a git bundle over SSH, checks it out in the host's clone, runs
`cargo build --locked -p reactive-tui --all-targets --message-format short`
and fails on a build error or any warning line.

The hosts are read from ~/.config/reactive-tui/test-hosts.json, which stays
outside the repository:

    {"macos": {"ssh": "user@host", "dir": "rtui-verify", "cargo": "~/.cargo/bin/cargo"},
     "windows": {"ssh": "user@host", "dir": "C:\\\\Users\\\\user\\\\rtui-verify",
                 "cargo": "cargo", "shell": "powershell"}}

A host that cannot be reached, or a missing file, reaches no verdict: the
script prints no result line, so Sudus records the run as unverified.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from _common import ROOT, report

CONFIG = Path.home() / ".config" / "reactive-tui" / "test-hosts.json"
HOSTS = ("macos", "windows")
REF = "refs/host-builds/snapshot"
BUILD = "build --locked -p reactive-tui --all-targets --message-format short"
WARNING = re.compile(r"(^|: )warning(\[\w+\])?: ")
SSH = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=10"]


class Unreachable(Exception):
    """A host that could not be reached or prepared: no verdict."""


def git(*args: str, env: dict | None = None) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                            env={**os.environ, **(env or {})}, check=True)
    return result.stdout.strip()


def snapshot(scratch: Path) -> tuple[str, Path]:
    """The working tree as a commit, tracked and untracked files alike, less
    what .gitignore excludes, and a bundle that carries it."""
    index = {"GIT_INDEX_FILE": str(scratch / "index")}
    git("read-tree", "HEAD", env=index)
    git("add", "-A", env=index)
    tree = git("write-tree", env=index)
    commit = git("commit-tree", tree, "-p", "HEAD", "-m", "host-builds snapshot")
    bundle = scratch / "snapshot.bundle"
    git("update-ref", REF, commit)
    try:
        git("bundle", "create", str(bundle), REF)
    finally:
        git("update-ref", "-d", REF)
    return commit, bundle


def remote(host: dict, command: str, timeout: int) -> subprocess.CompletedProcess:
    return subprocess.run(["ssh", *SSH, host["ssh"], command], capture_output=True, text=True,
                          timeout=timeout, encoding="utf-8", errors="replace")


def build_on(name: str, host: dict, commit: str, bundle: Path) -> tuple[bool, str]:
    powershell = host.get("shell") == "powershell"
    # scp puts the bundle in the host user's home directory.
    sent = subprocess.run(["scp", "-q", *SSH, str(bundle), f"{host['ssh']}:rtui-host-builds.bundle"],
                          capture_output=True, text=True, timeout=600)
    if sent.returncode != 0:
        raise Unreachable(f"{name}: scp failed: {sent.stderr.strip()[:200]}")
    cargo = host.get("cargo", "cargo")
    if powershell:
        command = (f"cd {host['dir']}; git fetch -q $HOME\\rtui-host-builds.bundle {REF}:{REF}; "
                   f"if ($LASTEXITCODE -ne 0) {{ 'PREPARE-FAILED'; exit 1 }}; "
                   f"git checkout -q -f --detach {commit}; git clean -fdq -e target; "
                   f"{cargo} {BUILD} 2>&1 | ForEach-Object {{ \"$_\" }}; \"EXIT=$LASTEXITCODE\"")
    else:
        command = (f"cd {host['dir']} && git fetch -q ~/rtui-host-builds.bundle {REF}:{REF} "
                   f"&& git checkout -q -f --detach {commit} && git clean -fdq -e target "
                   f"|| {{ echo PREPARE-FAILED; exit 1; }}; {cargo} {BUILD} 2>&1; echo EXIT=$?")
    result = remote(host, command, timeout=3600)
    output = result.stdout.replace("\r", "")
    print(f"--- {name} ---")
    print(output, end="" if output.endswith("\n") else "\n")
    if "PREPARE-FAILED" in output or "EXIT=" not in output:
        raise Unreachable(f"{name}: could not check out the snapshot: {result.stderr.strip()[:200]}")
    code = output.rsplit("EXIT=", 1)[1].split()[0]
    warnings = [line.strip() for line in output.splitlines() if WARNING.search(line)]
    if code != "0":
        errors = [line.strip() for line in output.splitlines() if line.startswith("error")]
        return False, f"{name}: the build failed (exit {code}): {'; '.join(errors[:3])}"
    if warnings:
        return False, f"{name}: {len(warnings)} warning lines, for example {'; '.join(warnings[:3])}"
    return True, f"{name}: no warnings"


def main() -> int:
    try:
        hosts = json.loads(CONFIG.read_text())
    except (OSError, ValueError) as error:
        print(f"BAR-009 unverified: no readable test host file at {CONFIG} ({error})")
        return 1
    missing = [name for name in HOSTS if name not in hosts]
    if missing:
        print(f"BAR-009 unverified: {CONFIG} names no {' or '.join(missing)} host")
        return 1
    with tempfile.TemporaryDirectory(prefix="host-builds-") as scratch:
        commit, bundle = snapshot(Path(scratch))
        results = []
        try:
            for name in HOSTS:
                results.append(build_on(name, hosts[name], commit, bundle))
        except (Unreachable, subprocess.TimeoutExpired) as error:
            print(f"BAR-009 unverified: {error}")
            return 1
    ok = all(passed for passed, _ in results)
    report("BAR-009", ok, "; ".join(why for _, why in results))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
