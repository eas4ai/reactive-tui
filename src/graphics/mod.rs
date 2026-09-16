//! Offscreen graphics without a window or a separate terminal presentation path.

mod animation;
mod canvas;
mod cpu;
mod hybrid;
pub use animation::{
    FrameClock, FrameRequest, GraphicsCancellation, GraphicsWorker, WorkerOutput, WorkerStats,
};
pub use canvas::GraphicsCanvas;
pub use hybrid::{GraphicsFault, GraphicsMode, GraphicsOptions, HybridCubeRenderer};

use crate::{builder::div, component::Element, layout::style::StyleBuilder};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

/// Maximum checked pixel width, independent of adapter limits.
pub const MAX_WIDTH: u32 = 800;
/// Maximum checked pixel height (two pixels per terminal row).
pub const MAX_HEIGHT: u32 = 600;

/// Smooth, independently wrapped Y/X rotation angles computed from elapsed time.
pub fn cube_angles(elapsed: Duration) -> [f32; 2] {
    let seconds = elapsed.as_secs_f64();
    [
        (seconds * 0.7 % std::f64::consts::TAU) as f32,
        (seconds * (0.7 * 0.63) % std::f64::consts::TAU) as f32,
    ]
}

/// A graphics failure that callers may replace with a CPU frame.
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    /// Invalid dimensions or a mismatched pixel count.
    #[error("invalid graphics dimensions or pixel count: {0}")]
    Dimensions(String),
    /// Adapter or device initialization failed.
    #[error("GPU initialization failed: {0}")]
    Initialization(String),
    /// GPU completion or readback failed.
    #[error("GPU readback failed: {0}")]
    Readback(String),
    /// The owner cancelled work during shutdown.
    #[error("graphics work cancelled")]
    Cancelled,
}

fn pixel_count(width: u32, height: u32) -> Result<usize, GraphicsError> {
    if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
        return Err(GraphicsError::Dimensions(format!(
            "{width}x{height}; maximum {MAX_WIDTH}x{MAX_HEIGHT}"
        )));
    }
    width
        .checked_mul(height)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| GraphicsError::Dimensions("pixel count overflow".into()))
}

/// An owned, tightly packed, opaque-or-alpha RGBA image.
#[derive(Debug, Clone)]
pub struct GraphicsFrame {
    width: u32,
    height: u32,
    pixels: Vec<[u8; 4]>,
    mode: GraphicsMode,
}

impl GraphicsFrame {
    /// Validate dimensions and ownership before accepting an RGBA frame.
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<[u8; 4]>) -> Result<Self, GraphicsError> {
        if pixel_count(width, height)? != pixels.len() {
            return Err(GraphicsError::Dimensions(
                "RGBA pixel count mismatch".into(),
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
            mode: GraphicsMode::CpuFallback("externally supplied pixels".into()),
        })
    }

    /// Pixel width.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Pixel height.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Row-major RGBA pixels.
    pub fn pixels(&self) -> &[[u8; 4]] {
        &self.pixels
    }

    /// Provenance of these pixels, never inferred from a requested mode.
    pub fn mode(&self) -> &GraphicsMode {
        &self.mode
    }

    /// Present two vertical pixels per cell using the existing styled text path.
    /// Terminal cells are assumed to be twice as tall as they are wide.
    pub fn to_half_block_element(&self) -> Result<Element, GraphicsError> {
        if !self.height.is_multiple_of(2) {
            return Err(GraphicsError::Dimensions(
                "half-block height must be even".into(),
            ));
        }
        let width = self.width as usize;
        let rows = self
            .pixels
            .chunks_exact(width * 2)
            .map(|pair| {
                let mut cells = Vec::new();
                let mut x = 0;
                while x < width {
                    let start = x;
                    let upper_pixel = pair[x];
                    let lower_pixel = pair[width + x];
                    x += 1;
                    while x < width && pair[x] == upper_pixel && pair[width + x] == lower_pixel {
                        x += 1;
                    }
                    let top = upper_pixel.map(|value| value as f32 / 255.0);
                    let bottom = lower_pixel.map(|value| value as f32 / 255.0);
                    cells.push(
                        div()
                            .class(&format!("w-{} h-1 shrink-0", x - start))
                            .text(&"▀".repeat(x - start))
                            .styles(
                                StyleBuilder::new()
                                    .fg_rgba(top[0], top[1], top[2], top[3])
                                    .bg_rgba(bottom[0], bottom[1], bottom[2], bottom[3]),
                            )
                            .build(),
                    );
                }
                div().class("flex-row h-1 shrink-0").children(cells).build()
            })
            .collect();
        Ok(div().class("flex-col").children(rows).build())
    }
}

/// Identity of the adapter that actually renders the pixels.
#[derive(Debug, Clone)]
pub struct GraphicsAdapterInfo {
    /// Driver-reported adapter name.
    pub name: String,
    /// Native graphics backend.
    pub backend: String,
    /// True only for an integrated or discrete hardware GPU.
    pub is_hardware: bool,
}

struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// Native adapter/device initialization only; never execute this on the app loop.
fn wait_future<T>(
    future: impl Future<Output = T>,
    cancellation: &GraphicsCancellation,
) -> Result<T, GraphicsError> {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        if cancellation.is_cancelled() {
            return Err(GraphicsError::Cancelled);
        }
        if std::time::Instant::now() >= deadline {
            return Err(GraphicsError::Initialization(
                "initialization deadline exceeded".into(),
            ));
        }
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => std::thread::park_timeout(Duration::from_millis(10)),
        }
    }
}

