# Renderer commitment review

commit: a4f610bd9a33f0aea8ede14a140052460e5248fd
findings:
  - closed: RND-001 complete frames enter through App before legacy patches; exact styled cells, grid, layers, replacement, removal, and unchanged frames are asserted.
  - closed: RND-002 text bypasses the char Surface; grapheme widths, combining sequences, continuation cells, and ancestor clipping are verified without emitting application control characters.
  - closed: RND-003 resize rebuilds worker buffers and layout with a forced frame; zero dimensions retain the last valid size; the resized VT screen matches a fresh render.
  - closed: RND-004 frame publication flushes and preserves I/O errors; retries retain the staged Element and force repaint; cancellation precedes output even when resize replaces a failed sink.
  - closed: RND-005 terminal output ownership survives renderer replacement; shutdown joins the worker and attempts raw-mode restoration even when output cleanup errors; PTY normal/error/panic paths restore exact termios.
  - closed: RND-006 root input fallback is additive and only receives unhandled events; the PTY drives counter changes, resize, Escape, and Ctrl+C through App.
  - closed: Backend remains Send + Sync through worker ownership, with no unsafe implementations or unbounded command queue; construction validates frame allocation limits.
  - closed: existing Backend and RootComponent implementations keep default behavior; dependency revision and run instructions are documented; baseline failures and direct-host verification limits are explicit.

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

Completed 2026-09-07 during implementation:

- The first `cargo test --locked --test suprtui_renderer` failed on the missing
  SuprTuiBackend API. The implemented adapter subsequently passed the suite.
- Concrete VT screen assertions caught a positioned parent covering its text
  child, and an explicit black background being mistaken for an unspecified
  background. Both failing cases became regressions and passed after repair.
- A temporary mutation replaced the frame flush with `Ok(())` in
  `src/backend/suprtui/output.rs`. Running the RND-004 test failed at the
  nonzero-flush assertion. The original code was restored, then the complete
  renderer runner passed.
- A temporary mutation omitted `disable_raw_mode` from RawMode::restore.
  The example rebuilt successfully, but `python3 scripts/check-renderer-pty.py`
  failed with `termios was not restored exactly`. The original code was
  restored, then all four PTY cases passed through the complete runner.
- Controlled writers return both BrokenPipe after a partial write and a flush
  error. Tests require the original error to reach the caller, then compare a
  successful retry with a fresh render using an independent VT parser.

No mutation remains in the candidate. Tests also assert exact RGB values,
bold text, shortening/removal, unchanged ASCII and Unicode output, grid and
layer positions, ancestor clipping, wide continuations, and actual resized
screen parity. The PTY probe checks live raw mode, input-driven counter
updates, shrinking/growing, both quit keys, input error, and Rust panic.
These probes exercise the Unix host interface; they do not prove real-host
font shaping or compatibility across every terminal.

## Final review

Reviewed the committed candidate on 2026-09-07 without changing product code.
Attacked ownership at setup failure, worker disconnect, resize, explicit
shutdown, and application unwind. Inspected that shutdown evaluates output,
join, and raw-mode cleanup separately before combining results, so an earlier
error cannot skip raw-mode restoration. The session is separate from frame
buffers and remains armed after partial setup failure.

Checked the complete-frame branch against the old patch branch and confirmed
that new trait methods have defaults. Inspected clipping before unsigned
coordinate conversion, grapheme-width checks, text-control filtering, forced
repaint after any failed presentation, and explicit background detection.
Reviewed the independent terminal parser assertions and both mechanism
mutations recorded above. No new finding remains open within this commitment.

Self-audit against the production coding rules: the changes stay inside the
agreed renderer milestone, preserve public trait compatibility, report I/O
errors, bound buffering and frame dimensions, document operation and limits,
and have reproducible runtime evidence. The recorded legacy failures and
performance-test stall prevent a whole-library production-readiness claim;
they do not invalidate the scoped renderer checks. Real Kitty, GNOME Terminal,
and Ghostty visual checks remain unverified.

## Regression and static checks

Final candidate checks on 2026-09-07:

- `sh scripts/check-renderer.sh`: nine integration tests, four PTY scenarios,
  example build, and specification lint passed.
- `cargo test --locked --no-fail-fast -- --test-threads=1`: 975 passed,
  six failed, 35 ignored. The failures match recon: animation JumpStart,
  combined z-index utilities, and four accordion doctests.
- A parallel full-suite run stalled in the two existing performance-metrics
  tests. After stopping that test process, the other targets completed with
  the same six assertion/doctest failures. Both stalled tests passed serially;
  the intermittent stall is captured in the backlog. It is not reported as a
  passing parallel suite.
- `cargo clippy --locked --lib --test suprtui_renderer --example suprtui_counter`:
  completed with 49 existing diagnostics, none in the added renderer modules,
  example, or tests. This is not a clean project-wide `-D warnings` result.
- `rustfmt --check` on all five new Rust files and `git diff --check` passed.
  The repository-wide formatting baseline remains recorded in recon.
- Ripwire edit checks for SuprTuiBackend and paint_frame reported new symbols
  with no incompatible callers. The build_nodes edit check reported an
  unchanged contract and no incompatible callers.
- Ripwire `--exclude=reference/ --quality-delta` exited 2; `--test-gate`
  exited 4. These aggregate static gates did not pass. Reviewed findings:
  App::run gains the guarded root-event fallback (reported complexity 27 to
  31); the painter and worker contain explicit clipping and error paths;
  constructor/Drop similarities are lifecycle boilerplate, not copied domain
  logic. Trait methods and Drop are reported as unused despite runtime tests.
  The test gate cannot connect the external Python PTY probe to new/poll_event.
  Other listed paths belong to unchanged legacy rendering, transitions, and
  FFI. Unfiltered runs also include local reference source trees. Compiler,
  screen assertions, and PTY execution supply the relevant runtime evidence.
