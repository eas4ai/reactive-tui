#!/usr/bin/env python3
"""Exercise real input, SIGWINCH, ANSI output, and termios restoration."""

import errno
import fcntl
import os
from pathlib import Path
import select
import signal
import struct
import subprocess
import sys
import termios
import time


sys.dont_write_bytecode = True

from pty_screen import SYNC_END, completed, screen_text


def counter_frame(data, count, after=0, dimensions=None):
    frame = completed(data)
    if len(frame) <= after or f"Count: {count}" not in screen_text(frame):
        return False
    if dimensions is not None:
        columns, rows = dimensions
        return f"\x1b[{rows};{columns}H".encode() in frame[after:]
    return True


def read_until(master, process, output, predicate, description):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if predicate(output):
            return
        if select.select([master], [], [], 0.1)[0]:
            try:
                data = os.read(master, 65536)
            except OSError as error:
                if error.errno == errno.EIO:
                    break
                raise
            if not data:
                break
            output.extend(data)
        elif process.poll() is not None:
            break
    assert predicate(output), f"{description}; exit={process.poll()}, tail={bytes(output[-500:])!r}"


def resize(slave, columns, rows):
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0))


def probe(binary, mode="", quit_key=b"\x1b"):
    master, slave = os.openpty()
    resize(slave, 52, 16)
    original = termios.tcgetattr(slave)
    output = bytearray()
    process = None
    try:
        process = subprocess.Popen(
            [str(binary)] + ([mode] if mode else []),
            stdin=slave, stdout=slave, stderr=slave,
            start_new_session=True,
            env={**os.environ, "TERM": "xterm-256color", "RUST_BACKTRACE": "0"},
        )
        read_until(master, process, output,
                   lambda data: counter_frame(data, 0),
                   "initial counter frame missing")
        assert termios.tcgetattr(slave) != original, "raw mode was never enabled"
        assert b"\x1b[?1049h" in output and b"\x1b[?25l" in output
        offset = len(output)
        os.write(master, b" ")
        if not mode:
            read_until(master, process, output,
                       lambda data: counter_frame(data, 1, offset),
                       "Space did not redraw the counter through App")
            for columns, rows in [(34, 12), (60, 18)]:
                offset = len(output)
                resize(slave, columns, rows)
                os.kill(process.pid, signal.SIGWINCH)
                read_until(master, process, output,
                           lambda data: counter_frame(data, 1, offset, (columns, rows)),
                           "resize did not produce a complete frame")
            os.write(master, quit_key)
        read_until(master, process, output, lambda data: b"\x1b[?1049l" in data,
                   "alternate screen was not restored")
        status = process.wait(timeout=10)
        # Drain diagnostic output after shutdown; the slave stays open to inspect termios.
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 65536))
        assert (status == 0) == (not mode), f"unexpected exit {status}: {bytes(output[-500:])!r}"
        assert termios.tcgetattr(slave) == original, "termios was not restored exactly"
        restored = output.rfind(b"\x1b[?1049l")
        assert b"\x1b[?25h" in output[:restored], "cursor was not restored"
        if mode:
            expected = b"controlled application input error" if mode == "--probe-error" else b"controlled application panic"
            assert expected in output, f"expected failure was not exercised: {bytes(output[-500:])!r}"
        print(f"PASS RND-003/005/006 PTY: {mode or ('Escape' if quit_key == bytes([27]) else 'Ctrl+C')}")
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    # Cargo's target directory can be overridden by the caller.
    binary = Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug/examples/suprtui_renderer_probe"
    binary = binary.resolve()
    probe(binary)
    probe(binary, quit_key=b"\x03")
    probe(binary, "--probe-error")
    probe(binary, "--probe-panic")
