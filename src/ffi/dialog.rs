//! Dialog system FFI functions

use super::*;
use crate::widgets::dialog::{
    ConfirmationButtons, ConfirmationDialogOptions, ConfirmationIcon, DialogEngine, DialogId,
    DialogPosition, DialogResult, InputDialogOptions, InputFieldConfig, InputType, ToastOptions,
    ToastPosition, ToastType,
};
use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::collections::HashMap;
use std::cell::RefCell;

/// Callback function type for dialog completion
pub type DialogCallbackFn = unsafe extern "C" fn(dialog_id: u64, result: i32, user_data: *mut std::ffi::c_void);

/// Dialog callback storage
struct DialogCallback {
    callback: DialogCallbackFn,
    user_data: *mut std::ffi::c_void,
}

// Global callback registry using thread_local for safety
thread_local! {
    static GLOBAL_CALLBACK_REGISTRY: RefCell<HashMap<u64, DialogCallback>> = RefCell::new(HashMap::new());
}

/// Invoke a stored callback and remove it from registry
pub fn invoke_dialog_callback(dialog_id: u64, result: i32) {
    GLOBAL_CALLBACK_REGISTRY.with(|registry| {
        let mut callbacks = registry.borrow_mut();
        if let Some(callback_info) = callbacks.remove(&dialog_id) {
            unsafe {
                (callback_info.callback)(dialog_id, result, callback_info.user_data);
            }
        }
    });
}

/// Opaque handle to a dialog engine
#[repr(C)]
pub struct RTuiDialogEngine {
    _private: [u8; 0],
}

/// Opaque handle to a dialog
#[repr(C)]
pub struct RTuiDialog {
    _private: [u8; 0],
}

/// Dialog types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiDialogType {
    Confirmation = 0,
    Input = 1,
    Toast = 2,
    Progress = 3,
    Wizard = 4,
    Autocomplete = 5,
}

/// Dialog positions
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiDialogPosition {
    Center = 0,
    TopLeft = 1,
    TopRight = 2,
    BottomLeft = 3,
    BottomRight = 4,
    Custom = 5,
}

/// Dialog result
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiDialogResult {
    Confirmed = 0,
    Cancelled = 1,
    Selected = 2,
    Dismissed = 3,
}

/// Confirmation button types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiConfirmationButtons {
    Ok = 0,
    OkCancel = 1,
    YesNo = 2,
    YesNoCancel = 3,
    RetryCancel = 4,
    Custom = 5,
}

/// Confirmation icons
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiConfirmationIcon {
    None = 0,
    Question = 1,
    Warning = 2,
    Error = 3,
    Info = 4,
    Success = 5,
}

/// Input types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiInputType {
    Text = 0,
    Password = 1,
    Email = 2,
    Number = 3,
    Url = 4,
    Search = 5,
    Tel = 6,
}

/// Toast types
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiToastType {
    Info = 0,
    Success = 1,
    Warning = 2,
    Error = 3,
}

/// Dialog callback function type
pub type RTuiDialogCallback = extern "C" fn(
    dialog_id: u64,
    result: RTuiDialogResult,
    data: *const c_char,
    user_data: *mut std::ffi::c_void,
);

/// Create a new dialog engine
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_create(
    out_engine: *mut *mut RTuiDialogEngine,
) -> ReactiveError {
    if out_engine.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let engine = DialogEngine::new();
        let boxed = Box::new(engine);
        unsafe {
            *out_engine = Box::into_raw(boxed) as *mut RTuiDialogEngine;
        }
        Ok(())
    }))
}

/// Destroy a dialog engine
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_destroy(engine: *mut RTuiDialogEngine) {
    if !engine.is_null() {
        unsafe {
            let _ = Box::from_raw(engine as *mut DialogEngine);
        }
    }
}

