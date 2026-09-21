# Terminal compatibility

People try the demo before they read anything. A framework that draws
garbage on a default desktop terminal loses at first impression. This file
is the launch-facing record of what was actually tested, where.

## Status rule

A cell says **Verified** only with a linked evidence record produced from a
real terminal or a pinned host build. Everything else is **Reported**
(someone saw it, no capture), **Known-divergent** (a real bug was found
here), or **Untested**. Parser-model results (`tests/terminal_parser_replay.rs`)
are a backstop for byte/sequence bugs, never a substitute for a terminal
column: parsers cannot see font glyph coverage.

## Matrix

| Terminal | Rendering | Images | Keyboard | Screen reader | Notes |
|---|---|---|---|---|---|
| Kitty (isolated build 0.45.0 + patch) | Verified | Verified (Kitty graphics) | Untested | Unverified | Image acceptance ran on the patched host only. Stock Kitty installs are explicitly not covered. See `docs/decisions/repair-and-verify-a-pinned-isolated-kitty-host-for-image-acceptance.md`, `scripts/kitty-host`. |
| iTerm2 3.7 | Verified (limited) | Verified (inline images, geometry only) | Untested | Unverified | Dominant-channel classification only. Exact sRGB and transparency are an approved non-goal on this host; fresh native acceptance pending. See `docs/decisions/verify-iterm-3-7-image-geometry-while-recording-its-approved-color-and-transparency-limit.md`, `scripts/check-iterm-host.py`. |
| GNOME Terminal | Verified | Untested | Untested | Verified (Orca pair) | Screen-reader guarantee is narrowed to the Orca + GNOME Terminal pair. All other pairs unverified. See `docs/decisions/verify-terminal-screen-reader-support-with-orca-in-gnome-terminal.md`, `tests/api_widget_behavior/orca_*.py`. |
| Windows Terminal (ConPTY) | Verified (session lifecycle) | Untested | Verified (input, resize) | Unverified | Pinned runtime Microsoft.Windows.Console.ConPTY 1.24.260710001 must ship with the app; no fallback to the OS pseudoconsole. See `docs/decisions/use-the-pinned-microsoft-conpty-runtime-for-windows-terminal-sessions.md`, `tests/api_widget_behavior/conpty_probe.rs`. |
| WezTerm | Partial | Partial (inline color acceptance) | Untested | Unverified | Inline color acceptance kept green per the iTerm2 decision record; dedicated captures still needed. |
| Konsole (Konsole-lineage) | Known-divergent | Untested | Untested | Unverified | CAN 0x18 frame boundary printed a visible glyph here; fixed via ESC ST (`src/backend/suprtui/output.rs`). Braille-blank font coverage unverified — report sightings. |
| Ghostty | Reported | Untested | Untested | Unverified | Developer daily use; VT parser pinned (`crates/libghostty-vt`, Uzaaft/libghostty-rs 5988a0b78b4aa804d1c12e66bbfe662bd97d81c0) and used as a replay model. No formal capture yet. |
| Foot, Alacritty, xterm, Terminal.app, mintty | Untested | Untested | Untested | Unverified | — |

## What the replay harness proves

`tests/terminal_parser_replay.rs` feeds identical byte streams through the
vt100 model (always) and Ghostty's real VT parser (`embedded-terminal`
feature) and asserts: synchronized-update envelopes stay silent, Kitty APC
payloads never leak into cells, Braille codepoints round-trip byte-identical,
and — the load-bearing one — that the models *disagree* on a bare CAN byte
(Ghostty surfaces it, vt100 drops it), which is why the renderer emits ESC
ST boundaries instead of CAN.

## Extending this matrix

1. Drive the demo with `scripts/record-demo.py`; it replays scripted keys
   through each showcase example in a PTY and writes one `.cast` file.
2. Render with a Braille-capable font so blank cells stay blank:
   `agg --font-family "DejaVu Sans Mono" demo.cast demo.gif`.
   The default agg font renders U+2800 as dotted tofu, which looks like a
   framework bug and is not one.
3. Add the terminal row above with status, scope, limits, and evidence
   links. Informal sightings go in as Reported, never Verified.
