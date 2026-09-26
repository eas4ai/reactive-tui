//! Basic FFI functionality tests
//!
//! Tests the core FFI functions to ensure they work correctly after fixes

#[cfg(feature = "ffi")]
mod ffi_tests {
    use reactive_tui::ffi::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_app_builder_creation_and_modification() {
        {
            // Test app builder creation
            let mut builder_ptr: *mut RTuiAppBuilder = ptr::null_mut();
            let result = rtui_app_builder_create(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!builder_ptr.is_null());

            // Test in-place debug modification (should not change pointer)
            let original_ptr = builder_ptr;
            let result = rtui_app_builder_debug(builder_ptr, true);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Test in-place performance mode modification
            let result =
                rtui_app_builder_performance_mode(builder_ptr, RTuiPerformanceMode::Performance);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Test backend setting
            let result = rtui_app_builder_backend_debug(builder_ptr, 80, 24);
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Clean up
            rtui_app_builder_destroy(builder_ptr);
        }
    }

    #[test]
    fn test_element_builder_creation_and_modification() {
        {
            // Test element builder creation
            let mut builder_ptr: *mut RTuiElementBuilder = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!builder_ptr.is_null());

            // Test in-place class modification (should not change pointer)
            let original_ptr = builder_ptr;
            let class_str = CString::new("test-class").unwrap();
            let result = rtui_element_builder_add_class(builder_ptr, class_str.as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Test in-place text modification
            let text_str = CString::new("Hello, World!").unwrap();
            let result = rtui_element_builder_set_text(builder_ptr, text_str.as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Test in-place key modification
            let key_str = CString::new("test-key").unwrap();
            let result = rtui_element_builder_set_key(builder_ptr, key_str.as_ptr());
            assert_eq!(result, ReactiveError::Success);
            assert_eq!(builder_ptr, original_ptr); // Pointer should remain the same

            // Build the element
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();
            let result = rtui_element_builder_build(builder_ptr, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_widget_creation() {
        {
            // Test text input creation
            let placeholder = CString::new("Enter text").unwrap();
            let initial_value = CString::new("").unwrap();
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();

            let result = rtui_text_input_create(
                placeholder.as_ptr(),
                initial_value.as_ptr(),
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_checkbox_creation() {
        {
            let label = CString::new("Test Checkbox").unwrap();
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();

            let result = rtui_checkbox_create(label.as_ptr(), false, &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_progress_bar_creation() {
        {
            let label = CString::new("Loading...").unwrap();
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();

            let result = rtui_progress_bar_create(
                0.0,   // min_value
                100.0, // max_value
                50.0,  // current_value
                label.as_ptr(),
                &mut element_ptr,
            );
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_button_creation() {
        {
            let text = CString::new("Click Me").unwrap();
            let mut element_ptr: *mut RTuiElement = ptr::null_mut();

            let result = rtui_button_create(text.as_ptr(), &mut element_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!element_ptr.is_null());

            // Clean up
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_root_component_functionality() {
        {
            // Test root component callback system
            extern "C" fn test_root_callback(
                _user_data: *mut std::ffi::c_void,
            ) -> *mut RTuiElement {
                // Create a simple element to return
                let mut element_ptr: *mut RTuiElement = ptr::null_mut();
                let result = rtui_text_element_create(
                    CString::new("Root Component Test").unwrap().as_ptr(),
                    &mut element_ptr,
                );
                if result == ReactiveError::Success {
                    element_ptr
                } else {
                    ptr::null_mut()
                }
            }

            // Test app builder with root component
            let mut builder_ptr: *mut RTuiAppBuilder = ptr::null_mut();
            let result = rtui_app_builder_create(&mut builder_ptr);
            assert_eq!(result, ReactiveError::Success);

            // Set root component
            let result = rtui_app_builder_root_component(
                builder_ptr,
                Some(test_root_callback),
                ptr::null_mut(),
            );
            assert_eq!(result, ReactiveError::Success);

            // Set backend
            let result = rtui_app_builder_backend_debug(builder_ptr, 80, 24);
            assert_eq!(result, ReactiveError::Success);

            // Build app
            let mut app_ptr: *mut RTuiApp = ptr::null_mut();
            let result = rtui_app_builder_build(builder_ptr, &mut app_ptr);
            assert_eq!(result, ReactiveError::Success);
            assert!(!app_ptr.is_null());
            let mut dimensions = RTuiDimensions {
                width: 0,
                height: 0,
            };
            assert_eq!(
                rtui_app_get_size(app_ptr, &mut dimensions),
                ReactiveError::Success
            );
            assert_eq!((dimensions.width, dimensions.height), (80, 24));

            // Note: We don't call run() as it would block and consume the app
            // Clean up
            rtui_app_destroy(app_ptr);
        }
    }

    #[test]
    fn test_null_pointer_safety() {
        {
            // Test that null pointers are handled safely
            let result = rtui_app_builder_debug(ptr::null_mut(), true);
            assert_eq!(result, ReactiveError::NullPointer);

            let result = rtui_element_builder_add_class(ptr::null_mut(), ptr::null());
            assert_eq!(result, ReactiveError::NullPointer);

            let result = rtui_text_input_create(ptr::null(), ptr::null(), ptr::null_mut());
            assert_eq!(result, ReactiveError::NullPointer);
        }
    }
}
