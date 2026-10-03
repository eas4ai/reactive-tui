# Changelog

This file records user-visible changes to Reactive TUI. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- On a terminal that takes Kitty graphics or Sixel, a line, area, scatter,
  bar or candlestick chart built with `wgpu-graphics` draws its plot area
  as one pixel picture at the terminal's cell size: strokes an eighth of a
  cell high with round joins, discs for markers, one-pixel grid, reference
  and crosshair lines, gradient and pattern area fills, bars with
  whole-pixel edges and `.corner_radius(cells)`, candles, and the hover's
  crosshair, dots in halos, gliding band with faded neighbours and ring,
  eased over 150 ms (`reduced-motion` snaps). Axis text, the legend, value
  labels and the tooltip stay cell text over the picture; without pixels
  the chart draws in cells as before, byte for byte.
  `charts::set_graphics_options` picks the renderer, font and output for
  every chart of the process, and `REACTIVE_TUI_CANVAS=blocks` keeps the
  plots in cells (CHT-037, CHT-038, CHT-039).
- `LayoutInfo::terminal` (`TerminalInfo`) tells a component what the
  terminal takes, the cell size in pixels and whether Kitty graphics, Kitty
  shared memory or Sixel reach the screen, before its first frame and after
  every resize. `CanvasProps::described_by_parent()` leaves a canvas's
  screen-reader description to the widget that holds it.

- Every canvas of a process that draws with the same renderer options draws
  on one `rtui-canvas-*` thread, with one adapter, device, set of pipelines
  and glyph atlas, instead of a thread and a GPU connection per canvas; a
  worker an application starts and hands to several canvases serves them
  all, each with its own waiting scene. `GraphicsWorker::shared(&options)`
  gives the process's thread for those options (GFX-003).
