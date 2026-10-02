//! Foreign state and callbacks retained by normal App components.
use super::controller::{self, Result};
use super::{catch_panic, RTuiElement, ReactiveError};
use crate::component::{Component, Element, Props};
use crate::event::router::EventResult;
use crate::reactive::ThreadSafeSignal;
use std::ffi::{c_char, c_void, CString};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, ThreadId};

mod input;

/// Owns callbacks and JSON state until explicitly destroyed on its owner thread.
#[repr(C)]
pub struct RTuiForeignComponent {
    _private: [u8; 0],
}

/// Return zero and an owned Element, or a native error code. Strings are borrowed.
pub type RTuiForeignRenderCallback =
    Option<extern "C" fn(*const c_char, *const c_char, *mut c_void, *mut *mut RTuiElement) -> i32>;
/// Return zero and set handled, or return a native error code. Strings are borrowed.
pub type RTuiForeignEventCallback = Option<
    extern "C" fn(*const c_char, *const c_char, *const c_char, *mut c_void, *mut bool) -> i32,
>;
/// Called once by successful explicit destruction, never by a renderer worker.
pub type RTuiForeignDisposeCallback = Option<extern "C" fn(*mut c_void)>;

struct Core {
    owner: ThreadId,
    props: ThreadSafeSignal<String>,
    state: ThreadSafeSignal<String>,
    render: extern "C" fn(*const c_char, *const c_char, *mut c_void, *mut *mut RTuiElement) -> i32,
    event: RTuiForeignEventCallback,
    dispose: RTuiForeignDisposeCallback,
    // Opaque address is passed back only after owner-thread validation.
    userdata: usize,
    busy: AtomicBool,
    closed: AtomicBool,
    failure: ThreadSafeSignal<i32>,
}

struct CallbackGuard<'a>(&'a AtomicBool);
impl Drop for CallbackGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn error(code: i32) -> ReactiveError {
    match code {
        -1 => ReactiveError::InvalidParameter,
        -2 => ReactiveError::NullPointer,
        -3 => ReactiveError::BufferTooSmall,
        -4 => ReactiveError::OutOfMemory,
        -5 => ReactiveError::InvalidUtf8,
        -6 => ReactiveError::TerminalNotAvailable,
        -7 => ReactiveError::NotSupported,
        -8 => ReactiveError::AlreadyExists,
        -9 => ReactiveError::NotFound,
        -10 => ReactiveError::InvalidState,
        -11 => ReactiveError::InvalidPointer,
        -12 => ReactiveError::InternalError,
        -99 => ReactiveError::Panic,
        _ => ReactiveError::Unknown,
    }
}

impl Core {
    fn live(&self) -> Result<()> {
        if self.owner != thread::current().id() || self.closed.load(Ordering::Acquire) {
            return Err(ReactiveError::InvalidState);
        }
        Ok(())
    }

    fn enter(&self) -> Result<CallbackGuard<'_>> {
        self.live()?;
        if self.busy.swap(true, Ordering::AcqRel) {
            return Err(ReactiveError::InvalidState);
        }
        Ok(CallbackGuard(&self.busy))
    }

    fn failed(&self, code: i32) -> ReactiveError {
        let error = error(code);
        // The App subscribes to failures independently of prop/state changes.
        self.failure.set(error as i32);
        error
    }

    fn render(&self) -> Result<Element> {
        let _guard = self.enter()?;
        let props = CString::new(self.props.get()).map_err(|_| ReactiveError::InvalidUtf8)?;
        let state = CString::new(self.state.get()).map_err(|_| ReactiveError::InvalidUtf8)?;
        let mut output = std::ptr::null_mut();
        let code = (self.render)(
            props.as_ptr(),
            state.as_ptr(),
            self.userdata as *mut c_void,
            &mut output,
        );
        // Ownership transfers even when a callback reports failure.
        let element =
            (!output.is_null()).then(|| unsafe { Box::from_raw(output.cast::<Element>()) });
        if code != 0 {
            return Err(self.failed(code));
        }
        let element = element.ok_or_else(|| self.failed(ReactiveError::InvalidState as i32))?;
        self.failure.set(0);
        Ok(*element)
    }

    fn dispatch(&self, event: &str) -> Result<bool> {
        let _guard = self.enter()?;
        let Some(callback) = self.event else {
            return Ok(false);
        };
        let event = CString::new(event).map_err(|_| ReactiveError::InvalidUtf8)?;
        let props = CString::new(self.props.get()).map_err(|_| ReactiveError::InvalidUtf8)?;
        let state = CString::new(self.state.get()).map_err(|_| ReactiveError::InvalidUtf8)?;
        let mut handled = false;
        let code = callback(
            event.as_ptr(),
            props.as_ptr(),
            state.as_ptr(),
            self.userdata as *mut c_void,
            &mut handled,
        );
        if code != 0 {
            return Err(self.failed(code));
        }
        Ok(handled)
    }
}

unsafe fn core<'a>(handle: *const RTuiForeignComponent) -> Result<&'a Arc<Core>> {
    if handle.is_null() {
        return Err(ReactiveError::NullPointer);
    }
    let core = unsafe { &*handle.cast::<Arc<Core>>() };
    core.live()?;
    Ok(core)
}

