//! Minimal test for the PROPER FFI builder implementation

#[cfg(feature = "ffi")]
mod minimal_ffi_tests {
    use std::ptr;

    // Ensure Cargo links the library that provides these C symbols.
    extern crate reactive_tui;
    // Import only the specific functions I implemented
    extern "C" {
        fn rtui_element_builder_div(out_builder: *mut *mut std::ffi::c_void) -> i32;
        fn rtui_element_builder_add_class(
            builder: *mut std::ffi::c_void,
            classes: *const i8,
        ) -> i32;
        fn rtui_element_builder_set_text(builder: *mut std::ffi::c_void, text: *const i8) -> i32;
        fn rtui_element_builder_build(
            builder: *mut std::ffi::c_void,
            out_element: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn rtui_element_builder_destroy(builder: *mut std::ffi::c_void);
        fn rtui_element_destroy(element: *mut std::ffi::c_void);
    }

    #[test]
    fn test_minimal_builder_functionality() {
        unsafe {
            // Test builder creation
            let mut builder_ptr: *mut std::ffi::c_void = ptr::null_mut();
            let result = rtui_element_builder_div(&mut builder_ptr);
            assert_eq!(result, 0); // Success
            assert!(!builder_ptr.is_null());

            let original_ptr = builder_ptr;

            // Test in-place modification - pointer should remain the same
            let result =
                rtui_element_builder_add_class(builder_ptr, b"test-class\0".as_ptr() as *const i8);
            assert_eq!(result, 0); // Success
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Test setting text
            let result =
                rtui_element_builder_set_text(builder_ptr, b"Hello\0".as_ptr() as *const i8);
            assert_eq!(result, 0); // Success
            assert_eq!(builder_ptr, original_ptr); // Pointer unchanged

            // Test building element
            let mut element_ptr: *mut std::ffi::c_void = ptr::null_mut();
            let result = rtui_element_builder_build(builder_ptr, &mut element_ptr);
            assert_eq!(result, 0); // Success
            assert!(!element_ptr.is_null());

            // Clean up element (builder was consumed by build)
            rtui_element_destroy(element_ptr);
        }
    }

    #[test]
    fn test_null_pointer_handling() {
        unsafe {
            // Test null out_builder
            let result = rtui_element_builder_div(ptr::null_mut());
            assert_eq!(result, -2); // NullPointer error

            // Test null builder
            let result =
                rtui_element_builder_add_class(ptr::null_mut(), b"test\0".as_ptr() as *const i8);
            assert_eq!(result, -2); // NullPointer error

            // Test null classes
            let mut builder: *mut std::ffi::c_void = ptr::null_mut();
            rtui_element_builder_div(&mut builder);
            let result = rtui_element_builder_add_class(builder, ptr::null());
            assert_eq!(result, -2); // NullPointer error

            rtui_element_builder_destroy(builder);
        }
    }
}
