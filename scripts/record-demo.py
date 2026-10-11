"""Scripted demo recording: drive the showcase examples in a PTY and write one
asciicast v2 file. Portable: launches examples via `cargo run`, so it works
regardless of CARGO_TARGET_DIR.

Usage:
    python3 scripts/record-demo.py demo.cast
    agg --font-family "DejaVu Sans Mono" demo.cast demo.gif

The font flag matters: agg's default font renders U+2800 blank Braille as
dotted tofu, which looks like a framework bug and is not one.
"""
import json
import os
import pty
import select
import subprocess
import sys
import termios
import time
import fcntl
import struct

WIDTH, HEIGHT = 110, 30
TAB, CTRL_Q = b"\t", b"\x11"
RIGHT = b"\x1b[C"

# (example name, [(delay_before_send, bytes_to_send), ...], settle_after_last)
# First delay in each segment is generous: it covers cargo startup.
PLAN = [
    ("animation_showcase",
     [(6.0, TAB), (3.0, TAB), (3.0, TAB), (3.0, TAB), (3.0, TAB), (3.0, CTRL_Q)],
     1.0),
    ("layout_showcase",
     [(6.0, TAB), (2.5, TAB), (2.5, TAB), (2.5, TAB), (2.5, CTRL_Q)],
     1.0),
    ("widget_catalog",
     [(6.0, RIGHT), (2.0, RIGHT), (2.0, RIGHT), (2.0, RIGHT), (2.0, CTRL_Q)],
     1.0),
]

events = []
t0 = time.monotonic()


def stamp():
    return round(time.monotonic() - t0, 6)


def run_segment(example, script, settle):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", HEIGHT, WIDTH, 0, 0))
    env = dict(os.environ, TERM="xterm-256color",
               COLUMNS=str(WIDTH), LINES=str(HEIGHT))
    proc = subprocess.Popen(
        ["cargo", "run", "-q", "--example", example],
        stdin=slave, stdout=slave, stderr=slave,
        env=env, close_fds=True,
    )
    os.close(slave)
    os.set_blocking(master, False)
    for delay, keys in script:
        deadline = time.monotonic() + delay
        while time.monotonic() < deadline:
            r, _, _ = select.select([master], [], [], 0.05)
            if r:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    chunk = b""
                if chunk:
                    events.append([stamp(), "o",
                                   chunk.decode("utf-8", errors="replace")])
                else:
                    break
        try:
            os.write(master, keys)
        except OSError:
            break
        if proc.poll() is not None:
            break
    end = time.monotonic() + settle
    while time.monotonic() < end:
        r, _, _ = select.select([master], [], [], 0.1)
        if r:
            try:
                chunk = os.read(master, 65536)
            except OSError:
                break
            if chunk:
                events.append([stamp(), "o",
                               chunk.decode("utf-8", errors="replace")])
            else:
                break
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
    os.close(master)
    print(f"{example}: exit={proc.returncode} events={len(events)}", flush=True)


for example, script, settle in PLAN:
    run_segment(example, script, settle)
    time.sleep(0.5)

out = sys.argv[1] if len(sys.argv) > 1 else "demo.cast"
with open(out, "w") as f:
    f.write(json.dumps({"version": 2, "width": WIDTH, "height": HEIGHT,
                        "timestamp": int(time.time()),
                        "env": {"TERM": "xterm-256color", "SHELL": "/bin/bash"}}) + "\n")
    for ev in events:
        f.write(json.dumps(ev) + "\n")
print(f"wrote {out}: {len(events)} events, "
      f"{os.path.getsize(out)/1e6:.1f} MB", flush=True)