/// Show a confirmation dialog
#[no_mangle]
pub extern "C" fn rtui_dialog_show_confirmation(
    engine: *mut RTuiDialogEngine,
    title: *const c_char,
    message: *const c_char,
    description: *const c_char,
    buttons: RTuiConfirmationButtons,
    icon: RTuiConfirmationIcon,
    position: RTuiDialogPosition,
    callback: RTuiDialogCallback,
    user_data: *mut std::ffi::c_void,
    out_dialog_id: *mut u64,
) -> ReactiveError {
    if engine.is_null() || title.is_null() || message.is_null() || out_dialog_id.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (engine as usize) % std::mem::align_of::<DialogEngine>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let engine_ref = &mut *(engine as *mut DialogEngine);

        let title_str = CStr::from_ptr(title)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;
        let message_str = CStr::from_ptr(message)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let description_opt = if description.is_null() {
            None
        } else {
            Some(
                CStr::from_ptr(description)
                    .to_str()
                    .map_err(|_| ReactiveError::InvalidUtf8)?
                    .to_string(),
            )
        };

        let button_type = match buttons {
            RTuiConfirmationButtons::Ok => ConfirmationButtons::Ok,
            RTuiConfirmationButtons::OkCancel => ConfirmationButtons::OkCancel,
            RTuiConfirmationButtons::YesNo => ConfirmationButtons::YesNo,
            RTuiConfirmationButtons::YesNoCancel => ConfirmationButtons::YesNoCancel,
            RTuiConfirmationButtons::RetryCancel => ConfirmationButtons::RetryCancel,
            RTuiConfirmationButtons::Custom => ConfirmationButtons::Custom(vec![]),
        };

        let icon_type = match icon {
            RTuiConfirmationIcon::None => None,
            RTuiConfirmationIcon::Question => Some(ConfirmationIcon::Question),
            RTuiConfirmationIcon::Warning => Some(ConfirmationIcon::Warning),
            RTuiConfirmationIcon::Error => Some(ConfirmationIcon::Error),
            RTuiConfirmationIcon::Info => Some(ConfirmationIcon::Info),
            RTuiConfirmationIcon::Success => Some(ConfirmationIcon::Success),
        };

        let dialog_position = match position {
            RTuiDialogPosition::Center => DialogPosition::Center,
            RTuiDialogPosition::TopLeft => DialogPosition::TopCenter, // Map to closest available
            RTuiDialogPosition::TopRight => DialogPosition::TopCenter, // Map to closest available
            RTuiDialogPosition::BottomLeft => DialogPosition::BottomCenter, // Map to closest available
            RTuiDialogPosition::BottomRight => DialogPosition::BottomCenter, // Map to closest available
            RTuiDialogPosition::Custom => DialogPosition::Center,            // Default for custom
        };

        let options = ConfirmationDialogOptions {
            title: title_str.to_string(),
            message: message_str.to_string(),
            description: description_opt,
            buttons: button_type,
            icon: icon_type,
            position: dialog_position,
            ..Default::default()
        };

        let dialog_id = engine_ref.show_confirmation(options);

        // Store callback for async completion
        if callback as *const () != std::ptr::null() {
            let callback_ptr = callback;
            let user_data_ptr = if user_data.is_null() { std::ptr::null_mut() } else { user_data };
            
            // Store in global callback registry for later invocation
            GLOBAL_CALLBACK_REGISTRY.with(|registry| {
                let mut callbacks = registry.borrow_mut();
                callbacks.insert(dialog_id.as_u32() as u64, DialogCallback {
                    callback: callback_ptr,
                    user_data: user_data_ptr,
                });
            });
        }

        *out_dialog_id = dialog_id.as_u32() as u64;
        Ok(())
    }))
}

/// Show an input dialog
#[no_mangle]
pub extern "C" fn rtui_dialog_show_input(
    engine: *mut RTuiDialogEngine,
    title: *const c_char,
    prompt: *const c_char,
    placeholder: *const c_char,
    default_value: *const c_char,
    input_type: RTuiInputType,
    position: RTuiDialogPosition,
    _callback: RTuiDialogCallback,
    _user_data: *mut std::ffi::c_void,
    out_dialog_id: *mut u64,
) -> ReactiveError {
    if engine.is_null() || title.is_null() || prompt.is_null() || out_dialog_id.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (engine as usize) % std::mem::align_of::<DialogEngine>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let engine_ref = &mut *(engine as *mut DialogEngine);

        let title_str = CStr::from_ptr(title)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;
        let prompt_str = CStr::from_ptr(prompt)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let placeholder_str = if placeholder.is_null() {
            None
        } else {
            Some(
                CStr::from_ptr(placeholder)
                    .to_str()
                    .map_err(|_| ReactiveError::InvalidUtf8)?
                    .to_string(),
            )
        };

        let default_str = if default_value.is_null() {
            None
        } else {
            Some(
                CStr::from_ptr(default_value)
                    .to_str()
                    .map_err(|_| ReactiveError::InvalidUtf8)?
                    .to_string(),
            )
        };

        let input_type_enum = match input_type {
            RTuiInputType::Text => InputType::Text,
            RTuiInputType::Password => InputType::Password,
            RTuiInputType::Email => InputType::Email,
            RTuiInputType::Number => InputType::Number,
            RTuiInputType::Url => InputType::Url,
            RTuiInputType::Search => InputType::Search,
            RTuiInputType::Tel => InputType::Phone, // Map Tel to Phone
        };

        let dialog_position = match position {
            RTuiDialogPosition::Center => DialogPosition::Center,
            RTuiDialogPosition::TopLeft => DialogPosition::TopCenter, // Map to closest available
            RTuiDialogPosition::TopRight => DialogPosition::TopCenter, // Map to closest available
            RTuiDialogPosition::BottomLeft => DialogPosition::BottomCenter, // Map to closest available
            RTuiDialogPosition::BottomRight => DialogPosition::BottomCenter, // Map to closest available
            RTuiDialogPosition::Custom => DialogPosition::Center,
        };

        let input_config = InputFieldConfig {
            input_type: input_type_enum,
            placeholder: placeholder_str,
            default_value: default_str,
            ..Default::default()
        };

        let options = InputDialogOptions {
            title: title_str.to_string(),
            prompt: prompt_str.to_string(),
            input: input_config,
            position: dialog_position,
            ..Default::default()
        };

        let dialog_id = engine_ref.show_input(options);

        *out_dialog_id = dialog_id.as_u32() as u64;
        Ok(())
    }))
}

