//! Theme system FFI functions

use super::*;
use crate::theme::{Theme, ThemeManager, ColorScheme, ThemeVariant};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::collections::HashMap;

/// Opaque handle to a theme
#[repr(C)]
pub struct RTuiTheme {
    _private: [u8; 0],
}

/// Opaque handle to a theme manager
#[repr(C)]
pub struct RTuiThemeManager {
    _private: [u8; 0],
}

/// Opaque handle to a color scheme
#[repr(C)]
pub struct RTuiColorScheme {
    _private: [u8; 0],
}

/// Theme variant enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiThemeVariant {
    Light = 0,
    Dark = 1,
    Auto = 2,
    HighContrast = 3,
    Custom = 4,
}

impl From<RTuiThemeVariant> for ThemeVariant {
    fn from(variant: RTuiThemeVariant) -> Self {
        match variant {
            RTuiThemeVariant::Light => ThemeVariant::Light,
            RTuiThemeVariant::Dark => ThemeVariant::Dark,
            RTuiThemeVariant::Auto => ThemeVariant::Auto,
            RTuiThemeVariant::HighContrast => ThemeVariant::HighContrast,
            RTuiThemeVariant::Custom => ThemeVariant::Custom,
        }
    }
}

/// Color role enumeration for semantic colors
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiColorRole {
    Primary = 0,
    Secondary = 1,
    Success = 2,
    Warning = 3,
    Error = 4,
    Info = 5,
    Background = 6,
    Surface = 7,
    OnPrimary = 8,
    OnSecondary = 9,
    OnBackground = 10,
    OnSurface = 11,
    Border = 12,
    Text = 13,
    TextMuted = 14,
    TextDisabled = 15,
}

/// Theme color structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiThemeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Create a new theme manager
#[no_mangle]
pub extern "C" fn rtui_theme_manager_create(
    out_manager: *mut *mut RTuiThemeManager,
) -> ReactiveError {
    if out_manager.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let manager = ThemeManager::new();
        let boxed = Box::new(manager);
        unsafe {
            *out_manager = Box::into_raw(boxed) as *mut RTuiThemeManager;
        }
        Ok(())
    }))
}

/// Destroy a theme manager
#[no_mangle]
pub extern "C" fn rtui_theme_manager_destroy(manager: *mut RTuiThemeManager) {
    if !manager.is_null() {
        unsafe {
            let _ = Box::from_raw(manager as *mut ThemeManager);
        }
    }
}

/// Set active theme variant
#[no_mangle]
pub extern "C" fn rtui_theme_manager_set_variant(
    manager: *mut RTuiThemeManager,
    variant: RTuiThemeVariant,
) -> ReactiveError {
    if manager.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &mut *(manager as *mut ThemeManager);
        manager_ref.set_variant(variant.into());
        Ok(())
    }))
}

/// Get active theme variant
#[no_mangle]
pub extern "C" fn rtui_theme_manager_get_variant(
    manager: *const RTuiThemeManager,
    out_variant: *mut RTuiThemeVariant,
) -> ReactiveError {
    if manager.is_null() || out_variant.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &*(manager as *const ThemeManager);
        let variant = manager_ref.get_variant();
        *out_variant = match variant {
            ThemeVariant::Light => RTuiThemeVariant::Light,
            ThemeVariant::Dark => RTuiThemeVariant::Dark,
            ThemeVariant::Auto => RTuiThemeVariant::Auto,
            ThemeVariant::HighContrast => RTuiThemeVariant::HighContrast,
            ThemeVariant::Custom => RTuiThemeVariant::Custom,
        };
        Ok(())
    }))
}

/// Create a new theme
#[no_mangle]
pub extern "C" fn rtui_theme_create(
    name: *const c_char,
    variant: RTuiThemeVariant,
    out_theme: *mut *mut RTuiTheme,
) -> ReactiveError {
    if name.is_null() || out_theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let theme = Theme::new(name_str, variant.into());
        *out_theme = Box::into_raw(Box::new(theme)) as *mut RTuiTheme;
        Ok(())
    }))
}

/// Destroy a theme
#[no_mangle]
pub extern "C" fn rtui_theme_destroy(theme: *mut RTuiTheme) {
    if !theme.is_null() {
        unsafe {
            let _ = Box::from_raw(theme as *mut Theme);
        }
    }
}

/// Set theme color for a specific role
#[no_mangle]
pub extern "C" fn rtui_theme_set_color(
    theme: *mut RTuiTheme,
    role: RTuiColorRole,
    color: RTuiThemeColor,
) -> ReactiveError {
    if theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut Theme);
        let color_tuple = (color.r, color.g, color.b, color.a);
        
        match role {
            RTuiColorRole::Primary => theme_ref.set_primary_color(color_tuple),
            RTuiColorRole::Secondary => theme_ref.set_secondary_color(color_tuple),
            RTuiColorRole::Success => theme_ref.set_success_color(color_tuple),
            RTuiColorRole::Warning => theme_ref.set_warning_color(color_tuple),
            RTuiColorRole::Error => theme_ref.set_error_color(color_tuple),
            RTuiColorRole::Info => theme_ref.set_info_color(color_tuple),
            RTuiColorRole::Background => theme_ref.set_background_color(color_tuple),
            RTuiColorRole::Surface => theme_ref.set_surface_color(color_tuple),
            RTuiColorRole::OnPrimary => theme_ref.set_on_primary_color(color_tuple),
            RTuiColorRole::OnSecondary => theme_ref.set_on_secondary_color(color_tuple),
            RTuiColorRole::OnBackground => theme_ref.set_on_background_color(color_tuple),
            RTuiColorRole::OnSurface => theme_ref.set_on_surface_color(color_tuple),
            RTuiColorRole::Border => theme_ref.set_border_color(color_tuple),
            RTuiColorRole::Text => theme_ref.set_text_color(color_tuple),
            RTuiColorRole::TextMuted => theme_ref.set_text_muted_color(color_tuple),
            RTuiColorRole::TextDisabled => theme_ref.set_text_disabled_color(color_tuple),
        }
        Ok(())
    }))
}

