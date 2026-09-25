//! Unicode editor access backed by the recovered native TextEditor.
use super::controller::{self, Handle};
use super::{catch_panic, RTuiElement, ReactiveError};
use crate::component::{Element, LayoutType};
use crate::editor::{cursor::Movement, TextEditor};
use crate::layout::style::StyleBuilder;
use std::ffi::c_char;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

/// Owning TextEditor handle; confined to its creating thread.
#[repr(C)]
pub struct RTuiTextEditor {
    _private: [u8; 0],
}

struct Editor {
    inner: TextEditor,
    width: u32,
    height: u32,
}

/// Create an empty editor with the native default viewport and line numbers.
#[no_mangle]
pub extern "C" fn rtui_text_editor_create(out_editor: *mut *mut RTuiTextEditor) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_editor)?;
        *out_editor = Box::into_raw(Box::new(Handle::new(Editor {
            inner: TextEditor::new(),
            width: 80,
            height: 24,
        })))
        .cast();
        Ok(())
    }))
}

/// Release an editor on its creating thread; null is accepted.
#[no_mangle]
pub extern "C" fn rtui_text_editor_destroy(editor: *mut RTuiTextEditor) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        Handle::<Editor>::destroy(editor)
    }))
}

/// Replace all text and reset cursor, selection and scroll.
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_content(
    editor: *mut RTuiTextEditor,
    content: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let text = controller::string(content)?;
        Handle::<Editor>::get_mut(editor)?.inner.set_content(text);
        Ok(())
    }))
}

/// Return an owned UTF-8 copy; release it with rtui_string_free.
#[no_mangle]
pub extern "C" fn rtui_text_editor_get_content_owned(
    editor: *const RTuiTextEditor,
    out_content: *mut *mut c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_content)?;
        controller::owned_string(out_content, Handle::<Editor>::get(editor)?.inner.content())
    }))
}

/// Insert UTF-8 text, replacing the complete selected graphemes.
#[no_mangle]
pub extern "C" fn rtui_text_editor_insert_text(
    editor: *mut RTuiTextEditor,
    text: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let text = controller::string(text)?;
        Handle::<Editor>::get_mut(editor)?.inner.insert_text(text);
        Ok(())
    }))
}

/// Delete the selection, or one preceding/following complete grapheme.
#[no_mangle]
pub extern "C" fn rtui_text_editor_delete(
    editor: *mut RTuiTextEditor,
    backward: bool,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor = &mut Handle::<Editor>::get_mut(editor)?.inner;
        if backward {
            editor.delete_backward();
        } else {
            editor.delete_forward();
        }
        Ok(())
    }))
}

/// Movement codes: left/right/up/down=0..3, line start/end=4..5,
/// document start/end=6..7, word backward/forward=8..9. Others are invalid.
#[no_mangle]
pub extern "C" fn rtui_text_editor_move(
    editor: *mut RTuiTextEditor,
    movement: u32,
    select: bool,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let movement = match movement {
            0 => Movement::Left,
            1 => Movement::Right,
            2 => Movement::Up,
            3 => Movement::Down,
            4 => Movement::LineStart,
            5 => Movement::LineEnd,
            6 => Movement::DocumentStart,
            7 => Movement::DocumentEnd,
            8 => Movement::WordBackward,
            9 => Movement::WordForward,
            _ => return Err(ReactiveError::InvalidParameter),
        };
        Handle::<Editor>::get_mut(editor)?
            .inner
            .move_cursor(movement, select);
        Ok(())
    }))
}

/// Set a bounded viewport measured in terminal cells; invalid dimensions do not mutate it.
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_size(
    editor: *mut RTuiTextEditor,
    width: u32,
    height: u32,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if width > u16::MAX.into()
            || height > u16::MAX.into()
            || u64::from(width) * u64::from(height) > crate::backend::CellFrame::MAX_CELLS as u64
        {
            return Err(ReactiveError::InvalidParameter);
        }
        let editor = Handle::<Editor>::get_mut(editor)?;
        editor.width = width;
        editor.height = height;
        editor.inner.set_size(width as usize, height as usize);
        Ok(())
    }))
}

/// Show or hide the native line-number gutter.
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_show_line_numbers(
    editor: *mut RTuiTextEditor,
    show: bool,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        Handle::<Editor>::get_mut(editor)?
            .inner
            .set_show_line_numbers(show);
        Ok(())
    }))
}

/// Copy the visible styled lines into an owned Element snapshot.
#[no_mangle]
pub extern "C" fn rtui_text_editor_element(
    editor: *const RTuiTextEditor,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_element)?;
        let editor = Handle::<Editor>::get(editor)?;
        let mut root = Element::layout(LayoutType::Flex).with_class(format!(
            "flex-col w-{} h-{} shrink-0 overflow-hidden",
            editor.width, editor.height
        ));
        for line in editor.inner.get_styled_lines() {
            let mut row = Element::layout(LayoutType::Flex).with_class("flex-row h-1 shrink-0");
            for run in line.runs {
                let mut text = Element::text(run.text).with_class("whitespace-pre shrink-0");
                let style = StyleBuilder::new()
                    .text_rgba(run.fg.r, run.fg.g, run.fg.b, run.fg.a)
                    .bg_rgba(run.bg.r, run.bg.g, run.bg.b, run.bg.a);
                text.metadata.styles = Some(Arc::new(style.snapshot()));
                row.children.push(text);
            }
            root.children.push(row);
        }
        *out_element = Box::into_raw(Box::new(root)).cast();
        Ok(())
    }))
}
