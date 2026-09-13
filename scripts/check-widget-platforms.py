#!/usr/bin/env python3
"""Record native HTTP, filesystem and terminal App workflows against committed inputs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
RECORDS = ROOT / "docs/analysis/widget-platforms"
INPUTS = ("Cargo.toml", "Cargo.lock", "build.rs", "src", "reactive-tui-macros",
          "tests/api_widget_behavior.rs", "scripts/check-dialog-http.py",
          "scripts/check-iterm-host.py",
          "scripts/install-conpty-runtime.py",
          ".github/workflows/clipboard-platforms.yml")
CASES = {
    "http-transport": ["--lib", "widgets::dialog::http::tests"],
    "http-app": ["--test", "api_widget_behavior", "dialog_http_acceptance::"],
    "filesystem-native": ["--lib", "widgets::display::file_explorer::"],
    "filesystem-app": ["--test", "api_widget_behavior", "file_explorer_acceptance::"],
    "terminal-app": ["--test", "api_widget_behavior", "terminal_acceptance::"],
    "terminal-native": ["--lib", "terminal::terminal_impl::tests::"],
    "terminal-screen": ["--lib", "terminal::screen::"],
    "image-native": ["--lib", "widgets::display::image::"],
    "image-app": ["--test", "api_widget_behavior", "image_acceptance::"],
    "platform-image-native": ["--lib", "platform::image::"],
    "surface-native": ["--lib", "core::surface::"],
}


def digest():
    rust_tests = tuple(name for name in subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", "HEAD", "--", "tests/api_widget_behavior"],
        cwd=ROOT, text=True).splitlines() if name.endswith(".rs"))
    paths = (*INPUTS, *rust_tests)
    untracked = subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard", "--",
         *INPUTS, "tests/api_widget_behavior"], cwd=ROOT, text=True).splitlines()
    if any(not name.startswith("tests/api_widget_behavior/") or name.endswith(".rs")
           for name in untracked):
        raise RuntimeError("Commit native widget inputs before recording evidence")
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--", *paths],
                     cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    names = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", "HEAD", "--", *paths],
                                      cwd=ROOT, text=True).splitlines()
    result = hashlib.sha256()
    for name in names:
        data = subprocess.check_output(["git", "show", "HEAD:" + name], cwd=ROOT)
        result.update(name.encode() + b"\0" + len(data).to_bytes(8, "big") + data)
    return result.hexdigest()


def execute(command, output, timeout):
    environment = {**os.environ, "CARGO_INCREMENTAL": "0", "CARGO_TERM_COLOR": "never"}
    with output.open("wb") as log:
        child = subprocess.Popen(command, cwd=ROOT, env=environment,
                                 stdout=log, stderr=subprocess.STDOUT,
                                 start_new_session=os.name != "nt")
        try:
            status = child.wait(timeout=timeout)
        except BaseException:
            if os.name == "nt":
                subprocess.run(["taskkill", "/PID", str(child.pid), "/T", "/F"],
                               check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            else:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            child.wait()
            raise
    if status:
        raise RuntimeError(f"Native widget command failed ({status}): {output}")
    return output.read_text(errors="replace")


def run(dedicated_desktop=False):
    system = platform.system()
    if system not in ("Linux", "Darwin", "Windows"):
        raise RuntimeError("No native widget platform contract for " + system)
    if system in ("Darwin", "Windows") and not dedicated_desktop:
        raise RuntimeError("Native widget verification requires --dedicated-desktop")
    initial = digest()
    directory = RECORDS / system.lower()
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "record.json").unlink(missing_ok=True)
    execute(["curl", "--version"], directory / "curl.out", 10)
    # Compile separately so case deadlines measure execution, not cold builds.
    execute(["cargo", "test", "--locked", "--lib", "--test", "api_widget_behavior", "--no-run"],
            directory / "build.out", 900)
    failures = []
    if system == "Windows":
        try:
            execute([sys.executable, "-B", "scripts/install-conpty-runtime.py",
                     str(ROOT / "target/debug/deps"), "--arch", "x64"],
                    directory / "runtime-install.out", 120)
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            failures.append(str(error))
    for name, arguments in CASES.items():
        try:
            output = execute(["cargo", "test", "--locked", *arguments, "--", "--test-threads=1"],
                             directory / (name + ".out"), 180)
            if not re.search(r"test result: ok\. [1-9][0-9]* passed; 0 failed;", output):
                raise RuntimeError("Native widget case did not execute tests: " + name)
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            failures.append(str(error))
    try:
        output = execute([sys.executable, "-B", "scripts/check-dialog-http.py"]
                         + (["--dedicated-desktop"] if dedicated_desktop else []),
                         directory / "https.out", 180)
        if not all(f"Dialog HTTPS {trust}: passed" in output for trust in ("trusted", "untrusted")):
            raise RuntimeError("Missing HTTPS trust checks")
    except (RuntimeError, subprocess.TimeoutExpired) as error:
        failures.append(str(error))
    if system == "Darwin":
        try:
            execute(["cargo", "build", "--locked", "--example", "image_host_probe"],
                    directory / "iterm-build.out", 300)
            execute([sys.executable, "-B", "scripts/check-iterm-host.py", "--dedicated-desktop"],
                    directory / "iterm-host.out", 240)
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            failures.append(str(error))
    if failures:
        raise RuntimeError("Native widget failures:\n" + "\n".join(failures))
    if digest() != initial:
        raise RuntimeError("Native widget inputs changed during execution")
    outputs = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
               for path in directory.glob("*.out")}
    if system == "Darwin":
        outputs.update({str(path.relative_to(directory)): hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in (directory / "iterm-host").rglob("*") if path.is_file()})
    record = {"system": system, "platform": platform.platform(), "inputs_digest": initial,
              "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "result": "pass", "outputs": outputs}
    (directory / "record.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"Native {system} HTTP, HTTPS, FileExplorer, TerminalWidget and image workflows passed", flush=True)


def verify():
    current = digest()
    for system in ("Darwin", "Windows"):
        directory = RECORDS / system.lower()
        record = json.loads((directory / "record.json").read_text())
        if (record.get("system") != system or record.get("result") != "pass"
                or record.get("inputs_digest") != current):
            raise RuntimeError("Stale or invalid native widget evidence: " + system)
        names = ["build", "curl", "https", *CASES]
        if system == "Windows":
            names.append("runtime-install")
        else:
            names.extend(["iterm-build", "iterm-host"])
        for name in names:
            data = (directory / (name + ".out")).read_bytes()
            if hashlib.sha256(data).hexdigest() != record.get("outputs", {}).get(name + ".out"):
                raise RuntimeError("Damaged native widget evidence: " + system + "/" + name)
        if system == "Darwin":
            required = ["iterm-host/host.json"]
            for mode in ("app-iterm", "app-auto", "surface-iterm", "iterm", "app-ascii"):
                required.extend(f"iterm-host/{mode}/{name}" for name in
                                ("pixels.json", "geometry-pixels.json", "fixture-exit.txt", "stage-0.png", "stage-1.png", "stage-2.png"))
            for name in required:
                if hashlib.sha256((directory / name).read_bytes()).hexdigest() != record.get("outputs", {}).get(name):
                    raise RuntimeError("Damaged native image evidence: " + name)
    print("Native macOS and Windows widget evidence matches committed inputs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--dedicated-desktop", action="store_true")
    arguments = parser.parse_args()
    verify() if arguments.verify else run(arguments.dedicated_desktop)
