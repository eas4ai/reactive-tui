# Default-suite repair review

## Contract and baseline plan

The six failures are two assertions and four doctest compile failures. The
JumpStart assertion conflicts with the already implemented jump-at-zero rule:
W3C CSS Easing Level 1 section 2.3 increments floor(progress * steps) for
jump-start and clamps the final value to one. Its API has no before flag.
Correct that assertion with boundary coverage instead of changing valid easing.
Numeric offsets currently call position_absolute, overwriting both positioning
and the existing z-index; non-numeric offsets already use inset setters alone.
Remove the duplicate numeric path and exercise each side and position mode.
Doctests need visible imports; do not ignore or convert the examples to text.
