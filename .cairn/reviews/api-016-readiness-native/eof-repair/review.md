# Bound post-exit terminal evidence draining

Native run 34740336825 passed every Windows test step and the macOS clipboard,
widget, hook and dependency readiness tests. The new macOS entry-point suite
hit its 1,200-second outer deadline. Its only host capture reached state empty
clicks1 at 48x12 after input and resize. No Rust host completion line followed.
The unchanged native failure record, output hash and capture are retained under
first-failure. This is not a complete macOS entry-point pass.

A real-pipe EOF probe independently demonstrated that Terminal.finish could
spin indefinitely: read returned no bytes while select kept reporting readable
EOF. The old source was killed and reaped after three seconds. The prepared
correction completed in milliseconds and retained raw-mode and host-restoration
assertions. This establishes the harness defect; the timed-out native process
was not instrumented enough to prove its exact stalled instruction.

The master is now nonblocking, read reports actual progress, and the post-exit
drain stops at EOF/no progress or fails after two seconds. All prior success,
exit-code, raw-mode, cursor and alternate-screen assertions remain. Four bounded
permanent guard cases now precede Cargo, including a real controlling PTY and
negative restoration/endless-output controls. Each subprocess has a five-second
watchdog with bounded cleanup. All four guards and the complete local entry-point
suite passed; raw output is full-entry-points.out and captures are under
api-entry-points/20260913T055857Z. Native verification must be repeated.

Self-audit: this routine mechanism repair changes no product API, preserves the
acceptance criteria, makes timeouts explicit, retains all failing evidence, and
has independent violating/corrected controls. Native acceptance remains open.
