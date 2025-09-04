//! Integration tests for FFI systems
//!
//! Tests the interaction between different FFI modules to ensure they work together correctly.

use super::super::*;
use crate::core::surface::{Rgba, Attr};
use std::ptr;

/// Test text buffer rendering to surface integration
#[test]
fn test_text_buffer_to_surface_integration() {
    // Skip if no TTY available (CI/test environment)
    let buffer = unsafe { createBuffer(80, 24) };
    if buffer.is_null() {
        eprintln!("Skipping test_text_buffer_to_surface_integration: Failed to create buffer");
        return;
    }

    let text_buffer = unsafe { createTextBuffer(100, 0) };
    if text_buffer.is_null() {
        unsafe { destroyBuffer(buffer) };
        eprintln!("Skipping test_text_buffer_to_surface_integration: Failed to create text buffer");
        return;
    }

    // Add some text to the buffer
    let fg = [1.0, 1.0, 1.0, 1.0f32]; // White
    let bg = [0.0, 0.0, 0.0, 1.0f32]; // Black
    let attr = 0u32;

    let text = std::ffi::CString::new("Hello, World!").unwrap();
    let chars_written = unsafe {
        textBufferAppendText(
            text_buffer,
            text.as_ptr(),
            text.as_bytes().len(),
            fg.as_ptr(),
            bg.as_ptr(),
            &attr,
        )
    };

    assert!(chars_written > 0, "Should have written some characters");

    // Render text buffer to surface
    let rendered_chars = unsafe {
        renderTextBufferToSurface(text_buffer, buffer, 5, 10, 50)
    };

    assert!(rendered_chars > 0, "Should have rendered some characters");
    assert_eq!(rendered_chars, chars_written, "Should render all written characters");

    // Cleanup
    unsafe {
        destroyTextBuffer(text_buffer);
        destroyBuffer(buffer);
    }
}

/// Test text buffer rendering to renderer integration
#[test]
fn test_text_buffer_to_renderer_integration() {
    // Skip if no TTY available (CI/test environment)
    let renderer = unsafe { createRenderer(80, 24, false, 0) };
    if renderer.is_null() {
        eprintln!("Skipping test_text_buffer_to_renderer_integration: Failed to create renderer");
        return;
    }

    let text_buffer = unsafe { createTextBuffer(100, 0) };
    if text_buffer.is_null() {
        unsafe { destroyRenderer(renderer) };
        eprintln!("Skipping test_text_buffer_to_renderer_integration: Failed to create text buffer");
        return;
    }

    // Add text to buffer
    let fg = [0.0, 1.0, 0.0, 1.0f32]; // Green
    let bg = [0.0, 0.0, 0.0, 1.0f32]; // Black
    let attr = 0u32;

    let text = std::ffi::CString::new("Renderer Test").unwrap();
    let chars_written = unsafe {
        textBufferAppendText(
            text_buffer,
            text.as_ptr(),
            text.as_bytes().len(),
            fg.as_ptr(),
            bg.as_ptr(),
            &attr,
        )
    };

    assert!(chars_written > 0, "Should have written characters");

    // Render to renderer
    let rendered_chars = unsafe {
        renderTextBufferToRenderer(text_buffer, renderer, 0, 0, 80)
    };

    assert!(rendered_chars > 0, "Should have rendered characters");
    assert_eq!(rendered_chars, chars_written, "Should render all characters");

    // Cleanup
    unsafe {
        destroyTextBuffer(text_buffer);
        destroyRenderer(renderer);
    }
}

/// Test stats collection with renderer integration
#[test]
fn test_stats_renderer_integration() {
    // Skip if no TTY available (CI/test environment)
    let renderer = unsafe { createRenderer(80, 24, false, 0) };
    if renderer.is_null() {
        eprintln!("Skipping test_stats_renderer_integration: Failed to create renderer");
        return;
    }

    // Enable stats collection
    unsafe {
        updateStats(renderer, 16.67, 60, 1.0);
        startProfiling(renderer);
    }

    // Get initial frame stats
    let mut avg_time = 0.0f32;
    let mut min_time = 0.0f32;
    let mut max_time = 0.0f32;
    let mut frame_count = 0u32;

    unsafe {
        getFrameStats(
            renderer,
            &mut avg_time,
            &mut min_time,
            &mut max_time,
            &mut frame_count,
        );
    }

    // Stats should be valid (even if zero initially)
    assert!(avg_time >= 0.0, "Average time should be non-negative");
    assert!(min_time >= 0.0, "Min time should be non-negative");
    assert!(max_time >= 0.0, "Max time should be non-negative");

    // Enable debug overlay
    unsafe {
        setDebugOverlay(renderer, true, 3); // Bottom right
    }

    // Test hit detection
    let hit_result = unsafe { checkHit(renderer, 10, 10) };
    assert!(hit_result <= 1, "Hit result should be 0 or 1");

    // Reset counters
    unsafe {
        resetPerformanceCounters(renderer);
        stopProfiling(renderer);
    }

    // Cleanup
    unsafe {
        destroyRenderer(renderer);
    }
}

