use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::{Backend, DebugBackend, FrameLayout, PaintedNode, PresentedLayout, SuprTuiBackend},
    component::{
        registry::register_component, Component, Element, LayoutType, LifecycleEvent, Props,
    },
    error::Result,
    event::types::{Event, ResizeEvent},
    render::{reconcile::PatchOp, RenderTree},
};
use std::{
    any::Any,
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Once,
    },
    time::{Duration, Instant},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
static EVENTS: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());
#[derive(Clone, Default, PartialEq)]
struct Label {
    text: String,
}
impl Props for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
struct Leaf {
    id: usize,
}
impl Component for Leaf {
    type Props = Label;
    type State = usize;
    fn new(_: Label) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
    fn update(&mut self, _: &Label, state: &mut usize) -> bool {
        *state += 1;
        true
    }
    fn render(&self, props: &Label, state: &usize) -> Element {
        EVENTS
            .lock()
            .unwrap()
            .push((self.id, format!("render:{}:{state}", props.text)));
        Element::text(format!("{}:{state}", props.text)).class("w-full h-1")
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut usize) {
        let event = match event {
            LifecycleEvent::Mount => "mount",
            LifecycleEvent::Unmount => "unmount",
            _ => return,
        };
        EVENTS.lock().unwrap().push((self.id, event.into()));
    }
}
struct Wrapper;
impl Component for Wrapper {
    type Props = Label;
    type State = ();
    fn new(_: Label) -> Self {
        Self
    }
    fn render(&self, props: &Label, _: &()) -> Element {
        Element::component_with_props("ApiExpansionLeaf", props.clone()).key("leaf")
    }
}
struct Recursive;
impl Component for Recursive {
    type Props = Label;
    type State = ();
    fn new(_: Label) -> Self {
        Self
    }
    fn render(&self, _: &Label, _: &()) -> Element {
        Element::component("ApiExpansionRecursive")
    }
}
fn register() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        register_component::<Leaf>("ApiExpansionLeaf").unwrap();
        register_component::<Wrapper>("ApiExpansionWrapper").unwrap();
        register_component::<Recursive>("ApiExpansionRecursive").unwrap();
    });
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
        let last = self.frames.len() - 1;
        let element = self.frames[index.min(last)].clone();
        let wake = self.wake.as_ref().unwrap();
        if index >= last {
            wake.request_stop();
        } else {
            wake.request_redraw();
        }
        element
    }
}
fn run_frames(elements: Vec<Element>) -> (Result<()>, Vec<String>) {
    let output = Capture::default();
    let frames = Arc::new(Mutex::new(Vec::new()));
    let backend = ProbeBackend {
        inner: SuprTuiBackend::with_writer(32, 6, output.clone()).unwrap(),
        output,
        frames: frames.clone(),
        // a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
        deadline: Instant::now() + Duration::from_secs(30),
    };
    let result = App::builder()
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
    (result, captured)
}
/// Records the text of each frame DebugBackend presents and forwards its
/// geometry, so the App does all of its own work on each frame. After the
/// first frame it reports one resize, so the App lays the tree out again
/// through `layout_frame` before painting at the new size.
struct DebugProbe {
    inner: DebugBackend,
    frames: Arc<Mutex<Vec<String>>>,
    layouts: Arc<AtomicUsize>,
    resized: bool,
    deadline: Instant,
}
impl Backend for DebugProbe {
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn layout_frame(&mut self, element: Arc<Element>) -> Result<Option<FrameLayout>> {
        let frame = self.inner.layout_frame(element)?;
        if frame.as_ref().is_some_and(|frame| !frame.nodes.is_empty()) {
            self.layouts.fetch_add(1, Ordering::SeqCst);
        }
        Ok(frame)
    }
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        self.inner.painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        self.inner.component_layouts()
    }
    fn hit_cells(&self) -> Option<&[u32]> {
        self.inner.hit_cells()
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height)
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
        self.frames
            .lock()
            .unwrap()
            .push(self.inner.screen_content());
        Ok(())
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
        if !self.resized && !self.frames.lock().unwrap().is_empty() {
            self.resized = true;
            return Ok(Some(Event::Resize(ResizeEvent::new(40, 8))));
        }
        wake.wait(Some(
            timeout
                .unwrap_or(Duration::from_millis(10))
                .min(Duration::from_millis(10)),
        ));
        Ok(None)
    }
}

