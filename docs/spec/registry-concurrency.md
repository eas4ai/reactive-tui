# Component registry concurrency

Status: Agreed 2026-09-07
Prefix: REG

The developer confirmed the registry stall as the next commitment.
The existing parallel performance tests have reproduced a bounded-timeout
failure. This work covers registry locking, cleanup and reliable tests.

[REG-001]
Registry instance registration, removal, cleanup and performance reads MUST complete under concurrent use without a registry lock cycle.
Falsifier: bounded concurrent workloads or the parallel performance test target hang, or final active counts disagree with completed operations.
Mechanism: scripts/check-registry-concurrency.sh.

[REG-002]
Registry construction and cleanup paths MUST invoke component constructors and lifecycle callbacks outside registry map and statistics locks.
Removal, replacement and bulk cleanup MUST unmount each removed instance once and release its ownership.
Falsifier: a callback that queries or updates the registry deadlocks, a removed instance unmounts twice, or registry ownership keeps removed instances alive.
Mechanism: scripts/check-registry-concurrency.sh, bounded reentry and cleanup tests.

[REG-003]
Performance tests MUST isolate their whole-registry assertions while regression tests exercise actual concurrent registry operations.
Existing App wakeup, renderer and embedded-terminal requirements MUST retain passing evidence.
Falsifier: concurrent test setup clears another test's state, concurrency coverage is replaced by serial-only checks, or an inherited requirement fails.
Mechanism: scripts/check-registry-concurrency.sh and inherited gates.
