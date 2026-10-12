# Key actions

Prefix: KEY

Every widget matches key codes in its own handler: 32 files under
src/widgets match `KeyCode::`, for example the tabs
(src/widgets/layout/tabs.rs:1044-1052) and the select
(src/widgets/input/select.rs:751-774), and a menu item's shortcut is a display
string beside a list of key names (`MenuShortcut`, src/widgets/menu/item.rs:3-14)
that `shortcut_matches` compares with the pressed key (src/widgets/menu/model.rs).
Read on 2026-10-09: an application cannot rebind a key, two widgets can
disagree about one action, and a hint can drift from the key it names. The
App keeps one key of its own, the quit key (src/app.rs:319-327). gpui-kit
defines shared actions (Confirm, Cancel, SelectUp, SelectDown, SelectLeft,
SelectRight, SelectFirst, SelectLast, SelectPageUp, SelectPageDown;
B/actions.rs:6-28, as docs/widget-study.md cites it) that the application's keymap binds to keys, and
its menus read an item's hint from the real binding (docs/widget-study.md,
change 7). Typing is not an action: a widget that takes text keeps taking a
plain character as text.

## Observed

(none yet)

## Draft

[KEY-001] The crate MUST define named key actions and an active keymap that binds keys to them, and every widget MUST read the listed actions through the active keymap in place of key codes; a key that belongs to one widget alone, such as a file explorer's toolbar letter or a select's type-ahead, stays that widget's own. The actions and their default bindings: Confirm (Enter), Activate (Space), Cancel (Escape), Up, Down, Left, Right, Home, End, PageUp and PageDown (the keys of those names), Next (Tab), Previous (Shift+Tab), Delete (Delete), Sort (`s`), Expand (`+`), Collapse (`-`), ContextMenu (Shift+F10), Copy (Ctrl+C), Cut (Ctrl+X), Paste (Ctrl+V), Undo (Ctrl+Z), Redo (Ctrl+Y) and Search (Ctrl+F). `Keymap::bind(action, key)` MUST add a binding, `unbind(action, key)` remove one and `rebind(action, keys)` replace them all, taking effect through `Keymap::set_active` or `App::set_keymap`; after `rebind(Confirm, [F2])` a checkbox, a radio button, a select, tabs, an accordion, a breadcrumb, a table row, a tree row, a menu item, a confirmation dialog and a text input's submit MUST act on F2 and no longer on Enter, and after `rebind(Up, [k])` and `rebind(Down, [j])` a select's open list, a slider, a tree, a table and a menu MUST move on those keys. A widget that takes typed text MUST keep taking a plain character as text, so a binding to a plain character reaches only widgets that take no text.
Falsifier: After `Keymap::rebind(Confirm, [F2])` one of the listed widgets still acts on Enter or does not act on F2; after `rebind(Down, [Char('j')])` a select's open list, a tree or a menu does not move down on `j`, or a text input stops inserting `j`; a listed action has another default binding; or `unbind(Activate, Space)` leaves a checkbox toggling on Space.
Mechanism: widget-behavior
Status: Agreed 2026-10-09

[KEY-002] A key binding MUST have one text form, `KeyBinding::display`, that writes the modifiers as `Ctrl+`, `Alt+`, `Shift+` and `Meta+` in that order before the key's name (`Enter`, `Escape`, `Space`, `Tab`, `F2`, `Up`, `Page Down`, a letter in upper case); a menu item that names an action (`MenuItem::bound_to(Action)`) MUST take both its hint and the key that triggers it from the active keymap's binding of that action, so that a rebind changes both; an item given an explicit shortcut keeps it; and the screen reader MUST be told the same text as the item's keyboard shortcut.
Falsifier: `KeyBinding::display` of Ctrl+Shift+F10 is other than `Ctrl+Shift+F10`, or of Page Down other than `Page Down`; a menu item naming Copy shows a hint other than `Ctrl+C` under the default keymap; after `rebind(Copy, [Ctrl+Shift+C])` the item still shows `Ctrl+C`, still triggers on Ctrl+C or does not trigger on Ctrl+Shift+C; or the item's node carries a keyboard shortcut other than the hint shown.
Mechanism: widget-behavior
Status: Agreed 2026-10-09
