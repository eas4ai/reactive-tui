# Make threaded event-loop queues cancellable and bounded

Level: Judged
Decided by: Codex
Rests on: RTR-002
Would be wrong if: Queue saturation loses accepted events or ordering, stop cannot wake every blocking state, or restart reuses a stopped queue incorrectly.

## Decision

Put the 512-event queue behind one mutex and condition variables. An input producer waits for space through the condition variable, which releases the queue mutex so the consumer can drain it. A caller posting directly to a full queue receives a capacity error because waiting on the same mutable EventLoop would prevent that caller from consuming. On Unix, reuse the owned socket-pair cancellation path to wake blocked input. On Windows, poll the console with a bounded timeout. stop and Drop signal the queue and reader, then join the worker; start resets the stopped state without discarding buffered events.

## Realized by

d6ce9949f99816dd624b173c5063e39cc18dd4cb fix: make threaded event loop cancellable

Implementation: The threaded loop now uses a condition-variable queue with explicit stopped state. Unix duplicates stdin and polls it with the existing socket-pair cancellation primitive. Windows reads with a 50 ms timeout. `stop` and `Drop` stop the queue, wake the reader, and join the worker.

Behavior check: The RTR-002 checker fills and drains the input queue through a PTY, stops an idle reader, verifies Drop restores the process thread count, and confirms direct posting reports capacity then recovers after consumption.
