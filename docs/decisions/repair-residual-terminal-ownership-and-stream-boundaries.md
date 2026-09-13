# Repair residual terminal ownership and stream boundaries

Level: Judged
Decided by: Codex
Rests on: API-019, API-011, escalation api-019-api-020
Would be wrong if: The repair breaks retained public entry points, calls user code in signal context, restores terminal state while another owner is live, loses bounded split input, or cannot restore prior signal ownership.
History: Earlier terminal and input reversals showed that compilation and single-owner tests do not prove descriptor, signal, callback, or terminal-mode ownership; this decision therefore requires positive and safe violating cases for concurrent owners and split reads.

## Decision

Keep the existing public terminal entry points. Deliver SIGWINCH through signal-hook's self-pipe so the signal path performs only an async-signal-safe write, invoke registered callbacks on an owned dispatcher, and unregister the library action when the final owner resets it. Count library raw-mode owners and restore only after the last owner, while preserving raw mode established by the caller. Buffer incomplete legacy terminal input behind the existing parser API with a fixed bound, and add an explicit flush for a lone Escape key. Reject oversized DebugBackend resizes before allocation and retain the previous dimensions through the infallible Backend method.

## Realized by

857f7b772ce808cb08b88629b06af35b55584295 Complete residual API behavior coverage
