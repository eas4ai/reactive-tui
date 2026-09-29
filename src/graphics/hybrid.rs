//! The canvas's renderer: the hardware adapter while it works, the software
//! renderer otherwise and after any fault, for the rest of the renderer's
//! life (GFX-002, GFX-007).

use super::compile::compile;
use super::fonts::{Font, FontSource};
use super::glyphs::Glyphs;
use super::gpu::GpuRenderer;
use super::output::CanvasOutput;
use super::scene::{Scene, Transform};
use super::{cpu, pixel_count, GraphicsAdapterInfo, GraphicsError, GraphicsFrame, GraphicsTimings};
use crate::layout::CellGrid;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;
use suprtui::blit::Blitter;

/// The renderer that drew a picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsMode {
    /// A hardware adapter.
    Gpu(GraphicsAdapterInfo),
    /// The software renderer, and why.
    CpuFallback(String),
}

impl GraphicsMode {
    /// The renderer in words, with the adapter's name or the reason for
    /// the software renderer.
    pub fn label(&self) -> String {
        match self {
            Self::Gpu(info) => format!("GPU · {} · {}", info.name, info.backend),
            Self::CpuFallback(reason) => format!("CPU fallback · {reason}"),
        }
    }
}

/// A fault to inject, so that what the canvas does after one can be checked
/// (GFX-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsFault {
    /// No adapter can be used.
    Adapter,
    /// The device is lost before the next picture.
    DeviceLoss,
    /// Reading the picture back fails.
    Readback,
    /// The software renderer fails.
    Software,
}

impl GraphicsFault {
    /// The fault's name, which the reason for the software renderer holds.
    pub fn label(self) -> &'static str {
        match self {
            Self::Adapter => "adapter",
            Self::DeviceLoss => "device-loss",
            Self::Readback => "readback",
            Self::Software => "software",
        }
    }
}

/// How a canvas renders and shows its pictures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphicsOptions {
    /// Draw on the software renderer although the host has a hardware
    /// adapter.
    pub force_cpu: bool,
    /// A fault to inject.
    pub fault: Option<GraphicsFault>,
    /// The font text is drawn in.
    pub font: FontSource,
    /// How the canvas shows its picture, instead of what the host takes
    /// (GFX-005).
    pub output: Option<CanvasOutput>,
}

/// The pixels of a cell that scenes are drawn for when the host has not
/// said what its cells measure.
pub(crate) const CELL_PIXELS: (u16, u16) = (8, 16);

/// What panicked, in words.
fn panic_text(panic: Box<dyn std::any::Any + Send>) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic".into())
}

/// The hardware adapter first, the software renderer after a fault.
pub struct HybridRenderer {
    gpu: Option<GpuRenderer>,
    glyphs: Glyphs,
    mode: GraphicsMode,
    fault: Option<GraphicsFault>,
}

