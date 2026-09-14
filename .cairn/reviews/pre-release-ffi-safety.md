# Review: pre-release-ffi-safety

commit: 6fe244d8
findings:
  - open: FFS-001 passes without the declared safe violating fixture for moved App ownership and bare nullable callback types.

## Scope examined

Read the five requirements, mechanism declarations, focused C consumers,
syntax-aware export inventory, current Rust ownership wrappers, generated C
header, TypeScript wrapper, and the latest committed evidence. Compared each
mechanism with its falsifier and checked whether the failure path is distinct
from the corrected case.

FFS-002 has a syntax-aware fixture with one panic boundary removed. FFS-003
exercises consumed builder and animation handles plus reentrant concurrent log
callback replacement. FFS-004 varies caller lengths, interior and misaligned
aliases, release order, repeated releases, repeated destroys, and self-parenting
under ASan and UBSan. FFS-005 verifies regenerated declarations, independent
Rust/C layouts and signatures, strict C and C++ consumers, the compiled header
example, TypeScript build/lint/examples, and a native TypeScript consumer.

## Finding

The FFS-001 declaration says a safe fixture with the former moved `Box<App>`
ownership and a bare nullable function pointer must fail under a sanitizer or
an equivalent Rust lifetime and representation assertion. The current
`run_ffs_001` only builds and runs the corrected C consumer. That consumer
passes null callbacks and exercises cross-thread quit, but a successful call
does not prove the Rust callback type is `Option<extern "C" fn>`: passing null
to a bare Rust function-pointer parameter is already undefined at the ABI
boundary and need not trap. The mechanism also has no deliberate moved-Box
fixture. Therefore its passing receipt does not establish the full declared
mechanism.

Add a syntax-aware Rust representation inventory that inspects the App owner
and nullable callback aliases structurally, accepts the corrected source, and
rejects a safe fixture containing both former violations. Run that inventory
as part of FFS-001 before its sanitizer-backed C consumer. Source substring
matching alone is insufficient.

No executable code changed during this review. No other blocking finding was
identified in the commitment footprint. The existing warnings and Linux-only
runtime coverage remain assigned to later remediation commitments.
