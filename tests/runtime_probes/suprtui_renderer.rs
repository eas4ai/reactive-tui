//! Internal renderer lifecycle acceptance probe.

use reactive_tui::app::{App, RootComponent};
use reactive_tui::backend::{Backend, SuprTuiBackend};
use reactive_tui::component::{Element, LayoutType};
use reactive_tui::error::Result;
use reactive_tui::event::router::EventResult;
use reactive_tui::event::types::{Event, KeyCode, KeyEventKind};
use reactive_tui::render::{reconcile::PatchOp, tree::RenderTree};
use std::sync::atomic::{AtomicI64, Ordering};

struct Counter {
    value: AtomicI64,
    probe_panic: bool,
}

impl RootComponent for Counter {
    fn render(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("flex flex-col w-full h-full p-0.5 bg-blue-900")
            .with_children(vec![
                Element::text("Reactive-TUI / SuprTUI")
                    .with_class("w-full h-1 text-cyan-300 font-bold"),
                Element::text(format!("Count: {}", self.value.load(Ordering::Relaxed)))
                    .with_class("w-full h-1 text-white"),
                Element::text("Space/+ adds; - subtracts").with_class("w-full h-1"),
                Element::text("Escape or Ctrl+C quits").with_class("w-full h-1"),
                Element::text("Unicode: e\u{301} 界 👩‍💻").with_class("w-full h-1"),
            ])
    }

    fn handle_event(&self, event: &Event) -> EventResult {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Release {
                return EventResult::Ignored;
            }
            match key.code {
                KeyCode::Char(' ' | '+') => {
                    assert!(!self.probe_panic, "controlled application panic");
                    self.value.fetch_add(1, Ordering::Relaxed);
                    return EventResult::Handled;
                }
                KeyCode::Char('-') => {
                    self.value.fetch_sub(1, Ordering::Relaxed);
                    return EventResult::Handled;
                }
                _ => {}
            }
        }
        EventResult::Ignored
    }
}

struct ExampleBackend {
    inner: SuprTuiBackend,
    probe_error: bool,
}

impl Backend for ExampleBackend {
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn layout_frame(
        &mut self,
        element: std::sync::Arc<Element>,
    ) -> Result<Option<reactive_tui::backend::FrameLayout>> {
        self.inner.layout_frame(element)
    }
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        self.inner.apply_patches(patches, tree)
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn present(&mut self) -> Result<()> {
        self.inner.present()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<Event>> {
        let event = self.inner.poll_event(timeout_ms)?;
        if self.probe_error
            && matches!(&event, Some(Event::Key(key)) if key.code == KeyCode::Char(' '))
        {
            return Err(std::io::Error::other("controlled application input error").into());
        }
        Ok(event)
    }
}

fn main() -> Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_default();
    App::builder()
        .backend(ExampleBackend {
            inner: SuprTuiBackend::new()?,
            probe_error: mode == "--probe-error",
        })
        .root(Counter {
            value: AtomicI64::new(0),
            probe_panic: mode == "--probe-panic",
        })
        .build()?
        .run()
}
