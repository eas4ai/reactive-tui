# Consume queued terminal output before resizing without discarding its events

Level: Judged
Decided by: Codex
Rests on: API-011 API-019
Would be wrong if: Resize silently drops output or title and exit events, retains an unbounded event backlog, delays on a guessed quiet period, or native resize corruption remains unreported.
History: Native run 34507954693 and a real-parser replay show that an old-screen newline parsed after height growth can leave a stale prompt under the next result. Reflow repairs the separately confirmed width-change mismatch but cannot establish the order of queued PTY output.

## Decision

Before Terminal changes the child and screen dimensions, consume a bounded batch of already queued PTY output using the old screen geometry. Keep the generated public events for the next poll_events call, preserving order and exactly-once delivery. Bound retained raw output to 64 KiB and report backpressure if repeated resizes without event polling would exceed it. Do not insert a sleep or infer completion from an idle timeout. Verify old-height scrolling with a real PTY whose output has been completely queued, then verify the raw bytes and title/exit events remain observable. Retain native immediate-resize coverage; this drain does not claim to serialize output that the child has not yet generated or delivered.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`Terminal::resize` reads queued output before changing geometry and retains its
events for polling. The retained raw output is bounded at 64 KiB; callers receive
a backpressure error when they must poll before another resize.
