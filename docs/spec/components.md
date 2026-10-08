# Components

Prefix: CMP

A component (src/component/mod.rs) renders an `Element`; the App and a
`ScreenManager` (src/screen) mount it, keep its instance in the component
registry under its key (src/component/registry.rs) and feed its hooks
(src/hooks): mouse hooks such as `use_hover`, `use_clicks` and
`use_mouse_position` (src/hooks/mouse.rs) through the mouse event processor
(src/hooks/processor.rs). A render tree (src/render/tree.rs) built from an
element registers the component instances it creates and unregisters them
when it is dropped. The crate also offers a standalone virtual DOM with
`vdom::diff_vnodes` (src/vdom/diff.rs), an event handler cache
(src/event/cache.rs: `HandlerChain`, `HandlerLookup`) beside the App's
`EventRouter`, and a component lookup cache (src/component/cache.rs).

Read on 2026-10-04 from the developer's production code review of 65e618ec
(findings C01 to C04, C09 to C11, C14 and C15) and checked against the code
on 2026-10-05: a dropped render tree unregisters by key whatever instance
holds it now, so dropping an old tree unmounts the component that replaced
it; `Component::poll_change` is offered and forwarded but no App or screen
polls it, and its wrapper compares two pointers taken from the same
reference as if that verified pinning; a screen routes mouse events to
handlers but never to the mouse hooks; a move out of a component leaves its
`use_mouse_position` inside; `diff_vnodes` patches a node in place when its
element name, component name, props or handlers changed; `HandlerLookup`
keys its slots by node id modulo 256 and `HandlerChain::execute` reports
handled input as ignored; and the "compile-time" component table is an
empty map that nothing fills or reads.

## Observed

(none yet)

## Draft

[CMP-001] Dropping a render tree MUST unregister and unmount only the component instances that tree registered; an instance a newer tree registered under the same key MUST stay registered and mounted.
Falsifier: Two render trees are built one after the other from elements holding a component under the key `panel`, and dropping the older tree unmounts the newer tree's instance or leaves no instance registered under `panel`.
Mechanism: review-core
Rationale: A render tree's drop unregistered by key whatever instance held it, so dropping an old tree unmounted its replacement (the developer's code review of 2026-10-04, C01).
Status: Agreed 2026-10-05

[CMP-002] An App and a `ScreenManager` MUST poll a mounted component's `Component::poll_change` when the component mounts and again each time the waker of its last poll is woken, and MUST render the component again for the next frame after a poll returns `Ready`; the poll MUST reach the component through a structural pin projection whose safety comment states the invariant that makes it sound, and no runtime check that cannot fail MAY be presented as verifying it.
Falsifier: A component whose `poll_change` counts its polls, keeps its waker and returns `Ready` after another thread changes its text and wakes it: under an App or a `ScreenManager` the count is still zero after mounting, or the new text is not in a frame within a second of the wake; or the wrapper's `poll_change_any` still compares two pointers taken from the same reference.
Mechanism: review-core
Rationale: `poll_change` was offered and forwarded but no runtime polled it, and its wrapper's pointer comparison could not fail (the developer's code review of 2026-10-04, C02 and C15).
Status: Agreed 2026-10-05

[CMP-003] A `ScreenManager` MUST feed the mouse events it dispatches, and the clicks it makes from a press and a release, to its components' mouse hooks as an App does, so `use_hover`, `use_clicks` and `use_mouse_position` of a component on a screen change as they would in an App.
Falsifier: On a `ScreenManager` screen, a component with `use_hover` and `use_clicks` still reports no hover after the pointer moves into its box, or a click count of 0 after a press and a release in it.
Mechanism: review-core
Rationale: A screen routed mouse events to handlers only, so the hooks of a component on a screen never changed (the developer's code review of 2026-10-04, C03).
Status: Agreed 2026-10-05

[CMP-004] A component's `use_mouse_position` MUST report `is_inside` false once the pointer moves to a point outside the component, over another component or over none, with no leave event needed.
Falsifier: After a move inside component A and then a move over component B, or over no component, A's `use_mouse_position` still reports `is_inside` true.
Mechanism: review-core
Rationale: A move cleared only the hover state of the component the pointer left, not its position hook (the developer's code review of 2026-10-04, C04).
Status: Agreed 2026-10-05

[CMP-005] `vdom::diff_vnodes` MUST replace a node whose element name or component name differs between the old and the new tree, or whose props or event handlers differ, a handler differing when it is not the same shared handler, instead of patching its attributes and children in place.
Falsifier: `diff_vnodes` of a keyed `flex` element against a `grid` element with the same key, or of two elements or components that differ only in a prop, an event handler or the component's name, gives no `Patch::Replace` for that node.
Mechanism: review-core
Rationale: Patching compared only the node's kind and key, so a changed element name, component name, prop or handler was patched in place (the developer's code review of 2026-10-04, C09).
Status: Agreed 2026-10-05

[CMP-006] `event::cache::HandlerLookup` MUST return a handler chain only for the node id, event kind and phase it was inserted under and MUST keep every insertion, whatever the ids; `HandlerChain::execute` MUST return `Consumed` at the first handler that consumes the event, `Handled` when a handler handled or captured it and none consumed it, and `Ignored` only when every handler ignored it.
Falsifier: Chains inserted for node ids 1 and 257 with the same event kind and phase come back for the other id, one of them is lost, or a lookup of an id never inserted returns a chain; or a chain whose one handler returns `Handled` or `Captured` executes to `Ignored`.
Mechanism: review-core
Rationale: The lookup kept one slot per node id modulo 256 with no check of the id, and `execute` returned `Ignored` after handlers that handled or captured the event (the developer's code review of 2026-10-04, C10 and C11).
Status: Agreed 2026-10-05

[CMP-007] The crate MUST NOT offer a component lookup that nothing fills: `component::cache::CommonComponents` and `common_components()` are removed, and no documentation in src/component says a component table is computed at compile time or uses perfect hashing.
Falsifier: The crate's source still defines `CommonComponents` or `common_components`, or a doc comment in src/component says a component table is computed at compile time or uses perfect hashing.
Mechanism: review-core
Rationale: The advertised compile-time perfect component lookup was an empty map that nothing filled or read (the developer's code review of 2026-10-04, C14).
Status: Agreed 2026-10-05
