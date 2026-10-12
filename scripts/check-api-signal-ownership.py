#!/usr/bin/env python3
"""Check native signal lifecycles, refusing the known unsafe baseline first."""
from pathlib import Path
import os
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def reject_direct_signal_casts(source):
    if re.search(r"as\s+\*(?:mut|const)\s+Signal\s*<", source):
        raise RuntimeError("API-001 unsafe direct Signal cast: native lifecycle tests were not run")


def run(command, **kwargs):
    subprocess.run(command, cwd=ROOT, check=True, timeout=900, **kwargs)


def main():
    # Safe violating and corrected examples test the guard itself, not Rust memory safety.
    try:
        reject_direct_signal_casts("Box::from_raw(signal as *mut Signal<String>)")
    except RuntimeError:
        pass
    else:
        raise AssertionError("unsafe baseline guard did not reject its negative control")
    reject_direct_signal_casts("signal.downcast_ref::<Signal<String>>()")
    reject_direct_signal_casts((ROOT / "src/ffi/reactive.rs").read_text())
    run(["cargo", "test", "--locked", "--features", "ffi", "--test", "api_signal_ownership"])
    run(["cargo", "build", "--locked", "--features", "ffi"])
    with tempfile.TemporaryDirectory(prefix="signal-ownership-", dir=ROOT / "target") as scratch:
        executable = str(Path(scratch) / "consumer")
        run(["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", "-Iinclude",
             "tests/api_signal_ownership.c", "-Ltarget/debug", "-lreactive_tui",
             "-Wl,-rpath," + str(ROOT / "target/debug"), "-o", executable])
        run([executable])
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(ROOT / "target/api-signal-miri")
    run(["cargo", "+nightly", "miri", "test", "--locked", "--features", "ffi",
         "--test", "api_signal_ownership"], env=env)
    print("API-001 Rust/C signal lifecycles and Miri passed", flush=True)


if __name__ == "__main__":
    main()
