# Expose VElement property maps as component Props and preserve native round trips

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: A registered component cannot receive VElement properties and attributes, a changed value stays stale, or an Element round trip discards behavior or identity.
History: The mixed-builder bridge previously discarded properties and behavior. This completes the recorded preservation decision within the existing App pipeline.

## Decision

Add VElementProps, a cloneable comparable wrapper over typed VElement values and string attributes, implementing Component Props so registered components can consume and update the advertised property maps. VComponent continues to carry caller-defined typed props. Represent native Elements returned by VNode::from_element as opaque VComponent payloads; restore the complete Element, including factories, metadata, focus and keys, when converting back. This avoids adding an enum variant or duplicating every native runtime field in VDOM. Native children remain separately represented for composition. Verify actual registered-component updates and native text-input/button round trips through App.

## Realized by

Implementation: `src/vdom/node.rs`, `src/vdom/bridge.rs`, `src/builder/mixed.rs`.

Behavior checks: `tests/api_widget_behavior/vdom.rs`, `tests/api_widget_behavior/core_builders.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
