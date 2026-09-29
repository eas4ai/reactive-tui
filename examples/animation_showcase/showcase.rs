#[path = "animations.rs"]
pub mod animations;

use animations::{donut_frame, fire_frame, plasma_frame, ripple_frame, warp_frame, Animation};
#[cfg(feature = "wgpu-graphics")]
#[path = "scene.rs"]
pub mod scene;
#[cfg(feature = "wgpu-graphics")]
use reactive_tui::graphics::{Canvas, CanvasProps, GraphicsOptions, GraphicsWorker};
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    builder::div,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind},
    },
};
#[cfg(feature = "wgpu-graphics")]
use std::sync::Arc;
use std::time::Instant;

/// What the showcase's command line asks of the canvas.
#[cfg(feature = "wgpu-graphics")]
pub fn graphics_from(args: impl IntoIterator<Item = String>) -> Result<GraphicsOptions, String> {
    let mut options = GraphicsOptions::default();
    for arg in args {
        match arg.as_str() {
            "--cpu" => options.force_cpu = true,
            _ => return Err(format!("unknown showcase option: {arg}")),
        }
    }
    Ok(options)
}

/// The Shader page's canvas: how it draws, the worker that draws it and
/// when the torus began to turn.
#[cfg(feature = "wgpu-graphics")]
struct CanvasStage {
    options: GraphicsOptions,
    worker: Option<Arc<GraphicsWorker>>,
    started: Instant,
}

/// Tabbed animation pages. The Shader page exists only with wgpu-graphics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowcasePage {
    Donut,
    Plasma,
    Warp,
    Fire,
    Ripples,
    #[cfg(feature = "wgpu-graphics")]
    Shader,
}

impl ShowcasePage {
    #[cfg(not(feature = "wgpu-graphics"))]
    pub const ALL: [Self; 5] = [
        Self::Donut,
        Self::Plasma,
        Self::Warp,
        Self::Fire,
        Self::Ripples,
    ];

    #[cfg(feature = "wgpu-graphics")]
    pub const ALL: [Self; 6] = [
        Self::Donut,
        Self::Plasma,
        Self::Warp,
        Self::Fire,
        Self::Ripples,
        Self::Shader,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Donut => "Donut · shaded torus",
            Self::Plasma => "Plasma · interference",
            Self::Warp => "Warp · starfield",
            Self::Fire => "Fire · rising flames",
            Self::Ripples => "Ripples · wave tank",
            #[cfg(feature = "wgpu-graphics")]
            Self::Shader => "Shader · canvas torus",
        }
    }

    pub const fn color(self) -> &'static str {
        match self {
            Self::Donut => "text-amber-300",
            Self::Plasma => "text-cyan-300",
            Self::Warp => "text-white",
            Self::Fire => "text-orange-400",
            Self::Ripples => "text-teal-300",
            #[cfg(feature = "wgpu-graphics")]
            Self::Shader => "text-fuchsia-300",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Donut => 0,
            Self::Plasma => 1,
            Self::Warp => 2,
            Self::Fire => 3,
            Self::Ripples => 4,
            #[cfg(feature = "wgpu-graphics")]
            Self::Shader => 5,
        }
    }

    fn offset(self, delta: isize) -> Self {
        let count = Self::ALL.len() as isize;
        let index = (self.index() as isize + delta).rem_euclid(count) as usize;
        Self::ALL[index]
    }

    fn render_fn(self) -> fn(std::time::Duration, usize, usize) -> String {
        match self {
            Self::Donut => donut_frame,
            Self::Plasma => plasma_frame,
            Self::Warp => warp_frame,
            Self::Fire => fire_frame,
            Self::Ripples => ripple_frame,
            #[cfg(feature = "wgpu-graphics")]
            Self::Shader => donut_frame,
        }
    }
}