/// Draws `elements` through DebugBackend, one frame each, with one resize
/// after the first, and returns the run's result, the text of each presented
/// frame and how many layouts `layout_frame` returned.
fn run_debug_frames(elements: Vec<Element>) -> (Result<()>, Vec<String>, usize) {
    let frames = Arc::new(Mutex::new(Vec::new()));
    let layouts = Arc::new(AtomicUsize::new(0));
    let backend = DebugProbe {
        inner: DebugBackend::new(32, 6),
        frames: frames.clone(),
        layouts: layouts.clone(),
        resized: false,
        // a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
        deadline: Instant::now() + Duration::from_secs(30),
    };
    let result = App::builder()
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
    (result, captured, layouts.load(Ordering::SeqCst))
}

fn child(key: &str, text: &str) -> Element {
    Element::component_with_props("ApiExpansionWrapper", Label { text: text.into() }).key(key)
}
fn column(children: Vec<Element>) -> Element {
    Element::layout(LayoutType::Flex)
        .class("flex flex-col w-full h-full")
        .children(children)
}

#[test]
#[serial_test::serial]
fn nested_components_keep_instances_through_props_reorder_and_removal() {
    register();
    EVENTS.lock().unwrap().clear();
    NEXT_ID.store(0, Ordering::SeqCst);
    let (result, frames) = run_frames(vec![
        column(vec![child("a", "A"), child("b", "B")]),
        column(vec![child("b", "B"), child("a", "A2")]),
        column(vec![child("b", "B")]),
        column(vec![child("b", "B")]),
        column(vec![Element::text("finished")]),
    ]);
    result.unwrap();
    assert_eq!(frames.len(), 5);
    assert!(
        frames[0].contains("A:0") && frames[0].contains("B:0"),
        "{:?}",
        frames[0]
    );
    assert!(frames[1].find("B:0").unwrap() < frames[1].find("A2:1").unwrap());
    assert!(frames[2].contains("B:0") && !frames[2].contains("A2"));
    assert!(frames[4].contains("finished") && !frames[4].contains("B:0"));
    assert_eq!(
        NEXT_ID.load(Ordering::SeqCst),
        2,
        "no clone or keyed recreation"
    );
    let events = EVENTS.lock().unwrap();
    for id in 0..2 {
        assert_eq!(
            events
                .iter()
                .filter(|(i, e)| *i == id && e == "mount")
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|(i, e)| *i == id && e == "unmount")
                .count(),
            1
        );
    }
    let a_removed = events
        .iter()
        .position(|(id, e)| *id == 0 && e == "unmount")
        .unwrap();
    assert!(events[a_removed + 1..]
        .iter()
        .any(|(id, e)| *id == 1 && e.starts_with("render:")));
}

#[test]
#[serial_test::serial]
fn unknown_names_keep_their_container_children_and_plain_text_control_paints() {
    let (result, frames) = run_frames(vec![column(vec![
        Element::component("ApiUnknownContainer").child(Element::text("fallback")),
        Element::text("control"),
    ])]);
    result.unwrap();
    assert!(frames[0].contains("fallback") && frames[0].contains("control"));
}

#[test]
#[serial_test::serial]
fn duplicate_keys_fail_without_ambiguous_instance_reuse() {
    register();
    let (result, _) = run_frames(vec![column(vec![child("same", "A"), child("same", "B")])]);
    assert!(result.is_err(), "duplicate sibling keys must fail");
}

