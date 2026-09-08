# Recovery roadmap

Status: Agreed 2026-09-07
Current: ffi-lint-format-repair

1. `suprtui-renderer` — the first screen: styled frames, updates, Unicode,
   resize, output errors, input, and terminal restoration (RND-001 through RND-006).

2. `embedded-terminal` — a real shell PTY interpreted by libghostty-rs and
   displayed through App/SuprTUI, with keyboard input, resize, and cleanup.

The first renderer commitment is complete; its requirements remain inherited.
Image protocols, clipboard, and remaining legacy defects stay in the backlog.

3. `app-wakeups` — shared wake notifications connect signals, scheduled work
   and terminal output to App, with idle waiting and bounded redraw bursts.

4. `registry-concurrency` — remove the concurrent registry metrics/cleanup
   stall and verify lifecycle reentry and concurrent progress.

5. `registry-cache-isolation` — isolate named lookup between registries, make
   clones observe shared registrations, and refresh the full-suite assessment.

6. `default-suite-repair` — repair numeric inset layering, step-easing test
   semantics and accordion doctests; make the default full suite pass.

7. `ffi-lint-format-repair` — repair FFI compile/link integration, strict
   default-feature Clippy failures and workspace formatting debt.
