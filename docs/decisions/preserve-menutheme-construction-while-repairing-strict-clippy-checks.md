# Preserve MenuTheme construction while repairing strict Clippy checks

Level: Judged
Decided by: Shawn and Codex
Rests on: MNT-002
Would be wrong if: The exception hides another warning or maintaining inline menu styles creates a measured resource problem.

## Decision

Keep the public MenuTheme::Custom(MenuStyle) construction contract. Boxing that field for the large-enum lint would break existing callers and introduce allocation. Use one documented expectation on that enum for large_enum_variant; do not suppress lint groups. Replace tautological assertions with observable checks and retain failure assertions as explicit panics. Other Clippy fixes retain behavior and existing defaults.

## Realized by

(none yet: recorded, not built)
