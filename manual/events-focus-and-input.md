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

The application rebuilds its event tree from the presented frame. Focus state
is preserved when possible and moved when a focused node disappears. Positions
preserve whether coordinates are terminal cells or pixels.

## Limits

- A mouse handler needs presented geometry before it can receive hit-tested
  input.
- Focus registration alone does not paint a focus style.
- Pixel positions require terminal support and must not be treated as cells.
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
