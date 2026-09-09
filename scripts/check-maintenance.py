#!/usr/bin/env python3
"""Run a maintenance gate with captured output and bounded execution."""
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile

COMMANDS = {
    "format": ["cargo", "fmt", "--all", "--", "--check"],
    "lint": ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
    "ffi": ["cargo", "test", "--locked", "--features", "ffi", "--no-run"],
}


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in COMMANDS:
        raise SystemExit("Expected format, lint or ffi")
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    target.mkdir(parents=True, exist_ok=True)
    environment = {**os.environ}
    if sys.argv[1] == "ffi":
        # rustc 1.95 panicked reusing an incremental dependency graph after an
        # editor type change. Compile acceptance targets without that cache.
        environment["CARGO_INCREMENTAL"] = "0"
    with tempfile.TemporaryDirectory(dir=target) as scratch, tempfile.TemporaryFile(dir=target) as output:
        process = subprocess.Popen(
            COMMANDS[sys.argv[1]],
            env={**environment, "TMPDIR": scratch},
            stdout=output, stderr=subprocess.STDOUT, start_new_session=True,
        )
        try:
            status = process.wait(timeout=300)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            status = 124
        output.seek(0)
        shutil.copyfileobj(output, sys.stdout.buffer)
        sys.stdout.buffer.flush()
        if status == 0 and sys.argv[1] == "ffi":
            return subprocess.call([sys.executable, "scripts/check-ffi-runtime.py"])
        return status


if __name__ == "__main__":
    raise SystemExit(main())
