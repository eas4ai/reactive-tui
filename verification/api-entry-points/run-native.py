"""Run public terminal entry points on a native Unix host with committed-input evidence."""
from pathlib import Path
import json
import os
import platform
import runpy
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
os.chdir(ROOT)
os.environ.update(CARGO_BUILD_JOBS="8", RUST_TEST_THREADS="8", CARGO_INCREMENTAL="0",
                  CARGO_TARGET_DIR=str(ROOT / "target"), RAYON_NUM_THREADS="8",
                  LP_NUM_THREADS="8", PYTHON_CPU_COUNT="8", GOMAXPROCS="8", GOFLAGS="-p=8")
INPUTS = ("src", "tests", "crates/reactive-tui-macros", "vendor", "include", ".cargo",
          "scripts", "verification", "Cargo.toml", "Cargo.lock", "build.rs",
          ".github/workflows/clipboard-platforms.yml")


def snapshot():
    entries = subprocess.check_output(["git", "ls-tree", "-r", "-z", "HEAD", "--", *INPUTS]).split(b"\0")
    blobs = {}
    for entry in entries:
        if entry:
            metadata, name = entry.split(b"\t", 1)
            blobs[name.decode()] = metadata.decode().split()[2]
    clean = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", *INPUTS]).returncode == 0
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "--", *INPUTS])
    return {"source": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
            "blob_ids": blobs, "committed_inputs": clean and not untracked}


def require_workflows(text):
    for marker in (
        "All public entry-point workflows passed",
        "PASS public Crossterm: 2048 bytes, exhausted zero-timeout poll and restoration",
    ):
        if marker not in text:
            raise AssertionError("Native entry-point output omitted " + marker)


if platform.system() not in ("Darwin", "Linux"):
    raise SystemExit("Native entry-point PTY checks require macOS or Linux")
before = snapshot()
check_module = runpy.run_path(str(ROOT / "scripts/check-api-residual.py"))
check = check_module["Check"]()
check.output = ROOT / "target/evidence/api-entry-points" / ("native-" + check.output.name)
check.output.mkdir(parents=True)
check.run("entry-points", [sys.executable, "-B", "scripts/check-api-entry-points.py"],
          timeout=1200, verify=require_workflows)
record = {"platform": platform.system(), "machine": platform.machine(),
          **before, "steps": check.steps}
record["stable_inputs"] = snapshot() == before
(check.output / "native-entry-points.json").write_text(json.dumps(record, indent=2) + "\n")
if (any(step["result"] != "pass" for step in check.steps)
        or not record["stable_inputs"] or not record["committed_inputs"]):
    raise SystemExit("Native entry-point checks failed; retained " + str(check.output))
print("NATIVE_ENTRY_POINTS_OK", check.output)
