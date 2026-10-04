# Input widgets

Crate modules: `widgets`

Widget module: `input`

## Purpose

Input widgets collect text, Boolean choices, one choice from a set, or a value
from a numeric range.

## Main API

- `TextInput`, `TextInputProps`, and `TextInputState` manage editable text,
  cursor position, selection, placeholder text, password display, and callbacks.
- `Checkbox`, `CheckboxProps`, and `CheckboxState` manage a Boolean value.
- `RadioButton` and its radio types manage one option. Named radio groups share
  selection by group name.
- `Select`, `SelectOption`, `SelectProps`, and `SelectState` manage a list and an
  open selection panel.
- `Slider`, `SliderProps`, and `SliderState` manage bounded numeric input.
- Each family exports a builder or is available through builder helpers.

## Basic use

Create props, set controlled or initial state as required by the widget, and
return the widget element from a root or component. Register change and submit
callbacks in props or through the matching builder.

## Behavior

Input widgets receive routed key and mouse events. They update retained widget
state, invoke callbacks once for accepted actions, and request redraws when the
visible state changes. Text input uses grapheme-aware cursor and selection data
and exposes accessible text without revealing password content.

Select and radio controls keep selection within the available options. Sliders
clamp values to their range and step. Disabled controls remain visible but do
not accept actions.

## Colors and sizes

Every control takes its colors from the active theme's roles
(`docs/spec/input-widgets.md`, CTL-001). A text input's and a select's
field is `bg-input text-foreground`; a placeholder, the select's caret and
a text input's line numbers are `text-muted`. A control's frame (the two
end cells of a field, a checkbox's `[ ]`, a radio's `( )`) is `border`, and
`ring` while the control holds the focus; a text input whose value fails
its validator paints its frame in `error`. A check mark, a radio's dot and
a slider's filled track are `primary`; the slider's track is `border`, its
thumb `foreground` and `ring` while focused. Labels are `foreground`, or
`text-muted` when the control is disabled, and the row of a checkbox, a
radio or a select's option under the pointer is `hover`. Focus, hover and
disabled are shown by these colors alone: no glyph is added and no text
changes.

A text input's field, a select's row and a slider's track fill the width
their parent allots (CTL-002): a `w-N` class on the builder, or `width`
on the props, sets that width instead. A checkbox, a radio and a button
are as wide as their box or padding and their label need.

## Pixel looks

Where the terminal takes Kitty graphics or Sixel and the crate is built
with `wgpu-graphics`, each control draws its look as one pixel picture
over the cells it paints, around its text (`docs/spec/pixel-looks.md`,
PIX-001 to PIX-005). Everywhere else, with `REACTIVE_TUI_CANVAS=blocks`,
or with a process-wide `GraphicsOptions` whose output is blocks, it draws
the cell look above, byte for byte. The picture is drawn on the drawing
thread the canvases share, kept across frames and sent again only when
the look changes; a cell that holds a glyph is cut out of it and painted
in the look's flat color, so the text cell and the picture beside it are
one color, and the text stays cell text for the screen reader. Until the
first picture is ready the cells show the text alone and none of the cell
look's glyphs.

- A text input's field is a rounded rectangle in `input`, its radius a
  quarter of the cell height, with a one-pixel border in `border`, two
  pixels in `ring` while it holds the focus and in `error` while its
  value is invalid, in place of the `[` and `]` cells. Its text,
  placeholder, line numbers, cursor and selection stay cells on `input`,
  and the picture shows in the blank part of the field. A disabled field
  keeps its fill, with its text in `text-muted`.
- A checkbox's box is a square of the cell height less two pixels with
  rounded corners, bordered in `border` (`ring` while focused), filled
  `primary` with a check mark in `primary-foreground` when checked and a
  dash when mixed. A radio's circle is bordered the same way, with a dot
  of half its size in `primary` when chosen. A disabled box or circle is
  drawn at half over what is under it.
- A horizontal slider's track is a bar four pixels tall with rounded ends
  in `border`, filled in `primary` to the exact pixel of its value, with a
  round thumb of the cell height less two pixels in `foreground` (`ring`
  while focused) centered on that pixel; a disabled slider is drawn at
  half. A vertical slider keeps its cells. A click or a drag still sets
  the value by the cell under the pointer.
