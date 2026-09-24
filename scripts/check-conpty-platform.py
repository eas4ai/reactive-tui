#!/usr/bin/env python3
"""Record native ConPTY behavior and reject a deliberately disabled resize."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
RECORDS = ROOT / "docs/analysis/conpty-platform"
INPUTS = ("Cargo.toml", "Cargo.lock", "build.rs", "src",
          "tests/api_widget_behavior/conpty_probe.rs", "scripts/install-conpty-runtime.py",
          ".github/workflows/clipboard-platforms.yml",
          "scripts/abi/baselines/binding-abi-baseline.json")
MARKERS = ("PASS console handles, input, environment, resize, real exit 259, repeated stop",
           "PASS failed-launch cleanup and restart", "PASS flood backpressure and drop cleanup",
           "PASS idle backpressure and drop cleanup", "PASS attached descendant cleanup")


def digest():
    if subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "--", *INPUTS], cwd=ROOT):
        raise RuntimeError("commit ConPTY inputs before recording evidence")
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--", *INPUTS], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    names = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", "HEAD", "--", *INPUTS], cwd=ROOT, text=True).splitlines()
    result = hashlib.sha256()
    for name in names:
        data = subprocess.check_output(["git", "show", "HEAD:" + name], cwd=ROOT)
        result.update(name.encode() + b"\0" + len(data).to_bytes(8, "big") + data)
    return result.hexdigest()


def execute(command, directory, log, timeout):
    env = {**os.environ, "CARGO_INCREMENTAL": "0"}
    with log.open("wb") as output:
        child = subprocess.Popen(command, cwd=directory, env=env, stdin=subprocess.DEVNULL,
                                 stdout=output, stderr=subprocess.STDOUT)
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            subprocess.run(["taskkill", "/PID", str(child.pid), "/T", "/F"], check=False,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            child.wait()
            raise RuntimeError("native ConPTY command exceeded its deadline") from None


def run():
    if platform.system() != "Windows":
        raise RuntimeError("ConPTY acceptance requires native Windows")
    initial = digest()
    RECORDS.mkdir(parents=True, exist_ok=True)
    build = RECORDS / "build.out"
    if execute(["cargo", "build", "--locked", "--example", "conpty_probe"], ROOT, build, 900):
        raise RuntimeError("native ConPTY build failed: " + str(build))
    executable = ROOT / "target/debug/examples/conpty_probe.exe"
    installer = ["python", "-B", str(ROOT / "scripts/install-conpty-runtime.py")]
    subprocess.run(installer + [str(executable.parent), "--arch", "x64"], check=True)
    runtime_errors = RECORDS / "runtime-errors.out"
    with tempfile.TemporaryDirectory(prefix="conpty-runtime-errors-") as scratch:
        copied = Path(scratch) / executable.name
        shutil.copyfile(executable, copied)
        status = execute([str(copied)], ROOT, runtime_errors, 30)
        missing = runtime_errors.read_text(errors="replace")
        if status == 0 or "Install the matched runtime" not in missing:
            raise RuntimeError("missing runtime did not report an actionable launch error")
        subprocess.run(installer + [str(copied.parent), "--arch", "x64"], check=True)
        with (copied.parent / "reactive-tui-conpty/conpty.dll").open("ab") as dll:
            dll.write(b"deliberately damaged runtime")
        status = execute([str(copied)], ROOT, runtime_errors, 30)
        damaged = runtime_errors.read_text(errors="replace")
        if status == 0 or "SHA-256 does not match" not in damaged:
            raise RuntimeError("damaged runtime was not rejected before loading")
        runtime_errors.write_text(missing + "\n" + damaged +
                                 "\nPASS missing and damaged ConPTY runtime rejected\n")
    corrected = RECORDS / "corrected.out"
    if execute([str(executable)], ROOT, corrected, 90):
        raise RuntimeError("native ConPTY behavior failed: " + str(corrected))
    text = corrected.read_text(errors="replace")
    if not all(marker in text for marker in MARKERS):
        raise RuntimeError("native ConPTY probe did not execute every required case")
    ffi = RECORDS / "ffi-exports.out"
    if execute(["cargo", "build", "--locked", "--features", "ffi"], ROOT, ffi, 900):
        raise RuntimeError("native Windows FFI build failed: " + str(ffi))
    # Load every baseline C entry point, without calling functions with arguments.
    library = ctypes.CDLL(str(ROOT / "target/debug/reactive_tui.dll"))
    exports = json.loads(
        (ROOT / "scripts/abi/baselines/binding-abi-baseline.json").read_text()
    )["rust_exports"]
    for name in exports:
        getattr(library, name)
    with ffi.open("a") as output:
        output.write(f"\nPASS {len(exports)} baseline C exports load\n")

    # Work on an isolated source copy. Never mutate the candidate being measured.
    with tempfile.TemporaryDirectory(prefix="conpty-violation-") as scratch:
        candidate = Path(scratch)
        for name in ("Cargo.toml", "Cargo.lock", "build.rs", "src", "tests", "benches",
                       "crates", "examples"):
            source = ROOT / name
            if source.is_dir():
                shutil.copytree(source, candidate / name, ignore=shutil.ignore_patterns("target", "__pycache__"))
            else:
                shutil.copyfile(source, candidate / name)
        native = candidate / "src/terminal/pty/windows/native.rs"
        source = native.read_text()
        start = source.index("        let result = unsafe {\n            (self.runtime.resize)(")
        end = source.index("        if result < 0", start)
        native.write_text(source[:start] + "        let result = 0; // Deliberately violate resize.\n" + source[end:])
        negative_build = RECORDS / "negative-build.out"
        if execute(["cargo", "build", "--locked", "--example", "conpty_probe"], candidate, negative_build, 900):
            raise RuntimeError("negative case failed to compile; this is not a behavior demonstration")
        negative = RECORDS / "negative.out"
        subprocess.run(installer + [str(candidate / "target/debug/examples"), "--arch", "x64"], check=True)
        status = execute([str(candidate / "target/debug/examples/conpty_probe.exe")], candidate, negative, 90)
        text = negative.read_text(errors="replace")
        if status == 0 or 'missing "INPUT:resized:SIZE:100:30"' not in text:
            raise RuntimeError("disabled resize did not fail for the intended reason")
    if digest() != initial:
        raise RuntimeError("ConPTY candidate changed during execution")
    outputs = {name: hashlib.sha256((RECORDS / name).read_bytes()).hexdigest()
               for name in ("build.out", "corrected.out", "negative-build.out", "negative.out", "ffi-exports.out", "runtime-errors.out")}
    record = {"system": platform.system(), "platform": platform.platform(), "inputs_digest": initial,
              "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "result": "pass", "outputs": outputs}
    (RECORDS / "windows.json").write_text(json.dumps(record, indent=2) + "\n")
    print("Native ConPTY cases passed; disabled resize failed for the intended reason")


def verify():
    record = json.loads((RECORDS / "windows.json").read_text())
    if record.get("system") != "Windows" or record.get("result") != "pass" or record.get("inputs_digest") != digest():
        raise RuntimeError("native ConPTY evidence is stale or invalid")
    for name in ("build.out", "corrected.out", "negative-build.out", "negative.out", "ffi-exports.out", "runtime-errors.out"):
        data = (RECORDS / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != record.get("outputs", {}).get(name):
            raise RuntimeError("damaged ConPTY evidence: " + name)
    if not all(marker in (RECORDS / "corrected.out").read_text() for marker in MARKERS):
        raise RuntimeError("native ConPTY cases are missing")
    if 'missing "INPUT:resized:SIZE:100:30"' not in (RECORDS / "negative.out").read_text():
        raise RuntimeError("ConPTY violating example is missing")
    if "PASS missing and damaged ConPTY runtime rejected" not in (RECORDS / "runtime-errors.out").read_text():
        raise RuntimeError("ConPTY runtime integrity checks are missing")
    print("Native ConPTY evidence matches the committed inputs")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    verify() if args.verify else run()
