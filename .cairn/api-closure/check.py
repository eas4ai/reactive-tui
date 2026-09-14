#!/usr/bin/env python3
"""Independently verify API-020 audit reconciliation and process cleanup."""

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[2]
CAPTURE = ROOT / "tests/api_widget_behavior/image_host_capture.py"
CONTROL = ROOT / ".cairn/reviews/api-020-image-capture-timeout/finding.json"
REVIEW = ROOT / ".cairn/reviews/rust-api-remediation.md"
TODO = ROOT / ".cairn/reviews/api-018-through-020-todo.md"
REVIEW_HEADING = "## API-020 final commitment review"
CAPTURE_PYTHON = "/usr/bin/python3"


def process_table():
    processes = {}
    for directory in Path("/proc").iterdir():
        if not directory.name.isdigit():
            continue
        try:
            stat = (directory / "stat").read_text()
            fields = stat[stat.rfind(")") + 2:].split()
            processes[int(directory.name)] = {
                "state": fields[0],
                "ppid": int(fields[1]),
                "pgid": int(fields[2]),
                "sid": int(fields[3]),
                "started": fields[19],
            }
        except (FileNotFoundError, PermissionError, ProcessLookupError, ValueError):
            continue
    return processes


def output_holders(output):
    prefix = str(output.resolve()) + os.sep
    holders = {}
    for pid, details in process_table().items():
        try:
            descriptors = (Path("/proc") / str(pid) / "fd").iterdir()
            if any(os.readlink(descriptor).startswith(prefix) for descriptor in descriptors):
                holders[pid] = details
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
    return holders


def group_members(groups):
    return {pid: details for pid, details in process_table().items()
            if details["pgid"] in groups and details["state"] != "Z"}


def wait_for_cleanup(groups, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        remaining = group_members(groups)
        if not remaining:
            return {}
        time.sleep(0.05)
    return group_members(groups)


def kill_groups(groups):
    for group in groups:
        try:
            os.killpg(group, signal.SIGKILL)
        except ProcessLookupError:
            pass


def run_capture(name, root, force_timeout):
    output = root / name
    output.mkdir()
    log_path = root / f"{name}.out"
    command = [CAPTURE_PYTHON, "-B", str(CAPTURE), "gnome", str(output), "app-auto"]
    environment = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
    owned_groups = set()
    observed_members = {}
    with log_path.open("wb") as log:
        process = subprocess.Popen(command, cwd=ROOT, env=environment,
                                   stdout=log, stderr=subprocess.STDOUT,
                                   start_new_session=True)
        deadline = time.monotonic() + 60
        started = time.monotonic()
        timed_out = False
        try:
            while process.poll() is None:
                holders = output_holders(output)
                owned_groups.update(details["pgid"] for details in holders.values()
                                    if details["pgid"] != process.pid)
                observed_members.update(group_members(owned_groups))
                if (force_timeout and len(owned_groups) >= 2
                        and time.monotonic() - started >= 3):
                    if len(observed_members) <= len(owned_groups):
                        raise RuntimeError("forced-timeout control did not observe guarded descendants")
                    os.killpg(process.pid, signal.SIGKILL)
                    timed_out = True
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"capture exceeded 60 seconds: {name}")
                time.sleep(0.05)
            process.wait(timeout=5)
            if force_timeout:
                if not timed_out:
                    raise RuntimeError("capture exited before the forced-timeout control")
            elif process.returncode:
                raise RuntimeError(f"normal capture failed ({process.returncode}): {log_path}")
            remaining = wait_for_cleanup(owned_groups)
            if remaining:
                raise RuntimeError(f"owned capture groups survived {name}: {remaining}")
            if len(owned_groups) < 2:
                raise RuntimeError(f"did not observe both host and Xvfb groups: {owned_groups}")
        finally:
            if process.poll() is None:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait(timeout=5)
            kill_groups(owned_groups)
    return {"command": command, "forced_timeout": force_timeout,
            "owned_groups": sorted(owned_groups),
            "observed_members": observed_members, "remaining": {}}


def validate_historical_control(control):
    if "not an acceptance pass" not in control.get("result", ""):
        raise RuntimeError("historical timeout defect is mislabeled as acceptance")
    if not control.get("surviving_capture_processes"):
        raise RuntimeError("historical timeout control no longer demonstrates the leak")
    if control.get("running_after_manual_cleanup"):
        raise RuntimeError("historical timeout control left processes running")


def validate_review(text):
    if REVIEW_HEADING not in text:
        raise RuntimeError("final API-020 review is missing")
    final = text.split(REVIEW_HEADING, 1)[1]
    required = ["Status: Complete", "Known open findings: none",
                "Historical defect controls are not acceptance evidence"]
    required.extend(f"RAPI-{number:02d}" for number in range(1, 16))
    required.extend(f"API-{number:03d}" for number in range(1, 21))
    missing = [item for item in required if item not in final]
    if missing:
        raise RuntimeError("final API-020 review is incomplete: " + ", ".join(missing))


def validate_todo(text):
    if "- Done: API-020" not in text or "- In progress:" in text or "- Pending:" in text:
        raise RuntimeError("API-020 work list is not complete")


def main():
    if not sys.platform.startswith("linux") or not Path("/proc").is_dir():
        raise RuntimeError("API-020 process ownership check requires Linux /proc")
    control = json.loads(CONTROL.read_text())
    try:
        validate_historical_control({**control, "surviving_capture_processes": []})
    except RuntimeError:
        pass
    else:
        raise RuntimeError("historical-control validator accepted a non-violating case")
    validate_historical_control(control)

    review = REVIEW.read_text()
    final_review = review.split(REVIEW_HEADING, 1)[-1]
    try:
        validate_review(REVIEW_HEADING + final_review.replace("RAPI-15", "RAPI-final"))
    except RuntimeError:
        pass
    else:
        raise RuntimeError("review validator accepted an incomplete mapping")
    validate_review(review)
    validate_todo(TODO.read_text())

    subprocess.run(["cargo", "build", "--locked", "--example", "image_host_probe",
                    "--jobs", "8"], cwd=ROOT, check=True, timeout=600)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = ROOT / ".cairn/reviews/api-020-closure" / stamp
    output.mkdir(parents=True)
    results = {
        "historical_control": "recognized as a defect demonstration",
        "normal_cleanup": run_capture("normal", output, False),
        "forced_timeout_cleanup": run_capture("forced-timeout", output, True),
    }
    (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print("API-020 closure: audit mapping, negative controls, normal cleanup and forced-timeout cleanup PASS")


if __name__ == "__main__":
    main()
