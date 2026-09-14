# Layout, style, and themes

Crate modules: `layout`, `theme`

## Purpose

Layout assigns terminal-cell rectangles to elements. Style controls color,
spacing, borders, text, opacity, gradients, positioning, clipping, and motion.
Themes supply reusable variables and color presets.

## Main API

- `LayoutEngine` wraps Taffy nodes and layout computation.
- `StyleBuilder` and inline style values define typed properties.
- `apply_utility_classes` parses utility class strings.
- CSS modules cover layout, sizing, spacing, typography, colors, gradients,
  effects, interaction states, accessibility states, variants, and animation.
- `Theme` stores a name and `ThemeVariables`, can extend another theme, and can
  apply classes through its variables.
- Theme presets include dark, light, high-contrast, Solarized Dark, and
  Gruvbox Dark.

## Basic use

Use builder classes for concise layouts. Use `StyleBuilder` when values are
computed or when typed construction is clearer. Pass a theme to the themed
utility-class path when classes reference variables.

## Behavior

The component bridge converts elements into layout `NodeSpec` values. Taffy
computes flex and grid geometry. The paint tree resolves classes and inline
styles, applies clipping and stacking, and writes the resulting cells and
placement metadata.

Theme extension starts from a base theme and replaces selected variables.
Utility classes are applied in input order, so later compatible utilities can
replace earlier values.

## Limits

- Layout values are terminal-cell measurements after computation.
- Unknown utility classes do not create new behavior.
- Invalid or non-finite numeric style values return errors in checked paths.
- Terminal color output may be reduced according to detected capabilities.
- Absolute positioning and z-index affect paint order and clipping.

## Source map

- Layout exports and Taffy wrapper: [`src/layout/mod.rs`](../src/layout/mod.rs)
- Utility-class entry points: [`src/layout/css/mod.rs`](../src/layout/css/mod.rs)
- Typed styles: [`src/layout/style.rs`](../src/layout/style.rs)
- Theme API: [`src/theme/mod.rs`](../src/theme/mod.rs)
- Styling API tests: [`tests/api_styling.rs`](../tests/api_styling.rs)
- Layout paint tests: [`tests/layout_paint_tests.rs`](../tests/layout_paint_tests.rs)

## Related chapters

- [Elements, builders, and the virtual DOM](elements-builders-and-vdom.md)
- [Rendering and backends](rendering-and-backends.md)
- [Animation and screens](animation-and-screens.md)

[Back to the manual](README.md)
