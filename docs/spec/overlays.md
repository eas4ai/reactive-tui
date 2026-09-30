# Overlays

Prefix: OVL

The overlay family is the modal, the popover, the toast and the five dialogs
of the dialog engine: confirmation, input, autocomplete, progress and wizard
(src/widgets/display/modal.rs, src/widgets/display/popover.rs,
src/widgets/dialog, src/builder/widgets/dialog.rs,
src/builder/dialog_builders.rs). Each opens a box over the screen's content
and closes on a key, a button or, for a toast, after its time. Read on
2026-09-29: the modal's default look is `bg-white text-black` with a
`bg-black/50` veil (src/widgets/display/modal.rs:184-189); `DialogTheme`,
the look of the five dialogs, is `bg-white` with `bg-blue-500 text-white`
buttons (src/widgets/dialog/mod.rs:259-275); a toast is `bg-green-700`,
`bg-red-700`, `bg-yellow-700` or `bg-blue-700` with white text, and its
`Custom` kind paints like `info` whatever its string
(src/widgets/dialog/toast/live.rs:147-152); the popover's body has no color
of its own and its outside-click veil is `bg-black/30`
(src/widgets/display/popover/live/render.rs:150). The looks
`DialogThemes::light`, `dark`, `minimal` and `high_contrast` are palette
classes with pixel-sized padding: a button is `px-16 py-8`, 16 cells by 8
rows (src/widgets/dialog/dialog_types.rs:196-293). A popover opens 8 rows
from its trigger with an arrow 8 rows deep (src/widgets/display/popover.rs:112-121,241).
A modal with no width set grows to its content up to the viewport, so a
long message is a box as wide as the terminal. The modal centers itself in
the clip rectangle of the element that owns it, not on the screen, so a
modal built inside a card is centered in the card and cut by it
(src/widgets/display/modal/live/render.rs:49-62). A toast in a corner
touches the screen's edge, and two toasts at one position are painted at
the same place: the engine gives each toast empty bounds
(src/widgets/dialog/engine/content.rs:128-131), which keep the position
(src/widgets/dialog/frame.rs:26-33). The modal, the popover and the engine's
first dialog all stack at 1000, a toast at 2000, a menu panel at 3000.
`DialogBounds` and `DialogComponent::get_bounds` are set by six dialogs and
read by nothing; `DialogUtils::calculate_size` and `DialogBuffer` have no
caller. The keyboard already opens a popover from its trigger (Enter or
Space) and closes it with Escape (src/widgets/display/popover/live/events.rs);
a confirmation dialog reports the role `Dialog`
(src/widgets/dialog/confirmation/live.rs:194). What the widget bar
asks of every reworked widget is in quality-bar.md and is not repeated
here.

## Observed

(none yet)

## Draft

[OVL-001] A modal, a popover, a toast or a dialog whose style the application did not set MUST look the same whether it is built from its props, through its builder or through the dialog engine, and MUST paint its box in the theme's `surface` with `foreground` text and a border in `border`; the veil behind one that is modal in `overlay` laid over what is under it; its primary button in `primary` with `primary-foreground` text, a danger button in `error` with `error-foreground` text and every other button in `secondary` with `secondary-foreground` text; the button that holds the focus in `selection` with `selection-foreground` text; a toast in `success`, `warning`, `error` or `info` by its kind with that fill's text role, and a toast of the `Custom` kind in the classes its string names; an error message in `error` and a warning in `warning`; and a popover's arrow in `surface`. A dialog theme that names the look `light`, `dark` or `high_contrast` MUST take the same roles from the built-in preset of that name.
Falsifier: Under a theme whose roles all differ, a default modal, popover, toast or dialog built through its builder or shown through the dialog engine paints a cell in another color than the same one built from its props; a box, border, button, focused button, toast, message or arrow is painted in a color other than its role's; a `Custom` toast is painted in a color its string does not name; the veil is painted in a color other than `overlay` laid over what is under it; or a dialog theme with the look `light`, `dark` or `high_contrast` paints its box, a button or its focused button in a color other than that role's in the built-in preset of that name.
Mechanism: overlays
Status: Draft

[OVL-002] A modal, a toast or a dialog whose width the application did not set MUST be as wide as its content, its title or its row of buttons needs plus one cell of padding at each side, and at most half the viewport's width, with a message longer than that wrapped onto more lines; every look of `DialogThemes` MUST give a title and a button one cell of padding at each side and none above or below; and a popover MUST open with its body one row from its trigger, or one cell when it opens beside it, with the arrow, one row deep, in that row when it is on.
Falsifier: In a viewport of 240 by 60 cells, a confirmation dialog whose title is shorter than its 30-cell message, with two short buttons, paints a box wider than 34 cells, or one with a 200-cell message paints a box wider than 120 cells or leaves a word of the message unpainted; a title or a button of a `DialogThemes` look is painted with more or less than one cell of padding at a side or with a row of padding above or below; or a popover with the default props opens with more than one row between its body and its trigger, or with an arrow deeper than one row.
Mechanism: overlays
Rationale: The developer's terminals are 240 to 512 columns wide; 16 cells of padding and a box as wide as the terminal are pixel numbers read as cells.
Status: Draft

[OVL-003] A modal or a dialog MUST be placed on the screen: centered in it unless its position names another place, or one row lower than the dialog it opened over; a toast at the corner or edge its position names with one cell between it and the screen's edge, and a later toast at the same position next to the earlier ones, away from the edge, so that no two overlap; a popover at the side of its trigger its placement names, or at the opposite side when only that side holds it, never covering its trigger while a side holds it. Each of them MUST be painted whole over what was on the screen when it opened, also when the element that owns it stands inside a modal, a popover or a box that clips its content, in this order from below: a modal or dialog, a popover, a toast, a menu panel.
Falsifier: In a viewport of 240 by 60 cells, a modal built inside a box of three rows that clips its content has its box centered in that box instead of in the viewport, or a cell of it unpainted; a second confirmation dialog opened over a first has its first row on the first's first row; a toast at the top right has a cell on the viewport's first row or last column; two toasts shown at the top right have a cell in common; a popover placed below a trigger on the viewport's last row covers the trigger although the space above holds it; a modal, a dialog, a toast or a popover has a cell painted over by the element that owns it, by a modal or by a box that clips; or a popover opened from inside a modal is painted under it, a toast under a popover, or a menu panel under a toast.
Mechanism: overlays
Rationale: gpui-kit's Root mounts the dialog and notification layers over the whole window and centers a dialog in it; the box keeps its own size.
Status: Draft

[OVL-004] The screen reader MUST be told a modal's or a dialog's title as its label, the role `AlertDialog` for a confirmation dialog and `Dialog` for the others, a toast's kind and message with the role `Status` for info and success and `Alert` for warning and error, and a popover's open state on its trigger; and a popover opened from its trigger by Enter or Space MUST move the focus into its content when the content holds a focusable element and return it to the trigger when the popover closes.
Falsifier: An accessibility node of a modal or dialog lacks its title as its label, a confirmation dialog's node has a role other than `AlertDialog` or another dialog's a role other than `Dialog`; a toast's node has a role other than `Status` for info and success or `Alert` for warning and error, or lacks its message; a popover's trigger does not report whether the popover is open; or after Enter on a popover's trigger whose content holds a button, the focus is not inside the popover, or after Escape it is not on the trigger.
Mechanism: overlays
Status: Draft
