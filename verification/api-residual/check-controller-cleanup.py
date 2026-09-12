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
controller = Path(sys.argv[1]) if len(sys.argv) > 1 else root / 'verification/api-residual/input-lifecycle.py'
libc = ctypes.CDLL(None, use_errno=True)
assert libc.prctl(36, 1, 0, 0, 0) == 0  # PR_SET_CHILD_SUBREAPER: this test process only
with tempfile.TemporaryDirectory(prefix='rtui-probe-cancellation-') as directory:
    parent = subprocess.Popen(['python3', '-B', str(controller),
                               str(root / 'target/api019-input-lifecycle-probe'), str(Path(directory) / 'cases')],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    child = None
    try:
        assert select.select([parent.stdout], [], [], 5)[0], 'controller did not report child startup'
        started = json.loads(parent.stdout.readline())
        child = started['pid']
        assert started['started'] == 'raw-receiver-idle'
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
                os.kill(child, signal.SIGKILL)
            except ProcessLookupError:
                pass
            os.waitpid(child, 0)
