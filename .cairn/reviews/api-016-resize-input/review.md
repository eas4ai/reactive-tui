# API-016 resize/input investigation

The formal receipt 20260913T050434756Z failed one Rust Crossterm 32x8
workflow after resize. The child remained live and painted the new dimensions,
but the subsequent e key did not remove content within eight seconds.
All other workflows passed. No failing receipt or capture was altered.

Twelve unchanged-source repeats passed (20260913T050651968517Z).
A second bounded diagnostic stopped at its first failure: case 28 in
20260913T051246757477Z, after 28 passes. On that failure, the diagnostic
saved the pre-recovery bytes and sent z. The original e then took effect.
Recovery is diagnostic only; the case remains failed in results.json.

Read-only review found that App retains dirty state after handled input.
Crossterm 0.29.0 unix/mio.rs returns from a readiness batch immediately
on a resize or parsed event and replaces that batch on its next poll.
An unread TTY edge can therefore be lost. SuprTUI's synthetic size event
can paint dimensions before the pending SIGWINCH is consumed.
The reproduced recovery supports this explanation; readiness order has
not yet been instrumented or forced deterministically.

No production change has been made. Next: demonstrate the readiness loss
with a controlled consumer, then repair the dependency boundary without
weakening the input or nonblocking contract.

The independent burst consumer in burst-baseline/ initializes Crossterm,
queues 2,048 a bytes before consumption, and waits up to two seconds.
The unchanged dependency returned COUNT 1024 and exit 1. The captured
consumer and terminal bytes retain this defect-confirming baseline.
The consumer has no App rendering or performance context.

The alternative use-dev-tty implementation skips its read loop for zero
timeout, including parser events. Switching it on would risk the existing
nonblocking contract. The intended repair retains readiness across returns
and checks readable input without blocking before reading retained work.
