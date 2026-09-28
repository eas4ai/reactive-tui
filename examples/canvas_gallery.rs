//! The canvas's reference scenes, one at a time: strokes, curves, shapes,
//! gradients, an image, text, a cell grid, transforms and clips, and the
//! cube. They are the scenes the canvas's checks draw
//! (tests/canvas_support), fitted to the terminal.
//!
//! ```sh
//! cargo run --locked --features wgpu-graphics --example canvas_gallery
//! cargo run --locked --features wgpu-graphics --example canvas_gallery -- --cpu
//! ```
#[path = "../tests/canvas_support/mod.rs"]
mod canvas_support;

use reactive_tui::{
    app::{App, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    builder::div,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind, KeyModifiers},
    },
    graphics::{Canvas, CanvasProps, GraphicsOptions, GraphicsWorker, Scene},
};
use std::sync::Arc;
use std::time::Duration;

struct Gallery {
    scenes: Vec<(&'static str, Arc<Scene>, (u32, u32))>,
    shown: usize,
    options: GraphicsOptions,
    worker: Option<Arc<GraphicsWorker>>,
    exit_requested: bool,
}

impl Gallery {
    fn new(options: GraphicsOptions) -> Self {
        // Started before the terminal is set up, so what a graphics driver
        // prints while it starts does not land on the App's screen.
        let worker = GraphicsWorker::spawn(options.clone()).ok().map(Arc::new);
        if let Some(worker) = &worker {
            worker.wait_ready(Duration::from_secs(10));
        }
        Self {
            scenes: canvas_support::references()
                .into_iter()
                .map(|reference| (reference.name, Arc::new(reference.scene), reference.size))
                .collect(),
            shown: 0,
            options,
            worker,
            exit_requested: false,
        }
    }

    fn step(&mut self, by: isize) {
        let count = self.scenes.len() as isize;
        self.shown = (self.shown as isize + by).rem_euclid(count) as usize;
    }
}

impl RootComponent for Gallery {
    fn render(&self) -> Element {
        let (name, scene, size) = &self.scenes[self.shown];
        // The renderer is named once it has drawn, so the line and the
        // first picture appear together.
        let renderer = self
            .worker
            .as_ref()
            .filter(|worker| worker.stats().rendered > 0)
            .and_then(|worker| worker.mode())
            .map_or_else(|| "Starting graphics…".to_owned(), |mode| mode.label());
        let mut props = CanvasProps::new(scene.clone())
            .options(self.options.clone())
            .view(size.0 as f32, size.1 as f32)
            .label(format!("Reference scene {name}"));
        if let Some(worker) = &self.worker {
            props = props.worker(worker.clone());
        }
        div()
            .class("w-screen h-screen flex-col bg-black text-gray-200")
            .child(
                div()
                    .class("w-full shrink-0 h-1 flex-row px-0.25 bg-gray-950")
                    .child(
                        div()
                            .class("flex-1 text-cyan-300 font-bold")
                            .text(&format!("◈ Canvas gallery · {name}"))
                            .build(),
                    )
                    .child(
                        div()
                            .class("text-gray-500")
                            .text(&format!("{}/{}", self.shown + 1, self.scenes.len()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                div()
                    .class("w-full shrink-0 h-1 px-0.25 text-cyan-300")
                    .text(&renderer)
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0")
                    .child(Element::typed::<Canvas>(props))
                    .build(),
            )
            .child(
                div()
                    .class("w-full shrink-0 h-1 px-0.25 bg-gray-950 text-gray-500")
                    .text("←/→ scene · 1–9 jump · Ctrl+Q quit")
                    .build(),
            )
            .build()
    }

    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::Result<EventResult> {
        let Event::Key(key) = event else {
            return Ok(EventResult::Ignored);
        };
        if key.kind == KeyEventKind::Release {
            return Ok(EventResult::Ignored);
        }
        match key.code {
            KeyCode::Escape => self.exit_requested = true,
            KeyCode::Char('q' | 'c') if key.modifiers.ctrl => self.exit_requested = true,
            KeyCode::Right | KeyCode::Down | KeyCode::Tab => self.step(1),
            KeyCode::Left | KeyCode::Up | KeyCode::BackTab => self.step(-1),
            KeyCode::Char(digit @ '1'..='9') => {
                let index = digit as usize - '1' as usize;
                if index < self.scenes.len() {
                    self.shown = index;
                }
            }
            _ => return Ok(EventResult::Ignored),
        }
        Ok(EventResult::Handled)
    }

    fn update(&mut self) -> reactive_tui::Result<RootUpdate> {
        Ok(if self.exit_requested {
            RootUpdate::Exit
        } else {
            RootUpdate::Unchanged
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = GraphicsOptions::default();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--cpu" => options.force_cpu = true,
            _ => return Err(format!("unknown gallery option: {arg}").into()),
        }
    }
    let gallery = Gallery::new(options);
    let backend = SuprTuiBackend::new()?;
    App::builder()
        .backend(backend)
        .root(gallery)
        .quit_key(
            KeyCode::Char('q'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::empty()
            },
        )
        .build()?
        .run()?;
    Ok(())
}
