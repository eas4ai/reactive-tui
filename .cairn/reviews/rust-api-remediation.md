# Review: rust-api-remediation

Status: In progress

## API-001 mechanism construction

Inspected every legacy and improved RTuiSignal constructor, getter, setter,
destructor and hook-handle clone, plus the distinct thread-safe signal family.
The old destructor reinterprets numeric allocations as Signal<String>; legacy
getters also permit unchecked mismatched types. No unsafe baseline was executed.

The mechanism rejects direct casts to Signal<T> before native test execution.
Its safe negative control rejects the known destructor expression; a downcast
control is accepted. Real Rust and C tests cover typed values, 64-bit preservation,
wrong live types, both destructor aliases, string ownership, shared hook lifetime,
and the separate thread-safe lifecycle. Miri runs the actual Rust integration
cases with leak checking enabled. Completion requires the corrected cases to run;
the source guard alone is not acceptance evidence.

This is a mechanism construction note, not the final commitment review.

## API-001 implementation verification

The safe baseline guard rejected the original direct casts before native tests.
After repair, all four Rust integration cases, the compiled C consumer, and all
four actual Rust cases under Miri passed. Miri leak checking was not disabled.
Legacy i64 extrema survive; mismatched i64/C-int access returns an error without
changing output. Shared hook handles remain valid after their context is freed.

Reviewed the implementation diff: all ordinary RTuiSignal constructors now box
FFISignal, both destructors release that allocation, and all legacy typed getters
and setters downcast before use. The separate thread-safe String allocation still
uses its matching destructor. No exported signature or layout changed.

Ripwire edit-check reported no FFISignal contract change. Test-gate named the new
ownership test and existing seamless tests; both ran. It also named rtui_free_string
through a broad static edge; this change uses rtui_string_free, which is exercised
with the owned getter. Quality-delta exited 2, dominated by ignored reference trees;
changed-file rows report repeated typed FFI wrappers and the new i64 constructor.
Those short explicit wrappers preserve distinct C signatures and existing patterns;
introducing a macro/generalized dispatch solely to remove those rows would obscure
the ownership repair. This is not a claim that the quality-delta gate passed.

## API-002 mechanism construction

Inspected App render ordering, registry factory/instance ownership, component
update/mount/unmount wrappers, Element props/children and render-tree conversion.
A real App delegates painting to SuprTUI and captures parsed terminal frames.
The baseline ran four cases: unknown-container/plain-text controls passed;
nested keyed rendering, duplicate-key rejection and bounded recursive output
failed. The nested frame was blank. No acceptance criterion was weakened.
The test also requires props to update state on the same instance, keyed reorder
to preserve identity, removed children to unmount once before later sibling work,
and all remaining instances to unmount on App exit.

## API-002 implementation verification

All five App/SuprTUI cases pass: nested output, prop/state updates, keyed reorder
and removal, component-type replacement, unknown-container fallback, duplicate-key
errors and bounded recursive expansion. Seven wakeup cases, seven legacy automatic
memory-management cases and nine renderer cases passed. Strict default-feature
Clippy across all targets passed.

The first post-repair lifecycle assertion assumed removal must precede sibling
rendering within the same frame. The runtime prunes after expansion and before
painting. Added a subsequent sibling frame to verify the actual boundary: removed
instances are gone before later frames. The requirement was not narrowed.

Inspected ownership and cleanup paths: the App instance map owns live components;
registry factories release locks before constructors run; no live clone enters
the paint tree. Replacement and pruning drop descendants before parents. The
existing public legacy converter and explicit legacy cleanup still function.
App-owned components are not registered in that global legacy instance map.

Ripwire marks the new recursive walker as complex (80 lines), but it contains one
bounded expansion traversal with explicit lifecycle and fallback branches. Its
new-symbol/dead-code and nested-function duplication reports are static-analysis
limits; real App tests exercise the walker and pure converter. Existing App::waker
is exercised by wakeup tests. Repository-wide quality-delta still exits 2, including
ignored reference trees and churn findings; it is not reported as passing.
Test-gate identifies broader inherited paths, which Cairn will rerun before the
next requirement. No final commitment-wide review is claimed.

## Inherited default-suite isolation repair

The independent DFT refresh failed test_component_lifecycle_safety with 99 live
components instead of 100 after the aggregate run passed. Inspection found four
other tests calling global_cleanup_all without the file's TEST_MUTEX while the
counting tests held it. Added that same guard to every remaining global-cleanup
test. Concurrent registration still spawns ten workers within its guarded test;
no concurrency or lifecycle assertion was disabled. Five repeated executions of
the eight-test production_readiness_test binary passed after the repair.

## API-003 mechanism baseline

The five new hook tests all failed against the existing code: generated renders reset state, memo handles stayed stale, and type, count and kind violations were accepted. These are safe Rust failure demonstrations. The corrected cases retain state for 1,000 renders and retain memo handles across value changes.

## API-003 corrected mechanism and implementation

Generated renders now enter a hook frame; one mutex protects positional index and slots. Count, kind and type violations panic after releasing the mutex, so a corrected subsequent render remains usable. State, reducer, previous and memo slots have separate kinds. Memo has no dependency-list API: it computes each call, retains its signal and publishes changed values. Manual reset remains supported.

The corrected mechanism passed all five integration cases and four hook unit cases. The bounded-storage test renders 1,000 frames with exactly three retained slots. Overlapping frames fail without poisoning state. Related macro, mouse, production-readiness and App wakeup suites passed 32 tests. Strict all-target Clippy and formatting passed.

Ripwire edit-check reports the private storage signature change with no incompatible callers. Test-gate named the six suites now exercised (including the new mechanism); its unmodelled runtime paths remain covered by inherited acceptance where declared, not by a claim of complete static coverage. Quality-delta exits 2 because ignored reference sources enter its baseline and normalized constructor/cleanup patterns trigger duplication. The hook rows were examined: the new types are used and the tests run; typed Arc/Mutex constructors do not warrant a shared abstraction. The macro change adds one render-boundary statement. No quality-gate pass is claimed.

## API-004 mechanism baseline

Four safe lifecycle cases failed before implementation: intervals did not fire, effects immediately ran cleanup before the render body completed, descendant context was absent, and standalone cleanup ran immediately. The captured screen assertions trim terminal padding; they still compare exact visible provider values. These tests exercise real App/SuprTUI frames and retain the App scheduler for post-removal inspection. Additional controlled dependency and cancellation cases will cover resource boundaries during implementation.

## API-004 corrected mechanism and resource review

App now supplies a root resource scope and each keyed instance supplies a child scope. A render binds the App scheduler and snapshots inherited provider values; a new App starts an independent context. Descendant cleanup retains its last inherited values. Component removal unmounts before closing resources, including resources retained by external Hooks clones. Standalone hooks keep effect cleanup until explicit cleanup or final drop.

Effects have stable positional slots. The no-dependency API runs after each completed generated render, with old cleanup before replacement. The added dependency API compares explicit values and skips unchanged renders. Aborted renders do not commit dependencies, and callback panic does not leave an effect permanently marked running. Cleanup callbacks run outside resource locks. Hook storage initializes new slots lazily using internal constructors.

Timers capture their owner and scheduler. Unchanged durations preserve deadlines; callbacks refresh. Timeouts become inactive after firing and require a duration change to restart. Cancellation remains valid outside the ambient render. Debounced calls after removal cannot add work. Scheduler callbacks hold weak timer references, and resource cleanup releases callback captures even if a handle remains. Tests also cover cancellation from inside an interval callback.

The first timer assertion was too weak: a fallback timer could run while the App scheduler stayed empty. The strengthened captured-frame check rejected that implementation with [false,false,false] instead of [true,false,false]. A nested-App probe also rejected inherited caller context before root boundaries were added. A temporary mutant replacing duration-aware effects with every-render effects failed the exact deadline assertion (exit 101); restoring the source passed. These tests demonstrate failures rather than relying on success-only execution.

Final targeted checks passed six App integration tests, seven hook unit tests, one nested-scope test and eight timer unit tests. The full repository runner passed, including 41 doctests; the first bare cargo run hit the known system /tmp quota during doctest linking, so the documented workspace-temp runner was used. The final added App-stop case and strict all-target Clippy passed. Formatting and diff whitespace checks passed.

Ripwire edit checks found no incompatible use_effect or storage callers. Test-gate listed the affected existing suites plus unmodelled/reference paths; the actual project suite was run. Quality-delta exits 2, not a pass: inspected changed rows are short-horizon churn from consecutive hook requirements, constructor/lock pattern duplication, duplicated fixture structure, and false dead-code reports for exercised types/tests and App::waker. The resource and timer logic remains separated by ownership responsibility; sharing normalized constructor or lock sequences would hide that responsibility. The component traversal grows by scope entry/exit only. This is an API-004 implementation review, not the final commitment review.

## API-005 mechanism and implementation review

The committed baseline keyboard probe failed with zero callback invocations instead
of one. Inspected builder storage, component expansion, App input/presentation
ordering, the worker acknowledgement, clipping and stable painter order, routing
propagation, subtree cleanup and callback ownership. The approved Element metadata
field and Send + Sync bounds are documented with Rust migration examples. C ABI
signatures and layouts are unchanged.

Nine focused cases pass. App cases use real SuprTUI output: keyboard activation
without release/repeat duplication; mouse focus and misses; resize geometry;
redraw callback refresh; overlap and ancestor clipping; callback capture release
before later App input; nested registrations; expanded component callbacks; and
a 120-column initial viewport with enough targets to split the spatial index.
The router phase test independently asserts capture/target/bubble ordering and
retained Handled status. Captured VT100 text and explicit expected coordinates
are checked alongside callback logs.

A safe violating case reversed only the returned geometry order, leaving painting
unchanged. The overlap test failed: callbacks were [] instead of [top, clipped].
The mutation was restored and the corrected suite passed. This is real failure
demonstration, not a defect-confirming acceptance assertion.

The first full default run exposed a pre-existing test race in debug_animate_none:
one case cleared the global animation count while another subtracted it, causing
unsigned underflow (initial count 1, final count 0). Reproduction failed on run 5
with three test threads. A mutex now isolates the two global-reset integration
cases; 20 parallel repetitions passed. No animation implementation or assertion
was weakened. The subsequent complete default suite passed, including 759 library
cases, all runnable integration targets, and 41 doctests (34 remain ignored).
Strict Clippy with all default-feature targets also passed.

Ripwire edit-check found no incompatible on_click call arities; it does not model
the approved Send/Sync change, which cargo check/clippy verified. Test-gate exited
4 with 39 named suites and a broad static blast radius that includes ignored
reference trees and non-built integration demos; this is not an all-static-path
coverage claim. Quality-delta exited 2, dominated by those reference imports,
normalized initializer/HashMap/fixture similarities and trait-dispatch dead-code
false positives. Its real nine-argument traversal finding was addressed by
grouping frame registration state; node traversal and registration are separate.
Short-horizon renderer churn reflects the consecutive committed recovery steps.

Self-audit: callback ownership is explicit, user code runs without router locks,
old registrations are replaced, and geometry comes from acknowledged painting.
No dependency, C ABI, terminal protocol or persistence changes were introduced.
Custom backend wrappers must forward painted_nodes; complete legacy backend
integration and focus traps remain governed by API-016 and API-006. This is an
API-005 review, not final acceptance of the entire commitment.

## API-006 mechanism and implementation review

The committed baseline failed all three initial App cases: focus callbacks were
[A+, A+, A+] across redraws instead of [A+, A-, B+]; BackTab and release events
navigated incorrectly; nested dialogs activated the opener or wrong outer control.
The tests share API-005's captured SuprTUI input harness. Its new file was added
to both mechanism footprints, and API-005 passed after the extraction.

Inspected both previous focus managers, stable event IDs, tab-order rebuilds,
trap activation/removal, App's public focus methods and focus-event dispatch.
The router now owns current focus for both App APIs and input. The private App
manager retains only declarative trap/autofocus bookkeeping. Rendered order
breaks equal-index ties; negative indices skip Tab; release and modified Tab
are not treated as ordinary traversal. Lost/Gained callbacks match identity
transitions rather than render count. Removed subtrees release registrations
and traps in one batch before restoration chooses a surviving target.

Twelve acceptance cases pass: keyed redraw/reorder; forward/reverse/release
navigation; nested autofocus/wrapping/restoration; parent removal with an inner
dialog open; mouse focus confinement; positive/default/negative ordering;
public App focus methods; missing opener fallback; empty traps; imperative
subtree cleanup; false-to-true autofocus; and explicit imperative reactivation.
The seven existing focus unit cases and all nine API-005 cases also pass.

Compatibility review caught an implementation regression before commitment:
an explicit create_focus_trap call must reactivate the named container, whereas
a declarative redraw must not reorder live traps. Added a failing regression
case, then preserved the imperative behavior separately from frame reconciliation.
Public App and EventRouter signatures and the C ABI remain unchanged.

A safe mutation discarded saved restoration targets. The nested test then
activated OUTER1 after closing the inner dialog, rather than the remembered
OUTER2. Restoring the implementation made the corrected cases pass. A frame
assertion was corrected to trim terminal row padding, while still checking
the exact C/B/A row order; no behavior assertion was weakened.

The full default runner passed all 57 suite groups, including 759 library cases,
all runnable integration targets and 41 doctests (34 ignored). After extracting
focus callback registration into its own helper, the 21 focus/event cases and
strict all-target Clippy passed again. The acceptance receipt is still
produced separately by cairn check; development test output is not a receipt.

Ripwire edit-check found unchanged public trap-call arity and no incompatible
callers. Test-gate exited 4 with 12 static suite paths, including non-built demos
and historical defect probes; those are not claimed as runnable coverage.
Quality-delta exited 2, again dominated by ignored reference imports and static
trait/callback blind spots. Its registration-complexity finding prompted the
focus callback helper extraction. The remaining system-event branches directly
express release, reverse traversal and modifier rules; tests exercise them.
Six private upsert arguments distinguish validated imperative creation from
empty declarative traps and their preferred/restored target. No gate-pass claim
is made for this advisory output.

Self-audit: one focus owner, bounded tree walks, explicit callback ownership,
no internal locks while user callbacks run, tested removal and fallback paths,
and no new dependencies or unrelated production changes. A focus trap confines
focus; a modal pointer backdrop remains part of API-012. This is API-006 review,
not final completion of the 54-requirement commitment.


## API-007 mechanism and implementation review

The committed baseline failed all eight initial cases. Both editors advanced
insert_text by byte length against a scalar buffer; cursor movement mixed the
same units, newline indices stayed stale, and selection/rendering split or
misplaced text. Inspected GapBuffer mutation/index maintenance, both editor
paths, Cursor movement/selection, syntax runs and Surface output/copy/mutation.

The shared edit path retains scalar offsets and converts explicitly at grapheme
and display boundaries. Insertions and deletions account for clusters formed by
joining adjacent text. Vertical movement preserves a terminal column, while word
movement retains graphemes. LF indexing updates for every mutation; CRLF moves
and deletes atomically. Both painters apply selection at actual scalar positions
across syntax runs, expand tabs, clip complete graphemes and render control text
as visible replacements. The documented behavior is in docs/editor-positions.md.

Private Surface metadata retains multi-scalar and wide glyphs without changing
Cell or the C ABI. Reviewed overwrite of either wide cell, copies, clear, resize,
raw mutable access and DiffWriter comparison/output. Existing editor signatures
remain intact. Mutable legacy Cell access cannot retain full graphemes and clears
that metadata explicitly. Other legacy writers/adapters retain their API-016
integration requirement; SuprTUI is unchanged.

Twenty-three acceptance cases and 22 existing editor unit cases passed. The
expanded cases cover mixed insert methods, CJK/combining/emoji clusters, selection
across lines and syntax runs, forward/backward deletion, CRLF, word and vertical
movement, tabs, control characters, zero-size and clipped viewports, Surface
ownership, and actual parsed terminal cells before and after edits. Those terminal
checks found a real stale-output defect during implementation: Cell::default
contains NUL, so clearing with it did not erase prior text. Clearing with spaces
fixed the failing update tests. One vertical fixture's explicit scalar offsets
were corrected after enumerating its text; no behavior assertion was removed.

A safe mutation emitted only each stored grapheme's first scalar. Both parsed
terminal tests failed with e instead of e-plus-accent (exit 101). Restoring complete
text passed. The full default runner passed all 58 groups: 759 library tests,
all runnable integrations and 41 doctests (one library and 34 doctests ignored).
After extracting Surface text comparison/output helpers, the 23 acceptance and
22 unit cases passed again. Strict all-target Clippy passed after fixing a blank
line in a documentation list. Formatting and whitespace checks run before commit.

Ripwire test-gate exited 4 and named 53 static paths, including ignored references,
non-built demos, engine suites and inherited integration checks. Actual default
and focused tests were run; inherited checks follow on the committed tree.
Edit-check's Cursor definition-count change includes reference-tree name collisions;
its parameter count is unchanged, and existing Rust consumers compile. Quality-delta
exited 2, dominated by those references and normalized initializer/wrapper/test
similarities. New Surface text helpers reduce added branching in the already
complex diff routine. The line painter's branches represent syntax style lookup,
control/tab representation and selection/cursor overlays in one traversal. Short
forward/backward wrappers and explicit tests remain clearer than generic dispatch.
No advisory gate pass or complete static-path coverage is claimed.

Self-audit: shared position/painting rules replace divergent editor code; the
new boundary map is line-sized and the painter avoids cloning entire syntax runs
per grapheme. Metadata ownership is local, overwrite cleanup is explicit, no
new dependency or unsafe operation was added, and public layouts are preserved.
The change has failure demonstrations and real output verification. This is an
API-007 implementation review, not the final commitment-wide review.


## Inherited FFI compiler-cache repair after API-007

API-007 and ABI-001 through ABI-003 passed, then ABI-004's inherited FFI
compile gate failed inside rustc 1.95.0's incremental dependency graph
(index 1025949 against length 90883, compiling app_wakeups). The receipt retains
the compiler backtrace. This was a compiler panic, not a Rust test assertion.
The maintenance FFI compile now sets CARGO_INCREMENTAL=0 for that subprocess.
Its command, target set, timeout, exit handling and following real FFI runtime
checks are unchanged. No application source or assertion changed. The corrected
maintenance FFI command passed its build, C consumer and Rust lifecycle checks.
This is a bounded build configuration repair inside the inherited footprint.

## Inherited default-suite race found during API-008 verification

The full default run failed in simple_css_animation_test at the unchecked
counter subtraction (initial animations 4, final 3). A rerun passed. Inspection
found four tests in that binary clear and mutate the same global registry and
animation state concurrently. The neighboring CSS animation integration suite
already serializes this shared-state pattern. The repair is to use the existing
serial_test guard on those four tests, preserving every assertion and leaving
the parser-only test parallel. This is an inherited full-suite reliability fix;
it does not change animation runtime behavior or satisfy pending API-010/013.

The repaired animation binary passed 20 runs with --test-threads=16 (100 tests).
The complete default-suite script then passed all 60 result groups, including
761 library tests (one ignored), 41 doctests (34 ignored), and all runnable
integration tests. Strict all-target Clippy, workspace formatting and diff
whitespace checks passed. Ignored platform/manual tests are not claimed as passes.

## API-008 implementation and mechanism review

The recorded baseline failed seven of eight isolated cases: nonzero exits and
missing tools reported success, detection spawned an unbounded which process,
and copy/paste/cancellation exceeded the fixture deadline. The corrected code
passed all 13 isolated cases plus both direct-child lifecycle unit checks.
The added cases reject invalid UTF-8 and oversized output, preserve the last
successful cache after errors, prevent commands after owner drop, and return
without waiting for inherited output handles. Fixture cleanup owns its process
groups and checks the direct child is no longer live or unreaped.

Inspected backend detection, fixed command arguments, deadline checks, temporary
stream ownership, child kill/wait paths, weak hook cancellation, cache updates
and successful Unix clipboard-server ownership. Clipboard text is neither
interpolated into shell code nor included in logs. The Windows fixed script
uses UTF-8 stdin/stdout; native verification remains required. Public callback
signatures remain unchanged. Transfers have an explicit 64 MiB limit; kernel
and filesystem calls are not real-time bounded. Successful copy may leave the
desktop tool's clipboard-serving process; failure/timeout/cancellation kills
the Unix group and reaps the direct child. Paste stops leftover group members.

Real development probes passed five exact round trips each for Wayland, Xsel
and Xclip on isolated Linux desktops. The first Wayland probe failed because
wl-copy invokes cat for redirected stdin; adding cat to the test-only PATH
made it pass. The harness therefore tests actual tools and their dependencies.
Native platform records are keyed to committed source, macros, Cargo inputs,
tests and the runner script, with canonical bytes across checkout platforms.
Dirty runs cannot satisfy acceptance; missing or damaged records fail closed.
macOS and Windows evidence is absent and API-008 is not yet accepted.

Ripwire edit-check reports use_clipboard unchanged with no incompatible callers.
Test-gate lists four test paths (covered by the suite/native runs) and animation
symbols it cannot connect to tests. Quality-delta exits 2 with extensive ignored
reference-tree findings; relevant rows include branch complexity in platform
selection/verification, fixture setup duplication, and standard Drop/error
handling patterns. These are distinct bounded cases, not a reason to combine
unrelated ownership implementations. Its dead-code rows include tests actually
executed and helpers called by the tested runner. No advisory pass is claimed.
The cleanup scanner catches process disappearance and inaccessible environment
files; other I/O errors propagate. Production self-audit: local behavior and
regression checks pass; native macOS/Windows acceptance remains incomplete.

## API-008 committed Linux platform evidence

All three native Linux runs passed against committed inputs: five round trips
and two process lifecycle checks per backend. Their records and captured output
are stored under docs/analysis/clipboard-platforms. A disposable copy of the
real Wayland record passed the verifier restricted to that backend; changing
committed_inputs to false, changing the source digest, and damaging captured
output each caused the expected rejection. That restricted verifier exercise
is not evidence for the other four platforms. Full API-008 acceptance still
requires native macOS and Windows records.

## Native CI build finding

The hosted macOS run 34307222183 failed compiling the real library because
UnixTty::spawn_input_thread called libc::__errno_location, which exists on
Linux but not macOS. Inspection confirms the only use is to recognize EAGAIN
or EWOULDBLOCK after read fails. Replace that accessor with the standard
last_os_error().kind() == WouldBlock classification. This preserves the branch
and lets the claimed macOS clipboard path build without a Linux-only symbol.
The actual native rerun is required; this note does not claim a corrected pass.

The portable error classification passed the full Linux default-suite run
(all 60 result groups) and strict all-target Clippy before the native rerun.

The first Windows job in run 34307222183 failed inside Sixel's configure script:
PKG_PROG_PKG_CONFIG and PKG_CHECK_MODULES were unexpanded. The MinGW pkgconf
package was present, but MSYS Autoconf lacked /usr/share/aclocal/pkg.m4.
The MSYS pkgconf package provides that file; add it alongside the MinGW tools.
This is a CI prerequisite repair, not a clipboard behavior pass.

The macOS jobs in runs 34307525622 and 34307683870 passed five real clipboard
round trips and both child lifecycle tests after the portable errno repair.
Downloaded artifacts matched their captured-output hashes. The later Windows
job built Sixel successfully, then found 56 Rust compile errors in the existing
Windows adapter. Complete CI diagnostic artifacts are now captured so those
prerequisites can be repaired. No Windows acceptance or final API-008 pass is
claimed. Hosted execution and artifact collection are working; the remaining
adapter repair has its own recorded decision and prerequisite inventory.

## Windows adapter prerequisite implementation

The complete native diagnostic artifact identified all 56 errors in the Windows
adapter and its consumers: wrong HANDLE ownership for Send/Sync, obsolete DWORD
aliases, missing feature-gated APIs, duplicate record parsers, mismatched event
families, invalid union access, nonexistent trait delegates and missing byte read.
The repair duplicates standard handles into OwnedHandle, keeps public methods,
uses the current Windows API types, and converts records into TerminalEvent.
UTF-16 surrogate pairs, press/release/repeat and partial UTF-8 output are retained.
The new byte reader buffers unread bytes and carries a timeout across records.

The native test starts a detached child, allocates its own console, and checks
empty-input timeout, repeated text and Unicode through one-byte reads, event
conversion, queued key events, output, size, restored input mode and invalid
initial handles. It never attaches to the parent's console. Native platform
records now require its success marker as well as the clipboard/process checks.
Linux default-suite verification passed all 61 result groups, strict all-target
Clippy passed, and formatting/diff checks passed. The Windows-only test is not
counted as a Linux pass; native Windows execution remains pending.

## Native Windows startup deadline finding

Run 34309300920 built the real Windows library successfully. The native process
checks then reported one failure: the first PowerShell startup exceeded two
seconds before its PID marker appeared, so the cancellation fixture got the
timeout error. The next warm timeout case passed, including its exit assertion.
The superseding decision gives Windows five seconds while retaining Unix's two.
The tests retain separate six/three-second upper limits and PID exit checks.
This repairs an observed startup allowance; it does not bypass cancellation.

The full Windows library and both native integration targets also passed a
local Windows cross-check with Zig's MinGW target and explicit Autoconf host/C11
settings. That is compilation evidence only, not native clipboard acceptance.
An isolated pre-change worktree made Ripwire's adapter delta readable: it flags
read/run_backend/verify branch complexity and the long console integration test,
plus false dead-code rows for state and a test. These flows separate actual I/O,
record validation and a sequential private-console lifecycle; no advisory pass
is claimed. Its four legacy nested integration paths are not default Cargo test
targets and remain part of the broader platform inventory. The new native console
fixture covers the prerequisite's actual public entry points and conversions.

Fixture review found two timing/cleanup hazards before final native acceptance:
PID-path existence could precede the PID write, and detaching the private console
could leave the test harness's standard handles pointing at the freed console.
Cancellation now waits for a parseable PID; the console guard restores inherited
redirected handles after detaching. The revised Windows fixture cross-checks,
and both Unix lifecycle tests, strict Clippy and formatting checks pass.

Run 34310205820 passed both Windows process lifecycle checks with the five-second
limit, then failed the private console's injected UTF-8 byte read with a timeout.
The adapter enabled virtual-terminal input while consuming INPUT_RECORD values.
Microsoft documents that this mode converts input for ReadFile/ReadConsole;
the adapter uses ReadConsoleInputW instead. Select window/mouse record input
with extended flags (and no Quick Edit), retaining virtual-terminal output.
The existing injected-text check is the failure demonstration for the rerun.

Run 34311479497 passed the Windows adapter and both process lifecycle guards
(the runner reached the subsequent clipboard executable), then failed the first
ASCII copy with PowerShell exit code 1. The public hook deliberately suppresses
command stderr because it can contain clipboard text. Reproduce only the fixed
ASCII fixture on the disposable Windows runner with stderr captured before the
full build, so the next correction follows the native error. macOS and all three
Linux tools passed at 04885d59; these records will need refreshing if inputs change.

Run 34312040125 reproduced the hook failure, but the fixed copy command passed
when GitHub invoked Python directly. That probe differed from the actual Bash
parent environment. Run it under the same Bash shell and fail early on its exit
code; do not infer a command fix from the mismatched diagnostic. All 13 local
clipboard failure-path tests passed at 4f95a7a.

The fixed ASCII copy also passed under Bash in run 34312614549. Inspection
found that the Rust assertion did not report which fixture failed, so the earlier
first-copy claim was unsupported. Label the input byte count and run all five
fixed fixtures through the exact source scripts with stderr captured. This
diagnostic remains separate from the native hook acceptance and uses no user data.

The expanded Python diagnostic in run 34312759483 timed out before its first
result while capturing a stderr pipe, unlike the actual file-backed process runner.
Use a temporary stderr file and report each case before launch. Make this extra
diagnostic nonblocking for the native Rust acceptance, which remains mandatory;
it must not replace the actual failing path with a mismatched probe.

Run 34312996266 identifies the actual failure: ASCII and quoted Unicode with
trailing newlines round-trip, then copying zero bytes fails. The exact-script
diagnostic reports Set-Clipboard ArgumentNullException (parameter text); the Rust
hook independently reports copy of 0 bytes. Windows PowerShell supports clearing
with Set-Clipboard -Value $null (PowerShell/PowerShell PR 14579 documents this
preexisting Windows behavior). Branch only for empty input to use that operation.
Retain the nonempty UTF-8 path and all five native cases. Remove the temporary
workflow diagnostic, whose duplicate clipboard operations are no longer needed.
Source: https://github.com/PowerShell/PowerShell/pull/14579 .

Native run 34313628354 at 09f1435 passed macOS and Windows. Windows executes
all five clipboard cases (including the previously failing empty copy), both
owned-child lifecycle tests, and the private console I/O/event fixture. The same
committed inputs passed Wayland, xsel and xclip on isolated desktops. Downloaded
output hashes and source digests were checked before copying; the five-backend
verifier passes. All 13 clipboard failure-path tests, strict default Clippy and
formatting also pass. The temporary workflow diagnostic has been removed.
Four existing Windows compiler warnings remain in the broader API-019 platform
inventory; this evidence does not claim native strict-lint or whole-API acceptance.

## API-009 mechanism declaration and baseline

The App input harness now retains the independently parsed VT100 screen as well
as text and acknowledged geometry. The new styling mechanism exercises inactive
focus/hover/disabled variants, keyboard and mouse focus, mount autofocus, pointer
enter/leave without application handlers, Unicode case conversion and inheritance,
grapheme-safe ellipsis/clipping, whitespace/word-break modes and cell alignment.
It compares explicit colors and text rather than recomputing the implementation.

Development baseline: 18 tests execute; 2 controls pass and 16 fail. Plain Unicode
text and exact basic colors pass, as does preserved whitespace. Inactive variants
paint red, state-only changes do not repaint, uppercase leaves text unchanged,
and overflow/wrapping/alignment assertions expose the missing behavior. The first
probe also compared unused screen padding; correcting only the observation helper
made the two controls pass while retaining leading and interior spaces.

The declaration is a failing baseline, not complete styling acceptance. During
implementation, add disabled on/off transitions using explicit metadata, combined
state and focus-within cases, and the remaining accepted typography token matrix.
Only then can a passing mechanism support API-009. Preserve the prior decision
that event routing is connected after successful presentation; state changes from
that connection must schedule repaint rather than moving registration before it.
The metadata type was introduced within this still-unfinished commitment; extending
its payload does not require another field or migration for the original Element
API. No production styling code changed in this declaration action.


## API-009 implementation and local verification

The original committed App baseline had 16 failures and two controls. After the
repair, 29 independent captured-output cases pass. They cover inactive variants,
Tab and mouse focus, idle autofocus/hover redraws, focus-within and compound state,
disabled activation/focus/re-enable, stationary-pointer geometry changes, Unicode
case conversion and inheritance, ellipsis/clip, whitespace and word breaking,
text alignment/justification, sibling placement from measured wrapping, baseline
spacing and resets, font/decorations, tab stops and control filtering.

The expanded strikethrough test initially failed: extract_paint_style discarded
the stored flag. The repair retains it without changing the public VisualStyle
or TextDecorations struct shape. The corrected test sees SGR 9 in real App output;
its plain-text control does not. Font reset cases begin with bold and baseline
spacing resets begin with leading-3 so unchanged defaults cannot satisfy them.

State classes are resolved from acknowledged router IDs before layout; publishing
new handlers, focus and hit bounds still follows successful presentation. A state
change after registration schedules a follow-up frame only when resolved styling
changes. The class cache contains resolved ordinary tokens; inherited text options
are applied after lookup. Disabled is stored in the owned metadata introduced in
this commitment, not in typed component props or an added original Element field.
The private text helper handles grapheme widths for both measurement and painting.
Large baseline spacing is represented arithmetically, without allocating empty
lines proportional to the numeric token. Terminal approximations and token behavior
are documented in docs/text-styling.md, now declared as a mechanism input.

Local checks ran: 29 API styling cases; 37 event/focus/wakeup/renderer cases;
24 layout/bridge/alignment cases; strict all-target default Clippy; formatting;
and the full default test targets (1088 passed, two ignored). The 761 library
passes are included in that full total, not additional tests. The latest font and
leading reset assertions were then rerun in the 29-case styling suite. These local
runs are development verification; committed Cairn receipts must still refresh.

Ripwire edit-check found no EventTree arity break. The initial TextStyle lookup
was ambiguous; the qualified private helper lookup found zero incompatible calls
and reported the shared-name definition count, not a public signature change.
Quality-delta exited 2: reviewed changed-code complexity in App::render, the legacy
painter and the wrapping helper; state acknowledgement and explicit word/grapheme
branches account for the changes. Most gate rows refer to ignored reference trees
or name-based dead-code false positives. No quality gate pass is claimed.
Test-gate exited 4 and named broad widget/legacy surfaces, reference-tree symbols
and integration demo files; compiled default targets ran, while actual embedded
and native acceptance stays with the inherited mechanisms. This is incremental
API-009 verification, not the final catalog review.


## API-008 refresh after styling: native cancellation fixture

Run 34317275077 built Windows successfully, but its cancellation fixture returned
the correct five-second timeout before the PowerShell child published its PID.
Cancellation was conditional on that publication, so the test never requested it.
The following deadline test passed. The Windows clipboard round trips had not yet
run; macOS and the three Linux backends passed at source commit 1c8c4f0.

Replace the shell sleeper with an ignored native test-binary child invoked only
by the two lifecycle tests. It publishes its PID and sleeps. On Windows, retain a
handle to that live process before cancellation and require the same handle to be
signaled after stop. This avoids shell startup in the cancellation trigger and
avoids a second PowerShell query or PID-reuse ambiguity in the exit assertion.
The production five-second Windows/two-second Unix deadlines and native clipboard
round-trip requirements remain unchanged. Fresh platform records must use this
fixture digest before acceptance; collected earlier records remain historical.


Native run 34318193514 passed the revised Windows lifecycle checks and reached
the real clipboard probe. The first 13-byte copy timed out at five seconds.
The previous five-second decision explicitly says it is wrong if normal startup
still exceeds the allowance. Before changing that decision, add a diagnostic job
on a separate fresh Windows runner to time the exact copy/paste scripts under a
30-second observation bound. It does not warm the acceptance runner, and its
results are diagnostics, not substitutes for the production-deadline checks.


Cold-start diagnostic job 102362246907 in run 34319319017 used a separate fresh
Windows desktop: copy took 3.250 seconds; paste took 0.343 seconds; the following
four calls took 0.266, 0.266, 0.281 and 0.281 seconds. Those timings do not bound
other runners: both the earlier readiness fixture and the first real Rust copy
exceeded five seconds. Record a Consequential replacement decision allowing up
to fifteen seconds on Windows, with a separate sixteen-second test assertion.
Unix remains two seconds. Remove the temporary diagnostic job; cold acceptance
must pass without any preceding PowerShell fixture or diagnostic in that VM.
Run 34319319017 was canceled after the diagnostic to avoid another redundant
five-second candidate run. Another normal-startup failure under the new allowance
will return to the developer rather than trigger another arbitrary increase.


## API-008 cold-start correction verified

Native run 34319776534 passed macOS and Windows at 50e5bfd. On Windows the five
exact clipboard cases passed in 6.47 seconds total; two lifecycle checks passed
in 15.01 seconds, including the forced fifteen-second deadline, and the private
console adapter test passed. No PowerShell warm-up fixture or diagnostic preceded
the real clipboard calls on that runner. The separate earlier timing job was
removed before this candidate. Wayland, Xsel and Xclip also passed at the same
source digest. All five records and both output hashes per backend were checked
before copying; the platform verifier reports all five current. Existing Windows
compiler warnings remain in the API-019 inventory; no native strict-Clippy pass
is claimed. The Consequential deadline decision remains queued for developer
review, as required by the working agreement.

## API-010 development and mechanism challenge

The mechanism exercises the real App/SuprTUI route: explicit styles and gradient
metadata survive typed component props and expansion, gradients have independent
expected cell colors, and animations produce intermediate frames without input.
Deterministic clock cases cover transitions and interruption, key continuity,
removal, independent Apps, named property sets, nonfinite values and border cycles.
The current development run passed 28 captured-frame cases and ten clock/property
cases. Strict default-feature all-target Clippy also passed after boxing the
private worker Element message; this introduces no public interface change.

Three deliberately violating local variants each compiled and failed behavior
assertions. Stripping gradient metadata in the bridge failed the independent
endpoint test. Disabling the App motion wake flag failed the bounded intermediate
frame test. Restoring degree conversion inside the radians TransformProperty path
failed the quarter-turn matrix assertion. The original source bytes were restored
after every probe. Captured failures are in api010-gradient-bridge-negative.txt,
api010-animation-wakeup-negative.txt and api010-rotation-units-negative.txt beside
this review. They are negative controls, not acceptance passes.

