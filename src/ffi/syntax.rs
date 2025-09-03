//! Syntax highlighting FFI functions

use super::*;
use crate::syntax::{SyntaxHighlighter, Language, Theme as SyntaxTheme, HighlightedSpan, TokenType};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to a syntax highlighter
#[repr(C)]
pub struct RTuiSyntaxHighlighter {
    _private: [u8; 0],
}

/// Opaque handle to a syntax theme
#[repr(C)]
pub struct RTuiSyntaxTheme {
    _private: [u8; 0],
}

/// Programming language enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiLanguage {
    Rust = 0,
    JavaScript = 1,
    TypeScript = 2,
    Python = 3,
    C = 4,
    Cpp = 5,
    Java = 6,
    Go = 7,
    Html = 8,
    Css = 9,
    Json = 10,
    Xml = 11,
    Yaml = 12,
    Toml = 13,
    Markdown = 14,
    Bash = 15,
    Sql = 16,
    PlainText = 17,
}

impl From<RTuiLanguage> for Language {
    fn from(lang: RTuiLanguage) -> Self {
        match lang {
            RTuiLanguage::Rust => Language::Rust,
            RTuiLanguage::JavaScript => Language::JavaScript,
            RTuiLanguage::TypeScript => Language::TypeScript,
            RTuiLanguage::Python => Language::Python,
            RTuiLanguage::C => Language::C,
            RTuiLanguage::Cpp => Language::Cpp,
            RTuiLanguage::Java => Language::Java,
            RTuiLanguage::Go => Language::Go,
            RTuiLanguage::Html => Language::Html,
            RTuiLanguage::Css => Language::Css,
            RTuiLanguage::Json => Language::Json,
            RTuiLanguage::Xml => Language::Xml,
            RTuiLanguage::Yaml => Language::Yaml,
            RTuiLanguage::Toml => Language::Toml,
            RTuiLanguage::Markdown => Language::Markdown,
            RTuiLanguage::Bash => Language::Bash,
            RTuiLanguage::Sql => Language::Sql,
            RTuiLanguage::PlainText => Language::PlainText,
        }
    }
}

/// Token type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiTokenType {
    Text = 0,
    Keyword = 1,
    String = 2,
    Comment = 3,
    Number = 4,
    Identifier = 5,
    Operator = 6,
    Punctuation = 7,
    Type = 8,
    Function = 9,
    Variable = 10,
    Constant = 11,
    Property = 12,
    Tag = 13,
    Attribute = 14,
    Error = 15,
}

impl From<TokenType> for RTuiTokenType {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Text => RTuiTokenType::Text,
            TokenType::Keyword => RTuiTokenType::Keyword,
            TokenType::String => RTuiTokenType::String,
            TokenType::Comment => RTuiTokenType::Comment,
            TokenType::Number => RTuiTokenType::Number,
            TokenType::Identifier => RTuiTokenType::Identifier,
            TokenType::Operator => RTuiTokenType::Operator,
            TokenType::Punctuation => RTuiTokenType::Punctuation,
            TokenType::Type => RTuiTokenType::Type,
            TokenType::Function => RTuiTokenType::Function,
            TokenType::Variable => RTuiTokenType::Variable,
            TokenType::Constant => RTuiTokenType::Constant,
            TokenType::Property => RTuiTokenType::Property,
            TokenType::Tag => RTuiTokenType::Tag,
            TokenType::Attribute => RTuiTokenType::Attribute,
            TokenType::Error => RTuiTokenType::Error,
        }
    }
}

/// Highlighted span structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiHighlightedSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: RTuiTokenType,
    pub color: RTuiColor,
}

impl From<HighlightedSpan> for RTuiHighlightedSpan {
    fn from(span: HighlightedSpan) -> Self {
        RTuiHighlightedSpan {
            start: span.start,
            end: span.end,
            token_type: span.token_type.into(),
            color: RTuiColor {
                r: span.color.r,
                g: span.color.g,
                b: span.color.b,
            },
        }
    }
}

/// Create a syntax highlighter
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_create(
    language: RTuiLanguage,
    out_highlighter: *mut *mut RTuiSyntaxHighlighter,
) -> ReactiveError {
    if out_highlighter.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let highlighter = SyntaxHighlighter::new(language.into());
        let boxed = Box::new(highlighter);
        unsafe {
            *out_highlighter = Box::into_raw(boxed) as *mut RTuiSyntaxHighlighter;
        }
        Ok(())
    }))
}

