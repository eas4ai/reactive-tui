# Registry concurrency review

commit: 7bf07fa8fafae3fa9e2f2c8d3624e2e028bed374
findings:
  - resolved: REG-001 instance mutation and metrics no longer nest their locks; bounded concurrent progress and final counts pass, and restored inversion fails.
  - resolved: REG-002 constructors and lifecycle cleanup run outside registry map/statistics guards; removed entries unmount once and the unused strong ownership cycle is gone.
  - resolved: REG-003 global-count assertions isolate their whole transactions while separate workers exercise real concurrency; all inherited requirements have current passing evidence.
  - resolved: inherited renderer resize probe now observes completed cell screens and new dimensions; stale frames and partial cursor digits are rejected by focused tests.
  - resolved: unrelated name-cache identity and clone invalidation findings are captured in the backlog, as excluded by this commitment's boundaries.

## Baseline investigation

The unchanged simple_performance_test binary hung on repetition 10 with two
test threads and a three-second deadline. Output stopped after initial metrics.
Source has opposite lock acquisition: register/unregister hold active_instances
then performance_stats; performance_metrics holds performance_stats then calls
active_count. Cleanup also drops user components under the active map lock.
Mechanism demonstrations and the final review are recorded below.

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

The final formal review follows the committed acceptance evidence below.

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

## Final review

Reviewed the committed registry and tracked-instance diff, creation paths,
all instance-removal paths, the bounded test harness, lifecycle probes,
shared screen interpreter, predicate tests, mechanism declarations and latest
receipts. No production code changed during this review.

Lock lifetime attack: the only remaining paired map acquisition in the changed
paths is factories then names, matching registration order. Instance lookup,
insertion and removal finish their map guard statements before statistics or
user code runs. Debug cloning retains the individual tracked instance briefly,
but not the registry map. Orphan removal collects detached entries under the
map guard and drops them afterward. Bulk cleanup takes the current map snapshot
before disposal. Reentrant work created after detachment belongs to a later
snapshot; cleanup does not repeatedly sweep callbacks' new registrations.

Ownership attack: the registry owns tracked entries; entries no longer retain
a clone that owns the same map. Public constructor arguments remain compatible.
A cloned registry keeps shared entries alive until its last owner drops. A
concurrent debug lookup can temporarily retain an entry that was removed;
removed-registration counts are documented accordingly. Metrics are observations
across concurrent operations, not an atomic multi-field transaction. The final
post-join counters and repeated removal behavior are verified. User callbacks
that block indefinitely, panic during destruction, or create their own strong
reference cycles are not made safe by this registry locking repair.

Probe attack: one-cell updates must change the reconstructed counter, and a
resize must add its requested bottom-right coordinate after the previous read
boundary. Incomplete escape bytes and historical text cannot satisfy the
predicate. The extracted interpreter is the existing bounded ASCII helper;
Unicode remains covered by the Rust renderer tests. Controlled old predicates
fail focused tests, and real PTYs verify resize plus normal/error/panic exit
restoration. Historical failures are retained, followed by current passes.

## Production self-audit

1. Outcome and affected call paths were mapped before editing; the original
   hang and cleanup ownership failures were reproduced.
2. Changes stay within registry cleanup and its inherited failing check;
   unrelated cache findings were captured without implementation.
3. Factory lookup and CSS registration reuse existing paths; disposal has one
   helper with explicit guard lifetimes and no new synchronization layer.
4. Public registry and tracked-instance signatures are preserved; observable
   concurrent metric/removal semantics are documented.
5. Changed lock failures return contextual errors; component callbacks run
   without poisoning a held registry map on their behalf. No secrets are read.
6. No new unsafe code, network boundary, dependency, shell interpolation of
   external data, authentication or deserialization path was introduced.
7. Concurrent replacement, repeated removal, snapshot cleanup and last-owner
   release have direct coverage; there is no persistence or migration change.
8. New waits exist only in bounded test watchdogs. Production cleanup releases
   locks before user work and retains no new queues, retries or ownership cycle.
9. Discovery, implementation/verification and acceptance/review were tracked;
   completion follows passing checks and this review.
10. Twelve registry regressions and both legacy performance tests pass 25
    repeated runs per target, with 15 component and 11 builder tests passing.
    Clippy and formatting checks ran; static-tool limitations are stated above.
11. REG-001..003, WAK-001..005, EMB-001..006 and RND-001..006 have current
    passing committed receipts. No full legacy-suite or direct-host claim is made.
12. The confirmed commitment was followed autonomously; observed inherited
    probe failure and its repair were reported during the work.
13. Lock ordering, drop ownership, reentry, check falsifiers and evidence were
    reviewed. No unresolved finding remains within the agreed scope.
14. Comments and records explain the behavior and limits in plain terms.
