# Menus

Prefix: MNU

The menu family is the menu bar, the context menu, the popup menu and the
dialog menu (src/widgets/menu, src/builder/widgets/menu.rs). Each draws
panels of rows; a row is an action, a checkbox, a radio item, a submenu or a
separator. Read and seen on 2026-09-29: `MenuStyle::default` is dark grey
with a blue current row, while a menu made through the builder is white with
black text, so the two ways to build one menu give two looks
(src/widgets/menu/style.rs, `convert_menu_style`). The looks `Dark`, `Light`
and `HighContrast` of `MenuTheme` are palette classes of their own. A panel
is at most 50 cells wide and shows at most 10 rows on any terminal. A panel
that would leave the viewport is moved back into it, which can put it over
the title or row that opened it (src/widgets/menu/panels.rs). A panel's
stacking number starts at 1000, the number the modal, the popover and the
dialog engine also use. A panel is a child of its menu, so a box that clips
its content clips the panel too: no cell of it is painted when its menu
stands inside a modal, a popover or a header of fixed height. A context menu
opened without a position opens at the screen's first cell; in the widget
catalog it covers the page's header line. Only the mouse opens a context
menu, by a right click or a long press (src/widgets/menu/context_live.rs).
The catalog's demo never opens its dialog menu, so its card is empty
(examples/widget_catalog/catalog.rs). The keyboard reaches every row (arrows
with wrap, Home, End, PageUp, PageDown, Enter, Space, Escape, item
shortcuts), and the screen reader is told each row's kind, checked state and
open state (src/widgets/menu/view.rs). What the widget bar asks of every
reworked widget (theme colors, filling the parent, resize, keyboard, screen
reader, goldens, frame budget, catalog and manual) is in quality-bar.md and
is not repeated here.

## Observed

(none yet)

## Draft

[MNU-001] A menu whose style the application did not set MUST look the same whether it is built from its props or through its builder, and MUST paint its panel in the theme's `surface` with `foreground` text and a border in `border`; its current row in `selection` with `selection-foreground` text while the menu holds the focus and in `hover` with `foreground` text while it does not; a disabled row, a shortcut and a separator in `text-muted`; the veil behind a dialog menu in `overlay`; and a panel's shadow in `shadow`. A menu whose style names the look `Dark`, `Light` or `HighContrast` MUST take the same roles from the built-in preset of that name.
Falsifier: Under a theme whose roles all differ, a default menu bar, context menu, popup menu or dialog menu built through the builder paints a cell in another color than the same menu built from its props; a panel, border, current row, disabled row, shortcut or separator is painted in a color other than its role's; the veil or a shadow is painted in a color other than its role's laid over what is under it; or a menu with the look `Dark`, `Light` or `HighContrast` paints its panel, a row or its current row in a color other than that role's in the built-in preset of that name.
Mechanism: menus
Status: Draft

[MNU-002] A panel MUST paint its widest row whole and every one of its rows, as far as the viewport holds them, and no default MAY limit a panel's width or its number of rows; a panel with more rows than the viewport holds MUST fill the viewport's height and scroll to keep the current row in view.
Falsifier: With the default style in a viewport of 240 by 60 cells, a menu whose widest row is 80 cells paints a panel that cuts that row's label or shortcut, or a menu of 30 rows paints fewer than 30; or, in a viewport of 20 rows, a menu of 30 rows paints fewer rows than fit between the viewport's first and last row, or moving to its last row leaves that row unpainted.
Mechanism: menus
Rationale: The developer's terminals are 240 to 512 columns wide; a limit of 50 cells and 10 rows is an 80-column habit.
Status: Draft

[MNU-003] A panel MUST open inside the viewport and beside what opened it: a menu bar's panel under its title, or over it when only the space above holds the panel; a submenu to the right of its parent row, or to the left when only the space to the left holds it; a popup menu at the side of its anchor that its placement names, or at the opposite side when only that side holds it; and a context menu at the pointer, or at the first cell of the area it serves when Shift+F10 opened it. While one of those places holds the panel, the panel MUST NOT cover the title, row or anchor that opened it.
Falsifier: A menu bar at the lower edge of a viewport of 40 rows opens a panel of five rows that covers its title; a submenu opened from a panel at the right edge of the viewport covers its parent row although the space to the left of the panel holds it; a popup menu placed below an anchor on the last row covers the anchor; a context menu opened by a right click at a cell inside the viewport has its first corner on another cell although the panel fits there; Shift+F10 on a context menu that holds the focus opens no panel, or one whose first corner is not the first cell of the area the context menu serves; or a panel that the viewport holds is opened so near the viewport's edge that a row or a corner of it is not painted.
Mechanism: menus
Status: Draft

[MNU-004] A panel MUST be painted whole, over everything that was on the screen when it opened: a modal, a popover or a box that clips its content, when it holds what opened the panel, MUST neither cover nor clip the panel, and a submenu MUST be painted over its parent panel.
Falsifier: A menu opened from an element inside a visible modal, inside an open popover or inside a box that clips its content has a cell of its panel that is not painted, although the viewport holds the panel; or a cell of a submenu is hidden by its parent panel.
Mechanism: menus
Rationale: On 2026-09-29 a menu bar in a box of three rows that clips its content opened a panel of which no cell was painted, and so did a popup menu inside a modal.
Status: Draft
