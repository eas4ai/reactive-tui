# Make threaded event-loop queues cancellable and bounded

Level: Judged
Decided by: Codex
Rests on: RTR-002
Would be wrong if: Queue saturation loses accepted events or ordering, stop cannot wake every blocking state, or restart reuses a stopped queue incorrectly.

## Decision

Put the 512-event queue behind one mutex and condition variables. An input producer waits for space through the condition variable, which releases the queue mutex so the consumer can drain it. A caller posting directly to a full queue receives a capacity error because waiting on the same mutable EventLoop would prevent that caller from consuming. On Unix, reuse the owned socket-pair cancellation path to wake blocked input. On Windows, poll the console with a bounded timeout. stop and Drop signal the queue and reader, then join the worker; start resets the stopped state without discarding buffered events.

## Realized by

(none yet: recorded, not built)