/// Get theme color for a specific role
#[no_mangle]
pub extern "C" fn rtui_theme_get_color(
    theme: *const RTuiTheme,
    role: RTuiColorRole,
    out_color: *mut RTuiThemeColor,
) -> ReactiveError {
    if theme.is_null() || out_color.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &*(theme as *const Theme);
        
        let color_tuple = match role {
            RTuiColorRole::Primary => theme_ref.primary_color(),
            RTuiColorRole::Secondary => theme_ref.secondary_color(),
            RTuiColorRole::Success => theme_ref.success_color(),
            RTuiColorRole::Warning => theme_ref.warning_color(),
            RTuiColorRole::Error => theme_ref.error_color(),
            RTuiColorRole::Info => theme_ref.info_color(),
            RTuiColorRole::Background => theme_ref.background_color(),
            RTuiColorRole::Surface => theme_ref.surface_color(),
            RTuiColorRole::OnPrimary => theme_ref.on_primary_color(),
            RTuiColorRole::OnSecondary => theme_ref.on_secondary_color(),
            RTuiColorRole::OnBackground => theme_ref.on_background_color(),
            RTuiColorRole::OnSurface => theme_ref.on_surface_color(),
            RTuiColorRole::Border => theme_ref.border_color(),
            RTuiColorRole::Text => theme_ref.text_color(),
            RTuiColorRole::TextMuted => theme_ref.text_muted_color(),
            RTuiColorRole::TextDisabled => theme_ref.text_disabled_color(),
        };

        *out_color = RTuiThemeColor {
            r: color_tuple.0,
            g: color_tuple.1,
            b: color_tuple.2,
            a: color_tuple.3,
        };
        Ok(())
    }))
}

/// Register a theme with the theme manager
#[no_mangle]
pub extern "C" fn rtui_theme_manager_register_theme(
    manager: *mut RTuiThemeManager,
    theme: *mut RTuiTheme,
) -> ReactiveError {
    if manager.is_null() || theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &mut *(manager as *mut ThemeManager);
        let theme_box = Box::from_raw(theme as *mut Theme);
        manager_ref.register_theme(*theme_box);
        Ok(())
    }))
}

/// Set active theme by name
#[no_mangle]
pub extern "C" fn rtui_theme_manager_set_active_theme(
    manager: *mut RTuiThemeManager,
    theme_name: *const c_char,
) -> ReactiveError {
    if manager.is_null() || theme_name.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(theme_name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let manager_ref = &mut *(manager as *mut ThemeManager);
        manager_ref.set_active_theme(name_str)
            .map_err(|_| ReactiveError::NotFound)?;
        Ok(())
    }))
}

/// Get active theme name
#[no_mangle]
pub extern "C" fn rtui_theme_manager_get_active_theme_name(
    manager: *const RTuiThemeManager,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if manager.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let manager_ref = &*(manager as *const ThemeManager);
        let theme_name = manager_ref.get_active_theme_name();
        
        if theme_name.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(theme_name).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Create a color scheme
#[no_mangle]
pub extern "C" fn rtui_color_scheme_create(
    name: *const c_char,
    out_scheme: *mut *mut RTuiColorScheme,
) -> ReactiveError {
    if name.is_null() || out_scheme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let name_str = CStr::from_ptr(name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let scheme = ColorScheme::new(name_str);
        *out_scheme = Box::into_raw(Box::new(scheme)) as *mut RTuiColorScheme;
        Ok(())
    }))
}

/// Destroy a color scheme
#[no_mangle]
pub extern "C" fn rtui_color_scheme_destroy(scheme: *mut RTuiColorScheme) {
    if !scheme.is_null() {
        unsafe {
            let _ = Box::from_raw(scheme as *mut ColorScheme);
        }
    }
}

/// Apply color scheme to theme
#[no_mangle]
pub extern "C" fn rtui_theme_apply_color_scheme(
    theme: *mut RTuiTheme,
    scheme: *const RTuiColorScheme,
) -> ReactiveError {
    if theme.is_null() || scheme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &mut *(theme as *mut Theme);
        let scheme_ref = &*(scheme as *const ColorScheme);
        theme_ref.apply_color_scheme(scheme_ref);
        Ok(())
    }))
}

/// Load theme from JSON string
#[no_mangle]
pub extern "C" fn rtui_theme_load_from_json(
    json_data: *const c_char,
    out_theme: *mut *mut RTuiTheme,
) -> ReactiveError {
    if json_data.is_null() || out_theme.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let json_str = CStr::from_ptr(json_data)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let theme = Theme::from_json(json_str)
            .map_err(|_| ReactiveError::InvalidParameter)?;
        *out_theme = Box::into_raw(Box::new(theme)) as *mut RTuiTheme;
        Ok(())
    }))
}

/// Save theme to JSON string
#[no_mangle]
pub extern "C" fn rtui_theme_save_to_json(
    theme: *const RTuiTheme,
    out_json: *mut *mut c_char,
) -> ReactiveError {
    if theme.is_null() || out_json.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let theme_ref = &*(theme as *const Theme);
        let json_string = theme_ref.to_json()
            .map_err(|_| ReactiveError::InternalError)?;

        let c_string = CString::new(json_string).map_err(|_| ReactiveError::InvalidUtf8)?;
        *out_json = c_string.into_raw();
        Ok(())
    }))
}
