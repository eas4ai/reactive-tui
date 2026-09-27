# Supported API and verification limits

This matrix covers the public modules declared in
[`src/lib.rs`](../src/lib.rs), plus the re-exported procedural macros.
It names behavior checks, not a blanket production-readiness claim.
The pre-release CI documentation commitment
requires fresh evidence for every inherited requirement before completion.
Compilation and an older acceptance pass do not prove current native behavior.

Check declarations live in [.sudus/mechanisms](../.sudus/mechanisms).
Each check's receipts are Sudus records on `refs/sudus/log`, and the output
a run printed is kept in `.sudus/output`. Check a receipt's candidate and
freshness with Sudus rather than treating a named test as a current pass.

## Public module matrix

| Module | Supported contract and route | Behavior checks | Limits | Status |
| --- | --- | --- | --- | --- |
| `accessibility` | [Element semantics and owned AT-SPI publication](accessibility.md) | API-011, RTR-001; [widget checks](../tests/api_widget_behavior.rs) | Linux reader-client fixture; see screen-reader limits below | Covered by named checks |
| `animation`, `screen` | [Keyframes, springs, transitions and screen input](animation-and-screens.md) | API-013, API-010; [animation/screen workflows](../tests/api_animation_screens.rs) | Cell output approximates visual properties; parallel-test interference remains under review | Covered by named checks |
| `app`, `component`, `prelude` | [Root rendering, keyed instances, props, updates and cleanup](app-and-components.md) | API-002, API-016, TRL-001; [component lifecycle](../tests/api_component_expansion.rs) | Registered child names; explicit backend ownership; no implicit Props validation | Covered by named checks |
| `backend`, `core`, `render` | [Complete grapheme frames through SuprTUI; debug and legacy adapters](rendering-and-backends.md) | RND-001 through RND-006, API-016; renderer contract | Host draws glyphs; layout patches are not cell diffs; output errors require restoration | Covered by named checks |
| `builder`, `vdom` | [Element builders, retained callbacks and keyed virtual nodes](elements-builders-and-vdom.md) | API-005, API-011, API-002; [widget workflows](../tests/api_widget_behavior.rs) | Component registration and routed layout bounds determine interaction | Covered by named checks |
| `display` | [Display configuration and performance settings](rendering-and-backends.md) | RTR-003, API-019; runtime resilience contract | Unsupported host settings degrade explicitly; per-App performance ownership | Covered by named checks |
| `editor`, `markdown`, `syntax` | [Grapheme editing, Markdown conversion and bounded syntax work](text-editing-markdown-and-syntax.md) | API-007, API-019, API-018; Rust API contract | Cursor units and rendered display columns differ; large-input limits remain explicit | Covered by named checks |
| `error` | [Typed operation errors](getting-started.md) | API-002, API-016; [negative component cases](../tests/api_component_expansion.rs) | Callers must handle returned errors; an error is not a successful no-op | Covered by named checks |
| `escape`, `terminal` | [ANSI interpretation, terminal modes and owned PTYs](terminal-and-embedded-sessions.md) | EMB-001 through EMB-006, API-019; embedded terminal contract | Cell dimensions; Unix and Windows legacy PTYs require separate native evidence | Covered by named checks |
| `event` | [Keyboard, mouse, focus and capture/bubble routing](events-focus-and-input.md) | API-005, API-006; [widget workflows](../tests/api_widget_behavior.rs) | Hit targets follow presented layout; nested traps own focus restoration | Covered by named checks |
| `ffi` | [Audited C ABI and TypeScript consumers](ffi-and-typescript.md) | API-001, API-017, FFS-001 through FFS-005, ABI-001 through ABI-004; [C consumer](../tests/binding_abi_consumer.c) | Requires ffi; matching allocation families, callbacks and library/header versions | Covered by named checks |
| `hooks`, `reactive` | [Owned hook slots, state, effects, timers and context](reactive-state-and-hooks.md) | API-003, API-004, RAC-001 through RAC-003, WAK-001 through WAK-005; Rust API contract | Local non-Send references require a local scope; unmount cancels owned work | Covered by named checks |
| `layout`, `theme` | [Flex/grid cells, dynamic style, gradients and scoped themes](layout-style-and-themes.md) | API-009, API-010, API-019, RTR-004; [widget workflows](../tests/api_widget_behavior.rs) | Terminal typography/animation approximations; invalid grid input rejects safely | Covered by named checks |
| `platform` | [Native adapters, bounded input and clipboard commands](images-and-clipboard.md) | API-008, API-019; [clipboard acceptance](../tests/clipboard_platform.rs) | Five clipboard backends need real desktop evidence; see native limits below | Covered by named checks |
| `ui` | [Per-App Updater registration and request delivery](app-and-components.md) | API-019; approved Updater migration | Implement update; requests coalesce and callbacks run on the App thread | Covered by named checks |
| `widgets` | [Input, layout, display, menu, dialog, image and terminal controls](README.md) | API-011, API-012, API-014; [widget workflows](../tests/api_widget_behavior.rs), [dialog lifecycle](../tests/api_dialog_lifecycle.rs) | Real viewport/input workflows required; direct image protocols have host-specific limits | Covered by named checks |
| `embedded` | [Owned libghostty Unix session](terminal-and-embedded-sessions.md) | EMB-001 through EMB-006; [session tests](../tests/embedded_terminal.rs) | Requires embedded-terminal on Unix and the pinned Zig toolchain | Covered by named checks |
| `graphics` | [Offscreen wgpu canvas with labeled CPU fallback](wgpu-graphics.md) | GPU-001 through GPU-005; wgpu graphics contract | Requires wgpu-graphics; bounded viewport/readback; no separate GPU window | Covered by named checks |
| `macros` | [Component generation and derived Props builders](app-and-components.md) | API-003, API-018; [documentation contract tests](../tests/api_documentation_contract.rs) | Explicit validate predicates; see Props contract below | Covered by named checks |

