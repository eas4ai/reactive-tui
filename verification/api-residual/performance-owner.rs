//! Diagnostic only: expose cross-App performance state and inaccurate mode reporting.
use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::SuprTuiBackend,
    component::Element,
    display::monitor::PerformanceMode,
    hooks::fps::FpsState,
    hooks::{use_fps, use_performance_mode},
    reactive::{Hooks, ThreadSafeSignal},
};
use std::sync::{Arc, Mutex};
type Setter = Arc<dyn Fn(PerformanceMode) + Send + Sync>;
struct Root {
    hooks: Hooks,
    wake: Option<AppWaker>,
    out: Arc<Mutex<Option<(ThreadSafeSignal<FpsState>, Setter)>>>,
}
impl RootComponent for Root {
    fn attach_waker(&mut self, w: AppWaker) {
        self.wake = Some(w);
    }
    fn render(&self) -> Element {
        *self.out.lock().unwrap() = Some((use_fps(&self.hooks), use_performance_mode(&self.hooks)));
        self.wake.as_ref().unwrap().request_stop();
        Element::text("performance owner probe")
    }
}
fn run(mode: PerformanceMode) -> (ThreadSafeSignal<FpsState>, Setter) {
    let out = Arc::new(Mutex::new(None));
    App::builder()
        .backend(SuprTuiBackend::with_writer(32, 4, std::io::sink()).unwrap())
        .performance_mode(mode)
        .root(Root {
            hooks: Hooks::new(),
            wake: None,
            out: out.clone(),
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
    let result = out.lock().unwrap().take().unwrap();
    result
}
fn main() {
    let (first, set_first) = run(PerformanceMode::PowerSave);
    let before = first.get();
    let (second, _) = run(PerformanceMode::Gaming);
    let after = first.get();
    println!("First App before second: {before:?}");
    println!("First App after second: {after:?}");
    println!("Second App: {:?}", second.get());
    set_first(PerformanceMode::Balanced);
    println!(
        "Escaped first-App setter feeds process queue: {:?}",
        reactive_tui::hooks::perf_context::take_requested_performance_mode()
    );
    assert_eq!(
        before, after,
        "another App changed the first App performance signal"
    );
    assert_eq!(
        before.mode,
        PerformanceMode::PowerSave,
        "reported mode disagrees with selected mode"
    );
}