- Canvas pictures are drawn one pixel per screen pixel at the terminal's
  cell size with no fixed cap; the 4096-pixel limit and the constants
  `graphics::MAX_WIDTH` and `MAX_HEIGHT` are gone, and the renderer's
  limits are `GraphicsWorker::limits` and `HybridRenderer::limits`
  (`PictureLimits`). `CanvasProps::cell_pixels(width, height)` pins fewer
  pixels per cell, and a picture that would exceed a hard limit (the
  renderer's, a frame's 64 MiB, a Kitty command's 12 million pixels, a
  Sixel picture's 64 MiB of text) is drawn with the most whole pixels per
  cell that fit; the terminal scales such a picture to its cells through
  the Kitty placement's `c` and `r` keys, and the Sixel encoder scales it
  before encoding (GFX-010).

- Actions dispatched to a `use_reducer` state from several threads are all
  applied: `ThreadSafeSignal::update_atomic` runs its callback under the
  signal's lock, so concurrent read-modify-writes lose nothing, and the
  reducer hook and the framework's own counters and states go through it.
  `ThreadSafeSignal::update` keeps running on a copy, so its callback may
  read and write the signal, and its documentation now says two concurrent
  `update` calls may overwrite each other. An `update_atomic` callback that
  calls back into its signal panics with a message instead of deadlocking.
- The native library builds with the `ffi` feature again, which the C
  header and the TypeScript binding build it with: the pointer trackers'
  statics no longer demand that the renderer be `Sync`, the ffi module is
  clean under `clippy -D warnings`, and its eight test targets build and
  run. **Breaking** for Rust code that calls the C functions directly: the
  115 exported functions that read or write through a pointer argument are
  `unsafe extern "C"` and say under `# Safety` what the caller must
  guarantee; C and TypeScript callers see no change. A per-commitment gate
  (BAR-011) builds, lints, documents and tests the crate with `ffi` and
  builds it with `ffi,wgpu-graphics` on every test host.
- The two-axis charts (line, area, scatter, bar, candlestick) meet the
  widget bar and gpui-kit 0.7.0 (docs/spec/charts.md, CHT-011 to CHT-036).
  **Breaking:** a line's dots are off until `.dot()` turns them on, as the
  reference's are; an area's `.fill(token)` sets its own fill color apart
  from the stroke and the fill shows at `fill_opacity` (0.4) over the chart
  background, a gradient fading toward the baseline; `ScatterChartBuilder::x`
  takes a number and places points on a linear x axis; a bar's `.label(..)`
  fills `DataPoint::value_label` instead of the tooltip's metadata; and
  `ChartsBuilder` holds its props, so its `Default` builds the same chart
  as `ChartProps::default()`. New on the typed builders, `ChartsBuilder`
  and `builder::chart()`: `interactive`, `aria_label`, `tooltip_title`,
  `tooltip_value`, `tooltip_value_color`, `tooltip_content`, `grid_dashed`,
  `y_padding`; for lines, areas and scatters `y_domain`, `point_count`,
  `y_axis`, `y_axis_label_placement`, `y_tick_count`, `y_tick_format`,
  `x_tick_count`, `grid_columns`, `reference_line`; for bars `fill_with`,
  `fill_gradient`, `label_color`, `label_axis`, `value_axis`,
  `value_tick_count`, `value_axis_label_placement`, `value_tick_format`,
  `band_count`, `band_tick_count`, `padding_inner`, `padding_outer`,
  `max_band_width`, `min_length`; for candlesticks `name`,
  `body_width_ratio`, `max_band_width`; and the `chart!` macro has a form
  for every chart type. The value axis covers stacked totals; `tick_margin`
  thins category labels on both orientations; value labels never overlap
  or leave the plot; candles reveal and animate; axes take the `border`
  role, labels, axis titles and legend names `text-muted`, the title
  `foreground`, the grid the new `--color-chart-grid` variable of every
  preset, the tooltip `surface` behind `foreground` text in a `border`
  frame, the crosshair `text-muted`, the band `hover` and the selected
  marker `ring`; the tooltip is opaque, opens with the category, shows a
  candle as open, high, low and close, lists the followed series first and
  measures its rows in cells; Up and Down choose the series the keys
  follow; a settled chart repaints when the theme changes; a chart whose
  worker cannot start says so; extreme values under tight limits draw a
  clipped segment instead of overflowing; ASCII mode covers the axes,
  grid, tooltip, crosshair, swatches and markers; and the screen reader is
  told the name from `aria_label` or the title, the type and state, the
  selection as the value, and one child per series. The widget catalog's
  Charts page shows every variant, and at 200 columns or more the large
  class at a real rectangle.

- On a terminal that takes pixels, `present` no longer waits while a
  canvas's picture is made ready (docs/spec/canvas.md, GFX-009). The
  backend copies the picture, writes it to shared memory or encodes it as
  base64 or Sixel on a thread of its own, `rtui-picture-` and a number, and
  writes it between frames when it is ready; the terminal keeps the last
  picture until then, and at most one picture of a canvas waits. With a
  new picture of 1920 by 960 pixels every frame, the App's wait in
  `present` at the 95th percentile fell from 38 ms with Kitty graphics
  sent in the command and 80 ms with Sixel to about 2 ms on the Linux
  development host, from 18 and 64 ms to under 1 ms on the Mac, and from
  133 and 487 ms to 8 and 6 ms on the Windows test tablet, where Sixel
  pictures now reach the terminal at about 13 a second. A slow encoder
  now costs pictures, not frames. `SuprTuiBackend::sync` waits up to 30
  seconds for the pictures being made ready as well and returns an error
  when some are still being made then, and
  `SuprTuiBackend::picture_threads` counts the pictures each thread has
  made ready. Of the pictures of a canvas finished before the backend
  writes one, only the newest is written; a picture sent through shared
  memory keeps its object until two newer ones of its canvas are written;
  on a Sixel or iTerm2 terminal a canvas leaves the cells an image or
  canvas above it covers to that plane; and a picture thread that cannot
  start makes the canvas show the reason and is tried again.
- **Breaking:** the table, the data table, the tree, the file explorer
  and the progress bar take every color from the active theme
  (docs/spec/data-widgets.md). A box sits on its parent's background with
  no fill of its own and its border in `border` (it was a grey fill under
  a white border); a header's title is `foreground` and the sort mark
  `primary`; the row with the keyboard cursor is `selection` while the
  widget holds the focus, a selected row `accent`, the row under the
  pointer `hover`, a disabled row `text-muted` (`TableProps::selected_style`
  and `TreeProps::selected_style` default to `None`, from `bg-blue fg-white`;
  `TreeProps::line_style` to `None`, from `fg-gray`, and lines, expanders
  and checkbox frames are `text-muted`); error lines are `text-error`; a
  progress bar's filled part is `primary` on a `border` track
  (`ProgressBarProps::color` and `background_color` default to `None`,
  from `bg-blue` and `bg-gray-200`). The file explorer's `>` and `*` marks
  before the cursor and the selected rows are gone; the rows show by
  color, and its hints, sizes and status line are `text-muted`.
- **Breaking:** the table, the data table, the tree and the file explorer
  fill the width and the height their parent allots, and the progress bar
  the width; `width` and `height` on `TableProps`, `TreeProps` and
  `FileExplorerProps` set a size instead, as a `w-N` or `h-N` class on a
  builder does. `TableColumn::new` is `DisplaySize::Flex(1.0)` with no
  minimum beyond its title and sort mark (it was `Auto` with a minimum of
  50 cells, and `builder::data_table().column()` asked for 100), so
  columns share the width by weight and a table scrolls sideways only
  when its columns' minimums exceed its width. A data table's
  `VirtualScrollConfig` defaults to a row height of 1 and a viewport
  height of 0, which means the table's own height (they were 32 and 400,
  which capped a data table at thirteen rows), and its pages are off
  unless `with_pagination(true, size)` or the builder's
  `.pagination(true, size)` turns them on (`DataTableProps::new` paged
  more than 25 rows and `builder::data_table()` always paged, 25 a page).
