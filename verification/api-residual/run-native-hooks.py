"""Run owned hook behavior on the native CI host and retain exact outputs."""
from pathlib import Path
import json
import os
import platform
import runpy
import subprocess

ROOT = Path(__file__).resolve().parents[2]
os.chdir(ROOT)
os.environ.update(CARGO_BUILD_JOBS="8", RUST_TEST_THREADS="1", CARGO_INCREMENTAL="0",
                  CARGO_TARGET_DIR=str(ROOT / "target"), RAYON_NUM_THREADS="8")
INPUTS = ("src", "tests", "crates/reactive-tui-macros", "scripts", "verification",
          "Cargo.toml", "Cargo.lock", "build.rs", ".github/workflows/clipboard-platforms.yml")

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

before = snapshot()
check_module = runpy.run_path(str(ROOT / "scripts/check-api-residual.py"))
check = check_module["Check"]()
for target, names in (("api_local_hooks", check_module["LOCAL_TESTS"]),
                      ("api_performance_context", check_module["PERFORMANCE_TESTS"])):
    check.run(target + "-discovery", ["cargo", "test", "--locked", "--test", target, "--", "--list"],
              verify=lambda text, names=names: check_module["require_registered"](text, names))
    check.run(target + "-behavior", ["cargo", "test", "--locked", "--test", target, "--", "--test-threads=1"],
              verify=lambda text, names=names: check_module["require_executed"](text, names))
record = {"platform": platform.system(), **before, "steps": check.steps}
record["stable_inputs"] = snapshot() == before
(check.output / "native-hooks.json").write_text(json.dumps(record, indent=2) + "\n")
if (any(step["result"] != "pass" for step in check.steps)
        or not record["stable_inputs"] or not record["committed_inputs"]):
    raise SystemExit("Native owned hook checks failed; retained " + str(check.output))
print("NATIVE_HOOKS_OK", check.output)
