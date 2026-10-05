//! Part of tests/review_core.rs.

use crate::common::app_input::{run_when_for, run_when_painted};
use reactive_tui::{
    app::RootComponent,
    backend::SuprTuiBackend,
    builder::core::div,
    component::{registry, Component, Element, LifecycleEvent, Props},
    event::types::{Event, MouseButton, MouseEvent, MouseEventKind, Position},
    hooks::{use_clicks, use_hover, use_mouse_position, MouseEventProcessor},
    reactive::Hooks,
    render::{tree::element_to_render_node, NodeKey, RenderTree},
    screen::ScreenManager,
    vdom::{diff_vnodes, Patch, VNode},
};
use std::{
    any::Any,
    io::{self, Write},
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, Once,
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

#[derive(Default)]
struct PollProbe {
    polls: AtomicUsize,
    ready: AtomicBool,
    text: Mutex<String>,
    waker: Mutex<Option<Waker>>,
    woke_at: Mutex<Option<Instant>>,
    unmounts: AtomicUsize,
}

#[derive(Clone, Default)]
struct ProbeProps(Arc<PollProbe>);
impl PartialEq for ProbeProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Props for ProbeProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct PanelRoot(Element);
impl RootComponent for PanelRoot {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

struct PollingPanel {
    probe: Arc<PollProbe>,
    text: String,
}
impl Component for PollingPanel {
    type Props = ProbeProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        let text = props.0.text.lock().unwrap().clone();
        Self {
            probe: props.0,
            text,
        }
    }
    fn render(&self, _: &Self::Props, _: &Self::State) -> Element {
        div().class("w-20 h-1").text(&self.text).build()
    }
    fn poll_change(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let panel = self.get_mut();
        panel.probe.polls.fetch_add(1, Ordering::SeqCst);
        *panel.probe.waker.lock().unwrap() = Some(cx.waker().clone());
        if panel.probe.ready.swap(false, Ordering::SeqCst) {
            panel.text = panel.probe.text.lock().unwrap().clone();
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.probe.unmounts.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn panel(probe: &Arc<PollProbe>, key: &str) -> Element {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| registry::register_component::<PollingPanel>("CmpPollingPanel").unwrap());
    Element::component("CmpPollingPanel")
        .with_props(ProbeProps(probe.clone()))
        .key(key)
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Capture {
    fn text(&self) -> String {
        let mut parser = vt100::Parser::new(4, 20, 0);
        parser.process(&self.0.lock().unwrap());
        parser.screen().contents()
    }
}
fn manager(output: &Capture) -> ScreenManager {
    ScreenManager::new(Box::new(
        SuprTuiBackend::with_writer(20, 4, output.clone()).unwrap(),
    ))
}

#[test]
fn cmp_001_dropping_an_old_tree_keeps_the_replacement_mounted() {
    let old = Arc::new(PollProbe::default());
    let new = Arc::new(PollProbe::default());
    let mut older = RenderTree::new();
    older.set_root(element_to_render_node(panel(&old, "panel")));
    let mut newer = RenderTree::new();
    newer.set_root(element_to_render_node(panel(&new, "panel")));
    drop(older);
    let registered = registry::get_global_registry()
        .get_instance(&NodeKey::named("panel"))
        .unwrap()
        .is_some();
    let unmounts = new.unmounts.load(Ordering::SeqCst);
    assert!(
        registered && unmounts == 0,
        "CMP-001: after dropping the older tree, panel registered={registered}, replacement unmounts={unmounts}"
    );
}

#[test]
fn cmp_002_app_polls_a_component_when_it_mounts() {
    let probe = Arc::new(PollProbe::default());
    *probe.text.lock().unwrap() = "before".into();
    let element = panel(&probe, "cmp-app-mount");
    run_when_painted(PanelRoot(element), (20, 4), 1);
    let polls = probe.polls.load(Ordering::SeqCst);
    assert!(
        polls > 0,
        "CMP-002: App mounted the component but poll count is {polls}"
    );
}

fn wake_panel(probe: Arc<PollProbe>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let waker = loop {
            if let Some(waker) = probe.waker.lock().unwrap().clone() {
                break Some(waker);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        *probe.text.lock().unwrap() = "after".into();
        probe.ready.store(true, Ordering::SeqCst);
        *probe.woke_at.lock().unwrap() = Some(Instant::now());
        if let Some(waker) = waker {
            waker.wake();
        }
    })
}

/// How long after the worker's wake `shown` came, if it woke at all.
fn since_wake(probe: &PollProbe, shown: Instant) -> Option<Duration> {
    probe
        .woke_at
        .lock()
        .unwrap()
        .map(|woke| shown.saturating_duration_since(woke))
}

#[test]
fn cmp_002_app_repolls_and_paints_after_the_component_wakes() {
    let probe = Arc::new(PollProbe::default());
    *probe.text.lock().unwrap() = "before".into();
    let element = panel(&probe, "cmp-app-wake");
    let worker = wake_panel(probe.clone());
    let frames = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_when_for(
            PanelRoot(element),
            (20, 4),
            vec![("after", None)],
            Duration::from_secs(6),
        )
    }));
    // run_when_for returns at the frame that shows the new text.
    let shown = Instant::now();
    worker.join().unwrap();
    let polls = probe.polls.load(Ordering::SeqCst);
    let painted =
        frames.is_ok_and(|frames| frames.iter().any(|frame| frame.text.contains("after")));
    let delay = since_wake(&probe, shown);
    assert!(
        polls >= 2 && painted && delay.is_some_and(|delay| delay <= Duration::from_secs(1)),
        "CMP-002: after the component wake, App poll count is {polls}, new text painted={painted}, {delay:?} after the wake"
    );
}

#[test]
fn cmp_002_screen_manager_polls_a_component_when_it_mounts() {
    let probe = Arc::new(PollProbe::default());
    *probe.text.lock().unwrap() = "before".into();
    let output = Capture::default();
    let mut screens = manager(&output);
    screens
        .create_screen("mount", "Mount".into(), panel(&probe, "cmp-screen-mount"))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while probe.polls.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        screens.update().unwrap();
        std::thread::sleep(Duration::from_millis(5));
    }
    let polls = probe.polls.load(Ordering::SeqCst);
    assert!(
        polls > 0,
        "CMP-002: ScreenManager mounted the component but poll count is {polls}"
    );
}

