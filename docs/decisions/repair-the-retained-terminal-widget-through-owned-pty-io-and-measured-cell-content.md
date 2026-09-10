# Repair the retained terminal widget through owned PTY IO and measured cell content

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-016 API-019 EMB-001 EMB-005
Would be wrong if: The retained public Terminal and VirtualScreen contracts require abandoning the approved libghostty session path or losing existing terminal behavior.
History: The approved libghostty session remains the native embedded-terminal route. API remediation also explicitly retains legacy public terminal APIs and requires repairing their advertised behavior rather than retiring them.

## Decision

Keep the approved libghostty session and its public TerminalView. Repair the retained legacy Terminal and TerminalWidget APIs alongside it: share the existing owned Unix PTY primitive instead of pipe emulation, provide actual native PTY behavior for claimed Windows operations, and bound input/output queues and shutdown. Preserve the public VirtualScreen interface and repair its observable screen operations. Make TerminalWidget own one retained terminal lifetime, paint actual styled grapheme cells inside measured layout, route focused input and signed scrolling, resize the child to its presented content area, and stop/reap on removal. Changed launch props replace the owned session; display props do not restart it. Record concrete platform evidence and terminal limitations in the inventory. No default feature or public API is removed; broader libghostty pane composition can reuse the same cell-paint representation without replacing its interpreter.

## Realized by

Implementation: `src/widgets/terminal.rs`, `src/terminal/terminal_impl.rs`, `src/terminal/screen.rs`, `src/terminal/owned_pty.rs`.

Behavior checks: `tests/api_widget_behavior/terminal.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
