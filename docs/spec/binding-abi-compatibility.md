# TypeScript and C-header ABI compatibility

Status: Agreed 2026-09-08
Prefix: ABI

The developer confirmed this commitment after ffi-lint-format-repair.
The contract covers the shipped C headers and TypeScript binding declarations
against the Rust shared library, including their sizes, layouts and ownership.

[ABI-001]
Every function and data type advertised by the shipped C headers and TypeScript
native loader MUST be inventoried and reconciled with the Rust exports.
Missing symbols and incompatible signatures/layouts MUST fail an automated check.
Existing Rust exports MUST remain compatible; corrections to broken consumer
contracts MUST be documented with migration guidance where callers change.
Falsifier: a declared callable symbol is absent, a calling signature or data
layout disagrees, or a public declaration is silently removed to pass the check.
Mechanism: ABI inventory and declaration/export consistency checks.

[ABI-002]
The shipped C headers MUST compile together as C and C++, and audited boundary
values MUST round-trip through the built shared library with matching sizes,
alignments, offsets and enum values. Terminal capabilities and legacy builder
aliases MUST be covered. Owning, borrowed and consuming handles MUST be documented.
Falsifier: a header fails compilation, a layout/value probe disagrees, a required
alias fails linkage, or a verified lifecycle violates its stated ownership.
Mechanism: compiled C/C++ probes and controlled runtime interoperability tests.

[ABI-003]
The TypeScript package MUST typecheck and load the real shared library using only
verified signatures. Its terminal, surface/rendering and builder/component paths
MUST have bounded interoperability tests for creation, configuration, observable
results and cleanup, with null/error handling where the API permits it.
Falsifier: typechecking or loader initialization fails, an interoperability test
fails, or a mismatched declaration is executed before its signature is audited.
Mechanism: TypeScript build plus isolated native-library integration tests.

[ABI-004]
All inherited maintenance, default-suite, registry, wakeup, embedded-terminal
and renderer requirements MUST retain passing evidence after the ABI repairs.
Falsifier: an inherited requirement fails or coverage is disabled to obtain a pass.
Mechanism: inherited acceptance declarations and final review.
