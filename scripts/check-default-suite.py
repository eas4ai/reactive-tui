#!/usr/bin/env python3
"""Run the whole default suite with bounded execution and workspace temp space."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile


def main():
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    target.mkdir(parents=True, exist_ok=True)
    # Rustdoc's parallel link jobs can exceed the system /tmp quota.
    with tempfile.TemporaryDirectory(prefix="default-suite-", dir=target) as scratch:
        process = subprocess.Popen(
            ["cargo", "test", "--locked", "--no-fail-fast"],
            env={**os.environ, "TMPDIR": scratch}, start_new_session=True,
        )
        try:
            return process.wait(timeout=300)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            print("Default suite exceeded its 300-second deadline", flush=True)
            return 124


if __name__ == "__main__":
    raise SystemExit(main())
