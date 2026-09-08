# Default-suite repair review

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
- In progress: record fresh Cairn evidence for all requirements.
- Pending: review the final change and all production rules.

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
