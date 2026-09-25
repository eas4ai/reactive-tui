use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::{Backend, DebugBackend, PaintedNode, SuprTuiBackend},
    component::{Element, ElementType},
    core::surface::Attr,
    error::Result,
    event::types::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, Position},
    render::{reconcile::PatchOp, RenderTree},
};
use std::{
    collections::VecDeque,
    fmt::Write as _,
    io::{self, Write},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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
pub struct Snapshot {
    #[allow(dead_code)]
    pub output: Vec<u8>,
    /// Main-loop work for this frame: from the input wait returning to present.
    #[allow(dead_code)]
    pub work_ms: f64,
    /// Of `work_ms`, the time the backend's present took: the worker's
    /// layout and paint. The rest is the App's own work.
    #[allow(dead_code)]
    pub present_ms: f64,
    pub text: String,
    #[allow(dead_code)]
    pub screen: vt100::Screen,
    #[allow(dead_code)]
    pub geometry: Vec<PaintedNode>,
    /// Whether an element in this frame was marked busy: a widget such as a
    /// chart was still preparing its content. Every step waits for a frame
    /// that is not busy, and a step's frame count counts only such frames.
    #[allow(dead_code)]
    pub busy: bool,
    /// The texts of this frame's live regions (elements with the
    /// `aria-live-polite` class): what a screen reader announces.
    #[allow(dead_code)]
    pub live: Vec<String>,
}

/// The texts of the live regions in `element` and its descendants.
fn live_texts(element: &Element, texts: &mut Vec<String>) {
    let live = element
        .class
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|c| c == "aria-live-polite"));
    if let (true, ElementType::Text(text)) = (live, &element.element_type) {
        texts.push(text.clone());
    }
    for child in &element.children {
        live_texts(child, texts);
    }
}

/// Whether `element` or a descendant is marked busy.
fn busy(element: &Element) -> bool {
    element
        .metadata
        .accessibility
        .as_ref()
        .is_some_and(|node| node.is_busy())
        || element.children.iter().any(busy)
}
struct Step {
    frame: usize,
    text: Vec<String>,
    absent: Vec<String>,
    occurrences: usize,
    event: Option<Event>,
    pointer_text: Option<(String, u16, Option<f32>)>,
    cell: Option<(u16, u16, String)>,
    output: Option<(String, usize)>,
    /// Wait until the latest frame holds any visible text.
    painted: bool,
}

/// The backend the App presents to: the terminal backend writing into the
/// capture, or the debug backend painting into memory.
enum Inner {
    SuprTui(Box<SuprTuiBackend>),
    Debug(Box<DebugBackend>),
}

impl Inner {
    fn backend(&self) -> &dyn Backend {
        match self {
            Inner::SuprTui(backend) => backend.as_ref(),
            Inner::Debug(backend) => backend.as_ref(),
        }
    }
    fn backend_mut(&mut self) -> &mut dyn Backend {
        match self {
            Inner::SuprTui(backend) => backend.as_mut(),
            Inner::Debug(backend) => backend.as_mut(),
        }
    }
}

/// The debug backend's presented frame written as ANSI: one absolute move per
/// row and each cell's own colors and attributes before its text, so one
/// vt100 screen model inspects both backends. Lossless for what the painter
/// writes: 24-bit colors and the bold, italic, underline, inverse and
/// strikethrough attributes.
fn debug_output(backend: &DebugBackend) -> Vec<u8> {
    let (width, height) = backend.size();
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut out = Vec::new();
    for y in 0..usize::from(height) {
        write!(out, "\x1b[{};1H", y + 1).unwrap();
        for x in 0..usize::from(width) {
            let (Some(text), Some(cell)) = (backend.cell_text(x, y), backend.cell(x, y)) else {
                continue;
            };
            // The second cell of a wide grapheme: the terminal advances past it.
            if text.is_empty() {
                continue;
            }
            let mut sgr = String::from("0");
            for (attribute, code) in [
                (Attr::BOLD, 1),
                (Attr::ITALIC, 3),
                (Attr::UNDERLINE, 4),
                (Attr::REVERSE, 7),
                (Attr::STRIKE, 9),
            ] {
                if cell.attr.contains(attribute) {
                    write!(sgr, ";{code}").unwrap();
                }
            }
            write!(
                out,
                "\x1b[{sgr};38;2;{};{};{};48;2;{};{};{}m{text}",
                byte(cell.fg.r),
                byte(cell.fg.g),
                byte(cell.fg.b),
                byte(cell.bg.r),
                byte(cell.bg.g),
                byte(cell.bg.b),
            )
            .unwrap();
        }
    }
    out.extend_from_slice(b"\x1b[0m");
    out
}

