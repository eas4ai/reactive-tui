commit: da046fe3c8fdee3c39a083ea71fe15115e0dbf34
examined:
  - Ghostty cell extraction, CellFrame validation, native occupancy, and host painting.
  - Shared shell launcher, feature gating, documentation, and rebuilt binary.
  - Unicode edge/resize tests, embedded integration tests, and controlling-PTY regressions.
findings:
  - resolved: EMB-002 Native UTF-8 C1 text caused the reproduced cell-95 failure. The snapshot adapter now replaces retained control text with U+FFFD while keeping strict frame validation and native occupancy.

## Authorization and scope

The developer explicitly requested this repair before answering LOOP-087.
That escalation remains unanswered. This work repairs the named rendering error;
it does not choose an unrelated next commitment or claim the Kitty segfault is
fixed. The existing shell binary was stale, so embedded_shell is now a buildable
launcher that includes the existing runtime-probe implementation without copying
its logic. Both launcher targets build without the duplicate-target warning.

## Failure demonstration

A child writes hex 1b5b313b393648c285 in a 96-column host PTY: move to column 96,
then emit UTF-8 U+0085. Before repair, the host exits 1 with the exact reported
invalid cell frame grapheme or occupancy at cell 95 error. The new native
Unicode/edge/resize test also fails on that character before repair. After
repair it passes. The reconstructed matching failure path is known; the developer
has not yet supplied the steps that triggered their particular occurrence.

The persistent PTY regression verifies a painted replacement, absence of UTF-8
U+0085 in host output, exit zero, and restored termios, cursor, and alternate
screen. The same case passes on the rebuilt embedded_shell binary. Ordinary
ANSI controls still act on the child interpreter; the conversion changes only
control characters retained as cell text. CellFrame still rejects control bytes,
multiple graphemes, orphan tails, invalid dimensions, and invalid cursors.

## Executed verification

- Ten embedded-terminal integration tests pass, including host frame acceptance.
- The Unicode edge/resize test passes over five widths and eighteen text cases.
- Both launcher targets build with the locked embedded-terminal feature.
- Three PTY checks pass: interactive input/background redraw/resize/Ctrl+C/Ctrl+Q
  cleanup, bounded worker failure cleanup, and the cell-95 control-text regression.
- The two original PTY checks and the exact regression also pass on embedded_shell.
- Renderer, paint, event routing, documentation, and catalog integration suites
  pass: 64 tests total.
- Formatting and diff whitespace checks pass.
- Package-only no-dependency Clippy passes with the previously recorded wizard
  collapsible-else-if lint allowed. This is not unrestricted strict lint health.

The unrestricted feature-enabled library suite aborts in the unchanged deep
render-tree test's 128 KiB thread. Its isolated run aborts even when the snapshot
fix is removed. With that one test explicitly skipped, 1053 tests pass, eight
are ignored, and one is filtered out. This independent failure is captured in
the backlog, not hidden by weakening its assertion or enlarging its stack.

## Tool checks and remaining limits

GitNexus capture impact reports one direct caller, publish, and two upstream
worker functions at low risk. The final staged diff reports six affected native
snapshot flows at high risk; that warning was presented before committing, and
the native, host-paint, and PTY tests cover the changed conversion boundary.
Rust-analyzer locates CellFrame. Its workspace reload did not complete promptly
and was stopped; no analyzer diagnostics sweep is claimed.

Ripwire's edit-check reports a parameter change after folding seven unrelated
capture definitions, although the Rust capture signature is unchanged and its
caller still compiles. Quality-delta exits 2 with complexity/nesting rows assigned
to an unchanged Python scratch capture, test symbols marked dead, and ignored
generated GitNexus metadata. Test-gate exits 4 with a broad name/documentation
reachability list; the relevant native, host-rendering, and catalog suites were
executed, not every broadly named target. Neither graph gate is reported passing.

Cairn refuses refreshed CAT-001 evidence because the shell manual chapter is
outside the catalog commitment's declared footprint. No receipt was fabricated,
no historical receipt was edited, and no Cairn Done verdict is claimed for this
separately authorized repair. The outstanding scope and promotion decisions are
not silently answered. The directly executed tests above remain distinct from
Cairn acceptance evidence.

The release self-audit considered the production rules. The repair is local to
the adapter, preserves the public API and validators, adds no dependencies or
secrets, keeps bounded cell allocation, and verifies terminal cleanup. No
persistence, auth, or migration is introduced. No executable code changed while
this review was recorded. The reproduced control-text defect needs no further
in-scope revision; host-specific segfault and deep-tree stack work remain separate.
