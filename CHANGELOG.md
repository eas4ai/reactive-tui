# Changelog

This file records user-visible changes to Reactive TUI. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Every built-in theme preset defines fourteen more colors, each a role a
  widget names as `bg-<role>` or `text-<role>`: the text on each fill
  (`primary-foreground`, `error-foreground`, and so on), `selection` and
  `selection-foreground` for the current row of the widget that holds the
  focus, `hover` for the row under the pointer, `input`, `ring`, `overlay`
  (the veil behind a modal) and `shadow`. Text contrasts with its fill by
  at least 4.5 to 1 in every preset. A theme that leaves a role out still
  resolves it: the text on a fill as black or white by contrast, `selection`
  and `ring` from `primary`, `input` from `surface`, `hover` from `surface`
  and `foreground`, and every other role from the light or the dark preset
  by the theme's `background`. So an application's theme written before
  these roles existed keeps working, and a class that names a role always
  colors its element. The image widget's captured screen under a theme
  without a foreground takes the dark preset's, where it inherited its
  parent's.
- **Breaking:** the menu bar, the context menu, the popup menu and the
  dialog menu take every color from the active theme. The default
  `MenuStyle` names roles: `bg-surface text-foreground` for a panel,
  `bg-selection text-selection-foreground` for the current row of a menu
  that holds the focus, `bg-hover` for one that does not, `text-muted` for
  a disabled row, a shortcut and a separator. It named palette colors
  before (`bg-gray-800`, `bg-blue-600`), and a menu from the builder was
  white with black text. `MenuTheme::Dark`, `Light` and `HighContrast` take
  their colors from the preset of that name; `MenuStyle::of(&theme)` does
  the same for any theme. `MenuStyle` has two new fields, `shadow_classes`
  and `veil_classes`, so a struct literal that names every field no longer
  compiles; `..Default::default()` and the setters do. The current row and
  a disabled row are one color from end to end: their icon and shortcut
  take the row's text color.
- **Breaking:** a menu panel has no default limit of width or rows. It
  paints its widest row whole and every row the viewport holds, and
  scrolls beyond that. `MenuStyle::max_width` defaults to `None` (it was
  50 cells) and `max_visible_items` and `max_dropdown_height` to no limit
  (they were 10 rows). A context menu made through the builder is closed
  until the user opens it; it opened at the screen's first cell when it was
  mounted.
- A menu panel opens beside what opened it and no longer covers it: under
  a menu bar's title or over it when only the space above holds the panel,
  right of a submenu's parent row or left of it, at the side a popup
  placement names or the opposite one. Shift+F10 opens a context menu that
  holds the focus, at its first trigger area or its own first cell.
- A menu panel, its shadow and the veil of a dialog menu are painted whole
  over everything on the screen, also from inside a modal, a popover or a
  box that clips its content, where they were cut or hidden. A dialog menu
  centers in the viewport. `StyleBuilder::unclipped` lets any element out
  of its ancestors' clip for the same purpose.
- The menu bar fills the width its parent allots in a plain box, where it
  was as wide as its titles. A dialog menu's buttons take the theme's
  `primary` fill and its text; they were blue with white text under any
  theme. A dialog menu's title, message and buttons stand one cell in from
  the border, as its rows do.
- The widget catalog changes its theme with F3 and names the active one in
  its header; its frame and cards follow the theme, and each card has one
  cell of padding. Its menu demos show a full set of rows, open in their
  cards, and take the focus.
- **Breaking:** the number in a padding, margin, gap or space class is a
  count of cells, as in the width and height classes: `p-1` pads one cell
  and `gap-2` leaves two. These classes counted in fours before: `p-1` was
  four cells and `gap-4` sixteen. To keep the size of an existing class,
  multiply its number by four. Whole numbers from 0 to 512 are accepted,
  and a number with a fraction counts as the next whole number.
  The gap argument of `builder::flex_row`, `flex_col` and `grid_layout` is
  the number of such a class and counts in cells too. So do the declarative
  grid's `gap`, `column_gap` and `row_gap`, which its documentation already
  gave in cells: `gap(1)` was four cells, and the default gaps were none
  across and four down where they are now one and one. The TypeScript
  binding's `CSSUtilities` (`flex`, `grid` and `spacing`) and the layouts
  built on it write the caller's numbers into these classes, so their gap,
  margin and padding numbers count in cells as well.
