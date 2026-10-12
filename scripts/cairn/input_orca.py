#!/usr/bin/env python3
"""input-orca: CTL-011 (docs/spec/input-widgets.md), what Orca speaks for the
input pieces on the accessibility probe's input page, on this Linux host.

Builds the example accessibility_probe (its path read from cargo's JSON
messages, so CARGO_TARGET_DIR is honoured), then runs
tests/api_widget_behavior/orca.py with `--catalog inputs`: the probe opens on
its input page (`--inputs`) in GNOME Terminal on a private Xvfb display and
D-Bus session with real Orca (`orca --debug-file`), keys are sent by XTest,
and tests/api_widget_behavior/orca_inputs.py reads the speech Orca logs after
each step: the number input Volume given the focus, emptied, typed 4 then 2,
and emptied again; 2025 typed into the number input Year; the switch Wi-Fi given the focus
and Space; the rating Stars given the focus and Right; the masked code input
given the focus and typed 1 and 2. The reader runs against the patched
libatspi that scripts/check-libatspi-lifetime.py builds under
target/libatspi-lifetime; this script builds it when it is missing.

Orca allows one instance per user and the harness never replaces one. When
an Orca already runs for this user, or Orca, Xvfb, GNOME Terminal, xwininfo,
dbus-run-session, gsettings or /usr/bin/python3 with the GI AT-SPI bindings
is missing, or the private display, D-Bus session or reader cannot be
started, or the toolchain crashed, no result line is printed, so Sudus
records the run as unverified. Otherwise prints `cairn: CTL-011: pass|fail`;
a failure quotes the harness's error, for a speech step the AssertionError
that names every step whose speech broke the requirement.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

from _common import JOBS, ROOT, report, run
from workspace_gates import toolchain_crashes

REQUIREMENT = "CTL-011"
EXAMPLE = "accessibility_probe"
PYTHON = "/usr/bin/python3"
HARNESS = "tests/api_widget_behavior/orca.py"
LIBRARY = ROOT / "target/libatspi-lifetime/environment.json"
TOOLS = ("orca", "Xvfb", "gnome-terminal", "xwininfo", "dbus-run-session", "gsettings")
# Errors raised before the probe is started: the host could not give the run
# a display, a session or a reader, which says nothing about the code.
SETUP_ERRORS = ("Xvfb did not become ready", "Xvfb returned an invalid display number",
                "Orca exited during startup", "Orca did not register with AT-SPI",
                "GNOME Terminal is absent from the private X display",
                "cannot open the private X display")
ERROR_LINE = re.compile(r"^([\w.]*(?:Error|Exception|Expired)): (.*)$")


class Unverified(Exception):
    """The run could not reach a verdict on this host."""


class BuildFailed(Exception):
    """The probe does not build: the check fails."""


def missing_tools() -> list[str]:
    missing = [tool for tool in TOOLS if shutil.which(tool) is None]
    if not Path(PYTHON).is_file():
        missing.append(PYTHON)
    else:
        probe = subprocess.run([PYTHON, "-c", "import gi; gi.require_version('Atspi', '2.0'); "
                                "from gi.repository import Atspi"], capture_output=True, text=True, timeout=30)
        if probe.returncode != 0:
            missing.append(f"{PYTHON} with the GI AT-SPI bindings")
    return missing


def running_orca() -> list[str]:
    out = subprocess.run(["pgrep", "-a", "-x", "-u", str(os.getuid()), "orca"], capture_output=True, text=True)
    return [line for line in out.stdout.splitlines() if line.strip()]


def build_probe() -> Path:
    """The probe's executable. A toolchain crash raises Unverified; any other
    build failure raises BuildFailed with cargo's error lines."""
    result = run(["cargo", "build", "--locked", "--example", EXAMPLE, "--jobs", JOBS,
                  "--message-format=json-render-diagnostics"], timeout=1800)
    if result.returncode != 0:
        print(result.stderr[-4000:])
        crashes = toolchain_crashes(result.stderr)
        if crashes:
            raise Unverified(f"the toolchain crashed building {EXAMPLE}: {', '.join(crashes)}")
        errors = [l.strip() for l in result.stderr.splitlines() if l.strip().startswith("error")]
        raise BuildFailed("; ".join(errors[:4]) or f"cargo build exited {result.returncode}")
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if (message.get("reason") == "compiler-artifact" and message.get("target", {}).get("name") == EXAMPLE
                and message.get("executable")):
            return Path(message["executable"])
    raise BuildFailed(f"cargo named no executable for the example {EXAMPLE}")