- `button()` and `primary_button()` are `rounded` with `focus:ring-2
  focus:ring-ring`: a rounded fill in `secondary` or `primary` (see
  [Layout, style, and themes](layout-style-and-themes.md#rounded-boxes-with-pixels)),
  a two-pixel ring in `ring` while focused, the fill at 90 percent under
  the pointer and at half when disabled.

`cargo test --features wgpu-graphics --test pixel_looks` draws each
control on a Kitty host and compares its picture with the reference under
`tests/snapshots/pixel-looks`.

## Text input

`TextInput` edits a line, or several lines, of text. Build it with
`text_input()` (`.value()`, `.placeholder()`, `.aria_label()`,
`.max_length()`, `.disabled()`, `.readonly()`, `.input_type()` for
`"password"`, `"number"` or `"email"`, `.class()`) or with
`TextInputProps`; the widget's own `TextInputBuilder` adds `.multi_line()`,
`.password()`, `.numeric()`, `.validator_pattern()`, `.error_message()`,
`.suggestions()`, `.show_line_numbers()`, `.wrap_text()`, `.tab_size()`
and `.auto_indent()`. The field is `[text]` on the `input` role; the
cursor cell is the field reversed, the selection `selection`, and an error
line under the field `text-error`. The suggestion list opens as a panel
under the field, or above it when only the space above holds it, painted
whole over the page; Up and Down move in it, Tab or a click accepts, Escape
closes it. The screen reader hears the field by its `aria_label` or its
placeholder, its text (bullets for a password) and its error line as an
alert.

## Checkbox

`Checkbox` holds a yes or no. Build it with `checkbox()` (`.label()`,
`.aria_label()`, `.checked()`, `.indeterminate()`, `.disabled()`,
`.class()`) or with `CheckboxProps`. It paints `[✓]`, `[▬]` for a mixed
state, or `[ ]`, then its label. Space or Enter toggles it, as a click does.
The screen reader hears it as a check box with its label or `aria_label`
and whether it is checked, unchecked or mixed.

## Radio button

`radio_button()` builds one radio of a named group (`.group()`, `.value()`,
`.label()`, `.aria_label()`, `.checked()`, `.disabled()`, `.class()`); the
radios of a group share one choice. `RadioButton` holds a whole group in
one widget, vertical or horizontal, built with `RadioButtonBuilder`
(`.option()`, `.disabled_option()`, `.selected()`, `.orientation()`,
`.aria_label()`). Each radio paints `(●)` or `( )` and its label. Up and
Down, or Left and Right, move between the options; Space or Enter chooses
the focused one, as a click does. The screen reader hears the group with
its orientation and each radio with its label, toggled and selected when
it is the choice.

## Select

`Select` chooses one option from a list, or several through
`select().multiple(true)`. Build it with `select()` (`.option()`,
`.options()`, `.selected()`, `.placeholder()`, `.aria_label()`,
`.disabled()`, `.multiple()`, `.class()`) or with `SelectProps`; the
widget's `SelectBuilder` adds `.width()` and `.max_visible_items()`. The
row is a field, `[value ▾]`, that fills its parent's width. Enter, Space,
Up or Down opens the list, as a click on the row does: a panel in
`surface` with a border, under the row, or above it when only the space
above holds it, painted whole over the page without moving what is under
it. It shows every option as far as the screen holds them, scrolling to
keep the current row, which is `selection`, in view; the option under the
pointer is `hover`, and the chosen option carries a `●`. Up and Down move, Home, End, Page Up and Page Down jump,
typed letters jump to the first option that starts with them, Enter or
Space chooses and closes, Escape closes, and so does losing the focus. The
screen reader hears a combo box named by its `aria_label` or placeholder
with its value and whether it is expanded, and each option with its label
and whether it is selected.

## Slider

`Slider` sets a number in a range. Build it with `slider()` (`.value()`,
`.min()`, `.max()`, `.step()`, `.label()`, `.aria_label()`, `.disabled()`,
`.class()`) or with `SliderProps`; the widget's `SliderBuilder` adds
`.range()`, `.orientation()`, `.show_value()`, `.show_labels()` and
`.width()`. It is one row: the label, the track `[════●────]` filling
what the label, the value and the end labels leave, and the value. Left
and Down lower the value by a step, Right and Up raise it, Page Up and
Page Down move a tenth of the range, Home and End jump to the ends; a
click on the track sets the value and a drag moves it. The screen reader
hears a slider with its label, value, minimum, maximum and step.

## Button

`button()` builds an element with the secondary look (`bg-secondary
text-secondary-foreground`) and `primary_button(text, on_click)` one with
the primary look, each one row tall with one cell of padding at each side,
`selection` while it holds the focus, and its text `text-muted` when it is
disabled; with pixels each is a rounded fill with a ring while focused
(see [Pixel looks](#pixel-looks)). `.on_click()` makes a button
interactive: Enter or Space presses it, as a click does, and the screen
reader hears it as a button.

## Limits

- Controlled values must be updated by the owning application after callbacks.
- Empty option lists and zero-size layouts do not provide a selectable item.
- Text positions are based on Unicode text boundaries, not raw terminal bytes.
- Password mode changes exposed and painted text; it does not encrypt the
  stored value.

## Source map

- Input exports: [`src/widgets/input/mod.rs`](../src/widgets/input/mod.rs)
- The look every control shares: [`src/widgets/input/look.rs`](../src/widgets/input/look.rs)
- The list panel of a select and of suggestions: [`src/widgets/input/panel.rs`](../src/widgets/input/panel.rs)
- Text input implementation: [`src/widgets/input/text_input.rs`](../src/widgets/input/text_input.rs)
- Select implementation: [`src/widgets/input/select.rs`](../src/widgets/input/select.rs)
- The button builders: [`src/builder/core.rs`](../src/builder/core.rs)
- Contract tests: [`tests/input_widgets_contract.rs`](../tests/input_widgets_contract.rs)
- Input widget tests: [`tests/api_widget_behavior/input.rs`](../tests/api_widget_behavior/input.rs)
- Select tests: [`tests/api_widget_behavior/select.rs`](../tests/api_widget_behavior/select.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
