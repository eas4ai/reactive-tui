//! CSS-in-Rust support for reactive-tui
//!
//! This module provides macros and utilities for writing CSS directly in Rust code
//! with type safety and compile-time validation.
//!
//! # Features
//! - Type-safe CSS property macros
//! - Compile-time validation of CSS values
//! - Integration with the existing CSS utility system
//! - Support for all CSS properties used in reactive-tui
//!
//! # Example
//! ```rust
//! use reactive_tui::css;
//! use reactive_tui::prelude::{Display, FlexDirection};
//!
//! let styles = css! {
//!     display: Display::Flex,
//!     flex_direction: FlexDirection::Column,
//!     background_color: (0.0, 0.0, 1.0, 1.0), // Blue RGBA
//!     padding: 16.0,
//!     opacity: 0.9,
//! };
//! ```

use crate::layout::style::{
    AlignItems as RAlignItems, Direction, JustifyContent as RJustifyContent, StyleBuilder,
};
use taffy::style::{AlignItems, Display, FlexDirection, JustifyContent, Position};

/// Main CSS-in-Rust macro for creating styles
///
/// This macro provides a type-safe way to write CSS properties directly in Rust.
/// All properties are validated at compile time and converted to the appropriate
/// internal representations.
///
/// # Supported Properties
/// - `display`: Display (Flex, Block, None, etc.)
/// - `flex_direction`: FlexDirection (Row, Column, RowReverse, ColumnReverse)
/// - `align_items`: AlignItems (Start, End, Center, Stretch, Baseline)
/// - `justify_content`: JustifyContent (Start, End, Center, SpaceBetween, etc.)
/// - `position`: Position (Relative, Absolute)
/// - `color`: (f32, f32, f32, f32) - RGBA tuple for foreground color
/// - `background_color`: (f32, f32, f32, f32) - RGBA tuple for background color
/// - `padding`: f32 - Padding for all sides in terminal cells
/// - `margin`: f32 - Margin for all sides in terminal cells
/// - `opacity`: f32 (0.0 to 1.0)
/// - `width`: f32 (width in terminal cells)
/// - `height`: f32 (height in terminal cells)
///
/// # Example
/// ```rust,ignore
/// use reactive_tui::css;
/// use reactive_tui::prelude::{Display, AlignItems, JustifyContent};
///
/// let button_styles = css! {
///     display: Display::Flex,
///     align_items: AlignItems::Center,
///     justify_content: JustifyContent::Center,
///     background_color: (0.0, 0.0, 1.0, 1.0), // Blue RGBA
///     color: (1.0, 1.0, 1.0, 1.0), // White RGBA
///     padding: 12.0,
///     opacity: 1.0,
/// };
/// ```
#[macro_export]
macro_rules! css {
    ($($property:ident: $value:expr),* $(,)?) => {
        {
            let mut sb = $crate::layout::style::StyleBuilder::new();
            $(
                sb = $crate::layout::css::css_in_rust::apply_css_property(sb, stringify!($property), $value);
            )*
            sb
        }
    };
}

/// Apply a single CSS property to a StyleBuilder
///
/// This function is used internally by the `css!` macro to apply individual
/// CSS properties. It provides type-safe conversion from Rust values to
/// the internal style representation.
pub fn apply_css_property<T>(sb: StyleBuilder, property: &str, value: T) -> StyleBuilder
where
    T: IntoCssValue,
{
    value.apply_to_style_builder(sb, property)
}

/// Trait for converting Rust values into CSS values
///
/// This trait allows the CSS-in-Rust system to accept various Rust types
/// and convert them to the appropriate CSS representations.
pub trait IntoCssValue {
    /// Apply this value to a StyleBuilder for the given CSS property
    ///
    /// # Arguments
    /// * `sb` - The StyleBuilder to modify
    /// * `property` - The CSS property name
    ///
    /// # Returns
    /// The modified StyleBuilder
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder;
}

// Implementations for common CSS value types

impl IntoCssValue for Display {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "display" => match self {
                Display::None => sb, // None means don't display - handled by visibility
                Display::Flex => sb.display_flex(),
                Display::Grid => sb.display_grid(),
                Display::Block => sb, // Block is the default in TUI context
            },
            _ => sb, // Ignore unknown properties
        }
    }
}

impl IntoCssValue for FlexDirection {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "flex_direction" => {
                let direction = match self {
                    FlexDirection::Row => Direction::Row,
                    FlexDirection::Column => Direction::Column,
                    FlexDirection::RowReverse => Direction::RowReverse,
                    FlexDirection::ColumnReverse => Direction::ColumnReverse,
                };
                sb.direction(direction)
            }
            _ => sb,
        }
    }
}

