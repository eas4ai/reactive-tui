# Mechanism: pre-release-terminal-input-safety

command: python3 -B scripts/check-pre-release-terminal-input-safety.py TRL-003
inputs:
  - Cargo.toml
  - Cargo.lock
  - src
  - tests/pre_release_terminal_input_safety.rs
  - scripts/check-pre-release-terminal-input-safety.py
requirements:
  - TRL-003

The check MUST use `syn` to inventory every public `Surface` and
`SurfaceSubview` string writer, plus every public title writer or
title-sequence constructor. The inventory MUST fail when a path is added,
removed, or changes signature without a matching behavioral case.

Property tests MUST cover C0 controls, DEL, Unicode C1 controls, ESC followed
by CSI and OSC payloads, and ordinary Unicode. They MUST exercise every
Surface writer and every title path that emits terminal protocol directly,
render Surface content through `DiffWriter`, and reject a case if
attacker-provided control data survives in stored graphemes or emitted payload
bytes. Builder-only title paths are protected by the shared Surface boundary
and remain in the inventory. Title tests MUST inspect returned or captured
bytes and child titles. Parser tests MUST feed BEL- and ST-terminated OSC 52
responses with valid clipboard data and prove that no `Paste` event is
emitted. The validator MUST first reject safe fixtures for a missing writer,
raw control output, and an unsolicited paste event.
