# Default-suite repair review

commit: 26d9d5f34c1bdeb9a1bcad2aaf01f671d7605241
findings:
  - resolved: DFT-001 numeric insets retain position and layer; explicit position defaults and ordering still pass.
  - resolved: DFT-002 jump-start expectations match the existing implementation and boundary regressions reject jump-end behavior.
  - resolved: DFT-003 all four accordion doctests compile and run with visible imports.
  - resolved: DFT-004 the complete default suite and all inherited requirements have fresh passing receipts.

## Contract and baseline plan

The six failures are two assertions and four doctest compile failures. The
JumpStart assertion conflicts with the already implemented jump-at-zero rule:
W3C CSS Easing Level 1 section 2.3 increments floor(progress * steps) for
jump-start and clamps the final value to one. Its API has no before flag.
Correct that assertion with boundary coverage instead of changing valid easing.
Numeric offsets currently call position_absolute, overwriting both positioning
and the existing z-index; non-numeric offsets already use inset setters alone.
Remove the duplicate numeric path and exercise each side and position mode.
Doctests need visible imports; do not ignore or convert the examples to text.

## Work tracking

- Done: implement the three repairs and verify the full default suite.
- Done: record fresh Cairn evidence for all requirements.
- Done: review the final change and all production rules.

The initial Cairn runner hung after its cargo child exited with the six known
failures. Its output was committed too early and two new tests were added while
the runner remained live. No receipt was produced. The child had stopped; the
hung runner was terminated and its dead lock removed. That run is diagnostic
output only, not acceptance evidence. Fresh checks will use a stable commit.
Development offset regressions before the repair: 9 passed, 3 failed, including
both new tests and the original combined-layer test.

## Failure demonstrations and corrected development run

The full default suite exits zero: 1014 passed, 0 failed, 35 ignored across
51 test targets, including all four repaired accordion doctests. No ignores
were added. The two new offset tests cover all four sides, five position
classes, negative and fractional coordinates, and offsets without positioning.
The new step test checks immediately before and exactly at boundaries for
1, 2, 4 and 8 steps, for both start and end semantics.

Replacing jump-start with jump-end makes the boundary regression fail (101).
Removing the visible Element imports makes the accordion doctests fail (101).
Both mutations were restored. The original offset implementation failed both
new regressions and the existing combined-layer test (101). The corrected full
suite passed before mutation; committed acceptance checks follow restoration.

The RTK proxy also hung after the successful development child exited. A direct
RTK run invocation returned zero; it avoids proxy tracking. The complete suite
log is retained with the later Cairn acceptance output. git diff --check passed.

## Final review

Reviewed the committed diff, the inset parser, step implementation, all changed
regressions and doctests, the mechanism and decision, and the fresh receipts.
No source or test code changed during this review. All 26 requirements pass.
The committed full-suite output independently totals 1014 passed, zero failed,
35 ignored across 51 targets. All mutations were restored before that receipt.

Compatibility attack: offsets use the existing numeric parser and inset setters.
They no longer call a position setter. Explicit position classes still assign
their documented layer defaults, and later explicit z-index still wins. Code
that relied on an offset choosing absolute positioning now needs the explicit
absolute or fixed class, as recorded in the commitment and decision. Numeric
coordinates, negative values and fractions retain their prior parsed values.

Semantics attack: the easing implementation did not change. The original test
contradicted its own immediate-jump comment. Boundary coverage includes zero,
one and just before each jump. The positive-count contract makes no new promise
for zero counts or CSS before-flag evaluation, which this API does not expose.
Doctests remain ordinary executable Rust; imports resolve from the public API.

Evidence attack: the first two interrupted Cairn runs have diagnostic output
but no receipts and do not establish acceptance. Capturing cargo output in a
regular temporary file before forwarding it resolved the observed capture hang.
The exact underlying descriptor owner was not established. A stand-in cargo
command that prints intentional failure and exits 23 is forwarded unchanged by
the final wrapper: stdout contains that message and the wrapper exits 23. The
wrapper uses the real cargo exit status, a 300-second deadline, and process-group
termination on timeout. It does not parse output to manufacture success.

Static checks: Ripwire reports apply_position has an unchanged public contract
and zero incompatible callers. Quality delta and the static test gate were run
and are not clean passes: the scan includes untracked reference terminals absent
from the Git baseline, and it marks the three newly executed tests as dead code.
The reference-code duplication reports pair unchanged animation functions with
unrelated reference methods. The broad test gate cannot establish dynamic test
coverage; actual full-suite and inherited mechanisms provide that evidence.
Focused rustfmt on z_index_tests.rs and git diff --check passed. Existing broad
formatting, strict Clippy and FFI debt remain outside this commitment, as recorded
in the prior assessment; this review does not claim those gates are clean.

## Production self-audit

1. Mapped the failing behavior, existing parser, easing semantics and public imports before repair.
2. Removed one redundant offset path, corrected one assertion, added imports and three regressions; no unrelated runtime cleanup.
3. Reused the inset parser instead of introducing another helper or hidden state.
4. Preserved signatures and explicit class ordering; recorded the intentional offset-position behavior change.
5. Propagated command failures and retained interrupted output without inventing receipts; no secrets added to artifacts.
6. No dependencies, unsafe code or new external-input boundary were introduced.
7. No persistent application state or migration changed; check artifacts are committed after execution.
8. Removed duplicate parsing work; verification has a deadline and temporary-file cleanup.
9. Tracked implementation, acceptance evidence and review through completion.
10. Ran the complete default suite, inherited checks, failing mutations, wrapper failure propagation and focused formatting checks.
11. Counted the actual passing output, kept existing ignores visible and stated the remaining out-of-scope lint and FFI limits.
12. Followed the requested six-failure repair and recorded the judged compatibility choice.
13. Reviewed every rule and found no unresolved defect within this commitment.
14. Kept the decision, behavior change and verification record explicit and concrete.
