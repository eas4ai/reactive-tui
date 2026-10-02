//! The graphics canvas (docs/spec/canvas.md): a
//! [`Canvas`](crate::graphics::Canvas) widget draws a
//! [`Scene`](crate::graphics::Scene) of paths, paint, images, text and cell
//! grids, on a hardware
//! wgpu adapter when the host has one and on the software renderer
//! otherwise, and shows the picture as Kitty graphics, Sixel or block
//! glyphs. Rendering runs on a named worker thread; the App's thread never
//! waits for it.

mod compile;
mod cpu;
pub mod fonts;
mod geometry;
mod glyphs;
mod gpu;
mod hybrid;
mod output;
mod paint;
mod raster;
mod scene;
mod widget;
mod worker;

pub use hybrid::{GraphicsFault, GraphicsMode, GraphicsOptions, HybridRenderer};
pub use output::CanvasOutput;
pub(crate) use output::{CanvasPaint, HostReport};
pub use scene::{
    CanvasImage, Color, GradientStop, LineCap, LineJoin, Paint, Path, PathBuilder, Scene, Stroke,
    Transform,
};
pub use widget::{Canvas, CanvasProps};
pub use worker::{GraphicsWorker, WorkerStats};

use std::time::Duration;

/// The widest picture the canvas draws, in pixels.
pub const MAX_WIDTH: u32 = 4096;
/// The tallest picture the canvas draws, in pixels.
pub const MAX_HEIGHT: u32 = 4096;

/// Why the canvas could not draw.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphicsError {
    /// A size outside what the canvas draws, or pixels that do not fill it.
    #[error("invalid graphics dimensions or pixel count: {0}")]
    Dimensions(String),
    /// No hardware adapter gave a device.
    #[error("GPU initialization failed: {0}")]
    Initialization(String),
    /// The hardware adapter failed while drawing or reading the picture
    /// back.
    #[error("GPU readback failed: {0}")]
    Readback(String),
    /// The software renderer failed.
    #[error("the software renderer failed: {0}")]
    Software(String),
    /// The glyph atlas, at its largest, cannot hold every glyph one frame
    /// draws: that frame is drawn in software, and the adapter is kept
    /// (GFX-007).
    #[error("the glyph atlas cannot hold the frame's glyphs: {0}")]
    AtlasFull(String),
    /// The worker could not start or has stopped.
    #[error("the canvas worker is not running: {0}")]
    Worker(String),
}

/// The pixels of a `width` by `height` picture, or why it cannot be drawn.
fn pixel_count(width: u32, height: u32) -> Result<usize, GraphicsError> {
    if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
        return Err(GraphicsError::Dimensions(format!(
            "{width}x{height}; at least 1x1 and at most {MAX_WIDTH}x{MAX_HEIGHT}"
        )));
    }
    Ok(width as usize * height as usize)
}

/// How long the renderer took over one picture.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GraphicsTimings {
    /// Preparing the scene's draws, on either renderer.
    pub prepare: Duration,
    /// Drawing: on the software renderer all of it; on the hardware
    /// adapter, building the frame and handing it to the GPU.
    pub render: Duration,
    /// Waiting for the GPU to finish the picture; none on the software
    /// renderer.
    pub wait: Duration,
    /// Copying the picture out of the GPU's memory; none on the software
    /// renderer.
    pub readback: Duration,
}

/// A finished picture: RGBA pixels with straight alpha, and who drew them.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsFrame {
    /// Shared with the painter, which sends the pixels to the terminal.
    image: std::sync::Arc<image::RgbaImage>,
    mode: GraphicsMode,
    timings: GraphicsTimings,
    draw_calls: u32,
    thread: String,
}

impl GraphicsFrame {
    /// A picture from pixels drawn elsewhere, row by row.
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<[u8; 4]>) -> Result<Self, GraphicsError> {
        if pixel_count(width, height)? != pixels.len() {
            return Err(GraphicsError::Dimensions(format!(
                "{} pixels for {width}x{height}",
                pixels.len()
            )));
        }
        let mut frame = Self::drawn(
            (width, height),
            pixels.into_flattened(),
            GraphicsMode::CpuFallback("pixels drawn elsewhere".into()),
            GraphicsTimings::default(),
            0,
        )?;
        frame.thread.clear();
        Ok(frame)
    }

    /// A picture a renderer drew on this thread, from its bytes: red,
    /// green, blue and alpha of each pixel in turn.
    pub(crate) fn drawn(
        (width, height): (u32, u32),
        bytes: Vec<u8>,
        mode: GraphicsMode,
        timings: GraphicsTimings,
        draw_calls: u32,
    ) -> Result<Self, GraphicsError> {
        let image = image::RgbaImage::from_raw(width, height, bytes).ok_or_else(|| {
            GraphicsError::Dimensions(format!("too few pixels for {width}x{height}"))
        })?;
        Ok(Self {
            image: std::sync::Arc::new(image),
            mode,
            timings,
            draw_calls,
            thread: std::thread::current().name().unwrap_or_default().to_owned(),
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.image.height()
    }

    /// The pixels, row by row.
    pub fn pixels(&self) -> &[[u8; 4]] {
        self.image.as_raw().as_chunks().0
    }

    /// The pixels as bytes: red, green, blue and alpha of each in turn.
    pub fn bytes(&self) -> &[u8] {
        self.image.as_raw()
    }

    /// The picture as the image the painter sends.
    pub(crate) fn image(&self) -> &std::sync::Arc<image::RgbaImage> {
        &self.image
    }

    /// The renderer that drew the picture (GFX-002).
    pub fn mode(&self) -> &GraphicsMode {
        &self.mode
    }

    /// How long the picture took.
    pub fn timings(&self) -> GraphicsTimings {
        self.timings
    }

    /// The draw calls the hardware adapter made for the picture; none on
    /// the software renderer.
    pub fn draw_calls(&self) -> u32 {
        self.draw_calls
    }

    /// The name of the thread that drew the picture.
    pub fn thread(&self) -> &str {
        &self.thread
    }
}

/// The adapter that draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsAdapterInfo {
    /// The name its driver reports.
    pub name: String,
    /// Vulkan, Metal or Dx12.
    pub backend: String,
    hardware: bool,
}

impl GraphicsAdapterInfo {
    /// Whether it is a discrete or integrated GPU, not a software adapter.
    pub fn is_hardware(&self) -> bool {
        self.hardware
    }
}
