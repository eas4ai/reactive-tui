#!/usr/bin/env python3
"""BAR-001: the five workspace gates pass on Linux.

Prints `cairn: BAR-001: pass` when all five commands exit zero and
`cairn: BAR-001: fail` when one does not. These checks make sure a gate that
exits zero actually ran:

- No cargo configuration this checkout reads may stand in for cargo's own
  build and test: a target runner, which runs each test binary and can
  print any result, or a replacement or wrapper for rustc or rustdoc.
  Cargo reads .cargo/config.toml (and .cargo/config) in the checkout, in
  every directory above it and in CARGO_HOME; the mechanism's identity
  hashes the same files. A file that includes another (`include`) is
  refused too, since cargo would apply settings from a file neither this
  check nor the identity reads. So is a setting that picks the program that
  links: a target's or the host's `linker`, or in `rustflags` a
  `-C linker=`, or a link argument that points the C compiler at a
  particular linker (`-fuse-ld=` with a path, `--ld-path=`, `-B<dir>`);
  `-fuse-ld=mold`, a linker named by the toolchain, is allowed. A
  `--runtool` in `rustdocflags`, which would run the doc-tests, is refused.
- The run prints which cargo, rustc, mbx and cc are on PATH and every
  linker cc would run (its own ld, and ld.<name> for each -fuse-ld=<name>
  the configurations set, as `cc -print-prog-name` finds them), with each
  file's path and hash, so a wrapper such as mbx's cargo shim, or a
  replaced ld.mold, shows in the output. `--programs` prints only that
  list; the mechanism's identity records it. The environment the gates run in keeps none of
  the variables that set these (see _common.ENV_KEEP).
- cargo test must print a result for every test binary and doc-test run it
  starts (the harness = false targets in Cargo.toml print their own
  output), run at least one test, and leave the proof that
  src/gate_proof.rs writes: a value the gate chose, which only running the
  unit test binary can put in the file.
- rustfmt must still report a badly formatted snippet under the
  configuration of the root and of every directory with its own
  rustfmt.toml or .rustfmt.toml, since rustfmt reads the nearest one for
  each file. Such a file with disable_all_formatting would make the fmt
  gate pass on any code below it.

When a gate fails only because rustc, rustdoc, clippy-driver or the linker
was killed by a signal, the run prints no result line, so Sudus records it
as unverified: it shows nothing about the code, and a rerun decides. A
compiler that overflows its stack is the code's doing and still fails. A
doc-test counts as failed by the crash when rustdoc could not compile it
because the compiler was killed, as its captured output shows; any other
failed test in the run still fails the gate.
"""

import hashlib
import os
import re
import secrets
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

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
    re.compile(rf"^error: ({TOOLCHAIN}) interrupted by (SIG[A-Z]+)", re.M),
    # gcc driving the linker: collect2: fatal error: ld terminated with signal 11 [Segmentation fault]
    re.compile(r"^\s*(?:= note: )?(?:\S*/)?collect2(?:\.exe)?: fatal error: (\S+) terminated with signal \d+ \[([^\]]+)\]", re.M),
]
# cargo: process didn't exit successfully: `<command>` (signal: 11, SIGSEGV: invalid memory reference)
SIGNALLED = re.compile(r"process didn't exit successfully: `([^`]*)` \(signal: \d+, (SIG[A-Z]+)")
# An error line that a toolchain crash does not explain: a compile error, a
# lint, a failed test, a formatting diff.
OWN_ERROR = re.compile(r"^error(?:\[E\d+\])?: (?!could not compile|could not document|linking with|build failed|"
                       rf"aborting due to|{TOOLCHAIN} interrupted by|(?:doc)?test failed, to rerun|\d+ targets? failed)")
