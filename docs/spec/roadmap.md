# Recovery roadmap

Status: Agreed 2026-09-07
Current: embedded-terminal

1. `suprtui-renderer` — the first screen: styled frames, updates, Unicode,
   resize, output errors, input, and terminal restoration (RND-001 through RND-006).

2. `embedded-terminal` — a real shell PTY interpreted by libghostty-rs and
   displayed through App/SuprTUI, with keyboard input, resize, and cleanup.

The first renderer commitment is complete; its requirements remain inherited.
Image protocols, clipboard, and remaining legacy defects stay in the backlog.
