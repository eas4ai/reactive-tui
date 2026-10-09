//! FFI integration tests

#![cfg(feature = "ffi")]

use reactive_tui::ffi::*;
use std::ffi::CString;

use std::ptr;

#[cfg(unix)]
mod pty_support;

/// Set in the copy of this binary that a test runs on a pseudo-terminal.
#[cfg(unix)]
const ON_TERMINAL: &str = "REACTIVE_TUI_FFI_TESTS_ON_TERMINAL";

/// Runs `body` on a real terminal: in a copy of this binary that runs `test`
/// (its path as libtest prints it) on a pseudo-terminal, or directly when
/// this process already is that copy. The test asserts that the copy passed
/// and shows what it wrote when it did not. The renderer and terminal
/// functions enable raw mode and enter the alternate screen, which the pipes
/// of `cargo test` refuse.
#[cfg(unix)]
fn on_terminal(test: &str, body: impl FnOnce()) {
    if std::env::var_os(ON_TERMINAL).is_some() {
        body();
        return;
    }
    let run = pty_support::run_test(test, &[(ON_TERMINAL, "1")]);
    let text = run.text();
    assert!(
        run.status.success() && text.contains("test result: ok. 1 passed"),
        "{test} failed on the pseudo-terminal (exit {:?}); it wrote:\n{text}",
        run.status.code()
    );
}

/// Without a pseudo-terminal the body runs in the test's own process.
#[cfg(not(unix))]
fn on_terminal(_test: &str, body: impl FnOnce()) {
    body();
}

/// Runs the named test's body on a real terminal (see `on_terminal`).
macro_rules! on_terminal {
    ($name:ident, $body:block) => {{
        let path = match module_path!().split_once("::") {
            Some((_, module)) => format!("{module}::{}", stringify!($name)),
            None => stringify!($name).to_owned(),
        };
        on_terminal(&path, || $body)
    }};
}

#[test]
fn test_version() {
    let version = rtui_version();
    let reported = format!("{}.{}.{}", version.major, version.minor, version.patch);
    assert_eq!(reported, env!("CARGO_PKG_VERSION"));
    assert_eq!(version.abi_version, 1);
}

#[test]
fn test_init_cleanup() {
    assert_eq!(rtui_init(), ReactiveError::Success, "FFI call failed");
    rtui_cleanup();
}

mod terminal_tests {
    use super::*;

    #[test]
    fn test_terminal_create_destroy() {
        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert!(!terminal.is_null());
        unsafe { rtui_terminal_destroy(terminal) };
    }

    #[test]
    fn test_terminal_null_pointer() {
        assert_eq!(
            unsafe { rtui_terminal_create(ptr::null_mut()) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );
    }

    #[test]
    fn test_terminal_dimensions() {
        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let mut dims = RTuiDimensions {
            width: 0,
            height: 0,
        };
        assert_eq!(
            unsafe { rtui_terminal_get_dimensions(terminal, &mut dims) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert!(dims.width > 0);
        assert!(dims.height > 0);

        unsafe { rtui_terminal_destroy(terminal) };
    }

    #[test]
    fn test_terminal_dimensions_null() {
        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );

        assert_eq!(
            unsafe { rtui_terminal_get_dimensions(terminal, ptr::null_mut()) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );
        assert_eq!(
            unsafe {
                rtui_terminal_get_dimensions(
                    ptr::null(),
                    &mut RTuiDimensions {
                        width: 0,
                        height: 0,
                    },
                )
            },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );

        unsafe { rtui_terminal_destroy(terminal) };
    }

    #[test]
    fn test_terminal_sync_operations() {
        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );

        assert_eq!(
            unsafe { rtui_terminal_sync(terminal, true) },
            ReactiveError::Success,
            "FFI call failed"
        ); // begin
        assert_eq!(
            unsafe { rtui_terminal_sync(terminal, false) },
            ReactiveError::Success,
            "FFI call failed"
        ); // end

        unsafe { rtui_terminal_destroy(terminal) };
    }

    #[test]
    fn test_terminal_poll_event_timeout() {
        on_terminal!(test_terminal_poll_event_timeout, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut event = RTuiEvent {
                event_type: RTuiEventType::Key,
                data: RTuiEventData {
                    key: RTuiKeyEvent {
                        key_code: 0,
                        modifiers: 0,
                    },
                },
            };

            // Non-blocking poll should return NotFound if no event
            let result = unsafe { rtui_terminal_poll_event(0, &mut event) };
            assert!(result == ReactiveError::NotFound || result == ReactiveError::Success);

            unsafe { rtui_terminal_destroy(terminal) };
        });
    }
}

mod surface_tests {
    use super::*;