- A column whose every cell is a number sorts by its numbers in the table
  and the data table ("10" no longer sorts before "9"), integers exactly
  however long. A tree reveals a node the application selects, at first or
  later, by expanding its ancestors, giving it the cursor and scrolling it
  into view; collapsing the parent of the cursor's node moves the cursor
  to that parent instead of the first row. A data table's sort marks are
  painted in `primary` and tell the screen reader their direction, as the
  table's do (the wrapper had appended the arrow to the title).
- `aria_label` on `TableProps`, `TreeProps`, `FileExplorerProps` and
  `ProgressBarProps`, and `.aria_label(..)` on `builder::data_table()`,
  `builder::tree()`, `builder::file_explorer()`, `builder::progress_bar()`
  and the widget-level tree and progress bar builders, name the widget for
  the screen reader; without it the table, the tree and the explorer have
  no name and the progress bar takes its label ("File explorer" and
  "Progress" are gone). A table tells its row and column counts, each row
  its index, each cell its column index and a sorted header its direction;
  a tree row its level, position and count; an explorer row its position
  and count; an indeterminate progress bar keeps its minimum and maximum
  and leaves only its value unknown. The file explorer's status line is
  `text-error` while it carries an error.
- A table's Left and Right move the column cursor and bring its column
  into view (they scrolled sideways by one cell), `s` sorts by that column
  as a click on its header does, Shift with Left or Right narrows or
  widens the cursor's column by a cell when `resizable_columns` is set,
  Enter on a clickable cell reports the cell's own action (it reported
  `select`), and Space selects the cursor row. The cursor is kept apart
  from the selection (`TableState::cursor_row`): a table that takes the
  focus with nothing selected shows its cursor on the first row, and the
  first Down moves to the second.