impl IntoCssValue for AlignItems {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "align_items" => {
                let align = match self {
                    AlignItems::Start => RAlignItems::Start,
                    AlignItems::End => RAlignItems::End,
                    AlignItems::Center => RAlignItems::Center,
                    AlignItems::Stretch => RAlignItems::Stretch,
                    _ => RAlignItems::Start, // Default fallback
                };
                sb.align_items(align)
            }
            _ => sb,
        }
    }
}

impl IntoCssValue for JustifyContent {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "justify_content" => {
                let justify = match self {
                    JustifyContent::Start => RJustifyContent::Start,
                    JustifyContent::End => RJustifyContent::End,
                    JustifyContent::Center => RJustifyContent::Center,
                    JustifyContent::SpaceBetween => RJustifyContent::SpaceBetween,
                    JustifyContent::SpaceAround => RJustifyContent::SpaceAround,
                    JustifyContent::SpaceEvenly => RJustifyContent::SpaceEvenly,
                    _ => RJustifyContent::Start, // Default fallback
                };
                sb.justify(justify)
            }
            _ => sb,
        }
    }
}

impl IntoCssValue for Position {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "position" => match self {
                Position::Relative => sb.position_relative(),
                Position::Absolute => sb.position_absolute(),
            },
            _ => sb,
        }
    }
}

impl IntoCssValue for (f32, f32, f32, f32) {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "color" => sb.text_rgba(self.0, self.1, self.2, self.3),
            "background_color" => sb.bg_rgba(self.0, self.1, self.2, self.3),
            _ => sb,
        }
    }
}

impl IntoCssValue for f32 {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "opacity" => sb.opacity(self),
            "width" => sb.size_px(Some(self), None),
            "height" => sb.size_px(None, Some(self)),
            "padding" => sb
                .padding_t_px(self)
                .padding_r_px(self)
                .padding_b_px(self)
                .padding_l_px(self),
            "margin" => sb
                .margin_t_px(self)
                .margin_r_px(self)
                .margin_b_px(self)
                .margin_l_px(self),
            _ => sb,
        }
    }
}

impl IntoCssValue for i32 {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "z_index" => sb.z_index(self),
            _ => sb,
        }
    }
}

impl IntoCssValue for u32 {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        match property {
            "z_index" => sb.z_index(self as i32),
            _ => sb,
        }
    }
}

// String support for dynamic values
impl IntoCssValue for &str {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        // For string values, parse them as CSS utility classes
        match property {
            "class" | "classes" => {
                // Apply CSS utility classes
                crate::layout::css::apply_utility_classes(self, sb)
            }
            _ => {
                // Unknown string property, return unchanged
                sb
            }
        }
    }
}

impl IntoCssValue for String {
    fn apply_to_style_builder(self, sb: StyleBuilder, property: &str) -> StyleBuilder {
        self.as_str().apply_to_style_builder(sb, property)
    }
}

/// Convenience macro for creating common layout patterns
///
/// # Example
/// ```rust,ignore
/// use reactive_tui::{flex_center, flex_column, absolute_fill};
///
/// let flex_center = flex_center!();
/// let flex_column = flex_column!();
/// let absolute_fill = absolute_fill!();
/// ```
#[macro_export]
macro_rules! flex_center {
    () => {
        $crate::css! {
            display: $crate::prelude::Display::Flex,
            align_items: $crate::prelude::AlignItems::Center,
            justify_content: $crate::prelude::JustifyContent::Center,
        }
    };
}

/// Create a flex column layout
#[macro_export]
macro_rules! flex_column {
    () => {
        $crate::css! {
            display: $crate::prelude::Display::Flex,
            flex_direction: $crate::prelude::FlexDirection::Column,
        }
    };
}

/// Create an absolutely positioned element that fills its container
#[macro_export]
macro_rules! absolute_fill {
    () => {
        $crate::css! {
            position: $crate::prelude::Position::Absolute,
        }
        .width_pct(100.0)
        .height_pct(100.0)
        .inset_all(0.0)
    };
}

/// Macro for responsive CSS with breakpoint support
///
/// App and ScreenManager select styles for their current terminal width. Minimum
/// widths are sm=40, md=80, lg=120 and xl=160 columns. Blocks cascade in ascending
/// width order; repeated widths keep declaration order. Each block evaluates once
/// during construction. Only owned style data crosses the renderer boundary.
/// Complete each configuration inside the macro. Shared Element utility classes
/// apply afterward. Standalone callers use `.at_width(columns)` before further
/// builder methods and `.build()`.
///
/// # Example
/// ```rust,ignore
/// use reactive_tui::responsive_css;
/// use reactive_tui::prelude::Display;
///
/// let responsive_styles = responsive_css! {
///     base: {
///         display: Display::Block,
///         padding: 1.0,
///     },
///     md: {
///         display: Display::Flex,
///         padding: 2.0,
///     },
///     lg: {
///         padding: 3.0,
///     },
/// };
/// ```
/// Unknown breakpoint names are rejected:
/// ```compile_fail
/// let styles = reactive_tui::responsive_css! { base: {}, mobile: { padding: 1.0 } };
/// ```
#[macro_export]
macro_rules! responsive_css {
    (
        base: { $($base_prop:ident: $base_val:expr),* $(,)? }
        $(, $breakpoint:ident: { $($bp_prop:ident: $bp_val:expr),* $(,)? })*
        $(,)?
    ) => {
        {
            let base = $crate::css! { $($base_prop: $base_val),* };
            let rules: ::std::vec::Vec<$crate::layout::css::css_in_rust::ResponsiveRule<'_>> = ::std::vec![$(
                ($crate::responsive_breakpoint!($breakpoint), ::std::boxed::Box::new(|mut sb| {
                    $(sb = $crate::apply_css_property(sb, stringify!($bp_prop), $bp_val);)*
                    sb
                }))
            ),*];
            $crate::layout::css::css_in_rust::responsive_style(base, rules)
        }
    };
}

