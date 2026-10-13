# Styles

Prefix: STY

An element's classes become its style through the class parser
(src/layout/css/optimizer.rs, `apply_utility_classes`), which hands a token
with a variant prefix to src/layout/css/variants.rs. In an App the variants
whose condition the App knows are decided before layout
(src/app/event_tree.rs, `style_node`): `focus:`, `focus-within:`, `hover:`,
`disabled:` and the width breakpoints `sm:` to `xl:`; a token whose
condition does not hold is dropped, and any other prefix reaches the parser
as it is. The `css!` macro (src/layout/css/css_in_rust.rs) builds a style
from property and value pairs, and the container utilities
(src/layout/css/containers.rs) include the aspect-ratio classes.

Read on 2026-10-04 from the developer's production code review of 65e618ec
(findings C05, C06, C12 and C13) and checked against the code on
2026-10-05: the parser applies the utility of `active:`, `visited:`,
`first:`, `last:`, `odd:`, `even:`, `group-hover:`, `group-focus:` and
`group-active:` without looking at the condition; `css!` drops
`display: Display::None`; `aspect-auto` leaves an earlier aspect ratio in
place; and the `css!` documentation promises compile-time validation and
type safety of properties that the macro does not check.

## Observed

(none yet)

## Draft

[STY-001] A class variant's utility MUST apply only while its condition holds. In an App: `first:` and `last:` on the first and last of a parent's children; `odd:` and `even:` on the first, third, fifth child and so on, and on the second, fourth and so on; `active:` from a mouse press inside the element until its release; `group-hover:`, `group-focus:` and `group-active:` while the nearest ancestor with the class `group` is hovered, focused or pressed; and `visited:` never. Outside an App, where no condition is known, the parser MUST apply none of these utilities.
Falsifier: `apply_utility_classes` of `group-hover:opacity-50`, or of the same utility under `active:`, `visited:`, `first:`, `last:`, `odd:`, `even:`, `group-focus:` or `group-active:`, changes the style it is given; or in an App, a column of three children with `first:`, `last:`, `odd:` or `even:` background classes paints a child whose position does not match, a child with `active:bg-primary` is painted primary with no press on it or not while a press is held on it, a child with `group-hover:bg-primary` inside an ancestor with the class `group` is painted primary while the pointer is outside the group or not while the pointer is over it, or `visited:` applies.
Mechanism: review-core
Rationale: The parser applied these variants' utilities without looking at their conditions (the developer's code review of 2026-10-04, C05).
Status: Agreed 2026-10-05

[STY-002] `display: Display::None` in `css!` MUST build a style whose display is none, and an element with that style MUST take no space in its parent's layout, paint no cell, and receive no mouse event, nor may its children.
Falsifier: `css! { display: Display::None }` builds a style whose display is not none; or such an element between two siblings in a row keeps them apart, paints a cell, or receives a click at the cells it would have taken.
Mechanism: review-core
Rationale: `css!` returned the style unchanged for `Display::None`, so the element stayed in layout and was painted (the developer's code review of 2026-10-04, C06).
Status: Agreed 2026-10-05

[STY-003] The class `aspect-auto` MUST clear an aspect ratio an earlier class set.
Falsifier: `apply_utility_classes("aspect-square aspect-auto", StyleBuilder::new())` builds a style whose aspect ratio is set.
Mechanism: review-core
Rationale: The `aspect-auto` branch returned the builder unchanged (the developer's code review of 2026-10-04, C12).
Status: Agreed 2026-10-05

[STY-004] The documentation of the `css!` macro MUST say what the macro checks: property names are matched when the style is built, and an unknown property or a value of the wrong kind is ignored; it MUST NOT claim that properties are validated or type-checked at compile time.
Falsifier: A doc comment in src/layout/css/css_in_rust.rs says properties are validated or type-checked at compile time, or none says that an unknown property or a value of the wrong kind is ignored.
Mechanism: review-core
Rationale: The module promised compile-time validation and type safety the macro does not provide (the developer's code review of 2026-10-04, C13).
Status: Agreed 2026-10-05
