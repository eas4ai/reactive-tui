# Bound live clipboard subprocess operations

Surfaced from: DFT-004
Captured: 2026-09-08T15:47:14.076Z

During ffi-lint-format-repair, two default-suite runs stalled for 300 seconds in the live wl-copy children of clipboard hook tests. ClipboardBackend copy and paste wait on external desktop commands without per-operation deadlines and do not consistently check child exit status. Add bounded production copy/paste execution, cleanup and actionable failure propagation with stalled/nonzero-command tests; verify actual desktop integration separately. The maintenance unit fixtures remove live-desktop coupling but do not repair this product limitation.
