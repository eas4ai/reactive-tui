# Preserve keyframe values through explicit interpolation and checked conversion

Level: Judged
Decided by: Codex
Rests on: API-013
Would be wrong if: Typed values still step or lose precision, untyped conversion drops values or picks an arbitrary property, keyframe easing is ignored, or custom value migration is undocumented.
History: The API-wide reversal history concerns clipboard deadlines and ownership, ConPTY ownership, terminal history reflow and AT-SPI translation. None supplies keyframe interpolation semantics. The custom-type source compatibility change has already been explicitly approved in api-013; this implementation decision remains Judged within that approval and requires failing and corrected examples.

## Decision

Apply the approved api-013 compatibility correction. Add a public keyframe value trait that supplies typed interpolation and checked conversion from KeyframeValue. Implement the documented numeric, tuple, color, transform, CSS and discrete types. Keep existing constructor and sampling names, use linear easing when typed keyframes omit easing, and use the destination keyframe easing for each segment. Sort typed offsets and resolve duplicate offsets with the last authored value; reject nonfinite/out-of-range offsets. Add fallible constructors and named-property conversion; the existing infallible constructor reports invalid/ambiguous input instead of substituting defaults. Sample sparse untyped properties using their own surrounding keyframes and retain discrete values until their destination keyframe. Preserve numeric endpoints, alpha, units and compound values. Verify exact values, easing, invalid input, custom trait implementations and actual intermediate App frames before acceptance.

## Realized by

- 8fbafb0200a7a49fac02a14667f2cf53388b43f5 Repair typed keyframes and retain hook playback through component lifecycle

src/animation/keyframes.rs and src/animation/keyframes/typed.rs; tests/api_animation_screens.rs covers typed values, checked conversion and easing. The custom-type migration example compiles as a doctest.
