# Binding ABI compatibility review

commit: 2af681c3194fa33728d6a2524f0dff1f511c3345
findings:
  - open: ABI-001 audit report introduction still describes baseline failures as current and says no declarations were removed; distinguish historical findings from the repaired contract.

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

## ABI-001 mechanism review after sentence clarification

The revised requirement separates native export preservation from migration
reporting into two sentences. Neither obligation nor the falsifier changed.
Read check-binding-abi.py against both obligations: it compares the original
native signatures/records/callbacks/enums, validates the full old C and TypeScript
inventories, and checks current compiler and loader types independently. The
existing missing-symbol, C-signature, capabilities-layout, TypeScript-signature,
Koffi-layout and migration-omission demonstrations reject the stated violations;
the restored gate passed in the committed ABI-001 receipt. Those demonstrations
still apply to the clarified wording. No mechanism mismatch found; no code
changed during this review.

## ABI-002 mechanism review after sentence clarification

The revised requirement separates C/C++ compilation from boundary round-trips;
its obligations and falsifier are unchanged. Read check-c-binding-abi.py and
binding_abi_consumer.c: the compiler-backed audit runs before native calls,
both C11 and C++17 compile with strict diagnostics, and PTY runs exercise
capability canaries, terminal mode, Unicode/RGB/attributes, resize/render,
legacy aliases, consumed children, owned clones and string release. The safe
wrong-heading-alias demonstration fails its observable class assertion; the
restored C and C++ consumers passed in the committed ABI-002 evidence. The
independent layout violations also fail before invocation. No mechanism mismatch
found; no code changed during this review.

## ABI-004 mechanism construction

The combined runner invokes the nine existing acceptance commands in order,
checks that they cover exactly the 30 inherited requirements, and verifies that
its declaration includes their input paths and declaration files. Each command
has a ten-minute deadline and a failing command stops the run. The underlying
commands and assertions are unchanged, and all 30 inherited requirements remain
individually named in the commitment.

Three guarded development mutations were restored in finally blocks: replacing
the format command with false failed with exit 1; removing MNT-001 failed the
coverage check; adding an undeclared dependency failed the input check. The
restored combined runner passed all nine commands. The earlier specification
lint failure and corrected pass also demonstrate inherited failure propagation.
Ripwire located the new runner with cyclomatic complexity 9; its qualified
edit-check lookup did not resolve the symbol and is not claimed as a pass.

## Final code review

Read the native builder diff, nullable callback signature, TypeScript loader,
terminal/surface/renderer/component/builders, package exports, native consumers,
compiler probe, loader probe, migration guides and audit report. The audit report
needs the documentation correction recorded above. No other blocking finding was
identified in the repaired paths.

Attacked allocation ownership across builder/component handles: repr(transparent)
provides their shared Element layout; array aliases reject null/duplicate children
before consumption; Rust-backed clones survive parent destruction. TypeScript
marks builders/children consumed before consuming calls, guards self-ownership,
frees owned strings through Rust, and invalidates borrowed renderer views after
resize/disposal. Constructor property errors dispose the new Component. Raw FFI
callers still must supply valid live pointers and retained callback lifetimes.

Attacked boundary values and agreement between tools: scalar, RGB, unsigned
coordinates, NUL and disposed-handle validation match the documented wrapper
behavior. Independent rustc function-item inference is not derived from the C
signature; field types, offsets, sizes and enum values are compared separately.
The loaded Koffi interface is also inspected before consumer invocation. The
original native snapshot and complete old declaration inventories guard against
quiet deletion. The twelve recorded violation demonstrations cover these checks
and inherited failure/coverage/dependency propagation.

Checked build/distribution: clean dist generation prevents retired JS artifacts;
ordinary downstream declarations compile without requiring resolveJsonModule;
package metadata locks Koffi and does not download a native library. Linux is the
verified platform; other recognized library suffixes are not cross-platform proof.
Only the required consumer workflows have runtime proof, not every legacy export.
The known legacy signal destructor defect remains explicitly documented/backlogged.

All 34 requirements have passing committed evidence. Earlier pre-commit Ripwire
quality/test-gate failures remain recorded with their FFI/scope limitations; the
final rerun exits zero against unchanged HEAD and adds no proof for the committed
diff. It was not used to dismiss those earlier findings or reset their baseline.

## Production rules self-audit

1. Outcome and paths are mapped by the initial inventories and judged decisions.
2. Native changes are limited to the shared layout guarantee and real aliases;
   unsupported wrappers are explicitly retired rather than replaced with stubs.
3. Generation/schema and boundary helpers have one purpose; ownership remains explicit.
4. Original Rust contracts are compiler-checked; consumer changes have migration inventories.
5. Native errors propagate; output storage and pointer ownership are explicit; no secrets added.
6. Input validation covers safe wrapper boundaries; raw native pointers remain a caller contract.
7. No persistent-data migration is introduced; constructor/disposal/consumption paths were reviewed.
8. Native calls are synchronous, tests have deadlines, and borrowed/owned resources have tested cleanup.
9. Work tracking retains one in-progress item until the documentation finding and final review close.
10. C/C++, TypeScript, downstream types, lint, package output and all inherited checks ran and passed.
11. Failed demonstrations and the specification-lint regression remain in history; platform/native limits are stated.
12. Scope and compatibility decisions are recorded; the separate legacy signal issue is captured for the developer.
13. The stale audit introduction needs revision before delivery; the code review found no other blocker.
14. The documentation correction must identify the historical baseline and current result in plain language.
