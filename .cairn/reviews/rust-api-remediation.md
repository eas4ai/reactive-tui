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
