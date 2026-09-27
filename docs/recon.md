# Recon: reactive-tui

Date: 2026-09-21. HEAD: d27c99d9 (main). Treated as a project that never ran
under Cairn. Every row is Exists (seen in the tree, cited), Documented (a doc
or commit claims it, cited), Contradicted (tree and document disagree, both
cited), or Unverified (needs a build, device or run this pass did not do).
Path references are relative to the repository root. `G/` means the reference
library at /home/shawn/workspace2/gpui-kit-0.6.6/crates/component/src.

Method: five read-only reviewers (manifests and CI, source architecture,
tests, documentation claims, history) plus one chart comparison against
gpui-kit 0.6.6, all forbidden from running cargo or editing files. The
coordinator ran the repository checks listed in section 2 and spot-read the
cited lines for every Contradicted row.

## 1. What it is

- Exists: Rust workspace, root crate `reactive-tui` 1.0.0, edition 2021,
  rust-version 1.91, MIT, rlib + cdylib (Cargo.toml:2-9, 93-96). Five
  workspace members under crates/: reactive-tui-macros, reactive-tui-crossterm
  (crossterm 0.29.0 fork, Unix Mio readiness patch,
  crates/reactive-tui-crossterm/REACTIVE_TUI_PATCH.md:3-9), reactive-tui-suprtui
  (renderer from eas4ai/suprtui 7793deb, OpenTUI origin, UPSTREAM.md:3-8),
  reactive-tui-libghostty-vt and -sys (uzaaft/libghostty-rs 5988a0b, Ghostty
  pinned 22d13172). All five members are `publish = false`
  (crates/*/Cargo.toml).
- Exists: source 157,372 lines under src/ (find | wc). Largest modules:
  widgets 46,213 (117 files), layout 13,606, core 10,098, animation 8,696,
  accessibility 8,377, ffi 8,062, platform 7,581, terminal 6,606, builder 6,304,
  hooks 5,256, component 4,483 (wc per directory).
- Documented positioning: "a Rust framework for terminal applications. It
  combines retained components, reactive state, CSS-like layout, routed input,
  accessible semantics, images, dialogs, and owned terminal sessions"
  (README.md:27-29). "Applications render complete, grapheme-aware cell frames.
  The primary SuprTUI backend owns terminal setup, input, presentation, and
  restoration. A debug backend renders into memory for deterministic tests"
  (README.md:31-33). Cargo.toml:6 says the same in one line.
- Documented what it is not: the wgpu feature is an offscreen texture, not a
  window or a terminal backend, not pixel image output, no FPS guarantee
  (manual/wgpu-graphics.md:9-11, 21, 86); markdown is styled terminal text, not
  a browser layout engine (manual/text-editing-markdown-and-syntax.md);
  the C headers are not a Python binding (include/README.md:54); TypeScript
  does not replace the native library (bindings/typescript/README.md:64).

## 2. Repository checks run today (Linux, rustc 1.95.0, warm cache)

| Check | Result |
| --- | --- |
| cargo build --locked --workspace | pass, 49 s |
| cargo fmt --all -- --check | pass |
| cargo clippy --locked --workspace --all-targets -- -D warnings | FAIL, 140 errors, all in crates/libghostty-vt (must_use 76, needless_pass_by_value 15, ptr_as_ptr 3, others); src/ is clean |
| cargo doc --locked --workspace --no-deps | pass |
| cargo audit | pass, 2 allowed warnings (wgpu chain) |
| cargo deny check | FAIL: async-std unmaintained; duplicates sha2, digest, crypto-common, cpufeatures, block-buffer (0.10 vs 0.12 lines) |
| cargo publish --dry-run --no-verify | FAIL: reactive-tui-crossterm not on crates.io (expected under the source-only decision) |
| cargo test --locked --workspace --no-fail-fast | pass: 2,886 passed, 0 failed, 32 ignored, 128 s |

Contradicted: AGENTS.md:133-135 and .github/workflows/ci.yml:62-68 name
`clippy --all-targets -- -D warnings` as the lint gate, and it fails on the
workspace today. CI runs `--locked --all-targets` on the root only
(ci.yml:66-68), which is why CI can be green while the workspace command fails.
Unverified: whether the last CI run on main passed (no run logs read).

## 3. Entry points and control flow (Exists)

- `App::builder()` -> `AppBuilder`; `build()` requires a backend and a root,
  else `io::Error InvalidInput` (src/app.rs:148, 801, 921-931). Options:
  debug, performance_mode, adaptive_config, quit_key, scheduler,
  screen_reader, accessibility_name (:863-915).
- Screen reader auto-enables when the backend is an interactive terminal and
  DBUS_SESSION_BUS_ADDRESS is non-empty; explicit `screen_reader(true)` makes
  connection failures fatal; `Some(true)` on non-Linux is a build error
  (:833-856, 943-947).
- `App::run` enters the hook scope, wraps the loop in catch_unwind, calls
  `backend.shutdown_after_panic` on panic and re-raises (:153-166). On Unix,
  termination signals route to the waker (:169-171). Shutdown closes
  updaters, AT-SPI, waker, performance, then the backend (:172-203).
- Loop per iteration (:219-286): wake flags -> updaters -> performance mode
  -> AT-SPI actions -> scheduler updates -> ready timers -> animation tick ->
  render when dirty and a frame is due -> `root.update()` (Unchanged, Redraw,
  Exit) -> deadline -> `poll_event_with_wake` or `wake.wait`.
- Input (:286-350): quit chord, resize (drops the RenderTree, :709-733),
  mouse to component runtime, router, root fallback, default Ctrl+C or Escape
  only when no quit key is set, then widget CustomEvent delivery.
- Render (:573-690): a root that returns a `CellFrame` goes straight to
  `render_cells` + `present`; otherwise `components.resolve(root.render())`,
  style, `backend.render_frame` (complete frame), with a reconciler diff
  fallback; then event-tree sync and accessibility snapshot.
- Backends: SuprTuiBackend (worker thread over a rendezvous channel,
  src/backend/suprtui.rs:106, 196-200), CrosstermBackend wrapper
  (src/backend/mod.rs:183-186), DirectTtyBackend (src/backend/direct_tty.rs,
  undocumented in README and manual), DebugBackend in memory with scripted
  events (mod.rs:361-373). Backend trait: src/backend/mod.rs:43-127.

## 4. Component and reactive model (Exists)

- RootComponent (src/app.rs:50-85): `render() -> Element`, optional
  `cell_frame`, `attach_waker`, `accepts_input`, `wake_driven`, `update`,
  `resize`, `try_handle_event`.
- Component trait (src/component/mod.rs:52-125): Props, State, new,
  initial_state, update, render, try_render, poll_change, handle_event,
  layout, on_lifecycle; instances owned by ComponentRuntime
  (src/component/runtime.rs).
- Element (src/component/element.rs:143-194): Component, Text, Layout (Flex,
  Grid, Stack, Absolute), Fragment, Empty; props are `Arc<dyn Any>` compared
  by pointer (:196-200, documented trade-off); class string carries utility
  CSS.
- Macros `#[component]`, `#[derive(Props)]`, `#[ffi_export]`
  (crates/reactive-tui-macros/src/lib.rs:6-7, 72-73, 316-317).
- Hooks: use_signal, use_effect(_with_deps), use_context, use_reducer,
  use_previous, use_memo (src/reactive/hooks.rs:278-430); ThreadSafeSignal
  (:186); clipboard, fps, mouse, refs, timer, animation hooks
  (src/hooks/mod.rs:26-56). Signals: src/reactive/signal.rs:22-300.
  Scheduler (src/reactive/scheduler.rs:141, 224-319). AppWaker and Scope
  (src/reactive/wake.rs:27-74, 131-140). Updaters (src/ui/mod.rs:10-12).
- Layout: taffy TaffyTree (src/layout/mod.rs:1-35), paint tree
  (src/layout/paint_tree.rs), utility classes with optimizer and cache
  (src/layout/css/mod.rs:1-45). Theme with variables and five presets
  (src/theme/mod.rs:10-80).

## 5. Widgets (Exists, src/widgets/mod.rs:14-58)

dialog: Autocomplete, Confirmation, DialogBuilder/Component, DialogEngine,
Input, Progress, Toast, Wizard, http (curl helper). display: Chart, DataTable,
FileExplorer, Image, Modal, Popover, ProgressBar, Table, Tree, Overlay. input:
Checkbox, RadioButton, Select, Slider, TextInput. layout: Accordion,
Breadcrumb, ScrollView, Stack, Tabs. menu: ContextMenu, DialogMenu, MenuItem,
MenuBar, PopupMenu, MenuStyle. terminal: TerminalWidget.

## 6. Platform surfaces (Exists unless noted)

- Capability detection: src/core/capabilities.rs:90, 158, 468-503;
  src/platform/capabilities.rs:51, 58. Contradicted (minor): hyperlinks,
  bracketed_paste and focus_events are hard-coded true in the conversion
  (src/platform/capabilities.rs:19-24) rather than detected.
- Images: modes Auto, Sixel, Chafa, Viu, KittyGraphics, ITerm2Inline,
  AsciiArt, Fallback (src/widgets/display/image/mod.rs:87-102); host emission
  gated by ImageOutputOptions, all default false (src/backend/suprtui.rs:26-42);
  decode limits 256 MiB (decoded.rs:12, live/animation.rs:8). HTTP sources
  rejected (decoded.rs:77-78). Host behavior on Kitty, iTerm2, WezTerm,
  GNOME: Unverified. Kitty acceptance needs a patched Kitty 0.45.0 built by
  scripts/kitty-host/ (a host patch, not a repository fix).
- Clipboard: wl-copy/wl-paste, xsel, xclip, pbcopy/pbpaste, powershell
  (src/hooks/clipboard.rs:11-92). Real desktop behavior: Unverified.
- Embedded terminal: EmbeddedSession and TerminalView, Unix only, feature
  embedded-terminal (src/embedded/mod.rs:31-42, 238-260). Documented open
  defect: Kitty embedded-shell crash (README.md:228-229,
  manual/terminal-and-embedded-sessions.md:46). PseudoTerminal Unix and
  Windows ConPTY with bundled DLLs and SHA-256 manifest
  (src/terminal/pty/windows/runtime/). Windows behavior: Unverified at HEAD;
  the only evidence is gitignored docs/analysis/conpty-platform/windows.json
  from commit ff8f1e02.
- Accessibility: accesskit 0.25 nodes; Linux AT-SPI over zbus; vendored
  accesskit_unix 0.23.0 adapter with patches
  (src/accessibility/platform/UPSTREAM.md, provenance.json), undocumented in
  README and manual. Target is Orca with GNOME Terminal only
  (src/accessibility/mod.rs:3-4). Orca behavior: Unverified (needs a desktop).
- wgpu: offscreen cube and canvas, max 800x600 (src/graphics/mod.rs:1-27,
  217), wgpu pinned =27.0.1. Real GPU output: Unverified; tests print
  `GPU SKIP` and pass without hardware (tests/wgpu_graphics.rs:30-45).
- Build-time network: crates/libghostty-vt-sys/build.rs clones Ghostty at
  22d13172 into OUT_DIR and runs `zig build` unless GHOSTTY_SOURCE_DIR is set
  (:6-7, 178, 234, 409-438). No Zig version check in build.rs.

## 7. C ABI and TypeScript (Exists)

- `#[cfg(feature = "ffi")] pub mod ffi` (src/lib.rs:72-73); 266 `extern "C"`
  functions; every export has a panic boundary, proven by a syn walk with a
  negative fixture (tests/ffi_export_inventory.rs:90, 105). Families:
  rtui_element 33, rtui_signal 26, rtui_app 17, rtui_animation 14, rtui_text
  12, rtui_foreign 10, plus camelCase legacy createRenderer/textBuffer*
  (src/ffi/mod.rs:84-130).
- Headers: include/reactive_tui.h plus 15 modular headers, native.h
  generated by cbindgen 0.29.4 (scripts/generate-native-header.py,
  scripts/abi/cbindgen.toml). Contradicted: include/reactive_tui.h says
  `@version 0.1.0`; crate is 1.0.0. Broken: include/README.md:12 links a
  deleted spec.
- TypeScript @reactive-tui/core 0.1.0 over koffi 3.2.1; loader order
  RTUI_LIBRARY_PATH, native/, target/ (bindings/typescript/src/ffi.ts:1-51).
  Contradicted: package.json:49 repository is github.com/reactive-tui/reactive-tui,
  Cargo.toml:9 is github.com/eas4ai/reactive-tui. Acceptance is Linux-only
  (scripts/check-typescript-abi.py hard-codes libreactive_tui.so).
- No chart exposure in FFI or TypeScript (grep chart in src/ffi, include,
  bindings/typescript/src: none).

## 8. Tests and verification

- Exists: tests/ holds 164 files (148 .rs, 8 .c, 9 .py, 7 .ansi goldens);
  1,096 in-source `#[test]` in src/; crates add 138 + 75 + 19 + 2. Shared App
  driver with vt100 screen and a 3 s budget that panics on timeout
  (tests/common/app_input.rs:186-471). Goldens compared byte-exact
  (tests/render_ops_snapshots.rs:15-40). proptest in three places. Miri only
  in libghostty-vt under cfg(miri), not in CI. No fuzz targets, no sanitizer
  runs.
- Exists: feature-gated binaries (ffi x8, wgpu x4, embedded-terminal x2)
  never run in CI because ci.yml uses default features (Cargo.toml:200,
  ci.yml:62-68).
- Exists: machine-bound tests skip or self-gate rather than fail: wgpu
  (prints GPU SKIP), clipboard_platform (the only #[ignore], env-gated),
  Windows console (cfg windows), Linux PTY tests (cfg linux, need /bin/sh,
  stty, python3).
- Exists: 32 ignored tests at run time; most are PTY fixtures invoked by
  deleted Cairn mechanisms (TRL-001, TRL-002, RTR-002 names in the ignore
  reasons), so those behaviors (main-thread panic restores the terminal,
  worker panic restores the terminal, termination signal uses the wake path)
  now have no runner. Unverified until a mechanism runs them again.
- Weak or vacuous tests found by sampling 24: tests/test_absolute_paint.rs:7,
  48 and tests/test_absolute_debug.rs:5 (print only), tests/animations_integration.rs:13
  (no assert), tests/production_readiness_test.rs:248 (asserts a signal value,
  named panic_recovery), tests/css_animation_integration_test.rs:213
  (contains and len checks). Dead files: tests/test_ffi_builder.c (no
  consumer), tests/test_absolute_positioning.rs.broken, tests/integration/
  (empty), benches/event_router_comparison.rs (hand main, not declared, never
  runs under cargo bench).
- Contradicted: verification/api-residual/README.md:3 says the acceptance
  command is `cairn check API-019`; no such mechanism exists.

## 9. Scripts and CI

- Exists: 100 check scripts under scripts/ plus 16 script unit tests. Two
  workflows: ci.yml (push and PR to main, weekly cron; ubuntu-24.04, macos-14,
  windows-2022 with the gnu target; Rust 1.91.0; build, test, fmt, clippy on
  the root crate; two Python gates; scheduled cargo-audit and cargo-deny) and
  clipboard-platforms.yml (fires only on branch
  codex/clipboard-platform-verification; Rust 1.95.0; uploads from
  .cairn/reviews/** with if-no-files-found: error, a path that no longer
  exists).
- Contradicted: scripts that read deleted paths and will crash:
  scripts/check-api-residual.py:19 and check-api-widget-behavior.py:15
  (docs/spec/rust-api-remediation.md), check-inherited-abi.py:21, 26
  (.cairn/mechanisms), check-widget-platforms.py:18, 81
  (.cairn/api-closure/check.py), check-documentation-retention.py:12-25
  (docs/spec, docs/commitments, docs/decisions), check-renderer.sh:13
  (spec-lint.mjs docs/spec), verification/api-entry-points/run-native.py:17.
  Roughly twenty more write to .cairn/reviews or .cairn/evidence.
- Contradicted: scripts/check-example-cleanup.py:12, 16 expects only
  gradient_blocks.rs and lists embedded_shell as removed, but
  examples/embedded_shell.rs and wgpu_benchmark.rs exist and Cargo.toml:123-126
  registers embedded_shell.
- Exists: upstream leftovers inside crates/reactive-tui-crossterm (.github,
  .travis.yml, CHANGELOG, Cargo.lock, docs/, examples/) that never run.
- Exists: crates/reactive-tui-macros carries its own Cargo.lock inside the
  workspace.

## 10. Documentation claims against the tree

| Claim | Where | Verdict |
| --- | --- | --- |
| Pre-release 0.1.0 candidate, not tagged | README.md:35-37 | Contradicted: Cargo 1.0.0; tags v0.1.0 and v1.0.0 both dated 2026-09-20 |
| [1.0.0] Unreleased | CHANGELOG.md:8 | Contradicted: tag v1.0.0 exists (3b75b7fd) |
| Use a path dependency until the crates.io release | README.md:62 | Documented; the 1.0.0 decision is source-only, no registry publish (CHANGELOG.md:10-11, scripts/check-crates-release.py:4-6) |
| Companion crates published separately | crates/reactive-tui-macros/README.md:6-7, reactive-tui-suprtui/README.md | Contradicted: all publish = false |
| Rust 1.70+ | CONTRIBUTING.md:13 | Contradicted: rust-version 1.91 |
| Zig 0.16.0 | README.md:73, scripts/check-crates-release-build.sh:23-31 | Contradicted by CHANGELOG.md:54 (0.15.2); build.rs pins neither |
| libghostty-vt from crates.io 0.2.1 | CHANGELOG.md:46-47 | Contradicted: vendored path crate (Cargo.toml:83); UPSTREAM.md says 0.2.1 lacks the API |
| Embedded dependency fetched from a pinned git revision | manual/getting-started.md:63 | Contradicted: path dependency |
| Syntax highlighting via Syntect | README.md:288 | Contradicted: lumis 0.13 (Cargo.toml:47) |
| cargo run --example image_widget_demo | CONTRIBUTING.md:37 | Contradicted: no such example |
| Release CI remains on the roadmap | README.md:176-177 | Contradicted: ci.yml exists |
| Cairn specifications record the release contracts | README.md:271-272 | Documented, no evidence in tree |
| Three official command sets | README.md:263-265, AGENTS.md:133-135, ci.yml:62-68 | Contradicted with each other |
| check-framework-manual.py MAN-001 | README.md:266-267 | Exists; the argument is ignored by the script |
| Eight Cargo features as listed | README.md:155-164 | Exists |
| SuprTUI, Crossterm, Debug backends | README.md:46 | Exists; DirectTty undocumented |
| Kitty, Sixel, iTerm2 protocols | manual/images-and-clipboard.md:20 | Exists in code; host behavior Unverified |
| Orca speech verified | CHANGELOG.md:19, manual/supported-api.md:52-56 | Unverified |

Broken links at HEAD: README.md:37 (docs/spec/roadmap.md);
include/README.md:12; manual/supported-api.md:6, 10, 13, 23, 25, 26, 28,
31, 34, 37, 48, 67 (docs/spec/*, docs/commitments/*, .cairn/*). Every check
ID cited in manual/supported-api.md (API, RND, EMB, GPU, FFS, ABI, RAC, WAK,
RTR, TRL, MAN, DQC, RID) has no declaration anywhere in the tree.

## 11. History

- Exists: 2,224 commits, one author. 25 commits in 2025-08/09 built the
  framework in large squash commits (root 489f464c). Nothing until
  2026-09-07, then 2,199 commits in 15 days, of which roughly 1,860 are
  process records (Refresh, Record, evidence:, Link, review:, decision:) and
  about 336 are engineering changes.
- Structural events: renderer vendored into src/backend (f8919cb2, 09-07);
  docs purge 45b7a9ab (09-14, 421 files, empty body); crates.io 0.1.0
  preparation (b9f7bc64, 09-14); .cairn data deleted 7fd91fc7 (09-19, 18,975
  files; subject says corrupted, body empty; .remember notes it was 871 MB and
  blocked packaging); companion crates vendored under crates/ and syntect
  replaced by lumis (94e5f808, 09-20, 129 renames); 1.0.0 source-only bump
  (3b75b7fd, 09-20); Cairn 1.x specs, decisions, commitments and plans removed
  (d27c99d9, 09-21, this recon's starting point).
- Documented, unfinished, from the deleted backlog plan
  (3b75b7fd:.agents/plans/2026-09-19-backlog-execution.md): Track A.1
  end-to-end Orca reader path, no commit; Track A.2 partial (node.rs TODOs
  11 to 7 in 748e3b00); Track B.1 size/get_size, meta(), None-id transition
  gap in src/app.rs and src/event/types.rs, no commit.
- Documented escalations never shown closed: ABI-004 fails under external
  build load (303812c8, f5f10863, 59308713); API-008 clipboard evidence stale
  on macOS and Wayland (a57da4d6, 9e815cbe); API-011 stale ConPTY artifact
  (e41cb3dd); macOS screen-capture consent (1723a26c); CI enforcement
  recorded as pending (a35f1547). Deadline knobs were loosened rather than
  fixed (0c053bc2, 074eb7a4, e819b1e0).

## 12. Earlier findings carried forward (audit of 2026-09-14 at 7489a52b)

Verified today unless marked.

- Repaired: C ABI use-after-free in rtui_app_quit, non-nullable callback
  types, text-buffer resize (src/ffi/app.rs:356-386, reactive.rs:107;
  FFS commits 09-14 to 09-18). Panic boundary on every export
  (tests/ffi_export_inventory.rs). include allowlist in Cargo.toml:13-24.
  ci.yml on main. deny.toml present; atty gone; quick-xml, bincode, yaml-rust
  gone with the Lumis swap; cargo audit passes.
- Unverified today: terminal restoration after a render-worker panic
  (tests exist but are ignored pending a PTY runner, section 8); animation
  hooks rebuilding state per render and use_transition spawning a thread per
  frame (src/hooks/animation.rs, not re-read); accessibility auto-enable
  ending App::run on a stale D-Bus address (src/app.rs:833-856 still
  auto-enables; failure handling not re-read); ThreadedEventLoop livelock and
  TokioEventLoop block_on (src/platform/loop.rs, not re-read); RefCell borrow
  across user effects (src/reactive/runtime.rs, not re-read); AdaptiveConfig
  panics (a commit "Remove constructor and reflow panic paths" 813ce4f3 may
  cover it); library println!/eprintln! in raw mode; dialog curl POSTs
  undocumented in README.
- Still open: README links to deleted docs (re-broken by today's wipe);
  version story disagrees across README, CHANGELOG, Cargo, header, npm.

## 13. Blast radius for the requested feature: charts modeled on gpui-kit

Requested: better and better-looking chart widgets and builder, mimicking
the gpui-kit 0.6.6 charts as far as a cell grid allows; later, other widgets.

Modules inside the radius (Exists):
- src/widgets/display/charts.rs (594 lines): ChartsBuilder (:15-192),
  ChartAxis (:225-254), ChartLegend (:258-290), DataPoint (:294-303),
  DataSeries (:341-354), ChartType BarVertical, BarHorizontal, Line, Area,
  Pie, Donut, Scatter (:416-431), ChartProps, ChartState, Chart.
- src/widgets/display/charts/live/ (LiveChart: live.rs, canvas.rs,
  canvas/cartesian.rs, pie.rs, motion.rs): cell canvas emitted as absolute
  Text elements per same-colour run (canvas.rs:99-142); glyphs are full block
  for bars and pie, bullet for scatter, box-drawing horizontal for lines,
  middle dot for grid, shade blocks for area fills (cartesian.rs:246-470);
  lines are stepped cell by cell with the horizontal glyph only, so steep
  slopes render as broken dashes (:372-386). No braille, no half or eighth
  blocks. Pie is a disc with 2:1 aspect correction (pie.rs:45-58). Tooltip is
  a full-width text band (canvas.rs:81-98) with keyboard Left/Right/Home/End
  (live.rs:215-254) and an aria-live announcement (live.rs:146-159). Reveal
  animation 0 to 1 on a 16 ms timer honouring reduced-motion
  (motion.rs:16-72). Hard-coded hex palette (charts.rs:205-214), no theme
  hook. Size is props width/height (default 80x20) capped at the parent; it
  never grows to fill (live.rs:120-139, canvas.rs:240-252).
- src/builder/widgets/chart.rs (:12-207) second builder with simple_series,
  labeled_series, axes, colors, animated, legend; `chart!` macro
  (src/builder/macros.rs:205).
- tests/charts_test.rs (9 config tests), tests/api_widget_behavior/charts.rs
  (15 behavior tests: geometry at two sizes, tooltips, animation, clipping,
  negatives), tests/pre_release_terminal_input_safety.rs (mentions Chart),
  examples/widget_catalog/catalog.rs:449-457 (one 32x7 line chart),
  manual/display-widgets.md:14-15, 35-36, 44. Orca display probe references
  charts (tests/api_widget_behavior/orca_display.py).
- Outside the radius but touched by any chart rewrite: src/theme (needs
  chart_1..5 and bullish/bearish colours), src/graphics/canvas.rs:99 and
  src/graphics/mod.rs:153 (an existing half-block emitter that a braille or
  block canvas could share), src/hooks/mouse (hover), accessibility node
  semantics for a chart role.

What gpui-kit provides that the current chart lacks (G/plot and G/chart,
cited in the comparison): a shared Plot trait with prepaint/paint/hover/
tooltip (G/plot/mod.rs:25-111); scales ScaleLinear, ScaleBand with
padding_inner/outer, ScalePoint, ScaleOrdinal (G/plot/scale/*.rs); axis and
grid with dash arrays (G/plot/axis.rs:54-165, grid.rs:5-42); label measuring
and truncation (G/plot/label.rs); a tooltip with swatch rows, crosshair band
and dots (G/plot/tooltip.rs:33-496); curve styles Natural, Linear, StepAfter
with Catmull-Rom to Bezier (G/plot/mod.rs:113-119, shape/line.rs:175); a d3
style Stack helper (G/plot/shape/stack.rs); a sankey layout engine
(G/plot/shape/sankey.rs, 1,323 lines); theme colours chart_1..5, bullish,
bearish (G/theme/theme_color.rs:133-145). Chart types: Line, Area (overlay,
gradient fill), Bar (alignment Bottom/Top/Left/Right, value labels, per-bar
fill closure and gradient, corner radii, negatives), Candlestick, Pie and
Donut (inner_radius_fn, pad_angle, leader-line labels), Radar, Sankey.
gpui has no reveal animation, no keyboard point navigation and no grouped
bars; reactive-tui already has all three.

Feasibility in a cell grid (from the comparison): natural for bar alignment
and labels, band padding, stacking, candlestick, donut radius, tooltip rows,
crosshair, theme colours, fill-parent sizing, a public Plot-like extension
point; approximable with braille or block glyphs for smooth lines and
curves, area gradients, radar, sankey ribbons, bar corner caps; not
meaningful for hover halos and springs, anti-aliasing, path caches.

Breadth beyond charts (G module inventory vs src/widgets): counterparts
exist for accordion, breadcrumb, checkbox, chart, dialog, highlighter, input
(text only), menu, notification (Toast), popover, progress, radio, scroll,
select, slider, tab, table, theme, tree, virtual list (DataTable config).
No counterpart for alert, avatar, badge (beyond tabs), button widget, bubble,
carousel, clipboard button, collapsible, color picker, combobox, command
palette, description list, dock, group box, hover card, icon, kbd, link,
number and OTP inputs, rating, sheet, shimmer and skeleton, sidebar, spinner,
status bar, stepper, switch, tag, time and date picker, tooltip.

## 13a. Developer-stated targets (2026-09-21) and what the tree says

Documented (developer, this session): Reactive TUI targets modern terminals,
often 500 to 700 columns, with 1440p as a common minimum; animation must run
at 60 fps at least; wgpu integration exists and is in scope for rendering.

- Cell budget. Exists: CellFrame and the Debug backend both cap a frame at
  262,144 cells and 65,535 per axis (src/backend/cell_frame.rs:37, 88;
  src/backend/mod.rs:378, 469-471). A 700-column terminal exceeds the cap at
  375 rows; 500 columns at 525 rows. A 1440p terminal at a 7 px cell width is
  roughly 365 by 90 cells (33,000 cells), well inside; the cap binds only at
  very small fonts or 4K. Whether the SuprTUI worker keeps 60 fps at 100,000
  to 260,000 cells per frame: Unverified, no benchmark record in the tree.
- Frame rate. Exists: AdaptiveFpsManager derives its ceiling from detected
  capabilities (src/display/adaptive.rs:186) and the chart reveal animation
  runs on a 16 ms timer (motion.rs:16-72), so 60 fps is the design target
  already. Documented against it: manual/wgpu-graphics.md:86 "No 30/60 FPS
  guarantee or general GPU speedup is claimed." Sustained 60 fps at 500 plus
  columns with a full-frame chart redraw: Unverified. At 60 fps the gpui
  hover and lift springs become worth porting; the comparison marked them
  not meaningful only under a cell-snapping assumption.
- wgpu path. Exists: offscreen GraphicsCanvas up to 800 by 600 px
  (src/graphics/mod.rs:24-26, 55) emitted as half-block cells, two pixels per
  cell vertically (src/graphics/mod.rs:127-153, canvas.rs:99), with a CPU
  fallback. No path today from a GraphicsFrame to the Kitty or Sixel image
  protocols; those exist only for the Image widget
  (src/widgets/display/image/protocol_renderer.rs). Consequence for charts:
  a wgpu-rendered chart gets 2x vertical resolution as half blocks on every
  terminal, and true pixel output on Kitty, iTerm2 and Sixel hosts if a
  GraphicsFrame to image-protocol bridge is added. That moves pie, donut,
  radar, sankey ribbons, smooth curves and gradients from "approximable" to
  "natural" on capable hosts, with the cell canvas as the fallback. The 800
  by 600 px cap would need lifting for a 700-column chart at pixel
  resolution.

- Workers. Documented (developer): CPU-heavy work should run on worker
  threads, not the main loop. Exists: four named worker threads already
  follow one pattern, spawn with thread::Builder, bounded channel, stop flag,
  join on drop, results delivered through the AppWaker: the SuprTUI render
  worker (src/backend/suprtui.rs:196), the embedded terminal session
  (src/embedded/mod.rs:79, a wake-driven root that hands back CellFrames,
  :252), the live image worker (src/widgets/display/image/live/worker.rs:50)
  and the accessibility transport (src/accessibility/platform/unix/transport.rs:114).
  The wgpu canvas has a CPU fallback but no dedicated worker (grep
  thread::spawn in src/graphics: none). Consequence for charts: rasterizing
  scales, braille or block canvases and wgpu frames belongs on a worker that
  publishes a cell snapshot; the chart then follows the TerminalView shape
  (cell_frame plus wake_driven) instead of today's tree of absolute Text
  elements (canvas.rs:99-142). The main thread keeps input, layout of the
  surrounding tree and presentation.

- wgpu assessment (developer ruling 2026-09-21: charts first on a cell
  canvas; a general wgpu canvas is a later commitment). Exists: src/graphics
  is a fixed demo, two WGSL shaders (cube.wgsl, torus.wgsl) hard-wired into
  GpuCubeRenderer's two pipelines (src/graphics/mod.rs:217-221, 324-328), an
  RGBA GraphicsFrame capped at 800x600, a half-block emitter, a CPU cube
  fallback, fault injection, one consumer (the widget catalog). No drawing
  API, no worker, no image-protocol output. Reusable piece for charts: the
  half-block emitter only.

- References considered 2026-09-21. rust_pixel (github.com/zipxing/rust_pixel,
  Apache-2.0): tile-first cell buffer with terminal, wgpu/OpenGL window and
  WASM adapters; its mdpt charts use braille 2x4 for lines and pies, block
  eighths for bars, box drawing for axes, and explicitly no pixel GPU chart
  rendering (openspec/changes/archive/2026-02-15-add-mdpt-charts/design.md);
  static, non-interactive, unanimated. Reusable: apps/mdpt/src/chart/braille.rs.
  beamterm (github.com/junkdog/beamterm, MIT): GPU terminal renderer over
  glow (OpenGL 3.3, WebGL2), one instanced draw per frame, dynamic font
  atlas, under 1 ms at 45,156 cells on 2019 hardware. Ruling: stay on wgpu,
  no GL stack, no window backend now; take the cell-frame-as-universal-output
  and single-instanced-draw lessons into the graphics-canvas commitment.

## 14. Open questions for the developer

1. Which version story is true: 1.0.0 source-only (Cargo and tags) or 0.1.0
   candidate (README)? The spec keystone needs one.
2. Is the workspace clippy gate (AGENTS.md, ci.yml) meant to include
   crates/libghostty-vt? It fails there today.
3. The 32 ignored PTY fixtures lost their runners with the Cairn 1.x wipe.
   Do terminal restoration on panic and signal shutdown belong in the first
   commitment's mechanisms, or in a later one?
4. For the chart commitment: is the target the seven gpui chart types with a
   shared scale, axis, grid, legend and tooltip layer, or a smaller first
   cut (line, area, bar, candlestick) with radar and sankey deferred?
5. Should the manual's supported-api page be retired until mechanisms exist
   again, since every ID it cites is now undeclared?
6. Should charts have two renderers from the start, a cell canvas with
   braille and block glyphs, and a wgpu canvas that emits half blocks or
   image protocols, selected by host capability? Or cell canvas first?
7. Is the 262,144-cell frame cap a limit you accept for the 500 to 700
   column target, or does the first commitment need to raise it and prove
   60 fps at that size?