/// A shaded cube rendered to an offscreen wgpu texture, then read back as RGBA.
pub struct GpuCubeRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    info: GraphicsAdapterInfo,
    failure: Arc<Mutex<Option<String>>>,
}

impl GpuCubeRenderer {
    /// Initialize a native GPU without creating a window or surface.
    pub fn new() -> Result<Self, GraphicsError> {
        Self::new_cancellable(&GraphicsCancellation::default())
    }

    fn new_cancellable(cancellation: &GraphicsCancellation) -> Result<Self, GraphicsError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = wait_future(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }),
            cancellation,
        )?
        .map_err(|error| GraphicsError::Initialization(error.to_string()))?;
        let adapter_info = adapter.get_info();
        let info = GraphicsAdapterInfo {
            name: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
            is_hardware: matches!(
                adapter_info.device_type,
                wgpu::DeviceType::DiscreteGpu | wgpu::DeviceType::IntegratedGpu
            ),
        };
        let (device, queue) = wait_future(
            adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("terminal cube device"),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            }),
            cancellation,
        )?
        .map_err(|error| GraphicsError::Initialization(error.to_string()))?;
        let failure = Arc::new(Mutex::new(None));
        let lost_failure = Arc::clone(&failure);
        device.set_device_lost_callback(move |reason, message| {
            *lost_failure.lock().unwrap() = Some(format!("device-loss: {reason:?}: {message}"));
        });
        let error_failure = Arc::clone(&failure);
        device.on_uncaptured_error(Arc::new(move |error| {
            *error_failure.lock().unwrap() = Some(format!("GPU operation: {error}"));
        }));
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube time and viewport"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cube uniform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube uniforms"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaded cube"),
            source: wgpu::ShaderSource::Wgsl(include_str!("cube.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cube pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("offscreen cube"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        Ok(Self {
            device,
            queue,
            pipeline,
            uniform,
            bind_group,
            info,
            failure,
        })
    }

    /// Identity of the selected renderer, not a requested adapter label.
    pub fn adapter_info(&self) -> &GraphicsAdapterInfo {
        &self.info
    }

    /// Render at the terminal viewport size, correcting the cell's 2:1 aspect.
    pub fn render_terminal(
        &self,
        columns: u32,
        rows: u32,
        elapsed: Duration,
    ) -> Result<GraphicsFrame, GraphicsError> {
        let height = rows
            .checked_mul(2)
            .ok_or_else(|| GraphicsError::Dimensions("row overflow".into()))?;
        self.render_pixels(columns, height, elapsed)
    }

    /// Render checked pixel dimensions; target and readback allocation follow each resize.
    pub fn render_pixels(
        &self,
        width: u32,
        height: u32,
        elapsed: Duration,
    ) -> Result<GraphicsFrame, GraphicsError> {
        self.render_cancellable(
            width,
            height,
            elapsed,
            &GraphicsCancellation::default(),
            false,
        )
    }

    fn render_cancellable(
        &self,
        width: u32,
        height: u32,
        elapsed: Duration,
        cancellation: &GraphicsCancellation,
        inject_readback_failure: bool,
    ) -> Result<GraphicsFrame, GraphicsError> {
        let count = pixel_count(width, height)?;
        if cancellation.is_cancelled() {
            return Err(GraphicsError::Cancelled);
        }
        if let Some(failure) = self.failure.lock().unwrap().clone() {
            return Err(GraphicsError::Readback(failure));
        }
        let [angle, angle_x] = cube_angles(elapsed);
        let values = [
            angle,
            width as f32 / height as f32,
            width as f32,
            height as f32,
            angle_x,
            0.0,
            0.0,
            0.0,
        ];
        let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_ne_bytes).collect();
        self.queue.write_buffer(&self.uniform, 0, &bytes);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen terminal cube"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let row_bytes = width
            .checked_mul(4)
            .ok_or_else(|| GraphicsError::Dimensions("row byte overflow".into()))?;
        let padded_row_bytes = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube RGBA readback"),
            size: u64::from(padded_row_bytes) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let view = texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cube render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        if inject_readback_failure {
            return Err(GraphicsError::Readback("injected readback failure".into()));
        }
        let slice = buffer.slice(..);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if cancellation.is_cancelled() {
                return Err(GraphicsError::Cancelled);
            }
            if let Some(failure) = self.failure.lock().unwrap().clone() {
                return Err(GraphicsError::Readback(failure));
            }
            match self.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_millis(10)),
            }) {
                Ok(_) | Err(wgpu::PollError::Timeout) => {}
                Err(error) => return Err(GraphicsError::Readback(error.to_string())),
            }
            match receive.try_recv() {
                Ok(result) => {
                    result.map_err(|error| GraphicsError::Readback(error.to_string()))?;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(GraphicsError::Readback(
                        "readback callback disconnected".into(),
                    ))
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    std::thread::park_timeout(Duration::from_millis(1))
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(GraphicsError::Readback("readback deadline exceeded".into()));
            }
        }
        let mapped = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity(count);
        for row in mapped.chunks_exact(padded_row_bytes as usize) {
            if cancellation.is_cancelled() {
                return Err(GraphicsError::Cancelled);
            }
            for pixel in row[..row_bytes as usize].chunks_exact(4) {
                pixels.push([pixel[0], pixel[1], pixel[2], pixel[3]]);
            }
        }
        drop(mapped);
        buffer.unmap();
        let mut frame = GraphicsFrame::from_rgba(width, height, pixels)?;
        frame.mode = GraphicsMode::Gpu(self.info.clone());
        Ok(frame)
    }
}
