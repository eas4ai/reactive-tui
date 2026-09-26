use reactive_tui::{
    app::{App, AppWaker, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    component,
    component::{registry::register_component, Element},
    display::monitor::PerformanceMode,
    error::{ReactiveError, Result},
    hooks::{
        perf_context::PerformanceContext, use_adaptive_quality, use_fps, use_frame_timing,
        use_performance, use_performance_mode,
    },
    reactive::{provide_context, Hooks},
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex, Once,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
struct Observed {
    renders: Arc<AtomicUsize>,
    modes: Arc<Mutex<Vec<PerformanceMode>>>,
}

impl PartialEq for Observed {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.renders, &other.renders) && Arc::ptr_eq(&self.modes, &other.modes)
    }
}

#[component]
fn PerformanceLeaf(hooks: &Hooks, observed: Observed) -> Element {
    assert_ne!(
        observed.renders.load(Ordering::SeqCst),
        usize::MAX,
        "component did not receive the observation props"
    );
    let state = use_fps(hooks).get();
    let _ = use_performance(hooks).0.get();
    let _ = use_frame_timing(hooks).get();
    let _ = use_performance_mode(hooks);
    let _ = use_adaptive_quality(hooks);
    observed.renders.fetch_add(1, Ordering::SeqCst);
    observed.modes.lock().unwrap().push(state.mode);
    Element::text(format!("{:?}", state.mode))
}
impl Default for PerformanceLeafProps {
    fn default() -> Self {
        Self {
            // Fail immediately if generated prop delivery leaves this sentinel.
            observed: Observed {
                renders: Arc::new(AtomicUsize::new(usize::MAX)),
                ..Observed::default()
            },
        }
    }
}

