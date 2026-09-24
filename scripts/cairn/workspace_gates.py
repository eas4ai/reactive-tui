#!/usr/bin/env python3
"""BAR-001: the five workspace gates pass on Linux.

Prints `cairn: BAR-001: pass` when all five commands exit zero and
`cairn: BAR-001: fail` when one does not. Two checks make sure a gate that
exits zero actually ran:

- cargo test must print a result for every test binary and doc-test run it
  starts (the harness = false targets in Cargo.toml print their own
  output), and run at least one test. A runner that swallows the binaries,
  such as `runner = "true"` in a .cargo/config.toml, prints none.
- rustfmt must still report a badly formatted snippet under the
  repository's configuration. A rustfmt.toml with disable_all_formatting
  would make the fmt gate pass on any code.

When a gate fails only because rustc, rustdoc, clippy-driver or the linker
was killed by a signal, the run prints no result line, so Sudus records it
as unverified: it shows nothing about the code, and a rerun decides. A
compiler that overflows its stack is the code's doing and still fails.
"""

import re
import sys
import tomllib

from _common import JOBS, ROOT, report, run

GATES = [
    ["cargo", "build", "--locked", "--workspace", "--jobs", JOBS],
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--locked", "--workspace", "--all-targets", "--jobs", JOBS, "--", "-D", "warnings"],
    ["cargo", "doc", "--locked", "--workspace", "--no-deps", "--jobs", JOBS],
    ["cargo", "test", "--locked", "--workspace", "--no-fail-fast", "--jobs", JOBS],
]

TOOLCHAIN = r"(?:rustc|rustdoc|clippy-driver)"
TOOLCHAIN_CRASH = [
    # rustc: error: rustc interrupted by SIGSEGV, printing backtrace
    re.compile(rf"\b({TOOLCHAIN}) interrupted by (SIG[A-Z]+)"),
    # gcc driving the linker: collect2: fatal error: ld terminated with signal 11 [Segmentation fault]
    re.compile(r"\b(ld|collect2)\b[^\n]*terminated with signal \d+ \[([^\]]+)\]"),
]
# cargo: process didn't exit successfully: `<command>` (signal: 11, SIGSEGV: invalid memory reference)
SIGNALLED = re.compile(r"process didn't exit successfully: `([^`]*)` \(signal: \d+, (SIG[A-Z]+)")
# An error line that a toolchain crash does not explain: a compile error, a
# lint, a failed test, a formatting diff.
OWN_ERROR = re.compile(r"^error(?:\[E\d+\])?: (?!could not compile|linking with|build failed|"
                       rf"{TOOLCHAIN} interrupted by|test failed, to rerun)")
STACK_OVERFLOW = "overflowed its stack"
BINARY_START = re.compile(r"^\s+(Running|Doc-tests) (\S+)(?: \((\S+)\))?")
RESULT = re.compile(r"^test result: \w+\. (\d+) passed; (\d+) failed")
CANARY = "fn   badly_formatted() {}\n"


def toolchain_crashes(output: str) -> list[str]:
    """What killed the toolchain in this output, when that is all that went
    wrong; empty when the code has a failure of its own or none crashed."""
    crashes = {f"{m.group(1)} killed by {m.group(2)}" for pattern in TOOLCHAIN_CRASH for m in pattern.finditer(output)}
    for m in SIGNALLED.finditer(output):
        if not re.search(rf"\b{TOOLCHAIN}\b", m.group(1).split(" ", 1)[0]):
            return []  # a test binary or build script died: that is the code's result
        crashes.add(f"{m.group(1).split(' ', 1)[0].rsplit('/', 1)[-1]} killed by {m.group(2)}")
    lines = output.splitlines()
    if (STACK_OVERFLOW in output or "test result: FAILED" in output or "Diff in " in output
            or any(OWN_ERROR.match(line) for line in lines)):
        return []
    return sorted(crashes)


def no_harness_targets() -> set[str]:
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return {t["name"] for kind in ("test", "bench") for t in manifest.get(kind, []) if t.get("harness") is False}


def unreported_binaries(output: str) -> tuple[list[str], int]:
    """Test binaries that printed no `test result:` line before the next one
    started, and the number of tests that ran."""
    exempt = no_harness_targets()
    silent, current, tests = [], None, 0
    for line in output.splitlines():
        start = BINARY_START.match(line)
        if start:
            if current:
                silent.append(current)
            name = start.group(2)
            binary = (start.group(3) or "").rsplit("/", 1)[-1]
            exempt_target = re.sub(r"-[0-9a-f]{16}$", "", binary) in exempt
            current = None if exempt_target else f"{start.group(1)} {name}"
            continue
        result = RESULT.match(line)
        if result:
            tests += int(result.group(1)) + int(result.group(2))
            current = None
    if current:
        silent.append(current)
    return silent, tests


def fmt_can_fail() -> bool:
    """rustfmt, reading stdin at the repository root, still reports a diff."""
    r = run(["rustfmt", "--check", "--edition", "2021"], timeout=120, interleave=True, stdin=CANARY)
    return "Diff in <stdin>" in r.stdout


def main() -> int:
    failed, crashed = [], []
    for cmd in GATES:
        test = cmd[1] == "test"
        r = run(cmd, timeout=3600, interleave=test)
        output = r.stdout + (r.stderr or "")
        name = " ".join(cmd[:3])
        if r.returncode != 0:
            lines = output.splitlines()
            # Name the failure: failed tests, panics, signals and errors, then the tail.
            marks = ("FAILED", "panicked at", "signal:", "signal ", "error", "failures:", "warning: unused")
            hits = [l for l in lines if any(m in l for m in marks)]
            print("\n".join(hits[:60] + lines[-10:]))
            crashes = toolchain_crashes(output)
            if crashes:
                crashed.append(f"{name} ({'; '.join(crashes)})")
            else:
                failed.append(name)
            continue
        if test:
            silent, tests = unreported_binaries(output)
            if silent or tests == 0:
                failed.append(f"{name} printed no test result for {len(silent)} test binaries "
                              f"({', '.join(silent[:5])}) and ran {tests} tests")
        if cmd[1] == "fmt" and not fmt_can_fail():
            failed.append(f"{name}: rustfmt reports no diff for a badly formatted snippet under this "
                          "repository's configuration, so the check cannot fail")
    if failed:
        report("BAR-001", False, f"gates failing: {', '.join(failed)}")
        return 1
    if crashed:
        # No result line: Sudus records the run as unverified.
        print(f"BAR-001 unverified: the toolchain crashed, so these gates reached no verdict: {', '.join(crashed)}")
        return 1
    report("BAR-001", True, "all five gates pass")
    return 0


if __name__ == "__main__":
    sys.exit(main())
