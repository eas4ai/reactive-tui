//! Common ownership and input rules for native controllers.
use super::ReactiveError;
use std::ffi::{c_char, CStr, CString};
use std::thread::{self, ThreadId};

pub(super) type Result<T> = std::result::Result<T, ReactiveError>;
pub(super) const JSON_LIMIT: usize = 1024 * 1024;

pub(super) struct Handle<T> {
    owner: ThreadId,
    pub value: T,
}

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Self {
            owner: thread::current().id(),
            value,
        }
    }

    /// The caller supplies a live handle of the matching type. Check thread
    /// identity before borrowing its non-thread-safe value.
    pub unsafe fn get<'a, O>(pointer: *const O) -> Result<&'a T> {
        if pointer.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        let handle = pointer.cast::<Self>();
        if unsafe { (*handle).owner } != thread::current().id() {
            return Err(ReactiveError::InvalidState);
        }
        Ok(unsafe { &(*handle).value })
    }

    pub unsafe fn get_mut<'a, O>(pointer: *mut O) -> Result<&'a mut T> {
        unsafe {
            Self::get(pointer)?;
        }
        Ok(unsafe { &mut (*pointer.cast::<Self>()).value })
    }

    pub unsafe fn destroy<O>(pointer: *mut O) -> Result<()> {
        if pointer.is_null() {
            return Ok(());
        }
        unsafe {
            Self::get(pointer)?;
        }
        drop(unsafe { Box::from_raw(pointer.cast::<Self>()) });
        Ok(())
    }
}

pub(super) unsafe fn string<'a>(pointer: *const c_char) -> Result<&'a str> {
    if pointer.is_null() {
        return Err(ReactiveError::NullPointer);
    }
    unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map_err(|_| ReactiveError::InvalidUtf8)
}

pub(super) unsafe fn json<T: serde::de::DeserializeOwned>(pointer: *const c_char) -> Result<T> {
    let text = unsafe { string(pointer)? };
    if text.len() > JSON_LIMIT {
        return Err(ReactiveError::InvalidParameter);
    }
    serde_json::from_str(text).map_err(|_| ReactiveError::InvalidParameter)
}

pub(super) unsafe fn out<T>(pointer: *mut *mut T) -> Result<()> {
    if pointer.is_null() {
        return Err(ReactiveError::NullPointer);
    }
    unsafe {
        *pointer = std::ptr::null_mut();
    }
    Ok(())
}

pub(super) unsafe fn owned_string(pointer: *mut *mut c_char, value: String) -> Result<()> {
    let value = CString::new(value).map_err(|_| ReactiveError::InvalidUtf8)?;
    unsafe {
        *pointer = value.into_raw();
    }
    Ok(())
}