struct InputBackend {
    inner: Inner,
    events: VecDeque<Step>,
    capture: Capture,
    snapshots: Arc<Mutex<Vec<Snapshot>>>,
    deadline: Instant,
    wait_returned: std::sync::Mutex<Option<Instant>>,
    /// Whether the element tree of the frame being rendered is busy.
    busy: bool,
    /// The live-region texts of the frame being rendered.
    live: Vec<String>,
}
impl Backend for InputBackend {
    fn painted_nodes(&self) -> Option<&[reactive_tui::backend::PaintedNode]> {
        self.inner.backend().painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[reactive_tui::backend::PresentedLayout]> {
        self.inner.backend().component_layouts()
    }
    fn hit_cells(&self) -> Option<&[u32]> {
        self.inner.backend().hit_cells()
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.busy = busy(element);
        self.live.clear();
        live_texts(element, &mut self.live);
        self.inner.backend_mut().render_frame(element)
    }
    fn layout_frame(
        &mut self,
        element: Arc<Element>,
    ) -> Result<Option<reactive_tui::backend::FrameLayout>> {
        self.inner.backend_mut().layout_frame(element)
    }
    fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
        panic!("complete frame required")
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.backend_mut().clear()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.backend().size()
    }
    fn present(&mut self) -> Result<()> {
        let started = Instant::now();
        self.inner.backend_mut().present()?;
        let present_ms = started.elapsed().as_secs_f64() * 1000.0;
        // Main-loop work ends when the backend has presented; parsing the
        // captured output below is the harness's own cost, not the App's.
        let work_ms = self
            .wait_returned
            .lock()
            .unwrap()
            .map_or(0.0, |t| t.elapsed().as_secs_f64() * 1000.0);
        let output = match &mut self.inner {
            // The bytes land after present returns (PIP-001); wait for them
            // before reading the capture.
            Inner::SuprTui(backend) => {
                backend.sync()?;
                self.capture.0.lock().unwrap().clone()
            }
            Inner::Debug(backend) => debug_output(backend),
        };
        let (width, height) = self.inner.backend().size();
        let mut parser = vt100::Parser::new(height, width, 0);
        parser.process(&output);
        self.snapshots.lock().unwrap().push(Snapshot {
            work_ms,
            present_ms,
            text: parser.screen().contents(),
            screen: parser.screen().clone(),
            output,
            geometry: self
                .inner
                .backend()
                .painted_nodes()
                .map(<[PaintedNode]>::to_vec)
                .unwrap_or_default(),
            busy: self.busy,
            live: self.live.clone(),
        });
        Ok(())
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.backend_mut().resize(width, height);
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.backend_mut().shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        panic!("wake-aware input required")
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        if Instant::now() >= self.deadline {
            let snapshots = self.snapshots.lock().unwrap();
            panic!(
                "App painted {} frames but input needs {:?} before the deadline (last frame busy: {}). Last frame:\n{}",
                snapshots.len(),
                self.events.front().map(|step| (step.frame, &step.text)),
                snapshots.last().is_some_and(|frame| frame.busy),
                snapshots.last().map_or("", |frame| frame.text.as_str())
            );
        }
        if self.events.front().is_some_and(|step| {
            let frames = self.snapshots.lock().unwrap();
            frames.last().is_some_and(|frame| frame.busy)
                || step.output.as_ref().is_some_and(|(needle, count)| {
                    frames.last().is_none_or(|frame| {
                        String::from_utf8_lossy(&frame.output)
                            .matches(needle.as_str())
                            .count()
                            < *count
                    })
                })
                || step.cell.as_ref().is_some_and(|(x, y, content)| {
                    frames
                        .last()
                        .and_then(|frame| frame.screen.cell(*y, *x))
                        .is_none_or(|cell| cell.contents() != *content)
                })
                || step.frame > frames.iter().filter(|frame| !frame.busy).count()
                || frames
                    .iter()
                    .filter(|frame| step.text.iter().all(|text| frame.text.contains(text)))
                    .count()
                    < step.occurrences
                || step
                    .text
                    .iter()
                    .any(|text| frames.last().is_none_or(|frame| !frame.text.contains(text)))
                || step
                    .absent
                    .iter()
                    .any(|text| frames.last().is_none_or(|frame| frame.text.contains(text)))
                || (step.painted
                    && frames
                        .last()
                        .is_none_or(|frame| frame.text.trim().is_empty()))
        }) {
            wake.wait(Some(Duration::from_millis(1)));
            *self.wait_returned.lock().unwrap() = Some(Instant::now());
            return Ok(None);
        }
        let event = self.events.pop_front().and_then(|step| {
            if let Some((text, offset, wheel)) = step.pointer_text {
                let snapshots = self.snapshots.lock().unwrap();
                let screen = &snapshots.last().unwrap().screen;
                let (height, width) = screen.size();
                let length = text.chars().count() as u16;
                assert!(text.is_ascii() && offset < length && length <= width);
                for y in 0..height {
                    for x in 0..=width - length {
                        if text.chars().enumerate().all(|(i, expected)| {
                            screen.cell(y, x + i as u16).unwrap().contents() == expected.to_string()
                        }) {
                            return if let Some(delta) = wheel {
                                let mut event = MouseEvent::new(
                                    MouseEventKind::Wheel,
                                    Position::cell(x + offset, y),
                                );
                                event.wheel = Some(reactive_tui::event::types::WheelEvent {
                                    delta: reactive_tui::event::types::WheelDelta::Lines {
                                        x: 0.0,
                                        y: delta,
                                    },
                                    phase: reactive_tui::event::types::WheelPhase::Changed,
                                });
                                Some(Event::Mouse(event))
                            } else {
                                click(x + offset, y)
                            };
                        }
                    }
                }
                panic!(
                    "Click target {text:?} is not painted: {}",
                    screen.contents()
                );
            }
            step.event
        });
        if event.is_none() {
            wake.request_stop();
        }
        *self.wait_returned.lock().unwrap() = Some(Instant::now());
        Ok(event)
    }
}

