# Wgpu Graphics Implementation Plan

Plan location follows the repository's tracked Cairn-document policy rather
than the ignored generic Superpowers plan directory.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking. The repository's Cairn agreement controls action order and supersedes generic branching and batch-execution guidance.

**Goal:** Deliver GPU-001 first: an optional wgpu dependency that preserves existing consumers and Rust 1.91; extend this plan for each subsequent requirement when Cairn names it.

**Architecture:** Offscreen wgpu rendering produces RGBA pixels for the ordinary terminal frame path. Keep the canvas, GPU worker, pixel conversion, demo, and measurement tooling separate. No GUI window or host-renderer replacement is introduced.

**Tech Stack:** Rust 1.91, wgpu 27.0.1, WGSL, existing App/SuprTUI, Python standard-library acceptance checks.

## File responsibilities

`Cargo.toml` and `Cargo.lock` own opt-in dependency wiring. The first check
lives in `scripts/check-wgpu-features.py`; its complete dependency footprint is
declared in `.cairn/mechanisms/wgpu-feature-compatibility`. Existing
`tests/api_feature_configurations.rs` and catalog behavior tests serve as consumer
regressions. No graphics Rust module is needed to prove GPU-001.

The next stage introduces `src/graphics/` for GPU resources and immutable
pixel output. Its exact App/canvas integration follows focused source and
impact inspection before GPU-002 edits; GPU-002 through GPU-005 are not claimed
implemented by this first dependency increment.

### Task 1: Declare GPU-001

- [x] Commit the mechanism declaration before execution.
- [x] Run `rtk proxy cairn wake`; follow its next action.

### Task 2: Add and attack the dependency validator

- [x] Create `scripts/check-wgpu-features.py` with these validation rules:

```python
def validate_manifest(manifest):
    dependencies = manifest.get("dependencies", {})
    features = manifest.get("features", {})
    dependency = dependencies.get("wgpu", {})
    assert dependency.get("optional") is True, "wgpu must be optional"
    assert features.get("wgpu-graphics") == ["dep:wgpu"], "missing opt-in wiring"
    pending = list(features.get("default", []))
    visited = set()
    while pending:
        feature = pending.pop()
        assert feature not in {"wgpu-graphics", "wgpu", "dep:wgpu"}, "default enables wgpu"
        if feature not in visited:
            visited.add(feature)
            pending.extend(features.get(feature, []))
    assert dependency.get("default-features") is False, "explicit native features required"
    assert {"std", "parking_lot", "vulkan", "metal", "dx12", "wgsl"} <= set(dependency.get("features", [])), "native backends or shaders missing"

def validate_default_tree(tree):
    assert not any(line.startswith("wgpu v") for line in tree.splitlines()), "default graph includes wgpu"
```

- [x] Add unittest fixtures that mutate each condition independently. Include a nested default feature alias and a default tree containing `wgpu v27.0.1`. The corrected manifest and a tree containing only existing dependencies pass.
- [x] Run the script before changing Cargo.toml. Require failure on missing optional wgpu, after fixture self-tests pass.
- [x] Add subprocess checks with a 20-minute timeout per Cargo command. Nonzero status fails the mechanism; no swallowed errors or compile-only success.

### Task 3: Enable the dependency, not the renderer

- [x] Add the dependency and feature exactly:

```toml
wgpu = { version = "=27.0.1", optional = true, default-features = false, features = ["std", "parking_lot", "vulkan", "metal", "dx12", "wgsl"] }

wgpu-graphics = ["dep:wgpu"]
```

- [x] Regenerate Cargo.lock through Cargo, preserving existing compatible resolutions.
- [x] Verify the default graph with `cargo tree --locked -p reactive-tui --edges normal --prefix none` and `validate_default_tree`.
- [x] Run `cargo +1.91.0 check --locked -p reactive-tui --lib`.
- [x] Run the same check with `--features wgpu-graphics`.
- [x] Run `cargo +1.91.0 test --locked -p reactive-tui --test api_feature_configurations --test widget_catalog_behavior` in both modes. These are existing consumers, not newly invented compatibility claims.
- [x] Run `cargo +1.91.0 build --locked -p reactive-tui --example widget_catalog` in both modes.

