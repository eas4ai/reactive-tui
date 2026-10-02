//! Application framework FFI functions

use super::*;
use crate::app::{App, AppBuilder, AppWaker, RootComponent};
use crate::backend::{Backend, CrosstermBackend, DebugBackend, SuprTuiBackend};
use crate::component::Element;
use crate::display::monitor::PerformanceMode;
use std::boxed::Box;
use std::sync::{Mutex, OnceLock};

#[derive(Default)]
struct NativeAppBuilder {
    inner: AppBuilder,
    terminal_selected: bool,
}

fn app_builder_tracker() -> &'static super::pointer::PointerTracker<NativeAppBuilder> {
    static TRACKER: OnceLock<super::pointer::PointerTracker<NativeAppBuilder>> = OnceLock::new();
    TRACKER.get_or_init(super::pointer::PointerTracker::new)
}

fn select_terminal_backend<B: Backend + 'static>(
    builder: *mut RTuiAppBuilder,
    create: impl FnOnce() -> crate::error::Result<B>,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder = &mut *builder.cast::<NativeAppBuilder>();
        // Both native terminal selectors name the complete-frame route. The
        // builder has not run it, so selecting it again retains the same owner.
        if !builder.terminal_selected {
            let backend = create()?;
            builder.inner = std::mem::take(&mut builder.inner).backend(backend);
            builder.terminal_selected = true;
        }
        Ok(())
    }))
}

/// Opaque handle to an app builder
#[repr(C)]
pub struct RTuiAppBuilder {
    _private: [u8; 0],
}

/// Opaque handle to an app instance
#[repr(C)]
pub struct RTuiApp {
    _private: [u8; 0],
}

/// Opaque handle to a root component
#[repr(C)]
pub struct RTuiRootComponent {
    _private: [u8; 0],
}

/// Performance mode enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RTuiPerformanceMode {
    /// Power saving mode (lower FPS)
    PowerSave = 0,
    /// Balanced mode (moderate FPS)
    Balanced = 1,
    /// Performance mode (high FPS)
    Performance = 2,
}

impl From<RTuiPerformanceMode> for PerformanceMode {
    fn from(mode: RTuiPerformanceMode) -> Self {
        match mode {
            RTuiPerformanceMode::PowerSave => PerformanceMode::PowerSave,
            RTuiPerformanceMode::Balanced => PerformanceMode::Balanced,
            RTuiPerformanceMode::Performance => PerformanceMode::Performance,
        }
    }
}

/// Performance metrics structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RTuiPerformanceMetrics {
    /// Current frames per second
    pub current_fps: f32,
    /// Average render time in milliseconds
    pub avg_render_time_ms: f32,
    /// Frame drop rate as percentage
    pub drop_rate_percent: f32,
    /// Whether performance is stable
    pub is_stable: bool,
}

/// Root component callback function type.
///
/// Pass `Some(callback)` to register a callback or `None` to leave it unset.
/// A bare function pointer is not a nullable callback:
///
/// ```compile_fail
/// use reactive_tui::ffi::RTuiElement;
/// use std::ffi::c_void;
///
/// type BareRootCallback = extern "C" fn(*mut c_void) -> *mut RTuiElement;
///
/// fn nullable(
///     callback: BareRootCallback,
/// ) -> Option<extern "C" fn(*mut c_void) -> *mut RTuiElement> {
///     callback
/// }
/// ```
pub type RTuiRootComponentCallback =
    Option<extern "C" fn(user_data: *mut std::ffi::c_void) -> *mut super::builder::RTuiElement>;

/// FFI-compatible root component wrapper
struct FFIRootComponent {
    callback: RTuiRootComponentCallback,
    user_data: *mut std::ffi::c_void,
}

struct NativeApp {
    app: Mutex<Option<App>>,
    wake: AppWaker,
}

impl NativeApp {
    fn new(app: App) -> Self {
        let wake = app.waker();
        Self {
            app: Mutex::new(Some(app)),
            wake,
        }
    }
}

