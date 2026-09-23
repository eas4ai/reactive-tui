#!/usr/bin/env python3
"""Run the whole default suite with workspace temp space."""
import os
from pathlib import Path
import shutil
import sys
import subprocess
import tempfile


def main():
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    target.mkdir(parents=True, exist_ok=True)
    # Rustdoc's parallel link jobs can exceed the system /tmp quota.
    with tempfile.TemporaryDirectory(prefix="default-suite-", dir=target) as scratch, tempfile.TemporaryFile() as output:
        process = subprocess.Popen(
            ["cargo", "test", "--locked", "--no-fail-fast"],
            env={**os.environ, "TMPDIR": scratch}, start_new_session=True,
            stdout=output, stderr=subprocess.STDOUT,
        )
        status = process.wait()
        output.seek(0)
        shutil.copyfileobj(output, sys.stdout.buffer)
        return status


if __name__ == "__main__":
    raise SystemExit(main())