#[test]
fn cmp_002_screen_manager_repolls_and_paints_after_the_component_wakes() {
    let probe = Arc::new(PollProbe::default());
    *probe.text.lock().unwrap() = "before".into();
    let output = Capture::default();
    let mut screens = manager(&output);
    screens
        .create_screen("wake", "Wake".into(), panel(&probe, "cmp-screen-wake"))
        .unwrap();
    let worker = wake_panel(probe.clone());
    let deadline = Instant::now() + Duration::from_secs(6);
    let mut shown = None;
    while Instant::now() < deadline {
        screens.update().unwrap();
        screens.sync().unwrap();
        if output.text().contains("after") {
            shown = Some(Instant::now());
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    worker.join().unwrap();
    let polls = probe.polls.load(Ordering::SeqCst);
    let delay = shown.and_then(|shown| since_wake(&probe, shown));
    assert!(
        polls >= 2 && delay.is_some_and(|delay| delay <= Duration::from_secs(1)),
        "CMP-002: after the component wake, ScreenManager poll count is {polls}, new text shown {delay:?} after the wake"
    );
}

#[test]
fn cmp_002_pin_projection_does_not_claim_an_infallible_pointer_check() {
    let source = include_str!("../../src/component/instance.rs");
    let start = source.rfind("fn poll_change_any(").unwrap();
    let method = &source[start..source[start..].find("\n    }\n").unwrap() + start];
    let expressions: Vec<_> = method
        .lines()
        .filter_map(|line| {
            let (_, expression) = line.trim().strip_prefix("let ")?.split_once(" = ")?;
            (expression.contains("get_ref()") && expression.contains("*const"))
                .then_some(expression.trim_end_matches(';'))
        })
        .collect();
    let duplicate = expressions
        .iter()
        .enumerate()
        .any(|(i, expression)| expressions[i + 1..].contains(expression));
    assert!(
        !duplicate,
        "CMP-002: poll_change_any still compares pointers obtained from the same reference expression"
    );
}

/// The poll pins the component field while every other method takes the
/// component by `&mut`, which may move it; that is sound only because a
/// component is `Unpin`, so this compiles only while the trait says so.
#[test]
fn cmp_002_a_component_is_unpin_so_pinning_it_for_a_poll_is_sound() {
    fn is_unpin<T: Unpin>() {}
    fn component_is_unpin<C: Component>() {
        is_unpin::<C>();
    }
    component_is_unpin::<PollingPanel>();
}

#[reactive_tui::component]
fn CmpMousePanel(hooks: &Hooks) -> Element {
    let hover = use_hover(hooks);
    let clicks = use_clicks(hooks);
    div()
        .class("w-20 h-1")
        .text(&format!(
            "hover:{} clicks:{}",
            hover.get().is_hovered,
            clicks.get().click_count
        ))
        .build()
}

fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left))
}
fn mouse_screen(output: &Capture) -> ScreenManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| registry::register_component::<CmpMousePanel>("CmpMousePanel").unwrap());
    let mut screens = manager(output);
    screens
        .create_screen(
            "mouse",
            "Mouse".into(),
            CmpMousePanel::element().key("cmp-mouse"),
        )
        .unwrap();
    screens.sync().unwrap();
    screens
}

