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
