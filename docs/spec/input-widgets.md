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

Read again on 2026-10-10 for the input pieces docs/widget-study.md
shortlists after the display pieces: the number input, the one-time code
input, the rating, the switch and the form. None exists as a widget. The
nearest things: the text input's `Numeric` mode filters typing to digits,
`.` and `-` (src/widgets/input/text_input.rs:682) and names the role
`NumberInput`, with no step, no range and no buttons; `max_length` caps a
text input and draws no cells; the slider has a step, a range, the arrow,
Page, Home and End keys and the `Slider` role (src/widgets/input/slider.rs)
and draws a track; the checkbox toggles on Activate and Confirm and reports
`CheckBox` with a toggled state (src/widgets/input/checkbox.rs:268), and no
control reports `Switch`; a form is `builder::label()`
(src/builder/layout.rs:80) over a grid the application lays out by hand,
and no node is labelled by another (`labelled_by` is set nowhere under
src/). What the pieces build on is there: the App delivers Increment,
Decrement and SetValue to the node that advertises them
(src/app/event_tree/accessibility.rs:159-161, CTL-005), a paste reaches a
focused text input as `Event::Paste` (src/widgets/input/text_input.rs:1070),
the keymap names Up, Down, Left, Right, Home, End, PageUp and PageDown
(KEY-001), the icon catalog has `Plus`, `Minus` and `Dot` and takes more
names (DIS-005), and the checkbox and the slider draw pictures where the
terminal takes pixels (PIX-004, PIX-005). gpui-kit 0.7.0 has a number
input, an OTP input, a rating, a switch and a form (docs/widget-study.md
cites them as G/input/number_input.rs, G/input/otp_input.rs, G/rating.rs,
G/switch.rs and G/form).

## Observed

(none yet)

## Draft

[CTL-001] A control whose style the application did not set MUST look the same whether it is built from its props or through any of its builders, and MUST paint: a text input's or a select's field in `input` with `foreground` text, its placeholder, a select's caret and a text input's line numbers in `text-muted`; a control's frame (a field's two end cells, a checkbox's `[` and `]`, a radio's `(` and `)`) in `border`, in `ring` while the control holds the focus; the text input's cursor cell in `foreground` with `input` text, its selection in `selection` with `selection-foreground` text, its error line in `text-error`; the mark of a checked or mixed box and the dot of the chosen radio in `primary`; a slider's track in `border`, its filled part in `primary`, its thumb in `foreground`, in `ring` while the slider holds the focus; a select's open list as a panel in `surface` with `foreground` text and a border in `border`, its current row in `selection` with `selection-foreground` text, the chosen option's mark in `primary`; `builder::primary_button()` in `primary` with `primary-foreground` text and `builder::button()` in `secondary` with `secondary-foreground` text, each with one cell of padding at each side and none above or below, in `selection` with `selection-foreground` text while it holds the focus; a control's label and text in `foreground`, in `text-muted` when it is disabled; and the row of a checkbox, a radio or a select's option under the pointer in `hover`; where a control draws its pixel look (PIX-004, PIX-005), its frame, mark, dot, track, filled part and thumb are the picture's in those same roles, and the cells they held are the look's background. A control MUST show focus, hover and disabled by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, a default text input, checkbox, radio button, select, slider or button built through a builder paints a cell in another color than the same control built from its props; a field, frame, cursor, selection, error line, placeholder, caret, line number, mark, dot, track, filled part, thumb, panel, border, current row, button, label or hovered row is painted in a color other than its role's; a control paints a cell in a color no role of the theme has; or a control with the focus, under the pointer or disabled paints a glyph outside its box, its field, its track or its label, or paints its label with other characters than the label's own.
Mechanism: input-widgets
Rationale: Revised 2026-10-04: where the terminal takes pixels the frame, mark, dot, track and thumb are drawn in the picture (pixel-looks.md); a control's state must be visible under every preset, and color is the cue a terminal has.
Status: Agreed 2026-10-04

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