/// How long an App run may take to reach every step before it fails instead
/// of hanging. It is a hang guard, not a timing check: frame work is
/// measured separately (BAR-005), and a busy machine must not fail a correct
/// test by running it slowly.
#[allow(dead_code)]
pub const HANG_GUARD: Duration = Duration::from_secs(30);

pub fn run(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(usize, Option<Event>)>,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        steps
            .into_iter()
            .map(|(frame, event)| Step {
                frame,
                text: Vec::new(),
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: None,
                painted: false,
            })
            .collect(),
    )
}

/// Paint at least `frames` frames and stop at the first one after that
/// which holds visible text, so a widget whose worker finishes between two
/// frames is observed painted rather than blank.
#[allow(dead_code)]
pub fn run_when_painted(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    frames: usize,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        VecDeque::from(vec![Step {
            frame: frames,
            text: Vec::new(),
            absent: Vec::new(),
            occurrences: 1,
            event: None,
            pointer_text: None,
            cell: None,
            output: None,
            painted: true,
        }]),
    )
}

/// Gate input on presented content when a worker may finish between any two frames.
#[allow(dead_code)]
pub fn run_when(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(&str, Option<Event>)>,
) -> Vec<Snapshot> {
    run_when_for(root, size, steps, HANG_GUARD)
}

