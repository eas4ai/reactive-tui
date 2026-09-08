# Binding ABI compatibility review

## Work tracking

- In progress: establish the commitment and inventory the advertised ABI.
- Pending: reconcile and verify C declarations, layouts and aliases.
- Pending: reconcile and verify TypeScript loading and consumer workflows.
- Pending: refresh inherited evidence and complete final review.

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
