# Backlog execution plan — post-alpha hardening

## Goal

Work through the 12-item audit backlog in an order where each phase makes
the next one trustworthy: fix real bugs first, make the tests hermetic
second, unblock release third, with screen-reader evidence alongside
throughout because it protects the headline claim.

## Success Criteria

- One canonical hex color parser; all five call sites delegate to it, and
  malformed input returns an error instead of panicking or silently
  producing black.
- No `unwrap`/`expect` on locks or constructors on the app hot path;
  poisoning and bad input surface as `ReactiveError`.
- `cargo test` passes on a headless machine with no GPU and no display.
- Every render-behavior test asserts against a golden or it is marked
  explicitly as a smoke test.
- A documented, user-approved path exists for the crates.io 0.1.0 publish
  (path dependencies resolved, syntect chain re-verdict recorded).
- Screen-reader coverage includes at least one asserted end-to-end path
  beyond style checks, and the `node.rs` TODO count goes down, not up.

## Context And Current Facts

- Audit (2026-09-19 Vollmer) produced 12 ranked items; this plan executes all 12.
- `hex_to_rgba` in `src/syntax/theme.rs:145` slices `hex[0..2]` (panics on
  short input) and falls back to silent zero; four sibling implementations
  exist in `src/theme/ansi.rs`, `src/layout/colors.rs`,
  `src/layout/css/colors.rs`, `src/layout/css/cache.rs` (all files verified
  present).
- Lock `unwrap`s confirmed on the hot path: `src/graphics/animation.rs`
  (lines 165, 167, 182, 205, 219, 224, 243) and `src/component/runtime.rs`
  (lines 88, 133, 275, 320).
- `tests/wgpu_graphics.rs` asserts `info.is_hardware`; fails on any
  headless/CI machine without a GPU.
- `tests/production_readiness_test.rs:14` serializes the file on a global
  `TEST_MUTEX`.
- Path dependencies in `Cargo.toml`: `reactive-tui-crossterm`,
  `reactive-tui-macros`, `suprtui`, `reactive-tui-libghostty-vt`.
- Terminal compatibility matrix and parser-replay harness landed in
  `cec054a6`; screen-reader guarantee is narrowed to the Orca + GNOME
  Terminal pair by decision record.

## Constraints And Non-goals

- No behavior change without a test asserting the new behavior first.
- No rewriting existing test assertions to fit a change; failing tests are
  a contract to satisfy or an explicit user-approved contract change.
- `main` stays green: each unit is committed and pushed separately, in
  plan order (Phase 1 → 2 → 3, screen-reader track alongside).
- Non-goals: new widgets, new features, perf optimization beyond what
  correctness fixes unlock, Revue competitive analysis.

## Key Decisions

- **One hex parser, strict.** New canonical parser returns `Result`;
  malformed input is an error, never silent black. Rejected: keeping
  per-module parsers with a shared test (drift already happened twice).
- **Poison is an error, not a panic.** Lock failures map to
  `ReactiveError`. Rejected: `unwrap_or_else(into_inner)` (hides poisoning).
- **GPU tests gate, not skip-silently.** Hardware-dependent tests detect
  the adapter and either run against a mock or fail loudly with the reason;
  `#[ignore]` without a tracking note is banned.
- **Goldens over print-only.** Render tests assert against checked-in
  golden output; anything that cannot is labeled `smoke_` and excluded
  from the correctness gate by name, not by silence.
- **Publish path decided by user (decided 2026-09-20): vendor.** The
  forked crates stay in-repo under `crates/` as path-only workspace
  members; no companion publishes on crates.io. Phase 3 is re-scoped
  around this call (see Phase 3 and Open Question 1).

## Recommended Approach

Sequential phases with per-phase commits; the screen-reader track runs
alongside because it is independent and claim-critical. Smallest viable
proof per unit: the failing-before test, then the fix.

## Work Plan

### Phase 1 — Correctness (units 1.1–1.3)

- **1.1 Hex parser unification [M].** Add canonical fallible parser
  (home: `src/layout/css/colors.rs` or a new `src/color.rs`; implementer
  chooses by fit). Migrate the four siblings to delegate. Add
  short/malformed/garbage unit cases. Files: the five color files above.
- **1.2 Hot-path poison [M].** Convert lock `unwrap`/`expect` in
  `src/graphics/animation.rs` and `src/component/runtime.rs` to
  `ReactiveError`. Add a poisoning test (panic a holder, assert error not
  process death).
