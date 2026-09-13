"""Independent EOF and restoration controls for the terminal evidence harness."""
import os
from pathlib import Path
import runpy
import signal
import sys
import termios
import types
import tty

ROOT = Path(__file__).resolve().parents[2]
Terminal = runpy.run_path(str(ROOT / "scripts/check-api-entry-points.py"))["Terminal"]


def stop(_signal, _frame):
    raise RuntimeError("harness consumer watchdog expired")


signal.signal(signal.SIGTERM, stop)
case, capture = sys.argv[1:]
capture = Path(capture)
if case in ("pty", "pty-raw-leak"):
    program = ("import termios,tty; saved=termios.tcgetattr(0); tty.setraw(0); "
               "print('ENTRY_POINT_CLEAN_EXIT',flush=True); "
               "termios.tcsetattr(0,termios.TCSANOW,saved)")
    if case == "pty-raw-leak":
        program = program.replace("termios.tcsetattr(0,termios.TCSANOW,saved)", "pass")
    terminal = Terminal([sys.executable, "-B", "-c", program], None, capture, (32, 8))
    try:
        assert not os.get_blocking(terminal.master), "PTY master must be nonblocking"
        try:
            terminal.finish(host_modes=False)
        except AssertionError as error:
            if case != "pty-raw-leak" or "left raw mode active" not in str(error):
                raise
            print("REAL_PTY_RAW_RESTORATION_REJECTED")
        else:
            assert case == "pty", "real PTY raw-mode leak unexpectedly passed"
            print("REAL_PTY_NONBLOCKING_EXIT_AND_RESTORATION_PASSED")
    finally:
        terminal.close()
else:
    terminal = Terminal.__new__(Terminal)
    terminal.master, writer = os.pipe()
    os.set_blocking(terminal.master, False)
    spare, terminal.slave = os.openpty()
    terminal.original = termios.tcgetattr(terminal.slave)
    terminal.child = types.SimpleNamespace(poll=lambda: 0, returncode=0)
    terminal.output = bytearray()
    terminal.capture = capture
    os.write(writer, b"ENTRY_POINT_CLEAN_EXIT\x1b[?1049l\x1b[?25h")
    os.close(writer)
    try:
        if case == "raw-leak":
            tty.setraw(terminal.slave)
        if case == "never-drained":
            terminal.read = lambda timeout=0: True
        try:
            terminal.finish()
        except AssertionError as error:
            if case == "raw-leak" and "left raw mode active" in str(error):
                print("RAW_RESTORATION_REJECTED")
            elif case == "never-drained" and "output did not stop after exit" in str(error):
                print("DRAIN_DEADLINE_REJECTED")
            else:
                raise
        else:
            assert case == "eof", "violating harness fixture unexpectedly passed"
            assert capture.read_bytes() == terminal.output
            assert b"ENTRY_POINT_CLEAN_EXIT" in terminal.output
            print("EOF_DRAIN_RETURNED_AND_RESTORATION_PASSED")
    finally:
        capture.write_bytes(terminal.output)
        termios.tcsetattr(terminal.slave, termios.TCSANOW, terminal.original)
        os.close(terminal.master)
        os.close(terminal.slave)
        os.close(spare)
