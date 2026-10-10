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

The five pieces that follow (number input, code input, rating, switch and
form) take the same roles (CTL-007): a field's and a code cell's frame in
`border` with `ring` while focused, digits and the number in `foreground`
on `input`, the step buttons `[-]` and `[+]` on `secondary` with
`secondary-foreground` text, lit rating stars in `text-warning`, a switch's
thumb in `primary`, a form's required marks in `text-error` and its
descriptions in `text-muted`. A piece shows focus, hover and disabled by
color alone, as the controls above do. Widths (CTL-008): a number input
fills its parent's width, its field taking what its two buttons leave,
unless `width` or a `w-N` or `w-full` class sets it; a code input is as
wide as its digit cells and the gaps between groups; a rating is as wide
as its stars; a switch is as wide as its box and its label; a form fills
the width its parent allots.

## Pixel looks

Where the terminal takes Kitty graphics or Sixel and the crate is built
with `wgpu-graphics`, each control draws its look as one pixel picture
over the cells it paints, around its text (`docs/spec/pixel-looks.md`,
PIX-001 to PIX-007). Everywhere else, with `REACTIVE_TUI_CANVAS=blocks`,
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
  is drawn at half over what is under it, its text cells on the same half
  fill and its text in `text-muted`.
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
- A number input's field is drawn as a text input's. Each of its buttons
  is a square of the cell height less two pixels with rounded corners in
  `secondary`, bearing a minus or a plus two pixels wide in
  `secondary-foreground`, in place of `[−]` and `[+]`; a button that
  cannot step, and both while the number input is disabled, are drawn at
  half. The value and the cursor stay cells.
- A code input's digit cell is a square of the cell height less two
  pixels with rounded corners in `input`, bordered one pixel in `border`,
  two pixels in `ring` for the current cell while the code input holds
  the focus, in place of `[` and `]`. The digit, `_` or dot stays cell
  text in the middle cell, which the square's color fills.
- A rating draws a five-pointed star of the cell height less two pixels
  in each of its cells, filled `warning` when lit and outlined one pixel
  in `border` (`ring` while focused) when not, in place of `★` and `☆`;
  a disabled rating is drawn at half.
- A switch's box is a pill of the cell height less two pixels tall over
  its four cells. On, it is filled `primary` with a round thumb in
  `primary-foreground` at its right end; off, it is `input` bordered one
  pixel in `border` with the thumb in `foreground` at its left end. While
  the switch holds the focus the pill is bordered in `ring`, and a
  disabled switch is drawn at half. The label stays cell text.

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
and `.auto_indent()`. `.on_change(f)` is called with the text after each
edit and `.on_submit(f)` with it on Enter (CMP-009). The field is `[text]`
on the `input` role; the
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
`.class()`, `.on_change(f)` with the new state) or with `CheckboxProps`. It paints `[✓]`, `[▬]` for a mixed
state, or `[ ]`, then its label. Space or Enter toggles it, as a click does.
The screen reader hears it as a check box with its label or `aria_label`
and whether it is checked, unchecked or mixed.

## Radio button

`radio_button()` builds one radio of a named group (`.group()`, `.value()`,
`.label()`, `.aria_label()`, `.checked()`, `.disabled()`, `.class()`,
`.on_change(f)` with the chosen value); the radios of a group share one
choice. `RadioButton` holds a whole group in
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
`.disabled()`, `.multiple()`, `.class()`, `.on_change(f)` with the chosen
value) or with `SelectProps`; the
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
`.class()`, `.on_change(f)` with the new value) or with `SliderProps`; the widget's `SliderBuilder` adds
`.range()`, `.orientation()`, `.show_value()`, `.show_labels()` and
`.width()`. It is one row: the label, the track `[════●────]` filling
what the label, the value and the end labels leave, and the value. Left
and Down lower the value by a step, Right and Up raise it, Page Up and
Page Down move a tenth of the range, Home and End jump to the ends; a
click on the track sets the value and a drag moves it. The screen reader
hears a slider with its label, value, minimum, maximum and step.

## Number input

`NumberInput` holds a number between a `[-]` and a `[+]` button, moved by a
step and kept inside an optional range. Build it with `number_input()`
(`.value()`, `.min()`, `.max()`, `.range()`, `.step()`, `.aria_label()`,
`.disabled()`, `.width()`, `.class()`, `.on_change(f)` with the new value,
`.on_submit(f)` with the value on Confirm) or with `NumberInputProps`
(`value`, `min`, `max`, `step`, `aria_label`, `disabled`, `width`). The
value shows the step's decimals: step 1 gives `2`, step 0.5 gives `2.0`.

