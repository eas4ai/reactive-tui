use super::{
    cpu, FrameRequest, GpuCubeRenderer, GraphicsAdapterInfo, GraphicsCancellation, GraphicsError,
    GraphicsFrame,
};
use std::time::Duration;

/// The renderer that actually produced a frame.
#[derive(Debug, Clone)]
pub enum GraphicsMode {
    /// A wgpu adapter rendered the pixels; its identity distinguishes hardware/software.
    Gpu(GraphicsAdapterInfo),
    /// CPU pixels, with the reason GPU rendering was not selected.
    CpuFallback(String),
}
impl GraphicsMode {
    /// Visible mode and adapter/fallback reason for the catalog and reports.
    pub fn label(&self) -> String {
        match self {
            Self::Gpu(info) => format!(
                "{} · {} · {}",
                if info.is_hardware {
                    "GPU"
                } else {
                    "Software wgpu"
                },
                info.name,
                info.backend,
            ),
            Self::CpuFallback(reason) => format!("CPU fallback · {reason}"),
        }
    }
}

/// Explicit failure injection for reproducible lifecycle verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsFault {
    /// Adapter initialization is unavailable.
    Adapter,
    /// Destroy the initialized device before its next frame.
    DeviceLoss,
    /// Fail texture readback after submission.
    Readback,
}
impl GraphicsFault {
    /// Stable command-line/report name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Adapter => "adapter",
            Self::DeviceLoss => "device-loss",
            Self::Readback => "readback",
        }
    }
}

/// Rendered effect. The shaded cube is the default; the torus powers the
/// animation showcase's shader page on the same worker pipeline.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsEffect {
    /// Raymarched shaded cube.
    #[default]
    Cube,
    /// Raymarched torus with rim lighting.
    Torus,
}

/// Selection options; faults are opt-in and never read from global environment.
#[derive(Debug, Default, Clone, Copy)]
pub struct GraphicsOptions {
    /// Select CPU rendering without initializing a GPU.
    pub force_cpu: bool,
    /// Inject one named failure through the normal fallback path.
    pub fault: Option<GraphicsFault>,
    /// Rendered effect; shared by the GPU shader and the CPU fallback.
    pub effect: GraphicsEffect,
}

/// Hardware rendering with a persistent, truthfully labeled CPU fallback.
pub struct HybridCubeRenderer {
    gpu: Option<GpuCubeRenderer>,
    mode: GraphicsMode,
    fault: Option<GraphicsFault>,
    effect: GraphicsEffect,
}
impl HybridCubeRenderer {
    /// Initialize outside the App loop; failures retain a usable CPU renderer.
    pub fn new(options: GraphicsOptions) -> Self {
        Self::new_cancellable(options, &GraphicsCancellation::default())
    }

    pub(crate) fn new_cancellable(
        options: GraphicsOptions,
        cancellation: &GraphicsCancellation,
    ) -> Self {
        let selected = if options.force_cpu {
            Err(GraphicsError::Initialization(
                "explicit CPU selection".into(),
            ))
        } else if options.fault == Some(GraphicsFault::Adapter) {
            Err(GraphicsError::Initialization(
                "injected adapter failure".into(),
            ))
        } else {
            GpuCubeRenderer::new_cancellable(cancellation)
        };
        match selected {
            Ok(gpu) if gpu.info.is_hardware => Self {
                mode: GraphicsMode::Gpu(gpu.info.clone()),
                gpu: Some(gpu),
                fault: options.fault,
                effect: options.effect,
            },
            Ok(gpu) => Self {
                gpu: None,
                mode: GraphicsMode::CpuFallback(format!("software adapter {}", gpu.info.name)),
                fault: None,
                effect: options.effect,
            },
            Err(error) => Self {
                gpu: None,
                mode: GraphicsMode::CpuFallback(error.to_string()),
                fault: None,
                effect: options.effect,
            },
        }
    }

    /// Render the checked viewport; GPU failures select CPU without quitting.
    pub fn render_terminal(
        &mut self,
        columns: u32,
        rows: u32,
        elapsed: Duration,
    ) -> Result<GraphicsFrame, GraphicsError> {
        self.render(
            FrameRequest::new(columns, rows, elapsed)?,
            &GraphicsCancellation::default(),
        )
    }

    pub(crate) fn render(
        &mut self,
        request: FrameRequest,
        cancellation: &GraphicsCancellation,
    ) -> Result<GraphicsFrame, GraphicsError> {
        if cancellation.is_cancelled() {
            return Err(GraphicsError::Cancelled);
        }
        if let Some(gpu) = &self.gpu {
            let fault = self.fault.take();
            if fault == Some(GraphicsFault::DeviceLoss) {
                gpu.device.destroy();
                *gpu.failure.lock().unwrap() = Some("injected device-loss failure".into());
            }
            match gpu.render_cancellable(
                request.columns(),
                request.rows() * 2,
                request.elapsed(),
                cancellation,
                fault == Some(GraphicsFault::Readback),
                self.effect,
            ) {
                Ok(frame) => return Ok(frame),
                Err(GraphicsError::Cancelled) => return Err(GraphicsError::Cancelled),
                Err(error) => {
                    self.mode = GraphicsMode::CpuFallback(error.to_string());
                    self.gpu = None;
                }
            }
        }
        let mut frame = cpu::render(request, cancellation, self.effect)?;
        frame.mode = self.mode.clone();
        Ok(frame)
    }
}