/// Test terminal and renderer integration
#[test]
fn test_terminal_renderer_integration() {
    // Skip if no TTY available (CI/test environment)
    let terminal = unsafe { createTerminal() };
    if terminal.is_null() {
        eprintln!("Skipping test_terminal_renderer_integration: Failed to create terminal");
        return;
    }

    let renderer = unsafe { createRenderer(80, 24, false, 0) };
    if renderer.is_null() {
        unsafe { destroyTerminal(terminal) };
        eprintln!("Skipping test_terminal_renderer_integration: Failed to create renderer");
        return;
    }

    // Setup terminal
    unsafe {
        setupTerminal(terminal, true); // Use alternate screen
    }

    // Write to renderer surface
    let surface = unsafe { getRendererSurface(renderer) };
    if !surface.is_null() {
        let fg = [1.0, 1.0, 0.0, 1.0f32]; // Yellow
        let bg = [0.0, 0.0, 1.0, 1.0f32]; // Blue
        let attr = 0u32;

        let text = std::ffi::CString::new("Terminal Test").unwrap();
        unsafe {
            writeToBuffer(
                surface,
                5,
                5,
                text.as_ptr(),
                text.as_bytes().len(),
                fg.as_ptr(),
                bg.as_ptr(),
                &attr,
            );
        }
    }

    // Render (this would normally output to terminal)
    unsafe {
        render(renderer);
    }

    // Cleanup in reverse order
    unsafe {
        destroyRenderer(renderer);
        destroyTerminal(terminal);
    }
}

/// Test memory safety with null pointers
#[test]
fn test_null_pointer_safety() {
    // All these should be safe and not crash
    unsafe {
        // Destroy functions with null pointers
        destroyBuffer(ptr::null_mut());
        destroyRenderer(ptr::null_mut());
        destroyTerminal(ptr::null_mut());
        destroyTextBuffer(ptr::null_mut());

        // Operations with null pointers
        render(ptr::null_mut());
        
        let mut stats = [0.0f32; 3];
        let mut count = 0u32;
        getFrameStats(
            ptr::null(),
            &mut stats[0],
            &mut stats[1],
            &mut stats[2],
            &mut count,
        );

        renderTextBufferToSurface(
            ptr::null(),
            ptr::null_mut(),
            0, 0, 100
        );

        renderTextBufferToRenderer(
            ptr::null(),
            ptr::null_mut(),
            0, 0, 100
        );

        checkHit(ptr::null_mut(), 10, 10);
        
        updateStats(ptr::null_mut(), 16.67, 60, 1.0);
        startProfiling(ptr::null_mut());
        stopProfiling(ptr::null_mut());
        resetPerformanceCounters(ptr::null_mut());
        setDebugOverlay(ptr::null_mut(), true, 0);
    }

    // Test should complete without crashing
    assert!(true, "Null pointer operations should be safe");
}

