# Registry cache isolation review

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
Final committed acceptance and formal review remain pending.

The inherited development gate initially encountered a compiler internal error
in the embedded feature build. Clearing this crate's generated incremental
cache and rebuilding unchanged source produced a complete passing gate.
The raw diagnostic and recovery are included in the assessment.
