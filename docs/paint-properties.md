# Gradients and motion in App

The App/SuprTUI route carries explicit builder styles and gradient data through
registered component expansion and layout. Utility classes override the explicit
base style. Typed component props remain separate from paint metadata.

## Gradients

Linear gradients support all eight directions, optional from/via/to stops and
alpha. A via stop lies halfway along the gradient. Missing end stops extend the
nearest supplied color. A single cell samples the midpoint; an empty gradient has
no color. Backgrounds blend over existing cells without erasing their graphemes
when the background is translucent.

Gradient borders sample clockwise from the top-left, along each inset border
ring. Their interior remains transparent unless a background is also supplied.
Zero dimensions or zero border width produce no border. A single border cell
samples the midpoint. `render_border` returns top, right, reversed bottom and
reversed left edge samples; this helper samples a static configuration.

`GradientBorder::new` is static. `rainbow_border` sets `cycle_duration` to two
seconds. App cycles each stop through its RGB channel permutations, interpolating
between them and preserving alpha. `None` or zero duration disables the cycle.
The default builder border uses this rainbow constructor; a border created from
explicit gradient stops is static. `conic_gradient` supplies colors around the
perimeter; its cycle duration defaults to None.

The developer approved adding the public `cycle_duration: Option<Duration>`
field. Existing Rust struct literals must add `cycle_duration: None` to retain
static behavior. Constructor signatures stay the same. Older serialized borders
without the field deserialize as static borders.

## Clocks and transitions

App owns animation clocks by parent-scoped element keys, with sibling indices
used for unkeyed elements. Redraws and keyed reorders preserve phase. Changed
animation configurations restart their animation; removing a node releases its
clock. Separate Apps have independent clocks even when their keys match.

CSS animation classes and `StyleBuilder::with_css_animation` use the registered
`CssAnimationSpec`. App schedules intermediate frames without requiring input.
Completed finite animations stop requesting frames; active loops keep requesting
frames through App's bounded frame scheduling. `animate-none` disables CSS
animations and automatic border cycling. A zero animation duration applies its
endpoint without an ongoing clock.

Transitions sample colors, opacity and transforms. Color changes interpolate all
RGBA channels; a missing background is transparent. Interrupted transitions start
from the current displayed value, using the previous curve to calculate that
value before adopting the new duration and easing. Shadow transitions follow the
existing terminal shadow approximation, which changes background color.

## Property projection

| Animation values | App behavior |
| --- | --- |
| Opacity, Color | Alpha and foreground RGB |
| Position, Size | Cell translation and layout dimensions |
| Scale, Rotation | Cell placement and box extent; Rotation uses degrees |
| Transform | Translation, separate scale axes, skew and affine matrices |
| Property, Custom | Named numeric terminal styles listed below |
| CssProperty | Named styles with numeric units, color or supported string values |
| Multiple | Apply all child properties in order |
| PropertySet | Preserve every name; duration_offset starts a property within the enclosing progress interval; an easing override applies to that property's remaining interval |
| Keyframes | Apply every sampled named value, including color alpha |

Named numeric styles include opacity, x/y and translateX/Y, rotate/rotation,
scale and scaleX/Y, skewX/Y, width/height and min/max dimensions, left/right/top/
bottom, padding and margin sides, gap, flexGrow/flexShrink, zIndex, fontWeight and
lineHeight. Conventional hyphenated equivalents are accepted where defined by
the property parser. Colors use color/foreground/textColor or background/
backgroundColor. Boolean decoration values support bold, italic, underline and
strike. String styles support auto dimensions/margin, bold/normal font weight,
italic/normal font style, text case and hidden/visible overflow.

`px`, `em` and `rem` use terminal cells. `vw` and `vh` use the current viewport.
Percent dimensions use the containing box; percent translation uses the node's
own laid-out size; percent opacity and scale use 100% as one. Degree and radian
units are accepted for named angle properties. Different CSS units switch at the
midpoint while preserving each endpoint's value and unit. Unknown style names,
unsupported value/unit combinations and nonfinite numeric styles report errors
rather than silently losing a property. Arbitrary application data values are
not automatically terminal styles.

`TransformProperty::Rotate` and `rotate_animation` use radians consistently.
`AnimatedProperty::Rotation`, CSS rotation utilities and the degree-based
convenience API keep degrees. This correction was explicitly approved: callers
that depended on the old wrapped TransformProperty degree interpretation must
convert those endpoints to radians.

## Terminal approximations

Transforms move cell centers about the box center. Scaling changes cell placement
and box extent; rotation and skew move upright glyphs rather than changing the
host terminal's glyph bitmaps. Coordinates round to whole cells. Glyphs can
overlap when compressed; whole wide graphemes are clipped at the screen and
ancestor boundaries. A singular transform, including zero scale, paints nothing.
Nested transforms compose; clipping follows the ancestor's local axes.

The event API exposes rectangles. Transformed hit areas use a bounding rectangle
of the transformed box after clipping; rotated corners can therefore contain
unpainted cells. Fully clipped children have empty hit bounds. Painted geometry
is registered only after presentation succeeds.

## Verification

`tests/api_paint_properties.rs` checks independent captured terminal cells and
scheduled frame changes through App. `src/app/motion.rs` has deterministic clock
tests for transitions, interrupted updates, property timing, invalid values,
keyed continuity, App isolation, border cycling and cleanup. The declared
API-010 mechanism runs both groups. These tests concern paint delivery; the
broader typed-keyframe, target-binding and screen contracts have their separate
API-013 acceptance requirement.