/// Test integrated system workflows
#[test]
fn test_integrated_system_workflows() {
    // Skip if no TTY available (CI/test environment)
    let terminal = unsafe { createTerminal() };
    if terminal.is_null() {
        eprintln!("Skipping test_integrated_system_workflows: Failed to create terminal");
        return;
    }

    let text_buffer = unsafe { createTextBuffer(200, 0) };
    if text_buffer.is_null() {
        unsafe { destroyTerminal(terminal) };
        eprintln!("Skipping test_integrated_system_workflows: Failed to create text buffer");
        return;
    }

    // Test 1: Direct text-to-terminal rendering
    unsafe {
        let fg = [0.0, 1.0, 0.0, 1.0f32]; // Green
        let bg = [0.0, 0.0, 0.0, 1.0f32]; // Black
        let attr = 0u32;

        let text = std::ffi::CString::new("Integrated Workflow Test").unwrap();
        textBufferAppendText(
            text_buffer,
            text.as_ptr(),
            text.as_bytes().len(),
            fg.as_ptr(),
            bg.as_ptr(),
            &attr,
        );

        // Test direct text buffer to terminal rendering
        let success = renderTextBufferDirect(text_buffer, terminal, 0, 0, 80, 24);
        assert!(success, "Direct text buffer rendering should succeed");
    }

    // Test 2: Surface-to-terminal rendering
    let surface = unsafe { createBuffer(80, 24) };
    if !surface.is_null() {
        unsafe {
            let fg = [1.0, 0.0, 0.0, 1.0f32]; // Red
            let bg = [0.0, 0.0, 0.0, 1.0f32]; // Black
            let attr = 0u32;

            let text = std::ffi::CString::new("Surface Test").unwrap();
            writeToBuffer(
                surface,
                10,
                10,
                text.as_ptr(),
                text.as_bytes().len(),
                fg.as_ptr(),
                bg.as_ptr(),
                &attr,
            );

            // Test surface to terminal rendering
            let success = renderSurfaceToTerminal(surface, terminal);
            assert!(success, "Surface to terminal rendering should succeed");

            destroyBuffer(surface);
        }
    }

    // Test 3: Renderer with stats integration
    let renderer = unsafe { createRenderer(80, 24, false, 0) };
    if !renderer.is_null() {
        unsafe {
            // Test integrated rendering with stats
            let success = renderWithStats(renderer, terminal, true);
            assert!(success, "Renderer with stats should succeed");

            destroyRenderer(renderer);
        }
    }

    // Cleanup
    unsafe {
        destroyTextBuffer(text_buffer);
        destroyTerminal(terminal);
    }
}

/// Test complete workflow integration
#[test]
fn test_complete_workflow_integration() {
    // Skip if no TTY available (CI/test environment)
    let terminal = unsafe { createTerminal() };
    if terminal.is_null() {
        eprintln!("Skipping test_complete_workflow_integration: Failed to create terminal");
        return;
    }

    let renderer = unsafe { createRenderer(80, 24, false, 0) };
    if renderer.is_null() {
        unsafe { destroyTerminal(terminal) };
        eprintln!("Skipping test_complete_workflow_integration: Failed to create renderer");
        return;
    }

    let text_buffer = unsafe { createTextBuffer(200, 0) };
    if text_buffer.is_null() {
        unsafe {
            destroyRenderer(renderer);
            destroyTerminal(terminal);
        };
        eprintln!("Skipping test_complete_workflow_integration: Failed to create text buffer");
        return;
    }

    // Complete workflow test
    unsafe {
        // 1. Setup terminal
        setupTerminal(terminal, true);

        // 2. Enable stats
        startProfiling(renderer);
        setDebugOverlay(renderer, true, 3);

        // 3. Create content in text buffer
        let fg = [1.0, 0.5, 0.0, 1.0f32]; // Orange
        let bg = [0.0, 0.0, 0.0, 1.0f32]; // Black
        let attr = 0u32;

        let line1 = std::ffi::CString::new("Complete Workflow Test").unwrap();
        let line2 = std::ffi::CString::new("Line 2 of text").unwrap();
        
        textBufferAppendText(
            text_buffer,
            line1.as_ptr(),
            line1.as_bytes().len(),
            fg.as_ptr(),
            bg.as_ptr(),
            &attr,
        );

        textBufferAppendText(
            text_buffer,
            line2.as_ptr(),
            line2.as_bytes().len(),
            fg.as_ptr(),
            bg.as_ptr(),
            &attr,
        );

        // 4. Render text buffer to renderer
        let chars_rendered = renderTextBufferToRenderer(text_buffer, renderer, 10, 5, 60);
        assert!(chars_rendered > 0, "Should render characters");

        // 5. Render to terminal
        render(renderer);

        // 6. Check stats
        let mut avg_time = 0.0f32;
        let mut min_time = 0.0f32;
        let mut max_time = 0.0f32;
        let mut frame_count = 0u32;

        getFrameStats(
            renderer,
            &mut avg_time,
            &mut min_time,
            &mut max_time,
            &mut frame_count,
        );

        // 7. Test hit detection
        let hit = checkHit(renderer, 15, 5);
        assert!(hit <= 1, "Hit result should be valid");

        // 8. Stop profiling
        stopProfiling(renderer);
    }

    // Cleanup in reverse order
    unsafe {
        destroyTextBuffer(text_buffer);
        destroyRenderer(renderer);
        destroyTerminal(terminal);
    }
}
