# Preserve Unix input readiness in the pinned Crossterm dependency

Level: Judged
Decided by: Codex
Rests on: API-016 API-019 API-020
Would be wrong if: Queued input still waits for another key, zero-timeout polling blocks, or native input and terminal restoration regress.
History: Earlier API reversals exposed platform assumptions and required native lifecycle proof. This remains Judged because it preserves the existing API through a narrow dependency repair; explicit burst, readiness, timeout, and native checks must precede acceptance.

## Decision

Maintain Crossterm 0.29.0 as a narrowly patched local dependency under src/backend, retaining its license and provenance. Preserve pending Mio readiness across event returns and guard retained input reads with a nonblocking readiness check. Verify bursts beyond 1024 bytes, mixed resize/input, wake interruption, zero-timeout exhaustion, and existing Rust/C entry points. Do not switch to use-dev-tty, whose zero-timeout loop skips input. Refresh native evidence after dependency changes.

## Realized by

- 4cd4a4bc98ad6f665a840a90247431dfbf5fc6b1 Preserve queued Crossterm input across readiness returns