[CTL-005] A control MUST advertise to the screen reader each action it handles besides Focus and Click, and the App MUST deliver each advertised request to the control whose node advertises it: a text input advertises SetValue and on it MUST replace its text with the requested string, filtered as typing is, and report the change; a slider advertises Increment, Decrement and SetValue and MUST move one step on the first two and take a requested numeric value snapped to its step and clamped to its range, reporting the change; a select advertises SetValue and MUST choose the option whose value or label equals the request; a request a node does not advertise, or one aimed at a disabled control, MUST change nothing.
Falsifier: A text input's node does not advertise SetValue, or a SetValue request of `abc` leaves another text or calls no `on_change`; a slider's node lacks Increment or Decrement, or an Increment leaves its value unchanged or moves it by other than one step, or a SetValue above the maximum leaves a value above it; a select's SetValue naming an option's value does not choose it; or a request on a disabled control changes it.
Mechanism: widget-behavior
Rationale: The App acts on Focus and Click only (src/app.rs:443-446) and every node advertises Click alone (src/accessibility/mod.rs:76), so a screen-reader user hears a slider's value and cannot set it (docs/widget-study.md, change 2).
Status: Agreed 2026-10-09

[CTL-006] The screen reader MUST be told each radio button's position in its group and the group's count, as it is told a tab's; a disabled radio counts.
Falsifier: In a group of three radios of which the third is disabled, the second radio's node lacks position 2 or set size 3.
Mechanism: widget-behavior
Rationale: Tabs, accordion headers, breadcrumb segments and tree rows report their position and count (NAV-004, DAT-004); radios do not (src/widgets/input/radio_button.rs:241-250; docs/widget-study.md, change 3).
Status: Agreed 2026-10-09

[CTL-007] A number input, a code input, a rating, a switch and a form whose style the application did not set MUST look the same whether built from their props or through `builder::number_input()`, `builder::code_input()`, `builder::rating()`, `builder::switch()` and `builder::form()`, and MUST paint: a number input's field as CTL-001 paints a text input's, with its `[-]` and `[+]` cells, their glyphs `Icon::Minus` and `Icon::Plus`, on `secondary` with `secondary-foreground` text, the one that cannot step further, and both while the input is disabled, in `text-muted` on `secondary`; a code input's digit cells in `input` with `foreground` digits and `_` in `text-muted` for an empty cell, `Icon::Dot` in `foreground` for a masked digit, each cell's `[` and `]` in `border`, the current cell's in `ring` while the input holds the focus; a rating's lit stars `Icon::Star` in `text-warning` and its unlit stars `Icon::StarOutline` in `border`, in `ring` while the rating holds the focus, and all of them in `text-muted` while it is disabled; a switch's `[` and `]` in `border`, in `ring` while it holds the focus, its thumb `Icon::Dot` in `primary` when on and in `text-muted` when off, its label in `foreground`, in `text-muted` when it is disabled, and its row under the pointer in `hover`; a form's labels in `foreground`, its required marks `*` in `text-error` and its descriptions in `text-muted`, its controls as their own requirements paint them. The catalog gains `Star` and `StarOutline` (`★` and `☆`, `*` and `-` in ASCII) under DIS-005's rule. A piece MUST show focus, hover and disabled by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, a default number input, code input, rating, switch or form built through its builder paints a cell in another color than the same piece built from its props; a field, button cell, digit cell, placeholder, dot, frame, star, thumb, label, mark or description is painted in a color other than its role's; a piece paints a cell in a color no role of the theme has; or a piece with the focus, under the pointer or disabled paints a glyph outside its cells or paints its label with other characters than the label's own.
Mechanism: input-widgets
Rationale: CTL-001 set this shape for the six controls; the pieces are controls of the same family, and gpui-kit draws its spin buttons, OTP cells, stars and switch in the same tokens as its inputs.
Status: Agreed 2026-10-10

