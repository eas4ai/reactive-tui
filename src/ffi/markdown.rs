//! Markdown rendering FFI functions

use super::*;
use crate::markdown::{MarkdownRenderer, MarkdownConfig, MarkdownTheme, RenderOptions};
use crate::component::Element;
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to a markdown renderer
#[repr(C)]
pub struct RTuiMarkdownRenderer {
    _private: [u8; 0],
}

/// Opaque handle to markdown config
#[repr(C)]
pub struct RTuiMarkdownConfig {
    _private: [u8; 0],
}

/// Opaque handle to markdown theme
#[repr(C)]
pub struct RTuiMarkdownTheme {
    _private: [u8; 0],
}

/// Markdown rendering options
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiMarkdownOptions {
    pub enable_syntax_highlighting: bool,
    pub enable_tables: bool,
    pub enable_strikethrough: bool,
    pub enable_task_lists: bool,
    pub enable_footnotes: bool,
    pub enable_smart_punctuation: bool,
    pub max_width: u16,
    pub tab_size: u8,
}

impl From<RTuiMarkdownOptions> for RenderOptions {
    fn from(options: RTuiMarkdownOptions) -> Self {
        RenderOptions {
            enable_syntax_highlighting: options.enable_syntax_highlighting,
            enable_tables: options.enable_tables,
            enable_strikethrough: options.enable_strikethrough,
            enable_task_lists: options.enable_task_lists,
            enable_footnotes: options.enable_footnotes,
            enable_smart_punctuation: options.enable_smart_punctuation,
            max_width: options.max_width,
            tab_size: options.tab_size,
        }
    }
}

/// Markdown heading level enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiMarkdownHeadingLevel {
    H1 = 1,
    H2 = 2,
    H3 = 3,
    H4 = 4,
    H5 = 5,
    H6 = 6,
}

/// Markdown list type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiMarkdownListType {
    Unordered = 0,
    Ordered = 1,
    TaskList = 2,
}

/// Create markdown configuration
#[no_mangle]
pub extern "C" fn rtui_markdown_config_create(
    out_config: *mut *mut RTuiMarkdownConfig,
) -> ReactiveError {
    if out_config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let config = MarkdownConfig::default();
        let boxed = Box::new(config);
        unsafe {
            *out_config = Box::into_raw(boxed) as *mut RTuiMarkdownConfig;
        }
        Ok(())
    }))
}

/// Destroy markdown configuration
#[no_mangle]
pub extern "C" fn rtui_markdown_config_destroy(config: *mut RTuiMarkdownConfig) {
    if !config.is_null() {
        unsafe {
            let _ = Box::from_raw(config as *mut MarkdownConfig);
        }
    }
}

/// Set syntax highlighting in markdown config
#[no_mangle]
pub extern "C" fn rtui_markdown_config_set_syntax_highlighting(
    config: *mut RTuiMarkdownConfig,
    enabled: bool,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut MarkdownConfig);
        config_ref.set_syntax_highlighting(enabled);
        Ok(())
    }))
}

/// Set table support in markdown config
#[no_mangle]
pub extern "C" fn rtui_markdown_config_set_tables(
    config: *mut RTuiMarkdownConfig,
    enabled: bool,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut MarkdownConfig);
        config_ref.set_tables(enabled);
        Ok(())
    }))
}

/// Set task list support in markdown config
#[no_mangle]
pub extern "C" fn rtui_markdown_config_set_task_lists(
    config: *mut RTuiMarkdownConfig,
    enabled: bool,
) -> ReactiveError {
    if config.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = &mut *(config as *mut MarkdownConfig);
        config_ref.set_task_lists(enabled);
        Ok(())
    }))
}

/// Create markdown theme
#[no_mangle]
pub extern "C" fn rtui_markdown_theme_create(
    name: *const c_char,
    out_theme: *mut *mut RTuiMarkdownTheme,
) -> ReactiveError {
    if name.is_null() || out_theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let theme = MarkdownTheme::new(name_str);
        *out_theme = Box::into_raw(Box::new(theme)) as *mut RTuiMarkdownTheme;
        Ok(())
    }))
}