/// Destroy a syntax highlighter
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_destroy(highlighter: *mut RTuiSyntaxHighlighter) {
    if !highlighter.is_null() {
        unsafe {
            let _ = Box::from_raw(highlighter as *mut SyntaxHighlighter);
        }
    }
}

/// Set language for syntax highlighter
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_set_language(
    highlighter: *mut RTuiSyntaxHighlighter,
    language: RTuiLanguage,
) -> ReactiveError {
    if highlighter.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let highlighter_ref = &mut *(highlighter as *mut SyntaxHighlighter);
        highlighter_ref.set_language(language.into());
        Ok(())
    }))
}

/// Get language from syntax highlighter
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_get_language(
    highlighter: *const RTuiSyntaxHighlighter,
    out_language: *mut RTuiLanguage,
) -> ReactiveError {
    if highlighter.is_null() || out_language.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let highlighter_ref = &*(highlighter as *const SyntaxHighlighter);
        let language = highlighter_ref.language();
        *out_language = match language {
            Language::Rust => RTuiLanguage::Rust,
            Language::JavaScript => RTuiLanguage::JavaScript,
            Language::TypeScript => RTuiLanguage::TypeScript,
            Language::Python => RTuiLanguage::Python,
            Language::C => RTuiLanguage::C,
            Language::Cpp => RTuiLanguage::Cpp,
            Language::Java => RTuiLanguage::Java,
            Language::Go => RTuiLanguage::Go,
            Language::Html => RTuiLanguage::Html,
            Language::Css => RTuiLanguage::Css,
            Language::Json => RTuiLanguage::Json,
            Language::Xml => RTuiLanguage::Xml,
            Language::Yaml => RTuiLanguage::Yaml,
            Language::Toml => RTuiLanguage::Toml,
            Language::Markdown => RTuiLanguage::Markdown,
            Language::Bash => RTuiLanguage::Bash,
            Language::Sql => RTuiLanguage::Sql,
            Language::PlainText => RTuiLanguage::PlainText,
        };
        Ok(())
    }))
}

/// Highlight text and return spans
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_highlight(
    highlighter: *const RTuiSyntaxHighlighter,
    text: *const c_char,
    out_spans: *mut *mut RTuiHighlightedSpan,
    out_span_count: *mut usize,
) -> ReactiveError {
    if highlighter.is_null() || text.is_null() || out_spans.is_null() || out_span_count.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let text_str = CStr::from_ptr(text)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let highlighter_ref = &*(highlighter as *const SyntaxHighlighter);
        let spans = highlighter_ref.highlight(text_str);

        // Convert spans to FFI format
        let mut ffi_spans: Vec<RTuiHighlightedSpan> = spans
            .into_iter()
            .map(|span| span.into())
            .collect();

        *out_span_count = ffi_spans.len();
        if ffi_spans.is_empty() {
            *out_spans = std::ptr::null_mut();
        } else {
            let spans_ptr = ffi_spans.as_mut_ptr();
            std::mem::forget(ffi_spans); // Prevent deallocation
            *out_spans = spans_ptr;
        }

        Ok(())
    }))
}

/// Free highlighted spans array
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_free_spans(
    spans: *mut RTuiHighlightedSpan,
    span_count: usize,
) {
    if !spans.is_null() && span_count > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(spans, span_count, span_count);
        }
    }
}

/// Create a syntax theme
#[no_mangle]
pub extern "C" fn rtui_syntax_theme_create(
    name: *const c_char,
    out_theme: *mut *mut RTuiSyntaxTheme,
) -> ReactiveError {
    if name.is_null() || out_theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let theme = SyntaxTheme::new(name_str);
        *out_theme = Box::into_raw(Box::new(theme)) as *mut RTuiSyntaxTheme;
        Ok(())
    }))
}

/// Destroy a syntax theme
#[no_mangle]
pub extern "C" fn rtui_syntax_theme_destroy(theme: *mut RTuiSyntaxTheme) {
    if !theme.is_null() {
        unsafe {
            let _ = Box::from_raw(theme as *mut SyntaxTheme);
        }
    }
}

