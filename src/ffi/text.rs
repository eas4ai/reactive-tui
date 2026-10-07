//! Text buffer FFI functions - OpenTUI-aligned
//!
//! Provides text buffer operations following OpenTUI's patterns for text handling

use super::*;
use crate::core::surface::{Attr, Cell, Rgba, Surface};
use crate::ffi::lib::RTuiBuffer;
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn text_buffer_tracker() -> &'static super::pointer::PointerTracker<TextBuffer> {
    static TRACKER: OnceLock<super::pointer::PointerTracker<TextBuffer>> = OnceLock::new();
    TRACKER.get_or_init(super::pointer::PointerTracker::new)
}

/// Text buffer handle (opaque)
#[repr(C)]
pub struct RTuiTextBuffer {
    _private: [u8; 0],
}

/// Line information structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LineInfo {
    /// Character start index
    pub char_start: u32,
    /// Line width in cells
    pub width: u32,
}

/// Text buffer implementation
pub struct TextBuffer {
    /// Character data
    chars: Vec<u32>,
    /// Foreground colors
    fg_colors: Vec<Rgba>,
    /// Background colors  
    bg_colors: Vec<Rgba>,
    /// Text attributes
    attributes: Vec<u8>,
    /// Line information
    lines: Vec<LineInfo>,
    /// Current length
    length: u32,
    /// Capacity
    capacity: u32,
    /// Selection start/end
    selection: Option<(u32, u32)>,
    /// Selection colors
    selection_fg: Option<Rgba>,
    selection_bg: Option<Rgba>,
    /// Default colors and attributes
    default_fg: Option<Rgba>,
    default_bg: Option<Rgba>,
    default_attr: Option<u8>,
}

impl TextBuffer {
    fn new(capacity: u32) -> Self {
        Self {
            chars: Vec::with_capacity(capacity as usize),
            fg_colors: Vec::with_capacity(capacity as usize),
            bg_colors: Vec::with_capacity(capacity as usize),
            attributes: Vec::with_capacity(capacity as usize),
            lines: Vec::new(),
            length: 0,
            capacity,
            selection: None,
            selection_fg: None,
            selection_bg: None,
            default_fg: None,
            default_bg: None,
            default_attr: None,
        }
    }
}

//
// TEXT BUFFER MANAGEMENT
//

/// Create a new text buffer.
///
/// `width_method` is accepted for compatibility and does not change the width
/// policy: text is painted using the crate's grapheme display width.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn createTextBuffer(length: u32, _width_method: u8) -> *mut RTuiTextBuffer {
    if length == 0 {
        return std::ptr::null_mut();
    }

    let text_buffer = TextBuffer::new(length);
    let raw = Box::into_raw(Box::new(text_buffer));
    if !text_buffer_tracker().register(raw) {
        unsafe {
            drop(Box::from_raw(raw));
        }
        return std::ptr::null_mut();
    }
    raw.cast::<RTuiTextBuffer>()
}

/// Destroy a text buffer
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn destroyTextBuffer(tb: *mut RTuiTextBuffer) {
    if tb.is_null() {
        return;
    }

    let tb_ptr = tb.cast::<TextBuffer>();
    if text_buffer_tracker().unregister(tb_ptr) {
        unsafe {
            let _ = Box::from_raw(tb_ptr);
        }
    }
}

/// Get direct pointer to character data
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferGetCharPtr(tb: *mut RTuiTextBuffer) -> *mut u32 {
    if tb.is_null() {
        return std::ptr::null_mut();
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };
    text_buffer.chars.as_mut_ptr()
}

/// Get text buffer length
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferGetLength(tb: *const RTuiTextBuffer) -> u32 {
    if tb.is_null() {
        return 0;
    }

    let text_buffer = unsafe { &*(tb as *const TextBuffer) };
    text_buffer.length
}

/// Get text buffer capacity
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferGetCapacity(tb: *const RTuiTextBuffer) -> u32 {
    if tb.is_null() {
        return 0;
    }

    let text_buffer = unsafe { &*(tb as *const TextBuffer) };
    text_buffer.capacity
}

