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