pub struct Showcase {
    page: ShowcasePage,
    width: u16,
    height: u16,
    motion: Animation,
    exit_requested: bool,
    /// Set when the showcase draws the Shader page on the canvas.
    #[cfg(feature = "wgpu-graphics")]
    canvas: Option<CanvasStage>,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            page: ShowcasePage::Donut,
            width: 100,
            height: 30,
            motion: Animation::new(Instant::now(), donut_frame),
            exit_requested: false,
            #[cfg(feature = "wgpu-graphics")]
            canvas: None,
        }
    }
}

impl Showcase {
    #[cfg(feature = "wgpu-graphics")]
    pub fn with_graphics(options: GraphicsOptions) -> Self {
        // Called before SuprTuiBackend::new: the worker makes its renderer
        // now, so what a graphics driver prints while it starts does not
        // land on the App's screen.
        let worker = GraphicsWorker::spawn(options.clone()).ok().map(Arc::new);
        if let Some(worker) = &worker {
            worker.wait_ready(std::time::Duration::from_secs(10));
        }
        Self {
            canvas: Some(CanvasStage {
                options,
                worker,
                started: Instant::now(),
            }),
            ..Self::default()
        }
    }

    /// Terminal cells available to the canvas after header, footer, and titles.
    fn stage_viewport(&self) -> (usize, usize) {
        (
            usize::from(self.width),
            usize::from(self.height.saturating_sub(4)),
        )
    }

    #[cfg(test)]
    pub fn page(&self) -> ShowcasePage {
        self.page
    }

    fn set_page(&mut self, page: ShowcasePage) {
        self.page = page;
        self.motion.retarget(Instant::now(), page.render_fn());
        let (columns, rows) = self.stage_viewport();
        self.motion.set_viewport(columns, rows);
    }

    fn page_for_number(ch: char) -> Option<ShowcasePage> {
        ch.to_digit(10)
            .and_then(|number| number.checked_sub(1))
            .and_then(|index| ShowcasePage::ALL.get(index as usize).copied())
    }

    fn braille_stage(&self) -> Element {
        let (columns, rows) = self.stage_viewport();
        div()
            .class("w-full flex-1 min-h-0 flex-col bg-black")
            .child(
                div()
                    .class("h-1 shrink-0 text-white font-bold")
                    .text(self.page.title())
                    .build(),
            )
            .child(
                div()
                    .class("h-1 shrink-0 text-gray-500")
                    .text("Braille subpixels · 50 ms · viewport-fitted")
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0 whitespace-pre")
                    .class(self.page.color())
                    .text(&self.motion.centered(columns, rows))
                    .build(),
            )
            .build()
    }

    /// The Shader page: the torus as a scene the canvas fits to the stage,
    /// under the name of the renderer that draws it.
    #[cfg(feature = "wgpu-graphics")]
    fn shader_stage(&self) -> Element {
        let title = div()
            .class("h-1 shrink-0 text-white font-bold")
            .text("Shaded torus · canvas scene")
            .build();
        let Some(stage) = &self.canvas else {
            return div()
                .class("w-full flex-1 min-h-0 flex-col bg-black")
                .child(title)
                .child(
                    div()
                        .class("whitespace-normal text-gray-400")
                        .text("The showcase was started without the canvas.")
                        .build(),
                )
                .build();
        };
        // The renderer is named once it has drawn, so the line and the
        // first picture appear together.
        let renderer = stage
            .worker
            .as_ref()
            .filter(|worker| worker.stats().rendered > 0)
            .and_then(|worker| worker.mode())
            .map_or_else(|| "Starting graphics…".to_owned(), |mode| mode.label());
        let mut props = CanvasProps::new(Arc::new(scene::torus_scene(stage.started.elapsed())))
            .options(stage.options.clone())
            .view(scene::VIEW.0, scene::VIEW.1)
            .label("Shaded turning torus");
        if let Some(worker) = &stage.worker {
            props = props.worker(worker.clone());
        }
        div()
            .class("flex-col flex-1 min-w-0 min-h-0 h-full bg-gray-900")
            .child(title)
            .child(
                div()
                    .class("h-1 shrink-0 text-fuchsia-300")
                    .text(&renderer)
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0")
                    .child(Element::typed::<Canvas>(props))
                    .build(),
            )
            .build()
    }