One new static-border test initially waited for unsolicited frames; its timeout
was a test scheduling error. The corrected test forces a resize to inspect another
static frame. This preserves the idle-wait contract. A previous wide-grapheme alpha
test was corrected to inspect color on the leading cell and assert continuation
on the second cell, matching VT100's representation.

The developer explicitly approved radians for TransformProperty::Rotate while
keeping degree-based Rotation/CSS helpers, and the new optional GradientBorder
cycle_duration field. The answered escalations and corresponding decision records
preserve those compatibility choices. Public constructor signatures and the C ABI
are unchanged. docs/paint-properties.md states the struct-literal migration,
unit semantics, upright glyph approximation and rectangular transformed hit areas.

Inventory boundary: this requirement establishes style/property delivery into
App's supported painter. The separate animation target APIs, CSS manager target
binding, typed-keyframe conversion, relative bases and screen transitions remain
explicit API-013 work; they are not declared repaired by these paint checks.

Ripwire edit-check found no incompatible callers for the new property projector.
Its quality-delta and test-gate do not pass: the index includes ignored reference
projects and produces false dead-code/cross-project duplication findings (including
executed tests and width_px/height_px calls visible in this change). Real complexity
growth was inspected in the property dispatch, affine/clipping painter and App
clock traversal. These branches implement distinct value types, clipping and
lifecycle cases covered by the focused checks. No suppression or baseline reset
was used. The relevant regression paths also run in the default full suite and
Cairn's inherited mechanisms. Final commitment-wide review remains pending.

The first full default-suite run found two compatibility assertions for the
existing css-in-rust-applied class marker. Restored that marker alongside the
functional style metadata, preserving the observable builder output without
using the marker as a substitute for painting. All 14 css_in_rust_simple_test
cases passed after this repair; the full suite is rerun before acceptance.

The corrected full default-suite run exited zero: 1,167 passed and 37 ignored
across 63 test groups. Formatting and git diff whitespace checks passed. The
ignored cases are unchanged; no ignored case is counted as acceptance.

## API-011 Accordion work and unresolved accessibility contract

The new App test accordion_builder_opens_real_content_and_skips_disabled_headers
failed against the old builder with an input deadline: opening the named widget
never produced the required content frame. The repaired named builder preserves
its component name and uses the built-in factory. The public Accordion remains a
unit struct; a retained private child owns measured headers and motion resources.
Seven App cases now cover expansion and disabled navigation, measured multi-line
custom headers, root callback IDs and mode constraints, empty/disabled/release
input, nested input retention and resize, intermediate animation/reduced motion,
and persistence when supplied props change. These run at two viewport sizes.

Editing verification: the combined api_widget_behavior, api_styling and
layout_paint_tests run passed 100 tests. The six accordion library tests passed,
including bounded spring timing, stagger, reversal and timer cleanup. The scoped
notification isolation/abandonment unit test passed separately. Formatting and
git diff whitespace checks passed. These are editing checks, not Cairn receipts;
API-011 remains in progress and its catalog is not complete.

A focused source examination found that Accordion's full ARIA/screen-reader claim
has no delivery path. AccordionSection and BreadcrumbSegment store aria_label,
but no code reads those fields. apply_aria_label, apply_aria_labelledby,
apply_aria_describedby and apply_aria_live in src/layout/css/accessibility.rs
return the unchanged StyleBuilder. Their unit checks only test parser acceptance.
No accessibility claim was replaced with those passing tests. The proposed
terminal/reader guarantee and observable acceptance criteria are recorded in
docs/widget-acceptance.md for a developer decision. Accessibility implementation,
remaining catalog families and the final commitment review remain outstanding.

## API-011 screen-reader construction checks

The real App/SuprTUI fixture now exposes labels that differ from its painted
headers. The isolated GNOME Terminal / Orca workflow at 60x16 passed label and
button-role announcements, collapsed/expanded state, disabled-action exclusion,
assistive focus on a nested input and another header, keyboard activation,
removal, exactly-once callbacks, and cleanup/state isolation across two App
instances in one process. These are editing checks, not Cairn receipts. The
smaller viewport and semantic-label negative control are still pending.

The test first found that focus-only nodes lacked an AT-SPI Component interface.
The recorded successor decision preserves that interface without inventing pixel
bounds. Four focused tests fail with the original upstream state/focus mapping
and pass with the repaired mapping restored. They check expandable/expanded,
disabled enabled/sensitive state, read-only enabled inputs, and focus without a
click action. The real workflow also failed when switching away from the host
left accessible focus set. Enabling terminal focus reports and disabling them on
restoration fixed that case. The full strict all-target Clippy check passed.

An early disposable launch used the inherited display for D-Bus activation.
Those App runs are invalid evidence; their diagnostic directories were removed,
including unintended input captures. The corrected runner creates the private
XDG environment and Xvfb display before starting D-Bus, passes the display
explicitly to GNOME Terminal, and verifies a mapped window on that display
before sending input. The desktop accessibility settings were read afterward;
both toolkit-accessibility and screen-reader-enabled remained false. Tests use
Orca's ordinary reader implementation; a private customization only makes its
real debug log line-buffered for bounded observation. Orca's launcher excludes
other instances by user rather than D-Bus session, so these tests run serially.

The same real-reader workflow subsequently passed at 32x10. The negative fixture
paints the same headings but omits their semantic labels; the workflow rejected
it at the missing-semantic-label assertion. A first attempt to run two readers
concurrently was rejected by Orca's per-user single-instance guard, not counted
as a negative-control pass. Serial execution passed. The mechanism retains both
viewport runs, the missing-label negative control, and the four state/focus unit
cases. All of this remains editing verification until the implementation and
mechanism are committed and Cairn records current evidence.

Remaining accessibility review obligations include semantic state for the other
catalog controls and CSS utilities, clipped/hidden focus ownership, transport
failure reporting and boundedness of the included adapter's internal message
queue. The owned App action receiver is bounded at 64, but the upstream adapter
uses an unbounded outbound queue; do not describe the complete transport as
bounded before that concern is resolved and tested.

## API-011 Breadcrumb implementation checks

The baseline failed all three new App workflows: named builder output, keyboard
callback delivery and measured mouse targeting. The padded fixture originally
had an eight-row viewport with eight rows of top padding (p-2 uses the existing
CSS spacing scale); its corrected 20/24-row viewports expose the intended target.
That fixture correction was not counted as a product fix.

The recorded retained-child decision preserves Breadcrumb's public unit type,
props, state and named/typed builders. The child measures actual segment widths,
fits separators and overflow into the allocated viewport, wraps and scrolls real
layout children, handles measured pointer targets and hover tooltips, and emits
one deferred JSON navigation notification. Current and nonclickable segments do
not activate. Public labels are carried independently from painted labels.

Ten App cases passed at two sizes, including all overflow strategies, keyboard
and mouse callbacks, wrapping, scrolling to focused segments, tooltips, resize,
prop reorder with focus retention, empty controls, disabled keyboard navigation
and key release. Strict all-target Clippy passed before the last three tests were
added; full widget regression and updated checks are running separately. Reader
coverage for Breadcrumb and current-page attribute translation remain pending.


## API-011 breadcrumb reader and replacement-focus construction checks

The isolated real App/Orca workflow exposed a replacement defect: after a
focused Accordion was replaced with Breadcrumb, assistive focus announced Home
but keyboard Right did not advance. A local App case reproduced the mismatch:
clicking Root and pressing Enter delivered Root then Docs instead of Root twice.
The event tree reused the flattened layout slot while the retained component had
been replaced, so the new child missed its focus notification. The runtime now
retains an App-local mount identity chain in expanded metadata; event, styling
and accessibility paths use that chain with the existing keys. Replacement gets
fresh targets and focus delivery. The corrected click/Enter/Right case passes.
All five component-expansion tests, 12 focus tests and then-current 79 widget
tests passed after this change, including keyed reorder and focus traps.

The next reader assertion failed because the current breadcrumb had no AT-SPI
Active state. AccessKit already retained AriaCurrent::Page. The maintained
translation now supplies Active and the exact current attribute token, following
https://w3c.github.io/core-aam/#ariaCurrent. Five adapter tests pass, covering
absent/false and all seven current tokens, focus/selection independence, existing
window active behavior, disclosure state, disabled/read-only state and focus
without pixel bounds. The first speech assertion used lowercase current page;
Orca actually emitted static (Current page). Correcting that test's case made
the complete workflow pass; no reader output or state was manufactured.

The final reader workflow passed serially at 60x16 and 32x10 with GNOME Terminal
3.58.0/VTE 0.84.0, Orca 50.1.2 and AT-SPI2 2.60.0 in the isolated X11 session.
It checks distinct breadcrumb labels and Link roles, inert current/disabled
activation, assistive focus followed by keyboard focus, one navigation callback
per activation, and removal. Orca's standard Ctrl+Orca+Right object navigation
reads the inert current item and announces Current page. Speech after keyboard
Right must occur after that action, rather than match the earlier autofocus
announcement. The separate negative run still rejects painted labels when the
semantic labels are removed. Both successive App instances detach cleanly.

Fourteen breadcrumb App cases now pass, adding replacement focus, home/icon and
compact/noncompact options, segment classes, inert mouse targets, signed/zero/
large wheel deltas and mouse targets after scrolling. One new spacing assertion
initially stripped spaces on only one side; correcting its expected normalization
made it pass without a product change. Strict all-target Clippy passed after the
runtime/adapter repairs; the later test-only additions still need that final pass.
These are editing checks, not committed Cairn receipts. The catalog, shared
accessibility helpers, outgoing transport bounds and error lifecycle remain open.


## API-011 closing-body input construction check

A two-second collapse test confirmed a closing accordion body stayed in Tab order:
Tab/Enter activated its inner button even though its semantic subtree was hidden.
The initial test fixture used the button's large default padding and clipped its
label; setting explicit one-row bounds made the intended failure observable.
The fix retains and paints the body with an inherited internal inert flag. Event
registration removes its focus, handlers and hit targets, and excludes its traps;
accessibility traversal hides the same subtree. Layout callbacks still run and
component instances stay mounted. Reopening restores ordinary registration.
The corrected test checks the body is still painted, attempts a pointer click,
and confirms Tab/Enter reaches the outside button without activating the inside
button. All nine accordion-related App cases passed, including retained input
across collapse/reopen and the breadcrumb replacement regression. The broader
focus/component/widget rerun is pending at this entry. No Cairn receipt claimed.


## API-011 bounded transport construction checks

The source audit found the private upstream Unix adapter used an unbounded
process-wide outgoing queue and an unowned worker, ignored most bus errors and
could not restart after that worker failed. The retained state translation was
correct for the reader fixture but did not establish the required ownership or
reliability. The new judged transport decision is implemented with one application
context and worker per App, a 4096-message queue, separate cancellation, returned
failure state and three-second asynchronous bus-operation deadlines. App wakes on
failure, checks it during publishing/input, cancels and joins the adapter before
terminal cleanup. The incoming queue and per-turn action drain are bounded to 64.

Five transport tests pass for overflow, cancellation during a pending operation,
worker panic, independent/later connections after failure and the operation
deadline. An incoming-action overload test passes. A temporary negative control
silently discarded outgoing overflow; the overload test failed on its expected
error assertion. The source was restored and all five transport tests passed
again. The negative-control log is /tmp/rtui-a11y-overload-negative.log; it is
editing evidence, not a Cairn receipt. The broader accessibility unit selection
passed 26 cases, including upstream cache/translation tests. The three existing
CSS recognition tests in that selection still do not prove semantic behavior.

The private-bus App fixture returns an error and restores raw mode and the
alternate screen for a missing endpoint (about 0.06 seconds) and an endpoint that
never answers authentication (about 3.05 seconds). Automatic mode without a
desktop bus also accepts ordinary F9 input and cleans up successfully (about 0.53
seconds). The first harness version failed to drain PTY output while waiting;
its full output buffer blocked the renderer before App could observe the error.
Concurrent draining repaired the harness without changing the product.

The real GNOME Terminal/Orca fixture now keeps an independent App alive while the
first foreground App closes and a second starts. Its action counter advances
once before and once after the first App closes, and all three adapters disappear
at final shutdown. This passed at both 60x16 and 32x10 along with the accordion
and breadcrumb reader workflow. The final 32x10 run used the current fixture,
including its automatic-mode option. The missing-semantic-label negative rerun
is pending at this entry. All 83 widget tests, 12 focus tests and five component
expansion tests passed after the transport changes. Strict all-target Clippy,
format checking and diff whitespace checks also passed. The catalog and shared
accessibility CSS behavior remain unfinished; no API-011 acceptance or current
committed Cairn evidence is claimed here.


## API-011 accessibility styles and frame publication

Source review found that the shared CSS helpers discarded values, roles, tab
indices and live-region settings. The repaired path retains those attributes in
StyleBuilder snapshots, validates label references against App-local IDs, and
uses the presented semantic candidate for event registration and reader updates.
Base disabled semantics are established before selecting disabled style variants.
Missing values, invalid booleans, duplicate IDs, missing references and cycles
fail before presenting a candidate. Existing Element and widget label APIs remain.

Attacked snapshot transport, reference labels/descriptions, false state overrides,
keyboard order, inherited disabled controls, keyboard-only hit targets, resize,
screen-reader-only text and animation cleanup. App tests exposed sr-only forcing
size zero permanently; visibility now carries a reversible flag, and nested
reader-only content stays in the reader tree even without cell geometry. Two App
workflows pass at 24x6 and 48x12. Three semantic unit cases and the owned reduced-
motion clock case pass. The public value-taking helper doctest passes.

The real Orca fixture now includes CSS labels that differ from painted text,
label/description references, heading and toggle roles, expanded/selected/checked
states, hidden and restored content, nested reader-only content, assistive focus,
pressed changes and a polite live announcement. An initial fixture attempted to
assert host focus for the deliberately noninteractive background App; those
assertions now run in the actual GNOME Terminal foreground App. Another fixture
combined expanded and pressed on one button; Orca prioritizes the disclosure
announcement, so the two semantics now have distinct controls. Neither fixture
error justified changing the adapter or weakening the state assertions.

The 32x10 live speech check also exposed Orca 50.1.2 suppressing same-App
announcements within 100 ms. Its own log recorded delivery followed by that
spam-filter decision; installed event_manager.py lines 343-353 implement it.
The fixture now pumps its event loop for 150 ms between initial announcement
and activation, preserving the speech assertion and leaving Orca unchanged.
The corrected 32x10 reader run passes; the prior 60x16 run passed all these
semantic cases. Both use GNOME Terminal 3.58.0/VTE 0.84.0 and AT-SPI2 2.60.0
in the isolated GNOME X11 session. The CSS-negative fixture must fail to locate
the semantic control after removing its styles; it has rejected that case.

Current widget/focus/component regression run: 102 tests passed. The complete
accessibility-filtered library run passes, including bounded transport and
action-queue tests. Strict all-target Clippy passed during this change; final
format/lint checks follow the last fixture edits. This is implementation review,
not a passing API-011 receipt: the catalog matrix still contains pending rows.

Final checks after the fixture edits: workspace format check, strict all-target
Clippy and git diff whitespace check all passed. The final CSS-negative run
rejected the missing semantic control. The widget inventory remains pending.


### API-011 implementation checks: Table and DataTable

This records local implementation checks, not a completed commitment review or
Cairn evidence receipt. API-011 and the remaining catalog are still in progress.

Examined the public Table/DataTable props, states, constructors, builder and macro,
the named component registry, App layout/input callbacks, shared disabled-state
preparation, and the retained breadcrumb/scroll ownership patterns. The old Table
assumed width 80 and origin zero, ignored widths while painting, split mouse and
keyboard selection between incompatible fields, and guessed wheel direction from
pointer position. DataTable painted an informational description, lacked controls,
ignored virtual scrolling, and its builder dropped filters and hidden columns.

The retained table now measures its viewport and cells, clips the body separately
from headers and borders, uses signed wheel input, and retains source-row identity.
Widths, alignment, styles, border shapes/color, sorting, selection, actions and
column dragging reach frames and App callbacks. Private full-sized row offsets
preserve access beyond the public compatibility state's 16-bit offset. Duplicate
row/column IDs, contradictory widths, invalid numeric widths and invalid sort
indices produce inert errors. Error text wraps so the diagnosis remains readable.

DataTable retains query and control state, applies filters and stable sort before
paging, translates callbacks to source indices, preserves selections across pages,
and keeps the current row independent of the selected set. Its controls use the
working button/input/scroll components. Virtual scrolling uses the configured row
count within measured terminal space, retains overscan rows and clips their input.
The builder retains filters and hidden columns, and enables its advertised sorting.
Removed the obsolete filter-copy helpers; the App tests exercise the live view and
shared cell predicate, including stale counts and changed filter props.

Failures found during implementation included a missing redraw after first layout
for fixed columns, drag state cleared by ordinary redraws, missing border glyphs
and content clipping, an unconstrained long table that could not reveal its last
row, selection collapse when DataTable changed its current row, and inherited
Element disabled state not reaching nested input. Corrected each and kept the
corresponding App regression. Two test additions initially sent input before a
panel was presented or stopped before its change was painted; corrected their
frame gates without changing the content/callback assertions.

Safe violating examples were tried and restored:
- Changed the measured cell handler to route to `(row + 1) % rows.len()`. The named
  Table App workflow failed its source-index/callback assertion. Diagnostic output:
  /tmp/api011-table-wrong-row-negative.log.
- Removed the DataTable builder's filter assignment. The real builder App workflow
  failed because Zed remained visible beside Ada. Diagnostic output:
  /tmp/api011-datatable-dropped-filter-negative.log.
The corrected cases subsequently passed in the combined Table/DataTable App run.

Verification run locally: 164 tests passed across api_widget_behavior, api_focus,
api_event_routing, api_component_expansion and api_styling after the shared disabled
fix. Subsequent targeted runs cover twelve Table and fourteen DataTable App cases
at two sizes; the public Table unit suite passed 11 tests and DataTable range
boundaries passed 2. Strict all-target Clippy and diff whitespace checks passed.
The default Orca probe was rebuilt and its 32x10 GNOME Terminal workflow passed
labels, roles, focus, disabled actions, expansion, nested input, breadcrumbs, CSS
semantics/live announcements, App isolation and cleanup. Environment: GNOME
Terminal 3.58.0/VTE 0.84.0, Orca 50.1.2, AT-SPI2 2.60.0, isolated X11/GNOME/Xvfb;
logs: /tmp/rtui-orca-ui0u8ske. This is the existing reader workflow, not new Table
screen-reader acceptance. The full API-011 mechanism still rejects pending rows.


### API-011 Tree implementation checks (not the final commitment review)

The Tree convenience builder painted a description, the named factory was absent,
mouse hits used guessed rows, redraws reset expansion, and lazy-load results were
ignored. The public unit component now renders a retained App child with measured
expander, checkbox and label targets. It retains state by ID, uses returned lazy
children, filters before hit testing, and implements bounded visible rows, callbacks,
per-node controls, in-memory drag/drop with root/cycle rejection, and explicit errors.
Public TreeProps and TreeState layouts remain intact. Builder callbacks and classes
reach the live control. The old guessed mouse fallback was removed.

Fourteen App tests pass at 24x8 and 48x14 (with resized viewports where applicable).
They attack both builders/typed construction, expansion persistence, returned and
initially expanded lazy children, duplicate/missing-loader errors, multi-selection
cursor behavior, per-node flags, checking, filter hit order, virtual row bounds,
negative wheels, drag/drop cycles, keyed prop replacement, duplicate mouse events,
key release, borders/classes, padding/resize and disabled/empty roots. A temporary
source mutation discarded the lazy callback result. The real App workflow failed;
restoring the source made the corrected suite pass. Negative log:
`/tmp/api011-tree-discarded-children-negative.log`.

A padded auto-sized parent exposed zero intrinsic width from absolute rows. The
same failure was reproduced for Table before repair. Both controls now contribute
intrinsic width without moving their scrolling viewport; the new Table App test
passes and Table coverage is now thirteen cases. The full current widget suite
passes 126 tests, and component expansion (5), event routing (9), focus (12) and
styling (29) also pass: 181 tests total. Strict library/widget-test Clippy passes.
These are editing checks against the unfinished action, not Cairn receipts or a
catalog-wide readiness claim. Tree semantic roles/states are present, but these
Tree tests are not a new screen-reader integration acceptance claim.


### API-011 FileExplorer implementation checks (catalog acceptance pending)

The retained FileExplorer now connects every builder route to App input, measured
layout and an owned filesystem worker. Seventeen App workflows passed, including
copy/move/rename/delete with confirmation and cancellation, native filename callback
payloads, sorting/filtering, preview completion while idle, all views, changed props,
disabled/empty/error states, padded auto-sized parents and viewport resizing. Eight
worker tests passed for rooted reads, external symlinks, non-UTF8 paths, preview
bounds, cancellation/stale responses, owner cleanup, exclusive destinations and
permission-preserving directory copy. A broader run of widget, component expansion,
event routing, focus and styling tests passed all 198 cases.

Failure demonstrations: temporarily replacing Unix RenameFlags::NOREPLACE with
empty flags made worker_never_overwrites_and_failed_copy_preserves_source_and_cleans_staging
fail at the existing-destination assertion (exit 101). The original source was
restored byte for byte and all eight worker tests passed again. Diagnostic output
is in target/api011-fileexplorer-overwrite-negative.log, not a Cairn receipt.
Earlier new tests exposed an unreachable final grid entry with a visible-item
limit smaller than the column count, and copied directory mode 0755 instead of
0700. Whole-row grid capacity and preserved directory permissions corrected these;
the corresponding App and worker cases now pass. Late destination collision also
checks removal of read-only private staging without changing source permissions.

The full library and api_widget_behavior test target compile for
x86_64-pc-windows-gnu using the existing cross tools. Five warnings remain in
capabilities/platform/router outside the new FileExplorer code. This is compilation,
not native Windows execution. Native Windows/macOS operation behavior and actual
Orca delivery for FileExplorer remain unverified. Worker cancellation cannot
preempt an OS filesystem call already running; no hard shutdown deadline is claimed.
The matrix stays PENDING until the remaining catalog and accessibility reviews.
No formal check or completion receipt is claimed for this uncommitted action.


### API-011 Chart implementation checks (catalog acceptance pending)

Chart and Charts now resolve both builder routes to a retained measured renderer.
Fourteen App cases passed across multiple viewport sizes. They assert distinct
geometry for all seven modes, exact point coordinates and RGB colors, visible
series and palette precedence, actual area fills and donut holes, signed bars,
line/fill styles, explicit axis clipping, axis titles/grid/custom ticks, legend
coordinates and width, hover metadata, keyboard details, disabled input, changed
props and padded resize. New series default to solid area fill; explicit None
still disables it. Invalid dimensions produce a readable fallback error.

The tests found and corrected erased shared line endpoints, clipped tooltip
metadata, and a duplicate mouse coordinate conversion inside padded parents.
Line segments now intersect axis bounds in data space; out-of-scale scatter
points are omitted instead of clamped into false edge points. Pie uses scaled
weights to avoid overflowing a finite-value total and binary search over
cumulative sectors, rather than scanning every point for every cell.

The finite reveal timer has a deterministic test for quarter progress, completion,
restart, reduced motion and removal; it passed. App frames separately show actual
intermediate bars. A temporary violation suppressed solid area fill: the distinct
geometry App test failed (exit 101). The original file was restored byte for byte;
its diagnostic is target/api011-chart-area-negative.log. After the final pattern
fill changes, all 220 tests across widget behavior, component expansion, event
routing, focus, styling and the original chart suite passed. Strict library/widget
Clippy and cargo fmt --check also passed. Ripwire reports Chart's public shape
unchanged and LiveChart as a new internal symbol; those structural results are not
behavior evidence. The catalog and actual reader acceptance remain pending; no
formal Cairn receipt is claimed for this uncommitted action.


### API-011 ProgressBar implementation checks (catalog acceptance pending)

The generic builder now constructs ProgressBar props instead of descriptive text.
The retained child paints measured horizontal thickness, vertical height, bounded
segments, stripes and configured colors/styles. It owns scheduler deadlines for
200 ms value transitions, indeterminate movement and pulse. The public component
owns completion detection; valid initial completion and later crossings invoke the
callback once, while invalid and indeterminate values cannot report completion.
Props comparison now observes changed formatter and completion callback Arcs.

Ten App tests passed at multiple sizes, with precise cell/count/color assertions,
all builder/factory routes, signed ranges/clamping, custom formatter inputs and
replacement, callback transitions/replacement, own padding and resize, invalid
input, disabled interaction, reduced motion and intermediate animation frames.
The initial-completion test exposed a missing initialization callback; setting
initial component state through the same completion transition corrected it.
Twelve unit cases passed, including deterministic interpolation, reversal,
completion, reduced motion and timer release on drop. Old private renderer tests
now exercise the cell-selection function used by the live painter.

Temporarily suppressing determinate fill made the App geometry test fail with
zero filled cells instead of twenty (exit 101); the source was restored byte for
byte. Diagnostic: target/api011-progress-fill-negative.log. The corrected broad
run passed all 231 tests across widget, component, event, focus, styling and chart
suites, and all twelve progress unit cases passed again. Strict library/widget
Clippy and formatting passed. Ripwire keeps ProgressBar's public shape unchanged
and identifies LiveProgress as a new internal class. No formal receipt is claimed.

The same work checked Chart's own padding, in addition to its padded-parent test.
Its canvas now sits at the measured content insets; the fifteenth Chart App case
passes for painted point placement and hover metadata inside that box. Actual
reader acceptance and the remaining widget families are still pending.

### API-011 popover implementation checkpoint (2026-09-09)

Work is still in progress; this is not final widget acceptance. Private capture metadata now survives component expansion and runs before consuming child controls. Three focused registration tests passed. Changing capture registration to bubbling made the ordering assertion fail (exit 101; `target/api011-capture-negative.log`); the original source was restored and all three checks passed again. Hover boundary delivery now includes capture handlers, with a focused own-boundary test passing.

Popover now retains visibility and measured geometry, renders its trigger while closed, opens actual child content, closes via Escape/outside clicks, and keeps child callbacks intact. Its private rendered child cancels scheduler deadlines even while a caller retains the public handle. A private nontrapping focus scope selects real expanded descendants and restores focus without stealing it after Tab has left the overlay. Nine App tests and 19 popover unit tests passed during editing, including all twelve positions at two viewport sizes, generic-builder defaults, hover deadlines, imperative idle wakeup and callback reentry. Strict library/widget-test Clippy passed. Remaining work includes arrow and boundary/resize acceptance, prop replacement, nested overlays, animation frames, validation, accessibility delivery and the rest of the widget inventory. No formal Cairn receipt is claimed.

### API-011 Popover placement and lifecycle checks

The expanded App cases cover all four boundary modes, intrinsic overflow before
terminal clipping, resize placement and initial position callbacks, three arrow
styles in four directions, arrow/body mouse targeting, all four animations, mixed
nested focus traps, callback/content replacement, independent dismissal flags,
invalid anchors and size constraints, empty content and disabled triggers. The
new tests exposed and repaired suppressed initial position callbacks, premature
viewport clamping in Ignore mode, disabled trigger observation, leaving the host
at the origin, and disabled Escape dismissal reaching App's quit fallback. The
Escape unit expectation now requires consumption without closing; the real App
case establishes why returning Ignored was wrong. Arrow transforms follow the
body, stale arrow positions clear, and generation is bounded by the viewport axis.

A safe mutation replaced measured body size with a fixed 10-by-5 size. The actual
placement assertion failed with exit 101; output is in
`target/api011-popover-guessed-size-negative.log`. The source was restored exactly.
After the final behavior edits, all seven related integration suites passed: 281
tests across widgets, events, focus, component expansion, styling, painting and hook
lifecycle. Twenty-one Popover unit cases passed, including controlled-clock
reversal/completion, hover cancellation, changing the retained public owner and
unmount cleanup. Strict library/widget-test Clippy passed. These are editing
checks, not committed Cairn evidence.

The arrow animation test uses a one-second duration so the measured frame can
show an intermediate scale before terminal-cell rounding reaches its final
appearance. Both 24-by-16 and 48-by-24 viewports pass. The screen-reader workflow,
remaining catalog families and final acceptance review are still pending.

### API-011 Modal implementation checkpoint

The first three App probes failed against the old Modal: fixed viewport
placement, hidden generic-builder content, and named-route button completion.
That baseline is retained in `target/api011-modal-baseline.log`. The retained
implementation now passes fifteen cases covering geometry and sizing, actual
child input and scrolling, close reasons, all resize edges, title dragging,
callbacks and focus restoration, prop replacement, animation, invalid sizes
and region styles. Eleven unit cases include controlled-clock reversal and
completion, unmount cancellation, and duplicate action suppression.

The old outer Modal handler intercepted Tab before App could move real focus.
It also emitted another close callback after the retained owner had already
closed; `target/api011-modal-duplicate-close.log` records that failure. Removed
those competing event and frame-counter animation paths. The public unit type,
props/state types, enums and construction helpers remain. The private child now
owns dispatch and lifecycle for all construction routes, including manual render.
The former index-only keyboard and frame-counter unit tests were replaced by
actual App navigation and controlled-clock lifecycle checks.

Cell-color checks found white text on Modal's white background, and then exposed
missing foreground inheritance in the shared layout builder. The default Modal
foreground is now black. The builder carries ancestor foreground colors unless
a child provides its own; a direct failure probe and corrected case also verify
sibling isolation (`target/api011-foreground-inheritance-baseline.log`).
After these edits, 296 tests passed across the seven related integration suites,
strict library/widget-test Clippy passed, formatting ran, and the whitespace
check passed. Ripwire reports a private builder parameter change with no known
incompatible callers. No committed Cairn evidence or catalog completion is claimed.

Modal still needs the remaining dismissal-option, stacking/nesting, closing-body
and public-route checks, plus actual reader delivery and final catalog review.
Other pending widget families remain part of API-011.

### API-011 Modal additional implementation checks

Added App workflows for dismissal flag combinations, explicit actions when dismissal is disabled, viewport drag clamping and focus-loss cancellation, visible generic construction with multiple children, public prop helpers and empty content, overlapping mouse z-order, nested Escape/focus restoration, inert closing fades with immediate reopening, and invalid-dimension recovery. All 23 modal module tests pass at the tested viewport sizes. Two new close assertions initially stopped the harness before its next presentation; they now wait for the post-input frame. This was a test sequencing correction, not a runtime defect.

The broad library run exposed seven stale builder tests expecting descriptive text or a short registry name. Replaced these expectations with real component/props/child checks, consistent with the existing App acceptance workflows. The corrected library run passed 814 tests, with two ignored; TTY-dependent legacy cases report skips in this headless run. The prior failed run is retained at target/api011-builder-unit-baseline.log. This remains an implementation checkpoint, not the final Cairn review or complete catalog acceptance.

### API-011 menu baseline

Four new App probes in tests/api_widget_behavior/menus.rs fail before menu repair: MenuBar, PopupMenu and DialogMenu do not paint their item labels; ContextMenu does not open a painted menu after right-click. Captured failure output is target/api011-menu-baseline.log. Source inspection confirms empty menu render containers, missing measured mouse targets, discarded generic MenuBar callbacks and a checkbox constructor that captures the original boolean forever. These are implementation findings under API-011, not accepted behavior. Menu repair and full interaction coverage remain pending.

### API-011 menu repair checkpoint: callbacks and menu bar

The menu checkbox constructor captured its initial checked value. A public item execution probe changed the current item state and observed two true callback values instead of true then false. A second probe replaced an action with the same ID and found props compared equal. Both failed before repair (target/api011-menu-callback-baseline.log). The scoped argument adapter and callback identity comparison now pass six focused tests, including nested/direct invocation, panic cleanup, cloned actions and concurrent scopes. These tests establish the adapter contract, not App isolation for the whole catalog.

The retained menu bar paints actual items and uses measured item and panel layouts. Seven App tests pass: visible items at two viewport sizes, nested action delivery with disabled-item skipping, generic builder callback retention, checkbox state across openings, pointer activation at painted positions, Home/End scrolling, and disabled/empty controls. The nested probe initially failed because the submenu painted off screen; measured panel bounds now keep it visible. Strict library/widget Clippy passed before the final Home/End and visible-target additions; rerun remains required. Menu bar shortcuts, separators/styles, hover/outside dismissal, callback/prop replacement and reader delivery still need acceptance. Popup, context and dialog menu render baselines remain failing and are not repaired by this checkpoint. API-011 remains in progress.

### API-011 menu shortcut and identity work, 2026-09-10

Added three real App workflows at 32×12 and 60×20. Before repair, Ctrl+S called no action; after a prop reorder, Enter called ALPHA instead of the selected BRAVO; arrow navigation invoked leaf actions. Their failing development outputs are preserved in target/api011-menu-{shortcut,reorder,direction}-baseline.log. These are diagnostics, not Cairn receipts. The corrected ten menu-bar workflows passed before and after extracting the shared MenuModel and MenuView. Shortcut traversal excludes hidden/disabled ancestors and requires exact modifiers. The reorder workflow also verifies the newly supplied callback. Arrow navigation opens submenus and leaves leaf invocation to Enter/Space.

MenuModel now owns retained item values, identity reconciliation, selection, action-state mutation and shortcut traversal. MenuView owns item rendering and measured pointer targets. Popup/context/dialog integration remains unfinished; API-011 is not accepted. Separators, shadow/width options, hover/outside/wheel behavior, state seeds, additional lifecycle and reader acceptance still require work.

### API-011 popup hover repair (2026-09-09)

The App regression `popup_menu_hover_opens_and_preserves_the_nested_panel` failed before the repair (0 passed, 1 failed): hovering RECENT never painted OPEN. The captured output is `.cairn/reviews/api-011-popup-hover-baseline.log`. After the repair the same command passed at 32x12 and 60x20. It also repeats hover on the parent before Enter, proving the open child remains selected and its action fires once. Popup outside dismissal now restricts measured panels to currently open depth and clipping. That dismissal change still needs its own regression. These are development runs, not Cairn evidence.

### API-011 menu row-height repair (2026-09-09)

`popup_menu_scroll_keeps_selected_rows_visible_with_separators` failed before the repair: End selected the last item but its row was clipped, so the App workflow could not proceed. Output is retained in `.cairn/reviews/api-011-menu-scroll-separators-baseline.log`. Shared panels now budget measured outer row heights, including attached separators, alongside panel insets and header/footer height. Before a row is measured, its text line count and separator supply the initial estimate. The same test passed at 32x8 and 60x10; all 30 menu App workflows passed together. Strict default-feature all-target Clippy passed. These runs are development verification, not committed Cairn receipts.

### API-011 menu wheel routing (2026-09-09)

`menu_wheel_navigation_reaches_actions_through_app` failed with no second-item callback for the menubar before the repair. Its baseline is `.cairn/reviews/api-011-menu-wheel-baseline.log`. Menubar and dialog menu had no wheel handler. The shared menu model now navigates the panel under the pointer, bounds numeric deltas, rejects nonfinite/zero values, and handles line or pixel wheel input. The corrected App test passed for menubar, popup and dialog at 32x12 and 60x20, checking the selected action callback. `popup_and_dialog_menu_outside_click_ignores_closed_submenu_bounds` also passed for popup and nonmodal dialog at two sizes; it passed before the dialog depth-filter cleanup, so it is coverage, not a demonstrated defect for that cleanup.

### API-011 dialog input and broad regression (2026-09-09)

`dialog_menu_input_pastes_scrolls_and_deletes_whole_graphemes` failed before repair waiting for pasted text. Baseline: `.cairn/reviews/api-011-dialog-paste-baseline.log`. The corrected App workflow passes at 24x10 and 48x14: paste a long Unicode value, see its tail, Home, delete a CJK grapheme and a combining grapheme, End, and submit the exact remaining value. Input paints a bounded grapheme window around the caret using its measured width; pointer placement includes that window offset. Paste inserts once rather than per character, and strips control characters for this single-line field.

The broader run initially found an old MenuStyle default-class assertion. It now checks the corrected class string and numeric one-cell padding, which has independent painted-cell coverage. A subsequent full run passed all 827 library tests (two ignored) but failed the popover slide probe requiring three visible body/arrow frames among the first twelve total frames. The isolated rerun and then the 248-test widget rerun passed unchanged apart from a diagnostic message. To remove dependence on the count of setup renders, the slide probe now waits for three presented frames containing both body and arrow, retaining the exact geometry and changed-frame assertions. Verification after that test-gate change is still pending. These are development results; no formal receipts have been recorded.

### API-011 measured page navigation and dialog hover (2026-09-09)