### Task 4: Commit and record real evidence

- [x] Run GitNexus staged change detection and diff whitespace checks. Confirm only the dependency, checker, declaration, and plan change.
- [x] Commit implementation before `rtk proxy cairn check GPU-001`.
- [x] Read the captured output and receipt. Commit all receipt/output/input files; never edit historical evidence.
- [x] Remove the in-progress record after the action is committed. Run wake again and extend this plan for the next named requirement.

### Task 5: Render a GPU cube offscreen

- [x] Declare and commit `.cairn/mechanisms/wgpu-rendering` before implementation.
- [x] Write `tests/wgpu_graphics.rs` first. It must fail because
  `reactive_tui::graphics` does not exist. Its real-adapter test renders at
  60x48, 144x100, and 200x120 pixel targets for the required terminal sizes,
  records adapter identity, rejects CPU adapters, checks exact output size,
  and requires at least 32 distinct opaque colors.
- [x] Add conversion tests that require one half-block cell per source column
  and one terminal row per two pixel rows. Paint the resulting Element through
  `DebugBackend` and assert the 60x24 frame uses the full requested bounds.
- [x] Implement feature-gated `src/graphics/mod.rs` with an offscreen
  `Rgba8UnormSrgb` texture, a fullscreen-triangle WGSL ray-box shader, elapsed
  time rotation, viewport aspect uniform, a 256-byte aligned readback buffer,
  and checked de-padding into owned RGBA pixels.
- [x] Keep dimensions within 800x600 pixels and reject zero or larger targets
  before multiplying allocation sizes.
- [x] Convert pairs of pixels to `▀` Elements using public `StyleBuilder::fg_rgba`
  and `bg_rgba`. This uses the existing bridge and terminal frame path without
  changing either paint backend.
- [x] Run the test red before implementation and green afterward. Build the
  default library to prove the public graphics module remains feature-gated.
- [x] Commit, run `cairn check GPU-002`, inspect and commit its evidence, then
  run wake before starting bounded scheduling.

Local GPU-002 verification selected AMD Radeon AI PRO R9700 (RADV GFX1201)
through Vulkan. The three viewports produced 200, 610, and 765 distinct opaque
colors, with unclipped pixel bounds and width/height ratios 1.036, 1.017, and
1.042. The native SuprTUI writer capture was parsed as terminal cells; every
half-block's foreground/background matched its two source pixels after resize.
All three Rust tests passed on Rust 1.91. The default library check also passed.
Five validator fixtures reject missing operations, placeholder output, window
paths, missing shading, and missing aspect correction. These local runs are
not Cairn receipts. PNG inspection is not the required host-terminal capture;
that visual obligation remains open for the integrated demo.

## Subsequent integration

### Task 7: Fallback and normal App ownership (GPU-004)

- [x] Declare the lifecycle mechanism, commit it, and run the missing-check baseline.
- [x] Write failing tests for hybrid renderer selection, truthful frame provenance,
  and cancellation of active/pending work before adding those APIs.
- [x] Add a CPU shaded-cube fallback that shares elapsed-time angles and checked
  dimensions, and document any quality difference after measurement.
- [x] Convert adapter initialization, device-loss, and readback errors into a
  persistent CPU selection with a visible reason; never relabel CPU as GPU.
- [x] Add cancellation tokens to the owned worker. Poll readback with bounded
  waits; shutdown clears pending work, suppresses publication, and joins.
- [x] Add a canvas owner attached to App's wake handle. The existing periodic
  update path uses the deadline clock; rendering remains off the App loop.
