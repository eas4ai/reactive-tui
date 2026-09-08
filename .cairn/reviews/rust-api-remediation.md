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
