# FFI, lint and formatting repair review

commit: e3b1ca0dd0daf5e016e0c61afbed68f7ab24401c
findings:
  - resolved: MNT-001 formatting passes and formatting-only edits preserve behavior.
  - resolved: MNT-002 strict default Clippy passes without broad new suppression or lost assertions.
  - resolved: MNT-003 all ffi test targets compile/link and the repaired ABI/lifecycle checks pass.
  - resolved: MNT-004 all 26 inherited requirements have fresh passing receipts.
  - resolved: the live-desktop test stall is isolated with real command fixtures; the production wait limitation is captured in the backlog.

## Work tracking

- Done: establish the commitment and reproduce the formatting gate.
- Done: format workspace Rust and verify unchanged behavior.
- Done: repair strict Clippy findings and verify behavioral changes.
- Done: repair FFI compile/link integration and verify repaired ABI entry points.
- Done: refresh all acceptance evidence and complete final review.

## Mechanism review plan

The prior assessment records real failures for all three commands. Each new
mechanism runs that exact command and propagates its exit status. Formatting
will be demonstrated by the existing differences and the corrected workspace.
Clippy findings need source review before automatic suggestions are accepted.
FFI compilation is necessary but not sufficient: add focused execution to its
mechanism when the missing functions and linkage corrections are implemented.
MNT-004 also requires all separately inherited requirements; the FFI receipt
alone does not prove those requirements.

Formatting baseline failed with real rustfmt differences. cargo fmt --all changed 158 files. The corrected formatting command and full default suite both exit zero; the suite retains 1014 passed, zero failed and 35 ignored. git diff --check passes. No manual behavior edits are mixed into this formatting commit.

## Clippy repair and failure demonstration

Strict cargo clippy --locked --all-targets -- -D warnings now exits zero.
The full default suite passes with 1014 passed, zero failed and 35 ignored.
Workspace formatting and git diff --check also pass. Existing asserts remain;
constant-false branches now panic explicitly, while tautological assertions now
check CSS properties, surface cells, animation defaults and capability mapping.
A deliberate mutation that discards a requested CSS foreground color fails the
strengthened color test with exit 101 (white instead of red). After restoration,
the full suite passes. The original strict gate supplies the failing lint case.

Reviewed the automatic suggestions: derived enum defaults keep the same variant;
unit struct construction, range checks and empty-collection checks retain their
meaning. Match guards retain the existing wildcard fallthrough. Reverse sort
keys retain descending z order and stable ties. Checked division keeps the zero
denominator defaults. Counter iterators start at the same offsets and retain
bounds checks. Menu callback aliases retain their exact Arc/dyn trait type.
The only new lint exception is the recorded expectation on MenuTheme to retain
Custom(MenuStyle) construction without boxing or adding a new allocation.

## FFI implementation and verification

Restored the five terminal symbols using the shipped terminal/event declarations.
The Rust extern-only target now links the crate. Stale widget tests call the
existing Rust exports; their success, non-null and pointer-stability assertions
remain. Removed callback arguments were empty stubs, not tested behavior.

The ordinary app-builder workflow exposed a released allocation in the setters.
The repair preserves that allocation and takes only its contained value. Fallible
backend construction precedes taking the value. The root callback now consumes
FFIElement, matching the constructors. Renderer shutdown retains its allocation;
borrowed surfaces have separate registration and cannot be freed as owned surfaces.
Modern and legacy renderer/terminal constructors share their ownership trackers.

The developer FFI gate passes all test-target compilation, the C shared-library
smoke fixture, seven FFI unit tests and 63 tests across six integration targets.
Each runtime command runs serially in a controlled 80x24 PTY and must restore
terminal settings. A safe mutation returning width plus one made the C dimension
assertion fail; restoring the implementation made the full gate pass. The initial
missing-symbol compile/link failure is the compile gate's failing case. No unsafe
original ownership path was deliberately rerun for a failure demonstration.

The final source diff retains public signatures. Unsupported legacy event payloads
return an error rather than inventing a representation. Remaining TypeScript and
modern-header ABI drift is backlogged; docs/ffi-maintenance.md states the tested
scope and caller serialization/lifetime requirements.

Ripwire's qualified terminal-create edit check passes. Its quality scan exits 2
and includes ignored reference trees, preexisting symbols, FFI exports and test
functions labeled dead. Reviewed the changed-file findings: short boundary
validation/registration blocks deliberately remain local; event translation is
one explicit match. Its test gate exits 4 listing test obligations and unmodeled
coverage, not executed test failures. The actual default, FFI and inherited Cairn
checks provide runtime evidence; no clean whole-repository static claim is made.

