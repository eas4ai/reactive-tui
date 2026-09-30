# Menus

Crate modules: `widgets`

Widget module: `menu`

## Purpose

Menus expose commands in a menu bar, context menu, popup, or dialog-style
menu.

## Main API

- `MenuItem`, `MenuItemBuilder`, `MenuItemType`, `MenuSeparator`, and
  `MenuShortcut` define entries.
- `MenuAction` carries the action invoked by an item.
- `MenuBar` displays top-level menus and dropdown panels.
- `ContextMenu` opens at an interaction point.
- `PopupMenu` opens relative to an anchor and `PopupPlacement`.
- `DialogMenu` provides confirmation, input, selection, message, progress, and
  custom menu flows.
- `MenuStyle` and `MenuTheme` control menu appearance.

## Basic use

Create items with stable identifiers and actions. Put them in the props for the
required menu type or use a menu builder. Route the returned element through an
application so keyboard focus, pointer hit testing, and callbacks are active.

## Behavior

Menus track the selected item and the current open panel. Keyboard commands
move between enabled items, enter and leave submenus, activate actions, and
close the menu. Pointer input uses presented menu rectangles. Disabled items
and separators are skipped during navigation.

Popup and context menus restore previous focus when they close. Menu selection
is bounded when items change or the visible window becomes empty.

### Colors

Every color of a menu is a role of the active theme (see the color roles in
[Layout, style, and themes](layout-style-and-themes.md)). The default
`MenuStyle` paints a panel in `surface` with `foreground` text and a
`border` border; the current row in `selection` with `selection-foreground`
text while the menu holds the focus, and in `hover` with `foreground` text
while it does not; a disabled row, a shortcut and a separator in
`text-muted`; the shadow under a panel in `shadow`; and the veil behind a
dialog menu in `overlay`. The current row and a disabled row are one color
from end to end: their icon and shortcut take the row's text color.

A menu made through the builder takes the same defaults as one made from
its props. A color the builder's `MenuStyle` names in its `background`,
`text_color`, `selected_background`, `selected_text_color` or
`disabled_text_color` field may be a role, a palette name or a hex value:
`"surface"` and `"blue-500"` both work.

`MenuStyle::new()` starts from the defaults; `.base_classes()`,
`.selected_classes()`, `.focused_classes()`, `.disabled_classes()`,
`.separator_classes()`, `.shortcut_classes()`, `.icon_classes()`,
`.border_classes()`, `.shadow_classes()` and `.veil_classes()` replace one
part's classes. `MenuTheme::Default` follows the active theme; `Dark`,
`Light` and `HighContrast` keep the colors of the built-in preset of that
name under any theme, and `MenuStyle::of(&theme)` does the same for any
theme.

### Panels

A panel paints its widest row whole and every one of its rows, as far as
the viewport holds them, and scrolls beyond that to keep the current row in
view. No default limits its width or its rows; `.min_width()`,
`.max_width()` on the style, and `max_visible_items` or
`max_dropdown_height` in the props, set limits when an application wants
them.

A panel opens beside what opened it: a menu bar's panel under its title, a
submenu right of its parent row, a popup menu at the side its placement
names, a context menu at the pointer. When that side does not hold the
panel and the other side does, the panel opens at the other side: over the
title, left of the row. When neither holds it, it stands as near as the
viewport allows.

A panel is painted whole, over everything that was on the screen when it
opened. No box that holds the menu clips it: a menu bar in a header of
fixed height, or a menu inside a modal or a popover, opens its panels past
the box's edge.

### Keyboard

Up and Down move between rows and wrap; PageUp and PageDown move by the
rows the panel shows; Home and End go to the first and last row; Right
opens a submenu and Left closes it; Enter or Space activates; Escape closes
the open panel; a row's shortcut activates it. Shift+F10 opens a context
menu that holds the focus, at its first trigger area or at the first cell
of its own box, as a right click opens it at the pointer.

## Menu bar

`menubar()` builds a bar of titles; `.title()` puts a label before them,
`.item()` and `.items()` add the titles, each of which may hold a submenu.
The bar fills the width its parent allots. `.on_item_selected()`,
`.on_dropdown_opened()` and `.on_dropdown_closed()` report what the user
does; `.max_dropdown_height()` limits the rows a panel shows. From props,
`MenuBar` takes `MenuBarProps`.

## Context menu

`context_menu()` builds a menu that opens on a right click in the area it
serves, or on Shift+F10 while it holds the focus; `.trigger_on_right_click()`
turns the right click off. It is closed until the user opens it.
`.on_opened()` and `.on_closed()` report the two, `.on_item_selected()`
the choice. From props, `ContextMenu` takes `ContextMenuProps`, whose
`trigger_areas` limit where a right click opens it.

## Popup menu

`popup_menu()` builds a menu that is open when it appears, beside its own
element at the side `.placement()` names. From props, `PopupMenu` takes
`PopupMenuProps`, whose `PopupPlacement` puts the panel at the pointer, at
a cell, or beside a rectangle. `.close_on_outside_click()` and
`.auto_close()` say when it closes.

## Dialog menu

`DialogMenuBuilder::confirmation()`, `::selection()`, `::multi_selection()`,
`::input()` and `::custom()` start a dialog menu; `.title()`, `.message()`
and `.items()` fill it; `.default_button()` and `.cancel_button()` name
the rows Enter and Escape choose; `.modal()` puts the veil over the page.
Its buttons take the theme's `primary` fill. The dialog menu centers in
the viewport unless `.position()` places it.

## Limits

- A shortcut value describes and matches input; the host application still
  controls which events reach the menu.
- Menu rectangles use terminal-cell coordinates.
- A panel larger than the viewport is cut by the viewport.
- An action callback runs during event processing and should return quickly.

## Source map

- Menu exports: [`src/widgets/menu/mod.rs`](../src/widgets/menu/mod.rs)
- Menu state rules: [`src/widgets/menu/state.rs`](../src/widgets/menu/state.rs)
- Menu runtime: [`src/widgets/menu/runtime.rs`](../src/widgets/menu/runtime.rs)
- Menu behavior tests: [`tests/api_widget_behavior/menus.rs`](../tests/api_widget_behavior/menus.rs)
- Colors, panels and the widget bar: [`tests/menus_contract.rs`](../tests/menus_contract.rs)
- Goldens: [`tests/menus_goldens.rs`](../tests/menus_goldens.rs)
- Menu API probe: [`tests/api_widget_behavior/menu_probe.rs`](../tests/api_widget_behavior/menu_probe.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Dialogs](dialogs.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
