# Registry cache isolation review

commit: 9334086cfedffa7de3a9c17d156a24c415482c25
findings:
  - resolved: CCH-001 named lookup reads only the requested registry map; equal-count, alternating-name/type and registry-lifetime regressions pass after failing the original cache.
  - resolved: CCH-002 clones share the authoritative map, including after clear and cross-thread publication; constructor reentry and all inherited requirements retain passing evidence.
  - resolved: the fresh full-suite assessment is committed with exact commands and output; its six legacy failures and FFI/format/lint debt are explicitly open outside this repair.

## Scope and mechanism plan

Exercise independent registries with equal registration counts and different
name/type assignments on one thread. Exercise clones made before registration,
clones mutated after lookup, clear/re-register and cross-thread publication.
Use real type-erased instances and check their TypeId, not a cache counter.
Keep the existing constructor reentry and registry concurrency acceptance.
Record the fresh full-suite assessment separately from passing requirements.

## Failure demonstration and development checks

All seven new tests failed against the original implementation with runtime
exit 101: wrong TypeId in equal-count registries, foreign names, stale names
after clear/re-register, missing names in replacement registries, preexisting
clones, warmed clones and the acknowledged cross-thread reader. They assert
public construction results; no cache implementation detail supplies a pass.
The corrected implementation passes all seven. The inherited registry suite
also passes its repeated concurrent runs and lifecycle reentry checks.

The repair removes NameCache, its thread-local storage and all version counters.
One map read obtains an owned TypeId and ends its guard before factory lookup.
The public create_by_name contract remains unchanged according to Ripwire.
No snapshot, per-thread map, weak identity token, generation counter or new
helper is introduced. Named lookup now pays a short read-lock acquisition;
no throughput improvement or contention benchmark is claimed.

Ripwire quality delta exits 2: removing the cache's HashMap::extend call makes
its name-based graph mark unrelated ChangeBatch::extend dead; the simpler Arc
initialization/clone implementations resemble unrelated event-loop/signal code
under token normalization. Those types do not share behavior worth abstracting.
The static test gate exits 4 and names broad widget paths; it does not model
all external integration tests. Its named builder tests run in the full suite.
Neither static output is reported as passing. Focused rustfmt/diff checks pass;
targeted Clippy exits zero with existing warnings and no changed-path diagnostic.

A fresh full-suite assessment is recorded in docs/recon-evidence/cache-isolation.
After avoiding an environmental temporary-disk quota, the suite reports 1005
passed, six existing failures and 35 ignored tests. FFI compile/link failures,
formatting debt and strict-Clippy errors are recorded without weakening gates.
Committed acceptance and the final formal review are recorded below.

The inherited development gate initially encountered a compiler internal error
in the embedded feature build. Clearing this crate's generated incremental
cache and rebuilding unchanged source produced a complete passing gate.
The raw diagnostic and recovery are included in the assessment.

## Final review

Reviewed the committed source diff, all seven public-API regressions, the
mechanism declaration, decision, assessment metadata and current receipts.
No production or test code changed during this review.

Isolation attack: names and factories belong to each independent registry.
A clone shares those same maps; there is no second snapshot to invalidate,
no counter to copy, and no thread-local state that can survive a registry.
Tests deliberately keep both component types registered while swapping names,
so checking for Some alone cannot conceal a wrong-type lookup. Distinct names
and sequentially dropped registries independently detect name leakage.

Ordering attack: name lookup copies TypeId and ends the map guard at the
statement boundary. Factory lookup and user construction follow without that
guard. Registration and clearing still acquire factories then names; removing
the cache counter adds no reverse acquisition. The cross-thread test waits for
an explicit acknowledgement of completed mutation rather than relying on a
sleep. Concurrent mutation that overlaps lookup can still affect its result;
this commitment guarantees visibility of completed operations, not a combined
transaction across independently called APIs.

Ownership and cost: no retained snapshot or identity allocation remains.
Cloning and cleanup ownership are unchanged from the prior repaired registry.
Direct lookup adds a short shared read lock; performance under heavy contention
has not been benchmarked. Future optimization needs measurement and must retain
these same isolation and reentry tests.

Evidence attack: the original seven failures demonstrate the mechanism rejects
the violating implementation. Current CCH, REG, WAK, EMB and RND receipts pass.
The assessment's recorded source hash matches the committed source, and its
aggregate was independently recomputed as 1005 passed, six failed, 35 ignored.
The initial quota failures and compiler internal error remain visible alongside
the successful environmental recovery. The full legacy suite, FFI, formatting
and strict lint are not represented as clean.

## Production self-audit

1. The requested behavior, cache ownership and lookup callers were mapped first.
2. The production change removes cache machinery in one file; no unrelated fix.
3. One authoritative map replaces hidden per-thread state and copied versions.
4. Public APIs, duplicate-registration behavior and constructor reentry remain.
5. The map lock failure has context; no errors or test failures are hidden.
6. No new dependency, unsafe code, external-data boundary or secret handling.
7. Completed mutations through clones, clearing and re-registration are tested.
8. No new queues, retries, retained maps or ownership cycles; lock cost is stated.
9. Baseline, implementation, assessment, acceptance and review were tracked.
10. Seven regressions, inherited acceptance, the full suite and maintenance
    checks ran; their outcomes and static-analysis limits are distinguished.
11. All 22 commitment requirements pass. Known out-of-scope failures remain in
    the assessment and do not become false project-wide completion claims.
12. The confirmed cache repair and assessment were completed autonomously.
13. The final ownership, ordering and mechanism review found no unresolved
    defect within this commitment. Remaining cleanup is explicitly recorded.
14. The report names concrete failing tests and their consequences.
