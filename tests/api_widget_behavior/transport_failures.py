#!/usr/bin/env python3
"""Force missing and stalled private buses through App and verify terminal cleanup."""
import argparse
import fcntl
import os
from pathlib import Path
import pty
import re
import select
import signal
import socket
import struct
import subprocess
import tempfile
import termios
import time

CONTROL_SEQUENCE = re.compile(rb"\x1b\[[0-?]*[ -/]*[@-~]")


def validate_probe(automatic, stalled, returncode, output, elapsed, terminal_restored):
    text = output.decode(errors="replace")
    if automatic:
        assert returncode == 0, text
        painted = CONTROL_SEQUENCE.sub(b"", output)
        assert b"Painted account" in painted, "automatic mode did not deliver its first frame"
    else:
        assert returncode != 0, "unavailable screen-reader transport reported success"
        assert "screen-reader session-bus connection" in text, text
        if stalled:
            assert "3000 ms deadline" in text, text
            assert elapsed >= 2.5, "the stalled connection did not reach its deadline"
    assert elapsed < 7, elapsed
    assert terminal_restored, "App left the terminal in raw mode"
    assert b"\x1b[?1049l" in output, "App did not leave its alternate screen"


def prove_validator_rejects_violations():
    screen = b"Painted account\x1b[?1049l"
    error = b"screen-reader session-bus connection failed\x1b[?1049l"
    violations = [
        (True, False, 1, screen, 0.2, True),
        (True, False, 0, b"\x1b[?1049l", 0.2, True),
        (False, False, 0, screen, 0.2, True),
        (False, False, 1, b"unknown failure\x1b[?1049l", 0.2, True),
    ]
    for observation in violations:
        try:
            validate_probe(*observation)
        except AssertionError:
            continue
        raise AssertionError(f"validator accepted violating observation: {observation!r}")
    validate_probe(True, False, 0, screen, 0.2, True)
    validate_probe(False, False, 1, error, 0.2, True)


def probe(binary, directory, stalled, automatic=False):
    mode = "automatic" if automatic else "explicit"
    state = "stalled" if stalled else "missing"
    endpoint = directory / f"{mode}-{state}.sock"
    server = socket.socket(socket.AF_UNIX) if stalled else None
    master, slave = pty.openpty()
    child = None
    try:
        if server:
            server.bind(str(endpoint))
            server.listen(1)  # Accept the connection without answering its authentication.
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 10, 32, 0, 0))
        original = termios.tcgetattr(slave)
        env = os.environ.copy()
        env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=" + str(endpoint)
        env.pop("AT_SPI_BUS_ADDRESS", None)
        env.pop("DBUS_STARTER_ADDRESS", None)
        start = time.monotonic()
        command = [binary, str(directory / "callbacks.jsonl")]
        if automatic:
            command.append("--automatic")
        child = subprocess.Popen(command,
                                 stdin=slave, stdout=slave, stderr=slave,
                                 env=env, start_new_session=True)
        output = bytearray()
        # For a stalled automatic connection, do not let F9 close the App until
        # after the transport deadline has exposed the failure to App::run.
        next_key = start + (3.5 if automatic and stalled else 0.2)
        # Drain terminal output during execution: a full PTY would block the
        # renderer before App can observe the independent transport failure.
        while child.poll() is None:
            if time.monotonic() - start > 8:
                raise AssertionError(f"App did not stop after bus failure: {output[-2000:].decode(errors='replace')!r}")
            if select.select([master], [], [], 0.05)[0]:
                output.extend(os.read(master, 16384))
                assert len(output) < 1_000_000, "unexpected fixture output volume"
            if automatic and time.monotonic() >= next_key:
                os.write(master, b"\x1b[20~")  # F9 closes each successive App.
                next_key = time.monotonic() + 0.2
        child.wait(timeout=1)
        elapsed = time.monotonic() - start
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 16384))
            assert len(output) < 1_000_000, "unexpected fixture output volume"
        validate_probe(
            automatic,
            stalled,
            child.returncode,
            bytes(output),
            elapsed,
            termios.tcgetattr(slave) == original,
        )
        if automatic:
            print(f"A11Y automatic mode with {state} bus: frame, input, and cleanup passed in {elapsed:.2f}s")
        else:
            print(f"A11Y {'stalled' if stalled else 'missing'} bus: error returned and terminal restored in {elapsed:.2f}s")
    finally:
        if child and child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=3)
        os.close(master)
        os.close(slave)
        if server:
            server.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    args = parser.parse_args()
    prove_validator_rejects_violations()
    with tempfile.TemporaryDirectory(prefix="rtui-a11y-failure-") as temporary:
        for stalled in [False, True]:
            probe(str(Path(args.binary).resolve()), Path(temporary), stalled)
        for stalled in [False, True]:
            probe(str(Path(args.binary).resolve()), Path(temporary), stalled, automatic=True)
