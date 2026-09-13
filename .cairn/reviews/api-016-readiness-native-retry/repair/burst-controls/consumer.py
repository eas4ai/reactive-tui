#!/usr/bin/env python3
import os, select, sys, termios, time, tty
from pathlib import Path
release = Path(sys.argv[1])
saved = termios.tcgetattr(0)
try:
    tty.setraw(0)
    assert not select.select([0], [], [], 0)[0]
    print('\x1b]900;READY\x07', end='', flush=True)
    deadline = time.monotonic() + 3
    while not release.exists():
        assert time.monotonic() < deadline
        time.sleep(.001)
    initial = int(release.read_text())
    delivery = 'fully-queued' if initial == 2048 else 'streamed'
    print(f'BURST_DELIVERY {delivery} initially_queued={initial}', flush=True)
    data = bytearray()
    deadline = time.monotonic() + 2
    while len(data) < 2048 and time.monotonic() < deadline:
        if select.select([0], [], [], .05)[0]:
            data.extend(os.read(0, 2048-len(data)))
    print('COUNT', len(data), flush=True)
    assert data == b'a' * 2048
    assert not select.select([0], [], [], 0)[0]
    print('ZERO_POLL_US 0', flush=True)
finally:
    termios.tcsetattr(0, termios.TCSANOW, saved)
print('ENTRY_POINT_CLEAN_EXIT', flush=True)
