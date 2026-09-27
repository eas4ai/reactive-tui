# Terminal input

Prefix: INP

How terminal input reaches the App on the default backend: the modes the
backend turns on and off, how crossterm's events become ours, and how mouse
events are dispatched. The model is ~/workspace2/textual-rs, whose mouse the
developer called as good as a desktop application's.

Observed at 18ebd3be, by reading and by one run:

- The default backend's setup writes only `ESC[?1049h ESC[?25l ESC[?1004h`
  (src/backend/suprtui/output.rs:171). A run of the widget catalog in a
  pseudo-terminal on 2026-09-26 set only the modes 1049, 25, 1004 and 2026:
  no mouse reporting and no bracketed paste, so no mouse event reaches a
  widget in a real terminal. Only the DirectTty backend turns the mouse on
  (src/backend/direct_tty.rs:69; src/platform/mod.rs:582-587).
- `CrosstermBackend::map_ct_event` turns every wheel report into
  `MouseEventKind::Wheel` with no `WheelDelta` (src/backend/mod.rs:339-348),
  and the scroll view and the table ignore a wheel event without one
  (src/widgets/layout/scroll_view.rs:413-418;
  src/widgets/display/table/live.rs:622-626).
- Every mouse event goes to the node under its own position
  (src/event/router.rs:677-691), and a component drops mouse events outside
  its bounds (src/component/runtime.rs:432-443), so a drag that leaves the
  pressed element is lost to it.
- Click counting exists for `MouseEventKind::Click`
  (src/hooks/processor.rs:537-574), but no backend produces Click,
  DoubleClick or TripleClick.
- The scroll view consumes every wheel event it handles
  (src/widgets/layout/scroll_view.rs:413-433), so a nested view never passes
  the wheel to its parent.
- `CrosstermBackend` does not pass the renderer's hit grid through
  (src/backend/mod.rs:357-416), so it falls back to painted bounds, against
  PNT-002.

Activation is not changed here: `on_click` still fires on the left press
(src/app/event_tree.rs:334-349). The keyboard protocol, the terminal's own
focus reports, suspend and resume, and terminal queries belong to the
keyboard-and-queries commitment.

## Observed

(none yet)

## Draft

[INP-001] On a terminal, the default backend MUST enter with crossterm's `EnableMouseCapture` and `EnableBracketedPaste` commands, which turn on press, release, drag and any-motion mouse reports in SGR coordinates and bracketed paste, and MUST leave with `DisableMouseCapture` and `DisableBracketedPaste` on a normal exit, an error exit and a panic on the App thread.
Falsifier: Run on the default backend in a pseudo-terminal, an app writes no `ESC[?1000h`, `ESC[?1002h`, `ESC[?1003h`, `ESC[?1015h`, `ESC[?1006h` and `ESC[?2004h` before its first frame, or after a normal exit, an error exit or a panic the last write of one of those modes sets it.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-002] Every mouse report and bracketed paste the terminal sends MUST reach the App as one event that keeps what the report says: a wheel report MUST carry its direction as a one-line `WheelDelta::Lines` (up and down on y, left and right on x); press, release and drag reports MUST keep their button; every mouse report MUST keep its Shift, Alt and Ctrl modifiers and its cell; a bracketed paste MUST arrive as one paste event with its whole text, newlines included, and as no key events.
Falsifier: For a fixed table of SGR mouse reports (the left, middle and right buttons pressed, dragged and released, motion with no button, and the wheel in four directions, each with no modifier and with Shift, Alt and Ctrl) and one bracketed paste of two lines, written to an app on the default backend in a pseudo-terminal, an event reaches the App with another kind, button, modifier, cell or wheel direction, one is missing, or the paste arrives as more than one event or as keys.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-003] After a press on an element, the drags and the release of that button MUST be delivered to that element's handlers until the release, also while the pointer is outside the element, and the elements under the pointer MUST NOT receive them.
Falsifier: In a pseudo-terminal, a press on element A, a drag across element B and a release over B deliver a drag or the release to B, or fail to deliver one of them to A.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-004] A release over the element that got the press MUST be followed by a `Click` event for that element; a click on the same element and cell within 500 ms of the previous click MUST be a `DoubleClick`, and the third and any later one a `TripleClick`; a release over another element MUST produce no click event.
Falsifier: In a pseudo-terminal, a press and release on element A produce no Click for A; two clicks on one cell 100 ms apart produce no DoubleClick, or three produce no TripleClick; two clicks 600 ms apart produce a DoubleClick; or a press on A released over B produces a click event for either.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-005] A wheel event MUST go to the element under the pointer and bubble to its ancestors until one scrolls; a scroll view, a table or a tree that cannot scroll further in the wheel's direction MUST leave the event unhandled so an ancestor can scroll; Shift with a vertical wheel MUST scroll sideways.
Falsifier: In a pseudo-terminal, with a scroll view holding a table, a tree and a second scroll view, a wheel down over any of the three at its last row leaves the outer view unscrolled, a wheel down over one that can still scroll scrolls the outer view, or Shift with the wheel down scrolls vertically.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-006] When several motion reports (moves and drags) are waiting, the App MUST handle only the latest of each run before the next frame, and MUST keep every press, release and wheel report in its order.
Falsifier: One hundred motion reports followed by a press, written to the pseudo-terminal in one write, reach the App as more than two Move events before the press, or the press is lost or handled out of order.
Mechanism: input-pty
Status: Agreed 2026-09-27
