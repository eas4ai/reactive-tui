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
(src/app/event_tree.rs:334-349).

Observed for keyboard-and-queries at 059bb9a6, by reading:

- The default backend pushes no keyboard protocol, so Shift+Enter arrives
  as Enter and Ctrl+H as Backspace. `map_ct_key_code` turns crossterm's
  lock, media, modifier, menu, pause and print-screen keys into
  `KeyCode::Unknown`, although `KeyCode` has CapsLock, NumLock,
  ScrollLock and six media keys (src/backend/mod.rs, map_ct_key_code;
  src/event/types.rs, KeyCode).
- The vendored crossterm can push the disambiguate and report-all-keys
  flags but not report-associated-text (16), and its `CSI u` parser skips
  the text field (crates/reactive-tui-crossterm/src/event.rs:295-309,
  crates/reactive-tui-crossterm/src/event/sys/unix/parse.rs,
  parse_csi_u_encoded_key_code). It parses the keyboard-flags reply and
  the device-attributes reply as internal events, not the
  background-color reply.
- The terminal's own focus reports (`ESC[I`, `ESC[O`, turned on by mode
  1004) become `Event::Focus`, the type element focus changes use, and the
  router sends them to the focused element (src/app.rs process_input;
  src/event/router.rs:476), so a widget cannot tell the window losing focus
  from itself losing focus.
- Nothing handles Ctrl+Z: raw mode turns signals off, so it arrives as a
  key and the App never suspends.
- The default backend sends no terminal query; the theme starts dark and
  changes only when the application sets one (src/theme/mod.rs,
  Theme::active).
- textual-rs, the reference, pushes flags 25 (1, 8 and 16) raw, pops them
  on exit, and reads its startup replies itself before its input reader
  starts, turning keys typed meanwhile back into events
  (~/workspace2/textual-rs/src/driver/platform/posix.rs:76-90,
  ~/workspace2/textual-rs/src/driver/live.rs,
  ~/workspace2/textual-rs/src/driver/typeahead.rs).

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

[INP-007] On a terminal that reports the Kitty keyboard protocol, the default backend MUST push the flags disambiguate (1), report all keys as escape codes (8) and report associated text (16) before its first frame and pop them on every exit and before a suspend; every key MUST then reach the App as one press event that keeps its key and its Shift, Ctrl, Alt and Super modifiers, and a key that types text MUST carry the text the terminal reports for it. On a terminal that does not report the protocol it MUST push nothing.
Falsifier: In a pseudo-terminal that answers the keyboard-protocol query, the app writes no `ESC[>25u` before its first frame, or after a normal exit, an error exit or a panic the last keyboard-flag write is not a pop; `ESC[13;2u` (Shift+Enter), `ESC[104;5u` (Ctrl+H), `ESC[105;5u` (Ctrl+I) or `ESC[91;5u` (Ctrl+[) reaches the App as Enter, Backspace, Tab or Escape or with other modifiers; `ESC[49;2;33u` (Shift+1 typing `!`) reaches it as anything but `!`; or, in a pseudo-terminal that answers only the device-attributes query, the app writes `ESC[>`.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-008] The default backend MUST deliver Caps Lock, Num Lock and Scroll Lock and the play, pause, play-pause, stop, next and previous media keys as their own `KeyCode`, and MUST deliver no event for a modifier key (Shift, Ctrl, Alt, Super, Hyper or Meta) pressed alone.
Falsifier: In a pseudo-terminal with the Kitty protocol on, the Kitty codes for those nine keys reach the App as `KeyCode::Unknown` or another key, or a Kitty code for a modifier key alone reaches the App as any event.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-009] The terminal's own focus reports MUST reach the root component as a focus event and no element's handlers, and the focused element MUST keep its focus while the terminal's window is unfocused and after it is focused again.
Falsifier: In a pseudo-terminal, `ESC[O` then `ESC[I` sent while an element has focus reach that element's handlers, reach no root handler, or leave a different element focused, or none.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-010] On Unix, Ctrl+Z that no handler consumes MUST suspend the App: the backend leaves the terminal as it found it, as on exit, and the process stops; when the process continues, the backend enters again as at start and paints the whole frame, and keys typed after that reach the App.
Falsifier: In a pseudo-terminal, Ctrl+Z sent to an app whose handlers ignore it leaves the process running, stops it with a mode, the keyboard flags, the alternate screen or raw mode still on, or after SIGCONT leaves one of them off, paints only part of the frame, or loses the next key.
Mechanism: input-pty
Status: Agreed 2026-09-27

[INP-011] Before its first frame the default backend MUST ask the terminal for its keyboard protocol and its background color and end with a device-attributes query, and MUST wait for the replies at most 200 ms; no reply byte MAY reach the App as an event, every key typed meanwhile MUST reach it in order, and when the application has set no theme the backend MUST make the light preset active for a background whose relative luminance is above 0.5 and keep the dark preset otherwise.
Falsifier: In a pseudo-terminal that answers the background query with `rgb:ffff/ffff/ffff`, the first frame is painted with the dark preset; with `rgb:0000/0000/0000`, or with an application theme set, the active theme changes; a reply byte reaches the App as a key; keys sent before the replies arrive late, out of order or not at all; or, with no reply at all, the first frame comes more than 400 ms after the queries were written.
Mechanism: input-pty
Status: Agreed 2026-09-27
