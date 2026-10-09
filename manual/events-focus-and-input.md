# Events, focus, and input

Crate modules: `event`

## Purpose

The event system converts terminal input into framework events and delivers
them to the correct component or handler.

## Main API

- `Event` variants carry key, mouse, wheel, resize, focus, paste, and custom
  events.
- `KeyCode`, `KeyModifiers`, `KeyEventKind`, `MouseButton`, `MouseEventKind`,
  `Position`, and `WheelDelta` describe input details.
- `EventRouter` owns an event-node tree, handlers, hit targets, and focus data.
- `EventPhase` identifies capture, target, and bubble delivery.
- `EventResult` reports ignored, handled, consumed, and redraw outcomes.
- `FocusManager` and application focus methods support tab order, directional
  movement, initial focus, and focus traps.

## Basic use

Components can override `handle_event`. Lower-level code can register handlers
on router nodes for selected event phases. Add hit bounds for mouse targets and
register focusable nodes for keyboard navigation.

## Behavior

Keyboard events normally target the focused node. Mouse events use hit testing
and z-order to choose a target. Routing walks capture handlers from the root,
runs target handlers, then bubbles toward the root unless a result stops it.

On the default backend the terminal reports every press, release, drag and
pointer move. On Unix a paste arrives whole as one `Paste` event; on Windows
the console delivers a paste as key presses, because crossterm reads console
input records, which have no paste event. After a press, that button's drags
and its release go to the pressed element, also when the pointer has left it;
a component receives them at its nearest cell. A release over the pressed
element is followed by a `Click`. A second click on the same element and cell
within 500 ms is a `DoubleClick`, and the third and later ones are
`TripleClick`. Controls activate on the press, not on the click. The wheel
goes to the element under the pointer and bubbles: an element that cannot
scroll further in the wheel's direction (a scroll view, table, tree, text
input, terminal, file explorer or breadcrumb bar) leaves the event for its
parent. In the scroll view, table and tree, Shift turns a vertical wheel into
a sideways one. When motion reports queue up faster than the application
handles them, only the latest of each run is delivered, and reports still
queued when the application exits are dropped.

Before the first frame the default backend on Unix asks the terminal whether
it speaks the Kitty keyboard protocol, what its background color is, and
whether it accepts Kitty graphics sent directly and through shared memory,
and reads Sixel support from the device attributes it answers last. It waits
at most 200 ms for the answers, and keys typed meanwhile reach the
application in order. A graphics protocol the terminal reports is added to
what the environment variables already said; a terminal that answers
nothing keeps that detection. When the terminal speaks the protocol, the backend turns
on its flags 1, 8 and 16 (unambiguous escape codes, every key as an escape
code, and the text a key types), and turns them off on every exit and before a
suspend. A key then keeps its code and modifiers: Ctrl+I stays apart from Tab,
and a text key carries the text the terminal reports. Text of several
characters, such as the text an input method commits, arrives as one `Char`
press per character. Caps Lock, Num Lock,
Scroll Lock and the media keys arrive with their own `KeyCode`. A modifier key
pressed alone sends no event. A terminal focus report reaches the root
component only, and the focused element keeps its focus. On Unix, a Ctrl+Z
that no handler consumes suspends the application as a shell expects. The
backend leaves the terminal as it found it and stops the process. When the
process continues, the backend enters the terminal again and repaints the
whole screen.

The application rebuilds its event tree from the presented frame. Focus state
is preserved when possible and moved when a focused node disappears. Positions
preserve whether coordinates are terminal cells or pixels.

## Key actions and rebinding

Every widget reads its keys through one keymap instead of matching key codes
of its own (KEY-001). The keymap binds keys to named actions; the defaults
are the keys the widgets answered to before:

| Action | Default | Action | Default |
|---|---|---|---|
| `Confirm` | Enter | `Next` | Tab |
| `Activate` | Space | `Previous` | Shift+Tab |
| `Cancel` | Escape | `Delete` | Delete |
| `Up`, `Down`, `Left`, `Right` | the arrows | `Sort` | `s` |
| `Home`, `End` | Home, End | `Expand`, `Collapse` | `+`, `-` |
| `PageUp`, `PageDown` | Page Up, Page Down | `ContextMenu` | Shift+F10 |
| `Copy`, `Cut`, `Paste` | Ctrl+C, Ctrl+X, Ctrl+V | `Undo`, `Redo`, `Search` | Ctrl+Z, Ctrl+Y, Ctrl+F |

`Keymap::default()` holds these; `bind(action, key)` adds a key to an
action, `unbind(action, &key)` removes one and `rebind(action, keys)`
replaces them all. A key is a `KeyBinding`: `KeyBinding::new(KeyCode::F(2))`,
with `.ctrl()`, `.alt()`, `.shift()` or `.meta()` for modifiers. The keymap
the widgets read is the active one: `Keymap::active()` returns it and
`Keymap::set_active(keymap)` or `App::set_keymap(keymap)` replaces it, from
the next event on. After `rebind(Action::Confirm, [F2])` a select chooses,
a checkbox toggles, a tab selects and a dialog's button presses on F2, and
no longer on Enter.

Typing wins: a widget that takes text keeps taking a plain character as
text, so a plain letter bound to an action reaches only widgets that take
no text. A key that belongs to one widget alone, a file explorer's toolbar
letter or a select's type-ahead, stays that widget's own. A widget that
treats Shift as a variant of a key, as the table does when Shift+Right
widens a column where Right moves to it, reads the unshifted key's action
with the shift flag.

Every binding has one text form, `KeyBinding::display()`: the modifiers as
`Ctrl+`, `Alt+`, `Shift+` and `Meta+` in that order, then the key's name,
`Enter`, `Escape`, `Space`, `Tab`, `F2`, `Up`, `Page Down`, a letter in
upper case. A menu item that names an action shows that binding as its
hint and triggers on it, so a rebind changes both (KEY-002).

## Limits

- A mouse handler needs presented geometry before it can receive hit-tested
  input.
- Focus registration alone does not paint a focus style.
- Pixel positions require terminal support and must not be treated as cells.
- The Kitty keyboard protocol, the startup questions and the Ctrl+Z suspend
  are Unix only; on Windows keys arrive as the console reports them.
- Event handlers must avoid long blocking work because they run in application
  event processing.

## Source map

- Event exports: [`src/event/mod.rs`](../src/event/mod.rs)
- Event values: [`src/event/types.rs`](../src/event/types.rs)
- Event router: [`src/event/router.rs`](../src/event/router.rs)
- Application event tree: [`src/app/event_tree.rs`](../src/app/event_tree.rs)
- Routing tests: [`tests/api_event_routing.rs`](../tests/api_event_routing.rs)
- Focus tests: [`tests/api_focus.rs`](../tests/api_focus.rs)

## Related chapters

- [Applications and components](app-and-components.md)
- [Input widgets](input-widgets.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