/// The error that stops expansion one level past its depth bound.
const DEPTH_ERROR: &str = "component tree exceeds expansion depth 128";

/// Set in the child process that `run_in_child` starts.
const CHILD: &str = "RTUI_EXPANSION_STACK_CHILD";

/// Runs the test `name` again in a child process with `env` added to its
/// environment, and returns whether the child exited successfully and its
/// output. A stack overflow aborts the process it happens in, so in the
/// child it fails the calling test instead of ending the whole run.
fn run_in_child(name: &str, env: &[(&str, &str)]) -> (bool, String) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

/// Whether this process is the child `run_in_child` started. The child
/// writes no core file when it aborts, so each failing run does not leave
/// one behind.
fn in_child() -> bool {
    if std::env::var_os(CHILD).is_none() {
        return false;
    }
    #[cfg(unix)]
    {
        let none = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: setrlimit only reads `none` during the call.
        assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_CORE, &none) }, 0);
    }
    true
}

/// Nests `depth` flex columns around a text, so the text sits at that depth.
fn nested(depth: usize) -> Element {
    (0..depth).fold(Element::text("deepest"), |inner, _| column(vec![inner]))
}

#[test]
#[serial_test::serial]
fn recursive_expansion_fails_with_a_bounded_error() {
    register();
    let (result, _) = run_frames(vec![Element::component("ApiExpansionRecursive")]);
    let error = result.expect_err("recursive component output must be bounded");
    assert!(
        error.to_string().contains(DEPTH_ERROR),
        "unexpected error: {error}"
    );
}

#[test]
#[serial_test::serial]
fn expansion_refuses_a_tree_one_level_past_its_bound() {
    let (result, _) = run_frames(vec![nested(129)]);
    let error = result.expect_err("a tree deeper than the bound must fail");
    assert!(
        error.to_string().contains(DEPTH_ERROR),
        "unexpected error: {error}"
    );
}

/// The depth bound is reached within a 1.5 MiB stack, well under a test
/// thread's 2 MiB. Each expansion level stays small enough that a build
/// whose libraries keep more thread-local storage still fits: glibc takes a
/// thread's static thread-local storage out of its stack, and the
/// embedded-terminal feature's terminal library keeps 256 KiB of it.
#[test]
fn recursive_expansion_reaches_its_bound_within_a_small_stack() {
    const NAME: &str = "recursive_expansion_reaches_its_bound_within_a_small_stack";
    if in_child() {
        let error = std::thread::Builder::new()
            .stack_size(1536 * 1024)
            .spawn(|| {
                register();
                run_frames(vec![Element::component("ApiExpansionRecursive")])
                    .0
                    .err()
                    .map(|error| error.to_string())
            })
            .unwrap()
            .join()
            .unwrap();
        let error = error.expect("recursive component output must be bounded");
        assert!(error.contains(DEPTH_ERROR), "unexpected error: {error}");
        return;
    }
    let (passed, text) = run_in_child(NAME, &[]);
    assert!(
        passed,
        "expansion to its depth bound does not fit a 1.5 MiB stack: {text}"
    );
    assert!(
        text.contains("1 passed"),
        "the child ran no expansion: {text}"
    );
}

