#!/usr/bin/env python3
"""Drive the actual embedded_shell example through a controlling host PTY."""
import errno
import fcntl
import importlib.util
import os
import re
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import termios

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location("renderer_pty", Path(__file__).with_name("check-renderer-pty.py"))
helpers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helpers)


# The renderer emits sparse cells, so stripping ANSI cannot recover a screen:
# unchanged spaces and letters are deliberately absent. This ASCII-only probe
# follows cursor positioning; Rust tests independently check Unicode via vt100.
def screen_text(data):
    cells = {}
    x = y = 0
    tokens = re.finditer(rb"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|[^\x1b]", bytes(data))
    for match in tokens:
        token = match.group()
        if token.startswith(b"\x1b["):
            params, command = token[2:-1], token[-1:]
            if command in (b"H", b"f"):
                numbers = params.split(b";")
                y = int(numbers[0] or b"1") - 1
                x = int(numbers[1] or b"1") - 1 if len(numbers) > 1 else 0
            elif command == b"J":
                if params in (b"2",b"3"):
                    cells.clear()
                elif params in (b"",b"0"):
                    cells = {p:c for p,c in cells.items() if p < (y,x)}
            continue
        if token.startswith(b"\x1b]"):
            continue
        if token == b"\r": x = 0
        elif token == b"\n": y += 1
        elif token == b"\b": x = max(0,x-1)
        elif len(token) == 1 and 32 <= token[0] < 127:
            if 0 <= x < 200 and 0 <= y < 100: cells[y,x] = chr(token[0])
            x += 1
    return "\n".join("".join(cells.get((row,col)," ") for col in range(200)) for row in range(100))


def child_setup():
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def assert_reaped(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return
    raise AssertionError(f"owned shell {pid} survived host exit")


def probe(binary, failure=False):
    master, slave = os.openpty()
    helpers.resize(slave, 70, 16)
    original = termios.tcgetattr(slave)
    output = bytearray()
    process = None
    with tempfile.TemporaryDirectory(prefix="reactive-shell-") as directory:
        pidfile = Path(directory) / "child.pid"
        env = {**os.environ, "TERM": "xterm-256color", "PS1": "RTUI> ", "RTUI_PID_FILE": str(pidfile), "RUST_BACKTRACE": "0"}
        if failure:
            # Each bounded input query requests a cursor-position response. A
            # child that produces replies faster than they can be written must
            # receive a visible resource error, not unbounded buffering.
            code = "import os,time; open(os.environ['RTUI_PID_FILE'],'w').write(str(os.getpid())); os.write(1,b'\\x1b[6n'*20000); time.sleep(30)"
            argv = [str(binary), "python3", "-u", "-c", code]
        else:
            argv = [str(binary), "/bin/sh", "-i"]
        try:
            process = subprocess.Popen(argv, stdin=slave, stdout=slave, stderr=slave, env=env, preexec_fn=child_setup)
            def expect(marker, offset=0):
                helpers.read_until(master, process, output,
                    lambda data: len(data) > offset and marker.decode() in screen_text(data),
                    f"embedded shell did not produce {marker!r}")
            def command(data, marker):
                offset = len(output)
                os.write(master, data + b"\n")
                expect(marker, offset)
            if not failure:
                expect(b"RTUI>")
                assert termios.tcgetattr(slave) != original, "host never entered raw mode"
                command(b"echo $$ > \"$RTUI_PID_FILE\"; printf '\\nRTUI_%s\\n' INPUT", b"RTUI_INPUT")
                command(b"sleep 0.2; printf '\\nRTUI_%s\\n' DELAYED", b"RTUI_DELAYED")
                for columns, rows in [(52,12),(76,18)]:
                    offset = len(output)
                    helpers.resize(slave, columns, rows)
                    os.kill(process.pid, signal.SIGWINCH)
                    last_cell = f"\x1b[{rows};{columns}H".encode()
                    helpers.read_until(master, process, output,
                        lambda data: last_cell in data[offset:] and helpers.SYNC_END in data[offset:],
                        "App did not finish the resized frame")
                    command(b'set -- $(stty size); printf "SIZE_%s_%s" "$1" "$2"', f"SIZE_{rows}_{columns}".encode())
                offset = len(output)
                os.write(master, b"sleep 30\n")
                expect(b"sleep 30", offset)
                os.write(master, b"\x03")
                command(b"printf '\\nRTUI_%s\\n' INTERRUPTED", b"RTUI_INTERRUPTED")
                assert process.poll() is None, "Ctrl+C quit the host"
                os.write(master, b"\x11")
            helpers.read_until(master, process, output, lambda data: b"\x1b[?1049l" in data, "host alternate screen was not restored")
            status = process.wait(timeout=10)
            while helpers.select.select([master], [], [], 0)[0]:
                output.extend(os.read(master, 65536))
            assert (status != 0) == failure, f"unexpected exit {status}: {output[-1000:]!r}"
            assert termios.tcgetattr(slave) == original, "host termios was not restored"
            assert b"\x1b[?25h" in output, "host cursor was not restored"
            assert_reaped(int(pidfile.read_text().strip()))
            if failure:
                assert b"pending input exceeded 64 KiB" in output, f"expected worker error missing: {output[-1000:]!r}"
            print("PASS EMB App PTY:", "worker error + cleanup" if failure else "input + background redraw + resize + Ctrl+C + Ctrl+Q + cleanup")
        except Exception:
            print("Captured host screen:", "\n".join(line.rstrip() for line in screen_text(output).splitlines()).rstrip())
            raise
        finally:
            if process is not None and process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            os.close(master)
            os.close(slave)


if __name__ == "__main__":
    binary = (Path(os.environ.get("CARGO_TARGET_DIR", "target")) / "debug/examples/embedded_shell").resolve()
    probe(binary)
    probe(binary, failure=True)