- **Breaking:** `grid-cols-auto-fit-N`, `grid-cols-auto-fill-N` and their
  row forms make as many tracks of at least N cells as the container holds.
  They made N tracks before. `col-span-full` and `row-span-full` span the
  grid they are in; they spanned twelve tracks before.
- **Breaking:** the columns of `grid-cols-N` and the rows of `grid-rows-N`
  take their share of the container whatever their items hold (each track
  is `minmax(0, 1fr)`). An item larger than its share widened its track
  before and pushed the tracks after it out.
- **Breaking:** removed `layout::css::parsers::parse_spacing_pixels`, which
  nothing called and which kept the scale of fours.
- Fixed gaps between neighbouring boxes that were painted one cell too wide
  or too narrow when their container started at a fraction of a cell, for
  example beside a box of a third of the screen. A gap is painted with
  exactly its number of cells at every width.
- Fixed `gap-x-N`, `gap-y-N`, `space-x-N` and `space-y-N` setting the gap
  of the other direction to zero. Added `StyleBuilder::gap_x_px`,
  `gap_y_px`, `col_span_full` and `row_span_full`.
- Fixed the data table growing taller on every layout while its filter and
  column panels were open.
- Changed the license from the MIT License to The Reactive TUI License,
  Version 1.0 (`LICENSE`, SPDX `LicenseRef-ReactiveTUI-1.0`): the MIT License
  with two riders. Offering Reactive TUI or a copy of it to developers as a
  framework needs a Framework License, and an organization that earns money
  from software built with Reactive TUI and has revenue above USD 1,000,000
  a year must sponsor the project. Versions published under the MIT License
  alone stay under it.

## [1.0.0] - Unreleased

- Vendored the companion crates under `crates/` as path-only workspace
  members and marked them `publish = false`: no companion publishes.
- Replaced syntect 5.3.0 with lumis 0.13.1 for syntax highlighting;
  bincode 1.3.3 and yaml-rust 0.4.5 leave the dependency graph.
- Added `Ref::update_atomic` for linearizable read-modify-write while
  keeping the spec'd reentrant snapshot semantics of `Ref::update`.
- Mapped named forms to Landmark and unnamed forms to Panel, resolved
  hyperlink start/end offsets through the text model, and pinned
  focused/selected states with adapter tests; screen-reader output
  verified against Orca speech.

## [0.1.0] - 2026-09-14

This is the first supported, non-yanked release. Earlier 0.0.x packages were
development snapshots and are not a compatibility baseline.

### Added

- Retained applications, keyed components, reactive signals, hooks, and
  coalesced wake notifications.
- CSS-like flex and grid layout with cell-based painting and Unicode grapheme
  handling.
- Retained controls for text input, tables, trees, menus, dialogs, tabs,
  accordions, wizards, charts, images, and terminal views.
- A SuprTUI renderer with bounded frames, terminal restoration, host input,
  image composition, and platform-specific terminal support.
- Optional embedded Unix PTY sessions interpreted by `libghostty-vt`.
- Linux AccessKit and AT-SPI accessibility support.
- An optional C ABI with generated C and TypeScript consumer bindings.
- A source-grounded framework manual under `manual/`.

### Changed

- Vendored the maintained Crossterm and SuprTUI implementations as in-repo
  companion crates under `crates/` (never published) while preserving the
  `crossterm` and `suprtui` Rust import names inside Reactive TUI.
- Replaced the pinned `libghostty-vt` git dependency with crates.io version
  0.2.1.
- Limited crate archives to public source, legal notices, the manual, and
  release documentation.
- Declared Rust 1.91 as the minimum supported Rust version.

### Platform limits

- `embedded-terminal` is available on Unix and requires Zig 0.15.2, as required
  by `libghostty-vt` 0.2.1.
- `simd` requires nightly Rust because it uses `portable_simd`.
- Linux screen-reader integration requires an AT-SPI desktop session.
- Windows PTY support uses the pinned Microsoft OpenConsole runtime included in
  the crate source.

[Unreleased]: https://github.com/eas4ai/reactive-tui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/eas4ai/reactive-tui/releases/tag/v0.1.0
