# Pre-release C ABI safety

Status: Agreed 2026-09-14
Prefix: FFS

This specification remediates Fable audit findings H1, H2, H3, M9, L13, L14,
L15, and L16. The Rust implementation, generated C header, and TypeScript
binding MUST describe the same ownership and lifetime rules.

[FFS-001]
Application handles and optional callbacks MUST remain valid for every documented
C call. Quitting a running application MUST NOT write through freed storage.
Every nullable callback MUST use a Rust representation that permits null.
Falsifier: A quit call accesses an App allocation after it was moved or freed,
or a C caller can pass null to a callback type that Rust declares non-null.
Mechanism: A focused C/Rust integration check exercises application start,
cross-thread quit, null callbacks, cleanup, and sanitizer-backed lifetime cases.

[FFS-002]
Text-buffer resize MUST keep length, capacity, element storage, and exported
pointers consistent. No exported C function MAY unwind across the ABI.
Falsifier: Shrink, grow, malformed input, or allocation failure causes
out-of-bounds access, a stale exported pointer, or an uncaught Rust panic.
Mechanism: Boundary tests exercise every resize direction and an export audit
proves that every extern function uses the common panic boundary.

[FFS-003]
FFI ownership transfers MUST be explicit and single-use. Animation-manager
insertion, failed AppBuilder builds, and the global log callback MUST be safe
under the documented caller sequence and concurrent use.
Falsifier: A documented sequence can double-free, use freed storage, or race on
the callback value.
Mechanism: Integration tests exercise success, failure, repeated cleanup, and
concurrent logging under Miri or a sanitizer where supported.

[FFS-004]
Returned buffer allocations MUST be released with allocator metadata owned by
the library. Element trees MUST reject self-parenting. Pointer validation docs
MUST state what is actually proven, and tracked handles MUST reject repeated
destruction.
Falsifier: A caller-supplied length controls deallocation layout, an element can
adopt itself, documentation claims arbitrary pointer validity, or a second
destroy reaches the allocator.
Mechanism: Negative ABI tests vary release lengths, aliases, alignment, and
destroy order and require defined error results without memory access.

[FFS-005]
The checked-in C header, TypeScript declarations, examples, and ABI policy MUST
match the Rust exports after remediation.
Falsifier: A declaration has the wrong arity, names a missing function, omits an
ownership rule, or differs from the compiled library.
Mechanism: Regenerate the header and run the existing header-to-Rust and
TypeScript consumer compatibility checks.