# The header of a failed test's captured output, libtest's closing list of
# failed tests, and cargo's lines naming the targets that failed.
CAPTURED = re.compile(r"^---- (.+?) stdout ----$", re.M)
FAILURE_LIST = re.compile(r"^failures:\n((?: {4}\S.*\n)+)\s*test result:", re.M)
NON_DOC_FAILED = re.compile(r"^error: test failed, to rerun pass", re.M)
FAILED_TARGETS = re.compile(r"^error: \d+ targets? failed:\n((?: {4}.+\n?)+)", re.M)
STACK_OVERFLOW = "overflowed its stack"
BINARY_START = re.compile(r"^\s+(Running|Doc-tests) (\S+)(?: \((\S+)\))?")
RESULT = re.compile(r"^test result: \w+\. (\d+) passed; (\d+) failed")
RESULT_LINES = re.compile(RESULT.pattern, re.M)
BINARY_HEADERS = re.compile(r"^[ \t]+(Running|Doc-tests) (\S+)", re.M)
CANARY = "fn   badly_formatted() {}\n"
RUSTFMT_CONFIGS = ("rustfmt.toml", ".rustfmt.toml")
# Build settings that replace the compiler or documentation tool, or wrap it.
SUBSTITUTE_BUILD_KEYS = ("rustc", "rustc-wrapper", "rustc-workspace-wrapper", "rustdoc")
PROOF, NONCE = "REACTIVE_TUI_GATE_PROOF", "REACTIVE_TUI_GATE_NONCE"


def toolchain_crashes(output: str) -> list[str]:
    """What killed the toolchain in this output, when that is all that went
    wrong; empty when the code has a failure of its own or none crashed."""
    crashes = {f"{m.group(1)} killed by {m.group(2)}" for pattern in TOOLCHAIN_CRASH for m in pattern.finditer(output)}
    for m in SIGNALLED.finditer(output):
        # The program's own name, not its path: a test binary under a
        # directory named rustc-out is still a test binary. A compiler
        # wrapper runs the tool given as its first argument
        # (`mbx-rustc /.../rustc ...`, as cargo runs RUSTC_WRAPPER).
        names = [word.rsplit("/", 1)[-1] for word in m.group(1).split()[:2]]
        program = next((n for n in names if re.fullmatch(rf"{TOOLCHAIN}(?:\.exe)?", n)), None)
        if program is None:
            return []  # a test binary or build script died: that is the code's result
        crashes.add(f"{program} killed by {m.group(2)}")
    lines = output.splitlines()
    tests_failed = "test result: FAILED" in output and not only_doctests_crashed(output)
    if (STACK_OVERFLOW in output or tests_failed or "Diff in " in output
            or any(OWN_ERROR.match(line) for line in lines)):
        return []
    return sorted(crashes)


def sections(output: str) -> list[tuple[str, str]]:
    """(kind, text) for each test binary and doc-test run in `output`, kind
    being "Running" or "Doc-tests", split at cargo's header lines."""
    starts = list(BINARY_HEADERS.finditer(output))
    return [(m.group(1), output[m.start():starts[i + 1].start() if i + 1 < len(starts) else len(output)])
            for i, m in enumerate(starts)]


def only_doctests_crashed(output: str) -> bool:
    """Whether the run's only failed tests are doc-tests the compiler's crash
    failed. A doc-test fails when rustdoc cannot compile it, so a compiler
    killed while compiling one fails it. This is decided from what cargo and
    libtest report, not from a test's own progress line, which another
    process writing to the same output can garble: no non-doc target may
    have failed, every target cargo lists as failed must be a doc-test run,
    and in each failed doc-test run the closing list of failed tests must
    hold as many names as its result line counts, each with the crash in
    its captured output."""
    if NON_DOC_FAILED.search(output):
        return False
    for listed in FAILED_TARGETS.finditer(output):
        if any("--doc" not in target for target in listed.group(1).splitlines() if target.strip()):
            return False
    crashed_any = False
    for kind, text in sections(output):
        results = RESULT_LINES.findall(text)
        failed = sum(int(count) for _, count in results)
        if kind != "Doc-tests":
            if failed or "test result: FAILED" in text:
                return False
            continue
        if not failed:
            continue
        listed = FAILURE_LIST.findall(text)
        names = [line.strip() for line in listed[-1].splitlines()] if listed else []
        if len(names) != failed or not all(compile_crash(captured(text, name)) for name in names):
            return False
        crashed_any = True
    return crashed_any


def captured(output: str, name: str) -> str:
    """The output libtest captured for the failed test `name`: from its
    `---- name stdout ----` header to the next header or `failures:` list."""
    for m in CAPTURED.finditer(output):
        if m.group(1) == name:
            end = re.compile(r"^(?:---- .+ stdout ----|failures:)$", re.M).search(output, m.end())
            return output[m.end():end.start() if end else len(output)]
    return ""


def crash_in(text: str) -> bool:
    """Whether `text` shows the toolchain killed by a signal."""
    return any(pattern.search(text) for pattern in TOOLCHAIN_CRASH)


