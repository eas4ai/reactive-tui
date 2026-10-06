//! Part of tests/review_facade.rs: FFI-003, the C widget constructors return
//! the native controls.

use crate::common::app_input::{self, Until};
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::event::types::KeyCode;
use reactive_tui::ffi::*;
use std::ptr;
use std::time::Duration;

/// A root that shows one element, the C constructor's, with the focus.
struct Root(Element);

impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// The element a C constructor returned, as the Rust element it wraps.
fn element_of(handle: *mut RTuiElement) -> Element {
    assert!(!handle.is_null(), "the constructor gave no element");
    unsafe { (*(handle as *const FFIElement)).inner.clone() }
}

fn type_of(handle: *mut RTuiElement) -> RTuiElementType {
    let mut kind = RTuiElementType::Empty;
    let status = unsafe { rtui_element_get_type(handle, &mut kind) };
    assert_eq!(
        status,
        ReactiveError::Success,
        "rtui_element_get_type failed"
    );
    kind
}

fn text_input(placeholder: &std::ffi::CStr, value: &std::ffi::CStr) -> *mut RTuiElement {
    let mut out = ptr::null_mut();
    let status = unsafe { rtui_text_input_create(placeholder.as_ptr(), value.as_ptr(), &mut out) };
    assert_eq!(
        status,
        ReactiveError::Success,
        "rtui_text_input_create failed"
    );
    out
}

fn checkbox(label: &std::ffi::CStr, checked: bool) -> *mut RTuiElement {
    let mut out = ptr::null_mut();
    let status = unsafe { rtui_checkbox_create(label.as_ptr(), checked, &mut out) };
    assert_eq!(
        status,
        ReactiveError::Success,
        "rtui_checkbox_create failed"
    );
    out
}

fn progress_bar(value: f64, label: &std::ffi::CStr) -> *mut RTuiElement {
    let mut out = ptr::null_mut();
    let status = unsafe { rtui_progress_bar_create(0.0, 100.0, value, label.as_ptr(), &mut out) };
    assert_eq!(
        status,
        ReactiveError::Success,
        "rtui_progress_bar_create failed"
    );
    out
}

/// FFI-003: the three constructors return native controls, which are
/// component elements, not text or layout.
#[test]
fn ffi_003_the_constructors_return_native_controls() {
    let input = text_input(c"Type", c"seed");
    let check = checkbox(c"Agree", false);
    let bar = progress_bar(50.0, c"half");
    // The element types as the header numbers them: 0 component, 1 text,
    // 2 layout, 3 fragment, 4 empty.
    let kinds = [
        type_of(input) as i32,
        type_of(check) as i32,
        type_of(bar) as i32,
    ];
    unsafe {
        rtui_element_destroy(input);
        rtui_element_destroy(check);
        rtui_element_destroy(bar);
    }
    assert!(
        kinds.iter().all(|kind| *kind == RTuiElementType::Component as i32),
        "FFI-003: the text input, checkbox and progress bar from the C constructors are the element types {kinds:?} (0 is a component), not native controls"
    );
}

/// FFI-003: a C text input takes typing in an App, like the Rust one.
#[test]
fn ffi_003_a_c_text_input_takes_typing() {
    let handle = text_input(c"Type", c"seed");
    let element = element_of(handle).auto_focus();
    unsafe { rtui_element_destroy(handle) };
    let frames = app_input::run_until_on_debug(
        Root(element),
        (60, 12),
        vec![
            Until {
                text: "seed",
                cell: None,
                event: app_input::key(KeyCode::Char('x')),
            },
            Until {
                text: "seedx",
                cell: None,
                event: None,
            },
        ],
        Duration::from_secs(20),
    );
    let last = frames
        .last()
        .map(|frame| frame.text.clone())
        .unwrap_or_default();
    assert!(
        last.contains("seedx"),
        "FFI-003: typing x into the focused C text input left the frame:\n{last}"
    );
}

/// FFI-003: a C checkbox toggles on Space in an App, like the Rust one.
#[test]
fn ffi_003_a_c_checkbox_toggles() {
    let handle = checkbox(c"Agree", false);
    let element = element_of(handle).auto_focus();
    unsafe { rtui_element_destroy(handle) };
    let frames = app_input::run_until_on_debug(
        Root(element),
        (60, 12),
        vec![
            Until {
                text: "Agree",
                cell: None,
                event: app_input::key(KeyCode::Char(' ')),
            },
            Until {
                text: "✓",
                cell: None,
                event: None,
            },
        ],
        Duration::from_secs(20),
    );
    let last = frames
        .last()
        .map(|frame| frame.text.clone())
        .unwrap_or_default();
    assert!(
        last.contains('✓'),
        "FFI-003: Space on the focused C checkbox left the frame without its mark:\n{last}"
    );
}

/// FFI-003: a C progress bar paints a bar for its value: the filled part and
/// the track differ in color.
#[test]
fn ffi_003_a_c_progress_bar_paints_its_value_as_a_bar() {
    let handle = progress_bar(50.0, c"half");
    let element = element_of(handle);
    unsafe { rtui_element_destroy(handle) };
    let frames = app_input::run_when_painted_on_debug(Root(element), (60, 12), 1);
    let frame = frames.last().expect("a painted frame");
    // The native bar paints its filled part in `primary` on the track's
    // `border`, each half the row at 50%. A percentage in a bordered box
    // gives one background most of the row.
    let balanced = (0..12)
        .map(|row| {
            let mut counts = std::collections::BTreeMap::new();
            for column in 0..60 {
                if let Some(cell) = frame.screen.cell(row, column) {
                    *counts
                        .entry(format!("{:?}", cell.bgcolor()))
                        .or_insert(0usize) += 1;
                }
            }
            let mut sizes: Vec<usize> = counts.into_values().collect();
            sizes.sort_unstable_by(|a, b| b.cmp(a));
            sizes.get(1).copied().unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    assert!(
        balanced >= 20,
        "FFI-003: the C progress bar at 50% painted no filled part against a track (the second background of a row covers at most {balanced} of 60 cells), no bar:\n{}",
        frame.text
    );
}

/// FFI-003: the header documents the four constructors.
#[test]
fn ffi_003_the_header_documents_the_constructors() {
    let undocumented: Vec<&str> = [
        "rtui_text_input_create",
        "rtui_checkbox_create",
        "rtui_progress_bar_create",
        "rtui_button_create",
    ]
    .into_iter()
    .filter(|function| !crate::header::documented(function))
    .collect();
    assert!(
        undocumented.is_empty(),
        "FFI-003: the header declares {undocumented:?} without documentation"
    );
}
