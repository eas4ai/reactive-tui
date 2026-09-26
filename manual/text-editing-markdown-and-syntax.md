# Text editing, Markdown, and syntax

Crate modules: `editor`, `markdown`, `syntax`

## Purpose

These systems edit Unicode text, convert Markdown into styled terminal lines,
and apply language-aware syntax highlighting.

## Main API

- `GapBuffer` stores editable text.
- `Cursor` tracks editor movement and selection.
- `TextEditor` provides insertion, deletion, movement, sizing, and element
  output.
- `SyntaxEditor` combines editing with a syntax highlighter.
- `MarkdownRenderer` converts CommonMark and configured GFM extensions into
  styled lines and can retain source positions.
- `SyntaxHighlighter` highlights text or lines and supports incremental cache
  invalidation.
- `SyntaxResources`, `ThemeSet`, `LineCache`, and syntax theme types manage
  language definitions, themes, and cached lines.

## Basic use

Use `TextEditor` when content changes interactively. Use `SyntaxEditor` when an
editor needs language highlighting. For read-only Markdown, create a
`MarkdownRenderer`, set options, and render to styled lines for a display
element or surface.

## Behavior

Editor positions follow Unicode grapheme boundaries. The gap buffer moves its
gap near edits and avoids replacing the whole string for each change. Painting
maps logical positions into visible rows and columns.

Markdown parsing walks the document tree and emits styled runs for headings,
lists, code, links, quotes, emphasis, tables, and task items according to
options. Fenced code can pass through the syntax highlighter. Incremental
highlighting invalidates changed line ranges and reuses unaffected cached work.

## Limits

- Checked Markdown and syntax entry points reject oversized sources.
- Syntax detection uses an explicit language or file extension and can return
  plain text when no definition matches.
- Terminal-cell width and grapheme count differ for wide and combining text.
- Markdown output is terminal styled text, not a browser layout engine.

## Source map

- Editor exports: [`src/editor/mod.rs`](../src/editor/mod.rs)
- Markdown exports: [`src/markdown/mod.rs`](../src/markdown/mod.rs)
- Syntax exports: [`src/syntax/mod.rs`](../src/syntax/mod.rs)
- Markdown renderer: [`src/markdown/renderer.rs`](../src/markdown/renderer.rs)
- Unicode editor API tests: [`tests/api_editor_unicode.rs`](../tests/api_editor_unicode.rs)
- Syntax integration tests: [`tests/syntax_highlighting_integration.rs`](../tests/syntax_highlighting_integration.rs)

## Related chapters

- [Input widgets](input-widgets.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [FFI and TypeScript](ffi-and-typescript.md)

[Back to the manual](README.md)
