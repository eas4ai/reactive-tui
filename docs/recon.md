# Reactive-TUI reconnaissance

Status: Observed
Date: 2026-09-07
Baseline: `a3ea1cc85` on `main`.

This report records current behavior, not agreed requirements. The developer has selected rehabilitation of the project, starting with a
dependable renderer and considering existing terminal libraries. The renderer
choice and acceptance requirements remain proposed; see [renderer assessment](renderer-foundation.md).
There was no prior recon report to carry forward. See
[baseline evidence](recon-evidence/baseline.txt:1).

## Exists

| Finding | Evidence |
| --- | --- |
| The product is a Rust terminal UI library, version 0.0.7, built as an rlib and cdylib. It uses a local procedural macro crate. | [Cargo.toml](../Cargo.toml:1), [macro manifest](../reactive-tui-macros/Cargo.toml:1) |
| Dependencies include Taffy layout, crossterm terminal access, syntect highlighting, comrak Markdown, and image libraries. Tokio is a default feature; C bindings require `ffi`. The lockfile uses format 4. | [Cargo.toml](../Cargo.toml:24), [Cargo.lock](../Cargo.lock:3), [library exports](../src/lib.rs:64) |
| A caller supplies a backend and a RootComponent to AppBuilder. Missing either produces an error. App owns scheduling, event routing, rendering, focus, animations, and frame timing. | [App fields](../src/app.rs:27), [builder](../src/app.rs:435) |
| App renders initially, polls input, exits on Ctrl+C or Escape, handles resize, routes events, processes pending updates, advances animations and timers, and records frame performance. Errors from polling and rendering propagate through Result. | [main loop](../src/app.rs:51) |
| A render calls the root component, processes focus properties, converts Elements into a RenderTree, and passes initial repaint or reconciliation patches to the backend. | [render flow](../src/app.rs:266) |
| Element and LayoutType describe component output. Signal stores shared local state with Rc/RefCell; the App performance context instead uses ThreadSafeSignal. These are distinct state mechanisms. | [Element](../src/component/element.rs:28), [Signal](../src/reactive/signal.rs:22), [performance context](../src/app.rs:72) |
| PlatformTty abstracts byte output, size, and restoration; Unix and Windows modules are conditionally compiled. | [platform boundary](../src/platform/mod.rs:11), [trait](../src/platform/mod.rs:86) |
| The public surface spans builders, components, widgets, layout, rendering, reactive state, animation, screens, editors, syntax, Markdown, themes, and terminal support. | [module exports](../src/lib.rs:41) |
| The Rust error enum uses thiserror; FFI defines separate numeric errors and a panic-catching helper. Debug App output uses stderr. This does not establish that every FFI entry point catches panics. | [Rust errors](../src/error.rs:11), [FFI helpers](../src/ffi/mod.rs:100), [debug output](../src/app.rs:63) |
| TypeScript uses ffi-napi and searches for the native library in development release, packaged native, and development debug locations. A missing library throws an error. | [loader](../bindings/typescript/src/ffi.ts:35), [package](../bindings/typescript/package.json:45) |
| Tests include inline Rust unit tests and 44 discovered integration targets. A pipeline fixture builds an App with DebugBackend. Nested demo files are not automatically established as runnable test targets by their presence. | [test fixture](../tests/full_pipeline_integration.rs:59), [target evidence](recon-evidence/baseline.txt:9), [inventory](recon-evidence/inventory.txt) |
| The latest changes concern responsive visuals and examples. The inspected history contains 25 commits; local main and origin/main are the listed branches. | [history and branches](recon-evidence/history.txt:1) |

## Documented

| Coverage | Evidence |
| --- | --- |
| README describes features, installation, utilities, images, terminal compatibility, and examples. These claims need verification in the selected work area. | [README](../README.md:1) |
| Contribution instructions name cargo build, cargo test, cargo clippy with no warnings, and formatting. They describe module organization and broad design principles. | [checks](../CONTRIBUTING.md:24), [architecture guidance](../CONTRIBUTING.md:115) |
| Adaptive performance documentation describes the context published by the App; animation documentation describes screen transition integration. | [performance guide](ADAPTIVE_PERFORMANCE.md:13), [animation guide](ANIMATION_INTEGRATION.md:1), [App context](../src/app.rs:72) |
| The ABI policy describes ownership, error handling, versioning, and platform promises. The TypeScript package declares build, test, lint, and example scripts. | [ABI policy](FFI_ABI_POLICY.md:7), [package scripts](../bindings/typescript/package.json:7) |

## Contradicted — unresolved

These are documentation conflicts or failing checks, not permission to change
behavior. No Agreed specification exists against which to classify formal drift.

