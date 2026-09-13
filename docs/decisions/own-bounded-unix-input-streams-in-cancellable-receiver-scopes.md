# Own bounded Unix input streams in cancellable receiver scopes

Level: Judged
Decided by: Shawn and Codex
Rests on: API-019 API-020
Would be wrong if: Dropping a receiver or final terminal owner leaves its worker alive, descriptors are reused while accessible to a reader, queues grow without bound, or ordinary receiving and iteration lose data.
History: The developer approved the concrete InputReceiver return-type migration in api-019-api-020. Earlier clipboard and terminal decisions retain their scopes; no additional compatibility narrowing is selected.

## Decision

Return the approved InputReceiver from both Unix async input methods. One worker owns a separate nonblocking controlling-TTY read descriptor and performs optional event parsing before a bounded standard channel. A private Unix socket wakes poll on cancellation; a condition variable wakes a full-queue producer. The receiver owns cancellation and join, while the terminal state retains weak worker registrations and cancels/joins before restoring or closing on final drop. Preserve recv, try_recv, recv_timeout and borrowed/owned iteration with standard receive errors; custom iterator wrappers preserve cancellation ownership. Opening the reader must not alter the existing terminal descriptor flags. Retain buffered receive semantics when producers disconnect. Prove idle and loaded receiver/session cleanup, clone lifetime, bounded backpressure, descriptor reuse and iterator behavior through real private PTYs and focused unit checks.

## Realized by

(none yet: recorded, not built)
