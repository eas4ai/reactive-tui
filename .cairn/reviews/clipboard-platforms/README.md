# Native clipboard evidence

The clipboard producer and verifier use this retained Cairn record directory.
Each backend needs its JSON record, round-trip output and process-cleanup output.
The verifier requires current committed source hashes and matching output hashes.
Historical passes do not replace fresh evidence after an input changes.

Linux uses private Wayland/X11 desktops. Windows and macOS require disposable
test desktops because the round-trip probe replaces their clipboard. Synthetic
validator records live only in temporary unit-test directories, never here.

Generate evidence with `python3 -B scripts/check-clipboard-platforms.py --backend <backend>`.
Add `--dedicated-desktop` only on a disposable Windows/macOS desktop.
Run `python3 -B scripts/check-clipboard-platforms.py --verify` after all five real
backend records have been generated and committed. Do not run hosted CI without
separate approval.
