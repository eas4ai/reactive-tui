# ABI failure demonstrations

These are development-time demonstrations, not Cairn evidence receipts. Every
mutation was restored in a finally block before the next scenario. No call with
an incompatible signature or layout was executed. Cairn will record the stable
committed candidate separately.

| Scenario | Violation | Observed rejection |
| --- | --- | --- |
| missing-symbol | Add an unexported C function | rustc cannot find the function item |
| c-signature | Change createRenderer width to uint16_t | Independent C/Rust signatures disagree |
| capabilities-layout | Change the rgb byte to uint32_t | Independent layouts disagree |
| typescript-signature | Restore the old three-argument terminal constructor | TypeScript schema differs from audited C |
| koffi-layout | Pack RTuiCell in the actual loader | Loaded Koffi layout differs from C/Rust |
| migration-omission | Omit one retired declaration from the inventory | Migration inventory is incomplete |
| typescript-value | Return the wrong foreground color | Real consumer round-trip assertion fails |
| builder-alias | Route h1 to h2 defaults | Real C consumer detects the wrong heading class |
| string-release | Skip the Rust string deallocator after conversion | Consumer detects zero native releases |

The corrected C/C++ and TypeScript programs passed after restoration. The final
consumer logs additionally cover raw-mode configuration/restoration and automatic
owned-string release. The package dry run contains current JS/declarations/schema
and migration guidance, with no retired compiled modules. npm ci, strict package
and example typechecking, and the configured ESLint check passed.

## Static analysis limits

The unfiltered Ripwire quality check exited 2 after including ignored reference
source trees. The owned-source rerun also exited 2: its 127 gates mainly identify
intentional ABI corrections, exported FFI aliases as dead code, and short repeated
boundary wrappers. No suppression or baseline reset was added. Compiler signature
and migration inventories establish the intended contract changes; native consumers
exercise the aliases the static graph cannot trace across the FFI boundary.

The test gate exited 4 and named three uncovered paths: animation creation,
spring creation, and the uncompiled style builder. Those Rust implementations were
not changed; the style-builder C declaration is explicitly retired. Existing native
animation signatures are independently verified. The gate also identifies five
Rust FFI test targets, covered by the inherited maintenance check.

Small repeated pointer/output/lifetime checks remain explicit because these types
have different owning, borrowing and consuming contracts. The compiler probe keeps
its record and enum generation together so the paired C/Rust evidence is reviewable.
The tools' nonzero results are recorded here and are not described as passes.