def compile_crash(text: str) -> bool:
    """Whether a doc-test's captured output shows rustdoc failing to compile
    it because the toolchain was killed: the crash, rustdoc's own "Couldn't
    compile the test." and no run of the test. A doc-test that compiled,
    ran and printed crash-like text still fails."""
    return crash_in(text) and "Couldn't compile the test." in text and "Test executable failed" not in text


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


def cargo_config_files() -> list[Path]:
    """Every configuration file cargo reads for this checkout, nearest first:
    .cargo/config.toml and .cargo/config in the checkout and each directory
    above it, then in CARGO_HOME."""
    home = Path(os.environ.get("CARGO_HOME") or Path.home() / ".cargo")
    found = []
    for directory in [*(d / ".cargo" for d in (ROOT, *ROOT.parents)), home]:
        for name in ("config.toml", "config"):
            path = directory / name
            if path.is_file() and path.resolve() not in {f.resolve() for f in found}:
                found.append(path)
    return found


def codegen_options(value) -> list[str]:
    """The -C (--codegen) options in a rustflags setting, written as one
    string or as a list."""
    tokens = value.split() if isinstance(value, str) else [str(v) for v in value] if isinstance(value, list) else []
    found = []
    for i, token in enumerate(tokens):
        following = tokens[i + 1] if i + 1 < len(tokens) else ""
        if token in ("-C", "--codegen"):
            found.append(following)
        elif token.startswith("--codegen="):
            found.append(token.split("=", 1)[1])
        elif token.startswith("-C"):
            found.append(token[2:])
    return [option for option in found if option]


def linker_choices(rustflags) -> list[str]:
    """The options in `rustflags` that pick the program that links."""
    found = []
    for option in codegen_options(rustflags):
        key, _, value = option.partition("=")
        if key == "linker":
            found.append(f"-C {option}")
        elif key in ("link-arg", "link-args"):
            found += [f"-C {key}={arg}" for arg in value.split()
                      if arg.startswith(("-B", "--ld-path")) or (arg.startswith("-fuse-ld=") and "/" in arg)]
    return found


def runtools(rustdocflags) -> list[str]:
    """The options in `rustdocflags` that run doc-tests through another program."""
    tokens = rustdocflags.split() if isinstance(rustdocflags, str) else [str(v) for v in rustdocflags] if isinstance(rustdocflags, list) else []
    return [token for token in tokens if token.startswith(("--runtool", "--test-runtool"))]


def config_tables(config: dict) -> list[tuple[str, dict]]:
    """The tables of a cargo configuration that can hold rustflags or a
    linker: build, host and each target, with their names."""
    tables = [("build", config["build"])] if isinstance(config.get("build"), dict) else [("build", {})]
    if isinstance(config.get("host"), dict):
        tables.append(("host", config["host"]))
    targets = config.get("target")
    tables += [(f"target.{name}", table) for name, table in (targets.items() if isinstance(targets, dict) else ())
               if isinstance(table, dict)]
    return tables


def substitutes(files: list[Path]) -> list[str]:
    """The settings in `files` that let something other than cargo's own
    build and test stand in for them, or that load settings from a file
    not in `files`."""
    found = []
    for path in files:
        try:
            config = tomllib.loads(path.read_text())
        except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
            found.append(f"{path} cannot be read ({error})")
            continue
        targets = config.get("target")
        for name, table in (targets.items() if isinstance(targets, dict) else ()):
            if isinstance(table, dict) and "runner" in table:
                found.append(f"{path} sets target.{name}.runner")
        build = config.get("build") if isinstance(config.get("build"), dict) else {}
        found += [f"{path} sets build.{key}" for key in SUBSTITUTE_BUILD_KEYS if build.get(key)]
        for label, table in config_tables(config):
            if label != "build" and "linker" in table:
                found.append(f"{path} sets {label}.linker")
            found += [f"{path} sets {choice} in {label}.rustflags" for choice in linker_choices(table.get("rustflags"))]
            found += [f"{path} sets {tool} in {label}.rustdocflags" for tool in runtools(table.get("rustdocflags"))]
        if "include" in config:
            found.append(f"{path} sets include, which applies settings from files this check does not read")
    return found


