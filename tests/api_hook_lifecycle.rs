use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::{Backend, SuprTuiBackend},
    component,
    component::{registry::register_component, Element, LayoutType},
    error::Result,
    event::types::Event,
    hooks::timer::{use_interval, use_timeout},
    reactive::{provide_context, use_context, use_effect, use_effect_with_deps, Hooks, Scheduler},
    render::{reconcile::PatchOp, RenderTree},
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Once,
    },
    time::{Duration, Instant},
};
static EVENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static TICKS: AtomicUsize = AtomicUsize::new(0);
#[component]
fn EffectLeaf(hooks: &Hooks, value: usize) -> Element {
    let value = *value;
    use_effect(hooks, move || {
        EVENTS.lock().unwrap().push(format!("effect:{value}"));
        Some(Box::new(move || {
            EVENTS.lock().unwrap().push(format!("cleanup:{value}"))
        }))
    });
    EVENTS.lock().unwrap().push(format!("render:{value}"));
    Element::text(format!("leaf:{value}"))
}
#[component]
fn Dependent(hooks: &Hooks, value: usize) -> Element {
    let value = *value;
    use_effect_with_deps(hooks, value, move || {
        EVENTS.lock().unwrap().push(format!("effect:{value}"));
        Some(Box::new(move || {
            EVENTS.lock().unwrap().push(format!("cleanup:{value}"))
        }))
    });
    EVENTS.lock().unwrap().push(format!("render:{value}"));
    Element::text(format!("dependent:{value}"))
}
impl Default for DependentProps {
    fn default() -> Self {
        Self { value: usize::MAX }
    }
}
#[component]
fn ContextLeaf(hooks: &Hooks) -> Element {
    Element::text(use_context::<String>(hooks).unwrap_or_else(|| "absent".into())).class("h-1")
}
#[component]
fn Provider(hooks: &Hooks, value: String) -> Element {
    provide_context(hooks, value.clone());
    Element::component("ApiLifecycleContextLeaf")
}
#[component]
fn Outer(hooks: &Hooks, value: String) -> Element {
    provide_context(hooks, value.clone());
    column(vec![
        Element::component("ApiLifecycleContextLeaf").key("before"),
        Provider::element("inner".into()).key("override"),
        Element::component("ApiLifecycleContextLeaf").key("after"),
    ])
}
#[component]
fn Timers(hooks: &Hooks) -> Element {
    use_interval(hooks, Duration::ZERO, || {
        TICKS.fetch_add(1, Ordering::SeqCst);
    });
    use_timeout(hooks, Duration::from_secs(3600), || {
        panic!("removed timeout ran");
    });
    Element::text("timer")
}
impl Default for EffectLeafProps {
    fn default() -> Self {
        Self { value: usize::MAX }
    }
}
impl Default for ProviderProps {
    fn default() -> Self {
        Self {
            value: "fixture default".into(),
        }
    }
}
impl Default for OuterProps {
    fn default() -> Self {
        Self {
            value: "fixture default".into(),
        }
    }
}
fn register() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        register_component::<Dependent>("Dependent").unwrap();
        register_component::<EffectLeaf>("EffectLeaf").unwrap();
        register_component::<ContextLeaf>("ApiLifecycleContextLeaf").unwrap();
        register_component::<Provider>("Provider").unwrap();
        register_component::<Outer>("Outer").unwrap();
        register_component::<Timers>("Timers").unwrap();
    });
}
fn column(children: Vec<Element>) -> Element {
    Element::layout(LayoutType::Flex)
        .class("flex flex-col w-full h-full")
        .children(children)
}
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct ProbeBackend {
    inner: SuprTuiBackend,
    output: Capture,
    frames: Arc<Mutex<Vec<String>>>,
    deadline: Instant,
    scheduler: Arc<Scheduler>,
    scheduled: Arc<Mutex<Vec<bool>>>,
}
impl Backend for ProbeBackend {
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
        panic!("complete frame required")
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn present(&mut self) -> Result<()> {
        self.inner.present()?;
        self.inner.sync()?;
        self.scheduled
            .lock()
            .unwrap()
            .push(self.scheduler.next_deadline().is_some());
        let mut parser = vt100::Parser::new(6, 32, 0);
        parser.process(&self.output.0.lock().unwrap());
        self.frames.lock().unwrap().push(parser.screen().contents());
        Ok(())
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        Ok(None)
    }
    fn poll_event_with_wake(
        &mut self,
        timeout: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        assert!(
            Instant::now() < self.deadline,
            "App exceeded component test deadline"
        );
        wake.wait(Some(
            timeout
                .unwrap_or(Duration::from_millis(10))
                .min(Duration::from_millis(10)),
        ));
        Ok(None)
    }
}
struct Root {
    frames: Vec<Element>,
    index: AtomicUsize,
    wake: Option<AppWaker>,
}
impl RootComponent for Root {
    fn attach_waker(&mut self, wake: AppWaker) {
        self.wake = Some(wake);
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn render(&self) -> Element {
        let index = self.index.fetch_add(1, Ordering::SeqCst);
        let element = self.frames[index].clone();
        let wake = self.wake.as_ref().unwrap();
        if index + 1 == self.frames.len() {
            wake.request_stop();
        } else {
            wake.request_redraw();
        }
        element
    }
}
fn run_frames(elements: Vec<Element>) -> (Result<()>, Vec<String>, Arc<Scheduler>, Vec<bool>) {
    let output = Capture::default();
    let frames = Arc::new(Mutex::new(Vec::new()));
    let scheduler = Arc::new(Scheduler::new());
    let scheduled = Arc::new(Mutex::new(Vec::new()));
    let backend = ProbeBackend {
        scheduler: scheduler.clone(),
        scheduled: scheduled.clone(),
        inner: SuprTuiBackend::with_writer(32, 6, output.clone()).unwrap(),
        output,
        frames: frames.clone(),
        // a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
        deadline: Instant::now() + Duration::from_secs(30),
    };
    let result = App::builder()
        .scheduler(scheduler.clone())
        .backend(backend)
        .root(Root {
            frames: elements,
            index: AtomicUsize::new(0),
            wake: None,
        })
        .build()
        .unwrap()
        .run();
    let captured = frames.lock().unwrap().clone();
    let observed = scheduled.lock().unwrap().clone();
    (result, captured, scheduler, observed)
}

#[test]
#[serial_test::serial]
fn effects_cleanup_on_rerender_and_removal_after_the_render_body() {
    register();
    EVENTS.lock().unwrap().clear();
    let (result, _, _, _) = run_frames(vec![
        EffectLeaf::element(1),
        EffectLeaf::element(2),
        Element::text("removed"),
    ]);
    result.unwrap();
    assert_eq!(
        *EVENTS.lock().unwrap(),
        [
            "render:1",
            "effect:1",
            "render:2",
            "cleanup:1",
            "effect:2",
            "cleanup:2"
        ]
    );
}
#[test]
#[serial_test::serial]
fn providers_inherit_override_and_restore_without_leaking_to_another_app() {
    register();
    let (result, frames, _, _) = run_frames(vec![
        Outer::element("outer".into()),
        Outer::element("updated".into()),
    ]);
    result.unwrap();
    assert_eq!(
        frames[0]
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["outer", "inner", "outer"]
    );
    assert_eq!(
        frames[1]
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["updated", "inner", "updated"]
    );
    let (result, frames, _, _) = run_frames(vec![Element::component("ApiLifecycleContextLeaf")]);
    result.unwrap();
    assert_eq!(frames[0].trim(), "absent");
}
#[test]
#[serial_test::serial]
fn app_timers_fire_and_removal_leaves_no_scheduled_work() {
    register();
    TICKS.store(0, Ordering::SeqCst);
    let (result, _, scheduler, scheduled) = run_frames(vec![
        Timers::element(),
        Element::text("removed"),
        Element::text("finished"),
    ]);
    result.unwrap();
    assert_eq!(
        scheduled,
        [true, false, false],
        "timers must belong to App and disappear on removal before paint"
    );
    let before = TICKS.load(Ordering::SeqCst);
    assert!(
        before > 0,
        "component interval must execute on the App scheduler"
    );
    assert!(scheduler.next_deadline().is_none());
    scheduler.process_timers();
    assert_eq!(TICKS.load(Ordering::SeqCst), before);
}
#[test]
#[serial_test::serial]
fn standalone_effect_cleanup_waits_until_explicit_cleanup() {
    let hooks = Hooks::new();
    let cleanup = Arc::new(AtomicUsize::new(0));
    let copy = cleanup.clone();
    use_effect(&hooks, move || {
        Some(Box::new(move || {
            copy.fetch_add(1, Ordering::SeqCst);
        }))
    });
    assert_eq!(cleanup.load(Ordering::SeqCst), 0);
    hooks.cleanup();
    hooks.cleanup();
    assert_eq!(cleanup.load(Ordering::SeqCst), 1);
}

#[test]
#[serial_test::serial]
fn effect_dependencies_skip_unchanged_renders_and_cleanup_before_replacement() {
    register();
    EVENTS.lock().unwrap().clear();
    let (result, _, _, _) = run_frames(vec![
        Dependent::element(1),
        Dependent::element(1),
        Dependent::element(2),
        Element::text("removed"),
    ]);
    result.unwrap();
    assert_eq!(
        *EVENTS.lock().unwrap(),
        [
            "render:1",
            "effect:1",
            "render:1",
            "render:2",
            "cleanup:1",
            "effect:2",
            "cleanup:2"
        ]
    );
}

#[test]
#[serial_test::serial]
fn app_stop_cleans_up_components_that_are_still_mounted() {
    register();
    EVENTS.lock().unwrap().clear();
    let (result, _, scheduler, scheduled) = run_frames(vec![column(vec![
        EffectLeaf::element(9),
        Timers::element(),
    ])]);
    result.unwrap();
    assert_eq!(
        *EVENTS.lock().unwrap(),
        ["render:9", "effect:9", "cleanup:9"]
    );
    assert_eq!(scheduled, [true]);
    assert!(scheduler.next_deadline().is_none());
}

// The API-013 keyframe mechanism also runs this real App/SuprTUI workflow.
static KEYFRAME_HANDLE: Mutex<Option<reactive_tui::hooks::animation::KeyframeHandle<f32>>> =
    Mutex::new(None);

#[component]
fn KeyframeLeaf(hooks: &Hooks, progress: f32) -> Element {
    use reactive_tui::{animation::Keyframe, hooks::animation::use_keyframes};
    let handle = use_keyframes(
        hooks,
        2.0_f32,
        vec![
            Keyframe::new(0.0).number("x", 2.0),
            Keyframe::new(1.0).number("x", 10.0),
        ],
    );
    handle.seek(*progress);
    let text = format!("keyframe:{:.0}", handle.value());
    *KEYFRAME_HANDLE.lock().unwrap() = Some(handle);
    Element::text(text)
}
impl Default for KeyframeLeafProps {
    fn default() -> Self {
        Self { progress: 0.0 }
    }
}

#[test]
#[serial_test::serial]
fn keyframe_values_paint_through_app_and_removal_disables_the_escaped_handle() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| register_component::<KeyframeLeaf>("KeyframeLeaf").unwrap());
    let (result, frames, _, _) = run_frames(vec![
        KeyframeLeaf::element(0.0).key("animated"),
        KeyframeLeaf::element(0.5).key("animated"),
        KeyframeLeaf::element(1.0).key("animated"),
        Element::text("removed"),
    ]);
    result.unwrap();
    assert_eq!(
        frames.iter().map(|frame| frame.trim()).collect::<Vec<_>>(),
        ["keyframe:2", "keyframe:6", "keyframe:10", "removed"]
    );
    let handle = KEYFRAME_HANDLE.lock().unwrap().take().unwrap();
    handle.play();
    handle.seek(0.0);
    reactive_tui::hooks::animation::update_hook_animations();
    assert_eq!(
        handle.value(),
        10.0,
        "removed component accepted animation work"
    );
}
