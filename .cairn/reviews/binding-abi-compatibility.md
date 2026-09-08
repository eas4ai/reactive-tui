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
