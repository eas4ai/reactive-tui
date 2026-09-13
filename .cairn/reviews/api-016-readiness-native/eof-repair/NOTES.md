# Prepared EOF drain repair (not applied)

`terminal-eof.patch` changes only Terminal initialization, read progress reporting, and its post-exit drain. It makes the PTY master nonblocking; read returns False on no readiness, EOF, or BlockingIOError; finish stops on no progress and rejects a drain exceeding two seconds. Exit-code, success marker, termios, alternate-screen, cursor, output cap and capture assertions remain intact.

`python3 -B /tmp/api016-eof-repair-53y_utem/run-probe.py` ran four bounded subprocess probes. Original source hung on real pipe EOF and was killed/reaped after three seconds. Corrected source consumed captured data then returned on EOF and passed restoration/host-mode assertions. A real raw-termios violation remained rejected. A controlled never-finished drain was rejected after two seconds. Results and exact commands are in results.json, with per-case output/captures beside it.

`pty-smoke.py` additionally passed with a real controlling PTY child that entered raw mode, emitted the success marker, restored mode, and exited; it verifies the actual Terminal constructor makes the master nonblocking. Its output and terminal capture are retained. These local probes establish the harness bug and repair; they do not establish that macOS CI encountered this particular EOF condition.

No tracked source was modified and no builds ran.