- **Breaking:** the tabs, the accordion, the breadcrumb, the scroll view
  and the stack take every color from the active theme
  (docs/spec/layout-widgets.md). A tab's label is `text-muted` and the
  selected tab's `foreground`, underlined in the `Line` variant, on
  `surface` in `Enclosed`, on `secondary` in `Soft` and on `primary` in
  `Solid` (they were `bg-gray-700` and `bg-blue-600 text-white`); a badge
  is `text-success`, `text-warning`, `text-error` or `text-info` by its
  kind; an accordion's title is `foreground` and its glyph `text-muted`;
  a breadcrumb's segments and separators are `text-muted`, a clickable
  segment underlined, the current one `foreground`; a scroll bar's track
  is `border` and its thumb `text-muted`; tooltips are `surface`. The tab,
  header or segment with the keyboard focus is `selection` while the
  widget holds the focus, the one under the pointer `hover`, a disabled
  one `text-muted`: the `▶` and `→` before a tab, the `▶ ` before an
  accordion header and the `~label~` of a disabled tab are gone.
- **Breaking:** a default scroll view fills the width and the height its
  parent allots: `ScrollViewProps::viewport_width` and `viewport_height`
  default to 0, which means the parent's size (they were 80 and 24), and
  `builder::scroll_view()` gained `viewport_size()`, `viewport_width()`
  and `viewport_height()`. `ScrollViewProps::scroll_x` and the widget-level
  `ScrollViewBuilder` default to `false`, as `builder::scroll_view()` always
  did, so a scroll view looks the same from its props and either builder and
  its content wraps to the view's width unless horizontal scrolling is
  asked for. The bar takes a column or a row only while the content
  overflows in that direction; it took one whenever bars were on. A click on
  the bar's track scrolls one page toward the click and a drag of the thumb
  scrolls with it.
- **Breaking:** a stack fills the width its parent allots in a row parent
  too (`stack_element` sets 100% width); it took its children's width.
  `StackProps::aria_label` and `.aria_label(..)` on both stack builders name
  the stack for the screen reader; without it the stack has no name.
- A disabled tab bar (`TabsProps::disabled`) paints its selected tab's label
  `text-muted` with no variant fill, like its other tabs; a vertical tab bar
  in a column that is shorter than its tabs scrolls to the focused tab as a
  horizontal bar does.
- **Breaking:** a `Medium` tab's label has one cell of padding at each
  side (it had two; `Small` none, `Large` two, it had four) and a
  breadcrumb's segment one cell (it had four; none when compact), with the
  separator's spaces gone, so `/catalog/layout/widgets` with its icons is
  47 cells. A tab bar wider than its parent scrolls to keep the tab with
  the focus, or the selected tab, whole in view; it was cut at the edge.
  A breadcrumb keeps its first and last segment whole before it cuts a
  label.
- Screen readers: `aria_label` on `TabsProps`, `AccordionProps`,
  `BreadcrumbProps` and `ScrollViewProps` and on their builders names the
  widget; a tab list has no name unless set (it was "Tabs"), a tab panel
  is named by its tab's label alone. A tab, an accordion header and a
  breadcrumb segment report their position and the count of their set,
  and a scroll view its offsets and their ranges.
- The widget catalog's Layout page, the manual's layout-widgets chapter
  and goldens at 80 by 24 and 400 by 100 show the five widgets.

