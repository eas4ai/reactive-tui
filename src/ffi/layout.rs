//! Layout system FFI functions

use super::*;
use crate::layout::css::apply_utility_classes;
use crate::layout::style::{StyleBuilder, ComputedStyle};
use crate::layout::css::css_in_rust::apply_css_property;
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to a style builder
#[repr(C)]
pub struct RTuiStyleBuilder {
    _private: [u8; 0],
}

/// Opaque handle to computed styles
#[repr(C)]
pub struct RTuiComputedStyle {
    _private: [u8; 0],
}

/// Display type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiDisplayType {
    None = 0,
    Flex = 1,
    Grid = 2,
    Block = 3,
    Inline = 4,
    InlineBlock = 5,
}

/// Flex direction enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiFlexDirection {
    Row = 0,
    RowReverse = 1,
    Column = 2,
    ColumnReverse = 3,
}

/// Justify content enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiJustifyContent {
    FlexStart = 0,
    FlexEnd = 1,
    Center = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
}

/// Align items enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiAlignItems {
    FlexStart = 0,
    FlexEnd = 1,
    Center = 2,
    Baseline = 3,
    Stretch = 4,
}

/// Position type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiPositionType {
    Relative = 0,
    Absolute = 1,
    Fixed = 2,
    Static = 3,
}

/// Dimension value (can be pixels, percentage, or auto)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiDimension {
    pub value: f32,
    pub unit: RTuiDimensionUnit,
}

/// Dimension unit enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiDimensionUnit {
    Pixels = 0,
    Percent = 1,
    Auto = 2,
    Flex = 3,
}

/// RGBA color for styling
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiStyleColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Create a new style builder
#[no_mangle]
pub extern "C" fn rtui_style_builder_create(out_builder: *mut *mut RTuiStyleBuilder) -> ReactiveError {
    if out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let builder = StyleBuilder::new();
        let boxed = Box::new(builder);
        unsafe {
            *out_builder = Box::into_raw(boxed) as *mut RTuiStyleBuilder;
        }
        Ok(())
    }))
}

/// Destroy a style builder
#[no_mangle]
pub extern "C" fn rtui_style_builder_destroy(builder: *mut RTuiStyleBuilder) {
    if !builder.is_null() {
        unsafe {
            let _ = Box::from_raw(builder as *mut StyleBuilder);
        }
    }
}