def rustfmt_config_dirs() -> list[Path]:
    """The repository root and every directory in the tree with its own
    rustfmt configuration, tracked or not."""
    try:
        out = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                             cwd=ROOT, capture_output=True)
        names = [p.decode() for p in out.stdout.split(b"\0") if p] if out.returncode == 0 else None
    except FileNotFoundError:
        names = None
    if names is None:
        skip = {".git", "target", "node_modules"}
        names = [str(p.relative_to(ROOT)) for p in ROOT.rglob("*rustfmt.toml")
                 if not skip & set(p.relative_to(ROOT).parts)]
    dirs = {ROOT} | {(ROOT / n).parent for n in names if Path(n).name in RUSTFMT_CONFIGS}
    return sorted(dirs)


def silent_rustfmt_dirs() -> list[str]:
    """Directories where rustfmt, reading stdin there, reports no diff for a
    badly formatted snippet: under their configuration the check cannot
    fail."""
    silent = []
    for directory in rustfmt_config_dirs():
        r = run(["rustfmt", "--check", "--edition", "2021"], timeout=120, interleave=True,
                stdin=CANARY, cwd=directory)
        if "Diff in <stdin>" not in r.stdout:
            silent.append(str(directory.relative_to(ROOT)) or ".")
    return silent


def linker_names(files: list[Path]) -> list[str]:
    """The linker programs cc may run for these configurations: its own ld,
    and ld.<name> for each -fuse-ld=<name> they set."""
    names = ["ld"]
    for path in files:
        try:
            config = tomllib.loads(path.read_text())
        except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError):
            continue
        for _, table in config_tables(config):
            for option in codegen_options(table.get("rustflags")):
                key, _, value = option.partition("=")
                if key in ("link-arg", "link-args"):
                    names += [f"ld.{arg.split('=', 1)[1]}" for arg in value.split()
                              if arg.startswith("-fuse-ld=") and "/" not in arg]
    return list(dict.fromkeys(names))


def describe(name: str, found: str | None) -> str:
    """`name: path [-> real path] (script|program, sha256 ...)`."""
    if not found:
        return f"{name}: not found"
    real = os.path.realpath(found)
    try:
        data = Path(real).read_bytes()
    except OSError as error:
        return f"{name}: {found} cannot be read ({error})"
    kind = "script" if data.startswith(b"#!") else "program"
    target = f" -> {real}" if real != found else ""
    return f"{name}: {found}{target} ({kind}, sha256 {hashlib.sha256(data).hexdigest()})"


def programs(configs: list[Path]) -> list[str]:
    """Which cargo, rustc, mbx and cc the gates find on PATH, and the linkers
    cc would run, with each file's hash: a wrapper in front of cargo, or a
    replaced linker, shows here."""
    lines = [describe(name, shutil.which(name)) for name in ("cargo", "rustc", "mbx", "cc")]
    for name in linker_names(configs):
        try:
            given = subprocess.run(["cc", f"-print-prog-name={name}"], capture_output=True, text=True,
                                   timeout=30).stdout.strip()
        except (OSError, subprocess.TimeoutExpired):
            given = ""
        found = given if os.path.isabs(given) and os.path.isfile(given) else shutil.which(given or name)
        lines.append(describe(f"linker {name}", found))
    return lines


def main() -> int:
    configs = cargo_config_files()
    for line in programs(configs):
        print(line)
    if "--programs" in sys.argv[1:]:
        return 0
    print("cargo configuration read:", ", ".join(map(str, configs)) or "none")
    standins = substitutes(configs)
    if standins:
        report("BAR-001", False, "a cargo setting could stand in for the build or the tests, so a passing "
                                 f"gate would prove nothing: {'; '.join(standins)}")
        return 1
    failed, crashed = [], []
    proof = Path(tempfile.mkdtemp(prefix="workspace-gates-")) / "proof"
    nonce = secrets.token_hex(16)
    for cmd in GATES:
        test = cmd[1] == "test"
        r = run(cmd, timeout=3600, interleave=test, env={PROOF: str(proof), NONCE: nonce} if test else None)
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
            written = proof.read_text() if proof.is_file() else None
            shutil.rmtree(proof.parent, ignore_errors=True)
            if written != nonce:
                failed.append(f"{name} did not run the unit tests: src/gate_proof.rs left "
                              f"{'no proof file' if written is None else 'a different value'}")
        if cmd[1] == "fmt":
            silent_dirs = silent_rustfmt_dirs()
            if silent_dirs:
                failed.append(f"{name}: rustfmt reports no diff for a badly formatted snippet under the "
                              f"configuration in {', '.join(silent_dirs)}, so the check cannot fail there")
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