/// Destroy markdown theme
#[no_mangle]
pub extern "C" fn rtui_markdown_theme_destroy(theme: *mut RTuiMarkdownTheme) {
    if !theme.is_null() {
        unsafe {
            let _ = Box::from_raw(theme as *mut MarkdownTheme);
        }
    }
}

/// Set heading color in markdown theme
#[no_mangle]
pub extern "C" fn rtui_markdown_theme_set_heading_color(
    theme: *mut RTuiMarkdownTheme,
    level: RTuiMarkdownHeadingLevel,
    color: RTuiColor,
) -> ReactiveError {
    if theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut MarkdownTheme);
        let level_num = level as u8;
        let color_tuple = (color.r, color.g, color.b);
        theme_ref.set_heading_color(level_num, color_tuple);
        Ok(())
    }))
}

/// Set code block background color in markdown theme
#[no_mangle]
pub extern "C" fn rtui_markdown_theme_set_code_background(
    theme: *mut RTuiMarkdownTheme,
    color: RTuiColor,
) -> ReactiveError {
    if theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut MarkdownTheme);
        let color_tuple = (color.r, color.g, color.b);
        theme_ref.set_code_background(color_tuple);
        Ok(())
    }))
}

/// Set quote border color in markdown theme
#[no_mangle]
pub extern "C" fn rtui_markdown_theme_set_quote_border(
    theme: *mut RTuiMarkdownTheme,
    color: RTuiColor,
) -> ReactiveError {
    if theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut MarkdownTheme);
        let color_tuple = (color.r, color.g, color.b);
        theme_ref.set_quote_border(color_tuple);
        Ok(())
    }))
}

/// Create markdown renderer
#[no_mangle]
pub extern "C" fn rtui_markdown_renderer_create(
    config: *const RTuiMarkdownConfig,
    theme: *const RTuiMarkdownTheme,
    out_renderer: *mut *mut RTuiMarkdownRenderer,
) -> ReactiveError {
    if out_renderer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let config_ref = if config.is_null() {
            MarkdownConfig::default()
        } else {
            (*(config as *const MarkdownConfig)).clone()
        };

        let theme_ref = if theme.is_null() {
            MarkdownTheme::default()
        } else {
            (*(theme as *const MarkdownTheme)).clone()
        };

        let renderer = MarkdownRenderer::new(config_ref, theme_ref);
        *out_renderer = Box::into_raw(Box::new(renderer)) as *mut RTuiMarkdownRenderer;
        Ok(())
    }))
}

/// Destroy markdown renderer
#[no_mangle]
pub extern "C" fn rtui_markdown_renderer_destroy(renderer: *mut RTuiMarkdownRenderer) {
    if !renderer.is_null() {
        unsafe {
            let _ = Box::from_raw(renderer as *mut MarkdownRenderer);
        }
    }
}

/// Render markdown text to element
#[no_mangle]
pub extern "C" fn rtui_markdown_render(
    renderer: *const RTuiMarkdownRenderer,
    markdown_text: *const c_char,
    options: RTuiMarkdownOptions,
    out_element: *mut *mut super::builder::RTuiElement,
) -> ReactiveError {
    if renderer.is_null() || markdown_text.is_null() || out_element.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(markdown_text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let renderer_ref = &*(renderer as *const MarkdownRenderer);
        let render_options = options.into();
        let element = renderer_ref.render(text_str, render_options);
        *out_element = Box::into_raw(Box::new(element)) as *mut super::builder::RTuiElement;
        Ok(())
    }))
}

/// Render markdown text to string (for debugging)
#[no_mangle]
pub extern "C" fn rtui_markdown_render_to_string(
    renderer: *const RTuiMarkdownRenderer,
    markdown_text: *const c_char,
    options: RTuiMarkdownOptions,
    out_string: *mut *mut c_char,
) -> ReactiveError {
    if renderer.is_null() || markdown_text.is_null() || out_string.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(markdown_text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let renderer_ref = &*(renderer as *const MarkdownRenderer);
        let render_options = options.into();
        let rendered_string = renderer_ref.render_to_string(text_str, render_options);
        
        let c_string = CString::new(rendered_string).map_err(|_| ReactiveError::InvalidUtf8)?;
        *out_string = c_string.into_raw();
        Ok(())
    }))
}