/// Construction-only responsive rule. Closures are consumed before style capture.
#[doc(hidden)]
pub type ResponsiveRule<'a> = (u16, Box<dyn FnOnce(StyleBuilder) -> StyleBuilder + 'a>);

/// Materialize responsive macro blocks as cumulative style data.
#[doc(hidden)]
pub fn responsive_style(
    mut base: StyleBuilder,
    mut rules: Vec<ResponsiveRule<'_>>,
) -> StyleBuilder {
    rules.sort_by_key(|(width, _)| *width);
    let mut current = base.clone();
    let mut profiles = Vec::new();
    for (width, rule) in rules {
        current = rule(current);
        if profiles
            .last()
            .is_some_and(|(previous, _)| *previous == width)
        {
            profiles.pop();
        }
        profiles.push((width, current.clone()));
    }
    base.responsive_profiles = profiles;
    base
}

/// Resolve the public responsive macro's supported terminal-column breakpoints.
#[doc(hidden)]
#[macro_export]
macro_rules! responsive_breakpoint {
    (sm) => {
        40_u16
    };
    (md) => {
        80_u16
    };
    (lg) => {
        120_u16
    };
    (xl) => {
        160_u16
    };
    ($other:ident) => {
        compile_error!("unknown responsive breakpoint; use sm, md, lg or xl")
    };
}

#[cfg(test)]
mod tests {
    use taffy::style::{Display, FlexDirection};

    #[test]
    fn test_basic_css_macro() {
        let styles = css! {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
        };

        let style = styles.build();
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn test_color_properties() {
        let mut styles = css! {
            color: (1.0, 0.0, 0.0, 1.0), // Red
            background_color: (0.0, 0.0, 1.0, 1.0), // Blue
        };

        let visuals = styles.take_visuals().expect("color properties");
        assert_eq!(
            visuals.fg,
            crate::core::surface::Rgba::new(1.0, 0.0, 0.0, 1.0)
        );
        assert_eq!(
            visuals.bg,
            crate::core::surface::Rgba::new(0.0, 0.0, 1.0, 1.0)
        );
    }

    #[test]
    fn test_spacing_properties() {
        let styles = css! {
            padding: 16.0,
            margin: 8.0,
        };

        let style = styles.build();
        assert_eq!(
            style.padding,
            taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::length(16.0),
                right: taffy::style::LengthPercentage::length(16.0),
                top: taffy::style::LengthPercentage::length(16.0),
                bottom: taffy::style::LengthPercentage::length(16.0)
            }
        );
        assert_eq!(
            style.margin.left,
            taffy::style::LengthPercentageAuto::length(8.0)
        );
        assert_eq!(style.margin.right, style.margin.left);
        assert_eq!(style.margin.top, style.margin.left);
        assert_eq!(style.margin.bottom, style.margin.left);
    }

    #[test]
    fn test_numeric_properties() {
        let mut styles = css! {
            opacity: 0.8,
            width: 100.0,
            height: 50.0,
        };

        let visuals = styles.take_visuals().expect("opacity");
        assert_eq!(visuals.fg.a, 0.8);
        assert_eq!(visuals.bg.a, 0.8);
        let style = styles.build();
        assert_eq!(style.size.width, taffy::style::Dimension::length(100.0));
        assert_eq!(style.size.height, taffy::style::Dimension::length(50.0));
    }

    #[test]
    fn test_convenience_macros() {
        let center = flex_center!();
        let column = flex_column!();
        let fill = absolute_fill!();

        let center = center.build();
        assert_eq!(center.align_items, Some(taffy::style::AlignItems::Center));
        assert_eq!(
            center.justify_content,
            Some(taffy::style::JustifyContent::Center)
        );
        assert_eq!(column.build().flex_direction, FlexDirection::Column);
        let fill = fill.build();
        assert_eq!(fill.position, taffy::style::Position::Absolute);
        assert_eq!(fill.size.width, taffy::style::Dimension::percent(1.0));
        assert_eq!(fill.size.height, taffy::style::Dimension::percent(1.0));
    }
}
