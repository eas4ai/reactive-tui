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
            let result = rtui_element_builder_add_class(builder_ptr, c"test-class".as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test adding more classes
            let result = rtui_element_builder_add_class(builder_ptr, c"another-class".as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test setting text
            let result = rtui_element_builder_set_text(builder_ptr, c"Hello, World!".as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged!

            // Test setting key
            let result = rtui_element_builder_set_key(builder_ptr, c"my-element".as_ptr());
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

            let result = rtui_element_builder_add_class(builder_ptr, c"container".as_ptr());
            assert_eq!(result, ReactiveError::Success);

            let result = rtui_element_builder_set_text(builder_ptr, c"Test content".as_ptr());
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

            let result = rtui_element_builder_add_class(parent_builder, c"parent".as_ptr());
            assert_eq!(result, ReactiveError::Success);

            // Create child element using text widget
            let mut child_element: *mut RTuiElement = ptr::null_mut();
            let result = rtui_text_element_create(c"Child text".as_ptr(), &mut child_element);
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
                c"Enter text".as_ptr(),
                c"Initial".as_ptr(),
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test checkbox
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_checkbox_create(c"Check me".as_ptr(), true, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test button
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_button_create(c"Click me".as_ptr(), &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());
            rtui_element_destroy(element_ptr);

            // Test progress bar
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_progress_bar_create(
                0.0,
                100.0,
                75.0,
                c"Loading...".as_ptr(),
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
            let result = rtui_element_builder_add_class(ptr::null_mut(), c"test".as_ptr());
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
