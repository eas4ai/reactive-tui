//! Platform integration FFI functions

use super::*;
use crate::platform::{Platform, Capabilities, TerminalInfo, SystemInfo};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Opaque handle to platform instance
#[repr(C)]
pub struct RTuiPlatform {
    _private: [u8; 0],
}

/// Platform capabilities structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiCapabilities {
    pub true_color: bool,
    pub unicode: bool,
    pub mouse_support: bool,
    pub keyboard_enhancement: bool,
    pub synchronized_output: bool,
    pub bracketed_paste: bool,
    pub focus_events: bool,
    pub window_title: bool,
    pub cursor_shape: bool,
    pub hyperlinks: bool,
    pub images: bool,
    pub kitty_graphics: bool,
    pub sixel_graphics: bool,
    pub iterm2_images: bool,
}

impl From<Capabilities> for RTuiCapabilities {
    fn from(caps: Capabilities) -> Self {
        RTuiCapabilities {
            true_color: caps.true_color,
            unicode: caps.unicode,
            mouse_support: caps.mouse_support,
            keyboard_enhancement: caps.keyboard_enhancement,
            synchronized_output: caps.synchronized_output,
            bracketed_paste: caps.bracketed_paste,
            focus_events: caps.focus_events,
            window_title: caps.window_title,
            cursor_shape: caps.cursor_shape,
            hyperlinks: caps.hyperlinks,
            images: caps.images,
            kitty_graphics: caps.kitty_graphics,
            sixel_graphics: caps.sixel_graphics,
            iterm2_images: caps.iterm2_images,
        }
    }
}

/// Terminal information structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiTerminalInfo {
    pub width: u16,
    pub height: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
    pub cell_width: u8,
    pub cell_height: u8,
}

impl From<TerminalInfo> for RTuiTerminalInfo {
    fn from(info: TerminalInfo) -> Self {
        RTuiTerminalInfo {
            width: info.width,
            height: info.height,
            pixel_width: info.pixel_width,
            pixel_height: info.pixel_height,
            cell_width: info.cell_width,
            cell_height: info.cell_height,
        }
    }
}

/// System information structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiSystemInfo {
    pub os_type: RTuiOSType,
    pub cpu_count: u32,
    pub total_memory: u64,
    pub available_memory: u64,
}

/// Operating system type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiOSType {
    Windows = 0,
    MacOS = 1,
    Linux = 2,
    FreeBSD = 3,
    OpenBSD = 4,
    NetBSD = 5,
    Android = 6,
    iOS = 7,
    Unknown = 8,
}

impl From<SystemInfo> for RTuiSystemInfo {
    fn from(info: SystemInfo) -> Self {
        let os_type = match info.os_type.as_str() {
            "windows" => RTuiOSType::Windows,
            "macos" => RTuiOSType::MacOS,
            "linux" => RTuiOSType::Linux,
            "freebsd" => RTuiOSType::FreeBSD,
            "openbsd" => RTuiOSType::OpenBSD,
            "netbsd" => RTuiOSType::NetBSD,
            "android" => RTuiOSType::Android,
            "ios" => RTuiOSType::iOS,
            _ => RTuiOSType::Unknown,
        };

        RTuiSystemInfo {
            os_type,
            cpu_count: info.cpu_count,
            total_memory: info.total_memory,
            available_memory: info.available_memory,
        }
    }
}

/// Create platform instance
#[no_mangle]
pub extern "C" fn rtui_platform_create(out_platform: *mut *mut RTuiPlatform) -> ReactiveError {
    if out_platform.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let platform = Platform::new();
        let boxed = Box::new(platform);
        unsafe {
            *out_platform = Box::into_raw(boxed) as *mut RTuiPlatform;
        }
        Ok(())
    }))
}

/// Destroy platform instance
#[no_mangle]
pub extern "C" fn rtui_platform_destroy(platform: *mut RTuiPlatform) {
    if !platform.is_null() {
        unsafe {
            let _ = Box::from_raw(platform as *mut Platform);
        }
    }
}

/// Detect platform capabilities
#[no_mangle]
pub extern "C" fn rtui_platform_detect_capabilities(
    platform: *const RTuiPlatform,
    out_capabilities: *mut RTuiCapabilities,
) -> ReactiveError {
    if platform.is_null() || out_capabilities.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        let capabilities = platform_ref.detect_capabilities();
        *out_capabilities = capabilities.into();
        Ok(())
    }))
}

/// Get terminal information
#[no_mangle]
pub extern "C" fn rtui_platform_get_terminal_info(
    platform: *const RTuiPlatform,
    out_info: *mut RTuiTerminalInfo,
) -> ReactiveError {
    if platform.is_null() || out_info.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        let terminal_info = platform_ref.get_terminal_info();
        *out_info = terminal_info.into();
        Ok(())
    }))
}

/// Get system information
#[no_mangle]
pub extern "C" fn rtui_platform_get_system_info(
    platform: *const RTuiPlatform,
    out_info: *mut RTuiSystemInfo,
) -> ReactiveError {
    if platform.is_null() || out_info.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        let system_info = platform_ref.get_system_info();
        *out_info = system_info.into();
        Ok(())
    }))
}

