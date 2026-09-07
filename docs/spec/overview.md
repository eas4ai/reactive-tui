# Reactive-TUI

Status: Observed

Reactive-TUI provides Rust components and CSS-like layout for applications
inside existing terminals. The developer confirmed Kitty, GNOME Terminal,
and Ghostty as target hosts on 2026-09-07. A separate GPU window is outside
this direction. Existing behavior and unresolved defects are recorded in
`docs/recon.md`.

## Agreed recovery direction

On 2026-09-07 the developer agreed to SuprTUI for application rendering and
libghostty-rs for embedded terminal sessions, accepting its Zig build dependency.
The first commitment delivers a Reactive-TUI screen through SuprTUI with
verified updates, Unicode, resizing, and clean terminal restoration. Embedded
sessions follow that foundation. Rendering and embedded-terminal requirements are
contract; the rest of the legacy feature catalog remains Observed.

## Architecture

`App` obtains an Element tree from a RootComponent (`src/app.rs`). The component
bridge converts it to the layout painter's NodeSpec (`src/component/bridge.rs`).
Taffy computes cell-space layout (`src/layout/paint_tree.rs`). The new backend
paints complete grapheme-aware frames into SuprTUI; The locally maintained SuprTUI engine owns cell comparison
and ANSI output. Crossterm supplies host input and raw mode.

A worker owns SuprTUI's local grapheme pool and renderer. Synchronous commands
preserve the existing Send + Sync Backend contract without unsafe promises
about Rc/RefCell. One frame has one acknowledged output operation. Detailed
choices and the dependency pin live in `docs/decisions/`.

## Spec map

| Area | Specification or observed source |
| --- | --- |
| Vocabulary | `docs/spec/glossary.md` |
| SuprTUI rendering and host lifecycle | `docs/spec/rendering.md`, prefix RND |
| Legacy render behavior | `docs/spec/legacy-rendering.md`, prefix OLD, Observed |
| Commitment order | `docs/spec/roadmap.md` |
| Widgets and builders | `src/widgets/`, `src/builder/`; outside first commitment |
| Reactive state and hooks | `src/reactive/`, `src/hooks/`; outside first commitment except root input delivery |
| Embedded terminal sessions | `docs/spec/embedded-terminal.md`, prefix EMB; libghostty-rs with a real child PTY |
| Images and clipboard | `src/widgets/display/image/`, `src/hooks/clipboard.rs`; separate later work |
| TypeScript and C bindings | `bindings/typescript/`, `src/ffi/`; existing defects remain in recon |
| Animation, screens, editing, Markdown, syntax, themes | `src/animation/`, `src/screen/`, `src/editor/`, `src/markdown/`, `src/syntax/`, `src/theme/`; outside first commitment |

Verification commands are owned by `.cairn/mechanisms/suprtui-renderer.md`
and `.cairn/mechanisms/embedded-terminal.md`. The renderer is maintained in
`src/backend/engine`; the embedded-session API is documented in
`docs/embedded-terminal.md`.
The initial project-wide failures are evidence in `docs/recon-evidence/`;
this commitment does not declare the entire legacy library production-ready.
