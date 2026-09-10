# Own FileExplorer navigation and filesystem work inside its retained App component

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Directory reads block App input, stale results replace a newer path, an operation overwrites an existing destination or escapes the configured root, views use guessed hit bounds, or removal leaves queued filesystem changes runnable.
History: The existing widget decision preserves public unit components through retained child instances. Earlier ownership reversals require explicit cancellation and actual cleanup evidence. FileExplorer has advertised filesystem operations but no existing operation implementation to preserve.

## Decision

Preserve the public FileExplorer unit component, props and state. Use an App-owned child for path-based selection, expansion, measured targets and asynchronous filesystem jobs. Keep one owned worker with a bounded request slot and result slot; publish completion through the existing App signal subscriptions. Navigation generations reject stale reads. Cancellation clears pending work and is checked between filesystem calls and copy chunks; joining the worker releases it, but an operating-system filesystem call already in progress cannot be preempted by a Rust thread. Use actual list, grid and lazily expanded tree rows with viewport-bounded painting. Keep canonical rooted navigation and path identities separate from lossy display labels. Operations require explicit user commands, never overwrite destinations, report errors and partial work, and require confirmation before deletion. Previews read a bounded prefix and distinguish text from binary. Deliver existing callback IDs through App custom events with path payloads. Add builder callback setters without changing existing signatures. Verify with disposable directories, changing props during reads, cancellation, symlinks, selection modes, every operation, views, resize and safe violating cases before acceptance.

## Realized by

Implementation: `src/widgets/display/file_explorer`.

Behavior checks: `tests/api_widget_behavior/file_explorer.rs`, `tests/api_widget_behavior/orca_data.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
