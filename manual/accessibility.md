# Accessibility

Crate modules: `accessibility`

## Purpose

Accessibility metadata describes element role, name, value, state, actions, and
relationships for assistive technology.

## Main API

- `accessibility::Node` wraps public AccessKit semantics used by elements.
- Re-exported `Role`, `Live`, and `Toggled` values describe common semantics.
- Node setters define labels, descriptions, expanded and selected state,
  toggles, disabled, hidden, read-only, value, live regions, click actions,
  and busy state: `set_busy` marks content that is still being prepared, as
  ARIA `aria-busy` does, and `is_busy` reads it.
- `Element::with_accessibility`, `with_accessibility_label`, and
  `with_accessibility_id` attach semantics.
- `AppBuilder::screen_reader` and `accessibility_name` configure publication.

## Basic use

Create a node with the closest semantic role, add a concise label and current
state, then attach it to the element. Give interactive elements an action and
keep the accessibility state synchronized with visible state.

## Behavior

After a frame is presented, the application builds an accessibility snapshot
from the element and layout trees. Geometry uses the presented cell placement.
On Linux, the owned platform adapter publishes changes through AT-SPI and maps
actions back into application events.

Text input exposes readable lines, cursor, and selection. Password input hides
private content in the accessibility tree. Hidden and disabled semantics affect
the published node state.

## Limits

- The native screen-reader connection is currently implemented for Linux.
- Semantics do not replace keyboard focus or event handlers.
- Labels must describe the control; the framework cannot infer a useful label
  from arbitrary painted text.
- Accessibility geometry is current only after successful presentation.

## Source map

- Public accessibility node: [`src/accessibility/mod.rs`](../src/accessibility/mod.rs)
- Application snapshot builder: [`src/app/event_tree/accessibility.rs`](../src/app/event_tree/accessibility.rs)
- Linux platform adapter: [`src/accessibility/platform/unix/adapter.rs`](../src/accessibility/platform/unix/adapter.rs)
- Accessibility API probe: [`tests/api_widget_behavior/accessibility_probe.rs`](../tests/api_widget_behavior/accessibility_probe.rs)
- Accessibility style tests: [`tests/api_widget_behavior/accessibility_styles.rs`](../tests/api_widget_behavior/accessibility_styles.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Input widgets](input-widgets.md)
- [Applications and components](app-and-components.md)

[Back to the manual](README.md)
