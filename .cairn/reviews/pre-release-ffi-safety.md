# Review: pre-release-ffi-safety

commit: 17e42e86
findings:
  - resolved: FFS-001 now rejects deliberate moved-App ownership and bare nullable callback fixtures while accepting the corrected representations.

## Scope examined

Re-read the five requirements, mechanism declarations, focused C consumers,
Rust representation tests, current ownership wrappers, generated C header,
TypeScript wrapper, and the latest committed evidence. Compared each mechanism
with its falsifier and checked whether the failure path is distinct from the
corrected case.

FFS-002 has a syntax-aware fixture with one panic boundary removed. FFS-003
exercises consumed builder and animation handles plus reentrant concurrent log
callback replacement. FFS-004 varies caller lengths, interior and misaligned
aliases, release order, repeated releases, repeated destroys, and self-parenting
under ASan and UBSan. FFS-005 verifies regenerated declarations, independent
Rust/C layouts and signatures, strict C and C++ consumers, the compiled header
example, TypeScript build/lint/examples, and a native TypeScript consumer.

## Resolved finding

The FFS-001 declaration says a safe fixture with the former moved `Box<App>`
ownership and a bare nullable function pointer must fail under a sanitizer or
an equivalent Rust lifetime and representation assertion. FFS-001 now runs
three Rust representation tests before the C consumer. The corrected app
source passes an AST ownership assertion for `Mutex<Option<App>>` and the
absence of `Box::from_raw` in `rtui_app_run`. A deliberate fixture containing
`NativeApp { app: App }` and `Box::from_raw` produces both expected errors.

Callback nullability uses the compiler rather than syntax matching. Type
identity functions compile only when the exported aliases are exactly
`Option<extern "C" fn>`. Two `compile_fail` doctests define bare callback
aliases and attempt the same conversion; rustdoc rejects both fixtures on every
run. The latest evidence records all three representation tests, both doctests,
and the sanitizer-backed C consumer passing. This satisfies the declared safe
violating case without relying on undefined behavior at the C ABI boundary.

Fresh FFS-002 through FFS-005 evidence also passes after the shared mechanism
inputs changed. No new blocking finding was identified in the commitment
footprint.

No executable code changed during this review. The existing warnings and
Linux-only runtime coverage remain assigned to later remediation commitments.

## FFS-004 rewording review (spec lint repair)

Re-read revised FFS-004 and its falsifier against mechanism
`pre-release-ffi-buffer-ownership`. The revision splits one two-obligation
sentence into two sentences with identical meaning: pointer validation docs
state what is actually proven, and tracked handles reject repeated
destruction. The falsifier is unchanged in meaning. The mechanism runs a
sanitizer-backed C consumer covering wrong-length release, double release,
double destroy, and self-parenting, and requires the pointer contract to state
that plausibility checks do not prove allocation, liveness, ownership, or
readability — so both revised obligations remain covered.

Failure demonstration: a throwaway probe (`/tmp/ffs004-review-probe.py`, not
committed) applied the check's own limit-phrase predicate to the current
`src/ffi/pointer.rs` (accepted) and to an in-memory copy with the "liveness"
limit removed (rejected). No mismatch found. No code changed during this
review.
