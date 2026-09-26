# Applications and components

Crate modules: `app`, `component`

## Purpose

Applications connect a root component to a backend. Components own typed props
and state and produce `Element` trees.

## Main API

- `RootComponent` has `render`, optional event handling, and optional update
  subscription methods.
- `App` owns the backend, root component, scheduler, waker, focus manager,
  animation manager, and render state.
- `AppBuilder` selects the backend, root, quit key, performance mode,
  accessibility options, and scheduler.
- `Component` defines props, state, initialization, update, render, events,
  layout notification, async polling, and lifecycle notification.
- `Element` is component output.
- `ComponentInstance`, `ComponentRegistry`, and `TrackedComponentInstance`
  manage mounted component values.

## Basic use

Implement `RootComponent` for a simple root. Use `Component` when a subtree
needs typed props, retained state, lifecycle callbacks, or local event handling.
Construct typed component elements with `Element::typed` or a generated
component helper.

## Behavior

The application loop waits for backend input, scheduled work, timer deadlines,
component polling, and wake notifications. A render expands typed components,
reconciles the tree, computes layout, paints a frame, sends it to the backend,
updates focus and accessibility state, and runs completed work.

Component methods have safe defaults. The default `update` requests a render.
The default event handler ignores the event. The default layout callback and
lifecycle callback do no work. `try_render` converts existing infallible
`render` implementations into the crate result type.

## Limits

- Root components and regular components must be `Send + Sync + 'static`.
- Component props must implement `Props`; state must be `Default + Send + Sync
  + 'static`.
- An application needs one backend and one root component before it can build.
- Component identity depends on type, position, and optional keys. Use stable
  keys when children can move.

## Source map

- Application loop and builder: [`src/app.rs`](../src/app.rs)
- Component traits and exports: [`src/component/mod.rs`](../src/component/mod.rs)
- Element model: [`src/component/element.rs`](../src/component/element.rs)
- Component runtime: [`src/component/runtime.rs`](../src/component/runtime.rs)
- Component API tests: [`tests/api_component_expansion.rs`](../tests/api_component_expansion.rs)
- Lifecycle tests: [`tests/api_hook_lifecycle.rs`](../tests/api_hook_lifecycle.rs)

## Related chapters

- [Elements, builders, and the virtual DOM](elements-builders-and-vdom.md)
- [Reactive state and hooks](reactive-state-and-hooks.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