    fn footer_text(&self) -> &'static str {
        #[cfg(feature = "wgpu-graphics")]
        if self.width < 80 {
            return "Tab next · 1–6 · Ctrl+Q quit";
        }
        #[cfg(not(feature = "wgpu-graphics"))]
        if self.width < 80 {
            return "Tab next · 1–5 · Ctrl+Q quit";
        }
        #[cfg(feature = "wgpu-graphics")]
        return "Tab next · Shift+Tab prev · 1–6 jump · Ctrl+Q quit";
        #[cfg(not(feature = "wgpu-graphics"))]
        return "Tab next · Shift+Tab prev · 1–5 jump · Ctrl+Q quit";
    }
}

impl RootComponent for Showcase {
    fn render(&self) -> Element {
        #[cfg(feature = "wgpu-graphics")]
        let stage = if self.page == ShowcasePage::Shader {
            self.shader_stage()
        } else {
            self.braille_stage()
        };
        #[cfg(not(feature = "wgpu-graphics"))]
        let stage = self.braille_stage();
        div()
            .class("w-screen h-screen flex-col bg-black text-gray-200")
            .child(
                div()
                    .class(
                        "w-full shrink-0 h-1 flex-row px-0.25 bg-gray-950 border-b border-gray-700",
                    )
                    .child(
                        div()
                            .class("flex-1 text-cyan-300 font-bold")
                            .text(&format!("◈ Animation Showcase · {}", self.page.title()))
                            .build(),
                    )
                    .child(
                        div()
                            .class("text-gray-500")
                            .text(&format!(
                                "{}/{} {}×{}",
                                self.page.index() + 1,
                                ShowcasePage::ALL.len(),
                                self.width,
                                self.height
                            ))
                            .build(),
                    )
                    .build(),
            )
            .child(stage)
            .child(
                div()
                    .class("w-full shrink-0 h-1 px-0.25 bg-gray-950 text-gray-500")
                    .text(self.footer_text())
                    .build(),
            )
            .build()
    }

    fn resize(&mut self, width: u16, height: u16) -> reactive_tui::Result<()> {
        self.width = width;
        self.height = height;
        let (columns, rows) = self.stage_viewport();
        self.motion.set_viewport(columns, rows);
        Ok(())
    }

    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::Result<EventResult> {
        let Event::Key(key) = event else {
            return Ok(EventResult::Ignored);
        };
        if key.kind == KeyEventKind::Release {
            return Ok(EventResult::Ignored);
        }
        if key.code == KeyCode::Escape
            || (key.modifiers.ctrl && matches!(key.code, KeyCode::Char('q' | 'c')))
        {
            self.exit_requested = true;
            return Ok(EventResult::Handled);
        }
        let next = match key.code {
            KeyCode::Tab if key.modifiers.shift => Some(self.page.offset(-1)),
            KeyCode::Tab | KeyCode::Right | KeyCode::Down => Some(self.page.offset(1)),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Up => Some(self.page.offset(-1)),
            KeyCode::Char(ch) => Self::page_for_number(ch),
            _ => None,
        };
        if let Some(page) = next {
            self.set_page(page);
            Ok(EventResult::Handled)
        } else {
            Ok(EventResult::Ignored)
        }
    }

    fn update(&mut self) -> reactive_tui::Result<RootUpdate> {
        // The torus turns with time: each frame of the Shader page is a
        // new scene, which the canvas's worker draws.
        #[cfg(feature = "wgpu-graphics")]
        if !self.exit_requested && self.page == ShowcasePage::Shader && self.canvas.is_some() {
            return Ok(RootUpdate::Redraw);
        }
        Ok(if self.exit_requested {
            RootUpdate::Exit
        } else if self.motion.advance(Instant::now()) {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
}
