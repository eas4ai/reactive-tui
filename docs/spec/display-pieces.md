# Display pieces

Prefix: DIS

The display pieces are the small shared widgets docs/widget-study.md
shortlists under "Widgets to build": an icon catalog, a spinner, a
separator, a badge and a tag, a key hint, an empty state, a skeleton and a
shimmer, a status bar, a description list, an inline alert, a link, a
pagination bar and a stepper. Each shows one thing and takes little or no
input, and the larger widgets use them. Read on 2026-10-10: none exists as
a widget. The pieces that stand in for them are scattered and disagree:
the tabs carry their own badge (`TabBadge` and `TabBadgeVariant`,
src/widgets/layout/tabs.rs:72-104) painted as a glyph beside the label;
the data table draws its own page bar from `PaginationConfig`
(src/widgets/display/data_table.rs:47) with Prev and Next and `n/total`;
the wizard dialog shows "Step N of M" and a progress bar; the menus draw
separator rows in six styles (`MenuSeparator`, src/widgets/menu/item.rs:63)
and a Markdown rule is a fixed line of 28 cells (src/markdown/ast_walker.rs);
the toast has four kinds and their colors (src/widgets/dialog/toast.rs) but
no inline form; the file explorer maps file kinds to emoji
(src/widgets/display/file_explorer.rs), the confirmation dialog maps its
kinds to glyphs (src/widgets/dialog/confirmation/live.rs) and the tab badge
to marks, so an error is `×` in one place and `✗` in another; a footer is a
plain block (`footer()`, src/builder/core.rs) every application fills by
hand; a Markdown link is underlined but its URL dropped
(src/markdown/ast_walker.rs, src/markdown/converter.rs), while the
platform knows whether the terminal takes hyperlinks
(src/platform/mod.rs:157-158) and can write OSC 8
(src/platform/mod.rs:497-498, src/backend/direct_tty.rs:80-85) and the
default backend's cells can carry a link
(crates/reactive-tui-suprtui/src/ansi.rs), but no element reaches those
paths; a `pulse` animation and `animate-pulse` exist (src/layout/css/animations.rs)
with no widget over them; and the App knows when motion is reduced
(src/app/motion.rs) while nothing cycles glyphs. gpui-kit has each piece
as a component (docs/widget-study.md cites them as G/alert.rs, G/badge.rs,
G/tag.rs, G/kbd.rs, G/empty.rs, G/skeleton.rs, G/shimmer.rs,
G/status_bar.rs, G/description_list.rs, G/link.rs, G/pagination.rs,
G/stepper, G/spinner.rs, G/separator.rs and G/icon.rs).

The pieces are built on the keymap (KEY-001, KEY-002), the theme's color
roles (THM-001) and the widget bar (BAR-003, BAR-006), as the layout and
data families were. The input pieces and the medium widgets of the same
shortlist follow in their own commitments.

## Observed

(none yet)

## Draft

[DIS-001] A display piece whose style the application did not set MUST look the same whether it is built from its props or through its builder, and MUST paint: a separator's line and a description list's border in `border` and a separator's label in `text-muted`; a badge or a tag of the `Default` kind on `secondary` with `secondary-foreground` text and of the `Info`, `Success`, `Warning` or `Error` kind on `info`, `success`, `warning` or `error` with that fill's `-foreground` text, an outline tag as its text between brackets in `text-info`, `text-success`, `text-warning` or `text-error` by its kind; a key hint on `secondary` with `secondary-foreground` text; an alert's bar and icon in `text-info`, `text-success`, `text-warning` or `text-error` by its kind and in `text-muted` for `Default`, its title in `foreground` and its message in `text-muted`, on `surface`; a status bar on `surface` with `foreground` text; a description list's labels in `text-muted` and values in `foreground`; an empty state's title in `foreground` and description in `text-muted`; a skeleton's rows in `border`; a shimmer's band in `foreground` over text in `text-muted`; a spinner in `text-muted`; a link underlined in `text-accent`, with `ring` while it has the focus; a stepper's passed steps in `text-success`, its current step in `foreground` and the others in `text-muted`; and a pagination bar's current page on `primary` with `primary-foreground` text and its other pages in `foreground`, with `ring` on the focused one. Focus, hover and a disabled state MUST be shown by color alone, with no added glyph.
Falsifier: Under a theme whose roles all differ, a default piece built through its builder paints a cell in another color than the same piece built from its props; a line, label, border, badge, tag, bracket, key hint, bar, icon, title, message, row, band, spinner, link, step or page is painted in a color other than its role's; or a focused, hovered or disabled piece carries a glyph it does not carry otherwise.
Mechanism: display-pieces
Rationale: BAR-003 asks every color from the theme; the layout and data families set this shape (NAV-001, DAT-001), and the study's `▶` and tildes were stand-ins for color.
Status: Agreed 2026-10-10