/// Ask the system not to slow the calling thread down to save power.
///
/// Windows moves a thread that waits for the GPU for much of its time to
/// the processor's efficiency cores, where the work around each picture,
/// handing it to the GPU and copying it out, takes four to five times as
/// long: on the test tablet a picture of 1920 by 960 pixels took 29 ms
/// there and 12 ms on the performance cores (GFX-004). Other systems take
/// no such request.
#[cfg(windows)]
fn keep_full_speed() {
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, SetThreadInformation, ThreadPowerThrottling,
        THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        THREAD_POWER_THROTTLING_STATE,
    };
    // Execution speed is under this thread's control and is not throttled.
    let state = THREAD_POWER_THROTTLING_STATE {
        Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: 0,
    };
    // SAFETY: the handle is the calling thread's own, and the pointer and
    // the size are those of `state`, which outlives the call. A system
    // that does not know the request refuses it, and the thread runs as
    // before.
    unsafe {
        SetThreadInformation(
            GetCurrentThread(),
            ThreadPowerThrottling,
            std::ptr::from_ref(&state).cast(),
            std::mem::size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

#[cfg(not(windows))]
fn keep_full_speed() {}

impl HybridRenderer {
    /// A renderer on the hardware adapter when the host has one that
    /// works, else on the software renderer. It waits for the adapter, so
    /// an App makes its renderers on a worker ([`super::GraphicsWorker`]).
    /// The thread that makes the renderer is the one that draws with it:
    /// on Windows it asks the system not to slow it down to save power.
    pub fn new(options: GraphicsOptions) -> Self {
        keep_full_speed();
        let chosen = if options.force_cpu {
            Err("explicit CPU selection".to_owned())
        } else if options.fault == Some(GraphicsFault::Adapter) {
            Err("injected adapter failure".to_owned())
        } else {
            catch_unwind(GpuRenderer::new)
                .map_err(|panic| format!("adapter: {}", panic_text(panic)))
                .and_then(|made| made.map_err(|error| error.to_string()))
        };
        let (gpu, mode) = match chosen {
            Ok(gpu) => {
                let mode = GraphicsMode::Gpu(gpu.info().clone());
                (Some(gpu), mode)
            }
            Err(reason) => (None, GraphicsMode::CpuFallback(reason)),
        };
        Self {
            gpu,
            glyphs: Glyphs::new(Font::load(&options.font)),
            mode,
            fault: options.fault,
        }
    }

    /// The renderer that draws the next picture.
    pub fn mode(&self) -> &GraphicsMode {
        &self.mode
    }

    /// The font text is drawn in, after the choice was made.
    pub fn font(&self) -> &FontSource {
        &self.glyphs.font().source
    }

    /// The picture of `scene` at `width` by `height` pixels, each at most
    /// 4096.
    pub fn render(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<GraphicsFrame, GraphicsError> {
        self.render_under(scene, (width, height), &Transform::identity())
    }

    /// The picture of `scene` as block glyphs for `columns` by `rows` cells
    /// of 8 by 16 pixels, drawn with the blitter image fallback uses
    /// (BLT-002).
    pub fn render_cells(
        &mut self,
        scene: &Scene,
        columns: u16,
        rows: u16,
    ) -> Result<CellGrid, GraphicsError> {
        let blitter = crate::widgets::display::image::image_blitter();
        self.render_blocks(
            scene,
            (columns, rows),
            CELL_PIXELS,
            blitter,
            &Transform::identity(),
        )
        .map(|(grid, _)| grid)
    }

    /// The picture of `scene` under `base`, which is drawn for cells of
    /// `cell` pixels, as `blitter`'s glyphs for `cells`, with the picture
    /// they were made from.
    pub(crate) fn render_blocks(
        &mut self,
        scene: &Scene,
        cells: (u16, u16),
        cell: (u16, u16),
        blitter: Blitter,
        base: &Transform,
    ) -> Result<(CellGrid, GraphicsFrame), GraphicsError> {
        let (across, down) = blitter.cell_pixels();
        let size = (u32::from(cells.0) * across, u32::from(cells.1) * down);
        let scale = base.then(Transform::scale(
            across as f32 / f32::from(cell.0.max(1)),
            down as f32 / f32::from(cell.1.max(1)),
        ));
        let frame = self.render_under(scene, size, &scale)?;
        let blitted = suprtui::blit::blit_image(blitter, frame.image());
        let mut grid = CellGrid::new(cells.0, cells.1);
        let color = |[r, g, b]: [u8; 3]| {
            (
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                1.0,
            )
        };
        let mut glyph = [0u8; 4];
        let stride = blitted.columns.max(1) as usize;
        for (index, block) in blitted.cells.iter().enumerate() {
            grid.set_with_background(
                (index % stride) as u16,
                (index / stride) as u16,
                block.glyph.encode_utf8(&mut glyph),
                block.fg.map(color),
                block.bg.map(color),
            );
        }
        Ok((grid, frame))
    }

    /// The picture of `scene` at `size`, with `base` applied to every scene
    /// coordinate.
    pub(crate) fn render_under(
        &mut self,
        scene: &Scene,
        size: (u32, u32),
        base: &Transform,
    ) -> Result<GraphicsFrame, GraphicsError> {
        pixel_count(size.0, size.1)?;
        let started = Instant::now();
        self.glyphs.trim();
        let glyphs = &mut self.glyphs;
        let draws = catch_unwind(AssertUnwindSafe(|| compile(scene, base, size, glyphs)))
            .map_err(|panic| GraphicsError::Software(panic_text(panic)))?;
        let prepare = started.elapsed();
        if let Some(gpu) = &mut self.gpu {
            let fault = self.fault.take();
            if fault == Some(GraphicsFault::DeviceLoss) {
                gpu.lose_device();
            }
            let fail_readback = fault == Some(GraphicsFault::Readback);
            let glyphs = &mut self.glyphs;
            let drawn = catch_unwind(AssertUnwindSafe(|| {
                gpu.render(&draws, size, glyphs, fail_readback)
            }))
            .unwrap_or_else(|panic| Err(GraphicsError::Readback(panic_text(panic))));
            match drawn {
                Ok(drawn) => {
                    let timings = GraphicsTimings {
                        prepare,
                        render: drawn.render,
                        wait: drawn.wait,
                        readback: drawn.readback,
                    };
                    return GraphicsFrame::drawn(
                        size,
                        drawn.bytes,
                        self.mode.clone(),
                        timings,
                        drawn.draw_calls,
                    );
                }
                Err(error) => {
                    // The software renderer draws from here on (GFX-007).
                    self.mode = GraphicsMode::CpuFallback(error.to_string());
                    self.gpu = None;
                }
            }
        }
        if self.fault == Some(GraphicsFault::Software) {
            return Err(GraphicsError::Software("injected software failure".into()));
        }
        let started = Instant::now();
        let glyphs = &mut self.glyphs;
        let bytes = catch_unwind(AssertUnwindSafe(|| cpu::render(&draws, size, glyphs)))
            .map_err(|panic| GraphicsError::Software(panic_text(panic)))?;
        let timings = GraphicsTimings {
            prepare,
            render: started.elapsed(),
            ..Default::default()
        };
        GraphicsFrame::drawn(size, bytes, self.mode.clone(), timings, 0)
    }
}