It paints one row with no gap: `[−][2.5     ][+]`. The field's two end
cells are `border`, `ring` while it holds the focus, and its value is
`foreground` on `input`, with the cursor cell as a text input has it. The
buttons are three cells each, `Icon::Minus` and `Icon::Plus` on `secondary`
with `secondary-foreground` text. A button that cannot step further is
`text-muted`, and both are `text-muted` while the input is disabled.

Typing takes digits, `.` and `-` as a text input in its numeric mode does.
Backspace, Delete, Left and Right edit the text. Up and Down step by one
step, Page Up and Page Down by ten steps, and Home and End go to `min` and
`max` when they are set. A click on `[-]` or `[+]` steps, and the mouse
wheel over the piece steps (wheel up adds one step). A step never passes
`min` or `max`. When the focus leaves, the typed text is read, clamped into
the range and written in the step's decimals. Each change calls `on_change`
with the value. A disabled number input takes nothing.

The screen reader hears a spin button, named by its label or `aria_label`,
with its value, minimum, maximum and step. It advertises Increment,
Decrement and SetValue and acts on them as a slider does.

## Code input

`CodeInput` holds a one-time code or a verification number as a row of digit
cells, `length` cells (6 by default), with a gap after each `group` digits.
Build it with `code_input()` (`.length()`, `.group()`, `.value()`,
`.masked()`, `.label()`, `.aria_label()`, `.disabled()`, `.class()`,
`.on_change(f)` with the code after each change, `.on_complete(f)` with
the code) or with `CodeInputProps` (`length`, `group`, `value`, `masked`,
`label`, `aria_label`, `disabled`).

Each digit cell is three cells, `[1]`, on `input`. An empty cell shows `_`
in `text-muted`, a filled cell its digit in `foreground`, and a masked
filled cell `Icon::Dot` in `foreground`. Each cell's brackets are `border`,
and the current cell's brackets (the first empty cell, or the last cell
when all are filled) are `ring` while the input has the focus. With
`.group(3)` and the value `123`, it reads `[1][2][3] [_][_][_]`. Its width
is three cells per digit plus one for each gap between groups.

A digit fills the current cell. Backspace clears the last filled cell, and
any other character is ignored. A paste replaces the whole code: its digits
fill the cells in order from the first cell, its other characters are
dropped, and any cells it does not reach stay empty. Filling the last cell
calls `on_complete` with the code, and so does Confirm when every cell is
filled. Each change calls `on_change`. A click gives the input the focus.

The screen reader hears one text input, or a password input when it is
masked, labelled with `aria_label`, else `label`, else `Code`, and its
length, as `Code, 6 digits`. Its value is its digits, or bullets when it is
masked. It advertises SetValue and takes the requested string as a paste.

## Rating

`Rating` sets a whole number from 0 to `max` as a row of stars, one cell
each. Build it with `rating()` (`.value()`, `.max()`, `.aria_label()`,
`.disabled()`, `.class()`, `.on_change(f)` with the new value) or with
`RatingProps` (`value`, `max`, `aria_label`, `disabled`). `max` is 5 unless
set.

The first `value` stars are `Icon::Star` (`★`, `*` in ASCII) in
`text-warning`, and the rest are `Icon::StarOutline` (`☆`, `-` in ASCII) in
`border`, in `ring` while the rating holds the focus. A disabled rating
paints every star in `text-muted`. Three of five reads `★★★☆☆`.

Left and Down lower the value by one, and Right and Up raise it, clamped to
0 through `max`. Home sets 0 and End sets `max`. A digit from 0 to 9 sets
the value, clamped to `max`. A click on star n sets the value to n. Each
change calls `on_change`.

The screen reader hears a slider with its label or `aria_label`, a numeric
value, a minimum of 0, a maximum of `max`, a step of 1, and the value text
`n of m`. It advertises Increment, Decrement and SetValue and acts on them
as a slider does.

## Switch

`Switch` is an on-off control with a label. Build it with `switch()`
(`.checked()`, `.label()`, `.aria_label()`, `.disabled()`, `.class()`,
`.on_change(f)` with the new state) or with `SwitchProps` (`checked`,
`label`, `aria_label`, `disabled`).

It paints `[ ●] Label` when on and `[● ] Label` when off. The box is four
cells: the two brackets in `border` (`ring` while focused), the thumb
`Icon::Dot` in `primary` when on or in `text-muted` when off, and a space
in the other cell. The label is `foreground`, or `text-muted` while the
switch is disabled. The row under the pointer is `hover`.