/// Parse markdown and get table of contents
#[no_mangle]
pub extern "C" fn rtui_markdown_get_table_of_contents(
    renderer: *const RTuiMarkdownRenderer,
    markdown_text: *const c_char,
    out_headings: *mut *mut *mut c_char,
    out_levels: *mut *mut u8,
    out_count: *mut usize,
) -> ReactiveError {
    if renderer.is_null() || markdown_text.is_null() || out_headings.is_null() || out_levels.is_null() || out_count.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(markdown_text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let renderer_ref = &*(renderer as *const MarkdownRenderer);
        let toc = renderer_ref.get_table_of_contents(text_str);

        if toc.is_empty() {
            *out_headings = std::ptr::null_mut();
            *out_levels = std::ptr::null_mut();
            *out_count = 0;
            return Ok(());
        }

        // Allocate arrays for headings and levels
        let mut heading_ptrs: Vec<*mut c_char> = Vec::with_capacity(toc.len());
        let mut levels: Vec<u8> = Vec::with_capacity(toc.len());

        for (heading, level) in toc {
            let c_string = CString::new(heading).map_err(|_| ReactiveError::InvalidUtf8)?;
            heading_ptrs.push(c_string.into_raw());
            levels.push(level);
        }

        // Transfer ownership to C
        *out_count = heading_ptrs.len();
        
        let headings_ptr = heading_ptrs.as_mut_ptr();
        std::mem::forget(heading_ptrs);
        *out_headings = headings_ptr;

        let levels_ptr = levels.as_mut_ptr();
        std::mem::forget(levels);
        *out_levels = levels_ptr;

        Ok(())
    }))
}

/// Free table of contents arrays
#[no_mangle]
pub extern "C" fn rtui_markdown_free_table_of_contents(
    headings: *mut *mut c_char,
    levels: *mut u8,
    count: usize,
) {
    if !headings.is_null() && count > 0 {
        unsafe {
            let heading_vec = Vec::from_raw_parts(headings, count, count);
            for heading_ptr in heading_vec {
                if !heading_ptr.is_null() {
                    let _ = CString::from_raw(heading_ptr);
                }
            }
        }
    }
    
    if !levels.is_null() && count > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(levels, count, count);
        }
    }
}

/// Validate markdown syntax
#[no_mangle]
pub extern "C" fn rtui_markdown_validate(
    markdown_text: *const c_char,
    out_is_valid: *mut bool,
    out_error_message: *mut *mut c_char,
) -> ReactiveError {
    if markdown_text.is_null() || out_is_valid.is_null() || out_error_message.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(markdown_text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        match MarkdownRenderer::validate(text_str) {
            Ok(()) => {
                *out_is_valid = true;
                *out_error_message = std::ptr::null_mut();
            }
            Err(error) => {
                *out_is_valid = false;
                let error_string = error.to_string();
                let c_string = CString::new(error_string).map_err(|_| ReactiveError::InvalidUtf8)?;
                *out_error_message = c_string.into_raw();
            }
        }
        Ok(())
    }))
}

/// Convert markdown to HTML (for export)
#[no_mangle]
pub extern "C" fn rtui_markdown_to_html(
    renderer: *const RTuiMarkdownRenderer,
    markdown_text: *const c_char,
    out_html: *mut *mut c_char,
) -> ReactiveError {
    if renderer.is_null() || markdown_text.is_null() || out_html.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(markdown_text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let renderer_ref = &*(renderer as *const MarkdownRenderer);
        let html_string = renderer_ref.to_html(text_str);
        
        let c_string = CString::new(html_string).map_err(|_| ReactiveError::InvalidUtf8)?;
        *out_html = c_string.into_raw();
        Ok(())
    }))
}
