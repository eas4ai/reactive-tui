# Separate Windows process initialization from PTY handle leak measurements

Level: Judged
Decided by: agent
Rests on: API-011 EMB-006 API-020
Would be wrong if: The revised probe misses per-launch handle growth or excludes a Reactive-TUI cleanup defect instead of independent Windows initialization.
History: Prior ConPTY reversals required direct OS and SDK controls before changing ownership assumptions. That history keeps this Judged and requires independent initialization evidence plus a real violating fixture; no library cleanup behavior or acceptance allowance is relaxed.

## Decision

On the developer Windows 11 machine, the first missing-executable PTY attempt raises handles once and nine further attempts remain stable. An independent Rust program with no Reactive-TUI code gains 46 handles on its first ordinary missing-executable launch, then raw OS and SDK ConPTY cycles remain stable after initial setup. Warm only the standard-library missing-process path before the existing PTY baseline. Keep the same ten PTY failure attempts and the same final allowance of two handles. Demonstrate that a deliberate per-attempt leaked file handle still fails, and that the corrected case passes.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`tests/api_widget_behavior/conpty_probe.rs` initializes an ordinary missing
process launch before taking the PTY handle baseline. On Windows 11/MSVC the
corrected probe remains at exactly 135 handles through all ten failed launches.
An isolated deliberate leak rises from 135 to 145 and fails the original
allowance. After restoring the source, the complete probe passes again, including
resize, exit status, restart, backpressure and descendant cleanup. Evidence is in
`.cairn/reviews/api-011-native-windows11/conpty-{warmed,leak-negative,corrected}.*`.
The independent reference program and output are
`.cairn/reviews/api-011-windows11-raw-handle-reference.{rs,log}`.