| ID | Finding | Evidence on both sides |
| --- | --- | --- |
| R-01 | README quick start calls title, size, and component builder methods; the current AppBuilder exposes backend and root instead. | [README](../README.md:29), [AppBuilder](../src/app.rs:435) |
| R-02 | README advertises four cargo example targets absent from Cargo metadata. The available examples are animated_patterns, gradient_blocks, and visual_effects. | [README](../README.md:104), [metadata evidence](recon-evidence/baseline.txt:7), [inventory](recon-evidence/inventory.txt) |
| R-03 | The documented all-tests-pass standard is unmet. The JumpStart easing test expects 0.0 but returns 0.25; the library run stops with 742 passing, 1 failing, and 1 ignored. Intended easing semantics remain for the developer to confirm. | [standard](../CONTRIBUTING.md:45), [assertion](../src/animation/mod.rs:1377), [run evidence](recon-evidence/baseline.txt:10) |
| R-04 | Formatting and warning-free Clippy standards are unmet by the baseline. Clippy with warnings denied reports 49 errors. | [standard](../CONTRIBUTING.md:44), [verification](recon-evidence/baseline.txt:13), [Clippy log](recon-evidence/clippy.log) |
| R-05 | ABI documentation names rtui.h and tests/ffi/, while tracked headers use reactive_tui.h and modular includes and tests occupy other paths. Binding documentation needs a focused ABI audit before its broader promises are accepted. | [ABI policy](FFI_ABI_POLICY.md:18), [test instructions](FFI_ABI_POLICY.md:184), [inventory](recon-evidence/inventory.txt) |

| R-06 | A combined utility string changes explicit z-50 to z-index 10. Four accordion documentation examples fail to compile. | [z-index assertion](../tests/z_index_tests.rs:156), [doctest failures](recon-evidence/default-tests.txt) |
| R-07 | FFI tests refer to missing functions and fail compilation despite the feature itself passing cargo check. | [FFI compile failure](recon-evidence/ffi-tests.txt), [FFI check](recon-evidence/ffi-check.txt) |
| R-08 | App leaves previous_tree empty in its first-render branch, so subsequent renders continue taking that branch. DirectTtyBackend has no paint path for small Insert/Remove/Update patch sets. These source findings still need controlled runtime reproductions. | [App render](../src/app.rs:284), [backend patches](../src/backend/direct_tty.rs:332) |

## Unverified

| Question or limit | Evidence / next verification surface |
| --- | --- |
| Which existing rendering foundation should be adopted, and which product capabilities must it preserve? | [Renderer assessment](renderer-foundation.md) |
| The later no-fail-fast run establishes default-feature results: 966 passed, 6 failed, 35 ignored. FFI code checks successfully but its test suite does not compile. TypeScript runtime remains untested. | [extended results](recon-evidence/baseline.txt:18), [default tests](recon-evidence/default-tests.txt), [FFI tests](recon-evidence/ffi-tests.txt) |
| Rust 1.70 compatibility is unverified; this run used Rust 1.95 and the checked-in lockfile uses format 4. | [stated minimum](../README.md:140), [lockfile](../Cargo.lock:3), [toolchain](recon-evidence/baseline.txt:2) |
| Real terminal behavior, Windows/macOS support, image protocols, performance promises, and ABI safety need targeted validation. | [platform code](../src/platform/mod.rs:11), [ABI claims](FFI_ABI_POLICY.md:64), [README](../README.md:120) |
| No tracked CI/container configuration, spec lint script, or decision records appeared in the inventory. External automation and undocumented reasons for technology choices remain unknown. | [inventory](recon-evidence/inventory.txt), [baseline](recon-evidence/baseline.txt:5) |
| Persistent storage/schema behavior was not established in this overview; local signal and application state were inspected. Inspect data ownership and recovery inside the selected work area. | [Signal](../src/reactive/signal.rs:22), [App](../src/app.rs:27) |
| A CSS optimizer comment leaves focused tests as a TODO; this is not proof that the optimizer has no coverage elsewhere. | [TODO](../src/layout/css/optimizer.rs:484) |
| Cairn wake exits 3 because no roadmap exists. Spec lint cannot run from this checkout because its script and spec set are absent. | [baseline](recon-evidence/baseline.txt:4) |

## Session todo

- Complete: inspect the baseline and record actual verification results.
- Complete: write the recon report and check cited paths and line bounds.
- Complete: extend baseline checks through integration, documentation, and FFI.
- In progress: assess the renderer foundation and define its acceptance proof.
- Pending: draft Observed vocabulary/specifications in that area, confirm behavior
  and falsifiers, and prepare the commitment and working agreement.

Observed material is not contract. Cairn refuses commitments naming Observed
requirements; developer confirmation is still required.
