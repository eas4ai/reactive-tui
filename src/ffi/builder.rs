//! Element builder FFI functions - PROPER IMPLEMENTATION

use super::*;
use crate::builder::core::ElementBuilder;
use crate::component::{Element, ElementType, LayoutType};
use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::c_char;

/// FFI wrapper for ElementBuilder that supports in-place modification
pub struct FFIElementBuilder {
    element_type: ElementType,
    classes: String,
    text_content: Option<String>,
    children: Vec<Element>,
    key: Option<String>,
}

impl FFIElementBuilder {
    /// Create a new element builder with the specified type
    pub fn new(element_type: ElementType) -> Self {
        Self {
            element_type,
            classes: String::new(),
            text_content: None,
            children: Vec::new(),
            key: None,
        }
    }

    /// Build the final element
    pub fn build(self) -> Element {
        let mut builder = ElementBuilder::new(self.element_type);

        if !self.classes.is_empty() {
            builder = builder.class(&self.classes);
        }

        if let Some(text) = self.text_content {
            builder = builder.text(&text);
        }

        if let Some(key) = self.key {
            builder = builder.key(&key);
        }

        for child in self.children {
            builder = builder.child(child);
        }

        builder.build()
    }
}

/// FFI wrapper for Element
pub struct FFIElement {
    /// The wrapped element
    pub inner: Element,
}

/// Opaque handle to an element builder
#[repr(C)]
pub struct RTuiElementBuilder {
    _private: [u8; 0],
}

/// Opaque handle to an element
#[repr(C)]
pub struct RTuiElement {
    _private: [u8; 0],
}

// =============================================================================
// BUILDER CREATION FUNCTIONS
// =============================================================================

/// Create a div element builder
#[no_mangle]
pub extern "C" fn rtui_element_builder_div(
    out_builder: *mut *mut RTuiElementBuilder,
) -> ReactiveError {
    if out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let ffi_builder = FFIElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        *out_builder = Box::into_raw(Box::new(ffi_builder)) as *mut RTuiElementBuilder;
        Ok(())
    }))
}

/// Create a span element builder
#[no_mangle]
pub extern "C" fn rtui_element_builder_span(
    out_builder: *mut *mut RTuiElementBuilder,
) -> ReactiveError {
    if out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut ffi_builder = FFIElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        ffi_builder.classes = "inline".to_string();
        *out_builder = Box::into_raw(Box::new(ffi_builder)) as *mut RTuiElementBuilder;
        Ok(())
    }))
}

/// Create a button element builder
#[no_mangle]
pub extern "C" fn rtui_element_builder_button(
    out_builder: *mut *mut RTuiElementBuilder,
) -> ReactiveError {
    if out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let mut ffi_builder = FFIElementBuilder::new(ElementType::Layout(LayoutType::Flex));
        ffi_builder.classes =
            "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer".to_string();
        *out_builder = Box::into_raw(Box::new(ffi_builder)) as *mut RTuiElementBuilder;
        Ok(())
    }))
}

// =============================================================================
// BUILDER MODIFICATION FUNCTIONS - PROPER IN-PLACE MODIFICATION
// =============================================================================

/// Add CSS classes to element builder (REAL in-place modification)
#[no_mangle]
pub extern "C" fn rtui_element_builder_add_class(
    builder: *mut RTuiElementBuilder,
    classes: *const c_char,
) -> ReactiveError {
    if builder.is_null() || classes.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let classes_str = CStr::from_ptr(classes)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        // Get mutable reference to FFI builder - NO Box::from_raw!
        let ffi_builder = &mut *(builder as *mut FFIElementBuilder);

        // Actually modify in place
        if !ffi_builder.classes.is_empty() {
            ffi_builder.classes.push(' ');
        }
        ffi_builder.classes.push_str(classes_str);

        Ok(())
    }))
}

/// Set text content for element builder (REAL in-place modification)
#[no_mangle]
pub extern "C" fn rtui_element_builder_set_text(
    builder: *mut RTuiElementBuilder,
    text: *const c_char,
) -> ReactiveError {
    if builder.is_null() || text.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        // Get mutable reference to FFI builder - NO Box::from_raw!
        let ffi_builder = &mut *(builder as *mut FFIElementBuilder);

        // Actually modify in place
        ffi_builder.text_content = Some(text_str.to_string());

        Ok(())
    }))
}

/// Set key for element builder (REAL in-place modification)
#[no_mangle]
pub extern "C" fn rtui_element_builder_set_key(
    builder: *mut RTuiElementBuilder,
    key: *const c_char,
) -> ReactiveError {
    if builder.is_null() || key.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let key_str = CStr::from_ptr(key)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        // Get mutable reference to FFI builder - NO Box::from_raw!
        let ffi_builder = &mut *(builder as *mut FFIElementBuilder);

        // Actually modify in place
        ffi_builder.key = Some(key_str.to_string());

        Ok(())
    }))
}

/// Add a child element to builder (REAL in-place modification)
#[no_mangle]
pub extern "C" fn rtui_element_builder_add_child(
    builder: *mut RTuiElementBuilder,
    child: *mut RTuiElement,
) -> ReactiveError {
    if builder.is_null() || child.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Take ownership of the child element
        let ffi_element = Box::from_raw(child as *mut FFIElement);

        // Get mutable reference to FFI builder - NO Box::from_raw!
        let ffi_builder = &mut *(builder as *mut FFIElementBuilder);

        // Actually modify in place
        ffi_builder.children.push(ffi_element.inner);

        Ok(())
    }))
}

/// Build the final element (consumes the builder)
#[no_mangle]
pub extern "C" fn rtui_element_builder_build(
    builder: *mut RTuiElementBuilder,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    if builder.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Take ownership of the builder (build consumes it)
        let ffi_builder = Box::from_raw(builder as *mut FFIElementBuilder);

        // Build the element
        let element = ffi_builder.build();
        let ffi_element = FFIElement { inner: element };

        *out_element = Box::into_raw(Box::new(ffi_element)) as *mut RTuiElement;
        Ok(())
    }))
}

/// Destroy an element builder
#[no_mangle]
pub extern "C" fn rtui_element_builder_destroy(builder: *mut RTuiElementBuilder) {
    if !builder.is_null() {
        unsafe {
            let _ = Box::from_raw(builder as *mut FFIElementBuilder);
        }
    }
}

/// Destroy an element
#[no_mangle]
pub extern "C" fn rtui_element_destroy(element: *mut RTuiElement) {
    if !element.is_null() {
        unsafe {
            let _ = Box::from_raw(element as *mut FFIElement);
        }
    }
}