/// The deepest tree expansion accepts also draws through SuprTuiBackend, and
/// so through the Crossterm and direct-TTY backends built on it. Its renderer
/// thread lays out and paints, recursing once per element level, so it must
/// not rely on the default thread stack, which RUST_MIN_STACK or a library's
/// thread-local storage can shrink. The child's default stack is 1 MiB,
/// less than the renderer needs at this depth, and
/// its App runs on a thread with room to spare, so only the renderer's own
/// stack decides the result.
#[test]
fn the_deepest_accepted_tree_draws_through_suprtui_whatever_the_default_thread_stack() {
    const NAME: &str =
        "the_deepest_accepted_tree_draws_through_suprtui_whatever_the_default_thread_stack";
    if in_child() {
        let (result, frames) = std::thread::Builder::new()
            .stack_size(8 << 20)
            .spawn(|| {
                let (result, frames) = run_frames(vec![nested(128)]);
                (result.map_err(|error| error.to_string()), frames)
            })
            .unwrap()
            .join()
            .unwrap();
        result.unwrap();
        assert!(
            frames[0].contains("deepest"),
            "the innermost text was not drawn: {frames:?}"
        );
        return;
    }
    let (passed, text) = run_in_child(NAME, &[("RUST_MIN_STACK", "1048576")]);
    assert!(
        passed,
        "the deepest accepted tree does not draw with a 1 MiB default thread stack: {text}"
    );
    assert!(text.contains("1 passed"), "the child drew nothing: {text}");
}

/// The deepest tree expansion accepts also draws through DebugBackend from
/// an app thread with a 1.5 MiB stack, before and after a resize. Laying
/// out and painting a tree 128 levels deep takes about 1.8 MiB of stack in
/// a debug build, more than a test thread has left with the
/// embedded-terminal feature, so DebugBackend does that work on a thread
/// with its own stack, as SuprTuiBackend does, and the app thread's stack
/// does not decide whether the tree draws. The child's default stack is
/// 1 MiB, less than that work needs, so a paint thread that took the
/// default stack would fail here too.
#[test]
fn the_deepest_accepted_tree_draws_through_debug_backend_from_a_small_app_thread() {
    const NAME: &str =
        "the_deepest_accepted_tree_draws_through_debug_backend_from_a_small_app_thread";
    if in_child() {
        let (result, frames, layouts) = std::thread::Builder::new()
            .stack_size(1536 * 1024)
            .spawn(|| {
                let (result, frames, layouts) = run_debug_frames(vec![nested(128), nested(128)]);
                (result.map_err(|error| error.to_string()), frames, layouts)
            })
            .unwrap()
            .join()
            .unwrap();
        result.unwrap();
        assert!(
            frames.len() >= 2 && frames.iter().all(|frame| frame.contains("deepest")),
            "the innermost text was not drawn before and after the resize: {frames:?}"
        );
        assert_eq!(layouts, 1, "the resize did not lay the tree out again");
        return;
    }
    let (passed, text) = run_in_child(NAME, &[("RUST_MIN_STACK", "1048576")]);
    assert!(
        passed,
        "the deepest accepted tree does not draw through DebugBackend from a 1.5 MiB app thread: {text}"
    );
    assert!(text.contains("1 passed"), "the child drew nothing: {text}");
}

#[test]
#[serial_test::serial]
fn replacing_component_type_at_a_key_releases_old_state() {
    register();
    EVENTS.lock().unwrap().clear();
    NEXT_ID.store(0, Ordering::SeqCst);
    let (result, frames) = run_frames(vec![
        column(vec![child("same", "old")]),
        column(vec![Element::component_with_props(
            "ApiExpansionLeaf",
            Label { text: "new".into() },
        )
        .key("same")]),
        column(vec![Element::text("done")]),
    ]);
    result.unwrap();
    assert!(frames[0].contains("old:0"));
    assert!(frames[1].contains("new:0") && !frames[1].contains("old"));
    assert_eq!(NEXT_ID.load(Ordering::SeqCst), 2);
    let events = EVENTS.lock().unwrap();
    for id in 0..2 {
        assert_eq!(
            events
                .iter()
                .filter(|(i, e)| *i == id && e == "mount")
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|(i, e)| *i == id && e == "unmount")
                .count(),
            1
        );
    }
    assert!(
        events
            .iter()
            .position(|(id, e)| *id == 0 && e == "unmount")
            .unwrap()
            < events
                .iter()
                .position(|(id, e)| *id == 1 && e == "mount")
                .unwrap()
    );
}
