# Embedded terminal review

## Specification review

The checks must prove that the child owns a real terminal, not just that a
process printed text. Host output must be interpreted independently from
Ghostty's snapshot. The input probe must drive commands through App, and
delayed output must render without keys. Cleanup checks must observe child
exit and host termios, not merely cleanup escape strings. Failure and mutation
cases will be recorded before completion. No direct-host compatibility claim
follows from changing TERM in a pseudo-terminal.

## Mechanism demonstrations

Pending.

## Final review

Pending.
