#!/usr/bin/env python3
"""Run repaired FFI paths in an isolated terminal and check the shipped C ABI."""
import fcntl
import os
from pathlib import Path
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time


def run_in_terminal(command):
    master, slave = os.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    original = termios.tcgetattr(slave)
    process = None
    output = bytearray()
    try:
        process = subprocess.Popen(
            command, stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            env={**os.environ, "TERM": "xterm-256color", "RUST_BACKTRACE": "0"},
        )
        deadline = time.monotonic() + 120
        while process.poll() is None:
            if time.monotonic() >= deadline:
                raise TimeoutError(f"FFI command timed out: {command}")
            if select.select([master], [], [], 0.1)[0]:
                output.extend(os.read(master, 65536))
            if len(output) > 16 * 1024 * 1024:
                raise RuntimeError("FFI output exceeded 16 MiB")
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 65536))
        sys.stdout.buffer.write(output)
        sys.stdout.buffer.flush()
        if process.returncode != 0:
            raise RuntimeError(f"FFI command exited {process.returncode}: {command}")
        if termios.tcgetattr(slave) != original:
            raise RuntimeError(f"FFI command did not restore terminal settings: {command}")
    finally:
        if process is not None and process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


def main():
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target")).resolve()
    subprocess.run(["cargo", "build", "--locked", "--features", "ffi"], check=True, timeout=180)
    with tempfile.TemporaryDirectory(prefix="ffi-runtime-", dir=target) as scratch:
        smoke = Path(scratch) / "terminal-smoke"
        subprocess.run([
            os.environ.get("CC", "cc"), "-std=c11", "-Wall", "-Wextra", "-Werror",
            "-Iinclude", "tests/ffi_terminal_smoke.c", "-L" + str(target / "debug"),
            "-Wl,-rpath," + str(target / "debug"), "-lreactive_tui", "-o", str(smoke),
        ], check=True, timeout=30)
        run_in_terminal([str(smoke)])
    run_in_terminal([
        "cargo", "test", "--locked", "--features", "ffi", "--lib",
        "ffi::", "--", "--test-threads=1",
    ])
    command = ["cargo", "test", "--locked", "--features", "ffi"]
    for test in ["ffi_tests", "ffi_basic_test", "test_minimal_ffi", "test_ffi_core_only", "test_ffi_integration", "test_ffi_reactive_seamless"]:
        command.extend(["--test", test])
    command.extend(["--", "--test-threads=1"])
    run_in_terminal(command)
    print("FFI compile, C ABI and Rust lifecycle checks passed")


if __name__ == "__main__":
    main()
