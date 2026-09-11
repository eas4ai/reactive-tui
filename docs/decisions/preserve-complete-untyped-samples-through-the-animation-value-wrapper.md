# Preserve complete untyped samples through the animation value wrapper

Level: Judged
Decided by: Codex
Rests on: API-013
Would be wrong if: Animation sampling drops a property, substitutes zero for a nonnumeric value, or loses a CSS unit or color alpha channel.
History: The approved API-013 keyframe value trait removed guessed typed conversion. This is the downstream wrapper of the same samples; it must not reintroduce value loss after checked keyframe sampling. Existing API reversals require checking real boundaries rather than only helper success.

## Decision

AnimatedProperty::Keyframes currently chooses one arbitrary sampled property and converts unsupported values to zero. Preserve all sampled properties using the existing AnimationValue::Map and its String, Boolean, Unit, Transform and Multiple variants. Preserve existing single numeric and opaque RGB representations. Represent nonopaque RGBA values with an explicit channel map because the existing RGB variant has no alpha field. This avoids adding variants to public exhaustive enums. Verify multi-property samples through Animation::seek, not only KeyframeSequence::sample.

Numeric `PropertyValue::Array` also discards interior values. Convert arrays with
more than two values to evenly spaced, linear KeyframeSequence frames, preserving
the existing two-endpoint representation. Bound samples follow those same segments.
The enclosing animation still applies its selected easing to overall progress.

## Realized by

- 96d93380f05256b3aa3124b269bf3ee39e77daed Resolve current animation values through owner-bound targets
