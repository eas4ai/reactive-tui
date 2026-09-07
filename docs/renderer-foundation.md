# Renderer foundation assessment

Status: Draft — recommendation, not an agreed architecture
Date: 2026-09-07

The developer wants to rehabilitate Reactive-TUI and has identified rendering
as the starting point. Existing terminal libraries, including Alacritty, are
candidates. This assessment assumes applications draw inside an existing
terminal; a standalone graphical terminal would change the choice.

## What needs replacing or repairing

The repository contains renderer code rather than an empty renderer slot:
[Renderer](../src/core/renderer.rs:21) owns terminal setup, two surfaces, a diff
writer, and statistics. [CrosstermBackend](../src/backend/mod.rs:111) also owns
a grapheme surface, previous frame, and a separate diff/output path. The
[layout painter](../src/layout/paint_tree.rs:67) uses Taffy and paints into the
custom Surface. Choosing a new output library therefore requires deciding
which layer owns the authoritative frame and terminal lifetime.

Two source findings undermine the current contract:

- [App's first-render branch](../src/app.rs:284) does not populate previous_tree;
  it remains on the initial full-repaint path. A regression test must exercise
  multiple frames without resizing before claiming a repair.
- [DirectTtyBackend](../src/backend/direct_tty.rs:332) handles large patch sets
  and replacement/reordering but returns success without painting small
  Insert/Remove/Update sets. The [DebugBackend](../src/backend/mod.rs:659)
  always repaints the full tree, so its passing tests do not prove equivalent
  behavior in the real backend.

These are source findings, not completed runtime reproductions. A new output
dependency alone cannot repair App scheduling, event routing, or focus.

## Candidate responsibilities

| Candidate | What it supplies | Assessment for this project |
| --- | --- | --- |
| Ratatui core with its Crossterm backend | Cell buffers, frame drawing, terminal diffing/output, and a test backend. Custom code can paint buffers without adopting Ratatui's widget or layout APIs. | First candidate for a conventional TUI. Retain Taffy provisionally and paint its layout into the chosen buffer. Validate Unicode, clipping, colors, and any image requirements before selection. |
| Termwiz | A Surface with tracked changes and an optimized change stream for a terminal renderer. | Strong alternative when the desired foundation is a lower-level terminal surface and capability layer. Compare integration cost and required terminal features against the same proof scene. |
| alacritty_terminal | Terminal grid/state, escape parsing through vte, PTY/event-loop functionality, selection, and terminal semantics. | Relevant to an embedded shell/terminal widget. Its exposed library surface does not supply the application-to-host-terminal frame output layer needed here. Alacritty's GPU display is a separate integration problem. |

Primary references, checked 2026-09-07:

- [Ratatui rendering model](https://ratatui.rs/concepts/rendering/under-the-hood/)
- [Ratatui core](https://docs.rs/ratatui-core/latest/ratatui_core/)
- [Ratatui backends](https://ratatui.rs/concepts/backends/)
- [Termwiz Surface](https://docs.rs/termwiz/latest/termwiz/surface/struct.Surface.html)
- [Termwiz Terminal](https://docs.rs/termwiz/latest/termwiz/terminal/trait.Terminal.html)
- [Alacritty terminal library](https://docs.rs/alacritty_terminal/latest/alacritty_terminal/)

The candidate fit judgments are inferences from those APIs and the local
boundaries above. No candidate has been integrated or benchmarked here.

## Proposed first proof

Build a narrow renderer experiment before migrating the widget catalog:

1. Paint a Taffy layout containing styled text, borders, overlapping content,
   and clipped text into the candidate's frame buffer.
2. Change and remove text across consecutive frames; verify that both new
   cells and erased cells match the expected result.
3. Include wide characters and combining sequences; verify cell boundaries
   and replacement of wide glyphs by narrow text.
4. Resize the frame and compare the result with a fresh render at that size.
5. Drive a small interactive application through input, redraw, and exit in
   a pseudo-terminal; verify terminal restoration on normal and error exits.

Use deterministic cell assertions for composition and a pseudo-terminal test
for actual output/lifecycle. Compare full redraw and incremental output against
the same expected frame. Include unchanged-frame output in the measurements;
set performance thresholds only after measuring the scene.

The experiment should establish a single owner of terminal setup, output,
resize, and cleanup. Input and reactive scheduling need their own integration
proof even if rendering succeeds. Images, animation, TypeScript bindings,
and embedded shells remain recorded requirements to clarify, rather than
capabilities silently discarded during the renderer change.

## Decision pending

Provisional recommendation: evaluate Ratatui's rendering core first and Termwiz
against any terminal capabilities the core cannot meet. Evaluate
alacritty_terminal separately if embedded shell sessions are part of the product.
Confirm the intended application and required visual features before declaring
the rendering architecture Agreed or scheduling a broad migration.

## Supplied references: Alacritty and clipboard-rs

The local Alacritty 0.17.0 tree contains two distinct integration surfaces:

- [alacritty_terminal 0.26.0](../reference/alacritty-0.17.0/alacritty_terminal/Cargo.toml:1)
  is the terminal-emulation library.
- The application's [OpenGL renderer](../reference/alacritty-0.17.0/alacritty/src/renderer/mod.rs:119)
  requires a current GL context. Its draw method accepts application-specific
  RenderableCell and SizeInfo types plus a GlyphCache. It is declared as a
  private module of the [binary](../reference/alacritty-0.17.0/alacritty/src/main.rs:46).
  The application depends on crossfont, glutin, and winit in its
  [manifest](../reference/alacritty-0.17.0/alacritty/Cargo.toml:27).

Inference: its GPU renderer is a credible reference for a desktop-window
backend, but extracting it is maintained source integration, not simply adding
alacritty_terminal as a dependency. The earlier Ratatui recommendation is
conditional on targeting existing terminals. The developer has been asked
whether existing terminals, a desktop window, or both are intended.

[clipboard-rs](https://github.com/ChurchTao/clipboard-rs) is a candidate for a
separate clipboard service. Its current upstream manifest declares version
0.3.5, an optional image feature enabled by default, and optional Wayland
support. The source selects Wayland when enabled and available, falling back
to X11 on initialization failure. These observations concern upstream master,
not a pinned release verified in this checkout.
Sources: [manifest](https://github.com/ChurchTao/clipboard-rs/blob/master/Cargo.toml),
[platform dispatch](https://github.com/ChurchTao/clipboard-rs/blob/master/src/platform/mod.rs).

The existing [clipboard hook](../src/hooks/clipboard.rs:22) locates command-line
tools and executes copy/paste synchronously. Copy waits for child processes
without checking their success status, and the unavailable backend returns
success at [line 164](../src/hooks/clipboard.rs:164). This makes failure visible
to neither the caller nor its cached state in those cases.

Proposed integration: keep clipboard ownership outside the renderer, adapt the
hook to a service that reports actual errors, and keep blocking clipboard work
off the rendering loop. The upstream [Clipboard trait and watcher API](https://github.com/ChurchTao/clipboard-rs/blob/master/src/lib.rs)
return Results for operations and document that watching blocks until stopped.
Text round trips, unavailable services, and shutdown need controlled tests.
Native desktop clipboard access does not establish remote-session clipboard
behavior; that remains a separate requirement if existing terminals are the
target. No clipboard contents were read or changed during this assessment.


## SuprTUI assessment

The developer confirmed that applications target existing modern terminals,
including Kitty, GNOME Terminal, and Ghostty, and supplied the sibling SuprTUI
repository. The desktop-window question above is resolved.

SuprTUI is a directly relevant candidate. Its `src/render.rs:775` compares
current and next cell buffers and emits cursor positioning plus ANSI-styled
cell output. `src/render.rs:172` defines a writer-backed StdoutBackend;
`src/lib.rs:14` publicly exports rendering. Its `src/term_embedded.rs:1` is
instead a virtual terminal for parsing child-program output and composing that
screen into an OptimizedBuffer. Both can contribute, at different stages.
All SuprTUI paths in this section are relative to the supplied sibling checkout
`../suprtui` from the Reactive-TUI repository root.

Verification run in SuprTUI on 2026-09-07:
`cargo test --locked --test render_core --test render_terminal --test render_stdout --test term_embedded`.
Result: 22 passed, zero failed or ignored (6 + 5 + 3 + 8).
`cairn wake` reported Done for sys-small. That verdict is scoped to its current
commitment; it does not establish cross-terminal compatibility.

The tests cover frame publication, unchanged-frame skipping, cursor changes,
rollback, backend byte parity, lifecycle sequences, hit testing, image fallback,
and embedded terminal behavior. The stdout parity test uses Cursor<Vec<u8>>
(`tests/render_stdout.rs:45`); these checks do not exercise Kitty, GNOME Terminal,
or Ghostty directly.

Integration constraints to resolve in a prototype:

- Feed completed cell frames from Reactive-TUI layout into SuprTUI's
  OptimizedBuffer and let SuprTUI own frame comparison and ANSI output.
- Preserve grapheme clusters and wide-cell continuations across the bridge;
  do not assume the old char-based surface is a lossless intermediate.
- Reconcile thread ownership: Reactive-TUI's Backend requires Send + Sync
  (`src/backend/mod.rs:16` here), while SuprTUI's renderer uses an Rc/RefCell
  grapheme pool (`tests/render_stdout.rs:15` there).
- Verify flush and partial-write behavior with actual terminal I/O. SuprTUI's
  StdoutBackend uses write_all without an explicit flush in end_frame or
  write_out (`src/render.rs:212` there); external bytes already written cannot
  be rolled back after a partial I/O failure.
- Measure output size. SuprTUI currently emits per-cell cursor/style sequences
  instead of coalesced runs and assumes truecolor (`src/render.rs:803` there).

Recommendation: evaluate SuprTUI in the first renderer prototype before adding
another rendering framework. This recommendation reflects a matching API and
passing focused tests, not a completed integration or production-readiness claim.