`menu_page_navigation_uses_the_presented_row_count` failed before repair: Page Down advanced by the configured menu maximum instead of the presented page size. Baseline: `.cairn/reviews/api-011-menu-page-baseline.log`. The shared view counts rendered row targets in the current panel; menubar, popup and dialog use that count for Page Up/Down. The corrected App workflow passed for all three at 32x10 and 60x14, comparing the chosen action to an independent count of ROW labels in the captured frame.

`dialog_menu_hover_selects_the_painted_item_without_activating_it` failed before repair with no second-item callback. Baseline: `.cairn/reviews/api-011-menu-dialog-hover-baseline.log`. Dialog and popup now share selection-only hover logic that preserves open ancestors and opens selectable child menus without firing item callbacks. The full menu rerun is pending.

The preceding slide test-gate change passed the full 248-test widget suite, and strict default-feature all-target Clippy passed before the page/hover additions.

### API-011 reopen/focus verification and current development suite (2026-09-09)

The corrected `popup_and_dialog_menus_reopen_and_restore_the_trigger_focus` test passes for popup, modal dialog menu and nonmodal dialog menu at 32x12 and 60x20. It opens, cancels, reopens, selects, opens again and cancels, asserting exact trigger/action/hide callback order. No production focus change was needed. The first probe was invalid: it gated later input only on OPEN, which was already painted behind the menu; subsequent keys could use stale frames. The shared `run_visibility` test helper now requires menu disappearance before each reopen.

All 35 menu workflows passed after the page/hover repair; the reopen test adds a 36th. The latest complete development command `cargo test --locked --lib --test api_widget_behavior` passed: 1,078 tests passed and two ignored (827 library and 251 widget App tests). Formatting passed. Strict default-feature all-target Clippy after this final helper addition passed, as did git diff --check. API-011 and the commitment remain incomplete; none of these results is a formal Cairn receipt.

### API-011 nested dialog multi-selection follow-up

The real App probe `dialog_menu_nested_multi_selection_paints_and_survives_prop_updates` failed before repair: the submenu painted ONE without its checkbox, so the expected `[ ] ONE` frame never arrived. Captured output is `api-011-dialog-nested-multi-baseline.log` in this directory. Dialog rendering now maps checkbox state recursively, and prop updates retain selectable nested IDs while removing selections whose parent becomes disabled. The corrected probe passed at 32x12 and 60x20 for both retained and disabled-parent cases, including the exact confirmed IDs. All 37 menu App tests passed. The broader development run passed 827 library and 252 App tests (1,079 total, two ignored); strict locked all-target Clippy and git diff --check passed. These are development checks, not committed Cairn receipts; API-011 remains unfinished.

The additional `disabled_and_empty_menus_ignore_activation_through_app` workflow passed for MenuBar, PopupMenu and DialogMenu at both viewport sizes: disabled actions emit no callbacks, and empty menus tolerate direction and activation keys. No production repair was needed for those cases. The subsequent development suite passed 1,080 tests (827 library, 253 App; two ignored), strict all-target Clippy passed, and diff whitespace checks passed.


API-011 menu scroll seed and lifecycle follow-up: the public PopupMenuState and
DialogMenuState scroll offsets were ignored by MenuView. The App test
menu_public_scroll_seed_keeps_its_window_through_navigation failed on the old
renderer (captured in api-011-menu-scroll-seed-baseline.log). Retained panel offsets
now start from the public seed, keep the selected row inside the measured cell
budget, and remain stable when navigation stays within the window. The corrected
case passes for both menu families at 32x12 and 60x20, including movement past the
window and back. Existing separator and measured-page workflows remain passing.

Additional App checks passed without production changes: all seven PopupPlacement
variants paint at independently expected screen cells; viewport shrink from both
sizes to 20x8 moves paint and the second item's mouse target together. Popup and
dialog prop reordering retains selection by ID and replaces action/selection/hide
callbacks. Closing and remounting resets state, and removing a visible owner emits
exactly one hide callback. The same scenario in fresh Apps produces independent
state and callbacks. These are sequential App isolation checks, not concurrent
screen-reader acceptance. Latest development library/widget run:
`cargo test --lib --test api_widget_behavior`: 1,084 passed, two ignored.
The initial filter `menus::` matched zero cases and is not acceptance evidence;
the corrected menu filter ran 41 cases before the lifecycle case was added.
Cairn receipts and remaining catalog acceptance are still pending.


API-011 public-builder and invalid-seed follow-up: 45 menu App workflows passed.
The public MenuBar/PopupMenu/DialogMenu props builders and MenuItemBuilder now have
activation checks at both sizes. ContextMenuBuilder trigger areas were exercised
inside an offset parent: outside and right-edge clicks do not open it, while an
inside click reports screen coordinates and the painted action responds there.
A new DialogMenu multi-selection seed probe demonstrated that disabled, hidden and
submenu entries were returned on confirmation. See
api-011-dialog-invalid-multi-seed-baseline.log. Seed normalization now excludes
those entries both initially and on updates; the corrected initial-state test passes.

The Orca suite is being extended with real-terminal menu workflows in menu_probe.rs
and orca_menus.py. The initial run passed the existing accordion, breadcrumb and
CSS stages, then failed because the menubar item did not expose an assistive-focus
interface. Baseline: api-011-menu-orca-focus-baseline.log (private session
/tmp/rtui-orca-y1ce1c3b). Menu rows now use the existing App virtual-focus metadata
and click dispatch; retained owners validate requested item paths. Corrected Orca
verification is still in progress; do not count this as a passing reader claim.


Orca menu follow-up: after enabling assistive row focus and clicks, the menubar,
popup and context workflows passed. DialogMenu exposed a second defect: nested
panels inherited the outer trap as their accessibility focus owner, while keyboard
focus remained on the main panel. The reader therefore did not receive nested-row
focus. Baseline: api-011-dialog-orca-submenu-focus-baseline.log. A single retained
content focus owner now encloses all dialog panels, preserving the outer trap and
footer controls. All 45 menu App cases still pass, including input and restoration.
The corrected real Orca workflow passed all four families at 60x16 in
/tmp/rtui-orca-un1ujrtj, including exact callback results. Development library/widget
suite after this correction: 1,087 passed, two ignored. The reader's checked-state
assertion has since been tightened to exclude 'not checked'; reruns are in progress.


The tightened Orca workflow passed at 32x10 (/tmp/rtui-orca-d0qtis5k) and 60x16
(/tmp/rtui-orca-tswe0bub). Checked-state speech now requires either an isolated
'checked' announcement or 'check menu item checked', excluding 'not checked'.
The captured speech and callback summaries are api-011-menu-orca-32x10.log and
api-011-menu-orca-60x16.log. Strict all-target Clippy passed after the focus repair;
`git diff --check` also passed. Reader negative controls are being rerun. These
remain development results, not committed Cairn receipts.


Both existing reader negative controls also passed after the menu extension:
`orca.py --negative` rejected painted account text without its semantic label,
and `orca.py --negative-css` rejected the missing CSS semantic control. The menu
focus failure demonstrations above independently establish that menu metadata
alone was insufficient. No new formal receipt has been recorded.

### API-011 menu state and resize continuation

Development checks: 1,091 library/App tests passed, two ignored; strict locked
all-target Clippy passed. Four new App cases cover public selection/callback
updates for all four menu families, long-press cancellation after release/move/drag
outside an offset context owner (waiting 200 ms past an 80 ms timer), menubar
authored dropdown scroll state, and menubar resize with open-dropdown clicks.
The existing popup resize case now also covers context and dialog menus.
All run at 32x12 and 60x20, with resize to 20x8.

The menubar scroll case failed against the implementation: it showed rows 2–4
when the caller supplied selected row 4 and scroll offset 4 (rows 4–6). The
captured failing assertion is api-011-menubar-scroll-seed-baseline.log. Menubar
now supplies that initial/changed public offset to the existing retained menu
view; the corrected case passes. Earlier test harness corrections (waiting for
a nonexistent fourth frame, and a clipped stage marker) are not defect evidence.
The context cancellation and other resize/state cases required no runtime repair.
API-011 and final catalog review remain unfinished; these development runs are
not Cairn evidence receipts.

### API-011 initial dialog-family repair

Generic Dialog now renders real editable children through Modal, retains classes
and dimensions, honors non-closable behavior and distinguishes modal background
input. The shared frame omits its mouse shield only when both backdrop style and
focus trap are absent. ConfirmationDialog and its generic builder now render
retained buttons with disabled/default state, ordered focus, vetoed actions and
a single close result. Toast now paints and owns an expiry timer; its six
positions use measured dimensions and do not steal background editing. Manual
close and unmount cancel its timer even while rendered output is retained.
ProgressDialog and its generic builder paint the real ProgressBar, update values
and callbacks, honor cancellation policy and show estimated-time text.

Baselines captured before repair: api-011-dialog-builder-baseline.log,
api-011-dialog-nonmodal-baseline.log, api-011-confirmation-app-baseline.log,
api-011-toast-app-baseline.log and api-011-progress-dialog-app-baseline.log.
Eight initial dialog App cases pass at 32x12 and 60x20. The latest complete
development run passed 1,100 library/App cases with two ignored tests.

A broad run also exposed the old Modal animation test waiting for exactly ten
frames when the animation had finished in nine. It now waits until a root marker
appears after the 200 ms transition, excludes marker frames from its formatted
intermediate-state comparison, and checks final content. The marker cannot itself
satisfy the animation assertion. The corrected target and complete run pass.
Two fixture corrections (50.0% formatting and toast border dimensions) are not
runtime defect evidence.

Dialog families remain under implementation: InputDialog, AutocompleteDialog and
Wizard are unfinished, as are detailed lifecycle/engine, positioning and reader
checks. Confirmation relative anchors currently report an unresolved-anchor error;
that advertised route still needs implementation. No catalog acceptance or Cairn
evidence receipt is claimed by these development runs.

### API-011 initial InputDialog recovery

InputDialog now renders the retained TextInput and Modal controls through App.
The initial App case rejects empty required input, accepts four graphemes, rejects
a fifth typed character, deletes a whole emoji, and sends the exact confirmed
value once. Direct InputDialog events now delete whole graphemes and accept paste.
Validation shares required/type/rule/custom-callback handling and a bounded regex
cache; invalid regular expressions no longer become literal-match fallbacks.

The original App and direct UTF-8 failures are captured in
api-011-input-dialog-app-and-unicode-baseline.log. The first repaired App path
exposed Modal auto-height omitting the horizontal scrollbar row, hiding its
validation error. That capture is api-011-input-dialog-validation-clipping-baseline.log.
Auto-height now includes the scrollbar row. Both corrected InputDialog cases pass.
The paste fixture was adjusted to use four graphemes followed by a rejected typed
character: TextInput rejects an oversized paste, while the legacy InputDialog
direct path truncates it. That difference remains for the detailed input review.

The complete development suite passed 1,102 tests with two ignored tests; strict
all-target Clippy passed after iterator/selection-branch corrections. Masks,
remote validation, password/multiline and detailed lifecycle/reader checks remain
unfinished. The temporary remote-validation message is not completed HTTP support
and must not be treated as acceptance. These runs are not Cairn evidence receipts.

## API-011 InputDialog grapheme-limit consistency

Direct-event edits previously counted the inserted text separately, which rejected a
combining mark joining an existing letter and silently truncated oversized paste.
The failing direct-event regression is captured in
`api-011-input-grapheme-limit-baseline.log`. Edits now validate the complete proposed
value and reject an oversized change without changing the value or selection, as
the App TextInput route already does. Four input-dialog tests passed, including
the new App workflow at 32x12 and 60x20. The App test explicitly moves to End before
adding the combining mark; TextInput initially places the cursor at the start.
This is editing-time verification, not a Cairn receipt or API-011 acceptance.

## API-011 InputDialog validation and HTTP checkpoint

InputDialog now shows nonblocking validation warnings, honors its read-only
attribute through the shared TextInput editor, and ignores direct key releases.
The warning and read-only regressions are captured in
`api-011-input-validation-warning-baseline.log` and
`api-011-input-readonly-baseline.log`. App tests cover password masking with exact
Unicode results, multiline edits and action submission, blur validation, warning
display, and read-only submission. Direct tests cover typed validators/rules and
protected input events.

The recorded curl transport decision is now implemented for App InputDialog. Its
worker has a request-size bound, one deadline covering version/startup/transfer,
a response bound that also applies to chunked transfer, and cancellation that
stops its process and joins the worker. The existing clipboard runner was
extracted into `src/core/owned_process.rs`; all thirteen clipboard integration
tests passed after extraction. Six HTTP unit tests passed with actual loopback
requests, including escaped payloads/headers, timeout, chunked overrun, missing
curl and actual-connection cancellation. The missing-curl subprocess fixture is
Unix-only because Windows executable lookup also searches system directories.

HTTP error tests exposed clipped status codes; the capture is
`api-011-input-http-error-clipping-baseline.log`. InputDialog now measures the
content clip width and wraps errors/warnings, allowing auto height to account for
all their lines. The test gates on the status code because wrapping may split
`status` and `503` across lines. Seventeen input-dialog integration tests then
passed, including remote HTTP/JSON errors at 32x12 and 60x20, replacement during a
request, removal before the server replies, and submission vetoes.

Rust's incremental compiler crashed once during editing; a build with
`CARGO_INCREMENTAL=0` proceeded normally. Strict all-target Clippy passed before
the HTTP extraction/addition; a new full check remains required. No Cairn receipt
is claimed. Masks, detailed prop/pointer/debounce checks, native engine HTTP/result
integration, HTTPS/platform verification and Orca remain unfinished.

## API-011 validator replacement and HTTPS follow-up

The full library/App/clipboard run passed 1,136 tests with two fixture tests
ignored before this follow-up. A new App regression then showed that fresh
validator callback Arcs on each root render cancelled an in-flight HTTP request.
The captured failure is `api-011-input-inline-validator-lifetime-baseline.log`.
Requests and accepted remote results now follow value and endpoint identity;
local rules and callbacks are rechecked at completion. Replacing callback Arcs
does not clear visible validation errors or discard a matching request. Pending
debounce work is rescheduled when its delay/settings change, and completed
timeout IDs are cleared. Twenty input-dialog tests passed after that repair,
including latest-local-validation rejection and reuse of remote warning results.
Strict all-target Clippy also passed at that checkpoint.

`scripts/check-dialog-http.py` generated an ephemeral certificate and explicitly
ran the ignored transport fixture for both trusted and untrusted cases on Linux.
Both executed and passed; the untrusted case delivered no HTTP request. The
wrapper rejects an empty test selection and owns subprocess groups on timeout.
It handles the expected connection reset after certificate rejection, while
recording other server failures. The widget mechanism declares and invokes this
fixture and the six ordinary HTTP unit tests. Native Windows/macOS HTTP evidence
remains pending. A newer direct-event regression also rejected disabled input
focus; its failure is `api-011-input-disabled-focus-baseline.log`, and disabled
focus metadata/selection were repaired before the 1,136-test run above.

Latest additions exercise another validation request after a submission veto
and a useful fallback message for empty remote rejection text. The fresh twenty
input-dialog tests passed, and the six ordinary HTTP unit tests passed with the
HTTPS fixture ignored in that ordinary run. The separate trusted/untrusted HTTPS
runs both executed and passed. Formatting, whitespace checks and Python syntax
checks passed. No formal Cairn evidence or API-011 acceptance is claimed.

### API-011 input formats and live prop/timer checks (2026-09-10)

The new mask workflow failed on the old implementation: `12-AB` closed a dialog configured with `AA-##`; see `api-011-input-mask-baseline.log`. The shared native/App validator now applies the recorded grapheme format syntax and the App shows its format. `api-011-input-masks-all.log` records 22 passing input cases, including corrected Unicode and native configuration cases. The subsequent prop-update check passed preservation of edits, dynamic read-only state, seed replacement and current callback delivery (`api-011-input-prop-updates.log`).

The first resize fixture used coordinates from another run and hit the backdrop while the resized modal was still settling (`api-011-input-pointer-fixture-diagnostic.log`); that is a fixture diagnostic, not product acceptance. The corrected helper locates ASCII click targets in the current presented screen, and the App pointer edit/submit test passes after both growing and shrinking (`api-011-input-pointer-resize.log`). Two additional debounce tests pass at both sizes: coalescing edits, replacing the pending delay, disabling change validation and removing the dialog (`api-011-input-debounce.log`). The cancellation cases keep the App alive past the old deadline and require no validator calls. Full regression and lint results are recorded separately when finished. No API-011 acceptance or dialog-engine/reader completion is claimed.

The completed regression run passed 834 library tests (three explicit fixture tests ignored) and all 298 App widget tests, including 26 input-dialog cases (`api-011-input-mask-lifecycle-regression.log`). Strict locked all-target Clippy passed (`api-011-input-mask-lifecycle-clippy.log`); workspace formatting and `git diff --check` also ran and passed. This run did not rerun native desktop clipboard platform probes or the explicit TLS fixture. Ripwire edit-check identifies mask_matches as one new private symbol with one direct caller and no incompatible call found; this structural scan does not replace the behavioral failure demonstration.


### API-011 autocomplete editing-time review

Four new native regressions first failed: joined-emoji backspace panicked, minimum length counted bytes, selection without a callback did not close, and submission bypassed the selection veto. The baseline is api-011-autocomplete-native-baseline.log; the corrected probe passes. Shared static filtering now serves native and App paths. The retained App component owns draft, selection, callbacks, debounce and one bounded HTTP job. Callback invocation releases option locks first. Replacement and removal cancel timers and drop the owned request. Scheduler review confirmed that cancellation also suppresses ready callbacks before invocation; no second cancellation mechanism was added.

Thirteen new App integration cases at 32x12 and 60x20 cover static and custom filtering/rendering, the public autocomplete builder, grapheme editing, empty results, keyboard selection, vetoed/resized clicks, Escape dismissal, wheel navigation, long-list visibility, callback and seed updates, real HTTP query/header/object parsing, malformed responses, debounce, request replacement and removal. Two additional unit cases cover whole-grapheme highlighting under case expansion and suggestion response validation. Native engine HTTP/results and actual reader acceptance remain separate outstanding work, and further advertised style/lifecycle cases remain in the inventory.

The long-list baseline could select Item9 without painting it. Row visibility must compare transformed row edges to the ancestor clip rectangle; clip.height alone is not the visible part of a row. The diagnostic and corrected App cases are retained. A separate HTTP test diagnostic showed REQUESTS 1 before any modal was painted: its early keys could not edit an absent input. The corrected replacement fixture types the first query through the painted control before waiting for its request. It then replaces that live request and observes only the new result. The initial Unicode App fixture also needed End before Left because the shared text input starts at column zero; no shared TextInput product change was made for that fixture correction.

Final editing-time regression: api-011-autocomplete-full-regression.log passed 840 library tests and 311 App tests, with three explicit library fixtures ignored. api-011-autocomplete-final-clippy.log passed locked strict all-target Clippy with -D warnings. cargo fmt --all -- --check and git diff --check passed. Ripwire edit checks report the new retained component and shared filter with no incompatible callers (static lower bounds only). These are not Cairn acceptance receipts; API-011 remains unfinished.

### API-011 Wizard implementation probes (2026-09-10)

The pre-repair native probes reproduced the empty-list subtraction panic, ignored
step validation and accepted empty/duplicate step IDs (api-011-wizard-native-baseline.log: three failures).
The first corrections exposed two distinct public ValidationResult types and their
different defaults; the adapter now preserves validator messages/warnings and treats
a missing optional validator as valid. Compilation failures are retained in the
native-corrected and native-live logs. The old builder unit test required descriptive
text, so it now checks component construction; real builder behavior is independently
covered through App.

The retained Wizard uses shared Modal buttons/progress, explicit caller data,
validation before navigation/completion, optional Skip, completion veto, and keyed
visited child panels. Nine App tests and five native/builder tests passed in
api-011-wizard-final-targeted.log, including both viewport sizes, pointer navigation
after resize, edited child state on Back, new data/callbacks after keyed step reorder,
builder initial step/can_proceed/cancelable/progress settings, empty configuration and
exactly one App completion/cancellation. An initial resize fixture attempted Back
while that button was not yet painted during reflow; it now waits for the actual
Back target, then clicks its measured coordinates. The empty-state message was
shortened to fit the narrow viewport. The initial App failures are preserved in
api-011-wizard-app-controls.log.

An additional keyboard/allow_back case and pruning of removed visited IDs are in
the subsequent full regression run; its result is recorded separately after execution.
Strict all-target Clippy passed before that final small change (api-011-wizard-clippy.log).
Ripwire edit-check for WizardDialog reported an unchanged struct contract and no
resolved callers; its caller counts are lower bounds, not proof of absent consumers.
Native engine result/focus/lifecycle repair remains API-012. Extended style, disabled,
removal and actual reader coverage remain explicit API-011 inventory obligations.
No API-011 acceptance receipt or completion claim is made by these editing probes.

Wizard follow-up verification completed: api-011-wizard-full-regression.log records 844 library passes, three explicit ignored library fixtures, and 321 App passes (including ten Wizard workflows). api-011-wizard-final-clippy.log records strict all-target Clippy passing. cargo fmt --all --check and git diff --check both ran and passed. These checks cover the final visited-ID pruning and keyboard/allow_back case.

### API-011 RelativeTo positioning probes (2026-09-10)

The baseline App test failed with an unresolved-anchor message and no dialog body
(api-011-relative-position-baseline.log). The new App-owned, scoped snapshot maps
requested Element keys to acknowledged visible bounds. It rejects ambiguous keys,
updates after movement/resize/removal, and releases names no longer requested.
RelativeTo uses all nine points on that measured rectangle and saturating offsets.
The corrected nine-anchor test passes at both viewports; two more App cases exercise
parent movement, resize, ambiguity and removal. Two snapshot tests check owner
isolation, changed/unchanged publication, removed geometry and requested-name cleanup.
The resize fixture gates events on independent expected border cells, rather than
assuming how many setup frames layout needs.

api-011-relative-full-regression.log passed 846 library tests (three explicit ignored
fixtures) and 324 App tests. Clippy initially rejected the test-module placement;
that module was moved after production items. api-011-relative-final-clippy.log
records strict all-target Clippy passing. The later core-input work has its own
verification; these results do not establish that subsequent source tree.

### API-011 core input construction probes (2026-09-10)

Two baseline App probes fail: core input does not paint its placeholder and cannot
edit a text seed (api-011-core-input-baseline.log). The builder now constructs the
existing TextInput with its value and placeholder while preserving authored metadata
and children. A further resize/disabled probe exposed a swallowed on_click callback
(api-011-core-input-controls.log); that input-specific callback now observes pointer
activation during capture on the same measured target, before text editing consumes
the bubble event. Disabled input suppresses the capture handler as well as editing.

Four App tests pass at both viewport sizes (api-011-core-input-macros.log): Unicode
entry replaces a visible placeholder, text seeds edit, all four input macro forms
are real controls, and resized clicks focus/edit while invoking a callback once.
Disabled controls neither edit nor invoke that callback; typing does not invoke it.
The final full regression run is recorded separately when complete. Strict all-target
Clippy passed in api-011-core-final-clippy.log. Formatting and diff whitespace checks
ran and passed. Ripwire ElementBuilder edit-check reports an unchanged struct contract
and no resolved callers; those counts are lower bounds. The core inventory row
retains pending helper/style/reader reconciliation.

Core-input full regression completed: api-011-core-full-regression.log records 846 library passes, three explicit ignored library fixtures, and 328 App passes. No API-011 Cairn receipt has been produced; catalog completion is still pending.

## API-011 external image source ownership

Inspection found that both external renderer methods deleted their prepared path
after command completion, including caller-owned FilePath and local Url sources.
A subprocess fixture reproduced deletion on a successful chafa call using only an
owned test file. The baseline failed with “renderer deleted caller-owned image”;
see `api-011-image-ownership-baseline.log`.

The repair distinguishes borrowed source paths from renderer-owned temporary files.
Only the latter have automatic cleanup. Temporary files use the existing tempfile
dependency for unique, exclusive creation instead of a predictable per-process path.
Public renderer signatures remain unchanged. The fixture covers both renderers
with success, nonzero exit and unavailable-tool outcomes. The initial corrected run passed all four external-renderer tests
(`api-011-image-ownership-corrected.log`). Local Url ownership and concurrent
temporary-file coverage are being added. Image decoding, bounded external execution and App placement remain
separate unfinished work under the current commitment.

## API-011 image payload failure demonstrations

`api-011-image-raw-baseline.log` records the external raw RGB test failing: the
renderer wrote unencoded pixel bytes to a .png file. The shared decoder and PNG
encoding repair passed all six external-renderer tests in
`api-011-image-raw-corrected.log`, including local-path Url ownership and unique
temporary-file cleanup.

`api-011-image-protocol-baseline.log` records three expected failures: Kitty
labelled RGBA bytes as f=24, iTerm2 used dimensions for the byte-size field, and
a short raw RGBA payload was accepted. The corrected serializer is being checked
against the official protocol definitions at
https://sw.kovidgoyal.net/kitty/graphics-protocol/ and
https://iterm2.com/documentation-images.html (read 2026-09-10). The tests decode
the actual emitted payload and compare pixels, lengths, dimensions and chunk
controls; they do not count a protocol marker as a valid image. Actual host
integration and retained placement remain unfinished.

The corrected native image suite passed all 26 tests, including decoded fallback
and zero-area output (`api-011-image-fallback-corrected.log`). Strict default
all-target Clippy passed (`api-011-image-native-clippy.log`). The owned-process
fixtures stop and reap a stalled child after the five-second deadline and reject
excess output, invalid UTF-8 and nonzero exits. Full library/App regression is
running before the retained image adapter work.

## API-011 retained image adapter work

The two initial real App image tests failed because ImageBuilder painted its
configuration description (`api-011-image-app-baseline.log`). The adapter now
retains source data, format hints and classes, loads on an owned worker, and
paints decoded ASCII fallback at the presented content size. Both initial App
tests passed (`api-011-image-app-corrected.log`). The margin fixture was corrected
to exercise a child in a parent and use the existing four-cell spacing scale.

The expanded source-update fixtures initially expected the wrong ramp character
for neutral 128. They now use neutral 160, which maps away from a character
threshold. Integer luminance coefficients have a separate identity-property test
covering all 256 neutral values; the original floating-point expression was
checked independently before recording any discrepancy.
The old ImageBuilder unit test asserted the configuration-description defect. It
now checks retained construction; actual behavior is verified through App. Worker
latest-result and removal tests are being added. Protocol placement remains open.

The independent rustc probe of the original floating-point neutral-channel
expression found 13 mismatches: 37→36, 61→60, 74→73, 93→92, 111→110, 122→121,
148→147, 186→185, 215→214, 222→221, 233→232, 244→243 and 253→252. The integer
identity-property test passes. This is separate from correcting the App fixture's
128 ramp expectation; that fixture expectation was an authoring error.

The retained image check passed 38 image-filtered library tests and all eight
App image workflows (`api-011-image-retained-tests.log`). These include actual
source changes, removal, resize, clipping, independent images and format errors.
Strict all-target Clippy passed (`api-011-image-retained-clippy.log`). Full
regression is running in `api-011-image-retained-regression.log`. Graphics-mode
transmission/cleanup, external-mode App output and actual host checks remain open.

## API-011 owned Kitty frame output, work in progress

The retained-adapter full regression passed 863 library tests and 336 App tests,
with three library fixtures ignored (`api-011-image-retained-regression.log`).
It first exposed a second stale Image conversion assertion in the specialized
builder suite; that assertion now checks a retained component and its factory.

Decoded Image metadata now reaches the existing Taffy/SuprTUI painter. The painter
uses actual content insets, transforms, ancestor clipping and cell coverage to
project RGBA image planes. Later text and opaque backgrounds mask earlier image
cells; translucent backgrounds tint them. Explicit Kitty layer values preserve
paint order independently of image IDs. The backend owns transmission, replacement,
removal and shutdown deletion. Image commands sit inside the engine's synchronized
update and share its checked flush; failed output keeps possible image IDs for
cleanup and retries. Unchanged placements do not retransmit. The owned-writer
constructor accepts explicit Kitty capability and physical cell pixels; native
construction uses environment capability detection and terminal pixel dimensions
when available, with an 8x16-cell fallback. Other protocol selection is not yet
connected to this App graphics path.

The first combined editing check passed 34 image-filtered library tests and ten
App image workflows (`api-011-image-graphics-combined.log`). New cases decode Kitty
payloads for authored positions, clipping, layering, translucent image pixels,
text/background occlusion, physical cell dimensions, changed sources and removal.
They also exercise failed-flush retries, shutdown deletion and capability fallback.
Real App raw, encoded-file and base64 workflows pass at two viewport sizes. A
violating-example run and strict all-target Clippy are still pending for this work.
These are editing checks, not Cairn receipts or actual host evidence.

Kitty cursor policy, positive layer ordering, chunk controls and owned-ID deletion
were checked against https://sw.kovidgoyal.net/kitty/graphics-protocol/ on 2026-09-10.
The App serializer keeps the cursor in place; the standalone serializer retains
its existing default cursor movement. iTerm2, Sixel, external renderer modes,
parallel platform/Surface adapters and actual host integration remain unfinished.

The safe violating example disabled the painter's graphics selection and the
owned-output test failed with zero image transmissions instead of one
(`api-011-image-graphics-violating.log`). Restoring selection passed 34 filtered
library tests and ten App workflows (`api-011-image-graphics-corrected.log`), then
strict all-target Clippy (`api-011-image-graphics-clippy.log`). Later follow-up
work refreshes native pixel metrics after resize and indexes coverage by cell
so glyph masking visits only overlapping images. That follow-up is being checked.

The resize/coverage follow-up passed 35 filtered library tests and ten App image
workflows, then strict all-target Clippy (same corrected/Clippy logs). It includes
metric changes, terminal resize and switching from image Elements to CellFrame
output. Formatting and diff whitespace checks also passed. Full library/App
regression is running in `api-011-image-graphics-regression.log`. Ripwire's
paint_frame edit-check reports the intended private parameter addition, one caller
and zero incompatible calls; counts are lower bounds. Local host discovery found
Kitty 0.45.0, Ghostty 1.3.1, GNOME Terminal and Xvfb. No graphics host check has run yet.

