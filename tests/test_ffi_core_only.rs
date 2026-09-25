//! Test only the core FFI builder functionality

#[cfg(feature = "ffi")]
mod ffi_core_tests {
    use reactive_tui::ffi::*;
    use std::ptr;

    #[test]
    fn test_ffi_builder_creation() {
        unsafe {
            // Test div builder creation
            let mut builder_ptr: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!builder_ptr.is_null());

            // Clean up
            rtui_element_builder_destroy(builder_ptr);
        }
    }

    #[test]
    fn test_ffi_builder_in_place_modification() {
        unsafe {
            // Create builder
            let mut builder_ptr: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);

            let original_ptr = builder_ptr;

            // Test in-place class addition - pointer should remain the same
            let result =
                rtui_element_builder_add_class(builder_ptr, b"test-class\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test adding more classes
            let result = rtui_element_builder_add_class(
                builder_ptr,
                b"another-class\0".as_ptr() as *const i8,
            );
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test setting text
            let result = rtui_element_builder_set_text(
                builder_ptr,
                b"Hello, World!\0".as_ptr() as *const i8,
            );
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test setting key
            let result =
                rtui_element_builder_set_key(builder_ptr, b"my-element\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Clean up
            rtui_element_builder_destroy(builder_ptr);
        }
    }

    #[test]
    fn test_ffi_builder_build() {
        unsafe {
            // Create and configure builder
            let mut builder_ptr: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);

            let result =
                rtui_element_builder_add_class(builder_ptr, b"container\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);

            let result =
                rtui_element_builder_set_text(builder_ptr, b"Test content\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);

            // Build the element
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_element_builder_build(builder_ptr, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up element (builder was consumed by build)
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_ffi_builder_with_children() {
        unsafe {
            // Create parent builder
            let mut parent_builder: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut parent_builder);
            assert_eq!(result, ReactiveError::Success);

            let result =
                rtui_element_builder_add_class(parent_builder, b"parent\0".as_ptr() as *const i8);
            assert_eq!(result, ReactiveError::Success);

            // Create child element using text widget
            let mut child_element: *mut RTuiElement = ptr::null_mut();
            let result =
                rtui_text_element_create(b"Child text\0".as_ptr() as *const i8, &mut child_element);
            assert_eq!(result, ReactiveError::Success);

            // Add child to parent
            let result = rtui_element_builder_add_child(parent_builder, child_element);
            assert_eq!(result, ReactiveError::Success);

            // Build parent element
            let mut parent_element: *mut RTuiElement = ptr::null_mut();
            let result = rtui_element_builder_build(parent_builder, &mut parent_element);
            assert_eq!(result, ReactiveError::Success);
            assert!(!parent_element.is_null());

            // Clean up
            rtui_element_destroy(parent_element);
        }
    }

    #[test]
    fn test_ffi_widget_creation() {
        unsafe {
            // Test text input
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_text_input_create(
                b"Enter text\0".as_ptr() as *const i8,
                b"Initial\0".as_ptr() as *const i8,
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test checkbox
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result =
                rtui_checkbox_create(b"Check me\0".as_ptr() as *const i8, true, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test button
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_button_create(b"Click me\0".as_ptr() as *const i8, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test progress bar
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_progress_bar_create(
                0.0,
                100.0,
                75.0,
                b"Loading...\0".as_ptr() as *const i8,
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_ffi_null_pointer_safety() {
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
    fn test_ffi_memory_management() {
        unsafe {
            // Create and destroy many builders to test memory management
            for i in 0..50 {
                let mut builder: *mut RTuiElementBuilder = ptr::null_mut();
                let result = rtui_element_builder_div(&mut builder);
                assert_eq!(result, ReactiveError::Success);

                let class_name = format!("test-class-{}\0", i);
                let result =
                    rtui_element_builder_add_class(builder, class_name.as_ptr() as *const i8);
                assert_eq!(result, ReactiveError::Success);

                let mut element: *mut RTuiElement = ptr::null_mut();
                let result = rtui_element_builder_build(builder, &mut element);
                assert_eq!(result, ReactiveError::Success);

                rtui_element_destroy(element);
            }
        }
    }
}
