# Renderer commitment review

## Specification review

Checked 2026-09-07 before implementation. Challenged whether comparing output
with itself could prove rendering; the frame tests will assert concrete cells
through an independent VT parser as well as compare unchanged byte counts.
Challenged Unicode loss in the old Surface bridge; the new path receives full
text clusters. Challenged successful writes without flush and failure after a
partial write; controlled writers will verify these paths. Challenged cleanup
claims based only on strings; the PTY probe will inspect termios too. Rust
unwinding is included; aborts and external SIGKILL cannot run destructors.

No rule claims that SuprTUI's initial test suite proves host compatibility.
No rule declares legacy FFI, animation, or CSS bugs fixed by this commitment.

## Mechanism demonstrations

Pending implementation and controlled violating/corrected cases.

## Final review

Pending.