The owned-Kitty full regression completed successfully:
`api-011-image-graphics-regression.log` records 869 library passes, three explicit
ignored fixtures and 338 App passes. No actual host check or new Cairn acceptance
receipt has been produced. Kitty/Ghostty host work is next; Ghostty's environment
capability branch still needs to recognize its advertised Kitty support (official
feature documentation: https://ghostty.org/docs/features, read 2026-09-10).

## API-011 real Linux image hosts

Added an App fixture and isolated Xvfb capture driver in tests/api_widget_behavior.
The fixture shows red/blue pixels, changes to green/yellow at a new position,
then removes the image. Kitty 0.45.0 and Ghostty 1.3.1 pass independent screenshot
color counts and bounds, stale-color absence and final removal assertions.
See api-011-image-host-kitty-corrected/ and api-011-image-host-ghostty-corrected/
for PNGs, pixels.json and host logs. The Ghostty safe baseline failed with zero
image-color pixels and visible ASCII fallback (api-011-image-host-ghostty-system-mesa-baseline/).
The correction recognizes TERM_PROGRAM=ghostty alongside kitty. Six existing
capability tests pass; Ripwire reports no signature or caller incompatibility.

Initial software-rendering experiments were not acceptance passes: the installed
AMD Mesa/LLVM crashed or did not render image textures. A captured Ghostty stack
located its crash inside /opt/amdgpu LLVM vector lowering. The driver now selects
the system Mesa library/driver paths for its own child processes, llvmpipe with
SSE2/128-bit vectors and disabled shader cache. It changes no desktop settings.
Mesa environment reference: https://docs.mesa3d.org/drivers/llvmpipe.html.
Ghostty protocol reference: https://ghostty.org/docs/features.
The Rust host fixture builds and its strict Clippy check passes.
These are editing checks; image protocol/platform completion and Cairn evidence
remain outstanding.

GNOME Terminal fallback captures also passed ink presence, movement and removal
(api-011-image-host-gnome-corrected/). Visual inspection of the three stages
confirmed different decoded-glyph patterns and an empty image area after removal.
Strict all-target Clippy, cargo fmt --check and git diff --check passed after
these changes. No surviving fixture or owned Xvfb process was found.

## API-011 platform image and Sixel repair in progress

Four platform acceptance probes failed on the original implementation: zero file
dimensions, malformed accepted input, incomplete Kitty upload, and invalid inline
file parameters (api-011-image-platform-baseline.log). Shared decoding and serializers
now cover platform files, PNG/JPEG/GIF/WebP memory, raw pixels, clipping, scaling and
pixel offsets. Sixel input uses a bounded two-pass decoder for one DCS image;
numeric overflow, malformed controls, resource limits and RGB/HLS palettes are tested.
The native sixel-rs encoder produced one red palette entry and a leading row advance
for a two-pixel red/blue image; that failure is recorded in
api-011-image-sixel-encoder-baseline.log. A recorded replacement uses owned Rust
palette encoding, exact small palettes, a bounded 256-color palette for larger images
and the original quality diffusion choices. Twelve platform cases now include
quantization quality, transparent offsets and randomized malformed Sixel bodies.

Standalone protocol fixtures pass actual screenshots in Kitty (Kitty), Xterm
(Sixel), and WezTerm (iTerm inline): api-011-image-host-kitty-platform/,
api-011-image-host-xterm-sixel/, and api-011-image-host-wezterm-iterm/.
These standalone fixtures clear their own test screen between stages; they prove
protocol pixels and placement, not App lifecycle ownership. The App fixture also
passes in Xterm/Sixel and WezTerm/inline (api-011-image-host-xterm-app-sixel/ and
api-011-image-host-wezterm-app-iterm/). The worker now clears and fully repaints its
owned screen when non-Kitty placements change, disappear or resize. Kitty retains
ID-specific cleanup. Sparse cell pixel tiles compose overlapping images without
scanning every previous image per pixel. A capture caught image crate alpha rounding
that changed opaque compositing to alpha 254; integer source-over now preserves 255.
Focused tests and strict Clippy for the final changes are running; no new Cairn
receipt or image-family completion claim is made.

The corrected multiprotocol checks passed: 49 image-filtered library tests and
11 App workflows (api-011-image-multiprotocol-tests.log), then strict all-target
Clippy (api-011-image-multiprotocol-clippy.log). Those checks include exact opaque
alpha, masking, cross-image blending, failed-flush retry cleanup, resize and
shutdown. A subsequent host-fixture extension checks full-viewport images for
unwanted terminal scrolling; it builds, and host captures are running.

### API-011 Sixel bottom-edge host repair (2026-09-10)

The full-height Xterm fixture exposed two separate facts: cursor-relative Sixel
scrolled away the bottom row (299 of 312 image rows remained), and the fixture
needed an explicit foreground label layer to test image occlusion unambiguously.
The label now uses an absolute top-row z-10 overlay. With this same corrected
fixture, mode 8452 still failed the label assertion; see
`api-011-image-host-xterm-app-sixel-full-overlay-scrolling/`. Restoring absolute
Sixel display mode passed all stages and retained all 312 rows; see
`api-011-image-host-xterm-app-sixel-full-corrected/`. This is a failure/correction
demonstration, not a weakened pixel assertion. Earlier captures without an
explicit label layer are diagnostic only for label behavior.

The worker saves, selects and restores DECSDM around Sixel output, pads from the
screen origin with bounded pixels, and preserves preceding image pixels in that
padding for hosts that replace complete image cells. Xterm and WezTerm both
passed source changes, nonzero-position movement and removal:
`api-011-image-host-xterm-app-sixel-absolute/` and
`api-011-image-host-wezterm-app-sixel-absolute/`. Nine focused graphics tests passed,
including non-square cells, top/left padding, partial-alpha preceding pixels,
text masks, unchanged-frame caching, resize, removal and failed flush. The last
small generalization of background composition still needs the focused rerun.
The overall image catalog and API-011 remain incomplete; these are local checks,
not committed Cairn evidence.

The native option regression first failed with transparent red/blue pixels instead
of the requested opaque green background; its actual failure output is retained
in `api-011-image-native-options-baseline.log`. The corrected Kitty pixel test
passes and also checks Fast nearest-neighbor resizing. Shared loading applies
the requested RGBA background for Kitty and inline output; both protocol and
Sixel resize paths now select the requested quality. The two private resize
signatures changed to grouped dimensions plus quality; Ripwire edit-check found
zero incompatible callers for each. No public signature changed.

After these edits: `cargo test --lib image -- --nocapture` passed 60 tests;
`cargo test --test api_widget_behavior image -- --nocapture` passed 11;
`cargo clippy --all-targets -- -D warnings`, `cargo fmt --all`, and
`git diff --check` passed. An attempted test target `api_component_tree` does
not exist; it ran no tests and was corrected to `api_widget_behavior`.

Inline multipart investigation remains separate from a demonstrated defect.
The official iTerm2 image documentation describes MultipartFile as a v3.5
extension for tmux integration. The local WezTerm reference has no such command.
Do not silently switch all existing inline hosts to an unsupported extension.
Original File transfer is parsed as multiple internal tokens by iTerm2; a
claimed universal 1 MiB limit on that older route is not established by the
multipart paragraph. Source: https://iterm2.com/documentation-images.html and
https://gitlab.com/gnachman/iterm2/-/blob/82e4781d462d0d2cc50fced133f31d60dc5076ef/sources/VT100Terminal.m .

### API-011 external image modes and automatic selection

The retained image worker now invokes Chafa/Viu in static symbol mode. Captured
ANSI is parsed into a bounded vt100 screen and composed as styled runs through the
ordinary layout; it is never written directly to the host. Requests are generation
checked and cancelled through the existing owned-process runner. Resize can reuse
decoded pixels after the source file has disappeared. The worker caches Auto's
fallback choice only after an uncancelled probe, with graphics priority retained.
Viu's two explicit dimensions otherwise stretch the source; its prepared pixels
now honor aspect, background and quality. Public signatures remain unchanged.

Failure demonstration: the real-host fixture forced to internal ASCII produces zero
red/blue/green/yellow pixels and fails its color assertion, retained in
`api-011-image-host-external-ascii-negative/` and its log. Actual Chafa 1.18.1 and Viu
1.6.1 pass Kitty captures for source update, movement, enlargement and removal in
`api-011-image-host-{chafa,viu}-resize/`. Auto with Chafa passes GNOME Terminal in
`api-011-image-host-auto-chafa/`. The executables were installed only under
`/tmp/rtui-image-tools` (Ubuntu package extraction and locked cargo install), with
PATH scoped to these fixture commands. These are Linux/Xvfb/system-Mesa captures.
The same fake-process worker tests check deletion of a source before resize and
cancel/reap an active process in less than one second on removal. A separate Viu
prepared-PNG test compares aspect/stretch dimensions and selected-background pixels.

Editing checks: `api-011-image-external-tests.log` has 64 image library tests passing;
`api-011-image-external-app-tests.log` has 11 App image workflows passing;
`api-011-image-external-clippy.log` records strict all-target Clippy passing before
the final equivalent config-allocation cleanup. Ripwire reports the public external
renderer contract unchanged. Full regression passed: 889 library tests (three ignored) and 339 App workflows
in `api-011-image-external-full-regression.log`. Animation, Surface and remaining image-platform acceptance are still
unfinished, and none of these editing checks are committed Cairn receipts.

### API-011 animated GIF delivery

The former static shared decoder now also supplies a bounded retained animation
path through a common source reader. GIF frame pixels use the image library's
compositor; the already locked gif 0.13.3 dependency supplies repetition and a
preflight frame count. The preflight rejects cumulative RGBA above 256 MiB before
allocating the frames. The image worker owns the clock, skips expired frames,
retains timing on layout changes, stops finite/hidden animations, and cancels and
releases state on replacement/removal. App observes every publication through an
incremented wake signal; frames use the same native/external output paths.

Safe violation: temporarily disabling the worker frame deadline made the new App
GIF workflow fail after only two initial App frames (`api-011-image-gif-frozen-baseline.log`).
The exact corrected source was restored before further work. The corrected test
observes red, green, red without input at two viewport sizes. The current
`api-011-image-gif-app-tests.log` records all 12 image App workflows passing.
`api-011-image-gif-library-tests.log` records 68 image library tests passing,
including zero-delay and tiny-frame-count bounds. The final broad editing run in
`api-011-image-gif-full-regression.log` passed 893 library tests (three ignored) and
340 App workflows. Strict all-target Clippy, formatting and diff checks also pass
in `api-011-image-gif-{clippy,format,diff-check}.log`.

The unchanged-source GIF fixture contains two six-second frames. Captures at four
and eight seconds show different decoded colors, then removal clears the image.
All five routes pass measured host pixels: Kitty, Xterm Sixel, WezTerm inline,
Chafa and Viu, in `api-011-image-host-gif-*/`. The original driver log's generic
'source update' phrase means frame advancement for these unchanged-source GIFs;
the driver now names that distinction. The Kitty first frame and Viu moved second
frame were visually inspected: correct paired colors, aspect and label placement.
These remain editing checks, not Cairn receipts; Surface and remaining platform
image paths still need work.


### API-011 Surface image output repair (editing checks)

The old region split covered only 63 of 64 source rows. The extended existing test
failed on that boundary (`api-011-surface-region-baseline.log`); proportional source
boundaries and at least one source pixel for upscaled cells correct it. Placement
loops now clip before iteration, checked alternatives report invalid input, zero is
reserved as the legacy registration error ID, and removing/clearing images removes
cell references. Registry IDs do not repeat after clear.

The old DiffWriter background-SGR/placeholder path is replaced with decoded
half-block fallback and shared Graphics state/serializers. The shared owner now
accepts a private raster interface; App keeps its existing plane implementation.
Surface uses bounded projected rasters and caller-confirmed protocol settings.
Kitty negative z-index draws behind text; legacy protocols retain text with a
sampled decoded background. Invalid preparation emits no bytes. Acknowledgment is
explicit and follows successful delivery; possible graphics IDs remain available
for cleanup after partial failures. Renderer now propagates writes and flushes,
updates the previous surface only after success, retries complete frames after
failure, discards failed buffered bytes, and cleans graphics before restoration.
Restoration is idempotent, preventing a repeated cleanup from clearing the normal
screen after leaving the alternate screen.

Seventeen focused Surface tests and nine shared App graphics tests passed. Real
Kitty, Xterm/Sixel (including full height), WezTerm/inline and GNOME fallback host
captures pass source updates, movement and removal. Kitty and Xterm behind-text
captures retain white glyph pixels without black holes in the image. A separate
Kitty fixture replaces only its own stdout with /dev/full, observes the frame
error, restores stdout and successfully redraws the same frame; its terminal byte
capture contains the failure/retry marker. The full-screen Xterm and moved GNOME
fallback screenshots were inspected visually. These are Linux host results.

A safe placeholder mutation makes the decoded fallback assertion fail, retained in
`api-011-surface-output-negative.log`; the production source is restored before
corrected/regression checks. Strict all-target Clippy passed before this temporary
mutation. Full corrected regression results will be recorded separately. No Cairn
receipt or completion claim is made by these editing checks.

Corrected full regression after the Surface repair: 899 library tests passed
(three ignored), all 340 App widget tests passed, strict all-target Clippy passed,
and formatting and diff checks passed. Logs are
`api-011-surface-regression-{library,app,clippy,format,diff}.log`. Ripwire reports
unchanged public contracts and zero incompatible callers for DiffWriter and
place_image_region. The API-011 mechanism now includes Surface and shared graphics
unit selectors, in addition to widget image and App workflows.


### API-011 retained terminal repair underway

The real-child baseline reports NOT_A_TTY because the legacy PseudoTerminal uses
pipes. The failure is retained in `api-011-terminal-pty-baseline.log`. Unix now
shares the existing PtyChild primitive with the libghostty session, through
`src/terminal/owned_pty.rs`; the native embedded module reexports that same owner.
A joined worker owns the retained PTY, bounded command/input/output queues and
actual resize/exit/reaping. Four PTY checks pass, including controlling streams,
child-observed sizes before and after resize, input/output, natural exit, missing
executables, invalid sizes, input bounds and cleanup while output is flooded.
The old Windows implementation is isolated in pty/windows.rs and is not repaired
or accepted yet.

Terminal no longer spawns a placeholder polling thread. Its caller drains bounded
output, observes one process-exit event and can inspect a distinct IO error.
Input without a running process now reports an error; the old test accepted a
successful no-op and has been corrected. Screen convenience operations apply to
the interpreted screen instead of sending escape bytes as shell input. Resize
retains cells and offers bounded scrollback access for the widget. Fifty-four
terminal-filtered library checks pass, with one preexisting ignored integration
case (`api-011-terminal-core-tests.log`). The widget still needs real measured
painting/lifecycle integration, and the legacy screen's Unicode/style/mode paths
need completion. These partial editing checks do not establish API-011 acceptance.

Strict all-target Clippy also passes for the partial terminal core changes
(`api-011-terminal-core-clippy.log`). Enabling embedded-terminal compiles and its
two native keyboard/buffering library checks pass after sharing PtyChild
(`api-011-terminal-shared-native-tests.log`). The full native integration gate
has not been rerun at this point.


### API-011 retained terminal widget: measured App ownership (development checks)

Replaced placeholder rendering with actual terminal-cell content, grouped by style
and cursor state. Title and scrollbar reserve measured cell rows/columns. App focus
callbacks control input delivery; key releases are ignored, wheel direction comes
from signed wheel data, and UTF-8 paste reaches the input queue. An owned monitor
publishes screen/exit/error changes through the existing wake signal and is joined
before stop/removal. Launch-prop changes replace the child; title and scrollbar
changes retain it. Prop updates apply the previously measured dimensions, because
unchanged layout does not produce another layout notification. Named Terminal and
TerminalWidget construction now resolve the same public component.

The strengthened scroll unit check initially failed (expected offset 5, observed 0):
line feed clamped the cursor before the screen could scroll. The screen now scrolls
at its bottom margin. The first launch-replacement App check also failed waiting
for GEN:two; applying measured size on prop updates fixes it. These were actual
failing development checks, not historical acceptance receipts.

A temporary placeholder-rendering mutation failed the real launch-error App check
with the placeholder visible instead of an error, exit 101. Source was restored in
a finally block. Output: api-011-terminal-widget-placeholder-negative.log.
Corrected development runs: five App cases PASS (including two viewport sizes for
PTY dimensions and signed scrollback), 54 terminal-filtered library cases PASS,
and all-target strict Clippy PASS. Captures: api-011-terminal-widget-app-corrected.log,
api-011-terminal-widget-unit-corrected.log, api-011-terminal-widget-clippy-corrected.log.
The prior ignored empty widget start/stop probe was replaced by real Linux child
PID/reaping assertions. No claim is made yet for remaining Unicode/style/mode,
full key/paste, Windows, accessibility or final catalog acceptance. This action
remains uncommitted API-011 implementation, not Cairn evidence or final review.


### API-011 retained terminal Unicode cells (development checks)

Replaced the retained cell width table with the existing unicode-width dependency,
including combined emoji widths and saturating the public u8 string width. Empty
zero-width cells now identify wide continuations. Cursor movement saturates before
clamping and output with wrapping disabled stays within the last column.
VirtualScreen joins streamed grapheme code points, limits a cell to 4096 UTF-8
bytes, clears both halves of overwritten wide glyphs and their old hyperlinks,
and wraps a two-column glyph before the last column. Public method signatures
remain unchanged; ripwire edit-check found no contract change for TerminalCell,
TerminalCursor or VirtualScreen (its call counts are lower bounds).

Failure demonstrations ran before repair: the Unicode cell check failed on emoji
width 1 versus 2; the cursor check failed on column 81 versus 79; all three screen
checks failed on split graphemes, stale wide leaders and last-column clipping.
Logs: api-011-terminal-cell-unicode-negative.log,
api-011-terminal-cursor-negative.log and
api-011-terminal-screen-unicode-negative.log. Corrected terminal unit run passed
59 tests with none ignored (api-011-terminal-screen-unicode-corrected.log).
Strict all-target Clippy passed (api-011-terminal-unicode-clippy.log).
The first App command used the wrong filter and selected zero tests; that log is
not acceptance. The corrected terminal_acceptance filter passed all five real-PTY
App tests (api-011-terminal-unicode-app-corrected.log). Formatting and diff checks
passed. These are development results, not committed Cairn evidence.

Remaining terminal work includes extended SGR styles and painting, terminal modes,
complete input/paste handling, parser edge cases, constructor bounds, Windows PTY
behavior and the advertised accessibility/platform acceptance. Unicode cell
coverage also needs the real-PTY styled-output fixture to include clusters and
wide overwrites before claiming full widget acceptance.


## API-011 retained terminal colors and private modes (development checks)

Extended SGR colors now consume their operands as a unit, reject out-of-range
components without interpreting them as attributes, and support RGB, indexed and
bright ANSI colors. Dim, blink, hidden and strike flags and their resets are
retained. The widget paints dim foreground at half intensity, hidden text as
same-width spaces, and strike through the existing painter attribute. Blink
animation remains pending; retaining its flag alone is not acceptance.

Two new screen tests failed before repair (wrong RGB and missing attributes):
`api-011-terminal-sgr-negative.log`. The corrected terminal unit run passed 61
checks; strict all-target Clippy passed (`api-011-terminal-sgr-corrected.log`,
`api-011-terminal-sgr-clippy.log`). An actual PTY/App test at 28x8 and 42x12 then
failed because dim output retained full intensity (`api-011-terminal-style-app-negative.log`).
After painting repair, six App workflows passed, including wide Unicode,
combining text, RGB/indexed colors, bold/italic/underline, dim and hidden text
(`api-011-terminal-style-app-corrected.log`). Strict Clippy, fmt and diff checks passed.

Private mode handling now switches main/alternate buffers, preserves/restores the
main cursor for 1049, changes cursor visibility and autowrap, retains application
cursor/bracketed paste state, and implements scroll margins and origin addressing.
Zero relative movement parameters mean one. Two new tests first failed because
the alternate screen still showed main content and scrolling destroyed the header
(`api-011-terminal-modes-negative.log`). The corrected terminal run passed 63 tests
with none ignored (`api-011-terminal-modes-corrected.log`). Seven App workflows
passed, including entering an alternate screen and restoring main content after
input at both sizes (`api-011-terminal-modes-app-corrected.log`). Strict all-target
Clippy, fmt and diff checks passed (`api-011-terminal-modes-clippy.log`).
Ripwire edit checks reported unchanged contracts and zero incompatible callers
for set_graphics_rendition and process_csi (static lower bounds).

These are development runs on the unfinished implementation, not Cairn receipts.
Remaining terminal work includes parser recovery/bounds, further screen operations,
key/paste modes, blink/cursor details, scrollbar pointer controls and native Windows
PTY evidence. API-011 catalog and actual reader acceptance also remain open.


## API-011 retained terminal parser recovery (development checks)

Attacked oversized OSC titles, embedded controls, interrupted UTF-8 and CSI,
omitted CSI parameters, unsupported colon parameters, and DCS payload delivery.
The added tests failed before repair: a full title swallowed subsequent text,
an embedded control leaked title bytes onto the screen, incomplete UTF-8 discarded
valid later characters, an interrupted CSI printed the replacement command, and
DCS never emitted its public payload event. Negative logs are
`api-011-terminal-{parser,utf8,csi,dcs}-negative.log` in this directory.

OSC and DCS now track their terminating escape independently of bounded payload
storage. OSC controls remain inside the title; cancellation resets the sequence.
UTF-8 decoding discards malformed fragments and resumes at a fresh leading byte;
DEL does not paint. CSI preserves empty parameters, transitions through parameter
state before intermediates, and discards unsupported colon syntax through its
final byte. DCS emits bounded data with its actual final byte and parameters.
No public event variant or parser signature changed.

Corrected terminal unit runs progressed from 65 to 68 passing tests, none ignored;
the final log is `api-011-terminal-dcs-corrected.log`. Seven real-PTY App workflows
passed after these changes (`api-011-terminal-dcs-app.log`). Strict all-target
Clippy passed (`api-011-terminal-dcs-clippy.log`); fmt and diff checks passed.
Ripwire edit checks found unchanged contracts for the changed sequence handlers.
These development checks do not complete API-011 or replace committed Cairn evidence.
Remaining terminal work includes screen operations and input/mode integration,
Windows PTY verification, and catalog/platform acceptance.

### API-011 retained terminal cursor and erase behavior (development checks)

Attacked saved cursor restoration after shrinking the screen, both ESC 7/8 and
CSI s/u. The new test failed with cursor (11, 5) after resize to 4 by 2;
restoration now uses the existing screen-bound constraint and preserves SGR.
Attacked erase-from-cursor at a wide glyph continuation and erase-to-cursor across
rows. Both failed: the wide glyph remained split and erase mode 1 did nothing.
Erase now reuses whole-glyph clearing with the current background, implements
inclusive mode 1, and clears saved history independently with mode 3.
Verified 72 terminal library tests, seven real-PTY App acceptance tests, strict
all-target Clippy, workspace formatting, and diff whitespace. Logs:
`api-011-terminal-cursor-{negative,corrected}.log` and
`api-011-terminal-erase-{negative,corrected,app,clippy}.log` in this directory.
These are development checks, not Cairn receipts; API-011 remains unfinished.

### API-011 retained terminal child input modes (development checks)

A new real-PTY App test makes the child inspect exact input bytes in raw mode.
Before repair it stopped at APP-READY: Up ignored the child's application cursor
mode. The corrected test passes at 28 by 8 and 42 by 12: application Up is SS3,
normal Up is CSI, Ctrl+Shift+Up includes modifiers, UTF-8 paste has delimiters only
while the child enables bracketed paste. Modified navigation and function keys
now carry modifiers in CSI rather than an extra Escape prefix. Plain Alt+text
retains its Escape prefix, and key release remains ignored. Bracketed paste checks
the shared 64 KiB PTY input limit before allocating the envelope.
Eight terminal App tests and 73 terminal library tests passed. Strict all-target
Clippy, workspace formatting, and diff whitespace passed. Logs are
`api-011-terminal-input-modes-{negative,corrected,unit,clippy}.log`.
These are development checks; API-011 is still in progress.

### API-011 terminal helpers and escape operations (development checks)

Three public helper tests failed before repair: maximum u16 cursor coordinates
panicked, a semicolon truncated the title, and a title containing BEL plus CSI
was accepted as terminal commands. Cursor conversion now saturates. The title
helper rejects controls and payloads larger than the parser's 8190-byte title
capacity before mutation. OSC titles, working-directory URLs and hyperlink URLs
retain semicolons; hyperlink termination still clears the link.
Two more tests showed ESC M reverse index and ESC c reset doing nothing. The
screen now handles index, next line, reverse index within scrolling margins, and
reset. Escape intermediates are kept distinct so charset designation cannot
accidentally execute a bare index/reset command. Header/footer preservation,
reset modes/styles/cursor and existing App workflows pass.
Latest verification: 79 terminal library tests, eight terminal App tests,
strict all-target Clippy, workspace formatting, and diff whitespace. Logs:
`api-011-terminal-helpers-{negative,corrected,app,clippy}.log` and
`api-011-terminal-escape-{negative,corrected,app,clippy}.log`.
These are development checks; the current implementation action remains open.

API-011 retained Terminal named-key follow-up: the real-PTY regression sent all six navigation keys with application-cursor mode enabled then disabled. Before repair the child received CSI sequences in both modes (api-011-terminal-named-keys-negative.log). Moved the existing widget encoder into the terminal module and reused it from the public string-key API; the child now receives SS3 then CSI as required. No public signature changed. The terminal subset passed 81 tests, all eight TerminalWidget App tests passed, strict all-target Clippy, formatting and diff checks passed (api-011-terminal-named-keys-{corrected,app,clippy}.log). An oversized-resize state-preservation test passed before changes: Unix PTY validation already rejects the oversized area before mutation; api-011-terminal-resize-negative.log is a baseline PASS despite its filename. Windows remains to repair and validate separately.

API-011 cursor-shape interpretation: a new parser-to-screen regression failed because every DECSCUSR selection stayed Default. The screen now honors the space intermediate and maps values 0–6 to blinking/steady block, underline and bar, ignoring unknown selections and unrelated intermediate sequences. All 82 terminal tests, eight real-PTY App tests, strict all-target Clippy, formatting and diff checks passed. Logs: api-011-terminal-cursor-shape-{negative,corrected,app,clippy}.log. This establishes interpreted cursor state; widget shape painting and blink deadlines still need repair and acceptance.

### API-011 terminal cursor on wide characters

A real shell prints `界READY` and moves the solid block cursor to column two,
the continuation cell of the wide character. The App/SuprTUI acceptance test
failed because the cursor disappeared (black background instead of white).
The widget now applies the cursor to the complete grapheme whose cell range
contains its position. The original glyph and its following cell remain intact.
All nine terminal-widget App tests passed after the repair. Logs:
`api-011-terminal-wide-cursor-negative.log` and
`api-011-terminal-wide-cursor-corrected.log`. This verifies wide-cell cursor
placement; shape rendering and owned blink deadlines remain unfinished.

### API-011 terminal cursor blink ownership

Added an owned 500 ms scheduler deadline for explicitly blinking cursor modes.
Output or input activity restarts the visible phase. Steady modes, hidden cursors,
scrollback, focus loss, stop and destruction cancel the owned deadline.
The deterministic clock test exercises off/on phases, activity, deactivation,
steady mode and destruction. The real-PTY App test observes off/on rendering
while the child is silent and receives no input. Disabling blink in the adapter
made that test fail after only two setup/output frames; restoration passed.
The negative mutation was restored before other changes. Logs:
`api-011-terminal-blink-negative.log`, `api-011-terminal-blink-clock.log`,
`api-011-terminal-blink-library.log` (83 tests) and
`api-011-terminal-blink-corrected.log` (10 App tests).
Underline/bar shape painting and clipping-sensitive deadline cancellation still
need completion; no claim of complete cursor support is made here.

### API-011 cursor clipping and stopped-session deadlines

A new retained-widget test reproduced a blink timer surviving clipping. The
widget now checks the cursor grapheme against measured content, ancestor clip,
and placement transform before scheduling. Collapsed transforms and stopped
sessions suppress the timer. Geometry changes request rendering even when
content size stays constant. The test passes for clipping, translated visibility,
focus loss/resumption, collapsed geometry, hidden cursor and stop followed by
render. All 84 terminal library tests and ten real-PTY App tests passed; strict
all-target Clippy, formatting and diff checks passed. Logs:
`api-011-terminal-clip-{negative,corrected,library,app,clippy}.log`.
Underline/bar shape painting remains unfinished.

### API-011 native underline and bar cursors

The real-PTY App test failed when underline/bar modes inverted the complete
wide glyph as a block. Private text metadata now carries the requested native
shape through layout. The painter selects a cursor only for a successfully
painted grapheme, using its actual transformed coordinates. Later text or opaque
background hides it; translucent background tints its color. SuprTUI emits its
steady shape inside the existing checked frame, while the widget owns blinking.
The original wide glyph remains intact, including a cursor on its continuation.

A separate failure demonstration found that an opaque image above the glyph
left the native cursor visible. Image output now retains compact cell coverage
from the already rasterized pixels. It suppresses a cursor under nontransparent
image pixels, including cached image frames; transparent pixels and images below
the glyph preserve it. Coverage follows candidate/acknowledgment state and adds
one bit per image cell, bounded by the existing raster allocation limit.

The corrected tests verify native shape bytes, wide text, both blinking shapes
with a silent child, clipping, covering content, image order/transparency/cache,
removal, failed flush/retry, plain CellFrame shape isolation and host style/color
restoration. Passed: 84 terminal library tests, 12 real-PTY TerminalWidget App
workflows, 13 SuprTUI backend tests, nine renderer integration tests, strict
all-target Clippy, formatting and diff checks. Logs:
`api-011-terminal-native-cursor-{negative,corrected,backend,image-negative,image-corrected,renderer,library,app-final,integration,clippy}.log`.
These captured-output tests verify emitted host commands; they do not add a new
claim of interactive visual verification in every named host.

### API-011 retained screen character, line and tab editing

Four parser-to-screen tests failed for missing character insertion/deletion,
insert mode, line insertion/deletion, scrolling and custom tab stops. Implemented
ICH/DCH/ECH, IL/DL, SU/SD, CHA/VPA/CNL/CPL, relative aliases, forward/backward
tabulation, tab clearing/setting and insert mode. Row rotations bound work to the
scrolling region and clear with the current background. Character shifts preserve
complete wide glyphs; combining marks do not insert a second cell. Backspace
cancels pending wrap. ESC keypad mode state is also retained. Sequence meanings
were checked against https://invisible-island.net/xterm/ctlseqs/ctlseqs.html.

A real shell performs character, line and scroll edits through App at two sizes.
Temporarily disabling character insertion made that workflow fail at cell (3,0);
the mutation was restored. All 88 terminal tests, 13 TerminalWidget App workflows,
strict all-target Clippy, formatting and diff checks passed after restoration.
Logs: `api-011-terminal-screen-edits-{negative,corrected,app-negative,library,app,clippy}.log`.

### API-011 retained terminal construction bounds

The zero-size constructor test failed because invalid screens were accepted
before later operations could underflow. VirtualScreen and Terminal now expose
`try_new`, with the existing nonzero/262144-cell resize limit checked before
allocation. Existing `new` signatures remain and document an immediate panic for
invalid dimensions. Widget construction uses the fallible path, preserves an
error and blocks start until valid resize; its internal 1x1 error placeholder
never starts a child. Normal measured App layout can repair the initial size.

Zero and oversized fallible constructions return InvalidSize without allocating
the requested buffers. The widget test verifies visible error state, failed start,
invalid resize rejection and valid resize/start recovery. A real-PTY App workflow
starts with zero configuration dimensions and verifies the child starts at the
actual measured size, at two viewports. Passed: 91 terminal library tests,
13 existing TerminalWidget App workflows plus the new recovery workflow, strict
all-target Clippy, formatting and diff checks. Logs:
`api-011-terminal-size-{negative,corrected,widget,library,app,recovery-app,clippy-final}.log`.

### API-011 terminal scrollbar pointer controls

The first fixture mistakenly used the four-cell `p-1` spacing utility; its
content height was zero, so the child correctly did not start. Replaced that
fixture spacing with explicit one-cell padding. The resulting negative test
reached READY but clicking the painted track never showed the oldest row.
The widget now maps left press/click and its owned drag through measured content
size, title and padding. Dragging continues across the widget width and ends on
release, leave, blur, stop or hidden scrollbar. No fixed screen coordinates are
used in the implementation.

The real-PTY App workflow passes at two offset/padded sizes: press the track top,
observe ROW00, drag down to READY, release, and verify an unowned drag does not
scroll. A lifecycle test verifies leave/blur/stop/hide cancellation. Passed:
92 terminal tests, 15 TerminalWidget App workflows, strict all-target Clippy,
formatting and diff checks. Logs:
`api-011-terminal-scrollbar-{fixture-padding,negative,corrected,library,app,clippy}.log`.

### Retained terminal tab stops survive resizing

The resize path recreated default stops, losing child-program settings even on height-only changes. The new regression clears the defaults, installs a stop at column 6, and tests height changes, shrinking and growth. It failed before repair (column 9 instead of 6); resize now retains surviving stops and adds new columns without resurrecting cleared defaults. All 93 terminal library tests passed after repair. Logs: `api011-tab-resize-negative.log`, `api011-tab-resize-corrected.log`. These are editing checks, not committed Cairn evidence.

### Native PTY and remaining builder checks in progress

All 355 App widget cases passed on Linux before the new helper regressions. The owned Windows ConPTY module and native probe pass a focused Windows cross-Clippy build. A full cross-build cannot run without the MinGW compiler needed by onig_sys; this is not native proof. Dedicated native verification run 34468541194 tests snapshot d476098127502cd11115a93a9d1b913910f9c74f. Its macOS build exposed openpty pointer mutability differences; the main candidate now supplies valid mutable raw pointers and four Linux PTY cases plus strict Clippy passed. Native rerun remains required.

New helper App cases fail on invisible default content padding and a dropped VDOM click handler. Content/card padding now uses p-0.5 and the default-container matrix passes at 40x16 and 80x24. The VDOM conversion finding remains open under the recorded decision. Logs: api011-core-helper-negative.log, api011-core-helper-corrected.log, api011-conpty-native-macos-job.log, api011-conpty-linux-app-regression.log and api011-conpty-focused-cross.log. None is committed Cairn acceptance evidence.

### API-011 mixed VDOM, helpers and native launch follow-up

The App click probe reproduced the VDOM handler loss. Expanded tests then
reproduced native Element round-trip loss of button callbacks and component
props, discarded element event metadata during component expansion, and a
Unicode hex-color parser panic. The bridge now delivers VElementProps to
registered components, retains native Element factories/metadata through opaque
VComponent payloads, shares activation matching with native builders, propagates
event metadata and applies inline styles after class resolution. Hex parsing
rejects non-ASCII digits before byte slicing. Six VDOM cases pass, including
props/handler replacement, disabled/resized routing, inherited paint and invalid
App style errors. Failure and corrected logs are retained alongside this review.

Search-input default padding hid editable text at 40 columns; smaller terminal
insets fix it. Responsive grid used two columns at 40 because its prefix ignored
the viewport. App now selects sm/md/lg/xl at 40/80/120/160 columns. Grid resizing
and exact boundary/focus tests pass. One initial fixed-grid expectation was
wrong: a widthless root grid sizes to content, so two one-cell columns are
adjacent; the test now asserts the actual content-sized contract. A focus test
also needed an actual callback and no-underline to override button cursor
styling, and no second-frame requirement when its pixels stay unchanged.
All 369 widget App cases pass serially (126.15 seconds); captured in
api011-vdom-all-widgets-corrected.log. These are editing verification, not
Cairn evidence. Remaining macro/catalog/native/Orca coverage is still pending.

Native run 34468541194 failed macOS openpty pointer mutability and Windows GNU
DLL export overflow. The corrected macOS build/clipboard run passed in
34469553622. A target-specific build script disables implicit GNU DLL exports
while keeping explicit FFI exports; all Cargo-input mechanisms now declare
build.rs and native digest scripts include it. Windows then passed its build
and clipboard cases, but ConPTY exposed copied redirected standard handles.
STARTF_USESTDHANDLES with null streams addresses the documented Windows case;
see Microsoft Terminal discussion 15814. Run 34470625419 then hit a clipboard
timeout before ConPTY; the next run executes ConPTY even after clipboard failure.
No native ConPTY acceptance or FFI export-table pass is claimed yet.

Ripwire source quality delta was examined: it flags new inline parser complexity
and VDOM bridge growth along with broad preexisting changes and false dead-code
findings for externally called/trait methods. Split inline parsing by layout,
paint and spacing, and extracted VElement conversion to keep responsibilities
readable. The full-repo invocation also indexed reference checkouts and is not a
useful pass/fail result. Retain source findings for the final production review;
no quality-delta pass is claimed.


### API-011 VDOM menus and public macro routes

The VDOM library suite passed all 19 tests, but its menu tests only inspected
component names. An App test produced an empty frame: `*_with_props` helpers
wrapped props twice and the named factory omitted all four menu families.
Removed the extra Arc and connected MenuBar, ContextMenu, PopupMenu and DialogMenu
to the existing public components. Nine VDOM-related App tests passed, including
eleven menu helper routes at 40x16 and 80x24 with real actions and right-click
context-menu opening. The negative and corrected logs are retained here.

Expanding macro coverage found `el!(div, [node])` matched the generic expression
list arm before the container arm. The advertised invocation failed to compile.
The specific arm now precedes the list arm. The mixed routing test also exercises
el, div and span forms while retaining disabled state, resized targets and native
event payloads. A 16-column input macro was entirely consumed by the original
12-cell horizontal padding. Its defaults now use two horizontal and one vertical
cell, following the existing terminal-sized helper decision. Checkbox accepted
Char(' ') but ignored KeyCode::Space; both now toggle through the same handler.

Six macro App tests passed at both sizes: button callbacks, eight input forms,
editable tab children, checkbox/select state, data-table and progress values,
four toast forms and all six chart forms with actual data geometry. Two fixture
errors were corrected: wait for the updated tab frame before asserting its text,
and expect the progress bar's documented decimal percentage. Neither was a code
repair. Full suite and strict lint refresh are still pending at this point.

Native run 34471509566 passed macOS and Windows clipboard checks. Its Windows
ConPTY probe passed console handles, actual input, environment, directory, resize,
exit 259 and repeated stop, then rejected handle growth after ten failed launches.
This is not a platform acceptance pass. Preserve its output and add per-attempt
counts to diagnose bounded initialization versus persistent leakage; do not
relax the assertion. The API-011 mechanism now invokes the ConPTY record verifier
and declares its complete source/test, workflow, baseline and record dependencies.

Verification refresh: all 377 App tests passed in 129.65 seconds. Strict
`cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --all -- --check`
also passed. Both modified Python mechanism scripts parsed successfully. Ripwire
edit-check reports unchanged public contracts and no incompatible callers for
`menubar_with_props` and `el`. These are editing checks, not Cairn receipts.
Native diagnostic snapshot 7508a65 is running as GitHub run 34472611040; its
per-attempt handle counts remain pending. No native acceptance claim is made.

### API-011 checkbox reader semantics

The actual Orca/GNOME Terminal fixture rejected the retained checkbox with
`checkbox semantic role absent` before implementation. Checkbox now supplies its
role, label, mixed/checked/unchecked state and enabled activation through the
existing App-owned semantic bridge. An explicit Element label remains distinct
from the painted label. Disabled controls expose no assistive action.

The corrected full reader workflow passed at 60x16 and 32x10, including assistive
focus/click, Space toggling, actual Orca mixed/checked/unchecked announcements,
and removal before menu replacement. Existing accordion, breadcrumb, CSS, menu
and simultaneous/successive App checks also passed. The four existing checkbox
unit tests passed. Logs: api011-checkbox-orca-{negative,negative-session,60,32}.log.
These are implementation checks; the full catalog and commitment remain pending.

### API-011 slider reader semantics and native handle diagnosis

The real reader fixture rejected Slider with `slider semantic role absent`.
The control now publishes its slider role, authored label, orientation, finite
current/minimum/maximum values and step. Orca/GNOME Terminal workflows at 60x16
and 32x10 pass focus, numeric range, Right changing 25 to 30, End reaching 100,
and actual changed-value speech. Six existing Slider App workflows pass.
Logs: api011-slider-orca-{negative,negative-session,60,32}.log. The reader adapter
currently offers focus and keyboard interaction; no unimplemented value-setting
action is advertised.

Native run 34472611040 passed normal ConPTY behavior but failed cleanup of ten
failed launches: handles grew from 73 to 84, one per attempt after first-use
initialization. api011-conpty-handles-growth.log records exact counts. The next
isolated native snapshot a142f2e adds per-stage handle diagnostics; it does not
relax acceptance or claim the leak repaired. Run 34473564931 is pending.

### API-011 radio reader semantics

The real reader first rejected the generic group (`radio group semantic role
absent`) and then the named builder (`named radio alpha absent`). Generic radio
options now render as individual cells inside the same horizontal/vertical
layout, with radio roles, undecorated labels and checked states. Their semantic
focus targets use the existing owned custom-focus event route; bounds remain
with the rendered options. Space's explicit public key variant also selects.
Named radios publish their existing App-owned group selection without new state.

All six radio App cases passed after both changes, including measured horizontal
pointer selection, disabled/empty controls, exactly-once callbacks, keyed named
groups, removal and multi-App isolation. Full Orca workflows passed at 60x16 and
32x10: role/labels, disabled actions, assistive focus, arrow navigation skipping
inert choices, selection speech, exclusive keyboard and assistive selection,
and named-builder selection. Logs: api011-radio-orca-negative{,-session}.log,
api011-named-radio-orca-negative{,-session}.log, api011-radios-orca-{60,32}.log.
Catalog reconciliation and remaining reader families are still pending.

The complete widget App suite passed after the reader changes: 377 tests,
129.14 seconds (api011-controls-reader-app.log). Strict all-target Clippy passed.
Native run 34473564931 remains failing: each attempted session creates four pipe
handles plus three console handles, then cleanup drops only six. The failed
CreateProcessW adds one first-use handle only; attributes add none. Per-stage
counts are in api011-conpty-stage-handles.log. This narrows the leak to console
cleanup but does not yet prove the OS is responsible.

## API-011 Select search and reader delivery (2026-09-10)

The new App workflow failed before repair: typing `br` left Alpha selected and Enter merely opened the dropdown. Select now retains a bounded, one-second Unicode lowercase prefix, skips disabled options, keeps matching options visible, cycles repeated letters, and clears the prefix on close and navigation. Both construction routes and multiple selection pass at 24x6 and 48x12; a deterministic clock test checks expiry and the 128-byte bound.

The initial real Orca run rejected the Select because its combo-box role was missing. The renderer now retains the visible header and option rows as real child elements with combo/list/option roles, selected and disabled states, distinct labels, expansion, multiple-selection state and owned assistive-focus events. Orca/GNOME Terminal passes at 32x10 and 60x16 with assistive choices, keyboard navigation that skips disabled options, multiple selection and actual selected-state speech. The workflow waits for the initial focused-option announcement before keyboard selection; otherwise Orca coalesces the immediately preceding focus/state events and does not announce the intervening change. Root combo activation while already expanded still needs review: the shared synthetic click uses the control's full bounds.

Editing checks: 379 App tests passed in 129.72s; strict all-target Clippy passed before the subsequent Windows-only runtime change. Logs: api011-select-search-negative.log, api011-select-search-unit.log, api011-select-semantic-app.log, api011-select-orca-negative-session.log, api011-select-orca-32.log, api011-select-orca-60.log, api011-select-full-app.log. These are editing/review observations, not committed Cairn receipts.

## API-011 Windows ConPTY leak isolation (2026-09-10)

Native run 34474501424 reproduces one leaked handle for each bare OS CreatePseudoConsole/ClosePseudoConsole cycle without Reactive-TUI session, process attributes or child-launch code. Five cycles rise from 73 to 78 handles and remain at 78 after 500 ms. Session failure cycles retain the same growth. See api011-conpty-raw-os-leak.log.

A separate native diagnostic, run 34476091245 at snapshot 1006f67e4b23050b990e548a2554a8447f987b58, compares the OS with the pinned Microsoft.Windows.Console.ConPTY 1.24.260710001 package. After initial runtime setup the OS grows by one each cycle; the SDK stays at exactly 194 handles across all six cycles and after 500 ms. Package SHA-256 is 175640566a3b59c4b132070ee96c2c77e5ab7edd2e92732a5eb3610bbf63d90e. This is diagnosis only, not an application acceptance pass. The recorded replacement decision requires a verified, packaged runtime and a new complete native test. See api011-conpty-sdk-diagnostic.log.

The expanded-combo action defect above is now reproduced and repaired. Orca's explicit combo activation no longer synthesizes a press at the center of the option list. A private click event follows the existing retained-owner focus-event route, with the same trap and disabled checks. Component expansion preserves that event; options keep their measured mouse fallback. Corrected Orca close/reopen and all prior reader workflows pass at 32x10 and 60x16. Targeted App selection workflows and the complete api_focus suite pass, as does strict all-target Clippy. Logs: api011-select-toggle-negative-session.log, api011-select-toggle-32.log, api011-select-toggle-60.log, api011-select-toggle-app.log, api011-select-toggle-focus.log, api011-select-toggle-clippy.log.

Select prop updates: the parent-authored selected value now replaces retained multiple choices when it changes, while unchanged parent props preserve user choices. The App test waits for the actual Alpha/Beta frame before replacing it with Gamma at both viewports. Removing the reset fails that same Gamma assertion (`api011-select-props-negative.log`); the corrected Select-filtered suite passes 38 tests (`api011-select-props-corrected.log`), and all 380 App widget tests pass (`api011-app380.log`). These are editing checks, not Cairn receipts.

Table reader acceptance: the real Orca/GNOME Terminal baseline did not announce navigation to Bea after skipping a disabled row (`api011-orca-tables-negative2/`). LiveTable now publishes its current cell as virtual accessible focus and handles assistive cell focus through its retained owner. Cell focus preserves selected rows and does not invoke row actions. Corrected 32x10 and 60x16 runs verify cell/table roles, labels, keyboard focus, selected and disabled state, assistive focus with unchanged selection, spoken row changes and adapter removal (`api011-orca-tables-focus32/`, `api011-orca-tables-focus60/`). The 35 App tests matching Table pass in `/tmp/api011-tables-a11y-app.log`; these remain editing checks.

DataTable uses the repaired LiveTable cell focus. The real 32x10 and 60x16 reader runs exercise Search table, Next/Prev availability, page replacement, search-driven page reset, cell focus and actual speech for paged and filtered rows (`api011-orca-datatable32/`, `api011-orca-datatable60/`). Strict default all-target Clippy passes (`api011-table-a11y-clippy.log`). The combined table App log is retained as `api011-tables-a11y-app.log`.

## API-011 data-widget reader delivery and transport shutdown

Tree and FileExplorer join Table/DataTable in real Orca 50.1.2 with GNOME Terminal 3.58.0 at 32x10 and 60x16. The focus32/focus60 logs and diagnostics retain both successful App rounds and adapter removal. File toolbar controls now expose descriptive action labels; the negative search-label run could not locate Search files. Seventeen FileExplorer App cases pass. Tree has fifteen App cases, including accessible child checkboxes.

The first 60-column collapse run failed: Orca processed an older Test folder focus event after the immediate refocus action, so it rejected the collapse event as outside its focus. The corrected sequence waits for Test folder speech, separates focus actions by 150 ms (Orca suppresses rapid repeated event types), and requires Documentation folder speech before collapsing. Both geometries pass without reader configuration changes or reduced assertions.

Repeated App shutdown exposed a transport race: closing the outgoing queue while a worker operation was already polling reported a false queue-disconnect error. A deterministic interleaving test fails before the fix and passes after cancellation is joined before queue closure. All six transport tests pass; genuine failures remain errors. These are development checks, not Cairn receipts.

## API-011 Windows process-job correction and Tabs follow-up

Native Windows run 34481972388 passes console IO, environment, resize, normal exit 259, repeated stop, failed-launch handle stability, flood and idle output, and descendant cleanup. The SDK console close alone had left the output reader blocked; suspended launch plus a private kill-on-close job repairs this. The run also builds and loads baseline C exports. Its negative source copy failed before compilation because benches/diff_benchmark.rs was omitted; including benches fixes the harness, and run 34483150446 is pending. No complete native acceptance pass is claimed yet.

The real tab workflow exposed active selection resetting when the parent rebuilds a child component. The reconstructed-child App regression fails with focus on Two while FIRST remains visible. Retaining effective tab props and reconciling authored selection/content fixes it: all nine tab App tests and seven tab unit tests pass, including keyed reorder, changed content, explicit authored selection, and closed-tab persistence/removal/readdition. Strict Clippy passes. Tab/list/panel semantics and explicit assistive focus/activation now reach Orca, but the complete run fails at input text reading because AT-SPI Text is absent; this remains an acceptance obligation. Earlier F11/stage-navigation fixture failures did not demonstrate the tab product defect.


## API-011 readable text inputs and tab reader completion

The missing AT-SPI Text interface is repaired with keyed text runs, editor-derived Unicode boundaries and exported caret/selection positions. Empty fields retain a text run, hard line breaks remain readable, and password runs contain only masks. Painted input decorations are excluded from reader navigation. The consumer test covers empty, Unicode, multiline, password and a grapheme longer than 255 bytes, including Select All. AccessKit length entries are bytes; unusually long graphemes use scalar entries while editor caret endpoints remain at the outer grapheme boundaries.

Orca/GNOME Terminal passes the tabs workflow at 100x32 and 100x60: tab/list roles, disabled skipping, selection and focus speech, initial input value speech, exact keyboard edit, reported caret movement, assistive tab activation, removed panel and App adapter removal. The Python fixture needed the explicit Atspi.Text.get_text interface method to avoid an Accessible method-name collision, and lowercase x to match the unshifted XTest key. The earlier missing-interface failure remains the genuine negative; these fixture errors are not product failures.

The consumer case, all 19 input_acceptance App cases and strict all-target Clippy pass. Logs are api011-text-consumer.log, api011-input-app.log, api011-text-clippy.log and api011-orca-tabs-text32/60. Other text-entry implementations and the remaining reader catalog still require reconciliation. These remain editing checks, not committed Cairn evidence.


Native Windows run 34483150446 is now complete and passes at snapshot b18a5aae47a2c8bf7f2cb3660b63d686c55c573d. The receipt and all six output hashes match. It proves console handles/input/environment, resize, real exit 259, repeated stop, ten failed-launch handle-stability cycles, flood and idle output, attached descendant cleanup, missing/corrupt runtime rejection and all 194 baseline C exports loading. The compiled violating resize returns the old 80x24 dimensions and fails the required 100x30 observation. Artifacts: api011-conpty-native-34483150446/. Final committed inputs still require a fresh native run; these snapshot results are not substituted for current Cairn evidence.


Tab close follow-up: closing an earlier unkeyed tab reset the active editor from seedX to seed. The failing App test is api011-tabs-sibling-negative.log. Header and panel keys now use the tab owner's retained identities; all ten tab App cases and seven unit cases pass. The real reader also reproduced the absent Close Settings tab action. Close now exposes a labelled button and routes to the same retained close handler, preserving disabled and parent-callback checks. Corrected Orca runs pass at 100x32 and 60x16, including assistive close, panel removal and preserved Overview selection. Strict all-target Clippy passes. Diagnostics are api011-orca-tabs-close-negative/32/60; these remain development checks.


## API-011 modal and popover reader delivery

The real overlay workflow at 60x16 and 100x32 verifies modal dialog/title speech, input value speech and editing, assistive focus rejection outside the trap, Escape and labelled Close dismissal, restored opener speech and removal. Popover verifies assistive trigger activation, automatic content focus, input speech/editing, Escape removal and restored trigger speech. Both App rounds detach successfully.

The baseline popover trigger had no assistive activation interface because its behavior lives in a capture handler. A private clickable option now exposes that existing mouse path, survives component expansion and leaves a typed trigger's role, value and text selection intact. The negative reader run is api011-orca-popover-action-negative/; the typed-input expansion regression passes. Twenty-three modal App cases, twenty-four popover App cases, twelve api_focus cases and strict all-target Clippy pass. Final reader logs are api011-orca-overlays-60/100. These are development checks; remaining catalog and platform acceptance are still pending.

### API-011 ProgressBar reader follow-up

The real Orca fixture rejected the baseline because the progress control lacked an AT-SPI Value interface (`api011-orca-progress-range-negative`). LiveProgress now supplies valid determinate numeric bounds and a clamped current value, using the existing Slider/AccessKit path. Invalid and indeterminate props supply no numeric value. The corrected real workflow verifies role/range/value, inert keyboard focus, indeterminate state and a clamped complete value. Orca object navigation reads the label and 25 percent, then 75 percent after a real App callback updates the value, at 60x16 and 100x32 (`api011-orca-progress-60`, `-100`). Automatic progress-update speech did not occur under default reader settings and is not claimed.

The initial speech assertion incorrectly matched digits in a log timestamp. That result is rejected. The shared speech matcher now searches only the text after SPEECH OUTPUT; the strict automatic-update attempt failed (`api011-orca-progress-speech-negative`), and only the corrected object-navigation runs count. Ten ProgressBar App tests and strict all-target Clippy passed. Remaining catalog/reader/platform work is still pending.

### API-011 Chart reader follow-up

The real reader announced the chart title/image role and focus but failed to read keyboard-selected points (`api011-orca-chart-negative`). The chart now exposes its existing tooltip text as a keyed reader-only polite live announcement while a tooltip is present; Escape removes it. There is no empty initial announcement. Corrected Orca/GNOME Terminal runs at 60x16 and 100x32 read both Low: 2 and High: 8 with unit metadata after Home/End (`api011-orca-chart-60`, `-100`); the latter also verifies Escape removes the point. Navigation waits 300 ms between announced points because Orca's installed event spam filter suppresses announcements less than 100 ms apart. This preserves the real reader's default settings. Sixteen chart App/macro cases pass after the final source change. Remaining catalog/reader/platform checks are pending.

### API-011 ScrollView and Image reader follow-up

ScrollView was delivered as a generic section; its added ScrollView metadata initially mapped to Panel. The recorded ScrollPane translation fixes the dedicated role while preserving Pane. Real Orca/GNOME Terminal at 60x16 and 100x32 verifies the scroll label/focus, rejects assistive focus on a clipped input, scrolls to the input, reads its label and draft, and edits it through the real keyboard. Seven ScrollView App cases pass. The same reader workflows verify Image role, public alternative label, hidden fallback cells, and reading an updated label after an App callback changes the image. No Image source change was needed. Evidence: api011-orca-scroll-negative, api011-orca-scroll-image-60 and -100. Strict all-target Clippy passed at this source state. Remaining terminal/dialog/platform and final catalog checks are pending.

### API-011 Terminal reader and shared text follow-up

The baseline terminal had no Terminal role or readable document (`api011-orca-terminal-role-negative`). TerminalWidget now publishes its parsed current screen through the existing TextInput text-run conversion, extracted into private accessibility/text.rs. The document keeps graphemes, masks invisible cells as spaces, excludes the decorative title/scrollbar/cell fragments, and maps the child caret while viewing the current screen. Scrolled history has no false current caret. The unit regression verifies styled CJK/combining text, hidden cells, byte-to-grapheme caret and scrollback; the existing editor Unicode/password/selection export regression also passes. Fifteen terminal App cases pass, including PTY input/output/resize/reaping and cursor shapes/blinking. Strict all-target Clippy passes.

Final display workflows at 60x16 and 100x32 (`api011-orca-display-final-60`, `-100`) verify all five display families, then read actual child output and send input to the retained terminal. Orca's normal terminal-focus presentation reads the current line; its standard keypad-Enter Where Am I command speaks Build terminal and terminal. New child output Received:x is spoken without that command. The fixture moves assistive focus to an external focusable text node before F9 shutdown, since terminal function keys correctly belong to the child. Both successive App adapters detach. Reader catalog workflows and the relevant text/terminal unit selectors are now included in the API-011 mechanism; remaining dialog/platform/catalog work is still pending.

### API-011 dialog reader delivery

The seven dialog families pass real Orca/GNOME Terminal workflows at 60x16
and 100x32. Confirmation checks default focus, disabled actions and completion;
Input checks spoken required validation, editing and its exact result;
Autocomplete checks selected-option speech and assistive selection; Progress
checks numeric value speech and cancellation; Wizard checks validation speech,
step navigation, retained child editing and completion; Toast checks message
speech and expiry without moving background focus; generic DialogBuilder checks
entry speech, editing and Escape dismissal. The probe verifies the six exact
completion/cancellation reports and adapter removal across two Apps.

Safe failure demonstrations retained in api011-orca-input-validation-negative,
api011-orca-wizard-validation-negative and api011-orca-toast-announcement-negative
show visible content without the required spoken announcement. Input and Wizard
validation now publish assertive live text; Toast publishes live text according
to its Alert/Status urgency. Final diagnostics and output are in
api011-orca-dialogs-final-60 and api011-orca-dialogs-final-100. The 26 input,
10 wizard and two toast App cases passed; strict all-target Clippy passed.
These working-tree checks are diagnostic evidence, not Cairn receipts.
Native engine, platform and remaining advertised-control reconciliation stay open.

The additional InputDialog warning case failed with the visible Status node
but no speech (api011-orca-input-warning-negative60). Adding polite live text
makes the warning speak while the entry retains focus. The complete seven-family
dialog workflow passes with that assertion at both sizes; see
api011-orca-dialogs-warning-final-60 and api011-orca-dialogs-warning-final-100.
All 26 input-dialog App cases passed again. Shared text-run extraction also
retains real reader behavior for Tabs and Modal/Popover at 60x16; diagnostics
are retained in api011-orca-shared-text-tabs60 and api011-orca-shared-text-overlays60.


### API-011 reader refresh and native platform diagnostics, 2026-09-10

Orca/GNOME Terminal passed the display, dialog and overlay catalogs at 60x16 and
100x32 after the dialog Escape-policy changes. The display run once timed out
closing its replacement App after completing every control assertion. The driver
had waited only for the replacement window registration. It now waits for a
rendered enabled control, requests its focus, establishes host input as it does
for the first App, and observes focused state before sending the exit key. An
initial handshake without host input failed because inactive adapters suppress
focused state; these diagnostic failures remain recorded. All six corrected runs
passed, including adapter removal, in api-011-reader-active-handoff-*.log. This
is development verification; final committed Cairn evidence remains pending.

Native run 34494618226 passed Windows HTTP transport and all 14 HTTP App cases.
FileExplorer directory copy failed with Windows error 87; operation-specific
errors and native short-name rename cases are added to identify the failing call.
The failure output is api-011-windows-filesystem-baseline.log. macOS HTTP passed;
its unsupported raw-byte filename fixture was corrected to test actual filesystem
identities (including normalization), while Linux byte names and Windows unpaired
UTF-16 names retain their own platform tests. Ten worker tests pass on Linux.
Native run 34497209141 tests these changes plus native image entry points and the
new iTerm2 desktop capture. Neither platform has complete current acceptance yet.

API-011 dialog follow-up: confirmation replacement/disabled-button callbacks and
all seven positioning modes plus explicit bounds pass at both viewport sizes,
including resized pointer activation. Logs: api-011-confirmation-updated-controls.log
and api-011-confirmation-position-modes.log. The noncancelable Wizard keyboard
failure is independently reproduced in api-011-wizard-keyboard-negative.log.
Separating cancellation from keyboard navigation passes all eleven Wizard cases
(api-011-wizard-keyboard-corrected.log). Removed-step reinsertion resets the child
editor as required (api-011-wizard-removed-step.log). The pre-fix broad App run
passes 392 tests in 140.22 seconds (api-011-catalog-app-reconciliation.log); its
coverage did not catch the Wizard bug. These are editing checks, not Cairn receipts.

## API-011 menu and native follow-up checks

The outline baseline failed on missing border glyphs. Menu panels now share the
measured table/modal border painter and reserve their edge independently of CSS
padding. Border colors stay on the glyphs, and border-0 suppresses the outline.
The updated resize test includes that edge. The outside-click fixture now uses
a parent narrower than the viewport and converts UTF-8 offsets to cell columns.
Both changes keep their original callback and dismissal assertions.

The complete local App rerun passes 396 cases in
`api-011-catalog-final-local.log`. Its earlier autocomplete failure was an input
fixture race: waiting for Alpha did not establish that Beta had emerged from the
resize animation. Waiting for the actual click target preserves the veto/result
assertions and passes the targeted and complete runs. Orca passes the base controls
and four menu families at 32x10 and 60x16 after the outline change.

Native macOS snapshot 34503199020 passes all widget groups after the owned-process
cleanup repair. iTerm still stops on its relocation prompt; the owned-window
screenshot proves that startup failure. The private Applications-directory repair
needs native execution. Windows evidence and final catalog reconciliation remain
open. These are editing checks, not Cairn receipts.

Ripwire was also run against src. Test-gate returned 4 with 11 named test files
and 1123 symbols lacking a modeled test edge; this src-only map excludes the
root App integration suite and cannot establish absent behavioral coverage.
Quality-delta returned 2 with 583 major existing-symbol findings, including
unchanged engine paths and new retained-control complexity. Neither tool passed.
Their findings remain advisory input to the final source review; they are not
substitutes for the behavioral checks or permission for unrelated rewrites.

The subsequent library run passes 953 cases with two fixture-only cases ignored
(`api-011-current-library.log`). Seven earlier shared-contract suites pass all
94 cases (`api-011-shared-contract-regressions.log`): component expansion, hook
state/lifecycle, event routing, focus, styling and paint properties. These remain
editing checks until committed Cairn execution records fresh receipts.

### API-011 style and native-host follow-up (2026-09-10)

The Wizard builder's background-color option was configured but not asserted.
Its existing two-size App case now checks the authored RGB value at the step
title and uses reduced motion to isolate styling from the opening fade. All
twelve Wizard App cases pass (`api-011-wizard-style.log`).

Autocomplete now has a two-size cell test for custom input/content backgrounds
and enabled/disabled matching emphasis. A first fixture version treated a UTF-8
byte offset past a border glyph as a terminal column; its failure
(`api-011-autocomplete-style-negative.log`) is not a product defect demonstration.
After correcting the coordinate, the case passes. Temporarily disabling only
the match-rendering branch makes it fail at the first matched letter
(`api-011-autocomplete-highlighting-disabled-negative.log`). The original
implementation was restored immediately. No changed product behavior is claimed.

Native snapshot 34505587248 passes all macOS widget groups. Its iTerm screenshot
shows a normal shell, resolving the earlier relocation prompt. The probe
startup marker is still absent. iTerm 3.7.0's legacy task route discards task
creation/execution errors, whereas its AutoLaunch-directory route reports them:
https://github.com/gnachman/iTerm2/blob/v3.7.0/sources/API/iTermScriptsMenuController.m
The driver now uses the pinned version's direct `--command` option instead:
https://github.com/gnachman/iTerm2/blob/v3.7.0/sources/AppKit/main.m
The child writes its actual terminal size before capture; the driver requires
exactly one visible regular window owned by its process. It still measures real
colored pixels and rejects ASCII fallback. No AppleScript is installed. This
change still needs native execution. Evidence:
`api-011-iterm-autolaunch-baseline/`. Windows remains in progress.

The complete local App suite passes all 398 cases in
`api-011-catalog-398-local.log`, including the additional styles and grapheme
minimum cases. Windows run 34505587248 passes both TLS trust cases and all
filesystem/image groups. Its TerminalWidget case still fails: the first frame
containing `53` retains `ADY>` after it. The test now waits for the complete
numeric row, retaining the exact-result assertion. A lower-level native
terminal check also resizes and submits a second command, waiting for complete
result lines and retaining raw output on failure. Whether this is an incomplete
output frame or a persistent terminal defect remains unverified until native
execution; no product repair is claimed. Logs are
`api-011-windows-34505587248-{terminal-app,https,filesystem-native,filesystem-app}.out`.

The final construction-route cross-check found an inventory wording error:
the core row named secondary/danger button helpers, but the current builder
module and baseline f9e8d7df contain only the public `primary_button` helper.
A baseline source/document/test search for secondary_button, danger_button,
button_secondary and button_danger returns no matches. The row now names
`button`, `input` and `primary_button`. Confirmation and Modal button variants
remain separate required controls and retain their tests. No public API was
removed or renamed. Core helper, macro, VDOM, input, named radio, slider,
layout and menu construction routes were cross-checked against their actual
App cases; the remaining platform blockers stay explicit.

API-011 matrix reconciliation records the existing App behavior and public
construction tests as reviewed. It retains two unresolved platform rows: actual
iTerm image capture and the Windows TerminalWidget complete-result workflow.
The decision `record-app-widget-evidence-while-retaining-the-engine-lifecycle-requirements`
keeps API-012 engine results/stacking/lifecycle and API-014/API-016 obligations
explicit. No claim of DialogEngine or whole-catalog readiness follows from the
App matrix, and all 54 requirements remain necessary before Done. Decision
records now point to their implementations and behavioral checks; the shared
App dialog decision explicitly marks its engine portion unfinished.

Native run 34507954693 confirms direct iTerm command launch and real terminal
startup. Two owned windows exist: the probe (Python) and an extra default shell.
The strict single-window selector rejects that ambiguity. The fixture now sets
a unique OSC window title from its runner and matches it within the child
PID's visible windows. Actual image captures still need a native run.
See `api-011-iterm-direct-launch-baseline/`.

The focused `ripwire scripts --edit-check=owned_windows` finds its two callers
and no incompatible arity. Script quality-delta exits 2 with 18 findings
(`api011-script-quality-delta.xml`), not a pass. Its same-name aggregation
attributes changed `run`/`main` counts to unchanged check-api-signal-ownership.py.
The capture function owns one temporary runner, host process and multi-stage
measurement/cleanup sequence; its branches enforce startup, window identity,
fixture exit and error cleanup. Reported HTTP Handler methods are standard
server overrides reached dynamically. Cancellation handlers accept only the
expected closed-connection/process-already-gone races. Separate standalone
evidence drivers retain explicit bounded subprocess handling and input digests.
No source refactor or baseline change was made solely to quiet these findings.
The script test-gate exits 0 but reports zero tests; it is not behavioral
verification. Native fixture execution and the retained violating cases remain
the required evidence (`api011-script-test-gate.xml`).

### API-011 Windows resize ordering investigation

Native run 34507954693 failed the strict second calculated-result row in both
the Terminal unit workflow and App. The captured bytes and screen are retained
in api-011-windows-34507954693-terminal-{native,app}.out. A deterministic replay
of those bytes through the real Terminal parser/screen reproduces `53ADY>` when
the trailing first-command prompt is processed after resize. Processing that
prompt before resize produces the correct bare `53` row. The diagnostic source
and successful comparison output are api-011-conpty-resize-ordering-replay.rs
and .log in this directory; this proves an ordering-sensitive corruption, not
a product fix or Cairn acceptance. The temporary integration test was removed
after running; its source is retained here for reproduction.

Native diagnostic snapshot 81c125e retains the original immediate-resize failure
case, adds per-stage raw-output/screen/cursor diagnostics, and adds a second
case that waits for the completed first prompt. Run 34510730395 is pending.
Microsoft upstream `_ResizePseudoConsole` writes a resize signal to a separate
pipe and returns after WriteFile; it does not acknowledge parsing of preceding
output (https://github.com/microsoft/terminal/blob/main/src/winconpty/winconpty.cpp).
No runtime resize repair has been made at this point.

The macOS side of native run 34510730395 reached the uniquely titled iTerm
window. Its stage screenshots show iTerm 3.7's “Allow Terminal-Initiated
Display?” confirmation blocking the generated image; pixel counts are all
zero and the fixture cannot finish. Captures are retained under
api-011-iterm-display-permission-baseline. The driver now launches with a
unique `-suite` and process argument defaults selecting Yes for this generated
fixture, then removes only its unique preference domains after the child exits.
This does not alter macOS privacy permissions or the user's iTerm settings.
Source checked at iTerm2 v3.7.0 commit 63cfc92a76881de34863c79f0fb4bd667f5d747f:
sources/AppKit/main.m (`-suite`), sources/Settings/iTermUserDefaults.m (suite
ownership), sources/PTYSession/PTYSession.m:19130 (inline image prompt), and
sources/Infrastructure/iTermWarning.m:602 (saved selection). Syntax passes;
native execution of this change is still pending.

### API-011 terminal resize repairs and remaining native image check

Native 34510730395 passed both direct Terminal cases but failed the App case
at 43 -> 47 content columns. The wider native console reflows a wrapped banner
line; the prior VirtualScreen only extended rows. Six new deterministic tests
failed on that implementation (api-011-terminal-reflow-negative.log). Soft-wrap
markers and compact main-screen reflow now preserve hard breaks, whole styled
graphemes, active/saved cursor positions and bounded newly scrolled rows; the
alternate screen remains fixed. Existing history is retained separately; history
reflow across the viewport boundary needs further review before acceptance.

A separate real-PTY regression failed because resize did not consume output
already fully queued by the reaped owner (api-011-terminal-queued-resize-negative.log).
Resize now drains a bounded batch with the old geometry and keeps public events
for exactly-once delivery at the next poll. A real-PTY burst test verifies the
64 KiB event budget and recovery after polling; a temporary 128 KiB limit was
rejected (api-011-terminal-resize-backpressure-negative.log), then restored.
All 105 local terminal unit tests and all 15 terminal App cases pass. Strict
all-target Clippy passes. The complete 398-case App suite passes in 139.22s
(api-011-catalog-after-terminal-resize.log). An additional retained diagnostic
source/log, api-011-terminal-reflow-property-probe.{rs,log}, checks 100 mixed
Unicode streams through ten width changes each and continued typing; it passes.
Its temporary integration-test copy was removed after execution.

Ripwire qualified resize edit checks and git diff --check pass. The full
quality-delta exits 2 and test-gate exits 4: the git-HEAD comparison includes
the entire unfinished API-011 change and untracked reference trees, with
21993 findings and 125 named test files/1506 reported untested symbols. These
are not passing gates. Focused inspection finds real complexity in reflow_main
(19) and pack_line (27), plus native driver/test complexity and syntactic test
duplication. Reflow branches are covered by the explicit regressions and the
Unicode property probe; graph dead-code results are not proof that those tested
functions are unreachable. Full commitment checks and final review still remain.

Native snapshot 52a3f211148e84b9f8e1aef5ee5cc15fe3f69b89 is running as 34515563869.
The macOS fixture now finishes and captures actual image display/update/removal,
but exact color assertions fail: red (255,38,0), blue (5,50,255), green (8,249,0),
yellow (255,251,0), both in raw screenshot RGB and after its ICC-to-sRGB conversion.
Stage 2 has no colored pixels. Captures are retained under
api-011-iterm-color-profile-baseline. No color tolerance was introduced.
Windows remains pending. These editing results do not establish Cairn acceptance.

History reflow decision process note: the first cairn decide call rejected a missing reversal-history field. A batched edit ran despite that failure; the subsequent decision records the required history before further implementation. This was an orchestration error, not an approval or acceptance pass.

### Retained history reflow follow-up (2026-09-10)

The previous visible-grid reflow left preexisting history rows at their old
width. Two screen regressions failed before repair: narrowing hid the right
columns, and widening failed to join a soft wrap crossing the history boundary.
The App negative also failed with a real PTY: after narrowing and scrolling to
the top, only ABCDEFGHIJK appeared; LMNOPQRST was inaccessible. Captures are
api-011-terminal-history-reflow-negative.log and
api-011-terminal-history-app-negative.log. These are defect demonstrations.

The correction reflows history and the main grid together. An origin anchor
keeps the viewport attached to its text; active and saved cursors use the same
mapping. Full historical lines remain above the origin, while a soft-wrapped
prefix may share the first visible row. The configured history limit is applied
as rows are retained. Height growth alone does not pull older rows into view.
Alternate-screen cells remain a fixed grid. The new screen cases also cover
bounded history, height growth and saved main cursor restoration from alternate.

The final local terminal suite passed 109 tests. The App terminal suite passed
16 tests, including the real PTY narrowing/scrolling workflow at two sizes.
Strict all-target Clippy passed. Logs: api-011-terminal-history-final.log,
api-011-terminal-history-app-final.log and
api-011-terminal-history-clippy-final.log. Formatting and git diff --check passed.
Ripwire edit-check reports the new reflow symbol with one caller and no arity
mismatch; its earlier repository-wide quality and test gate findings remain
recorded above, not relabeled as passes. Native verification of this follow-up
and final commitment evidence are still pending.

The macOS run 34515563869 passed its non-host widget groups, including terminal,
filesystem, HTTP/HTTPS and decoded-image tests, but failed iTerm pixel color
acceptance. Its screenshot also shows white transparent padding. The new
api-011-iterm-color-diagnostic.py compares otherwise identical untagged, sRGB
chunk and ICC-profile PNGs through the existing owned-window capture driver.
Local generation checks confirmed all three images have the expected opaque
primaries, transparent padding, geometry and distinct profile metadata; this is
not macOS rendering evidence. No product encoder or color acceptance threshold
has been changed on this hypothesis.

Native follow-up: Windows job 102999974673 in run 34515563869 completed
successfully, including clipboard, ConPTY failure demonstrations and all widget
workflow groups. Its terminal output logs are retained as
api-011-windows-34515563869-terminal-{screen,native,app}.out. That snapshot predates
the history follow-up. Snapshot 8c587111c4afae31e24092d90f87351a7188d189 is now
running as 34517617898. Only the isolated native verification checkout adds a
macOS diagnostic workflow step; the main workflow's acceptance checks remain.
The fourth diagnostic PNG case resets current SGR background before inline
output to distinguish host default-background effects on transparent padding.

Focused Ripwire follow-up: scanning src/terminal with --quality-delta still exits
2 against the original f9e8d7 baseline (163 findings, 11 preexisting-worse major).
The new history mapping raises reflow_main complexity from the prior measured
19 to 20; pack_line remains 27 with nesting 5. The changed path's --test-gate
exits 4 and reports resize as untested, despite the screen regressions and real
App failure/correction just executed. Its static graph does not establish Rust
test reachability here. Dead-code reports include executed #[test] functions
and used Anchor/Position types. Retained logs name these limits explicitly:
api-011-terminal-history-quality-delta.log and
api-011-terminal-history-test-gate.log. No static-gate pass is claimed.

A standalone history stress probe also passed: 100 deterministic mixed-Unicode
streams (CJK, combining accents and emoji), ten width changes each, continued
input after each resize, viewport heights 2 through 6 and retained history.
It reconstructs public visible/history cells and compares their complete logical
text after every transition. The probe source and output are retained as
api-011-terminal-history-property-probe.{rs,log}; the temporary integration test
was removed after execution. It supplements the permanent targeted tests.

Native color diagnostic 34517617898: untagged PNG, explicit sRGB chunk and sRGB
ICC PNG all produce identical shifted primaries (red 255,38,0; blue 5,50,255;
green 8,249,0; yellow 255,251,0). All also show white transparent padding, with
both explicit ANSI black and reset default background. This rules out the
missing-PNG-profile hypothesis and current SGR background as a remedy. Captures
are retained in api-011-iterm-color-diagnostic/. The next experiment runs the
same generated image with iTerm's UseMetal=NO argument; it is diagnostic only.
The verification branch temporarily runs that short macOS diagnostic with a
separate concurrency group, so full Windows run 34517617898 continues. Main
acceptance workflow is unchanged. Restore the full isolated workflow before
collecting final platform acceptance. Diagnostic run: 34518697890, snapshot
5f4650fe137954cb8045afbeba0b8798d3e7d2a6.

The corrected renderer diagnostic 34519052052 completed successfully. Both
normal launch and explicit UseMetal=NO produced the same four shifted primaries
and white transparent padding; captures are in api-011-iterm-renderer-diagnostic/.
This demonstrates the failure with a generated PNG independent of Reactive-TUI,
and disabling Metal does not remedy it. It does not prove the normal launch
actually selected Metal on this virtual desktop. The first software launch
attempt's command-order error is retained separately in
api-011-iterm-software-launch-negative/ and is not image-rendering evidence.
A focused Swift reproduction now checks sample colors and alpha after iTerm's
DeviceRGB worker round trip and calibrated resize to locate the conversion.

### Native history origin correction

Windows 34517617898 passed ConPTY ownership/resize checks but failed the Terminal
and App command-shell workflows after history reflow: the next result became
53ADY> instead of 53. The combined reflow pulled a historical prefix into the
visible first row, shifting the cursor relative to ConPTY's addressed output.
The captured byte sequence reproduces this exactly on Linux; the failing replay
is api-011-terminal-history-origin-negative.log. The revised decision keeps a
partly historical packed row above the viewport, unless doing so would hide the
active cursor's own line. A separate failing cursor-first-line test demonstrates
that required exception (api-011-terminal-history-origin-cursor-negative.log).
This supersedes the earlier origin choice; it does not discard historical text.

The correction passes 111 terminal tests, 16 App terminal tests and strict
all-target Clippy; logs are api-011-terminal-history-origin-{final,app,clippy}.log.
The native Windows command workflow still needs a fresh run. Earlier full native
Windows success belongs to 34515563869 and is not evidence for this correction.

### Independent native image reference

Diagnostic 34520546487 renders the identical untagged PNG in a small owned
AppKit window next to explicit sRGB red/blue rectangles. Both the PNG and native
rectangles produce exact (255,0,0)/(0,0,255) pixels: 1568 pixels per color, split
between the 784-pixel image region and its 784-pixel reference rectangle. The
transparent part preserves the black window background. This confirms the
pixel expectation against an independent native display path. Its captures and
source are retained under api-011-iterm-native-reference/ and
api-011-native-image-reference.{py,swift}.

The same PNG through iTerm 3.7.0 changes colors and displays white behind its
transparent part, including with explicit sRGB/ICC metadata and UseMetal=NO.
The Swift pipeline reproduction retains alpha=0 through decode/resize. Its
NSBitmapImageRep.colorAt samples change interpretation between color spaces;
those values alone are not treated as proof of the exact color-loss stage.
The independent displayed reference is the reliable pixel comparison.
No product encoder or acceptance threshold was altered to match the defect.

A cell-region workaround for App transparency was considered but not built:
opaque flattening alone would cover clipped text, and region splitting would
still leave iTerm's independently reproduced color failure. Maintaining a host
terminal patch is outside this library commitment. An explicit host-support
decision is required rather than silently narrowing API-014 or weakening tests.
Latest full native run 34520423069 verifies the corrected history origin; its
Windows job is still running at this recording. Main acceptance workflow is
unchanged; the verification branch currently has a temporary macOS diagnostic
workflow and must restore the full workflow before final acceptance work.

Latest macOS acceptance snapshot aa8970450dba4373d6bb7499549cb07896e2e662
(run 34520423069) passes 37 screen tests, 11 native Terminal tests and 14 Terminal
App tests; only the iTerm host output file reports failure among the widget
groups. The history-origin stress probe also passed after the correction
(api-011-terminal-history-origin-property.log). The isolated verification
checkout now has the full original native workflow restored in a local commit;
it is not pushed while that Windows verification is running, to avoid canceling
its job. Remote branch diagnostics are not final acceptance receipts.

The host repair is captured in
.cairn/backlog/repair-iterm-3-7-inline-image-color-and-transparency-behavior-in-the-host.md.
The proposed scope decision preserves the public inline-image APIs and all
other host guarantees, but would explicitly exclude iTerm 3.7 inline rendering
from accepted color/transparency coverage until the host is repaired. No such
exclusion is implemented or accepted yet. The alternative keeps that guarantee
blocking and requires a separately agreed host repair or an upstream fix.
The full commitment, API-011 implementation and final self-audit remain open.


### Approved iTerm 3.7 limit and retained geometry checks (2026-09-10)

The developer answered `ok` to escalation api-011-api-014-api-020. Recorded
that answer with Cairn before updating the specification and widget inventory.
No image API or encoder changed. Exact sRGB measurements remain available in
native captures and the strict diagnostic remains runnable with
`--require-exact-srgb`. The native acceptance path now measures dominant-channel
regions solely for presence, replacement, movement and removal on pinned iTerm
3.7. The approved exclusion covers color accuracy and transparency only.

The local generated-image demonstration passed corrected geometry with a shifted
red channel, and rejected strict color accuracy, missing images, stale images,
unremoved images and unmoved images. See api-011-iterm-geometry-falsification.log.
The default Python lacked Pillow; reran successfully with /usr/bin/python3,
which provides Pillow 12.1.1. Native rerun 34529569174 is pending; none of this
local demonstration establishes native acceptance. Prior run 34520423069 completed
with Windows clipboard, ConPTY and all native widget groups passing; macOS failed
its strict iTerm color check, as retained in the diagnostic evidence.


### Widget quality review before committing the implementation

Ran Ripwire quality-delta and test-gate. They returned 2 and 4 respectively;
neither is recorded as a pass. The whole-tree scan includes downloaded reference
projects and cannot distinguish external Rust trait calls or App-routed handlers
from unused code. The retained filtered report api-011-current-quality.json
contains 3,036 project-source/test findings, including 704 gating rows. Large
clone pairs primarily join independent App scenario setup; small clone rows also
pair unrelated constructors and accessors across subsystems. Retained public API
methods must not be deleted merely because this graph has no internal caller.

Read the error-handling findings in clipboard capture, ConPTY execution, HTTPS
fixture handling and Orca process cleanup. Timeout wrappers raise explicit errors;
Orca retries termination with SIGKILL and a bounded wait; HTTPS tolerates a reset
from the deliberate untrusted-certificate case. These are not successful no-ops.
Read the new nesting and state paths in ElementBuilder::build, paint_frame,
FocusManager::apply and ComponentRuntime::expand. The branches retain callbacks,
image/fallback ownership, focus restoration and caller metadata. Existing App
cases exercise those paths. Kept those cohesive steps together rather than
introducing a shared abstraction across unrelated widget behaviors. Churn counts
include the prior API recovery sequence and do not establish a present defect.
The formal inherited checks and native proof remain required.

Reprocessed the historical run 34520423069 app-iterm screenshots through the new
geometry classifier: both initial regions and both replacement regions contained
784 pixels, moved down/right, and the removed stage contained zero classified
pixels. This is diagnostic corroboration only; fresh native evidence is pending.


Current editing checks: 970 library tests passed (two ignored), all 399 App
acceptance tests passed, strict all-target Clippy passed, and cargo fmt --check
passed. Logs are api-011-current-{library,app,clippy}.log. These are editing checks,
not Cairn receipts; the candidate still needs its implementation commit.


The broader default-suite editing check also passed: 1,766 passing tests across
64 reported targets, zero failures and 37 ignored cases (including 34 documentation
examples). It ran through scripts/check-default-suite.py with its 300-second
process-group deadline. See api-011-current-default-suite.log. Native macOS run
34529569174 passed all widget groups and five iTerm modes; its unchanged receipts
and captures are retained under docs/analysis/widget-platforms/darwin. The Windows
job passed clipboard verification and is still checking ConPTY and widgets.


### Standard compiler checks on developer-provided machines

The developer made Windows and macOS machines available for testing. Both SSH
connections succeeded. Windows is x64 build 26200 with Rust 1.98 MSVC; the MacBook
runs macOS 26.6.1 arm64 with Rust 1.97.1. Created uniquely named temporary source
checkouts from the same local Git bundle, revision ceff3560cc7f097a8d97128557582af0b36672ac.
These supplemental probes do not change clipboard contents, trust stores or
terminal application settings. They exercise terminal, HTTP, filesystem and
image App/native paths without a GUI host capture.

The repaired Sixel encoder no longer imports sixel_rs anywhere in the Rust tree.
Removed its unused Cargo dependency and the four orphaned lockfile packages
(sixel-rs, sixel-sys, autotools and make-cmd). This realizes the existing encoder
replacement decision and removes an unnecessary MinGW/autotools prerequisite.
Cargo check passed; 87 image-filtered tests passed afterward. The new dependency
set requires refreshed formal native receipts even though the renderer code did
not change. The earlier passing macOS receipt remains historical evidence.


Hosted run 34529569174 completed with macOS passing and Windows failing only
`a_silent_endpoint_hits_the_owned_deadline`: the request reported a timeout, but
the whole call exceeded the test's one-second bound. Windows clipboard, ConPTY,
terminal screen/native/App groups and the other widget groups passed. Retained
its HTTP and terminal logs with the run number. Do not treat that run as an
overall native acceptance pass. Added elapsed deadline-check diagnostics to the
same test without relaxing either its 200 ms requested deadline or its one-second
assertion. The local HTTP group passed. Fresh run 34531816183 checks that diagnostic
and the dependency cleanup; it is pending.

The developer's MacBook passed all eleven selected groups (188 tests total) on
snapshot ceff356 with Rust 1.97.1. Its unmodified logs and machine record are in
api-011-native-macbook. Windows/MSVC remains in progress on that same snapshot.
These supplemental checks do not substitute for GUI host or TLS trust evidence.


### Windows 11 failed-launch handle baseline

The initial supplemental MSVC build actually completed successfully, but the
PowerShell launcher lost its process exit-code handle and reported an empty
status. Preserved that log and fixed the launcher to retain the process handle;
the bounded build rerun returned zero. The first full ConPTY probe then failed
its handle assertion: 102 before failed launches, 141 after every one of ten
attempts. All eleven selected Windows groups separately passed (171 tests).

Independent std-only Rust probes isolate initialization from PTY ownership.
Formatting an OS error alone retains one handle. An ordinary missing-executable
std::process::Command launch, without Reactive-TUI, increases from 77 to 123 once
and then remains at 123. Raw OS ConPTY stabilizes at 126; raw SDK ConPTY stabilizes
at 129 across six cycles, unload/reload, another six cycles and 500 ms waits.
See api-011-windows11-raw-handle-reference.{rs,log}. This establishes independent
Windows initialization, not a reason to increase the leak allowance.

Recorded a Judged decision before warming the independent missing-process path
in the acceptance probe. The original ten failed PTY attempts and allowance of
two handles remain unchanged. The corrected native probe passed. A temporary
isolated mutation deliberately leaked one file handle per attempt and failed the
same handle assertion; restored the exact corrected source immediately afterward.
The final corrected binary rebuild/rerun is in progress. Original failure and
selected-group logs remain in api-011-native-windows11; final logs will supplement
them. No production cleanup code changed for this measurement correction.


The final corrected Windows 11 ConPTY rerun passed, with 135 handles before and
after every failed launch. The deliberate leak control grew to 145 and failed.
Thirty separate HTTP deadline trials on that machine each passed the unchanged
one-second internal assertion; full process durations ranged from 222.7 to
273.0 ms. The PowerShell summary command failed to measure hashtable keys; checked
the retained 30 raw zero-exit test records independently in Python and wrote a
separate summary. No trial was rerun or discarded to manufacture a pass.

Hosted run 34531816183 also passed both macOS and Windows, including the unchanged
HTTP bound with added diagnostic output. The historical single Windows timing
failure was not reproduced; its cause remains unconfirmed. The final failed-launch
fixture change requires another committed native run, now being started from
bd3ba95. The widget matrix now records reviewed coverage for its last image and
terminal rows. Dedicated lifecycle, image, entry-point and remaining requirements
are still required by the commitment; no overall readiness claim is made.


### API-011 acceptance refresh: stable Python inputs and departed App discovery

The first committed widget run passed all 399 App workflows and all Orca
catalogs, but imported Python helpers created bytecode inside declared inputs.
Cairn correctly withheld a receipt. The mechanism now sets
PYTHONDONTWRITEBYTECODE for its children; the next run kept the candidate stable.
That run recorded a failure while discovering the replacement App after the
display workflow at 100x32. AT-SPI reported that the old application had already
exited during child enumeration. The captured session is
`api-011-orca-disappeared-app-session.log`.

Discovery now skips only the exact atspi_error code 0 message indicating that
the application no longer exists. It checks remaining applications and retains
the existing five-second replacement deadline. Other errors still propagate.
The retained baseline and fixture demonstrate the original failure at both name
and child-count reads, successful discovery of the replacement, and propagation
of three unrelated errors. The real 100x32 display workflow passes with reader
speech, child input/output and adapter removal. Its corrected session, Orca and
callback logs are retained beside this review. Full committed acceptance and
refreshed native receipts remain required because the reader helper is inside
the native recorders' declared tests directory.

Dependency inspection also found that the ConPTY negative-build source copy
includes `benches`, whose presence is required by the Cargo manifest, but the
native recorder and widget mechanism do not declare it. Add that dependency
before the next native run; do not edit an old receipt to treat it as covered.


The departed-App correction also passed three further complete 100x32 display
reader workflows (`api-011-orca-disappeared-app-repeat-{1,2,3}.log`). Each retained
its speech, child input/output and replacement/removal assertions.

The ConPTY dependency declaration now includes `benches` in both its native
recorder and the widget mechanism. A disposable copy of the exact negative-build
inputs, initially omitting that directory, reproduces Cargo's missing
`diff_benchmark` manifest error with `cargo build --locked --offline --example
conpty_probe`. Restoring the benchmark sources makes that build pass. The paired
logs are `api-011-conpty-bench-build-{negative,corrected}.log`. An initial metadata
probe accepted the missing files and therefore did not demonstrate the build
requirement; its diagnostic is retained separately. Native run 34537629660 checks
the committed input union at snapshot eb453944cc27341d6b99c71f790708127bfd1711.
No native receipt from the preceding snapshot was edited to change its digest.


Native run 34537629660 completed the macOS job and Windows ConPTY and widget
steps successfully. All captured native widget and ConPTY output hashes match
the current input digests, including the corrected reader helper and benchmark
dependency. The Windows clipboard step failed independently: its first 13-byte
copy returned the existing 15-second timeout error; the test finished in
15.23 seconds. The retained failure diagnostic does not explain the PowerShell
stall. The previous same-input Windows clipboard record passed; neither it nor
this failed run establishes the cause. The failed Windows job is being retried
once on the unchanged snapshot to check recurrence. No timeout was extended,
no application code changed, and the failed output remains evidence.

### Windows clipboard retry, 2026-09-10

GitHub run 34537629660 attempt 2 completed successfully on the unchanged
`eb453944cc27341d6b99c71f790708127bfd1711` snapshot. Its clipboard probe
completed five native round trips in 7.85 seconds; both process lifecycle
cases and the Windows console adapter check passed. I verified the current
committed input digest and both captured output hashes before importing the
three Windows clipboard evidence files. ConPTY and widget steps also passed;
their already-valid first-attempt records remain in place.

The first attempt timed out on its first 13-byte copy at the existing
15-second deadline. Its diagnostics remain in
`api-011-windows-34537629660-clipboard-platform-failure.log`. The cause is
unconfirmed. The unchanged retry demonstrates a passing run, not a runtime
fix or proof that the intermittent timeout cannot recur. No deadline was
extended for this retry.


## API-012 engine implementation and failure demonstrations

The construction-only engine now owns shared sessions and presents all six
dialog families through retained App controls. I examined the close path,
mailbox reservation, callback ownership, host release, input revision, modal
presentation and remote-work activity checks. Open reserves a Closed event
slot; close removes the active entry before waking consumers or invoking
callbacks. Host removal cancels sessions even when a controller survives.
The exit presentation has its own cancelled deadline and cannot accept input.
Callbacks stored by the same engine can use a weak controller to avoid cycles.

The original DialogBuffer ignored z-index: its acceptance assertion returned
[2, 1, 3] where [1, 3, 2] was required. The retained diagnostic is
`api-012-dialog-buffer-before.out`. The corrected implementation sorts by
priority and replaces duplicate IDs; its test passes in
`api-012-corrected.out`.

Two controlled mutations tested the actual falsifiers. Removing the input
revision comparison left the edited `seedX` visible after resetting to `seed`;
the frame-driven test failed at its deadline. Replacing the close-event result
with Cancelled failed the assertion for Selected("archive"). The logs are
`api-012-input-reset-mutation.out` and `api-012-discarded-result-mutation.out`.
Both mutations were restored in a finally block. All 25 lifecycle tests then
passed (`api-012-corrected.out`). The earlier
`api-012-input-reset-before.out` used a faulty fixture that matched `seed`
inside `seedX`; it is retained as a diagnostic and is not a failure proof.

The dedicated development runner passes 25 lifecycle tests, 37 dialog unit
tests, 12 modal unit tests and two real HTTP engine workflows, and builds the
documented example. One TLS fixture probe is ignored in the unit selection;
it is exercised by the separate inherited HTTP mechanism and is not counted
as a pass here. The modal clock test covers an 800 ms duration, closing and
zero duration. Strict Clippy passed before the final clock test addition;
the full default suite and final Clippy run are still in progress.

Ripwire edit-check reported no incompatible caller it could identify. Its
quality-delta exited 2 and test-gate exited 4, not passes. The map includes
downloaded terminal reference sources and names trait methods and tests as
dead code despite their execution. Relevant changes include four additional
branches in modal rendering for custom motion and error propagation, plus
owned shutdown cleanup in the new engine. I inspected those paths and retained
the explicit cleanup logic. Duplication reports include short lock accessors,
lifecycle handlers and the moved input-render adapter; combining unrelated
owners to remove these small patterns would obscure ownership. The test gate
also names legacy dialog handlers and native bindings; the full library/widget
suite covers the retained Rust paths, and native binding recovery remains
API-017. This inspection does not replace committed Cairn evidence or the final
commitment review.

The final development `cargo test --locked` run passed, including 974 library
tests, all 401 widget workflow tests, the 25 engine lifecycle tests, and 42
doctests. Ignored tests remain explicitly ignored in the captured output.
Strict `cargo clippy --locked --all-targets -- -D warnings` also passed after
the clock test addition. Logs are `api-012-full-suite.out` and
`api-012-clippy.out`. Formatting and `git diff --check` passed.

### API-012 native dependency correction

Native run 34544351853 failed on macOS while compiling the library: the
new platform-independent engine referenced async-channel, but Cargo declared
that crate only under Linux. The complete failed job log is
`api-012-macos-34544351853-job.log`. The same target restriction also applied
to futures-util; the lifecycle tests use futures-lite. The correction promotes
async-channel and futures-util to shared dependencies and adds futures-lite
to development dependencies while retaining its Linux runtime declaration.
No dependency version changes. Native verification must rerun on the corrected
manifest; the preceding Linux records are now stale too.

A development no-default-features check also failed at the preexisting
`src/display/adaptive.rs` unconditional Tokio import. This is unresolved
API-015 work, not a passing configuration or a clipboard runtime finding.
It remains in the current commitment's required work.

After the dependency correction, the default all-target build and all 25
engine lifecycle tests passed. Linux Wayland, xsel and xclip records were
regenerated against the corrected manifest; all round trips and child cleanup
checks passed. Corrected native snapshot
`61e65faf9c9b1c5e6bb5a0d2610e59a75012852a` runs as GitHub 34544666230.
The superseded 34544351853 run was cancelled to release its concurrency slot;
its Windows job is incomplete and is not a pass. Both earlier job logs are
retained. No stale native output is imported as corrected evidence.

The corrected macOS job passed completely. Its clipboard record matches
the current input digest and both captured output hashes. The native widget
record matches the current digest and all 72 captured output hashes. I also
inspected the three App/iTerm image stages: initial placement, moved update,
and removal. The existing approved iTerm2 3.7 color/transparency limitation
remains explicit in host.json; this run does not expand that claim.
Windows verification is still running.

Corrected native run 34544666230 completed successfully on both macOS and
Windows. I verified the Windows clipboard digest and two output hashes,
all six ConPTY output hashes, and all 15 Windows widget output hashes against
the current committed inputs before importing them. All three platform
verifiers now pass: five clipboard backends, ConPTY, and macOS/Windows widget
evidence. Both new engine HTTP completion/cancellation workflows also passed
on Windows (within its 16 HTTP App cases). The failed original macOS build
and incomplete cancelled Windows run remain in the review history.

### API-011 Orca menubar reopen ordering

The refreshed mechanism passed all 401 App workflows, then failed the 60x16
reader workflow at `menu checked state absent`. The retained session and
Orca logs are `api-012-orca-menu-checked-session.log` and
`api-012-orca-menu-checked-orca.log`. Three unchanged repetitions passed, so
this was not a consistently failing checkbox. The failed reader log ends its
menu announcements at Menu file, after the menu checkbox and Recent menu.

I traced assistive activation through PlatformNode::do_action to
accessibility::connection::Actions::do_action: it enqueues an action and
wakes App. A successful D-Bus reply does not mean App has already handled
the click and removed the menu. The reader test sent Down immediately after
that reply and could reuse the old checkbox before its queued close.

A deterministic ordering probe executes the actual first-stage Python
workflow with delayed click delivery. The original workflow fails at the
same checked-state assertion; waiting for menu removal and menubar focus
restoration before Down passes. A checkbox that never becomes checked still
fails the corrected workflow. The probe models that scheduling boundary;
it is not evidence of real Orca speech and does not establish the exact
thread interleaving of the original failure. Its source and before, corrected
and unchecked outputs are `api-012-menu-ordering-*`.

The six-line fixture correction adds those two state waits, retaining the
existing deadlines, checkbox assertion, speech assertion and remaining menu
workflows. It changes no Rust control behavior. Four complete real Orca runs
then passed: one at 32x10 and three at 60x16, with all four menu families and
remaining core controls. Their session, reader and callback logs are retained
in `api-012-orca-menu-corrected-*`; unchanged repetitions remain in
`api-012-orca-menu-repeat-*`. Whitespace validation passed. The full committed
mechanism still must rerun; the native recorders also declare the changed
tests directory, so their records must be refreshed without editing receipts.

Native run 34546990264 verified the changed test-input directory on snapshot
`b4b9f1fcc3230f5eb83416138b9fb3b7e2dd57ed`. macOS passed completely;
Windows ConPTY and widget checks passed. I verified the current input digests
and every captured output hash before importing those records. The ConPTY
and widget verifiers pass.

Windows clipboard failed its first 13-byte copy at the existing 15-second
deadline (15.14 seconds for the test). The diagnostic is
`api-012-windows-34546990264-clipboard-failure.log`. This repeats the earlier
intermittent hosted-runner timeout; its cause remains unconfirmed. The
clipboard inputs did not change with the Orca fixture, and their earlier
34544666230 passing record still matches the current digest. I retained that
record and the new failure, and requested one unchanged Windows-job retry.
No timeout or acceptance assertion was relaxed. The retry is pending; the
current API-011 native prerequisites are the separately passing ConPTY and
widget records.

## API-013 keyframe contract boundary (2026-09-11)

Declared the animation/screen acceptance footprint. The first Cairn baseline
failed because the dedicated test target did not yet exist; that receipt does
not demonstrate a runtime defect. Added two public API acceptance cases and ran
`cargo test --test api_animation_screens -- --test-threads=1`: both compiled and
failed. Sampling typed f32 0 to 10 at 0.5 returned 0 instead of 5. Converting
untyped x endpoints 4 and 12 returned 0 at the initial endpoint instead of 4.
No corrected-case pass is claimed. Screen, relative-value, easing and lifecycle
coverage remains to be built; these two tests are not complete API-013 acceptance.

Examined `src/animation/keyframes.rs` (constructors and sampling),
`src/hooks/animation.rs` (AnimatableValue and use_keyframes), the API-013
contract, RAPI-10 and the already approved radians escalation. KeyframeAnimation
currently accepts every Clone type for typed construction/sampling and only
adds Default for untyped conversion. Clone supplies no interpolation or
conversion behavior. The separate AnimatableValue trait exists, but is not a
bound of the direct keyframe API; its scalar conversion also cannot preserve
arbitrary colors, transforms or tuples. A proper explicit value trait changes
the source contract for callers using their own Clone types. Escalate that
compatibility choice before implementing it, as the commitment requires.

## Windows clipboard retry result (2026-09-11)

GitHub run 34546990264 attempt 2 completed successfully on unchanged snapshot
b4b9f1fcc3230f5eb83416138b9fb3b7e2dd57ed. Imported only the Windows clipboard
record and its two captured outputs after comparing its committed-input digest
with this candidate and checking both output SHA-256 hashes. The initial
PowerShell clipboard timeout remains recorded in the earlier failure log; this
retry does not establish its cause or claim a runtime fix. No deadline or
assertion changed. Current test additions do not change the clipboard footprint.

## API-011 file-announcement observation race (2026-09-11)

API-011 receipt 20260911T100432988Z failed in the 32x10 data-control
Orca workflow at "Orca did not announce file". Its retained Orca log shows
focus moving from Docs to guide.txt and actual guide.txt speech at
06:04:27.413342. The fixture set its speech-start marker only after querying
the navigated tree, then requested focus on the already-focused file. It could
therefore exclude the navigation announcement without causing another one.
An unchanged 32x10 repeat passed; this was not an absent file announcement.

Demonstrated the ordering with the real App, GNOME Terminal and Orca: inserting
a wait for the guide.txt announcement immediately before the old marker made
the old assertion fail. Moving the marker before Enter passed with that same
forced ordering. Both scripts and complete session/Orca logs are retained in
api-013-file-speech-forced-{negative,corrected}. The original failure logs are
in api-013-file-speech-failure. Removed the artificial ordering wait from the
final fixture. Kept the speech assertion and deadline, and added an explicit
FOCUSED assertion. The final 32x10 data-control workflow passed.

Ripwire impact/uses points to the Orca runner. edit-check reports zero
incompatible callers; its definition-count change includes the saved proof
copies of exercise, not a changed call signature. git diff --check passed.
This repairs the evidence observer; no runtime or Orca behavior was changed.
Full formal widget acceptance and fresh native records still remain pending.

The final 60x16 data-control workflow also passed; its session log is retained.

## Fresh native records after API-013 regression-test additions (2026-09-11)

GitHub run 34587766694 passed on Windows 2022 and macOS 14 against snapshot
33a16cc7fed82d343b212bc03325ae9cca8b604a. This snapshot matches the current
committed union of the native mechanisms' inputs, including the keyframe tests
and the final Orca file-speech fixture correction. The superseded run
34587207016 was cancelled because its test inputs were obsolete.

Downloaded platform-specific artifacts and compared their source commit, system,
pass result, current declared-input digest, and every recorded output SHA-256
before importing. Imported Windows ConPTY and Windows/macOS widget records;
both native verification scripts pass. The macOS record includes all 72 output
hashes and preserves the approved iTerm2 3.7 color/transparency limitation.
Windows clipboard also passed in the fresh job; the already-current clipboard
record from the earlier unchanged retry remains sufficient and was not replaced.
These records still require the complete API-011 Cairn acceptance run.

## API-013 implementation progress: keyframes and runtime (2026-09-11)

This is development evidence, not API-013 acceptance. After the approved
keyframe compatibility decision, seven public API tests compiled and failed
against the old sampling/conversion code: numeric midpoint, destination easing,
sorted/duplicate offsets, tuple conversion, numeric endpoint conversion, sparse
property interpolation, and retained discrete properties. The expanded target
now passes 12 tests, including exact f64 values, RGBA, CSS units, compound
values, custom trait implementation, ambiguous/missing-property errors and
checked lossy-conversion rejection. Command: cargo test --test
api_animation_screens -- --test-threads=1.

The keyframe hook depends on the shared hook runtime. Three new isolated unit
tests failed before its repair: final progress was never delivered (empty list
instead of [1.0]), an ID collided after cancellation (1 equals 1), and callback
reentry exceeded its two-second deadline. After assigning monotonic IDs and
delivering snapshots outside registry locks, those three passed. A fourth
bounded test then exposed callback-capture Drop reentry under cancellation
locks; moving disposal outside the lock made all four pass. Command: cargo
test --lib keyframe_runtime_ -- --test-threads=1. The negative reentry runs
used isolated test processes; a deadlocked worker was not joined after the
assertion deadline, and the failed test process exited.

The implementation is incomplete: keyframe hook ownership, relative values,
active-screen input, transition output, App integration evidence, migration
documentation, lint, regression verification and formal API-013 acceptance
remain pending. Do not use the 12-test target alone as proof of API-013.

## API-013 resumed work checklist

- Complete: finish retained keyframe hook ownership and verify rerender,
  completion, cancellation and unmount behavior.
- Pending: repair and verify relative property values and active screen input
  with visibly changing transition frames.
- Pending: document custom keyframe migration; run formatting, lint and
  regression checks, commit implementation, and follow Cairn acceptance.

The account switch preserved the working tree at base 0448aee. Earlier test
results above are recorded development results, not fresh acceptance receipts.

## API-013 resumed keyframe ownership results

The two added ownership tests first failed: cleanup left the task registered,
and rerendering created a different animation owner. Retained storage and weak
runtime callbacks corrected both. An aborted-render test then failed because
the pending effect never installed cleanup. A pending cleanup guard now also
cancels that work; unchanged renders do not own a second cancellation guard.
Cancellation serializes with value delivery after user-defined interpolation,
and a generation check rejects a sample that stopped/restarted playback. A
bounded reentry test verifies interpolation can stop its own animation.

Fresh development verification using this repository's target directory:
- cargo test --lib keyframe_ -- --test-threads=1: 13 passed.
- cargo test --test api_hook_lifecycle -- --test-threads=1: 7 passed, including
  actual App/SuprTUI frames keyframe:2, keyframe:6, keyframe:10, then removal.
  Escaped play/seek calls cannot update the removed component's value.
- cargo test --test api_animation_screens -- --test-threads=1: 12 keyframe
  cases passed; the new active-screen Enter test failed (0 callbacks, expected 1).
- cargo clippy --lib --tests -- -D warnings: passed.

CARGO_TARGET_DIR was inherited as another project's shared target directory.
One waiting test command was terminated (exit 143), and verification resumed
with CARGO_TARGET_DIR=/home/shawn/workspace2/reactive-tui/target. Other owners
and their builds were left running.

Ripwire edit-check reports seven callers and no incompatible arities for
use_keyframes. Its quality-delta exits 2 with 1557 gating findings, including
untracked reference checkouts; this is not a passing quality gate. Its test-gate
exits 4 and lists 74 test files plus untested symbols, also including reference
code. These broad static results do not establish runtime correctness or
replace the requirement mechanisms. Remaining API-013 screen input, relative
values and transitions are not repaired or accepted by these keyframe passes.

The full library run passed: 984 tests, two intentionally ignored fixtures.
The custom KeyframeType migration doctest passed. Workspace formatting passed
after formatting the newly added screen-input test. The repaired keyframe slice
is verified development work; the screen-input failure remains intentionally
visible and prevents API-013 acceptance. The mechanism now includes the hook
unit tests and the App lifecycle target as well as the keyframe/screen target.

Examined the 70 Ripwire findings on changed paths. Trait implementations and
tests marked dead are exercised by Rust dispatch and the executed test runner.
The short infallible constructors delegate to different checked constructors;
their shared panic-report shape does not justify another helper. Value-type
conversions have distinct error contracts. The owner-liveness comparisons to
text-input, renderer and scheduler predicates are unrelated state ownership,
not reusable implementations. Similar cleanup tests exercise explicit cleanup
and last-owner drop separately. Runtime churn records the repair history. No
production behavior was weakened to reduce these static counts.

## API-008 platform refresh after keyframe repair

- Complete: refresh native clipboard records and rerun API-008 through API-012.
  All now have committed passing receipts.
- Complete: implement and verify the approved API-013 owner-bound animation targets.
- Complete: commit the target repair and refresh API-001 through API-010 evidence.
- Complete: repair and verify the native API-011 cancellation fixture.
- Complete: refresh API-011 through API-013 acceptance.
- Complete: refresh ABI acceptance and maintenance checks.
- Complete: repair and verify the default-suite keyframe cleanup assertion.
- Done: refresh committed evidence after the test correction; the embedded rerun passed, with its unexplained intermittent timeout retained.
- Done: declare and verify dedicated API-014 image acceptance; current receipt passed.
- Done: complete corrected API-015 matrix and current evidence (20260911T180926733Z).
- Done: repair the API-011 cancellation marker and refresh native evidence; API-011/20260911T163552235Z passed.
- Done: refresh dependent evidence; API-012, API-013 and API-014 passed.
- Done: record the API-015 baseline; no-default compilation failed E0433.
- Done: repair Orca image navigation and verify API-011 against current native records.
- Done: implement and verify API-015 feature configurations (receipt 20260911T180926733Z).
- Done: refresh all 34 inherited requirements after API-015.
- Done: implement and verify API-016 entry-point acceptance (receipt 20260911T233538835Z).
- Done: refresh all 34 inherited requirements after API-016 (last receipts 20260911T234522876Z/877Z).
- Done: API-017 native C/TypeScript behavior and ownership; formal receipt 20260912T120716719Z.
- Done: repair the approved isolated Kitty host and verify all API-014 image routes (receipt 20260912T152756874Z).
- Done: refresh API-001 through API-017 and all 34 inherited requirements after the Kitty and toast repairs (last renderer receipts 20260912T155254797Z/798Z).
- In progress: implement the approved API-018 Props contract and complete its documentation/validation mechanism.
- Pending: API-019 complete residual audit contracts.
- Pending: API-020 final regression and independent review, including the captured image-runner timeout cleanup defect.
- Complete: refresh native API-008 evidence after the committed screen repair.
- Complete: refresh API-011 native records and acceptance after the screen repair.

API-001 through API-007 have fresh passing Cairn receipts committed after the
keyframe repair. API-008 failed only at the platform-record verifier: all 13
clipboard behavior tests and two process-lifecycle tests passed, but Wayland's
record had the old source digest. Native Windows/macOS records also need the
new source digest. No clipboard code or deadline has been changed.

The new Wayland and xsel runs each passed five real native round trips and
two process-lifecycle checks. The xclip run is also being refreshed. Native
Windows/macOS GitHub run 34593782610 uses snapshot
baeaf6ca4b65f614278296175b001bf1574676e1. Before pushing, the snapshot's full
union of clipboard, widget and ConPTY declared inputs was compared byte-for-byte
by Git tree entries against this candidate. The snapshot advances the existing
verification branch without changing this working branch. Import requires
matching input digests, executed case markers and intact output hashes.

The xclip run passed too. All three Linux records were checked against the
current committed-input digest and both captured-output hashes.

The macOS job completed successfully. Imported its clipboard record only after
checking snapshot identity, current input digest, native system, case markers
and both output hashes. Windows remains pending.

Windows completed successfully too. Imported its clipboard record after the
same snapshot, input-digest and output-hash checks, plus the Windows adapter
marker. The platform verifier now reports all five backends current.
The complete native run 34593782610 passed on both platforms.

## API-011 native records after keyframe repair

The fresh API-011 run passed its App widget workflows, focused behavior checks,
and real GNOME Terminal/Orca cases. It then failed because the existing native
ConPTY record had the previous source digest. The Windows/macOS job already
completed successfully on snapshot baeaf6ca4b65f614278296175b001bf1574676e1
(GitHub run 34593782610). Imported ConPTY and both widget records only after
checking that commit, the current declared-input digest, platform, pass result
and every recorded output hash. Both native verifiers now pass. The approved
iTerm2 3.7 color/transparency limitation and its measurements remain intact.

Native record bytes retain their produced line endings. git diff --check flagged
CRLF on changed Windows JSON lines during the clipboard import; that check was
not a pass. Output hashes were verified without normalizing captured evidence.
No runtime changes or weaker acceptance criteria were used for either refresh.


## API-013 retained screens and visible transitions

ScreenManager now retains generated component instances, hooks, event routing and
focus per screen. Input uses the active screen and acknowledged painter bounds;
resize updates those bounds. Removed screens release retained effects once.
Transitions prepare both trees and paint composed layers. Their source remains
active until completion, and its preorder geometry is remapped without changing
its local event identities. The shared backend has one previous presented tree
for legacy patch reconciliation across switches. Immediate switches and removal
cancel obsolete transitions. Removing the final screen presents an empty frame.

Failure demonstrations: the Enter callback originally stayed at zero. The fade
midpoint initially painted full blue (59,130,246), not the expected blend
(149,99,157). Both now pass. Transition tests compare all nine animated kinds at
zero, quarter, three-quarter and completion; slide additionally checks independent
left/right colors. Tests cover source-only pointer/keyboard input, final activation,
wide/combining text, release/repeat hotkeys, cancellation and submillisecond timing.
Flip and cube are explicitly documented cell-placement approximations.

Fresh development checks on the current source:
- Full library: 992 passed, two intentionally ignored fixtures.
- Eight screen-manager unit tests passed (also included in the full library).
- api_animation_screens: 16 passed, one known relative-opacity regression failed.
- api_hook_lifecycle: seven passed.
- api_component_expansion: five passed; api_focus: 12 passed;
  screen_system_tests: nine passed.
- Strict cargo clippy --lib --tests -- -D warnings passed.
- cargo fmt --all and git diff --check passed.

The mechanism includes the screen-manager tests and keeps the relative regression
visible. A target painted with opacity 0.5 currently yields numeric zero from
animate(target_id, Relative("+0.25")); the required values are opacity 0.5 to 0.75.
Inspection confirms animate discards target IDs and neither Animation nor its
manager retains a reference to the owning App/screen or its computed properties.
No relative-value runtime repair or public compatibility change was made here.
This is verified screen work, not complete API-013 acceptance or a final release.
Formal evidence and native records must be refreshed after implementation resumes.

Self-audit: public constructors remain intact; Screen already had private fields.
App event/focus changes only widen crate-internal visibility, with regression tests.
Presentation publishes event geometry only after backend success. Screen effects
are retained on switches and closed on removal. Source and destination clocks do
not share component state. The callback tests and pixel-independent cell colors
attack behavior rather than merely checking the composition implementation.
Remaining relative semantics prevent declaring the requirement complete.

Ripwire edit-check reports the ScreenManager contract unchanged and no incompatible
callers (its zero caller count is only a lower bound). Test-gate exits 4, listing
36 test paths and 25 untested symbols, including executed tests it did not model.
Quality-delta exits 2 with 1585 broad gating findings, dominated by reference
checkouts. Examined changed-path findings: exercised tests/Drop/trait methods marked
dead, preserved legacy patch API no longer used by ScreenManager, simple writer
fixtures, normalized constructors/cleanup incorrectly equated with unrelated
reference methods, and short-horizon visibility churn. These are not passing gates;
no unrelated refactors or deleted compatibility APIs were used to silence them.


## API-013 target approval and screen-change evidence refresh

The developer approved api-013-2. Approval is committed at 11c723a. Cairn first
required fresh inherited checks. API-001 through API-007 now have passing receipts.
API-008 behavior and process tests passed; its platform verifier failed on stale
Wayland evidence. Native run 34598114032 verifies snapshot
34111591c45d2077c6123b93463ca7ac512600da. The union of declared clipboard, ConPTY
and widget inputs was compared by Git tree entries with this committed source
before the verification branch was advanced. Target-handle implementation waits
for Cairn to return that action; the approval will not be requested again.

All five clipboard backends passed on the screen-repair candidate. Imported
macOS and Windows records only after matching snapshot, current input digest,
platform, execution markers and both output hashes. Native GitHub run 34598114032
completed successfully on both platforms. The clipboard verifier passes.

API-008 through API-010 now have fresh passing receipts. API-011 passed local
widget and Orca behavior, then rejected stale ConPTY evidence. Imported the new
ConPTY and Windows/macOS widget records from run 34598114032 after checking
snapshot, current input digest and every recorded output hash (6, 15 and 72
outputs respectively). Both native verifiers pass. Existing iTerm2 limitations
and their approved exception remain recorded; no acceptance criteria changed.

API-011 and API-012 now have passing receipts. The refreshed API-013 mechanism
passes keyframe and screen tests and fails the known relative-opacity case. Cairn
returned implement API-013. The api-013-2 approval remains binding.


## API-013 owner-bound targets and complete samples

Implemented the approved owner-bound target API. App and each screen publish
numeric properties only after successful presentation. Weak handles identify a
single target generation, retain no owner, and reject removed/replaced elements
and duplicate IDs. Relative operators read those properties at play, refresh at
the first sample after delay, retain endpoints across pause/resume, and recapture
on restart. Numeric samples paint through existing styles and wake their owner.
Percentage translation uses presented box dimensions and clears the percentage
term when applying its resolved cell value. Custom numeric values are declared
explicitly and inherited through generated component output.

Checked construction/play/seek return actionable errors. Legacy convenience calls
retain their return types and report the same errors by panic. The approved
migration requires handles for Single/Relative values; explicit ID endpoints and
the exhaustive serialized AnimationTargets enum remain available. Bound numeric
handles reject other property families explicitly. A working rustdoc demonstrates
lookup, playback, presentation and clearing overrides. Overrides persist after
completion until replaced or cleared; this behavior is documented.

Failure demonstrations and corrected behavior:
- The original opacity-0.5 relative +0.25 case now samples 0.625 halfway and
  presents grey 159, then grey 191 at the endpoint; clearing restores grey 128.
- Delayed playback originally retained construction-time 0.25. The deterministic
  test now captures the newly presented 0.5 and resolves its endpoint to 0.75.
- Generated component output originally replaced an authored custom count of 8
  with its internal 2. Metadata propagation now keeps 8 and samples 9.
- The Animation wrapper originally selected one arbitrary untyped property and
  discarded strings, units, booleans and alpha. It now preserves the complete map
  and nonopaque channel values without adding public exhaustive-enum variants.
- Numeric array [0, 1, 0] originally sampled zero at its midpoint. Arrays now keep
  evenly spaced interior frames; samples and actual screen painting reach one at
  the midpoint and one-half at three-quarter progress.

App tests exercise real SuprTUI output, failed presentation, independent owners,
cleanup, managed animation samples and percentage geometry. They run in bounded
child processes because App publishes an existing global performance-hook context;
sharing it with unrelated hook tests changed their defaults. The initial full
library run exposed two such failures. The isolated cases and full library pass.
The underlying global-context concern is captured in the API-019 backlog, not
silently changed as part of this animation work.

Reviewed ownership and locks: publication follows backend success, invalid handles
cannot bind a reused ID, samples wake only after releasing target locks, and
registry traversal uses stable owner-local identities. Built-in scale is applied
before explicit axis overrides. Screen event geometry and animation layout offsets
use the same preorder node count. No new dependencies or unsafe code were added.

Ripwire qualified Animation edit-check reports unchanged contract and zero observed
incompatible callers; its counts are lower bounds. The unqualified name was
ambiguous and was rerun with its file selector. Quality-delta exits 2 with 1594
broad gating findings, including ignored reference checkouts. Changed-path review
found added validation branches, explicit target-lock checks, exercised tests and
trait methods marked dead, small panic adapters, and normalized constructor
matches to unrelated types. The actual duplicated screen node counter was reused.
Test-gate exits 4, naming 99 tests and reporting 707 unmodelled symbols; it is not a
passing gate. Targeted integration suites and inherited Cairn mechanisms provide
behavioral verification, not a claim of complete static coverage.

Development verification: full library 997 passed, two existing ignored fixtures;
api_animation_screens 27 passed; component expansion five, hook lifecycle seven,
and painting properties 28 passed. The migration doctest passed (three existing
ignored examples). Strict library/test Clippy, formatting and git diff --check
passed. The final sort cleanup was checked again with Clippy and the 27 animation
integration tests. These are development checks; committed-tree Cairn evidence
is still required. This is not the final commitment-wide self-audit.

## API-013 target-change evidence refresh

API-001 through API-007 have new committed passing receipts. API-008 passed its
behavior and process tests, then rejected stale Wayland evidence. Native GitHub
run 34604250701 checks snapshot 8f1e2e3cf7957844b6f21c1b489f84dbae6dc57b.
Compared every Git tree entry in the union of clipboard, ConPTY and widget inputs
with committed candidate 4bd206d30e396b2a54673eaca067f0d4673cf347 before pushing
the existing verification branch. Linux platform records are being regenerated
on private test desktops; native records will be imported only after validation.

Wayland, xsel and xclip each passed five native round trips and two process checks;
all source digests and output hashes match. The macOS clipboard step also passed
and its record was imported after snapshot, digest, markers and hash validation.
The macOS job as a whole FAILED: HTTP App cancellation waited for REQUESTS 1 while
the last painted frame showed REMOVED. Fifteen other HTTP App cases passed, and
the iTerm host cases passed. No macOS widget pass record was produced or imported.
Failure artifacts remain in /tmp/reactive-tui-target-native-34604250701/widgets-macos
and the job log in /tmp/reactive-tui-target-native-macos-failure.log.

Windows completed successfully. Its clipboard record passed snapshot, input-digest,
platform, behavior-marker and output-hash validation and was imported. All five
clipboard backends now have current verified native evidence. GitHub run
34604250701 remains failed overall because of the macOS HTTP App test; no widget
acceptance claim is made from the clipboard refresh.


## API-011 cancellation fixture after native macOS failure

The API-011 refresh passed its local widget and accessibility cases, then rejected
stale ConPTY evidence. The completed native snapshot passed Windows but macOS
failed the HTTP engine cancellation case: its two-second response completed before
the input harness observed REQUESTS 1. Fifty local repetitions passed, so runner
scheduling is a plausible cause, not a proven production defect.

The revised fixture holds that response until teardown. Cancellation is still sent
only after a painted server-observed request. Its existing one-second bound now
measures close_dialog itself, excluding unrelated App startup and frame scheduling.
Result, no-submission, active-count and exactly-one-request assertions remain.
The server checks its stop flag and joins at teardown; no worker is left waiting.

A deliberate no-close variant failed at the bounded frame deadline while waiting
for REMOVED. Its output and the original macOS failure are retained alongside this
review. Restored the close call; all 16 HTTP App integration cases passed, as did
strict Clippy for the integration target, formatting and git diff --check. Native
verification remains outstanding, so this is not an API-011 completion claim.

Ripwire reports the fixture Server contract unchanged with lower-bound zero
incompatible callers. Quality-delta exits 2 (1583 broad gating findings); inspected
changed-path rows show the extra hold condition and timing measurement, historical
churn, and normalized constructors matched to unrelated reference code. The close
handler comparison crosses different test-root types and is not a reusable helper.
Test-gate exits 4 with eight test paths and zero untested symbols; its static result
is not a passing execution gate. The complete 16-case fixture suite ran.

After fixture commit dc8763d, the repeated API-011 check passed local widget and
accessibility behavior and again stopped at stale native ConPTY evidence. Native
run 34607088497 checks snapshot c3b8e49f94d142ea9ba15d2806ee025feadac64e, whose
complete union of declared native inputs matches committed candidate 7ee1e6e.
No candidate inputs were changed during the local or native run. Native record
validation and import remain outstanding.

macOS completed successfully in run 34607088497, including the revised HTTP
cancellation case. Imported its widget record and all 72 captured outputs only
after checking the snapshot, current input digest, successful cancellation marker
and each SHA-256 hash. The existing approved iTerm2 limitations remain intact.
Windows native verification is still running.

Windows completed successfully in native run 34607088497. Imported its six ConPTY
and 15 widget outputs only after matching snapshot, platform, pass result, current
input digest and every captured SHA-256 hash. Both native verifiers now pass.
The whole native run passed on macOS and Windows. The earlier macOS failure and
the no-close negative control remain recorded; no acceptance history was edited.

API-011 passed with current native records at receipt 20260911T141930550Z.
API-012 passed at 20260911T141947972Z and API-013 passed at 20260911T141957317Z.
Their evidence is committed. Cairn now names inherited ABI/regression freshness
before remaining API implementation; the overall commitment remains in progress.


## DFT-001 keyframe cleanup assertion under parallel runtime updates

ABI-001 through ABI-004 passed, including the combined 30 inherited contracts.
Standalone maintenance receipts passed too. The subsequent default-suite run
failed one library assertion: keyframe_hook_last_context_drop_cancels_escaped_handle
expected initial value zero but retained ten. A different test can update the
shared runtime between play and owner drop; delivering that live frame is valid.
The documented stop contract retains the last delivered value, not the initial one.

Made the premise deterministic by seeking to the midpoint before cleanup. The old
zero assertion failed with actual value five. The repaired test captures the value
after cleanup, attempts play and a seek to a different endpoint, and requires the
captured value and absent scheduled-task ID to remain unchanged. Choosing the
opposite endpoint prevents an already-completed value from masking a live seek.
No runtime code or synchronization was changed.

Five complete parallel library executions passed after the correction: each had
997 passes and two existing ignored fixtures. Strict library/test Clippy, formatting
and git diff --check passed. Ripwire edit-check reports the test contract unchanged.
Quality-delta exits 2 with 1578 broad findings; the changed test has historical churn,
while other matched hook rows are unchanged code normalized against reference
checkouts. Test-gate exits 4 (10 paths, 60 unmodelled symbols); no static gate pass is
claimed. The actual full library ran repeatedly under parallel scheduling.

The receipt runner initially staged only the requested requirement. Shared
mechanisms also produced MNT-004 and DFT-002 through DFT-004 receipts; those were
subsequently committed intact. Future runs stage all emitted evidence after each
completed check. The failed default-suite receipt and successful aggregate receipt
both remain in history.

## Native refresh after the keyframe cleanup test correction

API-001 through API-007 passed after 9f57a34. API-008 behavior and process tests
passed; its verifier rejected stale platform records. Native run 34610611752
checks snapshot f7da3e24ea58fb4418d37bfa3a81c032dc65bd7f, whose declared input
union exactly matches candidate 679747d. Wayland, xsel and xclip each passed five
native round trips and two process checks. macOS and Windows are still running.

Native run 34610611752 passed on macOS and Windows. Both clipboard records were
imported after snapshot, platform, current input digest, behavior markers and
output hashes were validated. All five clipboard backends now verify against the
committed candidate. ConPTY/widget artifacts are downloaded for their next action.

API-008 through API-010 passed after the clipboard refresh. API-011 passed local
widget and accessibility behavior and stopped at stale ConPTY evidence. Imported
six ConPTY, 15 Windows widget and 72 macOS widget outputs from run 34610611752
after checking snapshot, current input digests, platform/pass results and every
output hash. Both native verifiers now pass. Historical failures remain intact.

## Embedded terminal evidence refresh: intermittent resize probe failure

The first refreshed EMB check passed both worker/keyboard tests and all nine
integration cases. Its interactive example probe then timed out waiting for
SIZE_12_52 after the first resize; the captured screen stopped at the preceding
shell prompt. The failure receipt and captured output remain committed.
The unchanged standalone probe passed once and then in 30 consecutive executions,
including input, both resizes, interrupts, normal cleanup and worker-error cleanup.
No cause has been established and no code or acceptance criterion was changed.
Rerunning the full mechanism distinguishes a repeatable failure from this single
observed intermittent failure; a later pass does not explain the original timeout.

## API-014 mechanism construction

Mapped widget decoding, platform Image, retained App images, Surface/DiffWriter,
and SuprTUI graphics output to the existing tests and real-host capture driver.
The dedicated runner requires nonzero passing test counts for each of five
selectors (38 decoder/widget, 12 platform, 17 Surface, 10 graphics, 12 App), checks
native record freshness/hashes, and runs 25 Linux host/route cases. Source formats,
raw extents, alpha, crop/scale/offsets, placement ownership, clipping, retries,
removal and GIF clocks are exercised through the existing implementation.
The local URL decision and approved iTerm2 3.7 exception remain unchanged.

Examined the retained protocol and fallback baselines: the old Kitty serializer
claimed RGB for RGBA data, the inline serializer supplied an invalid byte size,
invalid extents succeeded, and unavailable graphics returned placeholder text.
Those historical failures are not acceptance. Current selected tests pass.
A fresh forced-ASCII host run reaches screenshot measurements with zero exact
fixture colors and fails the same color assertions used by real graphics cases.
The mechanism rejects startup errors and timeouts as negative-control evidence.

The initial development capture run passed Kitty/Ghostty App and Xterm/WezTerm
App modes (including full-screen output), then failed Chafa with an image-error
screen. Chafa was absent from this session's PATH. The existing isolated tools
remain at /tmp/rtui-image-tools/bin (Chafa 1.18.1 and Viu 1.6.1). Added executable
preflight and version output; restoring that PATH passes Chafa, Viu and GNOME
Auto captures. Both failed and corrected captures are retained under
 docs/analysis/api-image-hosts. No image runtime code changed.

Python syntax and git diff --check passed. Ripwire edit-check compares the new
private main against unrelated historical main symbols (no incompatible callers);
quality-delta exits 2 with 1576 broad reference-tree findings, not a passing gate.
Test-gate exits 0 with no modelled impacted tests; actual execution of this Python
mechanism remains necessary and is not inferred from that static result.

All 25 Linux host cases passed across the development runs after restoring the
external-tool PATH, including every GIF mode. The final runner uses bounded
process-group cleanup for the negative case too; that version rejected forced
ASCII and passed the remaining 19 cases. Formal acceptance follows commitment.

Captured command output retains its original trailing blank lines; the staged
diff whitespace check flags those output bytes. Source and authored documentation
pass the whitespace check. Initial development captures are a declared immutable
archive; fresh check outputs go under .cairn/reviews/api-image-hosts so the
mechanism cannot change its own declared inputs while running.

## API-014 formal acceptance

The first formal run received no receipt: a surviving GNOME portal helper wrote
an additional libpng diagnostic into a declared development log. That run also
failed standalone Kitty color capture. Its terminal.bin is byte-for-byte identical
to the earlier passing standalone Kitty output; the screenshot has the stage label
but no image. The cause remains unknown. Both outputs and screenshots are retained.
Identified the surviving portal/key-store processes by open capture-log descriptors
and stopped only their private fixture process groups before committing final logs.

The unchanged mechanism then passed formally: API-014/20260911T153649284Z.
All 89 selected tests, current native record verification, forced-ASCII rejection
and all 25 Linux host captures passed. No pixel assertion or image implementation
was changed to obtain this result. A passing rerun does not explain the intermittent
Kitty blank capture. The private GNOME helpers from this run were stopped before
committing captured logs too. The driver currently needs this explicit cleanup;
record that harness lifecycle issue for the final mechanism review.

## API-015 mechanism construction

The fresh no-default baseline fails E0433 at display/adaptive.rs importing Tokio;
its output is retained as api-015-no-default-baseline.log. The public async FPS
methods have no Tokio-runtime-dependent work beyond their private lock. Existing
futures-util is mandatory and can supply an executor-independent async mutex.
The new behavior check drives the public manager with futures-lite, compares
state/frame duration with its synchronous equivalent, and exercises color paths.
Both default-feature tests passed before implementation changes.

The mechanism checks the exact Cargo feature catalog, default/no-default, each
non-SIMD feature independently and their combined build. SIMD alone and all
features use nightly as already documented in Cargo.toml. Every row compiles all
targets and must execute nonzero passing behavior tests. Existing real App
component workflows also execute with and without defaults. This is the declared
matrix, not a claim that the broken no-default or nightly rows already pass.

## Native refresh after feature acceptance declaration

Adding tests/api_feature_configurations.rs invalidated the broad native widget
and ConPTY inputs. API-011 local App, library and Orca checks passed, then its
native verifier rejected stale ConPTY records (20260911T155247167Z). No feature
implementation has changed yet. Run 34618800536 checks snapshot
1d416fe039c72c97031be3ba62801b7b24fa2207; its complete declared input union was
compared byte-for-byte with committed candidate 4ae3cee before the existing
verification branch was advanced. Both native jobs are pending completion.

## API-011 cancellation marker above the backdrop

Run 34618800536 failed one macOS HTTP App case: the cancellation fixture waited
for REQUESTS 1 while the fully opened modal covered that synchronization marker.
All other 15 HTTP App cases passed. The native diagnostic is retained in
api-011-cancel-marker-macos-failure.out. Cancelled the still-running obsolete
Windows job after determining the fixture must change; it is not passing evidence.

Disabling the opening animation makes the same hidden-marker failure deterministic
locally (api-011-cancel-marker-negative.log). Kept that fully open backdrop premise
and gave the marker the existing ObservedInput fixture's explicit 16x1 size and
maximum z-index. This is a test-only repair. It retains the actual server-request
signal before cancellation, pending response, one-second close bound, cancelled
completion, zero submissions, zero active dialogs and exactly one request.

All 16 HTTP App tests pass after the marker correction, as does strict all-target
Clippy. Formatting and source diff whitespace checks pass. Ripwire edit-check
reports the cancellation test contract unchanged. Its test gate exits 4 and
misattributes the local Root name to eight unrelated fixture files; that private
Root cannot be called from those files. The complete HTTP module was executed;
all-target Clippy compiled the other targets. Quality-delta exits 2 with 1579 broad
findings, not a passing static gate. Native acceptance remains required.

The corrected native run is 34620004744, snapshot
a1d3d4c2eeafebfbe682a3561f8447ab1798165e. Its entire native input union exactly
matches 694e97d54a48b1bce1c187616e2860bed4ff91c3. Obsolete run 34618800536 was
cancelled after retaining its macOS failure. Do not import it as current evidence.
The corrected macOS and Windows jobs are running. After completion, validate
artifact snapshot, input digest and all output hashes before importing ConPTY
and widget records. API-011 is the next Cairn check; API-015 implementation has
not begun. No .cairn/in-progress marker remains. Preserve PATH containing
/tmp/rtui-image-tools/bin for the subsequent API-014 host check and explicitly
set CARGO_TARGET_DIR to this repository's target directory for every Cargo/check.

Run 34620004744 attempt 1 passed macOS completely. Windows clipboard failed at
its approved 15-second bound while copying 13 bytes; the later always-running
ConPTY and widget steps passed independently. My progress update incorrectly
inferred clipboard success from the next step starting; that was corrected.
The timeout diagnostic is api-008-windows-15-second-timeout.out. Rerunning the
failed Windows job unchanged (attempt 2); do not count the timeout as a pass.

Validated snapshot, current input digest and every captured-output hash for
72 macOS widget outputs, 15 Windows widget outputs and six ConPTY outputs.
Imported those independently valid records without claiming the overall first
workflow attempt passed. The corrected cancellation test passed on both hosts.
No timeout bound or clipboard implementation changed.

API-014/20260911T164401876Z passed all 89 selected tests and 25 Linux host
captures against current native records. Stopped the identified private GNOME
helper groups before committing their captured logs. Windows retry attempt 2
has passed the clipboard step with the unchanged 15-second bound; its remaining
steps and final artifact upload are still running.

## API-015 implementation and baseline

The formal baseline API-015/20260911T164511636Z passed the default build and
behavior tests, then failed the no-default build at its unconditional Tokio
import. Replaced only the async FPS manager's private lock with the existing
futures-util Mutex. Public methods and optional feature names remain unchanged;
each method acquires the lock and performs synchronous work without another
await while holding its guard. Gated Tokio-only error and test imports. The
existing nightly SIMD feature now activates portable_simd at the crate root.
The full development matrix and committed acceptance are pending.

Native run 34620004744 attempt 2 completed successfully on Windows; both native
jobs now pass, including clipboard, ConPTY and widget workflows. The downloaded
Windows clipboard record names snapshot a1d3d4c2eeafebfbe682a3561f8447ab1798165e,
and both raw output hashes match. These are results for that earlier immutable
candidate; the API-015 source edits need new native evidence when required.
No timeout was widened and ConPTY is retained as the developer confirmed.

All twelve feature configurations passed their all-target builds and behavior
tests in the development matrix: 26 feature tests plus five real App component
cases with defaults and the same five without defaults. Nightly was
1.100.0-nightly (e71c0f1e3 2026-08-18). The Tokio-only test import was gated
during this development run; the committed matrix must verify the final tree.
Strict all-target Clippy passed both default and no-default configurations after
that edit. Formatting and authored-source whitespace checks passed. The feature
matrix retains existing FFI warnings and the newer nightly compiler's deprecation
and float-inference warnings; this is not a warning-free all-features claim.

Ripwire edit-check reports the public AsyncAdaptiveFpsManager contract unchanged.
Quality-delta exits 2 with 1576 major rows in the ignored reference trees and no
major rows outside them; it is not a passing gate. Test-gate exits 4: generic
lock-method names produce 265 impacted symbols, including unrelated reference
terminal implementations, and 212 untested edges. The seven identified test
paths include the directly executed feature suite and platform workflows; real
native evidence remains required independently. These static results do not
substitute for the feature matrix or inherited checks.

## Native refresh after API-015 implementation

API-001 through API-007 passed again against the feature repair, including
Rust/C signal ownership and Miri. API-008/20260911T170517937Z passed its local
behavior and process cleanup tests, then rejected stale Wayland platform evidence.
All three Linux clipboard backends have now passed their five real round trips
and two process lifecycle checks on dedicated private desktops.

Native workflow 34625784358 checks snapshot
7f521e0740c9dde279ac0bf8b3e8fc563822570f, whose complete 18-path native input
union exactly matches committed source f8271f6da398fd8cc5e07e64f6ca8430a86d4129.
The Windows/macOS jobs are running. Validate their snapshot, current input digest
and every captured output hash before import. Keep the implementation marker
for API-008 until the refresh is committed. The API-015 development matrix passed,
but its formal receipt remains pending the earlier evidence refreshes.

Run 34625784358 completed successfully on both native platforms. Validated
macOS and Windows clipboard record snapshots, current input digests, all output
hashes and executed round-trip/process markers before importing their raw bytes.
All five clipboard platform records now pass the verifier. The same snapshot's
widget and ConPTY artifacts are retained under
/tmp/api-015-implemented-native-34625784358 for the API-011 refresh.

## API-011 Orca image navigation synchronization

API-011/20260911T172917586Z passed the local widget tests and earlier screen-reader
workflows, then failed image speech at 100x32. The unchanged isolated workflow
passed on rerun. The failing reader log identifies the ordering defect: at
13:29:12.119752 Orca started next-sibling navigation from the old Report annotation
context; it processed focus moving to Change sample at 13:29:12.131996 and spoke
that button at 13:29:12.180032. The earlier navigation presented the enclosing
frame, so the required Black square sample/image speech never occurred. The App's
FOCUSED state alone does not prove the separate reader consumed the focus event.
Raw session and reader logs are retained as the negative demonstration.

Added the existing speech-acknowledgement pattern used by the chart and scroll
cases: capture the speech position before changing catalogs and require Orca to
announce Change sample before next-object navigation. The same image role, label,
hidden-fallback, changed-prop speech, real terminal input/output and cleanup
assertions remain. No timeout or production source changed. Four corrected
workflows passed: three at 100x32 and one at 60x16.

Ripwire identifies the actual orca.py inside caller and reports its signature
unchanged with no incompatible callers. Test-gate exits 4 and names orca.py, which
executed the four corrected workflows; this is not a passing static gate.
Quality-delta exits 2 with 1576 major reference-tree findings and none outside
reference. Authored-source whitespace checks passed. The native widget and ConPTY
input declarations include all tests, so the earlier successful 34625784358
artifacts must be refreshed for this fixture edit. Clipboard inputs are unchanged.

The full corrected local API-011 check passed all widget, library, transport,
negative-control and screen-reader workflows, including image speech at both
viewports and the following dialog cases. Receipt 20260911T173928786Z fails only
because the stored ConPTY evidence is stale. Native run 34628467736 checks
snapshot cc679c577916c70979f7365ea050f505d25b4d5a; its full input union was
compared with b67238ef25e8770d630917979b8a59400aa77a32 before push. macOS has
passed; Windows is still running. Do not reuse 34625784358 widget/ConPTY records
for the changed Orca fixture. Clipboard inputs did not change. A development
probe created one orca_display bytecode file; it was removed before this check,
and the existing mechanism disables child-process bytecode generation.

Run 34628467736 passed completely on Windows and macOS. Imported the current
widget and ConPTY records only after matching snapshot, source digest and every
output hash (72 macOS, 15 Windows widget outputs, six ConPTY outputs). Both
native verifiers pass against the corrected Orca fixture. Raw Windows bytes
were preserved. No production code, timeout or acceptance assertion changed.

API-014/20260911T180352813Z passed all 89 selected tests, native evidence
verification, forced-ASCII rejection and all 25 Linux host captures. Stopped
the two private GNOME helper process groups identified by descriptors pointing
into this run's capture logs before committing those logs. No image code or
pixel assertion changed. API-015 is now the next formal check.

API-015/20260911T180926733Z passed all twelve feature builds and behavior rows
plus the default and no-default App component workflows. The optional Tokio
compatibility decision is realized by fcd24aa. Five API requirements remain:
API-016 through API-020. Cairn now requires inherited evidence refreshes before
it advances to those implementations.

## API-016 entry-point mechanism construction

Added separately compiled public Rust and C consumers and an entry-point inventory.
The legacy consumer rejects the eleven fabricated initial insertions and repeated
insertions on an unchanged frame. Its patch, presentation, root-update and shutdown
error cases pass. The PTY control through SuprTUI passes at 32x8 and 48x12: measured
button input, updates, reordering, removal, resize, ordinary/error exit and terminal
restoration. The retained Crossterm, alternate Crossterm output mode and DirectTty
App routes paint blank screens. Source inspection identifies another cause: the
resolved tree converter removes children from the Element read by legacy backends.
The C consumer rejects the missing SuprTUI builder declaration. These are negative
baselines, not API-016 acceptance.

The first harness run sampled partial output and used vt100 to compose a ZWJ emoji
that this parser splits into extra cells. It also used a C callback name conflicting
with the shipped render export. Corrected those fixture defects before the final
baseline: wait for all relevant frame state, inspect combining/wide cells plus the
intact emitted ZWJ sequence, and name the callback render_root. The inherited host
rendering tests retain their visual obligations. Both original and corrected logs
and PTY captures are retained. No production source has changed.

Consumers live under verification/api-entry-points and are compiled against the
public library and shipped headers; their files and the exact runner are declared
inputs. This is separate from the inherited native-widget test program, whose
source inputs did not change during mechanism construction.

Ripwire edit-check identifies the new initial-frame test with no callers. The test
gate exits 0 with no modeled impacted tests, so it does not prove these separately
compiled consumers ran. Quality-delta exits 2 with 1577 broad gated findings,
including unchanged reference and application paths; no production file was edited.
It is not a passing quality gate. Python syntax and Rust consumer compilation pass.
Formal acceptance and production repairs remain pending.


## API-016 implementation and development evidence

App now keeps the last acknowledged render tree and supplies the complete candidate
with real initial/update/removal patches. It advances that tree only after present
succeeds. The resolved-tree conversion retains Element children without constructing
another component instance. Crossterm and DirectTty share complete-frame painting;
the former retains its incremental/full-output option and debug overlay. DirectTty
retains native input, capabilities and hyperlinks, queues every event from a read,
and observes actual size changes. Unix clones share one descriptor/restoration owner.
DebugBackend stages the shared painter output and publishes graphemes and geometry
on present, allowing real component input without taking a host terminal.

The C builder exposes SuprTUI and retains its opaque allocation across setters.
A fresh negative re-selection case lost raw mode and exited on SIGINT (-2):
api-016-native-reselection-negative.log and .bin preserve that result. A private
builder owner now keeps repeated terminal selection idempotent; selecting Debug
releases the terminal before a later terminal selection acquires another session.
The corrected case passes, including resize, Ctrl-C and exact termios restoration.
A separate early-resize failure showed input subscription could miss SIGWINCH;
SuprTUI also compares actual terminal dimensions before waiting for input.

The expanded development mechanism passed five public Rust consumer tests, eight
Rust App PTY workflows, three manual backend/Unix workflows and four C workflows.
It checks unchanged output, incremental/full equivalence, debug overlay removal,
complete input batches, cloned owner survival and successive sessions. Results:
api-016-expanded-development.log; raw captures api-entry-points/20260911T224109Z.
The earlier adapter-only development log/captures are retained separately. These
are editing checks; committed Cairn acceptance and dependent/native refreshes remain.

## API-019 platform concerns recorded before further implementation

These entries extend the residual inventory; they are not acceptance claims or
permission to remove retained features. API-019 must reconcile them with the full
coverage table and its other named gesture, updater, theme, Markdown and editor work.

| Concern | Contract | Falsifier / required attack |
| --- | --- | --- |
| Unix input thread ownership | A reader cannot access a closed/reused descriptor; output and retained work are bounded; dropping its consumer/session releases work. | Drop the receiver/session while idle and under input load; reused descriptors must remain untouched and no owned reader may remain. Current reader copies a raw fd and uses an unbounded queue. |
| Native signal callbacks | Window-size callbacks cannot deadlock in a signal handler or use released callback state; session cleanup preserves another owner's signal policy. | Deliver SIGWINCH during registration/cleanup and across successive owners. Current Unix handler locks a mutex and replaces the handler without retaining the old action. |
| Legacy pointer/key translation | Actual button, modifiers, wheel direction and key kind reach the documented public route; split input is retained until parseable. | Right/middle/modified clicks, each wheel direction, press/repeat/release and split escape/UTF-8 sequences produce the expected event once. Current DirectTty button inference uses modifiers; advanced keys fall back to Unknown; Crossterm wheel conversion needs scrutiny. |
| Debug dimensions and mixed rendering routes | Accepted dimensions remain consistent with allocated cells; invalid sizes fail without excessive allocation; switching full/cell/patch routes cannot expose stale state. | Zero/oversized dimensions and resize beyond u16; stage/fail/present and route-switch cases. Existing Debug resize casts usize dimensions after allocating. |
| Multiple terminal owners | Overlapping construction and destruction cannot restore modes or close transport still required by a live owner. | Create/drop independent Rust terminal owners in both orders. API-016 proves C re-selection and Unix clones only; independent constructors need their own evidence. |
| Public render-tree candidates | Retained root types either render their children correctly or return a documented error; conversion remains bounded for accepted input. | Fragment/custom roots and deep/broad resolved trees; no silent missing frame or repeated component construction. Complete Element children currently duplicate descendant values in render nodes. |
| Legacy backend test reachability | Claimed key/focus/paste/resize tests execute in the test harness. | Verify registered test names and force a safe mismatch. Existing mapping tests are nested inside DebugBackend::render_full. |


API-016 regression verification on the implemented source: the full library passed
997 tests (two existing ignored fixtures). The selected renderer/component/hook/
focus/animation/screen integration suites passed 69 tests. Strict default Clippy
(--lib --tests -- -D warnings), cargo fmt --check, standalone consumer rustfmt and
authored-source whitespace checks passed. The independent C/Rust compiler and Koffi
ABI probe agreed on 211 exports and 13 record layouts. Six existing FFI compiler
warnings remain; this is not a warning-free FFI build claim. Individual logs retain
commands and outputs under api-016-*-development.log.

Header regeneration initially removed the hand-added signal ownership comment and
introduced unused opaque DialogId/Rect forward declarations. Retained the ownership
text in cbindgen configuration and excluded those unused internal types; generation
and --verify now pass with the guidance intact. This does not change any old ABI.

Ripwire edit-check finds App, CrosstermBackend, DirectTtyBackend and DebugBackend
contracts unchanged. The qualified UnixTty report counts an additional internal
same-name construction site as a definition change; public Clone and constructor
signatures remain unchanged and the external clone workflow passed. The C SuprTUI
selector is reported as new. The initial quality delta exited 2 (1616 gated rows),
including reference checkouts, retained public/trait methods reported dead, generic
name-based duplication and short-horizon churn. The changed output-worker complexity
increase is the bounded optional overlay and frame-output option; the 30 added lines
remain in the owner that orders painting, graphics and acknowledgment. Extracting
unrelated lifecycle methods to satisfy name-normalized clone rows would add coupling.
The header forward-declaration findings were repaired. The final bounded report is
api-016-static-summary.json. Test-gate exits 4 with 84 test paths and 240 unmodelled
or untested impacted symbols; it does not establish test success. Direct consumer
and library checks above passed; the remaining inherited and native mechanisms still
have to run against the committed tree. No static gate is being claimed as passing.


## Native refresh after API-016 entry-point implementation

API-001 through API-007 have new passing receipts after source 6f879a2.
API-008/20260911T224832622Z passed local behavior/process checks, then rejected
stale platform records. Wayland, xsel and xclip have now each passed five native
round trips and two process-lifecycle tests on private desktops. Their committed
input digest, executed markers and both raw-output hashes were verified.

Native run 34655618855 uses snapshot f6effcb23d9984f0056da24af6970c3115a4c251.
Its entire 18-path native input union exactly matches committed source
03673543ded30b3942550f6a395bc0a42236dcca. Only the existing verification branch
was advanced. macOS and Windows are running; their records remain pending.
Keep the API-008 implementation marker until the refresh has been verified and
committed. This is not a new clipboard code change.

Native run 34655618855 completed successfully on macOS. Imported its clipboard
record after validating snapshot identity, current source digest, native system,
executed markers and both raw-output hashes. Its widget record and 72 captured
outputs were also validated and remain in /tmp/api-016-native-34655618855 for
the API-011 refresh. Windows remains pending.

Native run 34655618855 completed successfully on Windows too, including clipboard,
ConPTY and widget steps. Imported Windows clipboard evidence after validating the
snapshot, current input digest, system, executed markers and both raw-output hashes.
All five clipboard backends now pass the verifier. Validated the Windows widget
record and 15 output hashes, plus the ConPTY record and six output hashes; they
remain in /tmp/api-016-native-34655618855 for API-011. Captures retain their original
bytes and line endings. No native criterion, timeout or implementation was changed.


## API-011 native import after entry-point recovery

API-011/20260911T231442714Z passed the complete local widget and Orca workflows,
then rejected the old ConPTY record. Imported both native widget records and the
ConPTY record from successful run 34655618855 after rechecking snapshot identity,
current declared-input digest, platform/result and every captured-output hash.
There are 72 macOS widget outputs, 15 Windows widget outputs and six ConPTY outputs.
The native verifiers now pass, including required behavior/failure markers and
image-host cases. Raw outputs retain their original bytes. Formal API-011 rerun
remains required; no widget, accessibility, terminal or image code changed.

API-011/20260911T232012997Z, API-012/20260911T232030147Z and
API-013/20260911T232036106Z passed against the current native records.
API-014/20260911T232713530Z passed all 89 selected tests, the forced-ASCII
negative control and all 25 Linux host workflows. Captures are retained in
api-image-hosts/20260911T232052898840Z. After the run, inspected and stopped
only the four leftover private GNOME portal/key-store helpers in groups
1823591 and 1838842; their capture-log descriptors are closed. The cleanup
record is api-016-image-helper-cleanup.json. This successful run does not
explain the previously recorded intermittent Kitty blank capture or remove
the documented need for explicit fixture-helper cleanup before commitment.


## API-016 formal acceptance

API-015/20260911T233414171Z passed all twelve feature configurations, 26
feature behavior tests and ten App component workflows. API-016 then passed
formally as 20260911T233538835Z: five public Rust consumer tests, eight Rust
App PTY workflows, three manual backend/Unix workflows and four C workflows.
Captures are in api-entry-points/20260911T233533Z. This acceptance follows the
committed failing baseline and retained native re-selection failure demonstration.
API-001 through API-016 now have current passing receipts. Four numbered API
requirements remain; inherited ABI and regression evidence still needs refreshing
before Cairn advances to the remaining implementation.

## API-017 acceptance declaration

All 34 inherited requirements now have current passing receipts after API-016.
The final renderer receipts are 20260911T234522876Z/877Z.

Declared compiled C and TypeScript consumers for editor Unicode editing and
styled output, real layout spacing, six native dialog families and result data,
foreign prop/state updates, independent controllers, wrong-thread rejection,
callback reentry, callback failures, explicit cleanup and real App restoration.
The first run is a baseline; editor/layout/dialog modules remain excluded and
the foreign controller is absent. The controller architecture was recorded
before implementation. A Python syntax parse passed; consumer compilation and
behavior are intentionally not yet claimed to pass.

## API-017 implementation and development verification

Implemented native editor, layout-style and dialog controllers using the recovered
Rust implementations, with a retained foreign component controller and TypeScript
ownership wrappers. Added compatible fallible Component rendering and a C owned-root
entry point so callback failures leave App through its normal cleanup path. The
existing infallible Rust methods and compiled C ABI remain available. The generated
header, Koffi metadata, exports, migration inventories and controller contract agree.

Creating-thread checks precede controller access. Foreign callbacks receive owned
JSON snapshots without a held prop/state lock; reads and updates can reenter, while
recursive callbacks and disposal are rejected. App retains its existing instance
serialization mutex. Clarified that distinction in the decision: it is not a promise
that all native locks disappear during callbacks. Errors wake the App through a
separate failure signal and preserve a native cause through last_error. Disposed
controllers cannot call released userdata even when old Elements remain. TypeScript
retains registered callbacks until successful native destruction and preserves the
original callback exception after cleanup, including an exception in disposal.

Development acceptance passed as api-017-final-development.log, with captures in
api-native-components/20260912T003627Z. Independent C/Rust compilers and Koffi agree
on 243 signatures and 13 layouts. Strict C and real TypeScript consumers passed
Unicode editing/selection, invalid arguments, independent state, state/prop updates,
wrong-thread calls, recursive calls, returned allocations on failure, missing output
on success, callback failures, explicit retry and cleanup. The retained-Element test
clears the prior callback error first, so it specifically tests disposal protection.
TypeScript also rejects nonfinite JSON and tests a throwing dispose callback.

Both languages passed real App PTYs for layout, Unicode editor paint, foreign state
and props, modal input and result data, restored focus, resize and terminal cleanup.
Both exercised confirmation, timed toast, updated progress cancellation, autocomplete
selection and a two-step wizard through their native controls. Render and event error
PTYs exited and restored the terminal. A separately compiled violating callback
acknowledged input without updating state: the frame assertion rejected it, then
shutdown restored the terminal. This demonstrates that a successful native return
code alone cannot pass the mechanism.

The initial direct baseline failed missing native types/functions and TypeScript
exports. During development independent ABI checking rejected an opaque Option
callback declaration emitted by cbindgen; nullable callback aliases fixed it. Later
fixture failures exposed a C callback name collision with an old export and an
execv launcher requiring an absolute env path. Dialog probes showed actual native
toast dismissal is Confirmed(None), and progress is displayed as 75.0%; corrected
those expectations without changing native behavior. Failed and corrected logs and
captures are retained. None of these development runs is a formal Cairn receipt.

Focused Rust regression tests passed: component expansion 5, editor Unicode 23,
event routing 9, focus 12 and hooks 7 (56 total). Strict default Clippy passed;
TypeScript ESLint passed with zero warnings; cargo fmt --all --check, generated
header --verify and git diff --check passed. The ffi build retains six existing
warnings, not a claim of strict FFI lint success.

Reviewed controller ownership, JSON bounds, callback error propagation, disposal,
Element consumption, documented editor snapshots and blocking Node App behavior.
Native custom dialog callbacks and custom Element wizard panels remain Rust APIs;
this binding provides the explicitly described native options and text panels.
Ripwire edit-check passed. Quality-delta exited 2 and test-gate 4; bounded findings
and specific assessments are in api-017-static-summary.json. Header findings match
previously excluded wrappers; trait/C callbacks are indirect; the reported
ComponentBuilder.key complexity of 88 is a name-attribution error (two statements,
no branches). Small typed ABI wrappers and opposite-direction enum conversions are
kept explicit. Static reports are not represented as passing acceptance evidence.

API-017 remains in progress until its committed formal acceptance and refreshed
inherited requirements pass. API-018 through API-020 remain pending.

The staged whitespace check flags raw terminal CRLF and captured log formatting.
Those evidence bytes are preserved. The source/document check excluding raw .bin
and .log captures passed; no historical or captured output was normalized.

## Native refresh after API-017 implementation

API-001 through API-007 have passing receipts after 0ce24af. API-008
passed its 13 behavior and two process-lifecycle tests, then rejected stale
platform records. Native snapshot cf05cbc7fcd4e8baf408e9079be8172b85d09eae has the exact
18-path input union from committed source ec2496ad7499f2cc2d458e49fdbc6405c1833d0c.
Only the authorized verification branch was advanced. Native results are pending.

Native run 34662535829 is executing the new snapshot on macOS and Windows.
Wayland, xsel and xclip each passed five native clipboard round trips and two
process-lifecycle checks on private desktops. Validated their committed input
digest, platform, executed markers and both raw-output hashes. Native results
remain pending; the API-008 action marker stays active.

Native run 34662535829 completed successfully for macos. Imported its clipboard
record after verifying the snapshot, current committed input digest, platform,
executed behavior markers and both raw-output hashes. Original bytes are preserved.

Native run 34662535829 completed successfully for windows. Imported its clipboard
record after verifying the snapshot, current committed input digest, platform,
executed behavior markers and both raw-output hashes. Original bytes are preserved.

## API-011 native import after foreign component recovery

API-011/20260912T010529017Z passed local widget and Orca workflows, then
rejected stale ConPTY evidence. Imported successful native run 34662535829:
72 macOS widget outputs, 15 Windows widget outputs and six ConPTY outputs.
Rechecked the exact snapshot, current input digest, platform/result and every
captured-output hash before copying. Both native verifiers now pass, including
ConPTY behavior, runtime integrity and the deliberately disabled resize case.
Native captures retain their original bytes and line endings. No behavior,
criterion, timeout or mechanism was changed. Formal API-011 rerun is required.

API-011/20260912T114853541Z, API-012/20260912T114910947Z and
API-013/20260912T114915729Z passed against the refreshed native records.
API-014/20260912T115555745Z passed its selected native/image tests, the
forced-ASCII negative control and all 25 Linux image-host workflows. Captures
are retained in api-image-hosts/20260912T114934145409Z. Inspected all remaining
members of private groups 3309037 and 3329299, then stopped only their four
GNOME portal/key-store helpers. No capture descriptors remain open; see
api-017-image-helper-cleanup.json. This pass does not explain the previously
recorded intermittent Kitty blank capture; helper cleanup and startup timing
remain explicit concerns for the final mechanism review.

API-015/20260912T120355662Z passed all twelve feature configurations, 26
feature behavior tests and ten App component tests. Also retained the 52 raw
image host/Xvfb logs excluded by the default ignore rules in the preceding
capture commit; their original bytes were preserved after helper cleanup.

## API-017 formal acceptance

.cairn/evidence/API-017/20260912T120716719Z passed. Both compiled consumers exercised native editor/layout,
all six dialog families, foreign state and props, recursive/wrong-thread rejection,
render and event callback errors, ownership and App restoration. The deliberately
broken input callback failed its frame assertion and restored the terminal.
Independent compilers and Koffi agree on 243 signatures and 13 layouts. Captures
are retained in api-native-components/20260912T120642Z. API-001 through API-017
now have current passing receipts. Three numbered API requirements remain;
inherited ABI and regression evidence is being refreshed before Cairn advances.


### ABI-004 regression repair: toast deadline before presentation (2026-09-12)

The formal ABI-004 receipt `20260912T121509058Z` failed after the default-suite
command exceeded its unchanged 300-second deadline while compiling. Its earlier
formatting, strict lint and FFI steps passed. A later cached `cargo test --no-run`
replay completed in 1.317 seconds without changing sccache settings. This does not
establish the cause of the compiler stall or prove it repaired. The failed receipt
and captured output remain intact. The shared sccache server was not restarted.

An unchanged developmental default-suite run then reached the tests and failed
`engine_toast_expiration_delivers_completion_without_input`: the 60 ms toast
completed without appearing in any captured frame. The first hypothesis, that
only the default 200 ms fade caused the failure, was incomplete. With animation
disabled and otherwise identical root structure, a zero-delay sibling passed; a
100 ms constructor delay reproduced the missing toast. Source inspection shows
that LiveToast scheduled its deadline during construction, while Modal first
requires root and body measurement frames. App processes ready timers before
those later frames. The delay consumed the entire lifetime before presentation.

The recorded Judged decision starts expiration after the fully opened, measured
modal body is presented. A private callback follows the existing App layout
publication path, which runs after successful backend presentation. The toast
retains one deadline and prevents restart after unmount, repeated presentation or
close. Duration updates still replace an active deadline; None remains persistent.
The public duration documentation states the App timing boundary. Public Rust
function signatures and C/TypeScript ABI layouts are unchanged.

The strengthened captured-frame regression first failed on the old implementation
with a blank frame. The corrected code passed all four cases: zero and 60 ms
durations, each with no animation and the normal fade, while first-frame expansion
is delayed 100 ms. No input causes completion. Three focused lifetime tests pass
for retained output, unmount before presentation, manual close, duration updates,
persistent duration and repeated presentation. The complete default suite passed
1,858 tests with zero failures and 37 pre-existing ignored tests across 71 result
summaries. Strict default-feature all-target Clippy and workspace formatting also
ran and passed. These are developmental checks; committed Cairn evidence still
needs refreshing. Raw results are retained in `.cairn/reviews/abi-004-toast/`.

Ripwire edit-check exited 0 and found the one changed private modal signature
with no incompatible caller. Quality-delta exited 2 and test-gate exited 4;
neither is called a pass. The bounded static report records all findings in the
changed files. Manual review covered the larger existing modal render method,
callback pointer equality, timer ownership and lock order, and retained callback
cleanup. The small cancellation/lifecycle similarities are component-local
resource handling, not a reusable generic timer owner: the terminal blink owner
has different state and scheduling semantics. New tests and Lifetime are reached
through Rust test discovery and component/callback dispatch, despite static
dead-code reports. Churn and line-count findings are retained; no baseline or
acceptance rule was weakened to suppress them. The full default suite covers the
changed private call sites and all six dialog families.

The developer reiterated the i9-13900K concurrency limit. Cargo jobs, Rust test
threads, nextest threads and linker threads remain at 8 in the existing machine
Cargo configuration; no limit was raised.

API-001 through API-007 have fresh passing receipts after the toast repair
(`20260912T124648564Z` through `20260912T124759800Z`). API-008 receipt
`20260912T124812707Z` passed its 13 local behavior tests and two child-lifecycle
tests, then rejected the old platform input digest. All three Linux desktop
backends have now passed five real round trips and two child-lifecycle checks
each against the committed repaired source. The runner preserved Cargo/test/link
limits at 8 and explicitly capped Mesa software rendering and Rayon pools at 8.

Authorized native verification run `34694702900` is testing snapshot
`90343e040e2e56a0c15e2f9e128fc9f3f4d74e2d`, whose declared platform input trees
match source commit `ccd9ef4e8809d2376876a9796d3e90c1d6560b6f` exactly. Only
`codex/clipboard-platform-verification` was pushed. Snapshot provenance is retained
in `abi-004-toast/native-snapshot.json`. macOS and Windows results remain pending;
they are not inferred from Linux success.

Native run 34694702900 completed successfully for macos. Imported its clipboard
record after verifying the snapshot, current committed input digest, platform,
executed behavior markers and both raw-output hashes. Original bytes are preserved.

Native run 34694702900 completed successfully for windows. Imported its clipboard
record after verifying the snapshot, current committed input digest, platform,
executed behavior markers and both raw-output hashes. Original bytes are preserved.

Native run `34694702900` completed successfully on both macOS and Windows.
The final clipboard verifier passed for all five backends after validating
current committed source digests, platform identities, executed behavior markers
and retained output hashes. The API-008 platform refresh action is complete;
Cairn must still record its acceptance result.

API-011 receipt `20260912T131336517Z` passed its local widget, library, HTTPS
and real Orca behavior checks, including toast speech/expiry at both dialog
viewports. It then rejected the older ConPTY input digest. Imported 96 native
artifact files from successful run `34694702900`: 72 macOS outputs, 15 Windows
widget outputs, six ConPTY outputs and their three records. Validation checked
actual job and step success, the exact verification snapshot, current committed
input digests, platform identity, every recorded raw-output hash, required image
artifacts and ConPTY success/violating/runtime-integrity markers. Original bytes
were copied unchanged. Both native platform verifiers passed after import. The
existing approved iTerm2 color/transparency limitation remains unchanged.
Validation counts are retained in `abi-004-toast/native-artifact-validation.json`.

API-011, API-012 and API-013 have current passing receipts after the toast
repair: `20260912T131915182Z`, `20260912T132006826Z`, and
`20260912T132011946Z`. Additional source inspection confirmed that Scheduler
does not attach timeout ownership implicitly to the active component scope;
LiveToast retains and cancels its timer handle, including after presentation.

API-014 receipt `20260912T132336815Z` failed on direct Kitty after nine host
routes passed and the forced-ASCII negative control was rejected correctly.
All captured bytes are retained under
`api-image-hosts/20260912T132044026958Z`. The failing stage-0 screenshot shows
the fixture label but no red/blue image pixels. Stage 1 contains the expected
green/yellow image and stage 2 removes it. This is an observed first-image
failure; whole-window startup delay is not established as its cause. The
previous intermittent image concern remains unresolved and requires diagnosis.
No acceptance threshold, timeout, or thread limit was changed.

After check execution stopped, two private GNOME helpers still held this run's
host log. Their complete process group and start identities were inspected,
then only that group was terminated. No process retains a capture file. The
inspection and cleanup result are in `abi-004-toast/image-helper-cleanup.json`.


### API-014 Kitty host ordering defect (2026-09-12)

The image investigation is retained in `api-014-kitty-layer-order/`. The
original Rust sender emits identical complete bytes in passing and failing
captures. Seventy parser partition cases preserve the expected decoded pixels
and placement. Additional writer, terminal-mode and tracing controls mostly
pass; those passing reruns do not repair the failure.

An independent C sender with independently generated compressed RGBA data
reproduces a missing image after clearing the prior placement in three runs.
Three additional controls leave it missing after another second of waiting,
then recover all 4,096 green and 4,096 yellow pixels after an X11 repaint,
without another image transmission. Both missing and recovered screenshots
are retained. Diagnostic command-syntax failures are retained separately and
are not counted as image failures.

The installed Kitty 0.45.0 library deterministically returns a stale false
image-layer predicate before preparing its placement counts, despite a loaded
image and a valid placement. Preparing first returns true. Its normal-window
render preparation queries that predicate before updating those counts and
uses the stale result to select a paint path without images. This connects
the native-library defect to the observed repaint recovery. The focused
self-contained diagnostic ran successfully and retains its output; this is
failure demonstration and ordering verification, not API-014 acceptance.

The proposed repair belongs to Kitty: prepare graphics counts before selecting
the image-layer render path, then prove the full host with unchanged pixel
checks and the independent sender. No host repair has been built. Changing the
host's source or narrowing the mandatory host behavior needs a scope decision,
as with the earlier iTerm2 host defect. API-014 remains failed. API-018,
API-019, API-020 and the remaining inherited refresh work remain unfinished.

Self-audit of this investigation: no production source, mechanism, threshold,
timeout, terminal installation or historical receipt was changed. The native
result and independent screenshot controls ran; none is relabeled acceptance.
All diagnostic outcomes and captures are retained, with original bytes and
provenance. Thread caps remain 8 and host runs were serial. The investigation
action ends with the failing requirement preserved for the developer's scope
decision; it does not claim the commitment or host repair complete.


### Approved Kitty host repair and updated machine limits (2026-09-12)

The developer answered `ok` to api-014-api-020, authorizing the narrow Kitty
host repair and unchanged image acceptance. The specification and commitment
now name this scope. A Judged implementation record precedes the host patch.
The existing image APIs, protocol requirements and iTerm2 exception remain.

The developer corrected the current thread limit to 12. The machine Cargo
configuration already sets jobs, Rust test threads, nextest threads and linker
threads to 12. New host builds and graphics pools use that limit. The observed
machine has 71 GiB total swap (34 GiB free) and a 128 GiB /tmp (115 GiB free).
Earlier evidence accurately retains its then-used limit of 8 and is not edited.

The isolated Kitty 0.45.0 build now succeeds. Comparing every archive source
file finds exactly one changed file, `kitty/child-monitor.c`: the image-layer
query moved after placement preparation. The independent C sender reproduced
the missing image in all three stock-host trials. The repaired host passed all
three trials with 4,096 pixels of each expected color and complete removal.
These development captures are in `.cairn/reviews/api-014-kitty-repair`;
`final-build` records the final builder and capture-driver versions separately.
They are failure demonstrations, not formal Cairn receipts.

The builder records runtime hashes and its own source digest, rejects corrupted
archives and changed runtime files, and checks that the selected Python really
honors the 12-worker override. A controlled runtime edit was rejected; restoring
the exact bytes passed. A requested limit of 13 was rejected. Compiler warnings
from two unchanged upstream declarations required targeted warning-as-error
exceptions, retained in the build record and explained in the build README.

The image driver now owns private process groups for its hosts and X servers.
It stops descendants even after a launcher exits. A controlled exited launcher
with a live sleep child demonstrated the leak shape; cleanup stopped that child.
Pixel assertions and capture intervals are unchanged. The optional fixture
argument preserves every existing caller and enables the independent C sender.

Python syntax, strict C compilation, diff whitespace and four focused Ripwire
edit checks passed. Ripwire quality-delta exited 2, with broad reference-tree
findings plus local duplication/churn/size observations; this is not a clean
gate. Its local duplication is the small process-group termination sequence
also used by the standalone Orca harness. Keeping that sequence local avoids
coupling image capture to the 666-line reader harness. The POSIX feature macro
is required by C headers, not dead code. The builder's cohesive orchestration
and this script's recorded churn were examined. Ripwire test-gate exited 4 and
named the Orca script through name-based cleanup edges; actual process ownership
is covered by the controlled descendant test and real host captures. Complete
reports are retained with this development evidence.

API-001 through API-010 have fresh passing receipts after the approved Kitty
scope change. API-011 is preparing native inputs: verification-branch run
34700619224 uses snapshot 2a68a4fa53382910a1997d24d18bc6ba25e23a92,
with all declared native input trees equal to ee893b13f4be020d68d2918b74061d5ff935bd54.
macOS completed successfully; its 72 recorded output hashes and current input
digest were checked before import. Windows is still running. Resume this run
and import validated widget/ConPTY artifacts before cairn check API-011.

Native run 34700619224 passed on macOS and Windows for snapshot 2a68a4fa53382910a1997d24d18bc6ba25e23a92. Its declared input trees exactly match local commit ee893b13f4be020d68d2918b74061d5ff935bd54. Imported the widget and ConPTY artifacts after checking job and step success, current committed input digests, system, every recorded output hash, and unchanged copied bytes. Both native verifiers passed. The earlier clipboard records remain current because their declared inputs did not change.

API-011 passed with fresh native records (receipt 20260912T151908154Z).
During read-only review of the host build notes, the description of the upstream
qualifier warning was found inaccurate: expand_tilde temporarily writes a NUL
through the slash pointer for ~user/path, then restores the slash. Its callers
include configuration environment strings and PyUnicode_AsUTF8. The note called
this read-only. Correct that description as a separate documentation change;
these image captures do not audit that upstream path-expansion behavior. No
source or mechanism changed during this review or the API-011 check.

API-012 and API-013 passed after the native refresh. API-014 now has a
fresh passing receipt, 20260912T152756874Z: all 89 selected tests, current native
records, the independent sender, the forced-ASCII rejection and all 25 host
routes passed. The verified repaired Kitty runtime remained intact. Normal
completion left no capture descriptors open.

Open API-020 review finding: the new image driver isolates host and Xvfb
process groups for normal cleanup, but the outer runner kills only its own
process group on timeout. A controlled three-second outer timeout left the
isolated groups running; all observed groups were then explicitly stopped.
The retained control in .cairn/reviews/api-020-image-capture-timeout is a defect
demonstration, not an acceptance pass. Resolve timeout ownership and prove
normal and timeout cleanup before commitment closure. Do not deliver with this
finding open. Pixel assertions and image-rendering acceptance remain unchanged.

API-015 passed with receipt 20260912T153933584Z after rebuilding the declared
stable, minimal, optional-feature, embedded-terminal and nightly SIMD paths.
The feature catalog, independent builds and runtime behavior checks passed.

### API-018 Props contract review (2026-09-12)

All 51 implemented requirements now have current passing receipts. Cairn names
`declare API-018`. The API-011/API-018 approval only narrows screen-reader
support; no existing decision approves a narrower Props contract.

Read the RAPI-15 audit, API-018 contract, applicable decisions, the Props trait,
derive implementation, and existing Props tests. The bare validation flag has
no rule, generated validate unconditionally returns true, attribute parse errors
are discarded, and optional-field documentation promises a type rewrite that
a derive macro cannot perform. The Rust Reference confirms derive output is
appended alongside the input declaration.

Four retained compiler probes in `.cairn/reviews/api-018-props-contract` show:
all sampled bare-validation values accepted; a plain optional String rejected
with E0308; an explicit Option<String> accepting absence and a supplied value;
and a proposed named predicate compiling but being ignored, so its invalid-name
assertion fails. Commands, compiler/library/source hashes, raw logs, fixtures,
and a proposed contract are retained. These observations are not API-018
acceptance passes. No public macro or library source changed during this review.

The proposal keeps caller-invoked validate(), requires named field predicates
and explicit Option types, and rejects bare/malformed validation annotations.
This needs the explicit narrower-contract approval required by API-018 before
implementation or a conformance mechanism can adopt it. API-019 and API-020
remain pending, including the known capture-timeout cleanup finding.


### API-018 approved Props implementation (2026-09-12)

The developer answered api-018-api-020 with `ok`. Approval and the consequential
contract decision were committed before implementation. The specification now
records caller-invoked named validation and explicitly authored Option fields.

The new behavior test first failed: zero size satisfied a named positive rule
because the generated method ignored it. After repair, all 16 targeted Props
integration tests pass, including invalid names, sizes and present optional
values, successful cases, defaults, builders and absence. A counter verifies
that construction does not invoke a predicate. The existing bare validation
annotations now name positive-size and bounded-age rules and test invalid values.
Two macro unit tests pass; their tables exercise five accepted declarations and
13 rejected forms with diagnostic assertions. Four compiled Props rustdoc cases
pass, including bare annotation, wrong optional type and wrong predicate return
type rejections. These editing checks are not formal Cairn acceptance evidence.

The derive now propagates syn parse errors; rejects empty, duplicate, unknown and
malformed options; validates optional field shape; and invokes rules in field
order with shared references, stopping on false. Defaults and builder signatures
are retained. No Props trait method or App validation hook was added. The macro
usage example now supplies the trait bounds and a valid optional field. The
public trait documentation explains caller responsibility and migration.

Strict Clippy for the library, three Props integration targets and macro library/tests passed.
Formatting and git diff --check passed. Ripwire edit-check passed; quality-delta
(exit 2) and test-gate (exit 4) did not. Their full outputs and hashes are in
`.cairn/reviews/api-018-props-implementation`. Reviewed the changed-path findings:
parser complexity reflects explicit diagnostic branches for three options and
type checking; each rejection is exercised. Dead-code findings cover executed
macro tests, generated calls and test structs. Suggested duplicates equate short
parsing/predicate patterns with unrelated reference code. The test gate names
registry registration through a static name edge; API-002/registry acceptance
will be refreshed after this committed macro change. These reports do not prove
absence of callers or replace executable checks.

API-018 documentation discovery also confirmed Markdown remains gated out of
rustdoc, the README contains old examples and broad host claims, and several
source examples are ignored. Those documentation repairs and the complete
supported matrix remain required under API-018. API-019 and API-020 remain open;
the image-capture outer-timeout ownership defect is still recorded for repair.

### API-018 Props change: platform evidence refresh

API-001 through API-007 have current passing receipts. API-008 passed 13
clipboard behavior cases and two process lifecycle cases, then rejected old
platform input digests. The three Linux backends now each pass five native
round trips and two lifecycle cases; their current input digests and both
output hashes were checked. macOS and Windows are running in GitHub run
34705931916 on snapshot 0d6f176bb8a46da1859a86aec0bd79962a9652cb.
The complete native-input Git trees match local source c2eeb695beaa8222bf04dc8694d3d8861b370867
byte-for-byte. This uses the previously authorized verification branch.
Native results remain pending; no source was changed to bypass freshness.

### API-018 review finding: optional/default compatibility

A further read-only review found an unintended compatibility break in the new
PropOptions parser: it rejects `#[prop(optional, default)]` and a compatible
explicit string default on `Option<&str>`. The approved proposal preserves
existing defaults. Compiler probes in `.cairn/reviews/api-018-optional-default`
confirm the original macro compiles and runs the None/Some assertions, while
the new macro rejects the same declaration. The initial direct macro compilation
needed Cargo's explicit proc_macro extern; that setup error is retained and is
not the regression result. The corrected baseline uses the old source unchanged.

Resolve this finding under API-018: retain explicit default precedence, remove
the extra combination rejection and its incorrect rejection test, and add
regression coverage for both combined defaults. This needs no broader contract
change. It prevents calling the Props work complete despite its targeted passes.
No candidate source changed during the active native API-008 evidence refresh.

Native clipboard run 34705931916 passed on macOS and Windows. Before import,
checked snapshot identity, successful jobs and the named clipboard step,
current input digests, native platforms, execution markers and both output
hashes. Copied original bytes. All five platform records now pass the verifier.

Native run 34705931916 passed on macOS and Windows for snapshot 0d6f176bb8a46da1859a86aec0bd79962a9652cb. Its declared input trees exactly match local commit c2eeb695beaa8222bf04dc8694d3d8861b370867. Imported the widget and ConPTY artifacts after checking job and step success, current committed input digests, system, every recorded output hash, and unchanged copied bytes. Both native verifiers passed. The earlier clipboard records remain current because their declared inputs did not change.

### API-014 refresh: restore the external image-tool PATH

API-011 through API-013 passed on the Props candidate, including real Orca delivery
and the refreshed native records. API-014 then stopped in executable preflight:
this resumed session omitted `/tmp/rtui-image-tools/bin` from PATH. The previous
passing receipt used that directory. Both executable files still exist and report
Chafa 1.18.1 and Viu 1.6.1. Their observed SHA-256 values are respectively
0767691e46bb0f850dbbc23763daf62dc8a9e31db35b1bd2499c9d2de5fa4a15 and
12ca5c02ced0a43ee728570025b752cb07837da15f7d2118b5423784268834c0.
The check did not reach pixel acceptance and its failure is retained. Restoring
that PATH prefix repairs the execution environment; no source or criterion
changed. All subsequent image checks must retain this prefix after resume.

Restoring the image-tool PATH passed API-014/20260912T171411905Z: all 25
host routes, the independent Kitty sender, forced-ASCII rejection and all
89 selected tests. The completed capture files had no open process descriptors
before commit. This verifies normal execution; the separately recorded outer-
timeout cleanup defect remains open for API-020.

### API-018 documentation mechanism declaration

The mechanism builds default, no-default and nightly all-feature rustdoc; requires
Markdown in the module index and all three public submodule pages; runs default
and minimal crate doctests, compiled Cargo examples and standalone public Rust,
C and TypeScript documentation consumers. Ignored host/clipboard examples compile
as no-run consumers so the check cannot seize the developer desktop. Complete C
examples also link against the current library. The generated consumer sources,
source locations, hashes and raw compiler outputs are retained. Standalone Rust
examples receive only the public facade dependency, exposing macro hygiene errors.
The matrix must cover every root public module plus macros, name behavior checks,
link existing evidence declarations and state explicit limits and pending reviews.
Historical receipt bytes are not inputs to their own check; Cairn validates them.

Four checker control tests passed: separate fence extraction; duplicate/incomplete
matrix rejection; missing Markdown index/submodule rejection; and rejection of
zero executed cases and nested failures. The first control run hit a test setup
error because its temporary output was outside the repository; the corrected setup
uses an owned target directory. The 16 existing Props behavior tests and two macro
parser tests passed, but the new standalone optional/default consumer failed on
the known regression. This is a baseline failure, not API-018 acceptance. Its
original passing behavior is independently recorded in api-018-optional-default.

Ripwire edit-check found a new checker symbol with no incompatible callers. The
quality report exited 2, including the ignored reference trees, dynamic dispatch
and generated test dead-code findings. Its long example method was separated into
collection, C and TypeScript compilation operations. The duplicated library build
pattern is deliberate Cargo JSON artifact selection, not a second production API.
Test-gate exited 0 but reported zero changed/impacted symbols; that is no proof
of coverage. Raw reports and actual control output are retained under
api-018-documentation-mechanism. Full failing/corrected documentation builds remain
required before acceptance, including public guides found outdated by inspection.

### API-018 documentation baseline and repairs in progress

Formal baseline receipt 20260912T175254297Z failed as required. Default/minimal
rustdoc hid Markdown; all-feature rustdoc had an unescaped packed-selection link.
The ordinary doctest suites each passed 48 cases while ignoring 34. The standalone
consumer run exposed obsolete public examples, including nonexistent C signatures,
hook arguments and CSS macro dependencies. Raw sources and compiler output remain
in api-documentation/20260912T175017936266Z. This failure is not acceptance.

The optional/default compatibility consumer now compiles and runs after removing
the extra rejection. The 17 Props integration cases and two parser tests passed.
An early development --only run printed the full-check success wording even though
it ran only Props; the checker now labels section-only runs as development checks,
and that historical output is not treated as full API-018 acceptance.

The public guides and source examples now use current signatures. A second
development example run passed all Rust groups except the CSS macros, all five
C compilations/links, TypeScript typechecking and the explicit Python ctypes loader.
It confirmed that responsive_css calls an absent method and discards breakpoint
values, and that convenience macros incorrectly require the caller to depend on
taffy. These are implementation defects, not reasons to remove the examples.
A judged decision records the owned responsive-profile repair before construction.

Four initial macro behavior cases passed. The fuller run exposed an over-specific
test assumption: an independent wake can add a seventh acknowledged frame to a
six-resize sequence. The test now requires the exact sequence of distinct painted
viewport widths and checks color/padding on every frame, including duplicates; it
does not weaken the required resize sequence. The two new ScreenManager tests
passed, including rejection of nonfinite values in inactive responsive profiles.
The complete updated run and violating/corrected runtime controls remain pending.

The residual inventory now includes gesture wiring, reference retention/callback
reentry, performance context isolation/mode reporting, legacy transition metadata
and CSS property-diagnostic claims alongside the original API-019 audit concerns.
Compiling these examples is not runtime acceptance and does not close those rows.

### API-018 completed development verification

The final development run at api-documentation/20260912T182059977951Z has no
failing step: 17 Props integration cases, two macro parser cases, four public CSS
macro/App cases and four responsive library cases (including the two new screen
cases). The separate optional/default consumer compiled and ran. Default, minimal
and nightly all-feature rustdoc each expose Markdown and all public submodules;
each configuration passed 74 doctests. All 52 standalone Rust examples across
17 source groups passed compilation/execution as declared, all five C examples
compiled (complete programs also linked), the two TypeScript examples typechecked
and the explicit Python ctypes loader resolved the current library symbols.
The inventory covers all public root modules and explicitly retains pending
API-019 work. The macro crate's facade-dependent examples are compiled externally
by this harness, including the Props sample; they are not silently excluded by
the macro crate's own ignored fences. Generated sources retain their locations
and hashes. Full outputs are preserved; this remains development verification
until a committed Cairn check records current acceptance.

The responsive negative control changed only width selection to zero. The actual
App color assertion failed with red instead of required green; restoring the exact
source bytes made all four public macro cases pass. Raw negative and corrected
outputs and command records are in api-018-responsive-controls. Screen tests also
prove reverse resize and reject NaN/infinity even in inactive profiles. The
absolute-fill helper now uses parent percentages and zero insets, tested with an
earlier sibling and a parent smaller than the viewport. Breakpoint closures run
once, in stable ascending width order, and only materialized data is serialized.
App and ScreenManager resolve before accessibility/state variants, motion and
painting. Explicit Element classes retain precedence and independent App widths
do not share profile selection.

Strict default all-target Clippy, macro library/test Clippy and workspace formatting
passed; logs are in api-018-final-development. Ripwire profile and snapshot
edit-checks exited 0. Quality-delta exited 2 and test-gate exited 4; neither is
reported as a pass. Reviewed changed-path findings: the language-specific example
collector adds Python/no-run selection; its branch count reflects explicit fence
inventory, outside the application hot path. Reported dead code includes macro
expansion and actual executed tests, and short matching expressions overlap
unmodified code and archived reference trees. Snapshot/Props churn is the
documented correction. The test gate's broad name-based reach includes archived
host code, manual examples and retained renderer/widget paths; fresh inherited
Cairn mechanisms and native records remain required before closure.

Production review of this change found no new unsafe code, no runtime callback
serialization, no hidden field rewriting and no new dependency. Nonresponsive
styles keep the existing path; responsive profiles validate before rendering.
The known residual behavior findings and image-capture timeout leak remain open
and recorded. This is not the API-020 final review or a commitment completion claim.

Native clipboard run 34711569893 passed on macOS and Windows. Before import,
checked snapshot identity, successful jobs and the named clipboard step,
current input digests, native platforms, execution markers and both output
hashes. Copied original bytes. All five platform records now pass the verifier.

Native run 34711569893 passed on macOS and Windows for snapshot de66089a8d536d2e6fdfbb5e2f87ac292ab7e5e9. Its declared input trees exactly match local commit ed8e19df97529178b8b005bdf222257318cd7122. Imported the widget and ConPTY artifacts after checking job and step success, current committed input digests, system, every recorded output hash, and unchanged copied bytes. Both native verifiers passed. The earlier clipboard records remain current because their declared inputs did not change.

## API-017 migration guide regression during API-018 verification

The new formal check caught drift between the packaged migration guide and its
canonical sources before running native consumers. Corrected the source ownership
paragraph and rebased the native-guide link in the exact-content comparison.
Both prose-drift and broken-link mutations still fail; the full corrected API-017
development mechanism passed. Details, original failure and control outputs are
in `.cairn/reviews/api-018-migration-guidance/`. Formal acceptance remains pending.

## Absolute-fill assertions found by inherited verification

ABI-004 caught two old tests expecting a 100-by-100-cell box after the API-018
parent-fill repair. Updated their four dimension assertions to the documented
parent-relative behavior and included both CSS groups in API-018 acceptance.
A fixed-cell macro mutation failed both tests and the independent App frame test;
restored behavior and the complete inherited development mechanism passed.
Evidence and review: `.cairn/reviews/api-018-fill-contract/`. This is a test and
acceptance-coverage correction; formal evidence still needs refreshing.

Native clipboard run 34714603188 passed on macOS and Windows. Before import,
checked snapshot identity, successful jobs and the named clipboard step,
current input digests, native platforms, execution markers and both output
hashes. Copied original bytes. All five platform records now pass the verifier.

Native run 34714603188 passed on macOS and Windows for snapshot 75f7ffe3ac89292e3c47acaeae07793416a3c926. Its declared input trees exactly match local commit 13c28444cc9b9e61a73549376a70e59feac39026. Imported the widget and ConPTY artifacts after checking job and step success, current committed input digests, system, every recorded output hash, and unchanged copied bytes. Both native verifiers passed. The earlier clipboard records remain current because their declared inputs did not change.


## API-019 declaration and initial behavior baseline

API-018 passed its expanded documentation mechanism in receipt
20260912T201523567Z. The inherited refresh then passed all 34 requirements;
RND-001 through RND-006 are the final receipts at 20260912T202313113Z/114Z.
Cairn now names declare API-019. No production implementation changes here.

The residual inventory now explicitly retains the legacy backend test-reachability
concern from this review at line 4161, separately from nested event routing.
Current source still nests all five mapping tests inside DebugBackend::render_full.
The platform entry also preserves review of the four Windows and six FFI warnings.
Graph discovery was attempted and returned Transport closed; focused Tilth reads
and Ripwire provided fallback discovery.

The new mechanism declares the complete dependency footprint and rejects missing
coverage. It begins with six actual public reference tests and discovery/execution
of the five existing backend mappings. Complete focused checks for every residual
contract remain mandatory; the inventory gate deliberately rejects all unfinished
coverage. It cannot be passed by editing inventory statuses or running one section.
These initial tests do not cover the entire reference ownership contract yet.

Development output: api-residual/20260912T203110244158Z. All six reference tests
compile and are registered. Five fail assertions: shared/local/callback/multi
references lose their retained state, and callback read/write reentry deadlocks.
The isolated reentry child was killed and reaped at its three-second deadline.
The forwarded-reference control passes and follows replacement parents correctly.
The backend filter executes zero tests; both discovery and execution reject it.
These are tests of required correct behavior, not defect-confirming acceptance.

All five checker control tests pass. Safe missing/duplicate inventory rows, absent
registered names, zero/ignored/failed tests, incomplete coverage and subprocess
failures are rejected; corrected controls pass. Actual mapping mutation and the
corrected reference behaviors remain to be demonstrated during implementation.
Rustfmt and git diff --check pass. Ripwire edit checks report new test/checker
symbols with no incompatible callers. Quality-delta exits 2 and test-gate exits 4
(outputs in api-019-declaration); neither is a passing quality gate. Most findings
come from unchanged reference trees or tests/dynamic dispatch classified as dead.
Two short output/group wrappers resemble the existing documentation checker;
this declaration reuses its process executor, keeps outputs separate, and makes
no production abstraction change. The test gate models no executed tests; the
actual compiler, harness output and checker controls above provide the evidence.

API-019 is not complete. No reference ownership, updater, gesture, transition,
platform or contract-narrowing decision has been made or implemented here.

API-011/20260912T203756669Z completed its App, widget, transport and real Orca
workflows, then rejected stale ConPTY evidence: tests/api_residual_refs.rs is
inside the existing broad native input declaration. No widget assertion failed.
Started native run 34717703879 on authorized branch
codex/clipboard-platform-verification, snapshot fd0aed29c3366e6f8e2d7e2959eaba438fc7eaee.
Its complete declared input tree was compared byte-for-byte with local source
56441c73d4202077b035931c1dcb060aee1d953b before non-forced push. Metadata is in
api-019-declaration-native/. Native import and formal refresh remain pending.

Clarification of declaration whitespace verification: authored source, inventory,
declaration and review paths passed git diff --check. The staged raw Rust harness
outputs had their original trailing whitespace and blank final lines; that broader
check exited 2. Those captured bytes were preserved, not rewritten or called clean.