/// Apply utility CSS classes to style builder
#[no_mangle]
pub extern "C" fn rtui_apply_utility_classes(
    builder: *mut RTuiStyleBuilder,
    classes: *const c_char,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || classes.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let classes_str = CStr::from_ptr(classes)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let new_builder = apply_utility_classes(classes_str, *builder_box);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set display property
#[no_mangle]
pub extern "C" fn rtui_style_builder_display(
    builder: *mut RTuiStyleBuilder,
    display: RTuiDisplayType,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_display = match display {
            RTuiDisplayType::None => taffy::style::Display::None,
            RTuiDisplayType::Flex => taffy::style::Display::Flex,
            RTuiDisplayType::Grid => taffy::style::Display::Grid,
            RTuiDisplayType::Block => taffy::style::Display::Block,
            RTuiDisplayType::Inline => taffy::style::Display::Block, // Map to block for terminal
            RTuiDisplayType::InlineBlock => taffy::style::Display::Block,
        };
        let new_builder = builder_box.display(taffy_display);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set flex direction property
#[no_mangle]
pub extern "C" fn rtui_style_builder_flex_direction(
    builder: *mut RTuiStyleBuilder,
    direction: RTuiFlexDirection,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_direction = match direction {
            RTuiFlexDirection::Row => taffy::style::FlexDirection::Row,
            RTuiFlexDirection::RowReverse => taffy::style::FlexDirection::RowReverse,
            RTuiFlexDirection::Column => taffy::style::FlexDirection::Column,
            RTuiFlexDirection::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
        };
        let new_builder = builder_box.flex_direction(taffy_direction);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set justify content property
#[no_mangle]
pub extern "C" fn rtui_style_builder_justify_content(
    builder: *mut RTuiStyleBuilder,
    justify: RTuiJustifyContent,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_justify = match justify {
            RTuiJustifyContent::FlexStart => taffy::style::JustifyContent::FlexStart,
            RTuiJustifyContent::FlexEnd => taffy::style::JustifyContent::FlexEnd,
            RTuiJustifyContent::Center => taffy::style::JustifyContent::Center,
            RTuiJustifyContent::SpaceBetween => taffy::style::JustifyContent::SpaceBetween,
            RTuiJustifyContent::SpaceAround => taffy::style::JustifyContent::SpaceAround,
            RTuiJustifyContent::SpaceEvenly => taffy::style::JustifyContent::SpaceEvenly,
        };
        let new_builder = builder_box.justify_content(taffy_justify);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set align items property
#[no_mangle]
pub extern "C" fn rtui_style_builder_align_items(
    builder: *mut RTuiStyleBuilder,
    align: RTuiAlignItems,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_align = match align {
            RTuiAlignItems::FlexStart => taffy::style::AlignItems::FlexStart,
            RTuiAlignItems::FlexEnd => taffy::style::AlignItems::FlexEnd,
            RTuiAlignItems::Center => taffy::style::AlignItems::Center,
            RTuiAlignItems::Baseline => taffy::style::AlignItems::Baseline,
            RTuiAlignItems::Stretch => taffy::style::AlignItems::Stretch,
        };
        let new_builder = builder_box.align_items(taffy_align);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set width property
#[no_mangle]
pub extern "C" fn rtui_style_builder_width(
    builder: *mut RTuiStyleBuilder,
    width: RTuiDimension,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_dimension = match width.unit {
            RTuiDimensionUnit::Pixels => taffy::style::Dimension::Length(width.value),
            RTuiDimensionUnit::Percent => taffy::style::Dimension::Percent(width.value / 100.0),
            RTuiDimensionUnit::Auto => taffy::style::Dimension::Auto,
            RTuiDimensionUnit::Flex => taffy::style::Dimension::Length(width.value), // Map to length
        };
        let new_builder = builder_box.width(taffy_dimension);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set height property
#[no_mangle]
pub extern "C" fn rtui_style_builder_height(
    builder: *mut RTuiStyleBuilder,
    height: RTuiDimension,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let taffy_dimension = match height.unit {
            RTuiDimensionUnit::Pixels => taffy::style::Dimension::Length(height.value),
            RTuiDimensionUnit::Percent => taffy::style::Dimension::Percent(height.value / 100.0),
            RTuiDimensionUnit::Auto => taffy::style::Dimension::Auto,
            RTuiDimensionUnit::Flex => taffy::style::Dimension::Length(height.value),
        };
        let new_builder = builder_box.height(taffy_dimension);
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set padding (all sides)
#[no_mangle]
pub extern "C" fn rtui_style_builder_padding(
    builder: *mut RTuiStyleBuilder,
    padding: f32,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let new_builder = builder_box.padding(taffy::geometry::Rect::all(taffy::style::LengthPercentage::Length(padding)));
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set margin (all sides)
#[no_mangle]
pub extern "C" fn rtui_style_builder_margin(
    builder: *mut RTuiStyleBuilder,
    margin: f32,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let new_builder = builder_box.margin(taffy::geometry::Rect::all(taffy::style::LengthPercentageAuto::Length(margin)));
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set background color
#[no_mangle]
pub extern "C" fn rtui_style_builder_background_color(
    builder: *mut RTuiStyleBuilder,
    color: RTuiStyleColor,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let new_builder = builder_box.background_color((color.r, color.g, color.b, color.a));
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Set text color
#[no_mangle]
pub extern "C" fn rtui_style_builder_color(
    builder: *mut RTuiStyleBuilder,
    color: RTuiStyleColor,
    out_builder: *mut *mut RTuiStyleBuilder,
) -> ReactiveError {
    if builder.is_null() || out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let new_builder = builder_box.color((color.r, color.g, color.b, color.a));
        *out_builder = Box::into_raw(Box::new(new_builder)) as *mut RTuiStyleBuilder;
        Ok(())
    }))
}

/// Build computed styles from style builder
#[no_mangle]
pub extern "C" fn rtui_style_builder_build(
    builder: *mut RTuiStyleBuilder,
    out_style: *mut *mut RTuiComputedStyle,
) -> ReactiveError {
    if builder.is_null() || out_style.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder_box = Box::from_raw(builder as *mut StyleBuilder);
        let computed_style = builder_box.build();
        *out_style = Box::into_raw(Box::new(computed_style)) as *mut RTuiComputedStyle;
        Ok(())
    }))
}

/// Destroy computed styles
#[no_mangle]
pub extern "C" fn rtui_computed_style_destroy(style: *mut RTuiComputedStyle) {
    if !style.is_null() {
        unsafe {
            let _ = Box::from_raw(style as *mut ComputedStyle);
        }
    }
}
