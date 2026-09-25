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

/// FFI wrapper for Element. Component and builder APIs share the same handle layout.
#[repr(transparent)]
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
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_builder_destroy(builder: *mut RTuiElementBuilder) {
    if !builder.is_null() {
        unsafe {
            let _ = Box::from_raw(builder as *mut FFIElementBuilder);
        }
    }
}

/// Destroy an element
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_destroy(element: *mut RTuiElement) {
    if !element.is_null() {
        unsafe {
            let _ = Box::from_raw(element as *mut FFIElement);
        }
    }
}

// Compatibility names from the original C builder header. These delegate to
// the existing implementation and retain its consuming ownership rules.
/// Create a legacy div builder; the caller owns the returned builder.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_div(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    rtui_element_builder_div(out)
}

/// Create a legacy inline span builder; the caller owns the returned builder.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_span(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    rtui_element_builder_span(out)
}

/// Create a legacy styled button builder without a click callback.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_button(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    rtui_element_builder_button(out)
}

// The paragraph and heading factories produce plain layout containers. Preserve
// their native type and classes instead of copying their style defaults here.
fn text_container_builder(
    factory: fn() -> ElementBuilder,
    out: *mut *mut RTuiElementBuilder,
) -> ReactiveError {
    if out.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        *out = std::ptr::null_mut();
        let element = factory().build();
        let builder = FFIElementBuilder {
            element_type: element.element_type,
            classes: element.class.unwrap_or_default(),
            text_content: None,
            children: element.children,
            key: element.key,
        };
        *out = Box::into_raw(Box::new(builder)).cast();
        Ok(())
    }))
}

/// Create a paragraph builder using the native paragraph classes.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_p(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    text_container_builder(crate::builder::p, out)
}

/// Create a level-one heading builder using the native heading classes.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_h1(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    text_container_builder(crate::builder::h1, out)
}

/// Create a level-two heading builder using the native heading classes.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_h2(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    text_container_builder(crate::builder::h2, out)
}

/// Create a level-three heading builder using the native heading classes.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_h3(out: *mut *mut RTuiElementBuilder) -> ReactiveError {
    text_container_builder(crate::builder::h3, out)
}

/// Append classes through the legacy non-consuming builder name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_builder_class(
    builder: *mut RTuiElementBuilder,
    classes: *const c_char,
) -> ReactiveError {
    rtui_element_builder_add_class(builder, classes)
}

/// Set text through the legacy non-consuming builder name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_builder_text(
    builder: *mut RTuiElementBuilder,
    text: *const c_char,
) -> ReactiveError {
    rtui_element_builder_set_text(builder, text)
}

/// Set the key through the legacy non-consuming builder name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_builder_key(
    builder: *mut RTuiElementBuilder,
    key: *const c_char,
) -> ReactiveError {
    rtui_element_builder_set_key(builder, key)
}

/// Append and consume a live child through the legacy builder name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_builder_child(
    builder: *mut RTuiElementBuilder,
    child: *mut RTuiElement,
) -> ReactiveError {
    rtui_element_builder_add_child(builder, child)
}

// All handles must be live, distinct and caller-owned. Validate the whole array
// before consuming anything; a null array is allowed only for zero children.
unsafe fn take_children(
    children: *const *mut RTuiElement,
    count: usize,
) -> Result<Vec<Element>, ReactiveError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if children.is_null() {
        return Err(ReactiveError::NullPointer);
    }
    if count > isize::MAX as usize / std::mem::size_of::<*mut RTuiElement>() {
        return Err(ReactiveError::InvalidParameter);
    }
    let handles = unsafe { std::slice::from_raw_parts(children, count) };
    let mut seen = std::collections::HashSet::new();
    for &handle in handles {
        if handle.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        if !seen.insert(handle) {
            return Err(ReactiveError::InvalidParameter);
        }
    }
    Ok(handles
        .iter()
        .map(|&handle| unsafe { Box::from_raw(handle.cast::<FFIElement>()).inner })
        .collect())
}

/// Append children, consuming every child after successful argument validation.
#[no_mangle]
pub extern "C" fn rtui_element_builder_children(
    builder: *mut RTuiElementBuilder,
    children: *const *mut RTuiElement,
    children_count: usize,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        let children = take_children(children, children_count)?;
        (*builder.cast::<FFIElementBuilder>())
            .children
            .extend(children);
        Ok(())
    }))
}

/// Create a caller-owned text element through the legacy name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_text(
    text: *const c_char,
    out: *mut *mut RTuiElement,
) -> ReactiveError {
    rtui_text_element_create(text, out)
}

/// Create a caller-owned empty element through the legacy name.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_element_empty(out: *mut *mut RTuiElement) -> ReactiveError {
    rtui_element_create_empty(out)
}

/// Create a native card, consuming its children after argument validation.
#[no_mangle]
pub extern "C" fn rtui_card(
    children: *const *mut RTuiElement,
    children_count: usize,
    out: *mut *mut RTuiElement,
) -> ReactiveError {
    if out.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        *out = std::ptr::null_mut();
        let children = take_children(children, children_count)?;
        let element = FFIElement {
            inner: crate::builder::card(children),
        };
        *out = Box::into_raw(Box::new(element)).cast();
        Ok(())
    }))
}

/// Free a string allocated by native string getters; null is allowed.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_free_string(string: *mut c_char) {
    rtui_string_free(string)
}
