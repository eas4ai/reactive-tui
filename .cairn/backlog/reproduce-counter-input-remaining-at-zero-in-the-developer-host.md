# Reproduce counter input remaining at zero in the developer host

Surfaced from: RND-006
Captured: 2026-09-07T22:28:38.392Z

Developer reported Space leaves Count at zero while Escape or Ctrl+C quits. Captured screen probes showed 0 to 1 to 2 to 1 for Space, Space, minus; controlling-PTY probes also handled ASCII and CSI-u input. The host terminal name was not supplied and the developer deferred further investigation. Keep the issue open; shell-input probes in the next commitment must not be claimed to fix this unobserved host-specific failure.
