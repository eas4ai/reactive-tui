# Mechanism: pre-release-terminal-panic-restoration

command: python3 -B scripts/check-pre-release-terminal-lifecycle.py TRL-001
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/lib.rs
  - src/app.rs
  - src/backend
  - src/platform
  - scripts/check-pre-release-terminal-lifecycle.py
  - .ripwire_quality_acks
requirements:
  - TRL-001

The check MUST build a focused probe and run it as a child attached to a real
PTY. One case MUST panic in the render worker after it has enabled terminal
modes. A second case MUST panic on the App thread after the backend has enabled
those modes. Each case MUST finish within a fixed deadline and leave the PTY's
termios flags equal to their pre-run values.

Captured output MUST show that every enabled mode is disabled, including the
alternate screen and hidden cursor, before the panic marker is printed. The
probe MUST reach process shutdown without hanging. Before the real probes, the
validator MUST reject safe fixtures with missing restoration bytes, raw termios
left enabled, and panic text emitted before restoration. Merely inspecting
source text or calling shutdown on a non-terminal writer cannot pass.