struct ElementRoot(Element);
impl RootComponent for ElementRoot {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// Retain an Element root, consuming it on success. Nested foreign components
/// use the normal fallible component runtime without an infallible root callback.
#[no_mangle]
pub extern "C" fn rtui_app_builder_root_element(
    builder: *mut RTuiAppBuilder,
    element: *mut RTuiElement,
) -> ReactiveError {
    if builder.is_null() || element.is_null() {
        return ReactiveError::NullPointer;
    }
    catch_panic(AssertUnwindSafe(|| unsafe {
        let builder = &mut (*builder.cast::<NativeAppBuilder>()).inner;
        let element = *Box::from_raw(element.cast::<Element>());
        *builder = std::mem::take(builder).root(ElementRoot(element));
        Ok(())
    }))
}

unsafe impl Send for FFIRootComponent {}
unsafe impl Sync for FFIRootComponent {}

impl RootComponent for FFIRootComponent {
    fn render(&self) -> Element {
        let Some(callback) = self.callback else {
            return Element::empty();
        };
        let element_ptr = callback(self.user_data);
        if element_ptr.is_null() {
            return Element::empty();
        }

        // SAFETY: Convert FFI element pointer back to native Element
        // The callback is expected to return a valid Element pointer that we own
        unsafe {
            let element_box = Box::from_raw(element_ptr as *mut super::builder::FFIElement);
            element_box.inner
        }
    }
}

/// Create a new app builder
#[no_mangle]
pub extern "C" fn rtui_app_builder_create(out_builder: *mut *mut RTuiAppBuilder) -> ReactiveError {
    if out_builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| {
        let builder = NativeAppBuilder::default();
        let boxed = Box::new(builder);
        unsafe {
            let raw = Box::into_raw(boxed);
            if !app_builder_tracker().register(raw) {
                drop(Box::from_raw(raw));
                return Err(ReactiveError::OutOfMemory);
            }
            *out_builder = raw.cast::<RTuiAppBuilder>();
        }
        Ok(())
    }))
}

/// Destroy an app builder
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_app_builder_destroy(builder: *mut RTuiAppBuilder) {
    if !builder.is_null() {
        let raw = builder.cast::<NativeAppBuilder>();
        if app_builder_tracker().unregister(raw) {
            unsafe {
                let _ = Box::from_raw(raw);
            }
        }
    }
}

/// Set debug mode for app builder (safe in-place modification)
#[no_mangle]
pub extern "C" fn rtui_app_builder_debug(
    builder: *mut RTuiAppBuilder,
    debug: bool,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Update the value while retaining the caller-owned allocation.
        let builder_ref = &mut (*builder.cast::<NativeAppBuilder>()).inner;
        let new_builder = std::mem::take(builder_ref).debug(debug);

        // Replace the empty value; the allocation and handle remain unchanged.
        *builder_ref = new_builder;
        Ok(())
    }))
}

/// Set performance mode for app builder (safe in-place modification)
#[no_mangle]
pub extern "C" fn rtui_app_builder_performance_mode(
    builder: *mut RTuiAppBuilder,
    mode: RTuiPerformanceMode,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Update the value while retaining the caller-owned allocation.
        let builder_ref = &mut (*builder.cast::<NativeAppBuilder>()).inner;
        let new_builder = std::mem::take(builder_ref).performance_mode(mode.into());

        // Replace the empty value; the allocation and handle remain unchanged.
        *builder_ref = new_builder;
        Ok(())
    }))
}

/// Set backend for app builder (creates debug backend with specified size)
#[no_mangle]
pub extern "C" fn rtui_app_builder_backend_debug(
    builder: *mut RTuiAppBuilder,
    width: u16,
    height: u16,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Update the value while retaining the caller-owned allocation.
        let backend = DebugBackend::new(width, height);
        (*builder.cast::<NativeAppBuilder>()).terminal_selected = false;
        let builder_ref = &mut (*builder.cast::<NativeAppBuilder>()).inner;
        let new_builder = std::mem::take(builder_ref).backend(backend);

        // Replace the empty value; the allocation and handle remain unchanged.
        *builder_ref = new_builder;
        Ok(())
    }))
}

/// Select the complete-frame SuprTUI renderer and enter its terminal session.
/// The builder owns that session until build transfers it to the App, or the
/// builder is destroyed. Setup errors leave the existing builder value intact.
/// Re-selecting either native terminal route retains its current session.
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_app_builder_backend_suprtui(builder: *mut RTuiAppBuilder) -> ReactiveError {
    select_terminal_backend(builder, SuprTuiBackend::new)
}

/// Set backend for app builder (creates crossterm backend)
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_app_builder_backend_crossterm(
    builder: *mut RTuiAppBuilder,
) -> ReactiveError {
    select_terminal_backend(builder, CrosstermBackend::new)
}

/// Set root component for app builder (safe in-place modification)
#[no_mangle]
pub extern "C" fn rtui_app_builder_root_component(
    builder: *mut RTuiAppBuilder,
    callback: RTuiRootComponentCallback,
    user_data: *mut std::ffi::c_void,
) -> ReactiveError {
    if builder.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        // Update the value while retaining the caller-owned allocation.
        let builder_ref = &mut (*builder.cast::<NativeAppBuilder>()).inner;
        let root_component = FFIRootComponent {
            callback,
            user_data,
        };
        let new_builder = std::mem::take(builder_ref).root(root_component);

        // Replace the empty value; the allocation and handle remain unchanged.
        *builder_ref = new_builder;
        Ok(())
    }))
}