/// Set color for token type in syntax theme
#[no_mangle]
pub extern "C" fn rtui_syntax_theme_set_color(
    theme: *mut RTuiSyntaxTheme,
    token_type: RTuiTokenType,
    color: RTuiColor,
) -> ReactiveError {
    if theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut SyntaxTheme);
        let rust_token_type = match token_type {
            RTuiTokenType::Text => TokenType::Text,
            RTuiTokenType::Keyword => TokenType::Keyword,
            RTuiTokenType::String => TokenType::String,
            RTuiTokenType::Comment => TokenType::Comment,
            RTuiTokenType::Number => TokenType::Number,
            RTuiTokenType::Identifier => TokenType::Identifier,
            RTuiTokenType::Operator => TokenType::Operator,
            RTuiTokenType::Punctuation => TokenType::Punctuation,
            RTuiTokenType::Type => TokenType::Type,
            RTuiTokenType::Function => TokenType::Function,
            RTuiTokenType::Variable => TokenType::Variable,
            RTuiTokenType::Constant => TokenType::Constant,
            RTuiTokenType::Property => TokenType::Property,
            RTuiTokenType::Tag => TokenType::Tag,
            RTuiTokenType::Attribute => TokenType::Attribute,
            RTuiTokenType::Error => TokenType::Error,
        };

        let rust_color = crate::syntax::Color {
            r: color.r,
            g: color.g,
            b: color.b,
        };

        theme_ref.set_color(rust_token_type, rust_color);
        Ok(())
    }))
}

/// Get color for token type from syntax theme
#[no_mangle]
pub extern "C" fn rtui_syntax_theme_get_color(
    theme: *const RTuiSyntaxTheme,
    token_type: RTuiTokenType,
    out_color: *mut RTuiColor,
) -> ReactiveError {
    if theme.is_null() || out_color.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &*(theme as *const SyntaxTheme);
        let rust_token_type = match token_type {
            RTuiTokenType::Text => TokenType::Text,
            RTuiTokenType::Keyword => TokenType::Keyword,
            RTuiTokenType::String => TokenType::String,
            RTuiTokenType::Comment => TokenType::Comment,
            RTuiTokenType::Number => TokenType::Number,
            RTuiTokenType::Identifier => TokenType::Identifier,
            RTuiTokenType::Operator => TokenType::Operator,
            RTuiTokenType::Punctuation => TokenType::Punctuation,
            RTuiTokenType::Type => TokenType::Type,
            RTuiTokenType::Function => TokenType::Function,
            RTuiTokenType::Variable => TokenType::Variable,
            RTuiTokenType::Constant => TokenType::Constant,
            RTuiTokenType::Property => TokenType::Property,
            RTuiTokenType::Tag => TokenType::Tag,
            RTuiTokenType::Attribute => TokenType::Attribute,
            RTuiTokenType::Error => TokenType::Error,
        };

        let color = theme_ref.get_color(rust_token_type);
        *out_color = RTuiColor {
            r: color.r,
            g: color.g,
            b: color.b,
        };
        Ok(())
    }))
}

/// Set syntax theme for highlighter
#[no_mangle]
pub extern "C" fn rtui_syntax_highlighter_set_theme(
    highlighter: *mut RTuiSyntaxHighlighter,
    theme: *mut RTuiSyntaxTheme,
) -> ReactiveError {
    if highlighter.is_null() || theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let highlighter_ref = &mut *(highlighter as *mut SyntaxHighlighter);
        let theme_box = Box::from_raw(theme as *mut SyntaxTheme);
        highlighter_ref.set_theme(*theme_box);
        Ok(())
    }))
}

/// Detect language from file extension
#[no_mangle]
pub extern "C" fn rtui_syntax_detect_language_from_extension(
    extension: *const c_char,
    out_language: *mut RTuiLanguage,
) -> ReactiveError {
    if extension.is_null() || out_language.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let ext_str = CStr::from_ptr(extension)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let language = Language::from_extension(ext_str);
        *out_language = match language {
            Language::Rust => RTuiLanguage::Rust,
            Language::JavaScript => RTuiLanguage::JavaScript,
            Language::TypeScript => RTuiLanguage::TypeScript,
            Language::Python => RTuiLanguage::Python,
            Language::C => RTuiLanguage::C,
            Language::Cpp => RTuiLanguage::Cpp,
            Language::Java => RTuiLanguage::Java,
            Language::Go => RTuiLanguage::Go,
            Language::Html => RTuiLanguage::Html,
            Language::Css => RTuiLanguage::Css,
            Language::Json => RTuiLanguage::Json,
            Language::Xml => RTuiLanguage::Xml,
            Language::Yaml => RTuiLanguage::Yaml,
            Language::Toml => RTuiLanguage::Toml,
            Language::Markdown => RTuiLanguage::Markdown,
            Language::Bash => RTuiLanguage::Bash,
            Language::Sql => RTuiLanguage::Sql,
            Language::PlainText => RTuiLanguage::PlainText,
        };
        Ok(())
    }))
}
