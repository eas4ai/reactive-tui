//! Text editor FFI functions

use super::*;
use crate::editor::text_editor::TextEditor;
use crate::editor::gap_buffer::TextPosition;
use crate::widgets::input::text_input::{CursorPosition, Selection};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to a text editor
#[repr(C)]
pub struct RTuiTextEditor {
    _private: [u8; 0],
}

/// Opaque handle to editor config
#[repr(C)]
pub struct RTuiEditorConfig {
    _private: [u8; 0],
}

/// Opaque handle to syntax highlighter
#[repr(C)]
pub struct RTuiSyntaxHighlighter {
    _private: [u8; 0],
}

/// Simple editor mode enumeration (simplified for FFI)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiEditorMode {
    Normal = 0,
    Insert = 1,
}

/// Cursor position structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiCursorPosition {
    pub line: usize,
    pub column: usize,
}

impl From<RTuiCursorPosition> for CursorPosition {
    fn from(pos: RTuiCursorPosition) -> Self {
        CursorPosition {
            line: pos.line,
            column: pos.column,
        }
    }
}

impl From<CursorPosition> for RTuiCursorPosition {
    fn from(pos: CursorPosition) -> Self {
        RTuiCursorPosition {
            line: pos.line,
            column: pos.column,
        }
    }
}

/// Text selection structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiSelection {
    pub start: RTuiCursorPosition,
    pub end: RTuiCursorPosition,
}

impl From<RTuiSelection> for Selection {
    fn from(sel: RTuiSelection) -> Self {
        Selection {
            start: sel.start.into(),
            end: sel.end.into(),
        }
    }
}

impl From<Selection> for RTuiSelection {
    fn from(sel: Selection) -> Self {
        RTuiSelection {
            start: sel.start.into(),
            end: sel.end.into(),
        }
    }
}

/// Editor change callback function type
pub type RTuiEditorChangeCallback = extern "C" fn(user_data: *mut std::ffi::c_void);

/// Editor cursor move callback function type
pub type RTuiEditorCursorMoveCallback = extern "C" fn(
    position: RTuiCursorPosition,
    user_data: *mut std::ffi::c_void,
);

/// Create a text editor with default settings
#[no_mangle]
pub extern "C" fn rtui_text_editor_create(
    out_editor: *mut *mut RTuiTextEditor,
) -> ReactiveError {
    if out_editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let editor = TextEditor::new();
        unsafe {
            *out_editor = Box::into_raw(Box::new(editor)) as *mut RTuiTextEditor;
        }
        Ok(())
    }))
}

/// Destroy editor configuration
#[no_mangle]
pub extern "C" fn rtui_editor_config_destroy(config: *mut RTuiEditorConfig) {
    if !config.is_null() {
        unsafe {
            let _ = Box::from_raw(config as *mut EditorConfig);
        }
    }
}

/// Set tab size in editor config
#[no_mangle]
pub extern "C" fn rtui_editor_config_set_tab_size(
    config: *mut RTuiEditorConfig,
    tab_size: usize,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut EditorConfig);
        config_ref.set_tab_size(tab_size);
        Ok(())
    }))
}

/// Set line numbers visibility in editor config
#[no_mangle]
pub extern "C" fn rtui_editor_config_set_show_line_numbers(
    config: *mut RTuiEditorConfig,
    show: bool,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut EditorConfig);
        config_ref.set_show_line_numbers(show);
        Ok(())
    }))
}

/// Set word wrap in editor config
#[no_mangle]
pub extern "C" fn rtui_editor_config_set_word_wrap(
    config: *mut RTuiEditorConfig,
    wrap: bool,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut EditorConfig);
        config_ref.set_word_wrap(wrap);
        Ok(())
    }))
}

/// Create a text editor
#[no_mangle]
pub extern "C" fn rtui_text_editor_create(
    config: *const RTuiEditorConfig,
    out_editor: *mut *mut RTuiTextEditor,
) -> ReactiveError {
    if out_editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_config = if config.is_null() {
            EditorConfig::default()
        } else {
            (*(config as *const EditorConfig)).clone()
        };

        let editor = TextEditor::new(editor_config);
        *out_editor = Box::into_raw(Box::new(editor)) as *mut RTuiTextEditor;
        Ok(())
    }))
}

/// Destroy a text editor
#[no_mangle]
pub extern "C" fn rtui_text_editor_destroy(editor: *mut RTuiTextEditor) {
    if !editor.is_null() {
        unsafe {
            let _ = Box::from_raw(editor as *mut TextEditor);
        }
    }
}