- **Breaking:** the text input, the checkbox, the radio button, the select,
  the slider and the button take every color from the active theme. A
  text input's and a select's field is `bg-input text-foreground`, its
  placeholder, the select's caret and line numbers `text-muted`; a
  control's frame (a field's two end cells, a checkbox's `[ ]`, a radio's
  `( )`) is `border` and `ring` while it holds the focus, and `error` on a
  text input whose value fails its validator; a check mark, a radio's dot
  and a slider's filled track are `primary`, the track `border`, the
  thumb `foreground`; labels are `foreground`, `text-muted` when
  disabled; the row under the pointer is `hover`; the text input's cursor
  is the field reversed, its selection `selection` (it painted `bg-white
  text-black`, `bg-blue-600 text-white`, `text-gray-500` and
  `text-red-500`). Focus, hover and disabled show by those colors alone:
  the `▶ ` before a focused control, the `🔒 ` and `❌ ` status cells, the
  underscores around a hovered label and the parentheses around a disabled
  one are gone, so a text input's field starts at its first cell (it
  started two cells in). `builder::button()` is `bg-secondary
  text-secondary-foreground` and `builder::primary_button()` `bg-primary
  text-primary-foreground`, each one row tall with one cell of padding at
  each side and `bg-selection` while focused; they were `px-16 py-8` in a
  fixed blue, and their text is `text-muted` when they are disabled.
  `builder::input()`, `styled_input()` and `search_input()` set no palette
  classes on the field. The input and autocomplete dialogs pass the `bg-`
  and `text-` classes of their `input` css class to the field's cells
  (`TextInputProps::field_class`, new), so an application's colors there
  still win; their goldens changed with the field.
- **Breaking:** a text input's field, a select's row and a slider's track
  fill the width their parent allots: `TextInputProps::width` and
  `SelectProps::width` default to `None` (they were `Some(30)`), a
  `SliderProps::width` of 0 (the default, it was 20) takes what the
  label, the value and the end labels leave, and a `w-N` class on the
  builder sets the width. A slider is one row: its label stands before the
  track, not above it. A select's list shows every option as far as the
  screen holds them, scrolling to keep the current row in view:
  `SelectProps::max_visible_items` defaults to `usize::MAX` (it was 5).
- **Breaking:** a select's open list and a text input's suggestion list are
  a panel in `surface` with a border in `border`, under the row, or above
  it when only the space above holds it, painted whole over the page (also
  inside a modal, a popover or a box that clips) without moving what is
  under the control; they were rows drawn inside the control's own box,
  which pushed the page down and were cut by a card. The list's current
  row is `bg-selection`, the option under the pointer `bg-hover` (the
  pointer moved the current row before; `SelectState` has a new field
  `hover_index`), the chosen option marked `●`; a pointer row is one lower
  than before, after the border.
- `TextInputProps`, `CheckboxProps`, `RadioButtonProps`, `SelectProps` and
  `SliderProps` have a new field `aria_label`, and their builders a method
  of that name: the name the screen reader hears apart from the visible
  label (a text input's placeholder names it otherwise, a select's
  placeholder too). A radio group's node carries its orientation and the
  chosen radio is selected as well as toggled; a text input's error line
  is an alert. A struct literal that names every field of one of those
  props no longer compiles; `..Default::default()` does.
- The widget catalog's Input widgets page paints its text input (its card
  was empty), opens with the text input focused, so Tab walks the
  controls and Enter opens the select, and has a Button card.

- **Breaking:** the modal, the popover, the toast and the confirmation,
  input, autocomplete, progress and wizard dialogs take every color from
  the active theme. `ModalProps::default()` is `bg-surface text-foreground`
  with a `bg-overlay` veil and a `text-muted` close button; it was white
  with black text and a half-black veil. `DialogTheme::default()` names the
  roles too: the box `bg-surface text-foreground`, the veil `bg-overlay`,
  the primary button `bg-primary text-primary-foreground`, the secondary
  `bg-secondary text-secondary-foreground`, the danger button `bg-error
  text-error-foreground`, each `bg-selection text-selection-foreground`
  while it holds the focus; it was a white box with blue buttons under any
  theme. Its `border_style` is empty: the box's border is the modal's own,
  in the `border` role (a `border-<color>` class there painted the box's
  background grey). `DialogThemes::light()`, `dark()` and `high_contrast()`
  take their colors from the preset of that name through the new
  `DialogTheme::of(&theme)`, and `minimal()` draws its buttons as text in
  the roles; their buttons had 16 cells and 8 rows of padding. A toast is
  painted in the fill of its kind (`success`, `warning`, `error`, `info`)
  with that fill's text, and `ToastType::Custom(classes)` paints the
  classes its string names, where it looked like `info`. An error line is
  `text-error`, a warning `text-warning`, a selected suggestion
  `bg-selection`. Every dialog's buttons take the dialog theme's looks:
  OK, Yes, Next and Finish are primary, Cancel, Back and Skip secondary.
  `Theme::hex(role)` writes a role's color as the literal a class can name.