    #[test]
    fn test_surface_create_destroy() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert!(!surface.is_null());
        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_surface_invalid_dimensions() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(0, 24, &mut surface) },
            ReactiveError::InvalidParameter,
            "Expected error {:?}",
            ReactiveError::InvalidParameter
        );
        assert_eq!(
            unsafe { rtui_surface_create(80, 0, &mut surface) },
            ReactiveError::InvalidParameter,
            "Expected error {:?}",
            ReactiveError::InvalidParameter
        );
    }

    #[test]
    fn test_surface_dimensions() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(100, 50, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let mut dims = RTuiDimensions {
            width: 0,
            height: 0,
        };
        assert_eq!(
            unsafe { rtui_surface_get_dimensions(surface, &mut dims) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert_eq!(dims.width, 100);
        assert_eq!(dims.height, 50);

        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_surface_clear() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        assert_eq!(
            unsafe { rtui_surface_clear(surface, 0, 0, 0) },
            ReactiveError::Success,
            "FFI call failed"
        );

        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_surface_set_get_cell() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let cell = RTuiCell {
            ch: 'A' as u32,
            fg: RTuiColor {
                r: 255,
                g: 128,
                b: 0,
            },
            bg: RTuiColor { r: 0, g: 0, b: 128 },
            attrs: RTuiTextAttributes {
                bold: true,
                italic: false,
                underline: true,
                strikethrough: false,
                reverse: false,
                blink: false,
                hidden: false,
            },
        };

        assert_eq!(
            unsafe { rtui_surface_set_cell(surface, 10, 5, &cell) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let mut retrieved = RTuiCell {
            ch: 0,
            fg: RTuiColor { r: 0, g: 0, b: 0 },
            bg: RTuiColor { r: 0, g: 0, b: 0 },
            attrs: RTuiTextAttributes {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                reverse: false,
                blink: false,
                hidden: false,
            },
        };

        assert_eq!(
            unsafe { rtui_surface_get_cell(surface, 10, 5, &mut retrieved) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert_eq!(retrieved.ch, 'A' as u32);
        assert_eq!(retrieved.fg.r, 255);
        assert_eq!(retrieved.fg.g, 128);
        assert_eq!(retrieved.fg.b, 0);
        assert_eq!(retrieved.bg.b, 128);
        assert!(retrieved.attrs.bold);
        assert!(retrieved.attrs.underline);

        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_surface_draw_text() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let text = CString::new("Hello, FFI!").unwrap();
        let fg = RTuiColor {
            r: 255,
            g: 255,
            b: 255,
        };
        let bg = RTuiColor { r: 0, g: 0, b: 0 };

        assert_eq!(
            unsafe { rtui_surface_draw_text(surface, 5, 10, text.as_ptr(), &fg, &bg) },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Verify first character
        let mut cell = RTuiCell {
            ch: 0,
            fg: RTuiColor { r: 0, g: 0, b: 0 },
            bg: RTuiColor { r: 0, g: 0, b: 0 },
            attrs: RTuiTextAttributes {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                reverse: false,
                blink: false,
                hidden: false,
            },
        };

        assert_eq!(
            unsafe { rtui_surface_get_cell(surface, 5, 10, &mut cell) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert_eq!(cell.ch, 'H' as u32);
        assert_eq!(cell.fg.r, 255);

        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_surface_fill_rect() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let rect = RTuiRect {
            x: 10,
            y: 5,
            width: 20,
            height: 10,
        };

        let fg = RTuiColor {
            r: 128,
            g: 128,
            b: 128,
        };
        let bg = RTuiColor {
            r: 32,
            g: 32,
            b: 32,
        };

        assert_eq!(
            unsafe { rtui_surface_fill_rect(surface, &rect, '#' as u32, &fg, &bg) },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Verify a cell in the filled area
        let mut cell = RTuiCell {
            ch: 0,
            fg: RTuiColor { r: 0, g: 0, b: 0 },
            bg: RTuiColor { r: 0, g: 0, b: 0 },
            attrs: RTuiTextAttributes {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                reverse: false,
                blink: false,
                hidden: false,
            },
        };

        assert_eq!(
            unsafe { rtui_surface_get_cell(surface, 15, 8, &mut cell) },
            ReactiveError::Success,
            "FFI call failed"
        );
        assert_eq!(cell.ch, '#' as u32);
        assert_eq!(cell.fg.r, 128);
        assert_eq!(cell.bg.r, 32);

        unsafe { rtui_surface_destroy(surface) };
    }
}

mod renderer_tests {
    use super::*;

    #[test]
    fn test_renderer_create_destroy() {
        on_terminal!(test_renderer_create_destroy, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );
            assert!(!renderer.is_null());

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }

    #[test]
    fn test_renderer_null_output() {
        assert_eq!(
            unsafe { rtui_renderer_create(80, 24, ptr::null_mut()) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );

        let mut renderer: *mut RTuiRenderer = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_renderer_create(0, 0, &mut renderer) },
            ReactiveError::InvalidParameter,
            "Expected error {:?}",
            ReactiveError::InvalidParameter
        );
    }

    #[test]
    fn test_renderer_frame_operations() {
        on_terminal!(test_renderer_frame_operations, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, true) },
                ReactiveError::Success,
                "FFI call failed"
            );
            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, false) },
                ReactiveError::Success,
                "FFI call failed"
            );

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }

    #[test]
    fn test_renderer_get_surface() {
        on_terminal!(test_renderer_get_surface, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, true) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut surface: *mut RTuiSurface = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_get_surface(renderer, &mut surface) },
                ReactiveError::Success,
                "FFI call failed"
            );
            assert!(!surface.is_null());

            // Should be able to draw to the surface
            let text = CString::new("Renderer Test").unwrap();
            let fg = RTuiColor {
                r: 255,
                g: 255,
                b: 255,
            };
            assert_eq!(
                unsafe { rtui_surface_draw_text(surface, 0, 0, text.as_ptr(), &fg, ptr::null()) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, false) },
                ReactiveError::Success,
                "FFI call failed"
            );

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }

    #[test]
    fn test_renderer_resize() {
        on_terminal!(test_renderer_resize, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_resize(renderer, 120, 40) },
                ReactiveError::Success,
                "FFI call failed"
            );

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }

    #[test]
    fn test_renderer_force_redraw() {
        on_terminal!(test_renderer_force_redraw, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_clear(renderer, 0, 0, 0) },
                ReactiveError::Success,
                "FFI call failed"
            );

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }

    #[test]
    fn test_renderer_shutdown() {
        on_terminal!(test_renderer_shutdown, {
            let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_terminal_create(&mut terminal) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut renderer: *mut RTuiRenderer = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_create(80, 24, &mut renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_shutdown(renderer) },
                ReactiveError::Success,
                "FFI call failed"
            );

            unsafe { rtui_renderer_destroy(renderer) };
            unsafe { rtui_terminal_destroy(terminal) };
        });
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ReactiveError::Success as i32, 0);
        assert!((ReactiveError::InvalidParameter as i32) < 0);
        assert!((ReactiveError::NullPointer as i32) < 0);
        assert!(ReactiveError::Panic as i32 == -99);
        assert!(ReactiveError::Unknown as i32 == -100);
    }

    #[test]
    fn test_error_is_success() {
        assert!(ReactiveError::Success.is_success());
        assert!(!ReactiveError::InvalidParameter.is_success());
        assert!(!ReactiveError::NullPointer.is_success());
    }

    #[test]
    fn test_null_pointer_checks() {
        // Test various null pointer scenarios
        // destroy should safely handle null
        unsafe { rtui_terminal_destroy(ptr::null_mut()) };

        assert_eq!(
            unsafe { rtui_terminal_sync(ptr::null_mut(), true) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );

        assert_eq!(
            unsafe { rtui_surface_clear(ptr::null_mut(), 0, 0, 0) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );

        assert_eq!(
            unsafe { rtui_renderer_frame(ptr::null_mut(), true) },
            ReactiveError::NullPointer,
            "Expected error {:?}",
            ReactiveError::NullPointer
        );
    }
}

mod memory_safety_tests {
    use super::*;

    #[test]
    fn smoke_test_double_destroy_safety() {
        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );

        unsafe { rtui_terminal_destroy(terminal) };
        // Second destroy should be safe (no crash)
        unsafe { rtui_terminal_destroy(terminal) };
    }

    #[test]
    fn test_use_after_free_protection() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        unsafe { rtui_surface_destroy(surface) };

        // These should not crash (undefined behavior protection)
        // They should either safely fail or be no-ops
        let result = unsafe { rtui_surface_clear(surface, 0, 0, 0) };
        assert!(result != ReactiveError::Success);
    }

    #[test]
    fn test_string_handling() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(80, 24, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Test with various string inputs
        let valid_text = CString::new("Valid UTF-8 text").unwrap();
        assert_eq!(
            unsafe {
                rtui_surface_draw_text(surface, 0, 0, valid_text.as_ptr(), ptr::null(), ptr::null())
            },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Test with empty string
        let empty_text = CString::new("").unwrap();
        assert_eq!(
            unsafe {
                rtui_surface_draw_text(surface, 0, 1, empty_text.as_ptr(), ptr::null(), ptr::null())
            },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Test with Unicode
        let unicode_text = CString::new("Hello 世界 🦀").unwrap();
        assert_eq!(
            unsafe {
                rtui_surface_draw_text(
                    surface,
                    0,
                    2,
                    unicode_text.as_ptr(),
                    ptr::null(),
                    ptr::null(),
                )
            },
            ReactiveError::Success,
            "FFI call failed"
        );

        unsafe { rtui_surface_destroy(surface) };
    }

    #[test]
    fn test_bounds_checking() {
        let mut surface: *mut RTuiSurface = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_surface_create(10, 10, &mut surface) },
            ReactiveError::Success,
            "FFI call failed"
        );

        // These should not crash even if out of bounds
        let cell = RTuiCell {
            ch: 'X' as u32,
            fg: RTuiColor {
                r: 255,
                g: 255,
                b: 255,
            },
            bg: RTuiColor { r: 0, g: 0, b: 0 },
            attrs: RTuiTextAttributes {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                reverse: false,
                blink: false,
                hidden: false,
            },
        };

        // Setting a cell out of bounds is clipped: the call succeeds and
        // the surface keeps its contents.
        assert_eq!(
            unsafe { rtui_surface_set_cell(surface, 100, 100, &cell) },
            ReactiveError::Success,
            "out-of-bounds set is clipped rather than rejected"
        );
        assert_eq!(
            unsafe { rtui_surface_set_cell(surface, 0, 0, &cell) },
            ReactiveError::Success,
            "in-bounds set succeeds"
        );

        unsafe { rtui_surface_destroy(surface) };
    }
}