[CTL-008] A number input whose width the application did not set MUST fill the width its parent allots, its field taking what its two button cells leave; `width` on its props or a `w-N` or `w-full` class on its builder MUST set the width instead. A code input MUST be as wide as its digit cells and the one-cell gap between each group (`group` digits to a group, none unless set), a rating as wide as its stars, and a switch as wide as its box, one cell and its label need. A form MUST fill the width its parent allots: with labels beside the controls (the default), its label column is as wide as its widest label, plus one cell for the required mark when a field is required, or `label_width` when set, and each control fills what the column and one cell of gap leave; with labels above, each label stands on the row over its control and the control fills the row; when the allotted width cannot hold the label column and ten cells of control, the labels MUST go above whatever the setting. A description MUST stand on its own row under its control, starting at the control's first column.
Falsifier: In a viewport of 240 by 60 cells, a default number input inside a box of 100 cells paints a field that ends before the cell before its `[+]`, or its `[+]` ends before the box's last cell, or `.class("w-40")` on its builder paints another width than 40 cells; a code input of six digits in groups of three paints other than 19 cells; a rating of five paints other than five cells; a switch with a 10-cell label paints a row wider than 15 cells; a form of three fields labelled `Name`, `Email` and `Description`, the first required, inside a box of 100 cells paints a control starting at another column than the fourteenth or a text input ending before the box's last cell; a form inside a box of 20 cells with a 12-cell label paints that label beside its control; or a description is painted beside its control or starts in the label column.
Mechanism: input-widgets
Rationale: CTL-002 and DIS-002 give every control the width its parent allots; the developer asked on 2026-10-10 that everything be responsive, and a label column that leaves no room for the control is the case a form meets first at 80 columns.
Status: Agreed 2026-10-10

[CTL-009] A number input MUST take typing as a text input in its `Numeric` mode takes it; Up and Down MUST move its value by `step` (1 unless set), Page Up and Page Down by ten steps, Home and End to `min` and `max` when set; a click on its `[-]` or `[+]` cell and the mouse wheel over it MUST step the same way; a step MUST keep the step's decimals and MUST never pass `min` or `max`; when the input loses the focus a typed value MUST be clamped into the range and written in the step's decimals; each change MUST be reported through `on_change` with the value and Confirm through `on_submit`. A code input of `length` digits MUST put a typed digit into its current cell, the first empty one, and Backspace MUST clear the last filled cell; a paste MUST fill the cells with the digits it holds in order and drop its other characters; filling the last cell, and Confirm while every cell is filled, MUST report the code through `on_complete`, and each change through `on_change`; a click MUST give it the focus. A rating of `max` stars (5 unless set) MUST move its value by one on Left, Down, Right and Up, to 0 on Home and `max` on End, set it to a digit typed from `0` to `9` up to `max`, and to a star's number on a click on that star, reporting each change through `on_change`. A switch MUST toggle on Activate, Confirm and a click, reporting the new state through `on_change`. A form's fields MUST take the focus in their order on Next and Previous. A disabled piece takes nothing. The builders MUST take the callbacks their components take, through methods of the same names, as CMP-009 has it for the six controls; and `data_table().on_page_change(f)` MUST take the page callback `DataTableProps.on_page_change` holds, which the display-pieces work left on the props alone.
Falsifier: A number input of step 0.5 between 0 and 10 at `2.0` shows other than `2.5` after Up, `7.5` after Page Up, `10.0` after End, or other than `10.0` after Up at `10.0`; one typed to `42` shows other than `10.0` after Tab; a click on `[+]` does not step or calls no `on_change` with the value; a code input of six digits typed `1`, `2`, `3` shows them in other than its first three cells, Backspace does not clear the third, a paste of `98-7654` fills other than `987654`, or the sixth digit calls no `on_complete` with `987654`; a masked code input paints a typed digit; a rating of five at 3 shows other than 4 after Right, 0 after Home, 2 after a typed `2`, or a click on its fifth star does not set 5 or calls no `on_change` with 5; a switch does not toggle on Space, Enter or a click, or a disabled one does; a form's second field does not take the focus on Tab from its first; a listed builder lacks a callback method its component has, or a piece built with `.on_change(f)` does not call f on the change its props-built twin reports; or `data_table()` has no `on_page_change`.
Mechanism: input-widgets
Rationale: The study's terminal readings of the five pieces (docs/widget-study.md, number input, OTP input, rating, switch, form); the keys are the slider's and the checkbox's under KEY-001, and the digits are the piece's own keys as KEY-001 allows.
Status: Agreed 2026-10-10