/// Show a toast notification
#[no_mangle]
pub extern "C" fn rtui_dialog_show_toast(
    engine: *mut RTuiDialogEngine,
    message: *const c_char,
    toast_type: RTuiToastType,
    duration_ms: u32,
    position: RTuiDialogPosition,
    out_dialog_id: *mut u64,
) -> ReactiveError {
    if engine.is_null() || message.is_null() || out_dialog_id.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (engine as usize) % std::mem::align_of::<DialogEngine>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let engine_ref = &mut *(engine as *mut DialogEngine);

        let message_str = CStr::from_ptr(message)
            .to_str()
            .map_err(|_| ReactiveError::InvalidUtf8)?;

        let toast_type_enum = match toast_type {
            RTuiToastType::Info => ToastType::Info,
            RTuiToastType::Success => ToastType::Success,
            RTuiToastType::Warning => ToastType::Warning,
            RTuiToastType::Error => ToastType::Error,
        };

        let toast_position = match position {
            RTuiDialogPosition::Center => ToastPosition::TopCenter,
            RTuiDialogPosition::TopLeft => ToastPosition::TopLeft,
            RTuiDialogPosition::TopRight => ToastPosition::TopRight,
            RTuiDialogPosition::BottomLeft => ToastPosition::BottomLeft,
            RTuiDialogPosition::BottomRight => ToastPosition::BottomRight,
            RTuiDialogPosition::Custom => ToastPosition::TopRight,
        };

        let options = ToastOptions {
            message: message_str.to_string(),
            toast_type: toast_type_enum,
            duration: Some(std::time::Duration::from_millis(duration_ms as u64)),
            position: toast_position,
            ..Default::default()
        };

        let dialog_id = engine_ref.show_toast(options);

        *out_dialog_id = dialog_id.as_u32() as u64;
        Ok(())
    }))
}

/// Close a dialog
#[no_mangle]
pub extern "C" fn rtui_dialog_close(
    engine: *mut RTuiDialogEngine,
    dialog_id: u64,
    result: RTuiDialogResult,
) -> ReactiveError {
    if engine.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Validate pointer alignment and basic sanity
        if (engine as usize) % std::mem::align_of::<DialogEngine>() != 0 {
            return Err(ReactiveError::InvalidPointer);
        }
        let engine_ref = &mut *(engine as *mut DialogEngine);

        let dialog_result = match result {
            RTuiDialogResult::Confirmed => DialogResult::Confirmed(None),
            RTuiDialogResult::Cancelled => DialogResult::Cancelled,
            RTuiDialogResult::Selected => DialogResult::Selected("selected".to_string()),
            RTuiDialogResult::Dismissed => DialogResult::Cancelled, // Map to closest available
        };

        // Note: We need a way to construct DialogId from u64
        // For now, we'll need to add a constructor method
        let dialog_id_struct = DialogId::from_u32(dialog_id as u32);
        engine_ref.close_dialog(dialog_id_struct, dialog_result);
        Ok(())
    }))
}

/// Update dialog engine (process events, animations, etc.)
/// Note: Current DialogEngine implementation doesn't have an update method
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_update(
    _engine: *mut RTuiDialogEngine,
    _delta_time_ms: u32,
) -> ReactiveError {
    // DialogEngine doesn't currently have an update method
    // This is a placeholder for future implementation
    ReactiveError::Success
}
