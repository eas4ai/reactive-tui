# Input widgets

Prefix: CTL

The input family is the controls an application collects a value with: the
text input, the checkbox, the radio button, the select, the slider
(src/widgets/input, src/builder/widgets/input.rs,
src/builder/specialized.rs) and the button, an interactive element with a
look (`builder::button()` and `builder::primary_button()`,
src/builder/core.rs:70-72, src/builder/layout.rs:100-109). A control shows
a value or a choice, changes it on a key or a click, and reports the change
through a callback. Read on 2026-09-30, in the widget catalog under the
dark and the light preset at 240 and 100 columns: the text input's card is
empty, although the same widget in the same card paints on the debug
backend; the checkbox, the radio button, the select and the slider are
text in the page's color, and no control names a role of the theme. A
control shows its state with glyphs: a `▶ ` before a focused control
(src/widgets/input/checkbox.rs:134, src/widgets/input/radio_button.rs:300,
src/widgets/input/slider.rs:393), the label between underscores under the
pointer and in parentheses with a `🔒` when disabled
(src/widgets/input/checkbox.rs:132-160). The text input paints its cursor
`bg-white text-black`, its selection `bg-blue-600 text-white`, its
placeholder and line numbers `text-gray-500` and its error line
`text-red-500` (src/widgets/input/text_input/paint.rs:275-354), and the
input and autocomplete dialogs inherit that. A text input is 30 cells wide
whatever its parent gives it and whatever class the builder sets
(src/widgets/input/text_input.rs:244, src/widgets/input/text_input/paint.rs:86);
a select is 30 cells wide (src/widgets/input/select.rs:281-283) and draws
its open list inside its own box, under its row, so opening it moves the
content below (src/widgets/input/select.rs:297-304); a slider's track is
20 cells (src/widgets/input/slider.rs:174), so at 100 columns its value
wraps under its label. `builder::button()` is `px-16 py-8 bg-blue-500
text-white`, 16 cells by 8 rows of padding in one blue under every theme.
The select's node has a value and no name (src/widgets/input/select.rs:268),
the radio group's node no orientation (src/widgets/input/radio_button.rs:211),
and the text input's error line no role. The keyboard already reaches every
action: Space and Enter toggle and choose, arrows move a radio's focus and a
slider's value, Enter or Space and the arrows open a select, Escape closes
it. What the widget bar asks of every reworked widget is in quality-bar.md
and is not repeated here. gpui-kit 0.7.0 paints a control's frame in
`input`, its focus in `ring`, a checked box and a slider's bar in `primary`,
placeholders and disabled text in `muted_foreground`, and its select's list
as a popover over the page.

## Observed

(none yet)

## Draft

[CTL-001] A control whose style the application did not set MUST look the same whether it is built from its props or through any of its builders, and MUST paint: a text input's or a select's field in `input` with `foreground` text, its placeholder, a select's caret and a text input's line numbers in `text-muted`; a control's frame (a field's two end cells, a checkbox's `[` and `]`, a radio's `(` and `)`) in `border`, in `ring` while the control holds the focus; the text input's cursor cell in `foreground` with `input` text, its selection in `selection` with `selection-foreground` text, its error line in `text-error`; the mark of a checked or mixed box and the dot of the chosen radio in `primary`; a slider's track in `border`, its filled part in `primary`, its thumb in `foreground`, in `ring` while the slider holds the focus; a select's open list as a panel in `surface` with `foreground` text and a border in `border`, its current row in `selection` with `selection-foreground` text, the chosen option's mark in `primary`; `builder::primary_button()` in `primary` with `primary-foreground` text and `builder::button()` in `secondary` with `secondary-foreground` text, each with one cell of padding at each side and none above or below, in `selection` with `selection-foreground` text while it holds the focus; a control's label and text in `foreground`, in `text-muted` when it is disabled; and the row of a checkbox, a radio or a select's option under the pointer in `hover`. A control MUST show focus, hover and disabled by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, a default text input, checkbox, radio button, select, slider or button built through a builder paints a cell in another color than the same control built from its props; a field, frame, cursor, selection, error line, placeholder, caret, line number, mark, dot, track, filled part, thumb, panel, border, current row, button, label or hovered row is painted in a color other than its role's; a control paints a cell in a color no role of the theme has; or a control with the focus, under the pointer or disabled paints a glyph outside its box, its field, its track or its label, or paints its label with other characters than the label's own.
Mechanism: input-widgets
Rationale: A control's state must be visible under every preset, and color is the cue a terminal has; the `▶`, underscores and parentheses were stand-ins for it.
Status: Agreed 2026-09-30