#[allow(dead_code)]
pub fn run_when_for(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(&str, Option<Event>)>,
    timeout: Duration,
) -> Vec<Snapshot> {
    run_steps_with_images(
        root,
        size,
        steps
            .into_iter()
            .map(|(text, event)| Step {
                frame: 1,
                text: vec![text.into()],
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: None,
                painted: false,
            })
            .collect(),
        None,
        timeout,
    )
}

#[allow(dead_code)]
pub fn run_when_all(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(&[&str], Option<Event>)>,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        steps
            .into_iter()
            .map(|(text, event)| Step {
                frame: 1,
                text: text.iter().map(|text| (*text).into()).collect(),
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: None,
                painted: false,
            })
            .collect(),
    )
}

/// Stop only after the dismissed content has disappeared from a presented frame.
#[allow(dead_code)]
pub fn run_until_hidden(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(&str, Option<Event>)>,
    hidden: &str,
) -> Vec<Snapshot> {
    let mut steps: VecDeque<_> = steps
        .into_iter()
        .map(|(text, event)| Step {
            frame: 1,
            text: vec![text.into()],
            absent: Vec::new(),
            occurrences: 1,
            event,
            pointer_text: None,
            cell: None,
            output: None,
            painted: false,
        })
        .collect();
    steps.push_back(Step {
        frame: 1,
        text: Vec::new(),
        absent: vec![hidden.into()],
        occurrences: 1,
        event: None,
        pointer_text: None,
        cell: None,
        output: None,
        painted: false,
    });
    run_steps(root, size, steps)
}