- [x] Integrate only the feature-enabled Motion page. Bypass the catalog's
  fixed-width ScrollView there and size the canvas to its available stage.
  Other paused catalog layout work remains untouched.
- [x] Add explicit example CPU/fault options for reproducible checks. Run
  existing consumers in default/enabled configurations.
- [x] Run isolated real PTY checks for each quit key, animation without input,
  resize, GPU mode, forced CPU, and each injected failure. Inspect actual
  restoration and process cleanup, then commit and record Cairn evidence.

Local verification passed four lifecycle tests, fifteen feature-enabled catalog
tests, and seven isolated PTY runs. Hardware and CPU modes animated without
input, resized to 200x60, and restored terminal state on the advertised quit
keys. Adapter, device-loss, and readback injections selected labeled CPU output.
PTY captures are not named desktop-host visual acceptance. Adjacent identical
half-blocks are batched into styled text runs; a failing structural fixture and
exact native color comparison verified that optimization. Focused Clippy passed
with the existing wizard lint explicitly allowed. Unrestricted strict Clippy
still fails on existing wizard/vendor findings, captured in the backlog.
Fresh Cairn evidence now records these runs; the closing review names the receipts.

## Source basis and review

### Task 9: Resolve the closing review's raw-renderer findings

- [x] Record the review before changing executable code.
- [x] Reproduce shared-uniform overwrite with eight concurrent hardware
  callers at different viewports/times. Correct the fixture's initial barrier
  early-exit bug, then require the actual pixel-mismatch RED failure.
- [x] Give every raw render submission its own uniform buffer/bind group,
  preserving public shared-reference APIs. The 64-frame concurrent test passes.
- [x] Add a software-adapter metadata label RED fixture, then distinguish
  Software wgpu from hardware GPU without weakening hardware acceptance.
- [x] Run final targeted checks, commit, refresh Cairn receipts, and review again.
- [x] Record the newly exposed shrink-back capture race before changing code.
  Demonstrate stale-valid/loading/valid premature acceptance, require stable
  samples and post-capture validation, and inspect all four fresh formal PNGs.

### Task 8: Measure stages and verify a named host (GPU-005)

- [x] Declare the measurement mechanism and record its missing-check baseline.
- [x] Write a missing-timings RED test, then measure draw completion separately
  from copy/map/de-padding in the same renderer path used by the catalog.
- [x] Add a bounded native-backend benchmark with explicit CPU selection,
  hardware-only GPU acceptance, finite sampling duration, terminal-sized
  targets, and create-new JSON reports. Report initialization separately.
- [x] Measure conversion, backend presentation, and total frame latency;
  state that App reconciliation and display scanout are not measured.
- [x] Compare fixed-time GPU and CPU pixels instead of assuming equal quality.
- [x] Attack missing fields, software adapters, mislabeled CPU output, hidden
  readback, nonfinite costs, and false FPS with failing/corrected fixtures.
- [x] Run six local comparisons and inspect actual Kitty screenshots in an
  owned Xvfb display, never the developer desktop. Expand the host predicate
  to require intact header/footer and no stale initialization placeholders.
- [x] Repair the newly exposed startup artifact: RADV diagnostics printed
  during lazy initialization scrolled the raw screen. Initialize the catalog
  renderer before terminal setup, then move it onto the owned worker. The
  strengthened host check fails before this change and passes afterward.
- [x] Document reproducible commands, fallback, finite limits, timing scope,
  quality measurements, and only the verified host configuration.
- [x] Run final focused checks, commit, record fresh Cairn receipts and artifacts.
- [x] Review requirement coverage and inspect committed host captures before Done.

Local debug comparisons selected AMD Radeon AI PRO R9700 / Vulkan for GPU
frames, and explicit CPU for the other samples. At 60x24, 144x50, and 200x60,
fixed-time pixels differed by at most one channel unit out of 255. Conversion
and terminal presentation dominated; no end-to-end speedup guarantee follows.
The actual Kitty 0.45.0 X11 captures show full-stage shaded cubes and intact
chrome after the startup fix. Native Wayland and other hosts remain unverified.
These local observations are not Cairn receipts.