- **Breaking:** a modal, a toast or a dialog whose width is not set is as
  wide as its content, title or buttons need plus one cell of padding at
  each side, and at most half the viewport; its content wraps at that
  width instead of scrolling sideways. A modal's content, title and footer
  have one cell of padding at each side; a `DialogTheme` title `px-1`. The
  input and autocomplete dialogs' field, the suggestion list and the
  progress dialog's bar are 36 cells wide, so those boxes are 40 cells
  unless a longer line widens them (they were a fixed 40). A confirmation
  dialog has no icon unless its options name one. A line breaks at a
  space only, so "sure?" is never split into "sure" and "?".
- **Breaking:** a modal, a dialog or a toast is placed on the screen and
  painted whole, also when the element that owns it stands inside a card
  or a box that clips its content; it was centered in that box and cut by
  it. A box at an edge or a corner keeps one cell from it (`ModalPosition::
  TopRight` and the rest were flush with the edge). `ModalProps` has two
  new fields, `offset`, cells the box is moved from its place, and
  `on_placed`, called with the box's position and size, so a struct
  literal that names every field no longer compiles; `..Default::default()`
  does. The dialog engine places each toast under the earlier toasts at
  its position (over them at a bottom position), one row apart, where they
  were painted on the same cells, and each dialog opened over another one
  row lower. The stacking order is a modal or dialog at 1000 and up, a
  popover at 2000, a toast at 2500 (`PopoverProps::z_index` was 1000 and a
  toast 2000), a menu panel at 3000.
- **Breaking:** a popover opens one row from its trigger, or one cell
  when beside it, with an arrow one row deep: `PopoverProps` has a new
  field `gap` (1 by default), the cells between the box and the trigger
  along the placement's axis, and `offset` moves the box from there (its
  default was `(0, 8)`, which is now `(0, 0)`; `PopoverArrow::size` was
  8). A struct literal that names every field of `PopoverProps` no longer
  compiles; `..Default::default()` does. Its box is `bg-surface
  text-foreground` with a border in `border` and one cell of padding
  inside it, and its arrow is a piece of the box, filled in `surface` with
  its outline in `border`; the box had no color, border or padding. The
  veil of `backdrop_filter` is `bg-overlay`. Opened by Enter or Space on
  its trigger, a popover moves the focus into its content when the content
  holds a focusable element and back to the trigger when it closes.
- A confirmation dialog tells the screen reader it is an alert dialog, not
  a dialog; a toast is labeled by its kind and described by its message;
  a popover's trigger reports whether the popover is open (it set a class
  no table knew). A popover is painted whole inside a modal or a box that
  clips its content. Alt with an arrow moves a draggable modal one cell
  and Alt and Shift with an arrow resize a resizable one. A confirmation
  button whose variant the dialog theme has no look for (Retry, a warning
  button) takes the fill of the role it names, where it painted with no
  fill and no padding. The progress dialog's bar is `bg-primary` on a
  `bg-border` track. A word keeps its punctuation when a line breaks, and
  a run without a space that is wider than the line (Han text, a path)
  still breaks at its word boundaries.
- **Breaking:** `DialogBounds`, `DialogMargin`, `DialogComponent::get_bounds`,
  `DialogUtils::calculate_size` and `DialogBuffer` are removed. The bounds
  were set by six dialogs and read by nothing; the size and position reach
  the modal through each dialog's options. The others had no caller.
- The widget catalog's popover demo opens from a primary button that holds
  the focus, so Enter opens it; its modal and dialog demos are centered on
  the screen over a veil, as an application shows them.
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
