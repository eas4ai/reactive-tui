# Preserve the absolute-fill contract in existing tests

The ABI-004 regression receipt 20260912T191940318Z found two convenience-macro
tests asserting fixed 100-cell width and height. The public macro's contract is
"fills its container". API-018 had repaired that behavior and added an independent
App/SuprTUI frame test with a 7-by-3 parent inside a 20-by-6 viewport, but missed
these two old assertions. The production macro was already correct.

Changed only the four old dimension assertions from 100 cells to 100 percent.
The existing numeric-width tests still require literal cell lengths. Added both
existing CSS test groups to API-018 acceptance, requiring five unit cases and
fourteen consumer cases, alongside the existing App frame test.

A safe macro mutation back to fixed 100-cell dimensions failed the unit test,
the consumer test, and the independent painted-parent test. All three failures
were executed assertions, not compiler failures. The first attempt used absent
width/height methods and failed to compile; that diagnostic was retained and
explicitly rejected as demonstration evidence. The corrected source bytes were
restored in finally. The expanded Props/CSS section then passed.

The complete inherited development command passed, including formatting, strict
default lint, FFI checks, the full default suite, registry checks, wakeups,
embedded terminals and rendering. Formal checks must follow the committed repair.

Focused Ripwire impact/uses found the two old tests; its floor missed the fully
qualified macro call in the new App test, which was inspected and exercised
separately. All three edit checks passed. Quality-delta exited 2 and test-gate
exited 4; neither is recorded as passing. Changed-path quality findings concern
correction-related churn, two added check calls (class length 212 to 214), and an
unmodified short match expression paired with an archived WezTerm SFTP expression.
Combining those unrelated dispatchers would create coupling. The named consumer
test and the additional real test groups all ran. Raw reports are retained.
