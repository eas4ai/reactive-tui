//! Integration test for the PROPER FFI implementation

#[cfg(feature = "ffi")]
mod ffi_integration_tests {
    use reactive_tui::ffi::*;
    use std::ptr;

    #[test]
    fn test_proper_builder_pattern() {
        unsafe {
            // Create a div builder
            let mut builder_ptr: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!builder_ptr.is_null());

            let original_ptr = builder_ptr;

            // Test REAL in-place modification - pointer should remain the same
            let result =
                rtui_element_builder_add_class(builder_ptr, b"test-class\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Add more classes
            let result = rtui_element_builder_add_class(
                builder_ptr,
                b"another-class\0".as_ptr() as *const i8,
            );
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Set text
            let result = rtui_element_builder_set_text(
                builder_ptr,
                b"Hello, World!\0".as_ptr() as *const i8,
            );
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Set key
            let result =
                rtui_element_builder_set_key(builder_ptr, b"my-element\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Build the element (this consumes the builder)
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_element_builder_build(builder_ptr, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up element (builder was consumed by build)
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_widget_creation() {
        unsafe {
            // Test text input creation
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_text_input_create(
                b"Enter text\0".as_ptr() as *const i8,
                b"Initial value\0".as_ptr() as *const i8,
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test checkbox creation
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_checkbox_create(
                b"Test checkbox\0".as_ptr() as *const i8,
                true,
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test button creation
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_button_create(b"Click me\0".as_ptr() as *const i8, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test progress bar creation
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_progress_bar_create(
                0.0,
                100.0,
                50.0,
                b"Loading...\0".as_ptr() as *const i8,
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test text element creation
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result =
                rtui_text_element_create(b"Simple text\0".as_ptr() as *const i8, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_builder_with_children() {
        unsafe {
            // Create a parent div
            let mut parent_builder: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut parent_builder);
            assert_eq!(result, ReactiveError::Success);

            let result = rtui_element_builder_add_class(
                parent_builder,
                b"container\0".as_ptr() as *const i8,
            );
            assert_eq!(result, ReactiveError::Success);

            // Create a child element
            let mut child_element: *mut RTuiElement = ptr::null_mut();
            let result =
                rtui_text_element_create(b"Child text\0".as_ptr() as *const i8, &mut child_element);
            assert_eq!(result, ReactiveError::Success);

            // Add child to parent
            let result = rtui_element_builder_add_child(parent_builder, child_element);
            assert_eq!(result, ReactiveError::Success);

            // Build the parent element
            let mut parent_element: *mut RTuiElement = ptr::null_mut();
            let result = rtui_element_builder_build(parent_builder, &mut parent_element);
            assert_eq!(result, ReactiveError::Success);
            assert!(!parent_element.is_null());

            // Clean up
            rtui_element_destroy(parent_element);
        }
    }

    #[test]
    fn test_null_pointer_safety() {
        unsafe {
            // Test null out_builder
            let result = rtui_element_builder_div(ptr::null_mut());
            assert_eq!(result, ReactiveError::NullPointer);

            // Test null builder
            let result =
                rtui_element_builder_add_class(ptr::null_mut(), b"test\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::NullPointer);

            // Test null classes
            let mut builder: *mut RTuiElementBuilder = ptr::null_mut();
            rtui_element_builder_div(&mut builder);
            let result = rtui_element_builder_add_class(builder, ptr::null());
            assert_eq!(result, ReactiveError::NullPointer);

            rtui_element_builder_destroy(builder);
        }
    }

    #[test]
    fn test_memory_management() {
        unsafe {
            // Create and destroy multiple builders to test memory management
            for _ in 0..100 {
                let mut builder: *mut RTuiElementBuilder = ptr::null_mut();
                let result = rtui_element_builder_div(&mut builder);
                assert_eq!(result, ReactiveError::Success);

                let result =
                    rtui_element_builder_add_class(builder, b"test\0".as_ptr() as *const i8);
                assert_eq!(result, ReactiveError::Success);

                let mut element: *mut RTuiElement = ptr::null_mut();
                let result = rtui_element_builder_build(builder, &mut element);
                assert_eq!(result, ReactiveError::Success);

                rtui_element_destroy(element);
            }
        }
    }
}