[DIS-002] A separator, a status bar, a description list, an alert, an empty state, a pagination bar, a horizontal stepper and a skeleton MUST fill the width their parent allots when the application set no size, and a vertical separator and a vertical stepper the height; a `w-N`, `h-N`, `w-full` or `h-full` class on the builder MUST set the size instead. A badge, a tag, a key hint, a spinner, an icon, a link and a shimmer MUST take the width of their content and no more. A status bar whose regions do not fit MUST cut text with `…`, the center region first, then the right, then the left, and MUST keep its left region's first cell and its right region's last cell on the row.
Falsifier: In a viewport of 240 by 60 cells, a default separator, status bar, description list, alert, empty state, pagination bar, horizontal stepper or skeleton inside a box of 100 cells paints a row that ends before the box's last cell, or a vertical separator or stepper inside a box of 20 rows ends before its last row; `.class("w-40")` on a builder paints another width than 40 cells; a badge of ` 3 `, a tag of ` done `, a key hint of `Ctrl+S`, a spinner or an icon paints more cells than its content; or a status bar of 100 cells with 60 cells of text in each region paints no `…`, cuts its left or right region before its center, or loses its first or last cell.
Mechanism: display-pieces
Status: Agreed 2026-10-10

[DIS-003] A spinner MUST cycle its frames at about ten per second, the braille frames `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` where the terminal takes Unicode and `|/-\` where it reports none, and MUST stand still on one frame while motion is reduced; a shimmer MUST move a band of brighter cells across its text once about every two seconds and MUST paint its text plain while motion is reduced; a skeleton MUST pulse between its two colors over about two seconds and MUST stand still while motion is reduced; none of the three MAY request a frame while motion is reduced. A badge MUST hide at a count of zero and MUST show `99+` above its `max` (99 unless set), and a dot badge MUST show `●`. A pagination bar MUST show the first page, the last page and the pages around the current one up to `visible_pages` (five unless set), an ellipsis for each hidden run, the current page marked, and `‹` and `›` for the previous and next page; Left and Right MUST move the current page by one, Home and End to the first and the last, Confirm or a click on a page MUST choose it and report it through `on_change`, and Confirm or a click on an ellipsis MUST open a popup menu of the hidden pages. A stepper MUST mark each step before the current one as passed with `✓`, the current with `●` and the rest with `○`, joined by `──` in a row or `│` in a column; when the application set `on_change`, Left and Right (Up and Down in a column) MUST move the focused step and Confirm or a click MUST choose it. A link MUST run its `on_open` callback on Confirm, Activate or a click, and when the terminal reports hyperlinks the App MUST write its text between `ESC ] 8 ; ; <url> ESC \` and `ESC ] 8 ; ; ESC \` so the terminal's own click opens it; a disabled link MUST take no action. An alert's `[×]` MUST close it on Confirm, Activate or a click and report through `on_close`; an empty state's actions are buttons and MUST act on Confirm, Activate or a click. A key hint built from an action MUST show the active keymap's first binding for it in `KeyBinding::display` form and MUST follow a rebind at the next render.
Falsifier: A spinner paints the same frame across 300 ms of frames while motion is not reduced, paints braille frames where the terminal reports no Unicode, or paints two frames or requests a frame while motion is reduced; a shimmer's band stands still, or a skeleton's color does not change, over two seconds while motion is not reduced, or either changes while it is; a badge of count 0 paints anything, a count of 120 paints other than `99+`, or a dot badge paints other than `●`; a pagination bar of 20 pages at page 5 with five visible paints other than `‹ 1 … 4 5 6 … 20 ›` with 5 marked current, Right does not move to 6, End does not move to 20, Confirm on 6 does not report 6, or Confirm on an ellipsis opens no popup; a stepper of three steps at the second paints other than `✓`, `●`, `○` in that order, or with `on_change` set Right does not move the focus and Confirm does not report the step; a link does not run `on_open` on Confirm, Activate or a click, a disabled one does, or on the debug backend with hyperlinks reported the recorded output around the link's text lacks `\x1b]8;;<url>\x1b\\` before and `\x1b]8;;\x1b\\` after; an alert's `[×]` does not close it on Confirm; an empty state's action does not run on Confirm; or a key hint for Copy shows other than `Ctrl+C` under the default keymap or still shows it after `rebind(Copy, [F3])`.
Mechanism: display-pieces
Rationale: gpui-kit's spinner and skeleton stand still under reduced motion (G/spinner.rs, G/skeleton.rs in the study) and its pagination's page math is the base's (B/pagination.rs); a terminal cannot overlap a corner, so the badge follows its content.
Status: Agreed 2026-10-10

[DIS-004] The screen reader MUST be told: a spinner as a status with its label, announced when it appears and not on each frame; a separator as a splitter; a badge's or a tag's text as part of the description of the element it follows, and a standalone tag as a label with its text; a key hint as text in its binding's display form; an alert as an alert for the `Error` and `Warning` kinds and as a status otherwise, with its title and message; a status bar as a status with the text of its regions; a description list with its terms and definitions as a description list, term and definition; an empty state as a group labelled by its title and described by its description, with its actions as buttons; a skeleton and a shimmer as nothing of their own, with their parent carrying `busy` while they show; a link as a link with its text and its URL as the description, disabled when it is; a pagination bar as a navigation landmark labelled from its `aria_label` with each page a button and the current one marked current, its position and the page count; and a stepper's steps as list items with their labels, their position and count, the current one marked current. Every action a display piece takes from the pointer MUST have a key.
Falsifier: A spinner's node lacks the status role or its label, or its App announces it on more than one frame; a separator's node is not a splitter; an element followed by a badge of ` 3 ` has a description without `3`; an alert of the `Error` kind has a node that is not an alert, or one of the `Info` kind that is not a status, or either lacks its title or message; a status bar's node lacks its regions' text; a description list's node lacks the description list role or a pair lacks the term and definition roles; an empty state's node is not a group labelled by its title and described by its description, or an action is not a button; a skeleton or a shimmer has a node of its own, or its parent's node lacks `busy` while it shows; a link's node lacks the link role, its text or its URL, or a disabled link is not marked disabled; a pagination bar's node is not a navigation landmark with the label its `aria_label` set, a page is not a button, or the current page is not marked current with its position and count; a step's node lacks its label, position or count, or the current step is not marked current; or an action of a display piece can be taken by the pointer and by no key.
Mechanism: display-pieces
Status: Agreed 2026-10-10

[DIS-005] The crate MUST define one icon catalog, `Icon`, naming at least `Info`, `Warning`, `Error`, `Success`, `Close`, `Check`, `Dot`, `ChevronUp`, `ChevronDown`, `ChevronLeft`, `ChevronRight`, `Folder`, `FolderOpen`, `File`, `Search`, `Settings`, `Plus`, `Minus`, `Ellipsis` and `Spinner`, each with a single-width Unicode glyph and an ASCII fallback, drawn through one function that gives the glyph where the terminal takes Unicode and the fallback where it reports none; an application MAY give the catalog a Nerd Font set that replaces the glyphs. Every glyph the following paint for a kind or a state MUST come from the catalog: the tab badge's marks, the confirmation dialog's kind glyphs, the file explorer's file and folder marks, the alert's and toast's kind icons, the stepper's marks, the pagination bar's arrows and ellipsis, the spinner's frames and the empty state's default icon; no widget MAY paint an emoji or a glyph wider than one cell for them.
Falsifier: `Icon::Error` and `Icon::Warning` paint glyphs of other than one cell, or the same glyph as each other; a widget paints `✗` for an error where another paints `×`; a terminal that reports no Unicode gets a non-ASCII glyph from any of the named widgets; the file explorer paints an emoji for a file or a folder; or one of the named widgets paints a kind or state glyph by a string literal outside the catalog.
Mechanism: display-pieces
Rationale: The study found the same idea drawn with different glyphs in four widgets (docs/widget-study.md, icon), and emoji take two cells.
Status: Agreed 2026-10-10

[DIS-006] The widgets the crate already has MUST draw the shared pieces in place of their own: the tabs' badge MUST be the badge widget, keeping NAV-001's colors by kind; the data table's page bar MUST be the pagination bar, reporting the page through the table's existing configuration; the wizard dialog's step line MUST be the stepper, with the current step from the wizard's state; the menus' separator rows and the Markdown renderer's rule MUST draw through the separator's glyph table, the rule filling the wrap width the renderer was given in place of a fixed 28 cells, keeping MNU-001's color by role; and the toast MUST take its kinds, colors and icons from the same table as the alert. No duplicate of a piece's glyph table, page math or kind-to-color map MAY remain in those widgets.
Falsifier: A tab with a badge of count 3 paints its badge in cells that differ from a badge widget of count 3 of the same kind; the data table's page bar paints other than the pagination bar's row for the same page and count, or choosing a page through it does not change the table's page; a wizard at step 2 of 3 paints other than the stepper's row for that state; a menu separator or a Markdown rule paints a glyph not in the separator's table, or a Markdown rule rendered at a wrap width of 100 cells paints other than 100 cells; a toast and an alert of the same kind paint their icon or their bar in different colors; or `TabBadge`'s marks, `PaginationConfig`'s page arithmetic or the toast's kind-to-color map survive as a second copy.
Mechanism: display-pieces
Rationale: The study's reason for the badge, pagination, stepper, separator and alert is to replace those copies (docs/widget-study.md, Call lines).
Status: Agreed 2026-10-10