#[test]
fn cmp_003_screen_mouse_move_updates_the_hover_hook() {
    let output = Capture::default();
    let mut screens = mouse_screen(&output);
    screens
        .process_event(&mouse(MouseEventKind::Move, 1, 0))
        .unwrap();
    screens.update().unwrap();
    screens.sync().unwrap();
    let text = output.text();
    assert!(
        text.contains("hover:true"),
        "CMP-003: after moving into the screen component, frame is {text:?}"
    );
}

#[test]
fn cmp_003_screen_press_and_release_update_the_click_hook() {
    let output = Capture::default();
    let mut screens = mouse_screen(&output);
    screens
        .process_event(&mouse(MouseEventKind::Down, 1, 0))
        .unwrap();
    screens
        .process_event(&mouse(MouseEventKind::Up, 1, 0))
        .unwrap();
    screens.update().unwrap();
    screens.sync().unwrap();
    let text = output.text();
    assert!(
        text.contains("clicks:1"),
        "CMP-003: after a screen press and release, frame is {text:?}"
    );
}

fn position_after_move(target: Option<&str>) -> bool {
    let hooks = Hooks::new();
    let position = use_mouse_position(&hooks);
    let processor = MouseEventProcessor::new();
    processor.register_position("A".into(), position.clone());
    processor.process_event(
        &MouseEvent::new(MouseEventKind::Move, Position::cell(1, 0)),
        Some("A"),
    );
    processor.process_event(
        &MouseEvent::new(MouseEventKind::Move, Position::cell(15, 0)),
        target,
    );
    position.get().is_inside
}

#[test]
fn cmp_004_a_move_over_b_clears_a_is_inside() {
    let inside = position_after_move(Some("B"));
    assert!(
        !inside,
        "CMP-004: after a move over B, A's position still reports is_inside true"
    );
}

#[test]
fn cmp_004_a_move_over_no_component_clears_a_is_inside() {
    let inside = position_after_move(None);
    assert!(
        !inside,
        "CMP-004: after a move over no component, A's position still reports is_inside true"
    );
}

fn replaces(old: VNode, new: VNode, change: &str) {
    let patches = diff_vnodes(&old, &new);
    assert!(
        patches
            .patches()
            .iter()
            .any(|patch| matches!(patch, Patch::Replace { index: 0, .. })),
        "CMP-005: changing only {change} produced no Replace for the root: {:?}",
        patches.patches()
    );
}

#[test]
fn cmp_005_a_changed_element_name_replaces_the_keyed_node() {
    let old = VNode::element("flex").key("panel");
    let mut new = old.clone();
    new.tag = "grid".into();
    replaces(old.build(), new.build(), "element name");
}

#[test]
fn cmp_005_a_changed_element_prop_replaces_the_node() {
    let old = VNode::element("flex").prop("value", 1_u32);
    let new = old.clone().prop("value", 2_u32);
    replaces(old.build(), new.build(), "element prop");
}

#[test]
fn cmp_005_a_changed_handler_replaces_the_node() {
    let old = VNode::element("flex").on("click", |_| {});
    let new = old.clone().on("click", |_| {});
    replaces(old.build(), new.build(), "event handler");
}

#[test]
fn cmp_005_a_changed_component_name_replaces_the_node() {
    let old = VNode::component("Old").key("panel");
    let mut new = old.clone();
    new.name = "New".into();
    replaces(old.build(), new.build(), "component name");
}

#[test]
fn cmp_005_a_changed_component_prop_replaces_the_node() {
    let old = VNode::component("Panel").props(1_u32);
    let new = old.clone().props(2_u32);
    replaces(old.build(), new.build(), "component prop");
}

#[test]
fn cmp_007_source_removes_the_unused_table_and_its_compile_time_claims() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![root.clone()];
    let mut violations = Vec::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                std::fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = std::fs::read_to_string(&path).unwrap();
            let component = path.starts_with(root.join("component"));
            for (index, line) in source.lines().enumerate() {
                let line = line.trim();
                let lower = line.to_ascii_lowercase().replace('-', " ");
                let definition = line.contains("struct CommonComponents")
                    || line.contains("fn common_components(");
                let claim = component
                    && (line.starts_with("///") || line.starts_with("//!"))
                    && (lower.contains("compile time") || lower.contains("perfect hash"));
                if definition || claim {
                    violations.push(format!(
                        "{}:{}: {line}",
                        path.strip_prefix(&root).unwrap().display(),
                        index + 1
                    ));
                }
            }
        }
    }
    violations.sort();
    assert!(
        violations.is_empty(),
        "CMP-007: source still offers the empty lookup or its claims: {}",
        violations.join("; ")
    );
}
