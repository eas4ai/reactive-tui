"""Bounded acceptance checks for EOF draining and terminal restoration assertions."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
CASES = {
    "pty-raw-leak": "REAL_PTY_RAW_RESTORATION_REJECTED",
    "eof": "EOF_DRAIN_RETURNED_AND_RESTORATION_PASSED",
    "raw-leak": "RAW_RESTORATION_REJECTED",
    "never-drained": "DRAIN_DEADLINE_REJECTED",
    "pty": "REAL_PTY_NONBLOCKING_EXIT_AND_RESTORATION_PASSED",
}
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
output = ROOT / "target/evidence/api-entry-points" / ("harness-" + stamp)
output.mkdir(parents=True)
results = []
for case, marker in CASES.items():
    command = [sys.executable, "-B", str(ROOT / "verification/api-entry-points/harness-consumer.py"),
               case, str(output / (case + ".bin"))]
    log = output / (case + ".out")
    row = {"case": case, "command": command, "timeout_seconds": 5}
    child = None
    try:
        with log.open("wb") as stream:
            child = subprocess.Popen(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT,
                                     start_new_session=True)
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                # Give the consumer's finally block a bounded chance to reap its PTY child.
                os.killpg(child.pid, signal.SIGTERM)
                raise
            finally:
                if child.poll() is None:
                    try:
                        child.wait(timeout=1)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait(timeout=2)
        assert child.returncode == 0, f"consumer exited {child.returncode}"
        assert marker in log.read_text(errors="replace"), "consumer omitted " + marker
    except (AssertionError, OSError, subprocess.SubprocessError) as error:
        row.update(result="fail", error=str(error))
    else:
        row["result"] = "pass"
    if child is not None:
        row["returncode"] = child.returncode
    row["files"] = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in (log, output / (case + ".bin")) if path.exists()}
    results.append(row)
    (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(row["result"].upper(), "terminal harness", case, flush=True)
print("Terminal harness captures:", output, flush=True)
if any(row["result"] != "pass" for row in results):
    raise SystemExit("Terminal harness guard checks failed")
print("TERMINAL_HARNESS_OK", flush=True)