Final post-FFI formatting and strict default Clippy commands exit zero. The
post-FFI default-suite developer run is pending: two existing clipboard tests
are waiting for their wl-copy children. No passing result is claimed for it.

## Inherited default-suite clipboard repair

The post-FFI developer run and the committed DFT mechanism both reached their
300-second deadlines in two live wl-copy children. Their process groups stopped.
Recorded the failing DFT receipts before repair. No FFI change touched clipboard
production code; the host dependency made these unit checks non-repeatable.

The two Unix clipboard operation tests now run themselves in bounded child test
processes. A private PATH selects fixture which/wl-copy/wl-paste commands. Actual
backend detection, process creation, stdin writes, stdout reads and hook state
updates still execute. All earlier assertions remain or are stronger; both copy
and paste now have result assertions, including a Unicode/newline round trip.
There is no global environment mutation and no new ignored test. The child must
report exactly one passed test. A drop guard stops its process group on failure
and removes the private fixture directory. Non-Unix execution remains unchanged.

Replacing the actual Wayland copy bytes with wrong data makes the child round-trip
assertion fail and propagates exit 101 to the parent. Restoring the code passes
all four clipboard unit tests and the full default suite: 1014 passed, zero failed,
35 ignored. Strict default Clippy, formatting and diff whitespace checks pass.
The production live-desktop wait remains separately backlogged; fixture evidence
is not represented as desktop integration coverage. All production clipboard code
is byte-for-byte unchanged. Ripwire edit-check passes; quality/test-gate retain
their documented static limitations. The new fixture length is test isolation and
cleanup in one local helper, with no production abstraction or dependency added.

## Final review of the committed candidate

Examined the final FFI signatures against the C terminal/event declarations and
checked the C fixture links the actual shared library. Reviewed app setters,
consuming build/run, root FFIElement transfer, terminal destruction, renderer
shutdown/destroy and borrowed-surface registration. Existing integration tests
exercise shutdown followed by destroy and surface drawing before presentation.
The trackers preserve registered ownership; they do not promise concurrent handle
use or protection against stale-address reuse. The documentation states those
limits. No modern capabilities-layout call was executed as a compatibility test.

Revisited the behavioral Clippy changes and retained MenuTheme construction.
Checked that clipboard changes are entirely inside cfg(test), that its child
assertion failures reach the parent, and that private child environment changes
do not affect concurrent tests. The wrong-data demonstration tests production
command input and the resulting paste output rather than only fixture setup.

All 30 requirements have fresh passing receipts. The full default suite records
1014 passed, zero failed, 35 ignored. The FFI runtime gate records seven unit tests,
63 integration tests and the C ABI fixture, with terminal settings restored.
Formatting and strict default-feature Clippy pass. The committed receipt history
also retains the real earlier failures. No code changed during this final review.

## Production rules self-audit

1. Understood the agreed maintenance contract, existing signatures and inherited
   requirements before edits; decisions and this review retain the evidence.
2. Kept changes to formatting, diagnosed lints, FFI integration and the inherited
   flaky unit check. No new dependency or unrelated product feature was added.
3. Kept ownership handling at FFI boundaries and fixture orchestration in tests.
   Reviewed local repeated validation instead of adding a generic abstraction.
4. Preserved exported signatures, MenuTheme construction and consuming ownership
   contracts. Updated the event comments and maintenance documentation together.
5. Restored APIs report errors and unsupported events explicitly. Tests propagate
   failed child status. Fixtures contain fixed test text and no secrets.
6. Reviewed pointer ownership before dereference/destruction and the caller's
   serialization obligation. Fixture commands receive text through stdin, not
   shell interpolation. No unsafe original ownership path was rerun deliberately.
7. Separated borrowed from owned surfaces and shutdown from destruction. No
   persistence migration is involved; check receipts refer to stable commits.
8. Runtime gates have deadlines and isolated terminals/processes. Clipboard test
   cleanup kills its owned process group; live product waits remain backlogged.
9. Maintained one active todo item and completed it only after verification.
10. Ran formatting, strict default Clippy, complete default tests, FFI compile and
    runtime checks, and every inherited mechanism. Recorded failure demonstrations.
11. Distinguished executed passes from static-tool limitations, mock desktop
    integration and the remaining ABI backlog. No ignored test was added.
12. Used the authorized commitment and recorded judged decisions without expanding
    into the separately captured TypeScript or live clipboard product work.
13. Satisfied with this maintenance change against the agreed contract after final
    source/evidence review. No unresolved finding remains within this commitment.
14. Reviewed explanations and documentation for concrete behavior and visible
    limits, without describing the whole legacy library as production-ready.