#[test]
fn test_full_integration() {
    on_terminal!(test_full_integration, {
        // Complete integration test simulating real usage
        assert_eq!(rtui_init(), ReactiveError::Success, "FFI call failed");

        let mut terminal: *mut ReactiveTerminal = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_terminal_create(&mut terminal) },
            ReactiveError::Success,
            "FFI call failed"
        );

        let mut renderer: *mut RTuiRenderer = ptr::null_mut();
        assert_eq!(
            unsafe { rtui_renderer_create(80, 24, &mut renderer) },
            ReactiveError::Success,
            "FFI call failed"
        );

        // Simulate a few frames
        for i in 0..3 {
            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, true) },
                ReactiveError::Success,
                "FFI call failed"
            );

            let mut surface: *mut RTuiSurface = ptr::null_mut();
            assert_eq!(
                unsafe { rtui_renderer_get_surface(renderer, &mut surface) },
                ReactiveError::Success,
                "FFI call failed"
            );

            // Draw frame number
            let text = CString::new(format!("Frame {}", i)).unwrap();
            let fg = RTuiColor {
                r: 255,
                g: 255,
                b: 255,
            };
            let bg = RTuiColor { r: 0, g: 0, b: 128 };

            assert_eq!(
                unsafe { rtui_surface_draw_text(surface, 10, 10, text.as_ptr(), &fg, &bg) },
                ReactiveError::Success,
                "FFI call failed"
            );

            assert_eq!(
                unsafe { rtui_renderer_frame(renderer, false) },
                ReactiveError::Success,
                "FFI call failed"
            );
        }

        assert_eq!(
            unsafe { rtui_renderer_shutdown(renderer) },
            ReactiveError::Success,
            "FFI call failed"
        );
        unsafe { rtui_renderer_destroy(renderer) };
        unsafe { rtui_terminal_destroy(terminal) };

        rtui_cleanup();
    });
}
