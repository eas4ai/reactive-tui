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

- [ ] Run GitNexus staged change detection and diff whitespace checks. Confirm only the dependency, checker, declaration, and plan change.
- [ ] Commit implementation before `rtk proxy cairn check GPU-001`.
- [ ] Read the captured output and receipt. Commit all receipt/output/input files; never edit historical evidence.
- [ ] Remove the in-progress record after the action is committed. Run wake again and extend this plan for the next named requirement.

### Task 5: Render a GPU cube offscreen

- [ ] Declare and commit `.cairn/mechanisms/wgpu-rendering` before implementation.
- [ ] Write `tests/wgpu_graphics.rs` first. It must fail because
  `reactive_tui::graphics` does not exist. Its real-adapter test renders at
  60x48, 144x100, and 200x120 pixel targets for the required terminal sizes,
  records adapter identity, rejects CPU adapters, checks exact output size,
  and requires at least 32 distinct opaque colors.
- [ ] Add conversion tests that require one half-block cell per source column
  and one terminal row per two pixel rows. Paint the resulting Element through
  `DebugBackend` and assert the 60x24 frame uses the full requested bounds.
- [ ] Implement feature-gated `src/graphics/mod.rs` with an offscreen
  `Rgba8UnormSrgb` texture, a fullscreen-triangle WGSL ray-box shader, elapsed
  time rotation, viewport aspect uniform, a 256-byte aligned readback buffer,
  and checked de-padding into owned RGBA pixels.
- [ ] Keep dimensions within 800x600 pixels and reject zero or larger targets
  before multiplying allocation sizes.
- [ ] Convert pairs of pixels to `▀` Elements using public `StyleBuilder::fg_rgba`
  and `bg_rgba`. This uses the existing bridge and terminal frame path without
  changing either paint backend.
- [ ] Run the test red before implementation and green afterward. Build the
  default library to prove the public graphics module remains feature-gated.
- [ ] Commit, run `cairn check GPU-002`, inspect and commit its evidence, then
  run wake before starting bounded scheduling.

## Source basis and review

[wgpu 27.0.1's published manifest](https://docs.rs/crate/wgpu/27.0.1/source/Cargo.toml)
declares Rust 1.88 and the selected native backend/WGSL features. Actual locked
Rust 1.91 builds remain the proof, including transitive dependencies.

The default graph excludes an optional dependency even though Cargo.lock may
record it. Do not confuse lockfile membership with default compilation.
Catalog visual findings remain paused; dependency success is not cube-quality
or runtime-GPU evidence. Keep the shared artifact-directory owner alive.
