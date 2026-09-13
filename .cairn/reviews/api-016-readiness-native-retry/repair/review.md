# Darwin terminal verification repair

The second native run failed before Cargo: its real controlling-PTY guard exited
cleanly but Darwin revoked the parent slave descriptor, making tcgetattr return
ENOTTY. Preserved native metadata identifies snapshot 17a88b15d79ca62530965265a27fbd16e74cc3b2;
its stable/committed flags and captured log SHA-256 were verified before copying.

Apple XNU tty_dev.c ptyioctl delegates TIOCGETA on either endpoint to the shared
tty. tty.c ttioctl_locked copies t_termios, and ttyclose does not reset it.
tty_ptmx.c retains the tty until both endpoints close. Therefore only Darwin
slave ENOTTY falls back to the surviving master; all other errors propagate and
full termios equality remains mandatory. A real controlling child deliberately
leaving raw mode active must fail, paired with the restored child. All five
harness guards passed locally; native proof remains pending.

Sources: https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/tty_dev.c
https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/tty.c
https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/tty_ptmx.c

A separate portability flaw was found before reaching the burst on macOS: XNU
TTYHOG is 1024 and ptcwrite can legally short-write a 2048-byte burst. Preserve
the strict single-write 2048-byte Linux falsifier. Other Unix PTYs record their
initial queued count and send the remainder with bounded nonblocking writes.
Every consumer still requires 2048 events, exhausted zero-timeout polls and
restoration; three dependency tests retain multi-buffer readiness checks.
Prepared controls reject a short Linux write and accept a simulated Darwin
short write plus EAGAIN without dropping bytes. These simulations are not
native macOS evidence. Raw controls are preserved alongside this review.

Focused Ripwire edit-check passed after rejecting an ambiguous unqualified
symbol. quality-delta exited 2 on broad preexisting/reference findings;
test-gate exited 4. Neither is counted as a pass. Rust consumer rustfmt passed.
Full local entry-point verification passed: five harness guards, three dependency
readiness tests, five legacy cases, eight Rust App routes, three manual routes,
the strict Linux 2048-byte queued consumer and four C routes.

Self-audit: changes are bounded to the evidence harness and its documented
transport assumptions. Production terminal code is untouched. No platform
acceptance is claimed until fresh native checks pass.