## Props contract

Derived `Props` keeps fluent builders and caller-invoked `validate() -> bool`.
Each `#[prop(validate = rule)]` predicate receives a shared field reference.
Every declared predicate must succeed; a field without a predicate adds no
constraint. Construction and App mounting do not invoke validation.
Bare or malformed validation annotations and unsupported options are errors.
`#[prop(optional)]` requires a written `Option<T>` and defaults to None.
See the approved Rust API remediation contract.

## Native and host limits

- Screen-reader acceptance is the approved Orca / GNOME Terminal Linux pair.
  It requires actual labels, roles, focus and state delivery, not painted text
  or retained metadata. The fixture uses an isolated repaired libatspi 2.60.6;
  it does not modify the installed desktop library. Other reader/host pairs
  are unverified. Fresh reader evidence is still a release gate.
- Kitty direct-image acceptance requires the approved isolated host patch
  that prepares placements before selecting the image layer. This does not
  update users' stock Kitty installations. Preserve stock-host failure
  evidence and require current placement, update and removal checks.
- iTerm2 3.7 inline-image color accuracy and transparency are unsupported by
  explicit approval. Placement, update and removal remain required. WezTerm
  is the approved inline-protocol color acceptance host. GNOME Terminal Sixel
  and cell fallback retain their own image requirements.
- Clipboard verification covers Linux Wayland, xclip and xsel, Windows native
  PowerShell, and macOS pbcopy/pbpaste. The native records of those runs,
  with source digest, host, output and cleanup, were removed with the old
  `.cairn` records on 2026-09-19 (commit 7fd91fc7) and remain in Git
  history. Synthetic fixtures are not desktop acceptance.
- Minimal/default and optional-feature build contracts remain under API-015.
  SIMD requires nightly. Native Windows/ConPTY and macOS checks cannot be
  inferred from Linux compilation or cross-compilation.

[Back to the manual](README.md)
