import os, runpy, sys
from pathlib import Path
out = Path(__file__).resolve().parent
Terminal = runpy.run_path(str(out/'corrected.py'))['Terminal']
program = "import sys,termios,tty; saved=termios.tcgetattr(0); tty.setraw(0); print('ENTRY_POINT_CLEAN_EXIT',flush=True); termios.tcsetattr(0,termios.TCSANOW,saved)"
t = Terminal([sys.executable, '-B', '-c', program], out/'unused-screen', out/'pty-smoke.bin', (32,8))
try:
    assert not os.get_blocking(t.master), 'master is still blocking'
    t.finish(host_modes=False)
    print('REAL_PTY_NONBLOCKING_EXIT_AND_RESTORATION_PASSED')
finally:
    t.close()