/// Set editor content
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_content(
    editor: *mut RTuiTextEditor,
    content: *const c_char,
) -> ReactiveError {
    if editor.is_null() || content.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let content_str = CStr::from_ptr(content)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.set_content(content_str);
        Ok(())
    }))
}

/// Get editor content
#[no_mangle]
pub extern "C" fn rtui_text_editor_get_content(
    editor: *const RTuiTextEditor,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if editor.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &*(editor as *const TextEditor);
        let content = editor_ref.content();
        
        if content.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(content).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Insert text at cursor position
#[no_mangle]
pub extern "C" fn rtui_text_editor_insert_text(
    editor: *mut RTuiTextEditor,
    text: *const c_char,
) -> ReactiveError {
    if editor.is_null() || text.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.insert_text(text_str);
        Ok(())
    }))
}

/// Delete text in selection or at cursor
#[no_mangle]
pub extern "C" fn rtui_text_editor_delete_text(
    editor: *mut RTuiTextEditor,
    count: usize,
) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.delete_text(count);
        Ok(())
    }))
}

/// Set cursor position
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_cursor_position(
    editor: *mut RTuiTextEditor,
    position: RTuiCursorPosition,
) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.set_cursor_position(position.into());
        Ok(())
    }))
}

/// Get cursor position
#[no_mangle]
pub extern "C" fn rtui_text_editor_get_cursor_position(
    editor: *const RTuiTextEditor,
    out_position: *mut RTuiCursorPosition,
) -> ReactiveError {
    if editor.is_null() || out_position.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &*(editor as *const TextEditor);
        let position = editor_ref.cursor_position();
        *out_position = position.into();
        Ok(())
    }))
}

/// Set text selection
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_selection(
    editor: *mut RTuiTextEditor,
    selection: RTuiSelection,
) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.set_selection(Some(selection.into()));
        Ok(())
    }))
}

/// Get text selection
#[no_mangle]
pub extern "C" fn rtui_text_editor_get_selection(
    editor: *const RTuiTextEditor,
    out_selection: *mut RTuiSelection,
    out_has_selection: *mut bool,
) -> ReactiveError {
    if editor.is_null() || out_selection.is_null() || out_has_selection.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &*(editor as *const TextEditor);
        if let Some(selection) = editor_ref.selection() {
            *out_selection = selection.into();
            *out_has_selection = true;
        } else {
            *out_has_selection = false;
        }
        Ok(())
    }))
}

/// Set editor mode
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_mode(
    editor: *mut RTuiTextEditor,
    mode: RTuiEditorMode,
) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.set_mode(mode.into());
        Ok(())
    }))
}

/// Get editor mode
#[no_mangle]
pub extern "C" fn rtui_text_editor_get_mode(
    editor: *const RTuiTextEditor,
    out_mode: *mut RTuiEditorMode,
) -> ReactiveError {
    if editor.is_null() || out_mode.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &*(editor as *const TextEditor);
        let mode = editor_ref.mode();
        *out_mode = match mode {
            EditorMode::Normal => RTuiEditorMode::Normal,
            EditorMode::Insert => RTuiEditorMode::Insert,
            EditorMode::Visual => RTuiEditorMode::Visual,
            EditorMode::Command => RTuiEditorMode::Command,
            EditorMode::Search => RTuiEditorMode::Search,
        };
        Ok(())
    }))
}

/// Undo last operation
#[no_mangle]
pub extern "C" fn rtui_text_editor_undo(editor: *mut RTuiTextEditor) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.undo();
        Ok(())
    }))
}

/// Redo last undone operation
#[no_mangle]
pub extern "C" fn rtui_text_editor_redo(editor: *mut RTuiTextEditor) -> ReactiveError {
    if editor.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        editor_ref.redo();
        Ok(())
    }))
}

/// Set syntax highlighter for editor
#[no_mangle]
pub extern "C" fn rtui_text_editor_set_syntax_highlighter(
    editor: *mut RTuiTextEditor,
    highlighter: *mut RTuiSyntaxHighlighter,
) -> ReactiveError {
    if editor.is_null() || highlighter.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &mut *(editor as *mut TextEditor);
        let highlighter_box = Box::from_raw(highlighter as *mut SyntaxHighlighter);
        editor_ref.set_syntax_highlighter(Some(*highlighter_box));
        Ok(())
    }))
}

/// Render editor to element
#[no_mangle]
pub extern "C" fn rtui_text_editor_render(
    editor: *const RTuiTextEditor,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if editor.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let editor_ref = &*(editor as *const TextEditor);
        let element = editor_ref.render();
        *out_element = Box::into_raw(Box::new(element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}