struct Root {
    observed: Observed,
    override_context: Option<Arc<PerformanceContext>>,
    failure: u8,
}
impl RootComponent for Root {
    fn render(&self) -> Element {
        if self.failure == 2 {
            panic!("controlled performance render panic");
        }
        if let Some(context) = &self.override_context {
            provide_context(&Hooks::new(), context.as_ref().clone());
        }
        PerformanceLeaf::element(self.observed.clone())
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if self.failure == 1 {
            Err(ReactiveError::terminal(
                "controlled performance update error",
            ))
        } else {
            Ok(RootUpdate::Unchanged)
        }
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
fn fixture(
    mode: PerformanceMode,
    override_context: Option<Arc<PerformanceContext>>,
    failure: u8,
) -> (App, Observed) {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| register_component::<PerformanceLeaf>("PerformanceLeaf").unwrap());
    let observed = Observed::default();
    let app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(Root {
            observed: observed.clone(),
            override_context,
            failure,
        })
        .performance_mode(mode)
        .build()
        .unwrap();
    (app, observed)
}

// Every started App has a stop watchdog and joins both threads, including on assertion unwind.
struct Running {
    wake: AppWaker,
    runner: Option<JoinHandle<Result<()>>>,
    watchdog: Option<JoinHandle<()>>,
    cancel: mpsc::Sender<()>,
}
impl Running {
    fn start(app: App) -> Self {
        let wake = app.waker();
        let deadline_wake = wake.clone();
        let (cancel, cancelled) = mpsc::channel();
        let watchdog = thread::spawn(move || {
            // A hang guard, not a timing check. It outlasts HANG_GUARD, so a
            // wait inside the test fails on its own before this stops the App.
            if cancelled.recv_timeout(4 * HANG_GUARD).is_err() {
                deadline_wake.request_stop();
            }
        });
        Self {
            wake,
            runner: Some(thread::spawn(move || app.run())),
            watchdog: Some(watchdog),
            cancel,
        }
    }
    fn finish(mut self) -> thread::Result<Result<()>> {
        self.wake.request_stop();
        let result = self.runner.take().unwrap().join();
        let _ = self.cancel.send(());
        self.watchdog.take().unwrap().join().unwrap();
        result
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.wake.request_stop();
        if let Some(runner) = self.runner.take() {
            let _ = runner.join();
        }
        let _ = self.cancel.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
    }
}
/// A hang guard, not a timing check: generous so a busy machine cannot fail
/// a correct test by running it slowly.
const HANG_GUARD: Duration = Duration::from_secs(30);
/// How long a render count must stay the same before the App counts as idle.
const QUIET: Duration = Duration::from_millis(300);

fn wait_for(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + HANG_GUARD;
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "performance state did not converge"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
/// Waits until `count` has stayed the same for `QUIET` while the App still
/// runs, and returns it. An App that keeps redrawing never goes quiet, so the
/// wait fails with `redrawing` once the hang guard runs out; one that stopped
/// fails it at once, since a stopped App is quiet without being idle.
fn wait_quiet(running: &Running, count: impl Fn() -> usize, redrawing: &str) -> usize {
    let deadline = Instant::now() + HANG_GUARD;
    let mut last = count();
    loop {
        thread::sleep(QUIET);
        let now = count();
        assert!(
            !running.runner.as_ref().unwrap().is_finished(),
            "the App stopped before it went idle: {redrawing}"
        );
        if now == last {
            return now;
        }
        assert!(Instant::now() < deadline, "{redrawing}");
        last = now;
    }
}
fn assert_closed(context: &PerformanceContext, wake: &AppWaker) {
    let before = context.fps_state.get();
    (context.set_mode)(PerformanceMode::Gaming);
    assert!(wake.is_closed());
    assert!(!wake.is_pending());
    assert_eq!(context.fps_state.get(), before);
}

#[test]
fn sequential_apps_keep_snapshots_and_closed_setters_isolated() {
    let (first, seen) = fixture(PerformanceMode::PowerSave, None, 0);
    let old = first.performance_context();
    let first = Running::start(first);
    let old_wake = first.wake.clone();
    wait_for(|| seen.renders.load(Ordering::SeqCst) > 0);
    first.finish().unwrap().unwrap();
    let snapshot = old.fps_state.get();
    let (second, seen) = fixture(PerformanceMode::Performance, None, 0);
    let current = second.performance_context();
    assert!(!Arc::ptr_eq(&old, &current));
    let second = Running::start(second);
    wait_for(|| seen.renders.load(Ordering::SeqCst) > 0);
    assert_closed(&old, &old_wake);
    assert_eq!(old.fps_state.get(), snapshot);
    assert_eq!(current.fps_state.get().mode, PerformanceMode::Performance);
    assert!(seen
        .modes
        .lock()
        .unwrap()
        .iter()
        .all(|mode| *mode == PerformanceMode::Performance));
    second.finish().unwrap().unwrap();
}

#[test]
fn concurrent_apps_route_worker_mode_requests_only_to_their_owner() {
    let (a, seen_a) = fixture(PerformanceMode::PowerSave, None, 0);
    let (b, seen_b) = fixture(PerformanceMode::Balanced, None, 0);
    let context_a = a.performance_context();
    let context_b = b.performance_context();
    let a = Running::start(a);
    let b = Running::start(b);
    wait_for(|| {
        seen_a.renders.load(Ordering::SeqCst) > 0 && seen_b.renders.load(Ordering::SeqCst) > 0
    });
    let setter = context_a.set_mode.clone();
    thread::spawn(move || setter(PerformanceMode::Performance))
        .join()
        .unwrap();
    wait_for(|| context_a.fps_state.get().mode == PerformanceMode::Performance);
    assert_eq!(context_b.fps_state.get().mode, PerformanceMode::Balanced);
    assert_eq!(context_a.fps_state.get().target_fps, 90);
    a.finish().unwrap().unwrap();
    b.finish().unwrap().unwrap();
}

#[test]
fn mode_bursts_keep_latest_request_and_metrics_do_not_redraw_idle_apps() {
    let (app, observed) = fixture(PerformanceMode::Balanced, None, 0);
    let context = app.performance_context();
    // Before the loop starts, the bounded request slot must retain the latest request.
    for _ in 0..5000 {
        (context.set_mode)(PerformanceMode::Gaming);
    }
    (context.set_mode)(PerformanceMode::PowerSave);
    let running = Running::start(app);
    wait_for(|| {
        context.fps_state.get().mode == PerformanceMode::PowerSave
            && observed.renders.load(Ordering::SeqCst) > 0
    });
    let count = wait_quiet(
        &running,
        || observed.renders.load(Ordering::SeqCst),
        "publishing metrics fed back into redraws",
    );
    // The behavior under test: an idle App stays idle while metrics publish.
    thread::sleep(Duration::from_millis(120));
    assert_eq!(
        observed.renders.load(Ordering::SeqCst),
        count,
        "publishing metrics fed back into redraws"
    );
    let timing = context.frame_timing.get();
    assert!((timing.target_frame_ms - 1000.0 / 30.0).abs() < 0.1);
    assert!(timing.last_frame_ms.is_finite() && timing.last_frame_ms > 0.0);
    assert!(
        (timing.budget_remaining_ms - (timing.target_frame_ms - timing.last_frame_ms)).abs() < 0.1
    );
    running.finish().unwrap().unwrap();
}

#[test]
fn descendant_provider_overrides_app_performance_context() {
    let (source, _) = fixture(PerformanceMode::Gaming, None, 0);
    let provider = source.performance_context();
    let (app, observed) = fixture(PerformanceMode::PowerSave, Some(provider), 0);
    let context = app.performance_context();
    let running = Running::start(app);
    wait_for(|| observed.renders.load(Ordering::SeqCst) > 0);
    assert!(observed
        .modes
        .lock()
        .unwrap()
        .iter()
        .all(|mode| *mode == PerformanceMode::Gaming));
    assert_eq!(context.fps_state.get().mode, PerformanceMode::PowerSave);
    running.finish().unwrap().unwrap();
    drop(source);
}

#[test]
fn errors_unwinds_and_unrun_drop_close_escaped_mode_setters() {
    for failure in [1, 2] {
        let (app, _) = fixture(PerformanceMode::Balanced, None, failure);
        let context = app.performance_context();
        let wake = app.waker();
        let running = Running::start(app);
        wait_for(|| running.runner.as_ref().unwrap().is_finished());
        let result = running.finish();
        if failure == 1 {
            assert!(result.unwrap().is_err());
        } else {
            assert!(result.is_err());
        }
        assert_closed(&context, &wake);
    }
    let (app, _) = fixture(PerformanceMode::Balanced, None, 0);
    let context = app.performance_context();
    let wake = app.waker();
    drop(app);
    assert_closed(&context, &wake);
}

fn render_hooks(hooks: &Hooks) {
    let render = hooks.begin_render();
    let _ = use_fps(hooks);
    let _ = use_performance(hooks);
    let _ = use_frame_timing(hooks);
    let _ = use_performance_mode(hooks);
    let _ = use_adaptive_quality(hooks);
    drop(render);
}
struct ScopeRoot {
    hooks: Hooks,
    rendered: Arc<AtomicUsize>,
}
impl RootComponent for ScopeRoot {
    fn render(&self) -> Element {
        assert!(reactive_tui::reactive::use_context::<PerformanceContext>(&self.hooks).is_some());
        render_hooks(&self.hooks);
        self.rendered.fetch_add(1, Ordering::SeqCst);
        Element::text("scope")
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn performance_hooks_keep_slots_when_context_appears_and_owner_closes() {
    let hooks = Hooks::new();
    render_hooks(&hooks);
    let rendered = Arc::new(AtomicUsize::new(0));
    let app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(ScopeRoot {
            hooks: hooks.clone(),
            rendered: rendered.clone(),
        })
        .build()
        .unwrap();
    let running = Running::start(app);
    wait_for(|| rendered.load(Ordering::SeqCst) > 0);
    running.finish().unwrap().unwrap();
    assert!(reactive_tui::reactive::use_context::<PerformanceContext>(&hooks).is_none());
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render_hooks(&hooks)));
    assert!(
        rejected.is_err(),
        "App-owned hooks must reject renders after cleanup"
    );
}

struct NestedRoot {
    inner: Mutex<Option<App>>,
    outer_modes: Arc<Mutex<Vec<PerformanceMode>>>,
}
impl RootComponent for NestedRoot {
    fn render(&self) -> Element {
        let hooks = Hooks::new();
        let before = use_fps(&hooks).get().mode;
        let inner = self.inner.lock().unwrap().take();
        if let Some(inner) = inner {
            let wake = inner.waker();
            let stop = wake.clone();
            inner
                .scheduler()
                .schedule_timeout(Duration::from_millis(30), move || stop.request_stop());
            let (cancel, cancelled) = mpsc::channel();
            let watchdog = thread::spawn(move || {
                if cancelled.recv_timeout(HANG_GUARD).is_err() {
                    wake.request_stop();
                }
            });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| inner.run()));
            let _ = cancel.send(());
            watchdog.join().unwrap();
            result.unwrap().unwrap();
        }
        hooks.reset();
        let after = use_fps(&hooks).get().mode;
        self.outer_modes.lock().unwrap().extend([before, after]);
        Element::text("outer")
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn nested_app_run_restores_outer_performance_context() {
    let (inner, seen) = fixture(PerformanceMode::Gaming, None, 0);
    let inner_context = inner.performance_context();
    let modes = Arc::new(Mutex::new(Vec::new()));
    let outer = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(NestedRoot {
            inner: Mutex::new(Some(inner)),
            outer_modes: modes.clone(),
        })
        .performance_mode(PerformanceMode::PowerSave)
        .build()
        .unwrap();
    let context = outer.performance_context();
    let running = Running::start(outer);
    wait_for(|| !modes.lock().unwrap().is_empty());
    assert!(modes
        .lock()
        .unwrap()
        .iter()
        .all(|mode| *mode == PerformanceMode::PowerSave));
    assert!(seen.renders.load(Ordering::SeqCst) > 0);
    assert!(seen
        .modes
        .lock()
        .unwrap()
        .iter()
        .all(|mode| *mode == PerformanceMode::Gaming));
    assert_eq!(inner_context.fps_state.get().mode, PerformanceMode::Gaming);
    assert_eq!(context.fps_state.get().mode, PerformanceMode::PowerSave);
    running.finish().unwrap().unwrap();
}

#[test]
fn standalone_globals_are_not_read_or_written_by_apps() {
    const CHILD: &str = "RTUI_PERFORMANCE_GLOBAL_CHILD";
    if std::env::var_os(CHILD).is_some() {
        use reactive_tui::{
            hooks::{
                perf_context::{
                    get_global_performance_context, request_performance_mode,
                    set_global_performance_context, take_requested_performance_mode,
                },
                FpsState, FrameTiming,
            },
            reactive::ThreadSafeSignal,
        };
        let standalone = Arc::new(PerformanceContext {
            fps_state: ThreadSafeSignal::new(FpsState {
                mode: PerformanceMode::Gaming,
                target_fps: 17,
                ..FpsState::default()
            }),
            metrics: ThreadSafeSignal::new(Default::default()),
            frame_timing: ThreadSafeSignal::new(FrameTiming::default()),
            set_mode: Arc::new(request_performance_mode),
        });
        set_global_performance_context(standalone.clone());
        request_performance_mode(PerformanceMode::PowerSave);
        let (app, seen) = fixture(PerformanceMode::Balanced, None, 0);
        let owned = app.performance_context();
        let running = Running::start(app);
        wait_for(|| seen.renders.load(Ordering::SeqCst) > 0);
        running.finish().unwrap().unwrap();
        assert_eq!(owned.fps_state.get().mode, PerformanceMode::Balanced);
        assert_eq!(standalone.fps_state.get().target_fps, 17);
        assert_eq!(standalone.fps_state.get().mode, PerformanceMode::Gaming);
        assert!(Arc::ptr_eq(
            &get_global_performance_context().unwrap(),
            &standalone
        ));
        assert_eq!(
            take_requested_performance_mode(),
            Some(PerformanceMode::PowerSave)
        );
        assert_eq!(use_fps(&Hooks::new()).get().target_fps, 17);
        return;
    }
    // Global compatibility state is confined to a child, keeping fallback tests independent.
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "standalone_globals_are_not_read_or_written_by_apps",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .spawn()
        .unwrap();
    let deadline = Instant::now() + HANG_GUARD;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(
                status.success(),
                "global compatibility child failed: {status}"
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("global compatibility child exceeded watchdog");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

struct RepeatedModeRoot {
    hooks: Hooks,
    renders: Arc<AtomicUsize>,
}
impl RootComponent for RepeatedModeRoot {
    fn render(&self) -> Element {
        let render = self.hooks.begin_render();
        let setter = use_performance_mode(&self.hooks);
        reactive_tui::reactive::use_effect(&self.hooks, move || {
            setter(PerformanceMode::Gaming);
            let cleanup = setter.clone();
            Some(Box::new(move || cleanup(PerformanceMode::Balanced)))
        });
        drop(render);
        self.renders.fetch_add(1, Ordering::SeqCst);
        Element::text("same mode")
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn repeated_mode_effects_and_cleanup_requests_return_to_idle() {
    let renders = Arc::new(AtomicUsize::new(0));
    let app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(RepeatedModeRoot {
            hooks: Hooks::new(),
            renders: renders.clone(),
        })
        .performance_mode(PerformanceMode::Gaming)
        .build()
        .unwrap();
    let context = app.performance_context();
    let running = Running::start(app);
    wait_for(|| renders.load(Ordering::SeqCst) > 0);
    running.wake.request_redraw();
    wait_for(|| renders.load(Ordering::SeqCst) >= 2);
    let count = wait_quiet(
        &running,
        || renders.load(Ordering::SeqCst),
        "same-mode effect requests kept redrawing",
    );
    // The behavior under test: repeated same-mode requests leave the App idle.
    thread::sleep(Duration::from_millis(100));
    assert_eq!(
        renders.load(Ordering::SeqCst),
        count,
        "same-mode effect requests kept redrawing"
    );
    assert_eq!(context.fps_state.get().mode, PerformanceMode::Gaming);
    let wake = running.wake.clone();
    running.finish().unwrap().unwrap();
    assert_closed(&context, &wake);
}

#[test]
fn actual_fps_tracks_presentation_cadence_instead_of_render_throughput() {
    let (app, observed) = fixture(PerformanceMode::PowerSave, None, 0);
    let context = app.performance_context();
    let running = Running::start(app);
    wait_for(|| {
        observed.renders.load(Ordering::SeqCst) > 0
            && context.frame_timing.get().last_frame_ms > 0.0
    });
    // Let mount effects settle, then leave a measured idle interval between presentations.
    let renders = wait_quiet(
        &running,
        || observed.renders.load(Ordering::SeqCst),
        "mount effects kept redrawing",
    );
    let previous_fps = context.fps_state.get().current_fps;
    // The behavior under test: a 100 ms idle interval between presentations.
    thread::sleep(Duration::from_millis(100));
    assert_eq!(observed.renders.load(Ordering::SeqCst), renders);
    running.wake.request_redraw();
    wait_for(|| {
        let state = context.fps_state.get();
        observed.renders.load(Ordering::SeqCst) > renders
            && state.current_fps != previous_fps
            && context.metrics.get().current_fps == state.current_fps
    });
    let fps = context.fps_state.get().current_fps;
    assert!(
        fps.is_finite() && fps > 0.0 && fps <= 35.0,
        "reported {fps} FPS despite PowerSave pacing and a 100 ms idle interval"
    );
    running.finish().unwrap().unwrap();
}
