# Own non-Send local hook values in explicit render scopes

Level: Judged
Decided by: Shawn and Codex
Rests on: API-004 API-019 API-020
Would be wrong if: Local values lose retention, cross-thread access becomes possible, scope exit leaks retained shares, or ordinary App runs lack automatic support.
History: The developer approved api-004-api-019-api-020, selecting explicit local scopes and deferred foreign-thread cleanup while preserving arbitrary non-Send values and thread-safe Hooks.

## Decision

Add hooks::with_local_hooks as a synchronous local arena owner. Retain thread-safe positional metadata in Hooks and non-Send values only in the creating scope. Require the same live arena and thread for each local slot. Same-thread owner cleanup removes entries; foreign cleanup invalidates metadata for reclamation at the next creator sweep or scope exit. Escape handles retain their normal ownership. App::run reuses an entered scope or creates one, owning self inside its closure so App cleanup precedes scope exit. Preserve kind/type/count checks, reject missing and expired scopes, and hold no arena borrow across user callbacks or value destruction.

## Realized by

- cdd3e77ff16adc4e2e259c18924af147cadaf022 Own local reference scopes and restore mapping test discovery
