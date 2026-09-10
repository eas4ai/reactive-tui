# Validate retained terminal dimensions before allocating screen buffers

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: Valid callers change behavior, invalid dimensions allocate large buffers before rejection, or a widget starts a child with fallback dimensions.
History: The retained terminal decision preserves public signatures and bounded owned resources. Resize already rejects zero dimensions and areas above 262144 cells, but constructors allocate first and can panic later or exhaust memory.

## Decision

Use the same nonzero and 262144-cell limit for construction and resize. Add fallible try_new constructors to VirtualScreen and Terminal. Preserve their existing new signatures and valid behavior; document that invalid dimensions panic immediately, directing callers with variable dimensions to try_new. TerminalWidget uses fallible construction, retains an actionable invalid-size error, and refuses to start until a valid measured or explicit resize succeeds. Its small internal placeholder never starts a child. Verify zero and oversized dimensions without attempting dangerous allocations, no child launch on invalid widget configuration, valid resize recovery, and unchanged normal PTY/App behavior.

## Realized by

Implementation: `src/widgets/terminal.rs`, `src/terminal/terminal_impl.rs`, `src/terminal/screen.rs`, `src/terminal/owned_pty.rs`.

Behavior checks: `tests/api_widget_behavior/terminal.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