Final formal evidence is recorded in `.cairn/reviews/wgpu-graphics.md`.
All five renewed graphics checks pass, and all four fresh Kitty PNGs show
intact controls and bounded shaded output after growing and shrinking.
Text acceptance cannot establish coherent host pixels by itself; one local
partially painted capture was rejected and visual inspection remains required.
An additional final default-library run passed 1051 tests with eight ignored.

Final local checks passed the broader nineteen selected integration test targets
(including 402 widget acceptance tests), formatting, both manual checks, and
focused Clippy with the existing wizard lint allowed. Rust 1.91 Clippy hit an
incremental metadata compiler crash once; rerunning with incremental compilation
and the compiler wrapper disabled passed. Ripwire identified growing renderer
complexity/length and a duplicate command wrapper: separate completion/readback
responsibilities and inline checker commands address those findings. Its
remaining test-discovery/dead-code findings include framework-invoked tests and
the public feature-gated timing API. Recent-edit churn is expected in this
increment. The owned-process shutdown pattern also appears in archived review
scratch scripts; importing historical evidence as a production dependency would
be inappropriate. These are reviewed limitations, not a claimed clean Ripwire
gate. Live PTY/Kitty runs cover the example entry point and fallback API beyond
the static test map.

### Task 6: Bound elapsed-time animation (GPU-003)

- [x] Declare and commit the animation mechanism; run wake for its next action.
- [x] Write feature-gated clock and worker tests first, then require their clean
  missing-symbol failure before implementing them.
- [x] Add a 50 ms deadline clock that produces checked viewport requests using
  elapsed time, skips missed deadlines, and never counts delivered frames.
- [x] Add one owned worker thread. A mutex/condition variable holds one pending
  request and one replaceable output; replacing work cannot grow a queue.
  The thread sleeps while no work exists and has one active render at a time.
- [x] Use a blocked first-render fixture to prove 1000 requests replace the
  pending slot, then release it and require the newest elapsed request.
- [x] Test zero, oversized, and overflowing dimensions before submitting work.
  Compare equal elapsed-time requests across different delivery schedules.
- [x] Render real GPU frames at two elapsed times and require changed pixels
  without any keyboard event.
- [x] Run targeted tests and validator attacks, commit before Cairn checks,
  then inspect and commit fresh receipts before catalog lifecycle integration.

Shutdown owns and joins the worker rather than detaching it. Driver failure,
CPU fallback, cancellable readback, mode labeling, and App integration remain
GPU-004 work. The existing paint bridge remains unchanged.

The clean missing-symbol RED failed only on FrameClock, FrameRequest, and
GraphicsWorker. Four runtime tests then passed, including a blocked first
render and 999 pending-slot replacements. Hardware frames at zero and one
second differed at 459 pixels. Two extra RED/GREEN attacks caught a 496-pixel
jump at the original one-turn wrap and repeated requests at Duration::MAX.
Independent axis wrapping and a checked deadline fix those cases. Four checker
fixtures also passed their violating and corrected cases. These are local runs;
GPU-003's committed check passed before lifecycle integration; its broad inputs
require another receipt after the GPU-004 implementation commit.

[wgpu 27.0.1's published manifest](https://docs.rs/crate/wgpu/27.0.1/source/Cargo.toml)
declares Rust 1.88 and the selected native backend/WGSL features. Actual locked
Rust 1.91 builds remain the proof, including transitive dependencies.

The default graph excludes an optional dependency even though Cargo.lock may
record it. Do not confuse lockfile membership with default compilation.
Catalog visual findings remain paused; dependency success is not cube-quality
or runtime-GPU evidence. Keep the shared artifact-directory owner alive.
