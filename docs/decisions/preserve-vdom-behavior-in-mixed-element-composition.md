# Preserve VDOM behavior in mixed Element composition

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: Converting a VDOM node still discards its handlers, style or props, callbacks receive no defined payload, or disabled and measured App routing are bypassed.
History: Earlier API reversals exposed nominal adapters without native behavior. This remains Judged because it completes the existing conversion route and reuses App ownership rather than introducing another event or rendering loop.

## Decision

Convert VElement event handlers, typed property maps and inline style declarations into the existing Element metadata and props. Use App routing and its measured targets, focus, disabled state and bubbling. Document event names and pass the native Event as the Any payload; activation works with both keyboard and pointer input. Preserve VComponent opaque props and VDOM keys. Define inline declarations in terms of the supported terminal layout and paint properties, report invalid declarations visibly, and verify conversion through mixed_container, from_vdom, IntoElement and composition macros. Add safe failing and corrected App cases, including callback replacement and disabled input.

## Realized by

Implementation: `src/vdom/node.rs`, `src/vdom/bridge.rs`, `src/builder/mixed.rs`.

Behavior checks: `tests/api_widget_behavior/vdom.rs`, `tests/api_widget_behavior/core_builders.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
