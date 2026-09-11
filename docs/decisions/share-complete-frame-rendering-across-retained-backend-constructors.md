# Share complete-frame rendering across retained backend constructors

Level: Judged
Decided by: Codex
Rests on: API-016,API-019,RND-001,RND-004,RND-006
Would be wrong if: A retained constructor or output option becomes a no-op, native input is replaced with fabricated events, frame geometry differs from painted cells, or terminal ownership leaks or restores another active session.
History: Prior API reversals exposed native lifetime and observation mistakes. Keep this Judged because the commitment explicitly permits functional adapters and retains SuprTUI. Require real input, output, option-effect and restoration checks; do not infer native-platform acceptance from local compilation.

## Decision

Use the existing SuprTUI complete-frame painter and acknowledged output for CrosstermBackend and DirectTtyBackend. Crossterm retains its constructor and input mapping; its output-optimization switch remains effective: enabled uses incremental output and disabled forces complete frame output. Both modes preserve graphemes and component geometry. Retain its debug overlay through the shared frame output. DirectTty retains native input, capability access and hyperlinks, with one owned raw terminal and no second Renderer terminal guard; retain unread events from each native batch and observe actual terminal size changes. Repair shared Unix descriptor ownership so cloned or successive terminal handles cannot double-close a reused descriptor. DebugBackend uses the same painter for complete frames without opening a host terminal and retains its existing scalar inspection and patch-history interfaces. The public standalone core Renderer and its C ABI remain separate retained APIs. Record unfinished asynchronous and advanced platform operations in the API-019 inventory before repairing those concerns.

## Realized by

(none yet: recorded, not built)