[CTL-010] The screen reader MUST be told: a number input as a spin button with its label or `aria_label`, its value, minimum, maximum and step, advertising Increment, Decrement and SetValue and acting on them as CTL-005 has a slider act, and, while the user types, with the text its field shows as its value text, the number that text reads as for its value, no value while the field is empty, and marked invalid when no characters the field takes, typed after its text, could make it a number inside the range; a code input as one text input, a password input when masked, labelled with its label and its length as `Code, 6 digits`, its digits as the value, or bullets when it is masked, advertising SetValue and taking a requested string as a paste; a rating as a slider with its label or `aria_label`, its value, a minimum of 0, a maximum of `max`, a step of 1 and the value text `n of m`, advertising Increment, Decrement and SetValue; a switch as a switch with its label or `aria_label`, toggled on or off; a form as a form labelled by its `aria_label` when set, each control labelled by its label's node and described by its description's node, a required control marked required; and each of them disabled when it is. Every action a piece takes from the pointer MUST have a key.
Falsifier: A number input's node lacks the spin button role, its label, value, minimum, maximum or step, does not advertise Increment, or an Increment request leaves its value unchanged; while the user types into a number input of 0 to 10, after `4` then `2` its node's value text is other than `42` or it is not marked invalid; in a number input of 1900 to 2100, after `2` it is marked invalid, or after `3` it is not; an emptied number input's node carries a value; a code input has a node per cell, or its node lacks its label with the length or its value, or a masked one's node has a role other than password input; a rating's node lacks the slider role, its value or its `n of m` text, or a SetValue of 4 leaves another value; a switch's node has a role other than switch, lacks its label, or does not report toggled on while it is on; a form's node is not a form, a control's node lacks `labelled_by` naming its label's node or `described_by` naming its description's node, or a required control's node is not marked required; a disabled piece's node is not marked disabled; or an action of a piece can be taken by the pointer and by no key.
Mechanism: input-widgets
Rationale: Revised 2026-10-10 after the developer's ruling (escalation 786f3e09): a masked code gives bullets as the password text field does (CTL-004), so a one-time code is not spoken or read off the accessibility bus; revised 2026-10-11 after the developer's ruling on escalation 22c14501 and a real Orca run: a number input is heard as its field shows it, marked invalid only when typing on cannot help (a year box would otherwise be invalid at its first digit), and an emptied one speaks no stale number. CTL-004 and CTL-005 for the six controls; gpui-kit's OTP input and rating carry no role at all (docs/widget-study.md), and its form does not link a label to its control, which the study marked as ours to add.
Status: Agreed 2026-10-11

[CTL-011] On the Linux development host, with Orca, an X server and GNOME Terminal at hand, Orca MUST speak the input pieces of the accessibility probe's input page this way: a number input of 0 to 10 named `Volume`, given the focus, with `Volume`, `spin button` and its value; typing `4` then `2` into it with `42` and `invalid entry`; emptying it with no number; typing `2025` into a number input of 1900 to 2100 named `Year` with `2025` and no `invalid entry`; a rating of five at 3 named `Stars`, given the focus, with `Stars`, `slider` and `3 of 5`, and Right with `4 of 5`; a masked code input of six digits labelled `Code`, given the focus, with `Code, 6 digits` and `password text`, and none of the digits typed into it; and a switch labelled `Wi-Fi`, given the focus, with `Wi-Fi` and that it is not pressed, and Space with `pressed`.
Falsifier: In a run of the probe's input page under Orca on a private X display and D-Bus session, the speech Orca logs after one of those steps lacks a phrase named for it, holds `invalid entry` while `2025` is typed into the year input, holds a number after the number input is emptied, or holds a digit typed into the masked code input.
Mechanism: input-orca
Rationale: CTL-010's checks read the published tree; a by-hand Orca run on 2026-10-11 found an emptied number input speaking its old number, which no tree check saw, so what Orca speaks becomes a check of its own (next-feature item input-pieces-in-the-orca-checks).
Status: Agreed 2026-10-11
