use reactive_tui::{
    app::{App, AppWaker, RootComponent},
    backend::{Backend, SuprTuiBackend},
    builder::core::div,
    component::Element,
    error::Result,
    event::types::{Event, KeyCode, KeyEvent},
    render::{reconcile::PatchOp, RenderTree},
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

struct InputBackend {
    inner: SuprTuiBackend,
    events: VecDeque<Event>,
}
impl Backend for InputBackend {
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
        self.inner.present()
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        Ok(self.events.pop_front())
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        if self.events.is_empty() {
            wake.request_stop();
        }
        Ok(self.events.pop_front())
    }
}
struct ClickRoot(Arc<AtomicUsize>);
impl RootComponent for ClickRoot {
    fn render(&self) -> Element {
        let calls = self.0.clone();
        div()
            .class("w-8 h-1")
            .on_click(move || {
                calls.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .auto_focus()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn builder_callback_receives_keyboard_activation_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let backend = InputBackend {
        inner: SuprTuiBackend::with_writer(16, 4, std::io::sink()).unwrap(),
        events: VecDeque::from([Event::Key(KeyEvent::new(KeyCode::Enter))]),
    };
    App::builder()
        .backend(backend)
        .root(ClickRoot(calls.clone()))
        .build()
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the focused builder callback must run once"
    );
}
