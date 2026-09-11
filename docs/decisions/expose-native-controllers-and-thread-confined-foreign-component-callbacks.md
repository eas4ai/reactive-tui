# Expose native controllers and thread-confined foreign component callbacks

Level: Judged
Decided by: Codex
Rests on: API-017, API-002, API-007, API-009, API-012, ABI-001, ABI-002, ABI-003
Would be wrong if: A compiled C or TypeScript consumer cannot edit and paint Unicode text, apply the native layout engine, open and complete a dialog, retain and update foreign state, observe callback failures, or release callback ownership without use after disposal.
History: Five API reversals are recorded: clipboard cold-start deadlines, ConPTY runtime provenance, terminal history placement, and AT-SPI translation. This design claims only exercised behavior, keeps the recovered implementations, and makes errors and ownership measurable rather than inferring them from declarations.

## Decision

Add native editor, layout-style and dialog controllers that use the recovered Rust implementations. Editor access includes Unicode editing, movement and selection, viewport, content and styled Element output. Layout styles use the existing validated inline CSS parser and shared painter. Dialog access opens each recovered family, updates sessions, returns the normal App host Element, and drains native opened/closed results with their data. Add a foreign component controller with owned JSON prop/state snapshots, synchronous render/event callbacks, explicit disposal, and creating-thread checks. Controller state survives Element recreation; separate controllers remain isolated. No native lock is held while invoking foreign code. Reads and state/prop updates during callbacks are allowed; recursive callback entry and disposal during a callback return InvalidState. Disposal disables later callbacks even if old Element snapshots survive. The caller retains callback/userdata storage until successful disposal; TypeScript retains registered callbacks and unregisters only afterward. Add fallible component rendering with compatibility defaults so callback errors reach App.run while terminal restoration and component cleanup still occur. Existing C signatures and Rust APIs are preserved. Uncompiled contradictory wrappers are replaced with these defined APIs; they are not counted as previously working ABI. Compiled C and TypeScript consumers must verify visible frames, real input, results, invalid arguments, state/props, reentry, errors and cleanup, alongside independent compiler and loader ABI checks.

## Realized by

(none yet: recorded, not built)
