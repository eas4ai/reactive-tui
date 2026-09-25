//! Widget library FFI functions - PROPER IMPLEMENTATION

use super::*;
use crate::builder::core::ElementBuilder;
use crate::component::{Element, ElementType, LayoutType};
use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Simple widget creation functions without callbacks
/// Callback support requires a registry system for function pointer management

/// Create a simple text input element (without callbacks)
#[no_mangle]
pub extern "C" fn rtui_text_input_create(
    placeholder: *const c_char,
    initial_value: *const c_char,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut builder = ElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        builder = builder.class("input border px-2 py-1 bg-white");

        if !placeholder.is_null() {
            let placeholder_str = CStr::from_ptr(placeholder)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            builder = builder.placeholder(placeholder_str);
        }

        if !initial_value.is_null() {
            let value_str = CStr::from_ptr(initial_value)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            builder = builder.text(value_str);
        }

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Create a simple checkbox element (without callbacks)
#[no_mangle]
pub extern "C" fn rtui_checkbox_create(
    label: *const c_char,
    initial_checked: bool,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut builder = ElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        builder = builder.class("checkbox flex items-center");

        let checkbox_text = if !label.is_null() {
            let label_str = CStr::from_ptr(label)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            format!(
                "[{}] {}",
                if initial_checked { "x" } else { " " },
                label_str
            )
        } else {
            format!("[{}]", if initial_checked { "x" } else { " " })
        };

        builder = builder.text(&checkbox_text);

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Create a simple progress bar element
#[no_mangle]
pub extern "C" fn rtui_progress_bar_create(
    min_value: f64,
    max_value: f64,
    current_value: f64,
    label: *const c_char,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut builder = ElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        builder = builder.class("progress-bar border rounded");

        let percentage = if max_value > min_value {
            ((current_value - min_value) / (max_value - min_value) * 100.0)
                .min(100.0)
                .max(0.0)
        } else {
            0.0
        };

        let progress_text = if !label.is_null() {
            let label_str = CStr::from_ptr(label)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            format!("{}: {:.1}%", label_str, percentage)
        } else {
            format!("{:.1}%", percentage)
        };

        builder = builder.text(&progress_text);

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Create a simple button element (without callbacks)
#[no_mangle]
pub extern "C" fn rtui_button_create(
    text: *const c_char,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if text.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let mut builder = ElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        builder = builder.class(
            "button px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer",
        );
        builder = builder.text(text_str);

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Create a simple text element
#[no_mangle]
pub extern "C" fn rtui_text_element_create(
    text: *const c_char,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if text.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let element = Element::text(text_str);
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}
