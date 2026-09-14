#!/usr/bin/env python3
"""Run focused pre-release terminal lifecycle checks."""

from __future__ import annotations

import errno
import fcntl
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import sys
import termios
import time


ROOT = Path(__file__).resolve().parents[1]
ENTER = b"\x1b[?1049h\x1b[?25l\x1b[?1004h"
RESTORE = b"\x18\x1b[?2026l\x1b[0m\x1b[?1004l\x1b[0 q\x1b]112\x07\x1b[?25h\x1b[?1049l"
JOBS = "8"


def validate_capture(
    output: bytes,
    before: list[object],
    after: list[object],
    marker: bytes,
) -> list[str]:
    errors: list[str] = []
    entered = output.find(ENTER)
    restored = output.rfind(RESTORE)
    visible_panic = output.rfind(marker)
    if entered < 0:
        errors.append("terminal enter sequence is missing")
    if restored < 0:
        errors.append("complete terminal restore sequence is missing")
    if before != after:
        errors.append("PTY termios state was not restored")
    if restored >= 0 and visible_panic < restored + len(RESTORE):
        errors.append("panic marker was not printed after terminal restoration")
    return errors


def prove_validator() -> None:
    cooked = [1, 2, 3]
    marker = b"PANIC"
    valid = ENTER + RESTORE + marker
    assert not validate_capture(valid, cooked, cooked, marker)
    assert validate_capture(ENTER + marker, cooked, cooked, marker)
    assert validate_capture(valid, cooked, [1, 2, 4], marker)
    assert validate_capture(ENTER + marker + RESTORE, cooked, cooked, marker)


def run_pty_probe(test_name: str, mode: str, marker: bytes) -> None:
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    before = termios.tcgetattr(slave)
    pid = os.fork()
    if pid == 0:
        try:
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
            for descriptor in (0, 1, 2):
                os.dup2(slave, descriptor)
            os.close(master)
            if slave > 2:
                os.close(slave)
            env = os.environ.copy()
            env["CARGO_BUILD_JOBS"] = JOBS
            env["REACTIVE_TUI_TRL_001_PROBE"] = mode
            argv = [
                "cargo",
                "test",
                "--locked",
                "--lib",
                test_name,
                "--jobs",
                env["CARGO_BUILD_JOBS"],
                "--",
                "--ignored",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ]
            os.execvpe(argv[0], argv, env)
        finally:
            os._exit(127)

    os.close(slave)
    output = bytearray()
    status: int | None = None
    deadline = time.monotonic() + 20
    try:
        while status is None:
            if time.monotonic() >= deadline:
                os.killpg(pid, signal.SIGKILL)
                os.waitpid(pid, 0)
                raise RuntimeError(f"{mode} panic probe exceeded 20 seconds")
            readable, _, _ = select.select([master], [], [], 0.1)
            if readable:
                try:
                    output.extend(os.read(master, 65536))
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
            waited, child_status = os.waitpid(pid, os.WNOHANG)
            if waited == pid:
                status = child_status
        while True:
            readable, _, _ = select.select([master], [], [], 0)
            if not readable:
                break
            try:
                chunk = os.read(master, 65536)
            except OSError:
                break
            if not chunk:
                break
            output.extend(chunk)
        after = termios.tcgetattr(master)
    finally:
        os.close(master)

    if not os.WIFEXITED(status) or os.WEXITSTATUS(status) != 0:
        sys.stderr.buffer.write(output)
        raise RuntimeError(f"{mode} panic probe failed with wait status {status}")
    errors = validate_capture(bytes(output), before, after, marker)
    if errors:
        sys.stderr.buffer.write(output)
        raise RuntimeError(f"{mode} panic probe: {'; '.join(errors)}")
    print(
        f"{mode} panic probe restored termios and terminal modes before visible panic text"
    )


def run_trl_001() -> None:
    if os.name != "posix":
        raise RuntimeError("TRL-001 PTY mechanism requires a POSIX host")
    prove_validator()
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        ["cargo", "test", "--locked", "--lib", "--no-run", "--jobs", env["CARGO_BUILD_JOBS"]],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    run_pty_probe(
        "backend::suprtui::trl_001_tests::worker_panic_restores_the_owned_terminal",
        "worker",
        b"TRL001_WORKER_PANIC",
    )
    run_pty_probe(
        "app::tests::main_thread_panic_restores_the_owned_terminal",
        "main",
        b"TRL001_MAIN_PANIC",
    )
    print("TRL-001 terminal restoration after worker and main-thread panics passed")


def main() -> int:
    if sys.argv[1:] != ["TRL-001"]:
        print(f"usage: {Path(sys.argv[0]).name} TRL-001", file=sys.stderr)
        return 2
    run_trl_001()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