def library_dir() -> str:
    """The patched libatspi's directory, built first when it is missing."""
    if not LIBRARY.is_file():
        built = run(["python3", "-B", "scripts/check-libatspi-lifetime.py"], timeout=900, interleave=True)
        print(built.stdout[-3000:])
        if built.returncode != 0 or not LIBRARY.is_file():
            raise Unverified("the patched libatspi could not be built (scripts/check-libatspi-lifetime.py "
                             f"exited {built.returncode})")
    try:
        return json.loads(LIBRARY.read_text())["library_dir"]
    except (OSError, ValueError, KeyError) as error:
        raise Unverified(f"no readable library_dir in {LIBRARY.relative_to(ROOT)} ({error})") from error


def last_error(text: str) -> tuple[str, str] | None:
    """The last `SomeError: message` line of a traceback in `text`."""
    found = None
    for line in text.splitlines():
        match = ERROR_LINE.match(line.strip())
        if match:
            found = (match.group(1), match.group(2).strip())
    return found


def verdict(output: str, code: int, summary: str = "A11Y inputs:") -> tuple[bool, str]:
    """Pass when the harness exited zero and printed the scenario's summary
    line, starting `summary`; otherwise the failure, read from the session
    log the harness names (the run inside the private session) before its
    own output."""
    if code == 0 and summary in output:
        return True, next(l for l in output.splitlines() if l.startswith(summary))
    logs = re.search(r"Orca diagnostic logs: (\S+)", output)
    inner = None
    if logs and (Path(logs.group(1)) / "session.log").is_file():
        inner = last_error((Path(logs.group(1)) / "session.log").read_text(errors="replace"))
    error = inner or last_error(output)
    if error is None:
        tail = [l.strip() for l in output.splitlines() if l.strip()][-3:]
        return False, f"the Orca run exited {code} without an error line: {' / '.join(tail)}"
    kind, message = error
    if any(message.startswith(setup) for setup in SETUP_ERRORS):
        raise Unverified(f"the private display, session or reader did not start: {message}")
    return False, message if kind == "AssertionError" else f"{kind}: {message}"


def main() -> int:
    missing = missing_tools()
    if missing:
        print(f"{REQUIREMENT} unverified: missing on this host: {', '.join(missing)}")
        return 0
    others = running_orca()
    if others:
        print(f"{REQUIREMENT} unverified: an Orca already runs for this user, and Orca allows one: "
              f"{'; '.join(others)}")
        return 0
    try:
        try:
            probe = build_probe()
        except BuildFailed as error:
            return 0 if report(REQUIREMENT, False, f"the accessibility probe does not build: {error}") else 1
        library = library_dir()
        env = {"LD_LIBRARY_PATH": library + (":" + os.environ["LD_LIBRARY_PATH"]
                                             if os.environ.get("LD_LIBRARY_PATH") else ""),
               # The harness imports its scenario modules: keep bytecode out of tests/.
               "PYTHONDONTWRITEBYTECODE": "1"}
        try:
            result = run([PYTHON, "-B", HARNESS, "--binary", str(probe), "--catalog", "inputs",
                          "--geometry", "100x32"], timeout=180, env=env, interleave=True)
        except subprocess.TimeoutExpired:
            return 0 if report(REQUIREMENT, False, "the Orca run did not finish within 180 s") else 1
        print(result.stdout[-6000:])
        ok, why = verdict(result.stdout, result.returncode)
    except Unverified as error:
        print(f"{REQUIREMENT} unverified: {error}")
        return 0
    return 0 if report(REQUIREMENT, ok, why) else 1


if __name__ == "__main__":
    sys.exit(main())