[CTL-002] A text input, a select or a slider whose width the application did not set MUST fill the width its parent allots: the text input's field and the select's row span it, and the slider's track takes what its label, its value and its end labels leave; `width` on the props and a `w-N` or `w-full` class on the builder MUST set that width instead. A checkbox, a radio button and a button MUST be as wide as their box or padding and their label need. A select's open list MUST be as wide as its widest option and at least as wide as its row, and MUST paint every option, as far as the viewport holds them, scrolling to keep the current row in view; no default MAY limit the number of options it shows.
Falsifier: In a viewport of 240 by 60 cells, a default text input, select or slider inside a box of 100 cells paints a field, a row or a track with labels that ends before the box's last cell, or `.class("w-40")` on its builder paints one of another width than 40 cells; a checkbox, a radio button or a button with a 10-cell label paints a row wider than that label and its box or padding need; or a default select of 30 options paints fewer than 30 rows when opened, or cuts the label of its widest option.
Mechanism: input-widgets
Rationale: BAR-003 asks a widget to fill the rectangle its parent allots; a fixed 30 cells was a pixel number read as cells, and five visible options was the desktop's habit.
Status: Agreed 2026-09-30

[CTL-003] A select's open list MUST open under its row, or above it when only the space above holds it, and MUST be painted whole over what was on the screen when it opened, also when the select stands inside a modal, a popover or a box that clips its content, without moving that content; it MUST close on Escape, on a choice, and when the select loses the focus, and the row MUST show the choice at once. A text input's suggestion list MUST open and be painted the same way.
Falsifier: In a viewport of 240 by 60 cells, a select opened on the viewport's last row has a row of its list unpainted although the space above holds the list; opening a select moves the element under it; a select inside a box of three rows that clips its content, or inside a modal, has a cell of its open list painted over by that box or modal; Escape, Enter on a row or Tab away leaves the list open; after Enter on a row the select's row shows the old value; or a text input's suggestion list, opened inside a box that clips its content, has a cell painted over by that box.
Mechanism: input-widgets
Rationale: gpui-kit's select opens its list as a deferred overlay (G/select.rs:601-604); ours drew it inline, which pushed the page down and was clipped by a card.
Status: Agreed 2026-09-30

[CTL-004] The screen reader MUST be told each control's name: its label, or the `aria_label` its props or its builder set, or a text input's placeholder when it has no label; and its state: a checkbox checked, unchecked or mixed; a radio selected as well as toggled, with its group's orientation; a select's name apart from its value, whether it is expanded, and each option's label and whether it is selected; a slider's value, minimum, maximum and step; a text input's text, or bullets for a password, and its error line with the role `Alert`; and disabled for any of them. Every action a control takes from the pointer MUST have a key: a click on a box, a radio, a button or a select's row is Space or Enter, a click or a drag on a slider's track is an arrow, Page Up, Page Down, Home or End, and a click on a select's caret is Enter, Space, Up or Down.
Falsifier: A control's accessibility node has no label although its props name a label, an `aria_label` or a placeholder, or its label is the visible label although an `aria_label` is set; a chosen radio's node is not marked selected, or its group's node has no orientation; a select's node has no label or does not report its expanded state, or an option's node lacks its label or its selected state; a slider's node lacks its value, minimum, maximum or step; a text input's error line has a role other than `Alert`; a disabled control's node is not marked disabled; or an action of a control can be taken by the pointer and by no key.
Mechanism: input-widgets
Status: Agreed 2026-09-30
