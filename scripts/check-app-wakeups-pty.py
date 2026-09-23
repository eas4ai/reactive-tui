#!/usr/bin/env python3
"""Check that a background signal wakes the real input wait without a key."""
import fcntl
import importlib.util
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import termios
import time

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("renderer_probe", Path(__file__).with_name("check-renderer-pty.py"))
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
screen_spec = importlib.util.spec_from_file_location("screen_probe", Path(__file__).with_name("check-embedded-terminal-pty.py"))
screen_probe = importlib.util.module_from_spec(screen_spec)
screen_spec.loader.exec_module(screen_probe)


def completed(data):
    end = data.rfind(probe.SYNC_END)
    return bytes(data[:end + len(probe.SYNC_END)]) if end >= 0 else b""


def shows(data, text):
    return text in screen_probe.screen_text(completed(data))



def controlling_terminal():
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def main():
    binary = (Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug/examples/app_wakeup_probe").resolve()
    master, slave = os.openpty()
    probe.resize(slave, 40, 8)
    original = termios.tcgetattr(slave)
    process = None
    output = bytearray()
    try:
        process = subprocess.Popen([str(binary), "--probe"], stdin=slave, stdout=slave, stderr=slave,
                                   preexec_fn=controlling_terminal,
                                   env={**os.environ, "TERM": "xterm-256color"})
        probe.read_until(master, process, output,
                         lambda data: shows(data, "Wake count: 0"),
                         "initial wake counter frame missing")
        offset = len(output)
        probe.read_until(master, process, output,
                         lambda data: shows(data, "Wake count: 1"),
                         "background signal did not interrupt the host input wait")
        # The worker is now finished, and this root has no polling or timer deadline.
        # There must be no further frame output until a real resize/key arrives.
        time.sleep(0.1)
        while select.select([master], [], [], 0)[0]:
            output.extend(os.read(master, 65536))
        offset = len(output)
        if select.select([master], [], [], 0.15)[0]:
            unexpected = os.read(master, 65536)
            raise AssertionError(f"idle App kept writing output: {unexpected!r}")
        probe.resize(slave, 44, 10)
        os.kill(process.pid, signal.SIGWINCH)
        probe.read_until(master, process, output,
                         lambda data: b"\x1b[10;44H" in completed(data)[offset:] and shows(data, "Wake count: 1"),
                         "resize did not interrupt the idle wait")
        os.write(master, b"\x1b")
        probe.read_until(master, process, output, lambda data: b"\x1b[?1049l" in data,
                         "quit did not restore the terminal")
        assert process.wait(timeout=5) == 0
        assert termios.tcgetattr(slave) == original, "termios was not restored"
        print("PASS WAK-001/002/004 real PTY: background signal, idle output, resize, quit and restoration")
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    main()