- **1.3 Constructor/reflow panics [S].** Same treatment for
  `src/layout/mod.rs` (`expect(root)`), `src/terminal/screen/reflow.rs`,
  `src/animation/keyframes.rs` constructors.

### Phase 2 — Test hermeticity (units 2.1–2.5)

- **2.1 GPU gate [S].** `tests/wgpu_graphics.rs`: detect adapter, run
  against mock or fail with reason. Must pass on this headless box.
- **2.2 Goldens [S].** `tests/test_flex_layout.rs`,
  `tests/syntax_highlighting_integration.rs`,
  `tests/render_ops_snapshots.rs`: assert against checked-in goldens or
  rename to `smoke_` and document the exclusion.
- **2.3 Timing suites [M].** `tests/embedded_terminal.rs`,
  `tests/tokio_event_loop_lifecycle.rs`, `tests/api_clipboard.rs`,
  `tests/clipboard_platform.rs`: replace sleeps with event-driven waits;
  deleted-or-justified SIGKILL usage; un-ignore or delete the dead test.
- **2.4 Harness dedupe [S].** Promote `tests/support/app_input.rs` to a
  shared test target; remove the 8+ `#[path]` inclusions.
- **2.5 De-serialize [M].** Remove `TEST_MUTEX` from
  `tests/production_readiness_test.rs` by eliminating the shared global
  state it protects.

### Phase 3 — Release (units 3.1–3.2; path decision recorded, vetoed publish)

- **3.1 Vendored layout [L] (done 2026-09-20).** Companions carry
  `publish = false` in all five manifests; `scripts/check-crates-release.py`
  enforces the veto plus exact workspace membership, version pins, and
  archive boundaries. Proof: checker logic green on all six crates plus
  `cargo package --list` per crate; durable guard in
  `tests/crates_vendored.rs`. (Checker runs clean only on a committed tree;
  its `cargo package --list` dirty-guard predates this work.)
- **3.2 Syntect chain [M] (done 2026-09-20 via removal).** Swapped
  syntect 5.3.0 for lumis 0.13.1 instead of re-verdicting: bincode 1.3.3
  and yaml-rust 0.4.5 left the locked graph, their
  `dependency-maintenance.toml` exceptions were dropped, and DQC-002
  (including `cargo audit`) passes. Decision:
  `docs/decisions/highlight-with-lumis-tree-sitter-backend.md`.

### Track A — Screen-reader evidence (alongside, claim-critical)

- **A.1** Assert one end-to-end reader path beyond style checks
  (label → role → focus announcement) using the existing orca probes.
- **A.2** Close `node.rs` TODOs in priority order (focus/selection, rich
  text, unnamed forms first); each closure carries a probe assertion.

### Track B — API tidy (fill-in work)

- **B.1** `size`/`get_size` + focus-split naming, `meta()`, missing doc
  examples, the `None`-id transition gap (`src/app.rs`,
  `src/event/types.rs`).

## Validation Plan

- Per unit: the new/updated test fails before the fix (`git stash` check
  on request) and passes after; `cargo test --test <name>` quoted per unit.
- Per phase: full `cargo test`, `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --check` — all green, quoted in the report.
- Phase 2 proof: full suite green on this headless box (no GPU, no
  display); GPU-gated tests report their skip reason visibly.
- Phase 3 proof: reworked release checker green plus `cargo package --list` per crate (vendored, no publishes).
- Track A proof: orca probe assertions passing, TODO count reduced
  (count quoted before/after).

## Risks / Rollback

- Hex unification touches five modules: behavior snapshots (goldens from
  2.2, or before/after `.cast` captures) guard against silent color
  shifts. Rollback: revert the unit commit; units are independent.
- Poison-as-error changes public error surfaces: semver-note in the
  commit message; no silent signature changes.
- Goldens can ossify: goldens live next to their tests and regenerate via
  a documented `REGENERATE=1` run, reviewed as a diff like any other.
- Biggest validation risk: Phase 2 green-on-headless is the gate the
  whole launch story rests on; if 2.1/2.3 resist hermeticity, say so and
  re-scope rather than weakening assertions.

## Open Questions

1. **Publish path (decided 2026-09-20, vetoed publish):** vendor the
   forked crates in-repo under `crates/`; no companion crates.io
   publishes. The embedded-terminal story stays intact via path
   dependencies.
2. **Syntect verdict:** accept an upgrade PR if clean, else who signs the
   fresh trusted-dumps-only verdict and with what expiry?
