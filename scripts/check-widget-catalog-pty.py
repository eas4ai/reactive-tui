#!/usr/bin/env python3
"""Observe real cube pixels and normal quit/terminal cleanup in a Unix PTY."""

from __future__ import annotations

import codecs
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parents[1]
CSI = re.compile(r"\x1b\[([0-?]*)([ -/]*)([@-~])")


class Screen:
    def __init__(self, width: int, height: int):
        self.width, self.height = width, height
        self.cells = [[" "] * width for _ in range(height)]
        self.x = self.y = 0
        self.pending = ""
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")

    def feed(self, data: bytes) -> None:
        self.pending += self.decoder.decode(data)
        while self.pending:
            if self.pending.startswith("\x1b["):
                match = CSI.match(self.pending)
                if not match:
                    break
                raw, _, code = match.groups()
                numbers = [int(part) if part.isdigit() else 0 for part in raw.split(";")]
                n = numbers[0] or 1
                if code in ("H", "f"):
                    self.y, self.x = n - 1, (numbers[1] or 1) - 1 if len(numbers) > 1 else 0
                elif code == "A": self.y -= n
                elif code == "B": self.y += n
                elif code == "C": self.x += n
                elif code == "D": self.x -= n
                elif code == "G": self.x = n - 1
                elif code == "d": self.y = n - 1
                elif code == "J" and numbers[0] in (2, 3):
                    self.cells = [[" "] * self.width for _ in range(self.height)]
                elif code == "K" and 0 <= self.y < self.height:
                    start, end = (0, self.width) if numbers[0] == 2 else (self.x, self.width)
                    for x in range(max(0, start), min(self.width, end)): self.cells[self.y][x] = " "
                self.pending = self.pending[match.end():]
            elif self.pending.startswith(("\x1b]", "\x1bP", "\x1b_")):
                end = re.search(r"\x07|\x1b\\", self.pending)
                if not end: break
                self.pending = self.pending[end.end():]
            elif self.pending.startswith("\x1b"):
                if len(self.pending) < 2: break
                self.pending = self.pending[2:]
            else:
                char, self.pending = self.pending[0], self.pending[1:]
                if char == "\r": self.x = 0
                elif char == "\n": self.y += 1
                elif char == "\b": self.x = max(0, self.x - 1)
                elif char >= " ":
                    if self.x >= self.width: self.x, self.y = 0, self.y + 1
                    if 0 <= self.y < self.height and 0 <= self.x < self.width:
                        self.cells[self.y][self.x] = char
                    self.x += 1

    def text(self) -> str:
        return "\n".join("".join(row) for row in self.cells)

    def cube(self) -> str:
        return "\n".join("".join(char if "\u2801" <= char <= "\u28ff" else " " for char in row) for row in self.cells)


def controlling_terminal() -> None:
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def check_run(executable: Path, key_name: str, key: bytes) -> None:
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 100, 0, 0))
    process = None
    screen = Screen(100, 32)
    output = bytearray()
    try:
        process = subprocess.Popen([str(executable)], cwd=ROOT, stdin=slave, stdout=slave, stderr=slave,
                                   env={**os.environ, "TERM": "xterm-256color"}, preexec_fn=controlling_terminal)
        deadline = time.monotonic() + 10
        navigated = False
        first_time = None
        frames: set[str] = set()
        while time.monotonic() < deadline and len(frames) < 2:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready:
                data = os.read(master, 65536)
                output.extend(data)
                screen.feed(data)
            if process.poll() is not None:
                raise AssertionError(f"example exited before capture: {process.returncode}")
            if not navigated and "Widget Catalog" in screen.text():
                os.write(master, b"7")
                navigated = True
            cube = screen.cube()
            if sum("\u2801" <= char <= "\u28ff" for char in cube) > 30:
                now = time.monotonic()
                digest = hashlib.sha256(cube.encode()).hexdigest()
                if first_time is None:
                    first_time = now
                    frames.add(digest)
                elif now - first_time >= 0.16:
                    frames.add(digest)
        assert len(frames) >= 2, f"two distinct cube frames were not observed:\n{screen.text()}"
        for frame in sorted(frames): print(f"CATALOG FRAME {frame}")
        os.write(master, key)
        deadline = time.monotonic() + 5
        while process.poll() is None and time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.1)
            if ready: output.extend(os.read(master, 65536))
        assert process.poll() == 0, f"{key_name} did not exit normally: {process.poll()}"
        process.wait()
        assert termios.tcgetattr(slave) == original, f"{key_name} left changed terminal settings"
        # The shutdown must leave the alternate screen and restore the cursor.
        assert b"\x1b[?1049l" in output, "alternate screen was not restored"
        assert b"\x1b[?25h" in output, "cursor was not restored"
        try:
            os.killpg(process.pid, 0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError("a catalog child process survived normal exit")
        print(f"{key_name} EXIT 0")
    finally:
        if process is not None and process.poll() is None:
            try: os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


def main() -> None:
    subprocess.run(["cargo", "+1.91.0", "build", "--locked", "--example", "widget_catalog"], cwd=ROOT, check=True)
    metadata = subprocess.check_output(["cargo", "+1.91.0", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT)
    executable = Path(json.loads(metadata)["target_directory"]) / "debug/examples/widget_catalog"
    for name, key in (("CTRL_Q", b"\x11"), ("CTRL_C", b"\x03"), ("ESCAPE", b"\x1b")):
        check_run(executable, name, key)


if __name__ == "__main__":
    main()
