# Vocabulary

Status: Observed

- **App:** the application event loop and root component owner (`src/app.rs`).
- **RootComponent:** the caller's root that produces an Element (`src/app.rs:21`).
- **Element:** component output containing type, children, key, classes, and focus properties (`src/component/element.rs:42`).
- **NodeSpec:** layout-facing classes, text, and children (`src/layout/paint_tree.rs:48`).
- **Backend:** Reactive-TUI's application boundary for frame painting, presentation, resize, and events (`src/backend/mod.rs:16`). SuprTUI uses the same name for its narrower byte sink; qualify it when discussing output ownership.
- **Frame:** the complete grid of styled grapheme cells representing one application screen. SuprTUI compares successive frames; application tree patches are not terminal cell updates.
- **Grapheme cluster:** a displayed character that may contain multiple Unicode code points (`src/core/grapheme_cell.rs`). A wide cluster occupies two cells; its continuation cell is not another character.
- **Host terminal:** the terminal running the application, such as Kitty or Ghostty. It draws the glyphs on the user's screen.
- **Embedded terminal:** a virtual terminal interpreting a child program's output before that screen is composed into an application frame (`docs/renderer-foundation.md`). It is not the host renderer.
