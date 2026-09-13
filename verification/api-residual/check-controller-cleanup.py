import ctypes
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import time

root = Path(__file__).resolve().parents[2]
if sys.platform != 'linux':
    raise SystemExit('This controller cancellation diagnostic requires Linux pidfds and subreaping')
controller = Path(sys.argv[1]) if len(sys.argv) > 1 else root / 'verification/api-residual/input-lifecycle.py'
libc = ctypes.CDLL(None, use_errno=True)
assert libc.prctl(36, 1, 0, 0, 0) == 0  # PR_SET_CHILD_SUBREAPER: this test process only
with tempfile.TemporaryDirectory(prefix='rtui-probe-cancellation-') as directory:
    parent = subprocess.Popen(['python3', '-B', str(controller),
                               str(root / 'target/api019-input-lifecycle-probe'), str(Path(directory) / 'cases'),
                               '--controller-hold'],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    child = None
    child_handle = None
    try:
        assert select.select([parent.stdout], [], [], 5)[0], 'controller did not report child startup'
        started = json.loads(parent.stdout.readline())
        assert started['started'] == 'controller-hold'
        candidate = started['pid']
        handle = os.pidfd_open(candidate)
        try:
            # Validate ownership after opening a stable PID handle. Cleanup never
            # sends a signal to an unverified/reused numeric grandchild PID.
            status_text = Path(f'/proc/{candidate}/status').read_text()
            assert f'PPid:\t{parent.pid}\n' in status_text, 'probe is not owned by the controller'
            child_handle, child = handle, candidate
        except BaseException:
            os.close(handle)
            raise
        deadline = time.monotonic() + 5
        probe_log = Path(directory) / 'cases/controller-hold.out'
        while 'CONTROLLER_HOLD_READY' not in probe_log.read_text():
            assert parent.poll() is None, 'controller exited before the held probe was ready'
            assert time.monotonic() < deadline, 'held probe did not become ready'
            time.sleep(0.01)
        parent.terminate()
        parent_status = parent.wait(timeout=5)
        deadline = time.monotonic() + 5
        while True:
            waited, status = os.waitpid(child, os.WNOHANG)
            if waited:
                child = None
                break
            assert time.monotonic() < deadline, 'private probe survived controller termination'
            time.sleep(0.01)
        assert os.WIFSIGNALED(status) and os.WTERMSIG(status) == signal.SIGKILL, status
        print(json.dumps({'controller_exit': parent_status, 'probe_pid': started['pid'],
                          'probe_signal': os.WTERMSIG(status), 'probe_reaped': True}))
    finally:
        if parent.poll() is None:
            parent.kill()
            parent.wait()
        if child is not None:
            try:
                signal.pidfd_send_signal(child_handle, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                os.waitpid(child, 0)
            except ChildProcessError:
                pass  # The controller may already have reaped it before exiting.
        if child_handle is not None:
            os.close(child_handle)
