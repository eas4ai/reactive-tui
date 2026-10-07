//! Widget library FFI constructors.

use super::*;
use crate::builder::core::ElementBuilder;
use crate::component::{Element, ElementType, LayoutType};
use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Return a native TextInput with its placeholder and initial value.
/// This constructor takes no callbacks.
///
/// # Safety
///
/// `placeholder` must be null or a NUL-terminated string that stays readable
/// during the call. `initial_value` must be null or a NUL-terminated string
/// that stays readable during the call. `out_element` must be null or a
/// pointer slot the caller owns, which this call may write.
#[no_mangle]
pub unsafe extern "C" fn rtui_text_input_create(
    placeholder: *const c_char,
    initial_value: *const c_char,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut builder = crate::builder::text_input();

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
            builder = builder.value(value_str);
        }

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Return a native Checkbox with its label and initial checked state.
/// This constructor takes no callbacks.
///
/// # Safety
///
/// `label` must be null or a NUL-terminated string that stays readable during
/// the call. `out_element` must be null or a pointer slot the caller owns,
/// which this call may write.
#[no_mangle]
pub unsafe extern "C" fn rtui_checkbox_create(
    label: *const c_char,
    initial_checked: bool,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut builder = crate::builder::checkbox().checked(initial_checked);
        if !label.is_null() {
            let label_str = CStr::from_ptr(label)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            builder = builder.label(label_str);
        }
        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Return a native ProgressBar with its range, current value and label.
/// This constructor takes no callbacks.
///
/// # Safety
///
/// `label` must be null or a NUL-terminated string that stays readable during
/// the call. `out_element` must be null or a pointer slot the caller owns,
/// which this call may write.
#[no_mangle]
pub unsafe extern "C" fn rtui_progress_bar_create(
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
        let mut builder = crate::widgets::display::progress_bar::ProgressBarBuilder::new()
            .range(min_value, max_value)
            .value(current_value);
        if !label.is_null() {
            let label_str = CStr::from_ptr(label)
                .to_str()
                .map_err(|_| ReactiveError::InvalidUtf8)?;
            builder = builder.label(label_str);
        }
        let element = builder.render();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Return a styled text element for a button.
/// This constructor takes no callbacks.
///
/// # Safety
///
/// `text` must be null or a NUL-terminated string that stays readable during
/// the call. `out_element` must be null or a pointer slot the caller owns,
/// which this call may write.
#[no_mangle]
pub unsafe extern "C" fn rtui_button_create(
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
            "button px-16 py-8 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer",
        );
        builder = builder.text(text_str);

        let element = builder.build();
        let ffi_element = super::builder::FFIElement { inner: element };
        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Create a simple text element
///
/// # Safety
///
/// `text` must be null or a NUL-terminated string that stays readable during
/// the call. `out_element` must be null or a pointer slot the caller owns,
/// which this call may write.
#[no_mangle]
pub unsafe extern "C" fn rtui_text_element_create(
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
