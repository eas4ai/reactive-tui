# Layout, style, and themes

Crate modules: `layout`, `theme`

## Purpose

Layout assigns terminal-cell rectangles to elements. Style controls color,
spacing, borders, text, opacity, gradients, positioning, clipping, and motion.
Themes supply reusable variables and color presets.

## Main API

- `LayoutEngine` wraps Taffy nodes and layout computation.
- `StyleBuilder` and inline style values define typed properties.
- `apply_utility_classes` parses utility class strings.
- CSS modules cover layout, sizing, spacing, typography, colors, gradients,
  effects, interaction states, accessibility states, variants, and animation.
- `Theme` stores a name and `ThemeVariables`, can extend another theme, and can
  apply classes through its variables.
- Theme presets include dark, light, high-contrast, Solarized Dark, and
  Gruvbox Dark.

## Basic use

Use builder classes for concise layouts. Use `StyleBuilder` when values are
computed or when typed construction is clearer. Pass a theme to the themed
utility-class path when classes reference variables.

## Behavior

The component bridge converts elements into layout `NodeSpec` values. Taffy
computes flex and grid geometry. The paint tree resolves classes and inline
styles, applies clipping and stacking, and writes the resulting cells and
placement metadata.

Theme extension starts from a base theme and replaces selected variables.
When the application has set no theme and the terminal reports a light
background (relative luminance above 0.5), the default backend on Unix makes
the light preset active before the first frame.
Utility classes are applied in input order, so later compatible utilities can
replace earlier values.

## Lengths, spacing and gaps

Every length is a count of terminal cells. A width of 24 is 24 columns and a
height of 3 is 3 rows.

- The number in a padding, margin, gap or space class is a count of cells,
  as it is in a width or height class. `p-1` pads one cell on each side,
  `px-2 py-1` pads two columns and one row, and `gap-2` leaves two cells
  between items.
- A cell is about twice as tall as it is wide. `px-2 py-1` looks even where
  `p-1` leaves more room above and below than beside.
- Whole numbers from 0 to 512 are accepted. A number with a fraction counts
  as the next whole number, so `gap-0.25` is one cell.
- `gap-x-N` sets the gap across and `gap-y-N` the gap down. Each keeps the
  other. `StyleBuilder::gap_x_px` and `StyleBuilder::gap_y_px` do the same
  from code.
- A gap is painted with exactly its number of cells at every width. Grid
  columns of equal weight differ by one cell at most when the width does not
  divide evenly. An item that spans columns ends where its last column ends.
- The columns of `grid-cols-N` and the rows of `grid-rows-N` each take
  their share of the container whatever their items hold. An item larger
  than its column does not widen the column; it reaches over the columns
  after it unless its own class cuts it, for example `overflow-hidden`.
- `col-span-full` and `row-span-full` span every track the grid's class
  names, from its first line to its last: the columns of `grid-cols-N`, the
  rows of `grid-rows-N`. A grid adds rows for the items that do not fit its
  columns, and a full span does not count those: in a grid with
  `grid-cols-2` and no `grid-rows-N`, `row-span-full` spans one row.
- `grid-cols-auto-fit-N` and `grid-cols-auto-fill-N` make as many columns of
  at least N cells as the container holds. `grid-rows-auto-fit-N` and
  `grid-rows-auto-fill-N` do the same for rows. The two differ when there
  are fewer items than tracks. `auto-fill` keeps the tracks that hold no
  item: two items in a container that holds four columns take two of the
  four. `auto-fit` gives the room of the empty tracks to the items: the two
  share the container.

The spacing classes counted in fours before: `p-1` was four cells and
`gap-4` sixteen. To keep the size of a class written for that scale, multiply
its number by four: `p-1` becomes `p-4`.

## The theme's color roles

A theme names its colors by role. A class names a role as `bg-<role>` or
`text-<role>`: `bg-surface`, `text-foreground`, `text-primary-foreground`.
Every built-in preset defines every role, and a widget that the widget bar
covers takes every color from a role, so it follows the application's
theme. `Theme::resolve_color` resolves a role, a palette name such as
`blue-500` or a hex value; `Theme::active` is the theme in use and
`Theme::set_active` or `App::set_theme` changes it, which shows in the
next frame.

| Role | What it colors |
|---|---|
| `background` | the page |
| `surface` | a panel, a card, a bar: what stands on the page |
| `foreground` | text on the page and on a surface |
| `text-muted` | quieter text: a shortcut, a disabled row, a hint |
| `border` | a border or a separator |
| `input` | a field the user types in |
| `ring` | the border of the widget that holds the focus |
| `hover` | the row under the pointer, and the current row of a widget without the focus |
| `selection`, `selection-foreground` | the current row, or the selected text, of the widget that holds the focus, and the text on it |
| `primary`, `secondary`, `accent` | the fills of buttons, titles and marks, each with its `-foreground` text |
| `success`, `warning`, `error`, `info` | the fills of a message by its kind, each with its `-foreground` text |
| `overlay` | the veil over the page behind a modal; it has an alpha |
| `shadow` | the shadow under a panel; it has an alpha |

In every preset, text contrasts with what it is drawn on by at least 4.5 to
1: each `-foreground` with its fill, `foreground` with `background`,
`surface`, `input` and `hover`, and `text-muted` with `background` and
`surface`. `ring` contrasts with `background` and `surface` by at least 3
to 1.

A theme that leaves a role out still resolves it:

- The text on a fill is black or white, whichever contrasts more with the
  fill.
- `selection` and `ring` are the theme's `primary`; `input` is its
  `surface`; `hover` is seven parts of its `surface` and one part of its
  `foreground`.
- Every other role comes from the built-in light preset when the theme's
  `background` is light (black contrasts with it more than white does),
  and from the dark preset otherwise.

So a theme written with only `background`, `foreground` and `primary` still
gives every widget a color. A name that is no role and that the theme does
not define resolves to nothing, as before.

## Limits

- Layout values are terminal-cell measurements after computation.
- Unknown utility classes do not create new behavior.
- Invalid or non-finite numeric style values return errors in checked paths.
- Terminal color output may be reduced according to detected capabilities.
- Absolute positioning and z-index affect paint order and clipping.

## Source map

- Layout exports and Taffy wrapper: [`src/layout/mod.rs`](../src/layout/mod.rs)
- Utility-class entry points: [`src/layout/css/mod.rs`](../src/layout/css/mod.rs)
- Typed styles: [`src/layout/style.rs`](../src/layout/style.rs)
- Theme API: [`src/theme/mod.rs`](../src/theme/mod.rs)
- The color roles and what a theme that leaves one out gets:
  [`src/theme/roles.rs`](../src/theme/roles.rs)
- Theme presets: [`src/theme/presets.rs`](../src/theme/presets.rs)
- Theme role tests: [`tests/theme_contract.rs`](../tests/theme_contract.rs)
- Styling API tests: [`tests/api_styling.rs`](../tests/api_styling.rs)
- Layout paint tests: [`tests/layout_paint_tests.rs`](../tests/layout_paint_tests.rs)
- Gap, spacing and grid tests: [`tests/layout_contract.rs`](../tests/layout_contract.rs)

## Related chapters

- [Elements, builders, and the virtual DOM](elements-builders-and-vdom.md)
- [Rendering and backends](rendering-and-backends.md)
- [Animation and screens](animation-and-screens.md)

[Back to the manual](README.md)
