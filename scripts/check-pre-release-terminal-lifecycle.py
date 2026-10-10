#!/usr/bin/env python3
"""Run focused pre-release terminal lifecycle checks."""

from __future__ import annotations

import errno
import fcntl
import os
from pathlib import Path
import pty
import re
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

    shutdown = b"TRL002_SIGNAL_SHUTDOWN"
    valid_shutdown = ENTER + RESTORE + shutdown
    assert not validate_completion(valid_shutdown, cooked, cooked, 0, shutdown)
    assert validate_completion(
        valid_shutdown, cooked, cooked, signal.SIGTERM, shutdown
    )
    assert validate_completion(ENTER + shutdown, cooked, cooked, 0, shutdown)
    assert validate_completion(
        valid_shutdown, cooked, cooked, 0, shutdown, timed_out=True
    )
    assert validate_hook_chain(ENTER + RESTORE + b"TRL002_HOOK_CHAINED")
    assert validate_ownership(b"TRL002_OWNER_DROPPED", cooked, cooked, 0)


def validate_completion(
    output: bytes,
    before: list[object],
    after: list[object],
    status: int,
    marker: bytes,
    *,
    timed_out: bool = False,
) -> list[str]:
    errors = validate_capture(output, before, after, marker)
    if timed_out:
        errors.append("probe exceeded its shutdown deadline")
    elif not os.WIFEXITED(status) or os.WEXITSTATUS(status) != 0:
        errors.append(f"probe failed with wait status {status}")
    return errors


def validate_hook_chain(output: bytes) -> list[str]:
    if output.count(b"TRL002_PRIOR_HOOK") != 1:
        return ["panic hook did not chain the prior hook exactly once"]
    return []


def validate_ownership(
    output: bytes,
    before: list[object],
    after: list[object],
    status: int,
) -> list[str]:
    errors: list[str] = []
    if not os.WIFEXITED(status) or os.WEXITSTATUS(status) != 0:
        errors.append(f"ownership probe failed with wait status {status}")
    if before != after:
        errors.append("ownership probe did not restore final PTY termios")
    active = output.find(b"TRL002_OWNER_ACTIVE")
    dropped = output.find(b"TRL002_OWNER_DROPPED")
    if active < 0 or dropped < active:
        errors.append("terminal did not remain active until its owner dropped")
    return errors


def execute_pty_probe(
    test_name: str,
    env_key: str,
    mode: str,
    signal_number: int | None = None,
) -> tuple[bytes, list[object], list[object], int]:
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
            env[env_key] = mode
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
    signal_sent = signal_number is None
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
            signal_target = re.search(rb"TRL002_SIGNAL_READY:(\d+)", output)
            if not signal_sent and signal_target is not None:
                os.kill(int(signal_target.group(1)), signal_number)
                signal_sent = True
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

    if not signal_sent:
        raise RuntimeError(f"{mode} probe exited before signal readiness")
    return bytes(output), before, after, status


def require_success(output: bytes, status: int, mode: str) -> None:
    if not os.WIFEXITED(status) or os.WEXITSTATUS(status) != 0:
        sys.stderr.buffer.write(output)
        raise RuntimeError(f"{mode} probe failed with wait status {status}")


def run_restore_probe(
    test_name: str,
    env_key: str,
    mode: str,
    marker: bytes,
    signal_number: int | None = None,
) -> bytes:
    output, before, after, status = execute_pty_probe(
        test_name, env_key, mode, signal_number
    )
    require_success(output, status, mode)
    errors = validate_capture(bytes(output), before, after, marker)
    if errors:
        sys.stderr.buffer.write(output)
        raise RuntimeError(f"{mode} probe: {'; '.join(errors)}")
    print(f"{mode} probe restored termios and terminal modes before its final marker")
    return output


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
    run_restore_probe(
        "backend::suprtui::trl_001_tests::worker_panic_restores_the_owned_terminal",
        "REACTIVE_TUI_TRL_001_PROBE",
        "worker",
        b"TRL001_WORKER_PANIC",
    )
    run_restore_probe(
        "app::tests::main_thread_panic_restores_the_owned_terminal",
        "REACTIVE_TUI_TRL_001_PROBE",
        "main",
        b"TRL001_MAIN_PANIC",
    )
    print("TRL-001 terminal restoration after worker and main-thread panics passed")


def run_trl_002() -> None:
    if os.name != "posix":
        raise RuntimeError("TRL-002 signal mechanism requires a POSIX host")
    prove_validator()
    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = JOBS
    subprocess.run(
        ["cargo", "test", "--locked", "--lib", "--no-run", "--jobs", JOBS],
        cwd=ROOT,
        env=env,
        check=True,
        timeout=600,
    )
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        run_restore_probe(
            "app::tests::termination_signal_uses_the_app_wake_path",
            "REACTIVE_TUI_TRL_002_PROBE",
            signal.Signals(signal_number).name,
            b"TRL002_SIGNAL_SHUTDOWN",
            signal_number,
        )
    hook = run_restore_probe(
        "platform::unix::tests::panic_handler_chains_the_prior_hook",
        "REACTIVE_TUI_TRL_002_PROBE",
        "hook",
        b"TRL002_HOOK_CHAINED",
    )
    hook_errors = validate_hook_chain(hook)
    if hook_errors:
        raise RuntimeError("; ".join(hook_errors))
    ownership, before, after, status = execute_pty_probe(
        "platform::unix::tests::foreign_thread_panic_keeps_terminal_owner_active",
        "REACTIVE_TUI_TRL_002_PROBE",
        "ownership",
    )
    ownership_errors = validate_ownership(ownership, before, after, status)
    if ownership_errors:
        sys.stderr.buffer.write(ownership)
        raise RuntimeError("; ".join(ownership_errors))
    print("TRL-002 bounded signal shutdown and panic-hook ownership passed")


def main() -> int:
    if sys.argv[1:] == ["TRL-001"]:
        run_trl_001()
    elif sys.argv[1:] == ["TRL-002"]:
        run_trl_002()
    else:
        print(f"usage: {Path(sys.argv[0]).name} TRL-001|TRL-002", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
