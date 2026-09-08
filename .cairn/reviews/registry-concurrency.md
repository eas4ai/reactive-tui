# Registry concurrency review

## Baseline investigation

The unchanged simple_performance_test binary hung on repetition 10 with two
test threads and a three-second deadline. Output stopped after initial metrics.
Source has opposite lock acquisition: register/unregister hold active_instances
then performance_stats; performance_metrics holds performance_stats then calls
active_count. Cleanup also drops user components under the active map lock.
Final mechanism demonstrations and independent review remain pending.

## Development verification and mechanism demonstrations

The new suite initially failed all ten tests against unchanged registry code:
nine bounded operations timed out; dropping an owning registry reported zero
unmounts instead of one. After the fix those ten passed. Two further tests
cover orphan preservation/idempotent removal and last-clone ownership.
Both regression binaries then passed 25 runs with four test-harness threads.
The metrics regression itself starts four simultaneous workers: metrics reads,
bulk cleanup, and two registration/removal loops, each with 1000 operations.
After joining, it requires zero active instances and 2000 creates/removals.
Lifecycle tests query metrics and registration names and remove an absent key
from callbacks; constructor tests register another component through every
creation path, including debug cloning. Each workload has a five-second
watchdog; the repeated runner also enforces a 30-second process deadline.
The two legacy global-metrics tests serialize their complete transactions,
not individual API calls. The separate stress workload remains concurrent.

Three temporary controlled mutations were each detected with runtime exit 101:
restoring register/metrics lock inversion hit the concurrent progress timeout;
calling constructors under the factory lock hit the reentry timeout; restoring
the tracked instance's strong registry field failed the ownership assertion.
Every mutation was restored. A debugger attach to the original hanging binary
was denied by the environment, so no stack trace is claimed; the source lock
cycle, repeated hangs and controlled inversion test establish the diagnosis.

Existing component and macro integration targets passed all 15 tests. The
inherited App, native terminal and renderer development gate passed. Clippy
exited zero with existing library warnings and no diagnostics in changed paths.
Ripwire's qualified performance_metrics contract check passed. Quality delta
returned exit 2 for two normalized-code duplication pairs (a short map read
versus map removal, and factory lookup versus name enumeration). These perform
different operations; a shared abstraction would hide lock lifetimes for no
useful reuse. A minor register line-count change is rustfmt expansion. The
static test gate returned exit 4 and names broad, dynamically connected legacy
widget paths; it does not model the new integration tests outside src. Its
named builder tests and relevant component/inherited suites are run directly.
These static outputs are reviewed limitations, not reported as passing gates.

Final formal review remains pending until committed acceptance evidence.

## Inherited acceptance finding

The first committed WAK run failed the renderer PTY resize predicate after all
Rust and earlier PTY tests passed. Captured output ends with a completed frame
and a sparse one-cell update to 1. The probe strips ANSI and accepts any digit 1
after a sync marker for input; a partially received cursor sequence can satisfy
that predicate prematurely. Its resize check also expects contiguous Count: 1
bytes, although sparse rendering can send the label and digit separately.
Repair the inherited probe to reconstruct the screen from completed frames
and require the new dimensions' bottom-right cell after each resize. This is
inside its existing declared scripts footprint. Keep input/resize/restoration
requirements unchanged. Historical failing receipts remain intact.

The corrected probe shares the existing ASCII cell interpreter with the
embedded probe and truncates input to its last completed frame. Four focused
predicate tests cover partial cursor digits, split sparse updates, stale resize
coordinates and cleared screens. Restoring the old byte predicates fails two
of those tests. The corrected predicates pass all four, five complete real-PTY
runs (20 exit/resize scenarios), and the complete inherited development gate.
