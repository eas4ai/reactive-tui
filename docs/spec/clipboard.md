# Clipboard

Prefix: CLP

The clipboard hook (src/hooks/clipboard.rs) copies and pastes by running a
local command: wl-copy and wl-paste under Wayland, xsel or xclip under X11,
pbcopy and pbpaste on macOS, PowerShell on Windows (`ClipboardBackend::detect`),
and reports Unavailable when none is found. The text input keeps the text it
copies or cuts in a buffer of its own and pastes from that buffer
(src/widgets/input/text_input.rs:394, 696-722). Read on 2026-10-09: nothing
writes OSC 52, the sequence by which an application asks the terminal itself
to set the clipboard, so a copy made inside an application reached over SSH
lands on the remote host, or nowhere, and never on the user's machine; the
terminal, which runs on the user's machine, is the one party that can always
reach their clipboard. The default backend writes its bytes on its output
worker (src/backend/suprtui/output.rs), the direct TTY backend through its
transport (src/backend/direct_tty.rs), and the debug backend paints into
memory (src/backend/mod.rs:667) and keeps no record of terminal sequences.
gpui-kit's clipboard button writes through the window system
(docs/widget-study.md, change 6); a terminal application has the terminal.

## Observed

(none yet)

## Draft

[CLP-001] A copy requested through the clipboard hook's writer (`use_clipboard`, `use_simple_clipboard`), and a text input's copy (Ctrl+C) and cut (Ctrl+X) of a selection, MUST reach the terminal's clipboard: the App MUST write `ESC ] 52 ; c ; <the text's UTF-8 bytes in base64> ESC \` to its terminal before its next frame, on the default backend and on the direct TTY backend alike and whatever local command is available, so that a copy over SSH lands on the user's machine; and when a local command is available (wl-copy, xsel, xclip, pbcopy or PowerShell) the copy MUST run it as well, so that it keeps working in a terminal that ignores OSC 52; a paste MUST keep coming from the terminal's bracketed paste or from the local command; the debug backend MUST record the terminal sequences an App writes, so a test can read them; and no OSC 52 sequence MAY be written without a copy.
Falsifier: On the debug backend, an App whose text input holds a selected `hello` receives Ctrl+C and the recorded output holds no `\x1b]52;c;aGVsbG8=\x1b\\`; a component that calls the clipboard writer with `hello` produces none; with a local copy command on the PATH, a copy through the hook or a text input does not run it; a text input's paste after that cut inserts nothing; or a frame presented with no copy requested carries an OSC 52 sequence.
Mechanism: widget-behavior
Rationale: The hook's local commands and the text input's private buffer never reach the user's machine over SSH; OSC 52 does, through the terminal (docs/widget-study.md, change 6); Terminal.app, iTerm2 by default and tmux without its clipboard option ignore OSC 52, so the local command stays beside it.
Status: Agreed 2026-10-09