/// Get terminal name/type
#[no_mangle]
pub extern "C" fn rtui_platform_get_terminal_name(
    platform: *const RTuiPlatform,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if platform.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        let terminal_name = platform_ref.get_terminal_name();
        
        if terminal_name.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(terminal_name).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Check if running in SSH session
#[no_mangle]
pub extern "C" fn rtui_platform_is_ssh_session(
    platform: *const RTuiPlatform,
    out_is_ssh: *mut bool,
) -> ReactiveError {
    if platform.is_null() || out_is_ssh.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        *out_is_ssh = platform_ref.is_ssh_session();
        Ok(())
    }))
}

/// Check if running in CI environment
#[no_mangle]
pub extern "C" fn rtui_platform_is_ci_environment(
    platform: *const RTuiPlatform,
    out_is_ci: *mut bool,
) -> ReactiveError {
    if platform.is_null() || out_is_ci.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        *out_is_ci = platform_ref.is_ci_environment();
        Ok(())
    }))
}

/// Get environment variable
#[no_mangle]
pub extern "C" fn rtui_platform_get_env_var(
    platform: *const RTuiPlatform,
    var_name: *const c_char,
    buffer: *mut c_char,
    buffer_size: usize,
    out_found: *mut bool,
) -> ReactiveError {
    if platform.is_null() || var_name.is_null() || buffer.is_null() || out_found.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let var_name_str = CStr::from_ptr(var_name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let platform_ref = &*(platform as *const Platform);
        if let Some(value) = platform_ref.get_env_var(var_name_str) {
            if value.len() >= buffer_size {
                return Err(ReactiveError::BufferTooSmall);
            }

            let c_string = CString::new(value).map_err(|_| ReactiveError::InvalidUtf8)?;
            let bytes = c_string.as_bytes_with_nul();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            *out_found = true;
        } else {
            *out_found = false;
        }
        Ok(())
    }))
}

/// Set environment variable
#[no_mangle]
pub extern "C" fn rtui_platform_set_env_var(
    platform: *mut RTuiPlatform,
    var_name: *const c_char,
    value: *const c_char,
) -> ReactiveError {
    if platform.is_null() || var_name.is_null() || value.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let var_name_str = CStr::from_ptr(var_name)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;
        let value_str = CStr::from_ptr(value)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.set_env_var(var_name_str, value_str);
        Ok(())
    }))
}

/// Enable mouse support
#[no_mangle]
pub extern "C" fn rtui_platform_enable_mouse_support(
    platform: *mut RTuiPlatform,
) -> ReactiveError {
    if platform.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.enable_mouse_support();
        Ok(())
    }))
}

/// Disable mouse support
#[no_mangle]
pub extern "C" fn rtui_platform_disable_mouse_support(
    platform: *mut RTuiPlatform,
) -> ReactiveError {
    if platform.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.disable_mouse_support();
        Ok(())
    }))
}

/// Enable bracketed paste mode
#[no_mangle]
pub extern "C" fn rtui_platform_enable_bracketed_paste(
    platform: *mut RTuiPlatform,
) -> ReactiveError {
    if platform.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.enable_bracketed_paste();
        Ok(())
    }))
}

/// Disable bracketed paste mode
#[no_mangle]
pub extern "C" fn rtui_platform_disable_bracketed_paste(
    platform: *mut RTuiPlatform,
) -> ReactiveError {
    if platform.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.disable_bracketed_paste();
        Ok(())
    }))
}

/// Set window title
#[no_mangle]
pub extern "C" fn rtui_platform_set_window_title(
    platform: *mut RTuiPlatform,
    title: *const c_char,
) -> ReactiveError {
    if platform.is_null() || title.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let title_str = CStr::from_ptr(title)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let platform_ref = &mut *(platform as *mut Platform);
        platform_ref.set_window_title(title_str);
        Ok(())
    }))
}

/// Get current working directory
#[no_mangle]
pub extern "C" fn rtui_platform_get_current_dir(
    platform: *const RTuiPlatform,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if platform.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        let current_dir = platform_ref.get_current_dir();
        
        if current_dir.len() >= buffer_size {
            return Err(ReactiveError::BufferTooSmall);
        }

        let c_string = CString::new(current_dir).map_err(|_| ReactiveError::InvalidUtf8)?;
        let bytes = c_string.as_bytes_with_nul();
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        Ok(())
    }))
}

/// Get home directory
#[no_mangle]
pub extern "C" fn rtui_platform_get_home_dir(
    platform: *const RTuiPlatform,
    buffer: *mut c_char,
    buffer_size: usize,
) -> ReactiveError {
    if platform.is_null() || buffer.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let platform_ref = &*(platform as *const Platform);
        if let Some(home_dir) = platform_ref.get_home_dir() {
            if home_dir.len() >= buffer_size {
                return Err(ReactiveError::BufferTooSmall);
            }

            let c_string = CString::new(home_dir).map_err(|_| ReactiveError::InvalidUtf8)?;
            let bytes = c_string.as_bytes_with_nul();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
        } else {
            return Err(ReactiveError::NotFound);
        }
        Ok(())
    }))
}