#[derive(Clone)]
struct ForeignProps(Arc<Core>);
impl PartialEq for ForeignProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Props for ForeignProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct Foreign;
impl Component for Foreign {
    type Props = ForeignProps;
    type State = ();
    fn new(_: Self::Props) -> Self {
        Self
    }
    fn render(&self, props: &Self::Props, state: &()) -> Element {
        self.try_render(props, state)
            .expect("foreign component rendering failed")
    }
    fn try_render(&self, props: &Self::Props, _: &()) -> crate::error::Result<Element> {
        let core = &props.0;
        let pending = core.failure.get();
        let result = if pending != 0 {
            Err(error(pending))
        } else {
            core.render()
        };
        result.map_err(|code| {
            crate::error::ReactiveError::component(format!(
                "foreign callback failed with code {}",
                code as i32
            ))
        })
    }
    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        _: &mut (),
    ) -> EventResult {
        match props.0.dispatch(&input::encode(event)) {
            Ok(true) => EventResult::Handled,
            Ok(false) => EventResult::Ignored,
            Err(_) => EventResult::Handled,
        }
    }
}

/// Create a state controller and retain callback/userdata addresses until successful destruction.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_create(
    props: *const c_char,
    state: *const c_char,
    render: RTuiForeignRenderCallback,
    event: RTuiForeignEventCallback,
    dispose: RTuiForeignDisposeCallback,
    userdata: *mut c_void,
    out_component: *mut *mut RTuiForeignComponent,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_component)?;
        let render = render.ok_or(ReactiveError::NullPointer)?;
        let props: serde_json::Value = controller::json(props)?;
        let state: serde_json::Value = controller::json(state)?;
        let value = Arc::new(Core {
            owner: thread::current().id(),
            props: ThreadSafeSignal::new(props.to_string()),
            state: ThreadSafeSignal::new(state.to_string()),
            render,
            event,
            dispose,
            userdata: userdata as usize,
            busy: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            failure: ThreadSafeSignal::new(0),
        });
        *out_component = Box::into_raw(Box::new(value)).cast();
        Ok(())
    }))
}

/// Disable callbacks and dispose userdata once; recursive or wrong-thread destruction fails.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_destroy(
    component: *mut RTuiForeignComponent,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        let value = core(component)?.clone();
        let _guard = value.enter()?;
        value.closed.store(true, Ordering::Release);
        if let Some(dispose) = value.dispose {
            dispose(value.userdata as *mut c_void);
        }
        drop(Box::from_raw(component.cast::<Arc<Core>>()));
        Ok(())
    }))
}

/// Return an owned typed Element referencing this controller, without invoking callbacks.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_element(
    component: *const RTuiForeignComponent,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_element)?;
        let element = Element::typed::<Foreign>(ForeignProps(core(component)?.clone()));
        *out_element = Box::into_raw(Box::new(element)).cast();
        Ok(())
    }))
}

/// Invoke render synchronously and return an owned snapshot; recursive entry is rejected.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_render(
    component: *const RTuiForeignComponent,
    out_element: *mut *mut RTuiElement,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        controller::out(out_element)?;
        let element = core(component)?.render()?;
        *out_element = Box::into_raw(Box::new(element)).cast();
        Ok(())
    }))
}

/// Send a JSON object to the event callback; recursive entry is rejected.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_dispatch(
    component: *const RTuiForeignComponent,
    event: *const c_char,
    out_handled: *mut bool,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if out_handled.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        *out_handled = false;
        let event: serde_json::Map<String, serde_json::Value> = controller::json(event)?;
        *out_handled = core(component)?.dispatch(&serde_json::Value::Object(event).to_string())?;
        Ok(())
    }))
}

unsafe fn get_json(
    component: *const RTuiForeignComponent,
    out_value: *mut *mut c_char,
    props: bool,
) -> Result<()> {
    unsafe {
        controller::out(out_value)?;
    }
    let core = unsafe { core(component)? };
    let value = if props {
        core.props.get()
    } else {
        core.state.get()
    };
    unsafe { controller::owned_string(out_value, value) }
}

unsafe fn set_json(
    component: *const RTuiForeignComponent,
    value: *const c_char,
    props: bool,
) -> Result<()> {
    let value: serde_json::Value = unsafe { controller::json(value)? };
    let core = unsafe { core(component)? };
    if props {
        core.props.set(value.to_string());
    } else {
        core.state.set(value.to_string());
    }
    Ok(())
}

/// Return an independently owned JSON copy of the current props.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_get_props(
    component: *const RTuiForeignComponent,
    out_value: *mut *mut c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        get_json(component, out_value, true)
    }))
}
/// Return an independently owned JSON copy of the current state.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_get_state(
    component: *const RTuiForeignComponent,
    out_value: *mut *mut c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        get_json(component, out_value, false)
    }))
}
/// Replace valid JSON props and notify subscribing Apps; callable during a callback.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_set_props(
    component: *const RTuiForeignComponent,
    value: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        set_json(component, value, true)
    }))
}
/// Replace valid JSON state and notify subscribing Apps; callable during a callback.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_set_state(
    component: *const RTuiForeignComponent,
    value: *const c_char,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        set_json(component, value, false)
    }))
}

/// Configure native keyboard focus for a foreign-rendered target.
#[no_mangle]
pub extern "C" fn rtui_element_set_focus(
    element: *mut RTuiElement,
    focusable: bool,
    auto_focus: bool,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if element.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        (*element.cast::<Element>()).focus = Some(crate::component::FocusProps {
            focusable,
            auto_focus,
            ..Default::default()
        });
        Ok(())
    }))
}

/// Read the last callback failure, or zero. A successful explicit render clears
/// it. App.run retains its existing generic error code; this preserves the cause.
#[no_mangle]
pub extern "C" fn rtui_foreign_component_last_error(
    component: *const RTuiForeignComponent,
    out_code: *mut i32,
) -> ReactiveError {
    catch_panic(AssertUnwindSafe(|| unsafe {
        if out_code.is_null() {
            return Err(ReactiveError::NullPointer);
        }
        *out_code = core(component)?.failure.get();
        Ok(())
    }))
}