#[allow(dead_code)]
pub enum Action {
    Event(Event),
    /// Click the given ASCII text in the current presented frame, at a cell offset.
    ClickText(&'static str, u16),
    /// Scroll over text in the current presented frame.
    WheelText(&'static str, f32),
}

#[allow(dead_code)]
pub fn run_actions_until_hidden(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    actions: Vec<(&str, Action)>,
    hidden: &str,
) -> Vec<Snapshot> {
    let mut steps: VecDeque<_> = actions
        .into_iter()
        .map(|(text, action)| {
            let (event, pointer_text) = match action {
                Action::Event(event) => (Some(event), None),
                Action::ClickText(text, offset) => (None, Some((text.into(), offset, None))),
                Action::WheelText(text, delta) => (None, Some((text.into(), 0, Some(delta)))),
            };
            Step {
                frame: 1,
                text: vec![text.into()],
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text,
                cell: None,
                output: None,
                painted: false,
            }
        })
        .collect();
    steps.push_back(Step {
        frame: 1,
        text: Vec::new(),
        absent: vec![hidden.into()],
        occurrences: 1,
        event: None,
        pointer_text: None,
        cell: None,
        output: None,
        painted: false,
    });
    run_steps(root, size, steps)
}

/// Gate each event on both visible and dismissed content.
#[allow(dead_code)]
pub fn run_visibility(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(&str, Option<&str>, Option<Event>)>,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        steps
            .into_iter()
            .map(|(text, absent, event)| Step {
                frame: 1,
                text: vec![text.into()],
                absent: absent.into_iter().map(str::to_owned).collect(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: None,
                painted: false,
            })
            .collect(),
    )
}

/// Wait for several actual content-bearing frames, independently of setup renders.
#[allow(dead_code)]
pub fn run_when_seen(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    text: &[&str],
    occurrences: usize,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        VecDeque::from([Step {
            frame: 1,
            text: text.iter().map(|text| (*text).into()).collect(),
            absent: Vec::new(),
            occurrences,
            event: None,
            pointer_text: None,
            cell: None,
            output: None,
            painted: false,
        }]),
    )
}

/// Gate input on an independently expected cell after geometry changes.
#[allow(dead_code)]
pub struct CellStep {
    pub x: u16,
    pub y: u16,
    pub content: &'static str,
    pub event: Option<Event>,
}
#[allow(dead_code)]
pub fn run_when_cell(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<CellStep>,
) -> Vec<Snapshot> {
    run_steps(
        root,
        size,
        steps
            .into_iter()
            .map(|step| Step {
                frame: 1,
                text: Vec::new(),
                absent: Vec::new(),
                occurrences: 1,
                event: step.event,
                pointer_text: None,
                cell: Some((step.x, step.y, step.content.into())),
                output: None,
                painted: false,
            })
            .collect(),
    )
}

fn run_steps(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: VecDeque<Step>,
) -> Vec<Snapshot> {
    run_steps_with_images(root, size, steps, None, HANG_GUARD)
}

/// `run` on the debug backend, which paints into memory with the SuprTUI
/// painter and writes no terminal bytes.
#[allow(dead_code)]
pub fn run_on_debug(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(usize, Option<Event>)>,
) -> Vec<Snapshot> {
    run_steps_on(
        root,
        size,
        steps
            .into_iter()
            .map(|(frame, event)| Step {
                frame,
                text: Vec::new(),
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: None,
                painted: false,
            })
            .collect(),
        Target::Debug,
        HANG_GUARD,
    )
}

/// `run_when_painted` on the debug backend.
#[allow(dead_code)]
pub fn run_when_painted_on_debug(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    frames: usize,
) -> Vec<Snapshot> {
    run_steps_on(
        root,
        size,
        VecDeque::from(vec![Step {
            frame: frames,
            text: Vec::new(),
            absent: Vec::new(),
            occurrences: 1,
            event: None,
            pointer_text: None,
            cell: None,
            output: None,
            painted: true,
        }]),
        Target::Debug,
        HANG_GUARD,
    )
}

/// Which backend a run presents to.
enum Target {
    SuprTui(Option<reactive_tui::backend::ImageOutputOptions>),
    Debug,
}

#[allow(dead_code)]
pub fn run_when_output(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    images: reactive_tui::backend::ImageOutputOptions,
    steps: Vec<(String, usize, Option<Event>)>,
) -> Vec<Snapshot> {
    run_steps_with_images(
        root,
        size,
        steps
            .into_iter()
            .map(|(needle, count, event)| Step {
                frame: 1,
                text: Vec::new(),
                absent: Vec::new(),
                occurrences: 1,
                event,
                pointer_text: None,
                cell: None,
                output: Some((needle, count)),
                painted: false,
            })
            .collect(),
        Some(images),
        HANG_GUARD,
    )
}
fn run_steps_with_images(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: VecDeque<Step>,
    images: Option<reactive_tui::backend::ImageOutputOptions>,
    timeout: Duration,
) -> Vec<Snapshot> {
    run_steps_on(root, size, steps, Target::SuprTui(images), timeout)
}

fn run_steps_on(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: VecDeque<Step>,
    target: Target,
    timeout: Duration,
) -> Vec<Snapshot> {
    let capture = Capture::default();
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let backend = InputBackend {
        inner: match target {
            Target::SuprTui(Some(images)) => Inner::SuprTui(Box::new(
                SuprTuiBackend::with_writer_and_images(size.0, size.1, capture.clone(), images)
                    .unwrap(),
            )),
            Target::SuprTui(None) => Inner::SuprTui(Box::new(
                SuprTuiBackend::with_writer(size.0, size.1, capture.clone()).unwrap(),
            )),
            Target::Debug => Inner::Debug(Box::new(DebugBackend::new(size.0, size.1))),
        },
        events: steps,
        capture,
        snapshots: snapshots.clone(),
        deadline: Instant::now() + timeout,
        // The first frame has no input wait before it: its work is measured
        // from the App's start, so the frame where a widget first appears
        // and starts animating counts like every other (BAR-005).
        wait_returned: std::sync::Mutex::new(Some(Instant::now())),
        busy: false,
        live: Vec::new(),
    };
    App::builder()
        .backend(backend)
        .root(root)
        .build()
        .unwrap()
        .run()
        .unwrap();
    Arc::try_unwrap(snapshots)
        .ok()
        .unwrap()
        .into_inner()
        .unwrap()
}
#[allow(dead_code)]
pub fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
}
#[allow(dead_code)]
pub fn click(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}
