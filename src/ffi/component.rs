//! Component system FFI functions

use super::*;
use crate::component::{Element, ElementType, LayoutType};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to an element
#[repr(C)]
pub struct RTuiElement {
    _private: [u8; 0],
}

/// Element types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiElementType {
    Component = 0,
    Text = 1,
    Layout = 2,
    Fragment = 3,
    Empty = 4,
}

/// Layout types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiLayoutType {
    Flex = 0,
    Grid = 1,
    Stack = 2,
    Absolute = 3,
}

/// Create a component element
#[no_mangle]
pub extern "C" fn rtui_element_create_component(
    name: *const c_char,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    if name.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let element = Element::component(name_str);
        let boxed = Box::new(element);
        *out_element = Box::into_raw(boxed) as *mut RTuiElement;
        Ok(())
    }))
}

/// Create a text element
#[no_mangle]
pub extern "C" fn rtui_element_create_text(
    text: *const c_char,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    if text.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let element = Element::text(text_str);
        let boxed = Box::new(element);
        *out_element = Box::into_raw(boxed) as *mut RTuiElement;
        Ok(())
    }))
}

/// Create a layout element
#[no_mangle]
pub extern "C" fn rtui_element_create_layout(
    layout_type: RTuiLayoutType,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let layout = match layout_type {
            RTuiLayoutType::Flex => LayoutType::Flex,
            RTuiLayoutType::Grid => LayoutType::Grid,
            RTuiLayoutType::Stack => LayoutType::Stack,
            RTuiLayoutType::Absolute => LayoutType::Absolute,
        };

        let element = Element::layout(layout);
        let boxed = Box::new(element);
        unsafe {
            *out_element = Box::into_raw(boxed) as *mut RTuiElement;
        }
        Ok(())
    }))
}

/// Create a fragment element
#[no_mangle]
pub extern "C" fn rtui_element_create_fragment(
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let element = Element::fragment();
        let boxed = Box::new(element);
        unsafe {
            *out_element = Box::into_raw(boxed) as *mut RTuiElement;
        }
        Ok(())
    }))
}

/// Create an empty element
#[no_mangle]
pub extern "C" fn rtui_element_create_empty(out_element: *mut *mut RTuiElement) -> ReactiveError {
    if out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let element = Element::empty();
        let boxed = Box::new(element);
        unsafe {
            *out_element = Box::into_raw(boxed) as *mut RTuiElement;
        }
        Ok(())
    }))
}

/// Destroy an element
#[no_mangle]
pub extern "C" fn rtui_element_destroy(element: *mut RTuiElement) {
    if !element.is_null() {
        unsafe {
            let _ = Box::from_raw(element as *mut Element);
        }
    }
}

/// Set element key
#[no_mangle]
pub extern "C" fn rtui_element_set_key(
    element: *mut RTuiElement,
    key: *const c_char,
) -> ReactiveError {
    if element.is_null() || key.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &mut *(element as *mut Element);
        let key_str = CStr::from_ptr(key)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        *element_ref = element_ref.clone().with_key(key_str);
        Ok(())
    }))
}

/// Set element class
#[no_mangle]
pub extern "C" fn rtui_element_set_class(
    element: *mut RTuiElement,
    class: *const c_char,
) -> ReactiveError {
    if element.is_null() || class.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &mut *(element as *mut Element);
        let class_str = CStr::from_ptr(class)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        *element_ref = element_ref.clone().with_class(class_str);
        Ok(())
    }))
}

/// Add child element
#[no_mangle]
pub extern "C" fn rtui_element_add_child(
    parent: *mut RTuiElement,
    child: *mut RTuiElement,
) -> ReactiveError {
    if parent.is_null() || child.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let parent_ref = &mut *(parent as *mut Element);
        let child_element = Box::from_raw(child as *mut Element);

        *parent_ref = parent_ref.clone().with_child(*child_element);
        Ok(())
    }))
}

/// Get element type
#[no_mangle]
pub extern "C" fn rtui_element_get_type(
    element: *const RTuiElement,
    out_type: *mut RTuiElementType,
) -> ReactiveError {
    if element.is_null() || out_type.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        *out_type = match &element_ref.element_type {
            ElementType::Component(_) => RTuiElementType::Component,
            ElementType::Text(_) => RTuiElementType::Text,
            ElementType::Layout(_) => RTuiElementType::Layout,
            ElementType::Fragment => RTuiElementType::Fragment,
            ElementType::Empty => RTuiElementType::Empty,
        };
        Ok(())
    }))
}

/// Get element key
#[no_mangle]
pub extern "C" fn rtui_element_get_key(
    element: *const RTuiElement,
    out_key: *mut *mut c_char,
) -> ReactiveError {
    if element.is_null() || out_key.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        if let Some(ref key) = element_ref.key {
            let c_string = CString::new(key.clone()).map_err(|_| ReactiveError::InvalidUtf8)?;
            *out_key = c_string.into_raw();
        } else {
            *out_key = std::ptr::null_mut();
        }
        Ok(())
    }))
}

/// Get element class
#[no_mangle]
pub extern "C" fn rtui_element_get_class(
    element: *const RTuiElement,
    out_class: *mut *mut c_char,
) -> ReactiveError {
    if element.is_null() || out_class.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        if let Some(ref class) = element_ref.class {
            let c_string = CString::new(class.clone()).map_err(|_| ReactiveError::InvalidUtf8)?;
            *out_class = c_string.into_raw();
        } else {
            *out_class = std::ptr::null_mut();
        }
        Ok(())
    }))
}

/// Get number of children
#[no_mangle]
pub extern "C" fn rtui_element_get_child_count(
    element: *const RTuiElement,
    out_count: *mut usize,
) -> ReactiveError {
    if element.is_null() || out_count.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);
        *out_count = element_ref.children.len();
        Ok(())
    }))
}

/// Get child element at index
#[no_mangle]
pub extern "C" fn rtui_element_get_child(
    element: *const RTuiElement,
    index: usize,
    out_child: *mut *mut RTuiElement,
) -> ReactiveError {
    if element.is_null() || out_child.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        if index >= element_ref.children.len() {
            return Err(ReactiveError::InvalidParameter);
        }

        let child = element_ref.children[index].clone();
        let boxed = Box::new(child);
        *out_child = Box::into_raw(boxed) as *mut RTuiElement;
        Ok(())
    }))
}

/// Get component name (if element is a component)
#[no_mangle]
pub extern "C" fn rtui_element_get_component_name(
    element: *const RTuiElement,
    out_name: *mut *mut c_char,
) -> ReactiveError {
    if element.is_null() || out_name.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        if let ElementType::Component(name) = &element_ref.element_type {
            let c_string = CString::new(name.clone()).map_err(|_| ReactiveError::InvalidUtf8)?;
            *out_name = c_string.into_raw();
        } else {
            *out_name = std::ptr::null_mut();
        }
        Ok(())
    }))
}

/// Get text content (if element is text)
#[no_mangle]
pub extern "C" fn rtui_element_get_text_content(
    element: *const RTuiElement,
    out_text: *mut *mut c_char,
) -> ReactiveError {
    if element.is_null() || out_text.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let element_ref = &*(element as *const Element);

        if let ElementType::Text(text) = &element_ref.element_type {
            let c_string = CString::new(text.clone()).map_err(|_| ReactiveError::InvalidUtf8)?;
            *out_text = c_string.into_raw();
        } else {
            *out_text = std::ptr::null_mut();
        }
        Ok(())
    }))
}