/// Build the app from the builder
#[no_mangle]
pub extern "C" fn rtui_app_builder_build(
    builder: *mut RTuiAppBuilder,
    out_app: *mut *mut RTuiApp,
) -> ReactiveError {
    if builder.is_null() || out_app.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        *out_app = std::ptr::null_mut();
        let raw = builder.cast::<NativeAppBuilder>();
        if !app_builder_tracker().unregister(raw) {
            return Err(ReactiveError::InvalidPointer);
        }
        let builder_box = Box::from_raw(raw);
        match builder_box.inner.build() {
            Ok(app) => {
                *out_app = Box::into_raw(Box::new(NativeApp::new(app))) as *mut RTuiApp;
                Ok(())
            }
            Err(_) => Err(ReactiveError::InternalError),
        }
    }))
}

/// Destroy an app
#[reactive_tui_macros::ffi_export]
pub extern "C" fn rtui_app_destroy(app: *mut RTuiApp) {
    if !app.is_null() {
        unsafe {
            let _ = Box::from_raw(app.cast::<NativeApp>());
        }
    }
}

/// Run the app as a blocking call. The handle remains valid until destroy.
/// While this call is active, only quit is available; other app calls return
/// InvalidState.
#[no_mangle]
pub extern "C" fn rtui_app_run(app: *mut RTuiApp) -> ReactiveError {
    if app.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        let owned = {
            let mut slot = native.app.lock().map_err(|_| ReactiveError::InvalidState)?;
            slot.take().ok_or(ReactiveError::InvalidState)?
        };
        match owned.run() {
            Ok(()) => Ok(()),
            Err(_) => Err(ReactiveError::InternalError),
        }
    }))
}

/// Stop the app
#[no_mangle]
pub extern "C" fn rtui_app_quit(app: *mut RTuiApp) -> ReactiveError {
    if app.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        native.wake.request_stop();
        Ok(())
    }))
}

/// Get app terminal size
#[no_mangle]
pub extern "C" fn rtui_app_get_size(
    app: *const RTuiApp,
    out_dimensions: *mut RTuiDimensions,
) -> ReactiveError {
    if app.is_null() || out_dimensions.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        let slot = native.app.lock().map_err(|_| ReactiveError::InvalidState)?;
        let app_ref = slot.as_ref().ok_or(ReactiveError::InvalidState)?;
        let (width, height) = app_ref.size();
        *out_dimensions = RTuiDimensions { width, height };
        Ok(())
    }))
}

/// Set app performance mode
#[no_mangle]
pub extern "C" fn rtui_app_set_performance_mode(
    app: *mut RTuiApp,
    mode: RTuiPerformanceMode,
) -> ReactiveError {
    if app.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        let mut slot = native.app.lock().map_err(|_| ReactiveError::InvalidState)?;
        let app_ref = slot.as_mut().ok_or(ReactiveError::InvalidState)?;
        app_ref.set_performance_mode(mode.into());
        Ok(())
    }))
}

/// Get current FPS
#[no_mangle]
pub extern "C" fn rtui_app_get_current_fps(
    app: *const RTuiApp,
    out_fps: *mut u32,
) -> ReactiveError {
    if app.is_null() || out_fps.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        let slot = native.app.lock().map_err(|_| ReactiveError::InvalidState)?;
        let app_ref = slot.as_ref().ok_or(ReactiveError::InvalidState)?;
        *out_fps = app_ref.get_current_fps();
        Ok(())
    }))
}

/// Get performance metrics
#[no_mangle]
pub extern "C" fn rtui_app_get_performance_metrics(
    app: *const RTuiApp,
    out_metrics: *mut RTuiPerformanceMetrics,
) -> ReactiveError {
    if app.is_null() || out_metrics.is_null() {
        return ReactiveError::NullPointer;
    }

    catch_panic(AssertUnwindSafe(|| unsafe {
        let native = &*app.cast::<NativeApp>();
        let slot = native.app.lock().map_err(|_| ReactiveError::InvalidState)?;
        let app_ref = slot.as_ref().ok_or(ReactiveError::InvalidState)?;
        let metrics = app_ref.get_performance_metrics();
        *out_metrics = RTuiPerformanceMetrics {
            current_fps: metrics.current_fps,
            avg_render_time_ms: metrics.avg_render_time_ms,
            drop_rate_percent: metrics.drop_rate_percent,
            is_stable: metrics.is_stable,
        };
        Ok(())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_callback_transfers_the_ffi_element_wrapper() {
        extern "C" fn render(_: *mut std::ffi::c_void) -> *mut super::super::builder::RTuiElement {
            let mut element = std::ptr::null_mut();
            assert_eq!(
                super::super::rtui_text_element_create(c"callback root".as_ptr(), &mut element),
                ReactiveError::Success,
            );
            element
        }
        let root = FFIRootComponent {
            callback: Some(render),
            user_data: std::ptr::null_mut(),
        };
        assert_eq!(
            root.render().element_type,
            crate::component::ElementType::Text("callback root".into())
        );
    }
}
