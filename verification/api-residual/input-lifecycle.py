"""Run a compiled public-API probe in disposable controlling PTYs (Linux only)."""
import ctypes
import fcntl
import json
import os
from pathlib import Path
import pty
import signal
import struct
import subprocess
import sys
import termios

if sys.platform != "linux":
    raise SystemExit("This diagnostic requires Linux /proc and private PTYs")

binary = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=False)
results = []
controller_pid = os.getpid()
prctl = ctypes.CDLL(None, use_errno=True).prctl
prctl.argtypes = [ctypes.c_int, ctypes.c_ulong, ctypes.c_ulong, ctypes.c_ulong, ctypes.c_ulong]
prctl.restype = ctypes.c_int


def private_session():
    # This single-threaded controller owns the child even across its new session.
    if prctl(1, signal.SIGKILL, 0, 0, 0) != 0:  # PR_SET_PDEATHSIG
        raise OSError(ctypes.get_errno(), "Cannot bind probe lifetime to controller")
    if os.getppid() != controller_pid:
        os.kill(os.getpid(), signal.SIGKILL)
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


cases = ("raw-receiver-idle", "parsed-receiver-idle", "reused-descriptor", "raw-receiver-active",
             "raw-receiver-full", "raw-session-full", "raw-session-idle", "parsed-receiver-full",
             "parsed-session-full", "parsed-session-idle", "clone-owner", "independent-streams",
             "owned-iterator", "concurrent-drop")
if len(sys.argv) > 3:
    if sys.argv[3:] != ["--controller-hold"]:
        raise SystemExit("Unknown input controller arguments")
    cases = ("controller-hold",)
for case in cases:
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    environment = {**os.environ, "TERM": "xterm-256color", "RUST_TEST_THREADS": "8"}
    with (output / (case + ".out")).open("wb") as log:
        child = subprocess.Popen([str(binary), case, str(master)], stdin=slave,
                                 stdout=log, stderr=subprocess.STDOUT, env=environment,
                                 pass_fds=(master,), preexec_fn=private_session)
        print(json.dumps({"started": case, "pid": child.pid}), flush=True)
        timed_out = False
        try:
            status = child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            timed_out = True
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            status = child.wait()
        except BaseException:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait()
            raise
        finally:
            os.close(master)
            os.close(slave)
    result = {"case": case, "pid": child.pid, "exit": status, "timeout": timed_out,
              "reaped": child.returncode is not None}
    results.append(result)
    print(json.dumps(result), flush=True)
(output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
raise SystemExit(1 if any(row["exit"] != 0 or row["timeout"] for row in results) else 0)
