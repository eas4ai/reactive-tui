# Give rainbow borders an explicit owned cycle duration

Level: Judged
Decided by: Shawn
Rests on: API-010,API-018
Would be wrong if: Static borders animate, clocks leak between Apps or survive removal, or constructors change signatures.
History: Clipboard deadline reversals do not affect this source compatibility decision. Shawn explicitly approved the additional GradientBorder field in api-010-api-018.

## Decision

Add public cycle_duration: Option<Duration> to GradientBorder. The new constructor uses None and rainbow_border uses a two-second cycle. Existing struct literals must supply the field; this source change is explicitly approved. App owns stable keyed clocks and removes them on unmount. Preserve constructor signatures and the C ABI. Verify intermediate frames, static borders, keyed continuity, removal and isolation.

## Realized by

src/app/motion.rs and src/app/motion/property.rs; src/component/bridge.rs; src/layout/paint_tree/suprtui.rs and transform.rs; src/layout/css/gradients.rs. Development coverage is in tests/api_paint_properties.rs and the app::motion::tests unit module. Formal acceptance is recorded separately by Cairn.
