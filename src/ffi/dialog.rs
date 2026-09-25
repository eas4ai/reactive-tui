//! Native dialog sessions and their real App host and completion queue.
use super::controller::{self, Handle};
use super::{catch_panic, RTuiElement, ReactiveError};
use crate::widgets::dialog::{DialogEngine, DialogEngineError, DialogEvent, DialogId};
use std::ffi::c_char;
use std::panic::AssertUnwindSafe;

mod options;

/// Owning, thread-confined controller. Its Element retains the native host.
#[repr(C)]
pub struct RTuiDialogEngine {
    _private: [u8; 0],
}

fn error(error: DialogEngineError) -> ReactiveError {
    match error {
        DialogEngineError::NotFound => ReactiveError::NotFound,
        DialogEngineError::WrongType
        | DialogEngineError::InvalidProgress
        | DialogEngineError::InvalidDuration
        | DialogEngineError::InvalidAnimationName
        | DialogEngineError::UnknownAnimation(_) => ReactiveError::InvalidParameter,
        _ => ReactiveError::InvalidState,
    }
}

/// Create an isolated native dialog engine with default limits.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_create(
    out_engine: *mut *mut RTuiDialogEngine,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_engine)?;
        *out_engine = Box::into_raw(Box::new(Handle::new(DialogEngine::new()))).cast();
        Ok(())
    }))
}

/// Cancel all remaining sessions before releasing this controller.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_destroy(engine: *mut RTuiDialogEngine) -> ReactiveError {
    if engine.is_null() {
        return ReactiveError::Success;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        Handle::<DialogEngine>::get_mut(engine)?.close_all();
        Handle::<DialogEngine>::destroy(engine)
    }))
}

/// Return the owned normal App host Element for this engine.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_element(
    engine: *const RTuiDialogEngine,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_element)?;
        let element = Handle::<DialogEngine>::get(engine)?.render();
        *out_element = Box::into_raw(Box::new(element)).cast();
        Ok(())
    }))
}

/// Open one of the six native families. JSON fields are validated per family.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_open(
    engine: *mut RTuiDialogEngine,
    options: *const c_char,
    out_id: *mut u32,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if out_id.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        *out_id = 0;
        let options: options::Open = controller::json(options)?;
        let id = options
            .open(Handle::<DialogEngine>::get_mut(engine)?)
            .map_err(error)?;
        *out_id = id.as_u32();
        Ok(())
    }))
}

/// Apply exactly one update: progress, input, wizardData, or zIndex.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_update(
    engine: *mut RTuiDialogEngine,
    id: u32,
    update: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let update: options::Update = controller::json(update)?;
        Handle::<DialogEngine>::get_mut(engine)?
            .update(DialogId::from_u32(id), update.into())
            .map_err(error)
    }))
}

/// Close an active dialog once, retaining its supplied result and data.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_close(
    engine: *mut RTuiDialogEngine,
    id: u32,
    result: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let result: options::Completion = controller::json(result)?;
        let engine = Handle::<DialogEngine>::get_mut(engine)?;
        let id = DialogId::from_u32(id);
        if !engine.is_open(id) {
            return Err(ReactiveError::NotFound);
        }
        engine.close_dialog(id, result.into());
        Ok(())
    }))
}

/// Consume one native Opened/Closed event. Success with null means empty.
#[no_mangle]
pub extern "C" fn rtui_dialog_engine_take_event(
    engine: *mut RTuiDialogEngine,
    out_event: *mut *mut c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_event)?;
        let Some(event) = Handle::<DialogEngine>::get_mut(engine)?.take_event() else {
            return Ok(());
        };
        let value = match event {
            DialogEvent::Opened(id) => serde_json::json!({"kind":"opened","id":id.as_u32()}),
            DialogEvent::Closed(id, result) => {
                serde_json::json!({"kind":"closed","id":id.as_u32(),"result":options::Completion::from(result)})
            }
            // DialogEngine currently queues only Opened and Closed, unlike the
            // lower-level DialogEvent vocabulary. Do not invent missing payloads.
            _ => return Err(ReactiveError::NotSupported),
        };
        controller::owned_string(out_event, value.to_string())
    }))
}
