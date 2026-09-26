use reactive_tui::{
    app::{App, AppWaker, RootComponent, RootUpdate},
    backend::Backend,
    component::{Element, ElementType},
    error::{ReactiveError, Result},
    event::types::{Event, KeyCode, KeyEvent},
    reactive::{Scheduler, ThreadSafeSignal},
    render::{reconcile::PatchOp, tree::RenderTree},
};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::{Duration, Instant};

/// A hang guard, not a timing check: generous so a busy machine cannot fail
/// a correct test by running it slowly.
const HANG_GUARD: Duration = Duration::from_secs(30);

#[derive(Default)]
struct Observed {
    frames: Mutex<Vec<(Instant, String)>>,
    polls: AtomicUsize,
    closed: AtomicBool,
    input: Mutex<VecDeque<Event>>,
    waits: Mutex<Vec<Option<Duration>>>,
}
struct ProbeBackend {
    observed: Arc<Observed>,
    entered: mpsc::Sender<()>,
    staged: String,
}
impl Backend for ProbeBackend {
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        let ElementType::Text(text) = &element.element_type else {
            panic!("text frame expected")
        };
        self.staged = text.clone();
        Ok(true)
    }
    fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
        panic!("complete frame expected")
    }
    fn clear(&mut self) -> Result<()> {
        Ok(())
    }
    fn present(&mut self) -> Result<()> {
        self.observed
            .frames
            .lock()
            .unwrap()
            .push((Instant::now(), self.staged.clone()));
        Ok(())
    }
    fn size(&self) -> (u16, u16) {
        (20, 4)
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        panic!("wakeable wait expected")
    }
    fn poll_event_with_wake(
        &mut self,
        timeout: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        self.observed.polls.fetch_add(1, Ordering::Relaxed);
        self.observed.waits.lock().unwrap().push(timeout);
        let _ = self.entered.send(());
        if let Some(event) = self.observed.input.lock().unwrap().pop_front() {
            return Ok(Some(event));
        }
        let start = Instant::now();
        wake.wait(Some(timeout.unwrap_or(HANG_GUARD)));
        if timeout.is_none() && start.elapsed() >= HANG_GUARD {
            return Err(ReactiveError::terminal(
                "test watchdog: idle wait was not woken",
            ));
        }
        Ok(self.observed.input.lock().unwrap().pop_front())
    }
    fn shutdown(&mut self) -> Result<()> {
        self.observed.closed.store(true, Ordering::Release);
        Ok(())
    }
}
struct Root {
    value: ThreadSafeSignal<usize>,
    updates: Arc<AtomicUsize>,
    polling: bool,
    fail: bool,
}
impl RootComponent for Root {
    fn render(&self) -> Element {
        Element::text(self.value.get().to_string())
    }
    fn wake_driven(&self) -> bool {
        !self.polling
    }
    fn update(&mut self) -> Result<RootUpdate> {
        self.updates.fetch_add(1, Ordering::Relaxed);
        if self.fail {
            Err(ReactiveError::terminal("controlled update error"))
        } else {
            Ok(RootUpdate::Unchanged)
        }
    }
}
fn fixture(
    polling: bool,
) -> (
    App,
    Arc<Observed>,
    mpsc::Receiver<()>,
    ThreadSafeSignal<usize>,
    Arc<AtomicUsize>,
) {
    let observed = Arc::new(Observed::default());
    let (entered, receiver) = mpsc::channel();
    let value = ThreadSafeSignal::new(0);
    let updates = Arc::new(AtomicUsize::new(0));
    let app = App::builder()
        .backend(ProbeBackend {
            observed: observed.clone(),
            entered,
            staged: String::new(),
        })
        .root(Root {
            value: value.clone(),
            updates: updates.clone(),
            polling,
            fail: false,
        })
        .build()
        .unwrap();
    (app, observed, receiver, value, updates)
}
fn wait_for(mut predicate: impl FnMut() -> bool) {
    let end = Instant::now() + HANG_GUARD;
    while !predicate() {
        assert!(Instant::now() < end, "condition did not become true");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn signal_wakes_idle_app_without_input_and_equal_writes_do_not_redraw() {
    let (app, observed, entered, signal, updates) = fixture(false);
    let wake = app.waker();
    let runner = std::thread::spawn(move || app.run());
    entered.recv_timeout(HANG_GUARD).unwrap();
    // The behavior under test: an idle App stays in one wait with nothing to do.
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(
        observed.polls.load(Ordering::Relaxed),
        1,
        "idle App must remain in one wait"
    );
    assert_eq!(updates.load(Ordering::Relaxed), 1);
    signal.set(0);
    // The behavior under test: writing the value a signal already holds draws nothing.
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(observed.frames.lock().unwrap().len(), 1);
    signal.set(42);
    wait_for(|| observed.frames.lock().unwrap().last().unwrap().1 == "42");
    wake.request_stop();
    runner.join().unwrap().unwrap();
    assert!(observed.closed.load(Ordering::Acquire));
    assert!(wake.is_closed());
    signal.set(43);
    assert!(
        !wake.is_pending(),
        "retained signals must not wake a finished App"
    );
}

#[test]
fn queued_work_and_new_earlier_timer_wake_app_and_callbacks_are_reentrant() {
    let (app, observed, entered, signal, _) = fixture(false);
    let wake = app.waker();
    let scheduler = app.scheduler();
    // A timer that must never fire: its delay outlasts every wait in the test.
    scheduler.schedule_timeout(Duration::from_secs(3600), || {
        panic!("late timer must be cancelled")
    });
    let runner = std::thread::spawn(move || app.run());
    entered.recv_timeout(HANG_GUARD).unwrap();
    let changed = signal.clone();
    scheduler.schedule_update(Box::new(move || changed.set(1)));
    wait_for(|| observed.frames.lock().unwrap().last().unwrap().1 == "1");
    let nested_scheduler = scheduler.clone();
    scheduler.schedule_timeout(Duration::from_millis(25), move || {
        nested_scheduler.clear();
        nested_scheduler.schedule_timeout(Duration::from_millis(10), move || signal.set(2));
    });
    wait_for(|| observed.frames.lock().unwrap().last().unwrap().1 == "2");
    wake.request_stop();
    runner.join().unwrap().unwrap();
}

#[test]
fn redraw_burst_retains_latest_state_and_respects_frame_pacing() {
    let (app, observed, entered, signal, _) = fixture(false);
    let wake = app.waker();
    let frame_duration = Duration::from_secs_f64(1.0 / app.get_current_fps() as f64);
    let runner = std::thread::spawn(move || app.run());
    entered.recv_timeout(HANG_GUARD).unwrap();
    for value in 1..=5000 {
        signal.set(value);
    }
    wait_for(|| observed.frames.lock().unwrap().last().unwrap().1 == "5000");
    wake.request_stop();
    runner.join().unwrap().unwrap();
    let frames = observed.frames.lock().unwrap();
    assert!(
        frames.len() < 100,
        "notifications must not each produce a frame"
    );
    // The behavior under test: frames stay at least a frame apart; a busy
    // machine only spreads them further.
    for pair in frames.windows(2) {
        assert!(
            pair[1].0.duration_since(pair[0].0) >= frame_duration.mul_f64(0.8),
            "frame pacing was bypassed"
        );
    }
}

#[test]
fn input_and_legacy_polling_roots_still_progress() {
    let (app, observed, entered, _, updates) = fixture(true);
    let wake = app.waker();
    let runner = std::thread::spawn(move || app.run());
    entered.recv_timeout(HANG_GUARD).unwrap();
    wait_for(|| updates.load(Ordering::Relaxed) >= 3);
    observed
        .input
        .lock()
        .unwrap()
        .push_back(Event::Key(KeyEvent::new(KeyCode::Escape)));
    wake.wake();
    runner.join().unwrap().unwrap();
    assert!(observed.closed.load(Ordering::Acquire));
}

#[test]
fn scheduler_cannot_be_shared_by_two_live_apps_and_detaches_after_drop() {
    let scheduler = Arc::new(Scheduler::new());
    let make = || {
        let (entered, _) = mpsc::channel();
        App::builder()
            .backend(ProbeBackend {
                observed: Arc::new(Observed::default()),
                entered,
                staged: String::new(),
            })
            .root(Root {
                value: ThreadSafeSignal::new(0),
                updates: Arc::new(AtomicUsize::new(0)),
                polling: false,
                fail: false,
            })
            .scheduler(scheduler.clone())
            .build()
    };
    let app = make().unwrap();
    assert!(make().is_err());
    drop(app);
    assert!(make().is_ok());
}

#[test]
fn app_error_closes_wake_handle_and_restores_backend() {
    let observed = Arc::new(Observed::default());
    let (entered, _) = mpsc::channel();
    let app = App::builder()
        .backend(ProbeBackend {
            observed: observed.clone(),
            entered,
            staged: String::new(),
        })
        .root(Root {
            value: ThreadSafeSignal::new(0),
            updates: Arc::new(AtomicUsize::new(0)),
            polling: false,
            fail: true,
        })
        .build()
        .unwrap();
    let wake = app.waker();
    assert!(app.run().is_err());
    assert!(wake.is_closed());
    assert!(observed.closed.load(Ordering::Acquire));
}

#[test]
fn animations_advance_then_app_returns_to_idle_waiting() {
    use reactive_tui::animation::{Animation, EasingFunction, LoopMode};
    let (mut app, observed, entered, _, _) = fixture(false);
    let mut animation = Animation::new(
        Duration::from_millis(90),
        EasingFunction::Linear,
        None,
        LoopMode::None,
    );
    animation.play();
    app.animation_manager().add_animation(animation);
    let wake = app.waker();
    let runner = std::thread::spawn(move || app.run());
    entered.recv_timeout(HANG_GUARD).unwrap();
    wait_for(|| observed.waits.lock().unwrap().iter().any(Option::is_none));
    assert!(observed.frames.lock().unwrap().len() >= 2);
    let polls = observed.polls.load(Ordering::Relaxed);
    // The behavior under test: a finished animation leaves the App idle.
    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(observed.polls.load(Ordering::Relaxed), polls);
    wake.request_stop();
    runner.join().unwrap().unwrap();
}
