//! Native layout styles use the same parser and snapshots as the App painter.
use super::controller::{self, Handle};
use super::{catch_panic, RTuiElement, ReactiveError};
use crate::component::Element;
use crate::layout::style::{StyleBuilder, StyleSnapshot};
use std::ffi::c_char;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

/// Owning style snapshot, confined to its creating thread.
#[repr(C)]
pub struct RTuiNativeStyle {
    _private: [u8; 0],
}

/// Parse native inline CSS; invalid declarations leave out_style null.
#[no_mangle]
pub extern "C" fn rtui_native_style_create(
    css: *const c_char,
    out_style: *mut *mut RTuiNativeStyle,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_style)?;
        let style = StyleBuilder::new()
            .with_inline_declarations(controller::string(css)?)
            .map_err(|_| ReactiveError::InvalidParameter)?
            .snapshot();
        style
            .restore()
            .map_err(|_| ReactiveError::InvalidParameter)?;
        *out_style = Box::into_raw(Box::new(Handle::new(style))).cast();
        Ok(())
    }))
}

/// Copy a style into a borrowed Element. Neither handle is consumed.
#[no_mangle]
pub extern "C" fn rtui_native_style_apply(
    style: *const RTuiNativeStyle,
    element: *mut RTuiElement,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if element.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        let style = Handle::<StyleSnapshot>::get(style)?;
        (*element.cast::<Element>()).metadata.styles = Some(Arc::new(style.clone()));
        Ok(())
    }))
}

/// Release a style handle on its creating thread; null is accepted.
#[no_mangle]
pub extern "C" fn rtui_native_style_destroy(style: *mut RTuiNativeStyle) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        Handle::<StyleSnapshot>::destroy(style)
    }))
}
