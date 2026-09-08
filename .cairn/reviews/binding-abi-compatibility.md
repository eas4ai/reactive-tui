# Binding ABI compatibility review

## Work tracking

- Complete: establish the commitment, capture the failing baseline, and record compatibility decisions.
- Complete: reconcile C/TypeScript declarations, ownership, consumer workflows and migration guidance; verify failure demonstrations and packaging.
- In progress: record committed acceptance evidence, refresh inherited requirements and complete final review.

## Verification approach

Audit declarations before any native invocation. Use compile/link and layout
probes to reject known mismatches, then run valid consumer workflows in bounded
processes and controlled terminals. Keep baseline failures and demonstrate that
corrected checks reject safe signature/layout/value violations.

## Baseline inventory

The static baseline failed with 75 missing C names and 38 missing TypeScript
loader names, against 194 built exports. All 13 headers compile together as C
and C++; that does not establish ABI compatibility. The baseline JSON includes
198 C function signatures, 58 loader names, 52 RTui typedefs, 20 record
definitions and 14 enums. Source review confirmed terminal, renderer, animation
and capability mismatches without invoking them. The inventory gate currently
checks symbol presence; compiler-backed signature and layout checks are still
required before ABI-001 can be considered complete.


## Mechanism construction and failure demonstrations

Replaced the name-only gate with independent rustc function-item type inference,
C/Rust record/enum programs, C/C++ compilation and actual Koffi interface inspection.
The gate checks original native compatibility and complete consumer migration
inventories. No native function runs during the declaration/type stage.

Nine guarded violations failed for their intended reasons: missing export, C
signature drift, capability layout drift, TypeScript signature drift, actual Koffi
packing, migration omission, wrong RGB result, wrong heading alias and omitted
native string release. Restored C/C++ and TypeScript consumers passed. Full logs
are in docs/binding-abi-demonstrations; these development runs are not receipts.

The package build removes stale generated files. npm ci, the package and example
TypeScript checks, configured ESLint and npm pack dry-run passed. Ripwire's
nonzero quality/test-gate results and their scope/FFI limitations are recorded in
the demonstration README; they have not been represented as passes.

The final development rerun passed both C/C++ and TypeScript consumer scripts,
including the independent ABI gate and a downstream declaration compile without
resolveJsonModule. Generated header/schema/type verification passed. The package
dry run contains 43 files, including the native schema, standalone native name
types and migration guide; retired compiled modules are absent.
