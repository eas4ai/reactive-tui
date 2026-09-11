# Retain one terminal owner while configuring a native App builder

Level: Judged
Decided by: Codex
Rests on: API-016,ABI-003,MNT-003
Would be wrong if: Repeated terminal selection closes an active raw session, selecting Debug retains a terminal, a failed setup corrupts the builder, or callback and builder ownership change for consumers.
History: The earlier native reversals require testing resource lifetimes directly. A compiled C probe now shows repeated terminal selection returning success but Ctrl+C killing the process because the prior owner disabled raw mode. Keep this Judged: preserve immediate setup, existing return types and handle consumption, and change only private builder storage.

## Decision

Store the Rust AppBuilder and its native terminal-selection state in one private FFI allocation. After either C terminal selector has acquired the shared SuprTUI rendering route, repeated selection of that same route is idempotent and keeps its single live terminal owner. Both the Crossterm and SuprTUI C selectors use this complete-frame route with default options. Selecting Debug replaces it and clears the terminal-selection state; selecting a terminal again acquires a fresh session. Keep setup failures at the selector call and preserve the rest of the builder value. Destroy and build continue to consume exactly the original opaque allocation.

## Realized by

- 6f879a215e1726a4690f57758e534a0d4881166a Restore complete rendering through retained App entry points
