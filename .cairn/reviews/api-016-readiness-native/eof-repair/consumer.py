import os, runpy, sys, termios, types, tty
from pathlib import Path
source, case, capture = sys.argv[1:]
Terminal = runpy.run_path(source)['Terminal']
t = Terminal.__new__(Terminal)
t.master, writer = os.pipe()
os.set_blocking(t.master, False)
spare, t.slave = os.openpty()
t.original = termios.tcgetattr(t.slave)
t.child = types.SimpleNamespace(poll=lambda: 0, returncode=0)
t.output = bytearray()
t.capture = Path(capture)
os.write(writer, b'ENTRY_POINT_CLEAN_EXIT\x1b[?1049l\x1b[?25h')
os.close(writer)
try:
    if case == 'raw-leak':
        tty.setraw(t.slave)
    if case == 'never-drained':
        t.read = lambda timeout=0: True
    try:
        t.finish()
    except AssertionError as error:
        if case == 'raw-leak' and 'left raw mode active' in str(error):
            print('RAW_RESTORATION_REJECTED')
        elif case == 'never-drained' and 'output did not stop after exit' in str(error):
            print('DRAIN_DEADLINE_REJECTED')
        else:
            raise
    else:
        assert case == 'eof', 'violating fixture unexpectedly passed'
        assert t.capture.read_bytes() == t.output
        assert b'ENTRY_POINT_CLEAN_EXIT' in t.output
        print('EOF_DRAIN_RETURNED_AND_RESTORATION_PASSED')
finally:
    termios.tcsetattr(t.slave, termios.TCSANOW, t.original)
    os.close(t.master)
    os.close(t.slave)
    os.close(spare)