/// Resize text buffer
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferResize(tb: *mut RTuiTextBuffer, new_length: u32) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    let retained = text_buffer.length.min(new_length) as usize;
    text_buffer.chars.truncate(retained);
    text_buffer.fg_colors.truncate(retained);
    text_buffer.bg_colors.truncate(retained);
    text_buffer.attributes.truncate(retained);
    text_buffer.chars.shrink_to(new_length as usize);
    text_buffer.fg_colors.shrink_to(new_length as usize);
    text_buffer.bg_colors.shrink_to(new_length as usize);
    text_buffer.attributes.shrink_to(new_length as usize);
    if new_length as usize > retained {
        let additional = new_length as usize - retained;
        text_buffer.chars.reserve_exact(additional);
        text_buffer.fg_colors.reserve_exact(additional);
        text_buffer.bg_colors.reserve_exact(additional);
        text_buffer.attributes.reserve_exact(additional);
    }
    text_buffer.length = retained as u32;
    text_buffer.capacity = new_length;
}

/// Reset text buffer
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferReset(tb: *mut RTuiTextBuffer) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    text_buffer.chars.clear();
    text_buffer.fg_colors.clear();
    text_buffer.bg_colors.clear();
    text_buffer.attributes.clear();
    text_buffer.lines.clear();
    text_buffer.length = 0;
    text_buffer.selection = None;
}

//
// TEXT OPERATIONS
//

/// Write a chunk of text
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle this library returned
/// and has not destroyed, and no other call may use it until this one
/// returns. `text_bytes` must be null or the start of as many readable bytes
/// as `text_len` says. `fg` must be null or four readable `f32` color
/// components. `bg` must be null or four readable `f32` color components.
/// `attr` must be null or a readable `u8`.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferWriteChunk(
    tb: *mut RTuiTextBuffer,
    text_bytes: *const u8,
    text_len: u32,
    fg: *const f32,
    bg: *const f32,
    attr: *const u8,
) -> u32 {
    if tb.is_null() || text_bytes.is_null() {
        return 0;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };
    let text_slice = unsafe { std::slice::from_raw_parts(text_bytes, text_len as usize) };

    let text_str = match std::str::from_utf8(text_slice) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let fg_color = if fg.is_null() {
        text_buffer
            .default_fg
            .unwrap_or(Rgba::new(1.0, 1.0, 1.0, 1.0))
    } else {
        super::lib::f32_ptr_to_rgba(fg)
    };

    let bg_color = if bg.is_null() {
        text_buffer
            .default_bg
            .unwrap_or(Rgba::new(0.0, 0.0, 0.0, 1.0))
    } else {
        super::lib::f32_ptr_to_rgba(bg)
    };

    let attribute = if attr.is_null() {
        text_buffer.default_attr.unwrap_or(0)
    } else {
        unsafe { *attr }
    };

    let mut chars_written = 0;

    for ch in text_str.chars() {
        if text_buffer.length < text_buffer.capacity {
            text_buffer.chars.push(ch as u32);
            text_buffer.fg_colors.push(fg_color);
            text_buffer.bg_colors.push(bg_color);
            text_buffer.attributes.push(attribute);
            text_buffer.length += 1;
            chars_written += 1;
        } else {
            break;
        }
    }

    chars_written
}

//
// SELECTION OPERATIONS
//

/// Set selection range
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
///
/// `bg_color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
///
/// `fg_color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferSetSelection(
    tb: *mut RTuiTextBuffer,
    start: u32,
    end: u32,
    bg_color: *const f32,
    fg_color: *const f32,
) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    text_buffer.selection = Some((start, end));

    if !bg_color.is_null() {
        text_buffer.selection_bg = Some(super::lib::f32_ptr_to_rgba(bg_color));
    }

    if !fg_color.is_null() {
        text_buffer.selection_fg = Some(super::lib::f32_ptr_to_rgba(fg_color));
    }
}

//
// RENDERING OPERATIONS
//

