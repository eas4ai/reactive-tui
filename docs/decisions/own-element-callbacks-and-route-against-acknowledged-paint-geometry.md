# Own element callbacks and route against acknowledged paint geometry

Level: Judged
Decided by: Shawn and Codex
Rests on: API-005
Would be wrong if: Callbacks outlive removed elements, handlers run twice, or hit testing differs from the acknowledged painted frame.

## Decision

Add owned Element metadata for thread-safe event callbacks, as approved in the api-005 escalation. Constructors supply defaults; literal callers add metadata: Default::default(). Builder on_click accepts Send + Sync callbacks. Return clipped node bounds and stable paint order from the SuprTUI worker with each acknowledged frame; App connects the resolved element tree to routing only after successful presentation. Track nodes by parent-scoped keys and positional slots, retaining IDs through updates. Route keyboard activation to focus and mouse activation to the topmost visible target, with ancestor propagation and retained handled status. Replace callback registrations on redraw and release them on removal. Keep the C ABI unchanged. Other backend adapters can forward the geometry accessor; their supported integration remains covered by API-016.

## Realized by

- 2d83e085878d76e68902835f0171df97792984b7 Retain builder callbacks and route against painted frames
