# API-016 Unix readiness repair

## Defect and boundary

The failing formal receipt and two independent reproductions remain under
api-016-resize-input. A resize stalled e until z arrived. The exact new
barrier consumer delivered only 1,024 of 2,048 queued bytes against the
registry dependency. Neither failure is counted as acceptance.

Crossterm returned early from a Mio readiness batch and discarded unfinished
work on its next poll. The local 0.29.0 dependency retains at most the three
registered readiness tokens, checks readable input before each read, and
harvests new readiness while draining a retained burst. Parser APIs, terminal
ownership and Windows implementation are preserved. Apple uses select for
the additional readiness check; other Unix targets use poll. The direct
Cargo path dependency also applies to downstream checkouts.

## Verification and adversarial review

The revised full API-016 command passed locally, including three dependency
readiness tests, five legacy tests, eight Rust App workflows, three manual
routes, the new barrier consumer, and four C workflows. Captures are under
api-entry-points/20260913T052150Z. The previously intermittent Crossterm
32x8 route then passed 64 consecutive runs under
api-016-resize-input/corrected-20260913T052240Z.

Default all-target Clippy with -D warnings and workspace format check passed.
Crossterm's unchanged upstream unused-parentheses warning remains visible;
this is not a warning-free build claim. Existing FFI warnings remain visible
in the FFI build. The dependency test's first compile failed on a byte versus
byte-slice fixture typo; its correction and passing rerun are retained.

Read-only review caught starvation in the first repair: retained input could
prevent new wake/resize edges from being harvested. The corrected code polls
nonblocking between parser batches, deduplicates tokens, and includes an
actual newly triggered wake during a 4 KiB burst. The revised command passed.
An independent worker reproduced the exact new barrier fixture against the
unmodified registry artifact and preserved its provenance and failure.

Native Apple readiness and refreshed platform evidence are still pending.
No commitment-complete or all-platform claim is made here.

## Production self-audit checkpoint

1. Traced the failing input through App, backend, stream and dependency.
2. Restricted the repair to readiness bookkeeping and its declared dependency.
3. Kept upstream source/license and documented the maintained delta.
4. Retained public APIs, nonblocking polls and terminal restoration.
5. Propagated read errors and retained diagnostic failures.
6. Used a pinned release archive with recorded SHA-256; no new external service.
7. Preserved queued work across partial consumption and wake interruption.
8. Bounded readiness storage and test waits; checked blocking descriptors.
9. Existing API-019 todo remains in progress while this regression is repaired.
10. Ran focused tests, full entry-point workflows, repeated failing route and lint.
11. Native verification and broader commitment work remain explicitly open.
12. Recorded the judged dependency maintenance decision before implementation.
13. Adversarial review produced a repair and regression test before this checkpoint.
14. Documented behavior, evidence and limits directly.

## Static-tool limits

Ripwire focused edit-check returned 0. Quality-delta returned 2 and test-gate
returned 4; neither is a pass. The broad graph includes the large untracked
WezTerm reference checkout and historical diagnostic scripts. Focused findings
on the repair include complexity in the retained upstream event loop, tests
and dynamic EventSource methods labelled dead, and unchanged upstream parser
duplication between optional backends. The three tests actually executed.
The short repeated test setup keeps the distinct wake and burst failure cases
readable; it does not duplicate production logic. Similar generic constructors
in unrelated application modules are not reusable inside this dependency.
No broad reference-code cleanup was included.