/// Paint complete clusters using their first scalar's style and selection.
/// The return value counts painted scalars, not clusters or occupied cells.
fn paint_text_buffer(
    text_buffer: &TextBuffer,
    surface: &mut Surface,
    x: u32,
    y: u32,
    max_width: u32,
) -> u32 {
    let (surface_width, surface_height) = surface.dims();
    let mut column = x as usize;
    let row = y as usize;
    if column >= surface_width || row >= surface_height {
        return 0;
    }
    let end = column.saturating_add(max_width as usize).min(surface_width);
    // Invalid scalar values from the writable character pointer become control
    // text, which set_grapheme rejects, while retaining scalar indices.
    let text: String = text_buffer.chars[..text_buffer.length as usize]
        .iter()
        .map(|code| char::from_u32(*code).unwrap_or('\0'))
        .collect();
    let mut scalar_index = 0;
    let mut scalars_painted = 0;
    for grapheme in text.graphemes(true) {
        let scalar_count = grapheme.chars().count();
        let index = scalar_index;
        scalar_index += scalar_count;
        let width = grapheme.width();
        if width > end.saturating_sub(column) {
            break;
        }
        let mut fg = text_buffer.fg_colors[index];
        let mut bg = text_buffer.bg_colors[index];
        if text_buffer
            .selection
            .is_some_and(|(start, end)| index >= start as usize && index < end as usize)
        {
            fg = text_buffer.selection_fg.unwrap_or(fg);
            bg = text_buffer.selection_bg.unwrap_or(bg);
        }
        if surface.set_grapheme(
            column,
            row,
            grapheme,
            Cell {
                fg,
                bg,
                attr: Attr::from_bits_truncate(text_buffer.attributes[index]),
                ..Cell::default()
            },
        ) {
            column += width;
            scalars_painted += scalar_count as u32;
        }
    }
    scalars_painted
}

/// Render text buffer to surface
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
///
/// `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn renderTextBufferToSurface(
    tb: *const RTuiTextBuffer,
    buffer: *mut RTuiBuffer,
    x: u32,
    y: u32,
    max_width: u32,
) -> u32 {
    if tb.is_null() || buffer.is_null() {
        return 0;
    }

    // Validate pointers before using
    if !super::pointer::validate_pointer::<TextBuffer>(tb as *const u8) {
        return 0;
    }
    if !super::pointer::validate_pointer::<Surface>(buffer as *const u8) {
        return 0;
    }

    let text_buffer = unsafe { &*(tb as *const TextBuffer) };
    let surface = unsafe { &mut *(buffer as *mut Surface) };

    paint_text_buffer(text_buffer, surface, x, y, max_width)
}

/// Render text buffer to renderer surface
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
///
/// `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn renderTextBufferToRenderer(
    tb: *const RTuiTextBuffer,
    renderer: *mut RTuiRenderer,
    x: u32,
    y: u32,
    max_width: u32,
) -> u32 {
    if tb.is_null() || renderer.is_null() {
        return 0;
    }

    // Validate pointers before using
    if !super::pointer::validate_pointer::<TextBuffer>(tb as *const u8) {
        return 0;
    }
    if !super::pointer::validate_pointer::<crate::core::renderer::Renderer>(renderer as *const u8) {
        return 0;
    }

    let text_buffer = unsafe { &*(tb as *const TextBuffer) };
    let renderer_ref = unsafe { &mut *(renderer as *mut crate::core::renderer::Renderer) };

    // Get the renderer's surface
    let surface = renderer_ref.surface_mut();

    paint_text_buffer(text_buffer, surface, x, y, max_width)
}

/// Integrated text rendering with automatic surface management
///
/// Creates a temporary surface, renders text to it, then renders to terminal
/// This provides a complete TextBuffer → Surface → Renderer → Terminal pipeline
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
///
/// `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn renderTextBufferDirect(
    tb: *const RTuiTextBuffer,
    terminal: *mut super::terminal::RTuiTerminal,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> bool {
    if tb.is_null() || terminal.is_null() {
        return false;
    }

    // The integrated path calls renderTextBufferToSurface and its grapheme painter.
    super::lib::renderTextToTerminal(tb, terminal, x, y, width, height)
}

/// Reset selection
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferResetSelection(tb: *mut RTuiTextBuffer) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };
    text_buffer.selection = None;
    text_buffer.selection_fg = None;
    text_buffer.selection_bg = None;
}

