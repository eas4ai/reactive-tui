# Reuse WezTerm components during framework cleanup

Surfaced from: EMB-005
Captured: 2026-09-07T23:46:14.261Z

Source inspection of reference/wezterm-20240203-110809-5046fc22 identified portable-pty for PTY portability, Termwiz wakeable polling and input parsing, and mux output notifications as cleanup candidates. Record the detailed assessment in this backlog entry. No adoption or implementation is authorized by this assessment.

## Purpose and status

Treat reusable WezTerm components as part of the framework cleanup sweep.
The renderer and embedded-terminal commitments are complete. This entry
records candidates for subsequent commitments; it does not change their
requirements or select a replacement dependency.

Assessment date: 2026-09-07. Evidence is source inspection of the local
20240203-110809-5046fc22 snapshot. No WezTerm integration, build, benchmark,
or host-compatibility test was performed. These findings describe this
snapshot, not the latest upstream release.

## Candidates

| Candidate | Cleanup opportunity | Integration work and limits |
| --- | --- | --- |
| portable-pty 0.8.1 | Replace custom PTY platform plumbing with its Unix and Windows implementations. | Adapt Command configuration, child status and IO ownership. Preserve our bounded buffering, shutdown, worker joining and direct-child reaping. Adding a Windows PTY does not establish Windows support for the native interpreter or full application. |
| Termwiz terminal waker | Let background state changes wake App instead of waiting for the next polling interval. | Define a shared wake handle for signals, scheduler work and terminal output. Coalesce notifications and retain frame-rate limits, timer deadlines and shutdown behavior. |
| Termwiz input parser | Compare host input decoding and add structured paste/mouse handling when specified. | Compare with Crossterm using captured host input. This snapshot's KeyEvent carries key and modifiers but lacks our press/repeat/release distinction. Preserve those semantics or record an explicit decision before replacement. |
| WezTerm mux notifications and changed-line tracking | Connect output changes to redraw and guide future pane invalidation. | Borrow the notification and sequence-number approach. The implementation depends on WezTerm's mux and terminal types; it is not a standalone replacement for our App or reactive system. |

## Source pointers

Paths below are relative to the repository root. The reference checkout is
local assessment material, not a new production dependency.

- `reference/wezterm-20240203-110809-5046fc22/pty/Cargo.toml`:
  portable-pty package and dependency boundaries.
- `reference/wezterm-20240203-110809-5046fc22/pty/src/lib.rs:87`:
  MasterPty, Child and ChildKiller interfaces.
- `reference/wezterm-20240203-110809-5046fc22/termwiz/src/terminal/mod.rs:87`:
  poll_input and waker contract.
- `reference/wezterm-20240203-110809-5046fc22/termwiz/src/terminal/unix.rs:300`:
  wake notification; `:410` polls input, resize and wake descriptors together.
- `reference/wezterm-20240203-110809-5046fc22/termwiz/src/input.rs:64`:
  input event variants; `:101` defines the key event representation.
- `reference/wezterm-20240203-110809-5046fc22/mux/src/lib.rs:117`:
  applying terminal actions emits a PaneOutput notification.
- `reference/wezterm-20240203-110809-5046fc22/mux/src/renderable.rs:62`:
  changed rows selected by sequence number.

## Suggested cleanup order and proof

1. Specify a shared App wake mechanism. Prove that a background change wakes
   a blocked event loop, bursts coalesce without losing the latest state,
   idle App avoids periodic redraw, and timers, input and shutdown remain
   responsive. Keep the existing renderer and Ghostty interpreter.
2. Reproduce the reported counter-input failure in the developer's host.
   Compare decoded events against the captured bytes before choosing an
   input-library replacement. Exercise modifiers, Unicode, Escape ambiguity,
   repeat/release and split input sequences. Bound paste buffering if added.
3. Evaluate portable-pty behind the existing session boundary. Rerun real-PTY
   command/configuration, resize, Ctrl+C, natural exit, startup failure,
   shutdown and reaping checks. Test each claimed platform directly.
4. Use pane notifications and changed-line tracking when pane composition
   or measured snapshot cost becomes a commitment. Demonstrate reduced work
   without losing erasure, cursor, style or resize updates.

Before adopting a crate or copying code, select the exact source revision,
review its dependency footprint and retain applicable provenance and notices.
No whole WezTerm GUI or mux import is proposed.

## Remaining cleanup stays open

These candidates do not resolve the component-registry concurrency stall,
JumpStart easing expectations, CSS/z-index failures, broken documentation
examples, legacy backend behavior, FFI test compilation or TypeScript
validation. Keep the existing backlog entries and baseline evidence until
fresh checks establish each repair. Passing embedded-shell probes do not
close the developer-host counter-input report.
