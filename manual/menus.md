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

## Limits

- A shortcut value describes and matches input; the host application still
  controls which events reach the menu.
- Menu rectangles use terminal-cell coordinates.
- Oversized menus are limited to the available viewport.
- An action callback runs during event processing and should return quickly.

## Source map

- Menu exports: [`src/widgets/menu/mod.rs`](../src/widgets/menu/mod.rs)
- Menu state rules: [`src/widgets/menu/state.rs`](../src/widgets/menu/state.rs)
- Menu runtime: [`src/widgets/menu/runtime.rs`](../src/widgets/menu/runtime.rs)
- Menu behavior tests: [`tests/api_widget_behavior/menus.rs`](../tests/api_widget_behavior/menus.rs)
- Menu API probe: [`tests/api_widget_behavior/menu_probe.rs`](../tests/api_widget_behavior/menu_probe.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Dialogs](dialogs.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