/// Get selection info as packed u64: `[start:u32][end:u32]`.
/// Returns 0xFFFFFFFF_FFFFFFFF if no selection
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferGetSelectionInfo(tb: *const RTuiTextBuffer) -> u64 {
    if tb.is_null() {
        return 0xFFFFFFFF_FFFFFFFF;
    }

    let text_buffer = unsafe { &*(tb as *const TextBuffer) };

    match text_buffer.selection {
        Some((start, end)) => ((start as u64) << 32) | (end as u64),
        None => 0xFFFFFFFF_FFFFFFFF,
    }
}

//
// DEFAULT STYLING
//

/// Set default foreground color
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
///
/// `fg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferSetDefaultFg(tb: *mut RTuiTextBuffer, fg: *const f32) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    if fg.is_null() {
        text_buffer.default_fg = None;
    } else {
        text_buffer.default_fg = Some(super::lib::f32_ptr_to_rgba(fg));
    }
}

/// Set default background color
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
///
/// `bg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferSetDefaultBg(tb: *mut RTuiTextBuffer, bg: *const f32) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    if bg.is_null() {
        text_buffer.default_bg = None;
    } else {
        text_buffer.default_bg = Some(super::lib::f32_ptr_to_rgba(bg));
    }
}

/// Set default attributes
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle this library returned
/// and has not destroyed, and no other call may use it until this one
/// returns. `attr` must be null or a readable `u8`.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferSetDefaultAttributes(tb: *mut RTuiTextBuffer, attr: *const u8) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };

    if attr.is_null() {
        text_buffer.default_attr = None;
    } else {
        text_buffer.default_attr = Some(unsafe { *attr });
    }
}

/// Reset all defaults
///
/// # Safety
///
/// `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
#[reactive_tui_macros::ffi_export]
pub unsafe extern "C" fn textBufferResetDefaults(tb: *mut RTuiTextBuffer) {
    if tb.is_null() {
        return;
    }

    let text_buffer = unsafe { &mut *(tb as *mut TextBuffer) };
    text_buffer.default_fg = None;
    text_buffer.default_bg = None;
    text_buffer.default_attr = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(text: &str) -> TextBuffer {
        let mut buffer = TextBuffer::new(text.chars().count() as u32);
        for ch in text.chars() {
            buffer.chars.push(ch as u32);
            buffer.fg_colors.push(Rgba::new(1.0, 1.0, 1.0, 1.0));
            buffer.bg_colors.push(Rgba::new(0.0, 0.0, 0.0, 1.0));
            buffer.attributes.push(0);
            buffer.length += 1;
        }
        buffer
    }

    #[test]
    fn grapheme_painter_keeps_scalar_selection_and_counts() {
        let mut text = buffer("界e\u{301}A");
        let selected = Rgba::new(1.0, 0.0, 0.0, 1.0);
        let first_style = Rgba::new(0.0, 1.0, 0.0, 1.0);
        text.fg_colors[1] = first_style;
        text.fg_colors[2] = selected;
        text.selection = Some((3, 4));
        text.selection_fg = Some(selected);
        let mut surface = Surface::new(6, 1);
        assert_eq!(paint_text_buffer(&text, &mut surface, 1, 0, 4), 4);
        assert_eq!(surface.grapheme(1, 0), "界");
        assert_eq!(surface.grapheme(2, 0), "");
        assert_eq!(surface.grapheme(3, 0), "e\u{301}");
        assert_eq!(surface.get(3, 0).fg, first_style);
        assert_eq!(surface.get(4, 0).fg, selected);
    }

    #[test]
    fn grapheme_painter_keeps_joined_emoji_and_clips_whole_clusters() {
        let text = buffer("👩\u{200d}💻A");
        let mut surface = Surface::new(4, 1);
        assert_eq!(paint_text_buffer(&text, &mut surface, 0, 0, 3), 4);
        assert_eq!(surface.grapheme(0, 0), "👩\u{200d}💻");
        assert_eq!(surface.grapheme(1, 0), "");
        assert_eq!(surface.get(2, 0).ch, 'A');
        let mut clipped = Surface::new(4, 1);
        assert_eq!(paint_text_buffer(&text, &mut clipped, 0, 0, 1), 0);
        assert!(clipped.get(0, 0) == Cell::default());
        assert_eq!(paint_text_buffer(&text, &mut clipped, 3, 0, u32::MAX), 0);
        assert_eq!(paint_text_buffer(&text, &mut clipped, 0, 1, 4), 0);
    }
}
