# Mechanism: pre-release-terminal-input-safety

command: python3 -B scripts/check-pre-release-terminal-input-safety.py TRL-003
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/lib.rs
  - src/core/surface.rs
  - src/core/terminal.rs
  - src/event
  - src/platform/parser.rs
  - src/terminal
  - src/widgets/terminal.rs
  - scripts/check-pre-release-terminal-input-safety.py
requirements:
  - TRL-003

The check MUST use `syn` to inventory every public `Surface` and
`SurfaceSubview` method that accepts string data, plus every public title
writer or title-sequence constructor. The inventory MUST fail when a path is
added, removed, or changes signature without a matching behavioral case.

Property tests MUST cover C0 controls, DEL, Unicode C1 controls, ESC followed
by CSI and OSC payloads, and ordinary Unicode. They MUST exercise every
inventoried path, render Surface content through `DiffWriter`, and reject a
case if attacker-provided control data survives in stored graphemes or emitted
payload bytes. Title tests MUST inspect returned or captured bytes and child
titles. Parser tests MUST feed BEL- and ST-terminated OSC 52 responses with
valid clipboard data and prove that no `Paste` event is emitted. The validator
MUST first reject safe fixtures for a missing writer, raw control output, and
an unsolicited paste event.