Activate and Confirm toggle it, and so does a click anywhere on the row.
Each toggle calls `on_change` with the new state. A disabled switch takes
nothing.

The screen reader hears a switch, named by its label or `aria_label`, and
whether it is on or off.

## Form

`Form` lays out labelled controls as rows. Build it with `form()`
(`.field()`, `.labels_above()`, `.label_width()`, `.aria_label()`,
`.class()`) or with `FormProps` (`fields`, `labels`, `label_width`,
`aria_label`). Each field is a `FormField`, built with `FormField::new(label,
control)`, then `.required()` to mark it with `*` and `.description(text)`
to add a line under its control. The labels stand beside the controls by
default (`FormLabels::Beside`), and `.labels_above()` puts them above.

Beside the controls, the label column is as wide as the widest label. It
is one cell wider when any field is required, and the `*` follows the label
with no space, as in `Name*`. `.label_width()` sets the column width
instead. Each row is the label, one cell of gap, then the control, which
fills the rest. A description stands on its own row under its control,
starting at the control's first column, in `text-muted`. When the width
allotted to the form is less than the label column, one cell, and ten
cells of control, the labels go above whatever the setting. Above the
controls, each label is on its own row, the control fills the row below
it, and the description sits under the control at the form's first column.
Labels are `foreground` and the required mark is `text-error`.

Next and Previous move the focus through the fields in their order, and
each control takes its own keys and clicks. The form has no callbacks of its
own: each control reports through its own `on_change`.

The screen reader hears the form as a form, named by `aria_label` when it is
set. Each control is named by its label and described by its description,
and a required control is marked required. A disabled control takes no
keys and is heard as disabled.

## Screen reader requests

A control tells the screen reader which requests it takes, and the App
delivers each to the control whose node advertises it (CTL-005). A text
input takes SetValue and replaces its text with the request, filtered as
typing is, and reports the change. A slider takes Increment and Decrement,
one step each, and SetValue, snapped to its step and clamped to its range.
A select takes SetValue and chooses the option whose value, else label, is
the request; a select of several choices toggles that option and opens no
list. A disabled control advertises nothing, and a request a node does not
advertise changes nothing. A radio button is heard with its position in
its group and the group's count, a disabled radio counted (CTL-006),
whether the group is one `RadioButton` or radios built with
`radio_button().group(name)`.

The five pieces take the requests the screen reader makes (CTL-010). A
number input takes Increment and Decrement, one step each, and SetValue,
snapped to its step and clamped to its range, as a slider does. A rating
takes Increment, Decrement and SetValue on the same terms as a slider, and
a code input takes SetValue as a paste, its digits filling the cells in
order. A disabled piece advertises nothing, as the controls above do.

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
- Number input: [`src/widgets/input/number_input.rs`](../src/widgets/input/number_input.rs), builder [`src/builder/widgets/input/number_input.rs`](../src/builder/widgets/input/number_input.rs)
- Code input: [`src/widgets/input/code_input.rs`](../src/widgets/input/code_input.rs), builder [`src/builder/widgets/input/code_input.rs`](../src/builder/widgets/input/code_input.rs)
- Rating: [`src/widgets/input/rating.rs`](../src/widgets/input/rating.rs), builder [`src/builder/widgets/input/rating.rs`](../src/builder/widgets/input/rating.rs)
- Switch: [`src/widgets/input/switch.rs`](../src/widgets/input/switch.rs), builder [`src/builder/widgets/input/switch.rs`](../src/builder/widgets/input/switch.rs)
- Form: [`src/widgets/input/form.rs`](../src/widgets/input/form.rs), builder [`src/builder/widgets/input/form.rs`](../src/builder/widgets/input/form.rs)
- Input builders: [`src/builder/widgets/input.rs`](../src/builder/widgets/input.rs)
- Goldens: [`tests/input_widgets_goldens.rs`](../tests/input_widgets_goldens.rs), [`tests/snapshots/input_widgets`](../tests/snapshots/input_widgets)
- The button builders: [`src/builder/core.rs`](../src/builder/core.rs)
- Contract tests: [`tests/input_widgets_contract.rs`](../tests/input_widgets_contract.rs)
- Input widget tests: [`tests/api_widget_behavior/input.rs`](../tests/api_widget_behavior/input.rs)
- Select tests: [`tests/api_widget_behavior/select.rs`](../tests/api_widget_behavior/select.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
