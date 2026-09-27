#[path = "animations.rs"]
pub mod animations;

use animations::{donut_frame, fire_frame, plasma_frame, ripple_frame, warp_frame, Animation};
#[cfg(feature = "wgpu-graphics")]
use reactive_tui::graphics::{GraphicsCanvas, GraphicsEffect, GraphicsOptions, HybridCubeRenderer};
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    builder::div,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind},
    },
};
use std::time::Instant;

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
            Self::Shader => "Shader · GPU torus",
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
    #[cfg(feature = "wgpu-graphics")]
    graphics: Option<GraphicsCanvas>,
    #[cfg(feature = "wgpu-graphics")]
    graphics_options: GraphicsOptions,
    #[cfg(feature = "wgpu-graphics")]
    prepared_graphics: Option<HybridCubeRenderer>,
    #[cfg(feature = "wgpu-graphics")]
    graphics_started: Instant,
    #[cfg(feature = "wgpu-graphics")]
    graphics_error: Option<String>,
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
            graphics: None,
            #[cfg(feature = "wgpu-graphics")]
            graphics_options: GraphicsOptions::default(),
            #[cfg(feature = "wgpu-graphics")]
            prepared_graphics: None,
            #[cfg(feature = "wgpu-graphics")]
            graphics_started: Instant::now(),
            #[cfg(feature = "wgpu-graphics")]
            graphics_error: None,
        }
    }
}

impl Showcase {
    #[cfg(feature = "wgpu-graphics")]
    pub fn with_graphics(mut options: GraphicsOptions) -> Self {
        options.effect = GraphicsEffect::Torus;
        Self {
            graphics_options: options,
            prepared_graphics: Some(HybridCubeRenderer::new(options)),
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

    #[cfg(feature = "wgpu-graphics")]
    fn graphics_viewport(&self) -> (u32, u32) {
        let (columns, rows) = self.stage_viewport();
        (columns as u32, rows as u32)
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

    #[cfg(feature = "wgpu-graphics")]
    fn shader_stage(&self) -> Element {
        if let Some(graphics) = &self.graphics {
            return div()
                .class("flex-col flex-1 min-w-0 min-h-0 h-full bg-gray-900")
                .child(
                    div()
                        .class("h-1 shrink-0 text-white font-bold")
                        .text("Raymarched torus · GPU shader")
                        .build(),
                )
                .child(
                    div()
                        .class("h-1 shrink-0 text-fuchsia-300")
                        .text(
                            &self
                                .graphics_error
                                .clone()
                                .unwrap_or_else(|| graphics.mode_label()),
                        )
                        .build(),
                )
                .child(
                    graphics
                        .element()
                        .unwrap_or_else(|error| Element::text(error.to_string())),
                )
                .build();
        }
        div()
            .class("w-full flex-1 min-h-0 flex-col bg-black")
            .child(
                div()
                    .class("h-1 shrink-0 text-white font-bold")
                    .text("Raymarched torus · GPU shader")
                    .build(),
            )
            .child(
                div()
                    .class("whitespace-normal text-gray-400")
                    .text("Starting graphics…")
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
    #[cfg(feature = "wgpu-graphics")]
    fn attach_waker(&mut self, wake: reactive_tui::app::AppWaker) {
        self.graphics = Some(match self.prepared_graphics.take() {
            Some(renderer) => GraphicsCanvas::with_renderer(wake, renderer),
            None => GraphicsCanvas::new(wake, self.graphics_options),
        });
    }

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
        #[cfg(feature = "wgpu-graphics")]
        {
            if self.exit_requested {
                if let Some(graphics) = &mut self.graphics {
                    graphics.shutdown().map_err(|error| {
                        reactive_tui::ReactiveError::invalid_state(error.to_string())
                    })?;
                }
                return Ok(RootUpdate::Exit);
            }
            if self.page == ShowcasePage::Shader {
                let (columns, rows) = self.graphics_viewport();
                if let Some(graphics) = &mut self.graphics {
                    return Ok(
                        match graphics.advance(self.graphics_started.elapsed(), columns, rows) {
                            Ok(true) => {
                                self.graphics_error = None;
                                RootUpdate::Redraw
                            }
                            Ok(false) => RootUpdate::Unchanged,
                            Err(error) => {
                                let message = error.to_string();
                                if self.graphics_error.as_ref() == Some(&message) {
                                    RootUpdate::Unchanged
                                } else {
                                    self.graphics_error = Some(message);
                                    RootUpdate::Redraw
                                }
                            }
                        },
                    );
                }
            }
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
