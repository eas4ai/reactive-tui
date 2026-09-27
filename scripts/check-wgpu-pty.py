#!/usr/bin/env python3
"""Real PTY catalog runs; not a claim of visual verification in Kitty/Ghostty."""

import fcntl
import hashlib
import importlib.util
import os
from pathlib import Path
import select
import signal
import struct
import subprocess
import termios
import time

ROOT = Path(__file__).resolve().parents[1]
helper_spec = importlib.util.spec_from_file_location("catalog_pty", ROOT / "scripts/check-widget-catalog-pty.py")
helper = importlib.util.module_from_spec(helper_spec)
helper_spec.loader.exec_module(helper)
END_FRAME = b"\x1b[?2026l"


def check_run(executable, name, key, options=(), expected="GPU"):
    master, slave = helper.pty.openpty()
    original = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 100, 0, 0))
    process = None
    output = bytearray()
    pending = bytearray()
    screen = helper.Screen(100, 32)
    frames = set()
    resized = False
    first_time = None
    try:
        process = subprocess.Popen([str(executable), "--motion", *options], cwd=ROOT,
                                   stdin=slave, stdout=slave, stderr=slave,
                                   env={**os.environ, "TERM": "xterm-256color"},
                                   preexec_fn=helper.controlling_terminal)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise AssertionError(f"{name} exited early: {process.returncode}: {output[-1000:]!r}")
            if not select.select([master], [], [], 0.1)[0]:
                continue
            data = os.read(master, 65536)
            output.extend(data)
            pending.extend(data)
            while END_FRAME in pending:
                end = pending.index(END_FRAME) + len(END_FRAME)
                frame_bytes = bytes(pending[:end])
                del pending[:end]
                screen.feed(frame_bytes)
                text = screen.text()
                viewport = (176 * 54) if resized else (76 * 26)
                if expected not in text or text.count("▀") != viewport:
                    continue
                if expected == "GPU" and not any(brand in text for brand in ("AMD", "NVIDIA", "Intel", "Apple")):
                    raise AssertionError(f"missing real adapter identity: {text}")
                # Only complete synchronized frames with actual colored cube writes
                # count. Partial reads/loading frames cannot satisfy animation.
                if b"\x1b[38;2;" not in frame_bytes or "▀".encode() not in frame_bytes:
                    continue
                if resized:
                    print(f"GPU PTY {name} RESIZE 200x60 cells={viewport} mode={expected}", flush=True)
                    break
                now = time.monotonic()
                if first_time is None:
                    first_time = now
                if not frames or now - first_time >= 0.15:
                    frames.add(hashlib.sha256(frame_bytes).hexdigest())
                if len(frames) >= 2:
                    print(f"GPU PTY {name} ANIMATION frames={len(frames)} mode={expected}", flush=True)
                    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 60, 200, 0, 0))
                    os.killpg(process.pid, signal.SIGWINCH)
                    screen = helper.Screen(200, 60)
                    resized = True
            else:
                continue
            if resized and screen.text().count("▀") == 176 * 54 and expected in screen.text():
                break
        else:
            raise AssertionError(f"{name} failed animation/resize: {screen.text()}")
        os.write(master, key)
        deadline = time.monotonic() + 5
        while process.poll() is None and time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                output.extend(os.read(master, 65536))
        assert process.poll() == 0, f"{name} abnormal exit: {process.poll()}"
        process.wait()
        assert termios.tcgetattr(slave) == original, f"{name} terminal flags not restored"
        assert b"\x1b[?1049l" in output and b"\x1b[?25h" in output, f"{name} screen/cursor not restored"
        try:
            os.killpg(process.pid, 0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError(f"{name} child process survived")
        print(f"GPU PTY {name} EXIT 0 RESTORED CLEAN", flush=True)
    finally:
        if process is not None and process.poll() is None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


def main():
    target = Path(os.environ["CARGO_TARGET_DIR"]) if "CARGO_TARGET_DIR" in os.environ else None
    if target is None:
        import json
        metadata = subprocess.check_output(["cargo", "+1.91.0", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT)
        target = Path(json.loads(metadata)["target_directory"])
    executable = target / "debug/examples/widget_catalog"
    for name, key in (("CTRL_Q", b"\x11"), ("CTRL_C", b"\x03"), ("ESCAPE", b"\x1b")):
        check_run(executable, name, key)
    check_run(executable, "CPU", b"\x11", ("--cpu",), "CPU fallback")
    for fault in ("adapter", "device-loss", "readback"):
        check_run(executable, fault, b"\x1b", ("--graphics-fault", fault), "CPU fallback")


if __name__ == "__main__":
    main()
