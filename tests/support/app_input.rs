use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::{Backend, PaintedNode, SuprTuiBackend},
    component::Element,
    error::Result,
    event::types::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, Position},
    render::{reconcile::PatchOp, RenderTree},
};
use std::{
    collections::VecDeque,
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
    pub text: String,
    #[allow(dead_code)]
    pub screen: vt100::Screen,
    #[allow(dead_code)]
    pub geometry: Vec<PaintedNode>,
}
struct Step {
    frame: usize,
    event: Option<Event>,
}

struct InputBackend {
    inner: SuprTuiBackend,
    events: VecDeque<Step>,
    capture: Capture,
    snapshots: Arc<Mutex<Vec<Snapshot>>>,
    deadline: Instant,
}
impl Backend for InputBackend {
    fn painted_nodes(&self) -> Option<&[reactive_tui::backend::PaintedNode]> {
        self.inner.painted_nodes()
    }
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
        let (width, height) = self.inner.size();
        let mut parser = vt100::Parser::new(height, width, 0);
        parser.process(&self.capture.0.lock().unwrap());
        self.snapshots.lock().unwrap().push(Snapshot {
            output: self.capture.0.lock().unwrap().clone(),
            text: parser.screen().contents(),
            screen: parser.screen().clone(),
            geometry: self.inner.painted_nodes().unwrap().to_vec(),
        });
        Ok(())
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        panic!("wake-aware input required")
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        assert!(
            Instant::now() < self.deadline,
            "App did not paint the expected frame before the input deadline"
        );
        if self
            .events
            .front()
            .is_some_and(|step| step.frame > self.snapshots.lock().unwrap().len())
        {
            wake.wait(Some(Duration::from_millis(1)));
            return Ok(None);
        }
        let event = self.events.pop_front().and_then(|step| step.event);
        if event.is_none() {
            wake.request_stop();
        }
        Ok(event)
    }
}

pub fn run(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    steps: Vec<(usize, Option<Event>)>,
) -> Vec<Snapshot> {
    let capture = Capture::default();
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let backend = InputBackend {
        inner: SuprTuiBackend::with_writer(size.0, size.1, capture.clone()).unwrap(),
        events: steps
            .into_iter()
            .map(|(frame, event)| Step { frame, event })
            .collect(),
        capture,
        snapshots: snapshots.clone(),
        deadline: Instant::now() + Duration::from_secs(3),
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
pub fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
}
pub fn click(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}
