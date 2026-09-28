//! The canvas on a hardware wgpu adapter (GFX-002). A frame is built on the
//! CPU as instances and drawn in a few passes: edges add their signed
//! areas into the coverage atlas, then covers paint each shape's window at
//! the coverage found there (canvas.wgsl). Glyph bitmaps and cell tiles
//! come from the glyph atlas, so a cell grid is one instanced draw
//! (GFX-001). A clip is a mask texture per depth of nesting.

use super::compile::Draw;
use super::cpu::{unpack_premultiplied, unpremultiply};
use super::geometry::Polygon;
use super::glyphs::{Bitmap, Glyphs};
use super::paint::{Shader, ShaderKind};
use super::raster::Window;
use super::scene::{CanvasImage, Transform};
use super::{GraphicsAdapterInfo, GraphicsError};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

/// How long one wait for the adapter, the device or a finished frame may
/// take before the canvas gives the hardware adapter up.
const DEADLINE: Duration = Duration::from_secs(5);

/// How many picture sizes keep their textures, and how many pixels they
/// may hold in all beside the size being drawn.
const KEPT_SIZES: usize = 3;
const KEPT_PIXELS: u64 = 24_000_000;

/// Floats per edge instance and per cover instance.
const EDGE_FLOATS: usize = 8;
const COVER_FLOATS: usize = 16;

/// How a cover finds its coverage (canvas.wgsl).
const FROM_ATLAS: f32 = 0.0;
const FROM_RECT: f32 = 1.0;
const FROM_GLYPH: f32 = 2.0;
const CELL: f32 = 3.0;
const CLIPPED: f32 = 8.0;

const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const COVERAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
const MASK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// Rows of rectangles packed left to right, a new row below when one fills.
struct Shelves {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    row: u32,
}

impl Shelves {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            row: 0,
        }
    }

    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width > self.width || height > self.height {
            return None;
        }
        if self.x + width > self.width {
            self.y += self.row;
            self.x = 0;
            self.row = 0;
        }
        if self.y + height > self.height {
            return None;
        }
        let at = (self.x, self.y);
        self.x += width;
        self.row = self.row.max(height);
        Some(at)
    }
}

/// The glyph atlas: coverage bitmaps by their id.
struct GlyphAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    shelves: Shelves,
    placed: HashMap<u32, (u32, u32)>,
    generation: u64,
}

impl GlyphAtlas {
    const START: u32 = 1024;
    const LARGEST: u32 = 4096;

    fn new(device: &wgpu::Device, side: u32, generation: u64) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("canvas glyph atlas"),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: MASK_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&Default::default()),
            texture,
            shelves: Shelves::new(side, side),
            placed: HashMap::new(),
            generation,
        }
    }

    fn side(&self) -> u32 {
        self.shelves.width
    }

    /// Place and upload `bitmap` unless it is there; false when it does not
    /// fit.
    fn add(&mut self, queue: &wgpu::Queue, bitmap: &Bitmap) -> bool {
        if self.placed.contains_key(&bitmap.id) {
            return true;
        }
        let Some((x, y)) = self.shelves.place(bitmap.width, bitmap.height) else {
            return false;
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap.coverage,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bitmap.width),
                rows_per_image: Some(bitmap.height),
            },
            wgpu::Extent3d {
                width: bitmap.width,
                height: bitmap.height,
                depth_or_array_layers: 1,
            },
        );
        self.placed.insert(bitmap.id, (x, y));
        true
    }
}

/// The textures of one picture size.
struct Targets {
    size: (u32, u32),
    picture: wgpu::Texture,
    picture_view: wgpu::TextureView,
    coverage_view: wgpu::TextureView,
    /// One mask per depth of clip nesting, made when first needed.
    masks: Vec<wgpu::TextureView>,
    readback: wgpu::Buffer,
    padded_row: u32,
}

/// An image a scene draws, as a texture, kept while the image lives.
struct ImageTexture {
    image: Weak<CanvasImage>,
    view: wgpu::TextureView,
}

/// Consecutive covers that share their image and their clip: one draw call.
struct Batch {
    covers: Range<u32>,
    image: Option<usize>,
    mask: Option<usize>,
}

/// Where a pass draws.
#[derive(Clone, Copy, PartialEq)]
enum Onto {
    Picture,
    Mask(usize),
}

enum Pass {
    /// Add edges into the cleared coverage atlas.
    Accumulate(Range<u32>),
    Cover {
        onto: Onto,
        clear: bool,
        batches: Vec<Batch>,
    },
}

/// A frame as instances and passes, before anything is sent to the GPU.
struct Frame {
    edges: Vec<f32>,
    covers: Vec<f32>,
    paints: Vec<[f32; 4]>,
    passes: Vec<Pass>,
    /// Images by the key of their texture.
    images: Vec<Arc<CanvasImage>>,
    shelves: Shelves,
    /// Edges and batches of the segment being built.
    first_edge: u32,
    batches: Vec<Batch>,
    cleared: bool,
    depth: usize,
    deepest: usize,
}

impl Frame {
    fn new(size: (u32, u32)) -> Self {
        Self {
            edges: Vec::new(),
            covers: Vec::new(),
            paints: Vec::new(),
            passes: Vec::new(),
            images: Vec::new(),
            shelves: Shelves::new(size.0, size.1),
            first_edge: 0,
            batches: Vec::new(),
            cleared: false,
            depth: 0,
            deepest: 0,
        }
    }

    fn edge_count(&self) -> u32 {
        (self.edges.len() / EDGE_FLOATS) as u32
    }

    fn cover_count(&self) -> u32 {
        (self.covers.len() / COVER_FLOATS) as u32
    }

    /// End the segment: its edges accumulate, then its covers draw onto
    /// `onto`. The next segment starts with an empty coverage atlas.
    fn end_segment(&mut self, onto: Onto) {
        let edges = self.first_edge..self.edge_count();
        if !edges.is_empty() {
            self.passes.push(Pass::Accumulate(edges));
        }
        let batches = std::mem::take(&mut self.batches);
        let clear = match onto {
            Onto::Picture => !std::mem::replace(&mut self.cleared, true),
            Onto::Mask(_) => true,
        };
        if clear || !batches.is_empty() {
            self.passes.push(Pass::Cover {
                onto,
                clear,
                batches,
            });
        }
        self.first_edge = self.edge_count();
        self.shelves = Shelves::new(self.shelves.width, self.shelves.height);
    }

    /// The rows of `shader` in the paint table, and its image.
    fn paint(&mut self, shader: &Shader) -> (f32, Option<usize>) {
        let first = self.paints.len() as f32;
        let inverse = shader.inverse;
        let (kind, shape, counts, stops, image) = match &shader.kind {
            ShaderKind::Solid(color) => (0.0, *color, [0.0; 4], None, None),
            ShaderKind::Linear { start, axis, stops } => (
                1.0,
                [start.0, start.1, axis.0, axis.1],
                [stops.len() as f32, 0.0, 0.0, 0.0],
                Some(stops),
                None,
            ),
            ShaderKind::Radial {
                center,
                radius,
                stops,
            } => (
                2.0,
                [center.0, center.1, *radius, 0.0],
                [stops.len() as f32, 0.0, 0.0, 0.0],
                Some(stops),
                None,
            ),
            ShaderKind::Image { rect, image } => (
                3.0,
                [rect.0, rect.1, rect.2, rect.3],
                [0.0, image.width as f32, image.height as f32, 0.0],
                None,
                Some(image),
            ),
        };
        self.paints
            .push([inverse.a, inverse.b, inverse.c, inverse.d]);
        self.paints
            .push([inverse.e, inverse.f, kind, shader.opacity]);
        self.paints.push(shape);
        self.paints.push(counts);
        for (offset, color) in stops.into_iter().flatten() {
            self.paints.push(*color);
            self.paints.push([*offset, 0.0, 0.0, 0.0]);
        }
        let image = image.map(|image| {
            self.images
                .iter()
                .position(|known| Arc::ptr_eq(known, image))
                .unwrap_or_else(|| {
                    self.images.push(image.clone());
                    self.images.len() - 1
                })
        });
        (first, image)
    }

    /// Add one cover to the segment's batches.
    fn cover(&mut self, floats: [f32; COVER_FLOATS], image: Option<usize>) {
        let index = self.cover_count();
        let mut floats = floats;
        let mask = self.depth.checked_sub(1);
        if mask.is_some() {
            floats[6] += CLIPPED;
        }
        self.covers.extend_from_slice(&floats);
        match self.batches.last_mut() {
            Some(batch) if batch.mask == mask && (image.is_none() || batch.image == image) => {
                batch.covers.end = index + 1;
            }
            Some(batch) if batch.mask == mask && batch.image.is_none() => {
                // No cover of the batch reads an image yet, so it takes
                // this one's.
                batch.image = image;
                batch.covers.end = index + 1;
            }
            _ => self.batches.push(Batch {
                covers: index..index + 1,
                image,
                mask,
            }),
        }
    }

    /// The edges of `polygons` into a window of the atlas; its top left
    /// corner there. Ends the segment first when the atlas is full.
    fn accumulate(&mut self, polygons: &[Polygon], window: Window) -> (u32, u32) {
        let (width, height) = (window.width(), window.height());
        let at = match self.shelves.place(width, height) {
            Some(at) => at,
            None => {
                self.end_segment(Onto::Picture);
                // A window is never larger than the picture, and the atlas
                // is the picture's size.
                self.shelves.place(width, height).unwrap_or((0, 0))
            }
        };
        let shift = (
            at.0 as f32 - window.left as f32,
            at.1 as f32 - window.top as f32,
        );
        let region = [
            at.0 as f32,
            at.1 as f32,
            (at.0 + width) as f32,
            (at.1 + height) as f32,
        ];
        for polygon in polygons {
            for index in 0..polygon.len() {
                let a = polygon[index];
                let b = polygon[(index + 1) % polygon.len()];
                let finite =
                    a.0.is_finite() && a.1.is_finite() && b.0.is_finite() && b.1.is_finite();
                if !finite || a.1 == b.1 {
                    continue;
                }
                self.edges.extend_from_slice(&[
                    a.0 + shift.0,
                    a.1 + shift.1,
                    b.0 + shift.0,
                    b.1 + shift.1,
                ]);
                self.edges.extend_from_slice(&region);
            }
        }
        at
    }

    fn fill(&mut self, polygons: &[Polygon], window: Window, shader: &Shader) {
        let at = self.accumulate(polygons, window);
        let (paint, image) = self.paint(shader);
        let mut floats = [0.0; COVER_FLOATS];
        floats[..4].copy_from_slice(&[
            window.left as f32,
            window.top as f32,
            window.right as f32,
            window.bottom as f32,
        ]);
        floats[4..8].copy_from_slice(&[at.0 as f32, at.1 as f32, FROM_ATLAS, paint]);
        self.cover(floats, image);
    }

    fn rect(&mut self, rect: [f32; 4], shader: &Shader) {
        let (paint, image) = self.paint(shader);
        let mut floats = [0.0; COVER_FLOATS];
        floats[..4].copy_from_slice(&rect);
        floats[4..8].copy_from_slice(&[0.0, 0.0, FROM_RECT, paint]);
        self.cover(floats, image);
    }

    fn push_clip(&mut self, polygons: &[Polygon], window: Window) {
        // What was drawn so far goes out under the clips it was drawn in.
        self.end_segment(Onto::Picture);
        let at = self.accumulate(polygons, window);
        // A mask holds how much of each pixel shows, not a color: the path
        // is painted at full value.
        let (full, _) = self.paint(&Shader {
            kind: ShaderKind::Solid([1.0; 4]),
            inverse: Transform::identity(),
            opacity: 1.0,
        });
        let mut floats = [0.0; COVER_FLOATS];
        floats[..4].copy_from_slice(&[
            window.left as f32,
            window.top as f32,
            window.right as f32,
            window.bottom as f32,
        ]);
        floats[4..8].copy_from_slice(&[at.0 as f32, at.1 as f32, FROM_ATLAS, full]);
        // The new mask shows what both the path and the clip around it show.
        self.cover(floats, None);
        self.end_segment(Onto::Mask(self.depth));
        self.depth += 1;
        self.deepest = self.deepest.max(self.depth);
    }

    fn pop_clip(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

/// A picture the hardware adapter drew.
pub(crate) struct Drawn {
    /// Straight RGBA, row by row.
    pub bytes: Vec<u8>,
    pub draw_calls: u32,
    /// Building the frame and handing it to the GPU.
    pub render: Duration,
    /// Waiting for the GPU to finish.
    pub wait: Duration,
    /// Copying the picture out.
    pub readback: Duration,
}

/// The canvas's renderer on one hardware adapter.
pub(crate) struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: GraphicsAdapterInfo,
    /// Set by the device when it is lost or an operation fails.
    failure: Arc<Mutex<Option<String>>>,
    edge_pipeline: wgpu::RenderPipeline,
    cover_pipeline: wgpu::RenderPipeline,
    mask_pipeline: wgpu::RenderPipeline,
    shared_layout: wgpu::BindGroupLayout,
    sources_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// One white pixel, bound where a batch has no image or no clip.
    blank_color: wgpu::TextureView,
    blank_mask: wgpu::TextureView,
    /// The textures of the sizes drawn last, the newest first. A renderer
    /// that draws two sizes in turn, such as a picture and its block
    /// glyphs, makes the textures of neither again.
    targets: Vec<Targets>,
    atlas: GlyphAtlas,
    images: HashMap<usize, ImageTexture>,
    edges: Option<wgpu::Buffer>,
    covers: Option<wgpu::Buffer>,
    paints: Option<wgpu::Buffer>,
}

/// Wait for `future` on this thread, up to the deadline. Only the canvas's
/// worker calls it (GFX-003).
fn wait<T>(future: impl std::future::Future<Output = T>) -> Result<T, GraphicsError> {
    use std::task::{Context, Poll, Wake, Waker};
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            return Err(GraphicsError::Initialization(
                "the adapter did not answer in time".into(),
            ));
        }
        std::thread::park_timeout(Duration::from_millis(10));
    }
}

fn floats_to_bytes(floats: &[f32]) -> Vec<u8> {
    floats
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

/// A texture of one pixel with every channel full.
fn blank(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    pixel: &[u8],
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("canvas blank"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(pixel.len() as u32),
            rows_per_image: Some(1),
        },
        size,
    );
    texture.create_view(&Default::default())
}

/// The hardware adapters of this host, the best first: discrete before
/// integrated, and the platform's own backend before Vulkan. A software
/// adapter such as WARP or lavapipe is not one (GFX-002).
fn hardware_adapters() -> Vec<wgpu::Adapter> {
    // The canvas draws into textures, so the instance needs no display.
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let mut adapters: Vec<wgpu::Adapter> = wait(instance.enumerate_adapters(wgpu::Backends::all()))
        .unwrap_or_default()
        .into_iter()
        .filter(|adapter| {
            matches!(
                adapter.get_info().device_type,
                wgpu::DeviceType::DiscreteGpu | wgpu::DeviceType::IntegratedGpu
            )
        })
        .collect();
    adapters.sort_by_key(|adapter| {
        let info = adapter.get_info();
        (
            info.device_type != wgpu::DeviceType::DiscreteGpu,
            match info.backend {
                wgpu::Backend::Metal | wgpu::Backend::Dx12 => 0,
                wgpu::Backend::Vulkan => 1,
                _ => 2,
            },
        )
    });
    adapters
}

impl GpuRenderer {
    /// The renderer on the best hardware adapter that gives a device; an
    /// error naming why when none does.
    pub fn new() -> Result<Self, GraphicsError> {
        let mut last = GraphicsError::Initialization("this host has no hardware adapter".into());
        for adapter in hardware_adapters() {
            match Self::on(&adapter) {
                Ok(renderer) => return Ok(renderer),
                Err(error) => last = error,
            }
        }
        Err(last)
    }

    fn on(adapter: &wgpu::Adapter) -> Result<Self, GraphicsError> {
        let adapter_info = adapter.get_info();
        let info = GraphicsAdapterInfo {
            name: adapter_info.name.clone(),
            backend: format!("{:?}", adapter_info.backend),
            hardware: true,
        };
        let (device, queue) = wait(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("canvas device"),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }))?
        .map_err(|error| {
            GraphicsError::Initialization(format!("{}: {error}", adapter_info.name))
        })?;
        let failure = Arc::new(Mutex::new(None));
        let lost = Arc::clone(&failure);
        device.set_device_lost_callback(move |reason, message| {
            if let Ok(mut failure) = lost.lock() {
                *failure = Some(format!("device-loss: {reason:?}: {message}"));
            }
        });
        let failed = Arc::clone(&failure);
        device.on_uncaptured_error(Arc::new(move |error| {
            if let Ok(mut failure) = failed.lock() {
                failure.get_or_insert_with(|| format!("GPU operation: {error}"));
            }
        }));

        let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let shared_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas shared"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture_entry(2),
                texture_entry(3),
            ],
        });
        let sources_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas sources"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture_entry(2),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("canvas"),
            source: wgpu::ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas"),
            bind_group_layouts: &[Some(&shared_layout), Some(&sources_layout)],
            immediate_size: 0,
        });
        let attributes = |count: u32| -> Vec<wgpu::VertexAttribute> {
            (0..count)
                .map(|location| wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: u64::from(location) * 16,
                    shader_location: location,
                })
                .collect()
        };
        let pipeline = |label: &str,
                        entries: (&str, &str),
                        vectors: u32,
                        format: wgpu::TextureFormat,
                        blend: wgpu::BlendState| {
            let attributes = attributes(vectors);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some(entries.0),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: u64::from(vectors) * 16,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &attributes,
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entries.1),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let add = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let adding = wgpu::BlendState {
            color: add,
            alpha: add,
        };
        let over = wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING;
        let edge_pipeline = pipeline(
            "canvas edges",
            ("edge_vertex", "edge_fragment"),
            (EDGE_FLOATS / 4) as u32,
            COVERAGE_FORMAT,
            adding,
        );
        let cover_pipeline = pipeline(
            "canvas covers",
            ("cover_vertex", "cover_fragment"),
            (COVER_FLOATS / 4) as u32,
            TARGET_FORMAT,
            over,
        );
        let mask_pipeline = pipeline(
            "canvas masks",
            ("cover_vertex", "cover_fragment"),
            (COVER_FLOATS / 4) as u32,
            MASK_FORMAT,
            over,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("canvas images"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let renderer = Self {
            blank_color: blank(&device, &queue, TARGET_FORMAT, &[255; 4]),
            blank_mask: blank(&device, &queue, MASK_FORMAT, &[255]),
            atlas: GlyphAtlas::new(&device, GlyphAtlas::START, 0),
            device,
            queue,
            info,
            failure,
            edge_pipeline,
            cover_pipeline,
            mask_pipeline,
            shared_layout,
            sources_layout,
            sampler,
            targets: Vec::new(),
            images: HashMap::new(),
            edges: None,
            covers: None,
            paints: None,
        };
        renderer.check()?;
        Ok(renderer)
    }

    /// The adapter that draws.
    pub fn info(&self) -> &GraphicsAdapterInfo {
        &self.info
    }

    /// Lose the device, as a fault would (GFX-007).
    pub fn lose_device(&self) {
        self.device.destroy();
        if let Ok(mut failure) = self.failure.lock() {
            *failure = Some("device-loss: injected".into());
        }
    }

    /// The failure the device reported, if it did.
    fn check(&self) -> Result<(), GraphicsError> {
        match self.failure.lock() {
            Ok(failure) => match failure.as_ref() {
                Some(failure) => Err(GraphicsError::Readback(failure.clone())),
                None => Ok(()),
            },
            Err(_) => Err(GraphicsError::Readback("the device's state is lost".into())),
        }
    }

    fn texture(
        &self,
        label: &str,
        size: (u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    }

    /// Put the textures for pictures of `size` first in `self.targets`,
    /// making them when no size drawn lately is this one.
    fn targets(&mut self, size: (u32, u32)) {
        if let Some(kept) = self.targets.iter().position(|kept| kept.size == size) {
            self.targets[..=kept].rotate_right(1);
            return;
        }
        let drawn = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let picture = self.texture(
            "canvas picture",
            size,
            TARGET_FORMAT,
            drawn | wgpu::TextureUsages::COPY_SRC,
        );
        let coverage = self.texture(
            "canvas coverage atlas",
            size,
            COVERAGE_FORMAT,
            drawn | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let padded_row = (size.0 * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canvas readback"),
            size: u64::from(padded_row) * u64::from(size.1),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.targets.insert(
            0,
            Targets {
                size,
                picture_view: picture.create_view(&Default::default()),
                picture,
                coverage_view: coverage.create_view(&Default::default()),
                masks: Vec::new(),
                readback,
                padded_row,
            },
        );
        // Older sizes go when there are too many or they hold too much.
        let mut pixels = 0u64;
        let mut kept = 0;
        for targets in &self.targets {
            pixels += u64::from(targets.size.0) * u64::from(targets.size.1);
            if kept > 0 && (kept >= KEPT_SIZES || pixels > KEPT_PIXELS) {
                break;
            }
            kept += 1;
        }
        self.targets.truncate(kept);
    }

    /// A buffer of at least `bytes.len()` bytes holding `bytes`, reusing
    /// `kept` when it is large enough.
    fn upload(
        &self,
        kept: Option<wgpu::Buffer>,
        bytes: &[u8],
        label: &str,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        // A binding needs a buffer that is not empty.
        let needed = (bytes.len() as u64).max(64);
        let buffer = match kept {
            Some(buffer) if buffer.size() >= needed => buffer,
            _ => self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: needed.next_power_of_two(),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        };
        if !bytes.is_empty() {
            self.queue.write_buffer(&buffer, 0, bytes);
        }
        buffer
    }

    /// Put every bitmap of `needed` into the glyph atlas, starting the
    /// atlas again, larger when it can grow, if they do not fit beside
    /// what it holds.
    fn place_glyphs(&mut self, needed: &[Arc<Bitmap>], generation: u64) {
        if self.atlas.generation != generation {
            self.atlas = GlyphAtlas::new(&self.device, self.atlas.side(), generation);
        }
        if needed
            .iter()
            .all(|bitmap| self.atlas.add(&self.queue, bitmap))
        {
            return;
        }
        let mut side = self.atlas.side();
        loop {
            self.atlas = GlyphAtlas::new(&self.device, side, generation);
            let fits = needed
                .iter()
                .all(|bitmap| self.atlas.add(&self.queue, bitmap));
            if fits || side >= GlyphAtlas::LARGEST {
                if !fits {
                    log::warn!("The canvas's glyph atlas is full; some glyphs are not drawn");
                }
                return;
            }
            side *= 2;
        }
    }

    fn image_texture(&mut self, image: &Arc<CanvasImage>) -> usize {
        let key = Arc::as_ptr(image) as usize;
        let known = self
            .images
            .get(&key)
            .is_some_and(|texture| texture.image.upgrade().is_some());
        if !known {
            let texture = self.texture(
                "canvas image",
                (image.width, image.height),
                TARGET_FORMAT,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.pixels.as_flattened(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width * 4),
                    rows_per_image: Some(image.height),
                },
                wgpu::Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
            );
            self.images.insert(
                key,
                ImageTexture {
                    image: Arc::downgrade(image),
                    view: texture.create_view(&Default::default()),
                },
            );
        }
        key
    }

    /// The instances and passes of `draws`.
    fn build(&mut self, draws: &[Draw], size: (u32, u32), glyphs: &mut Glyphs) -> Frame {
        // Every bitmap the frame needs goes into the atlas first, so no
        // draw finds the atlas changed under it.
        let mut needed: Vec<Arc<Bitmap>> = Vec::new();
        let mut tiles: Vec<Vec<Option<Arc<Bitmap>>>> = Vec::new();
        for draw in draws {
            match draw {
                Draw::Glyphs { glyphs, .. } => {
                    needed.extend(glyphs.iter().map(|glyph| glyph.bitmap.clone()));
                }
                Draw::Cells { grid, cell, .. } => {
                    // A grid names each of its glyphs once, so one lookup
                    // serves every cell that shows the glyph.
                    let mut known: HashMap<*const u8, Option<Arc<Bitmap>>> = HashMap::new();
                    let cells = grid
                        .painted()
                        .map(|(_, _, glyph, wide, _, _)| {
                            if glyph.is_empty() {
                                return None;
                            }
                            known
                                .entry(glyph.as_ptr())
                                .or_insert_with(|| glyphs.tile(glyph, *cell, wide))
                                .clone()
                        })
                        .collect();
                    needed.extend(known.into_values().flatten());
                    tiles.push(cells);
                }
                _ => {}
            }
        }
        self.place_glyphs(&needed, glyphs.generation());
        let mut frame = Frame::new(size);
        let mut tiles = tiles.into_iter();
        for draw in draws {
            match draw {
                Draw::Fill {
                    polygons,
                    window,
                    shader,
                } => frame.fill(polygons, *window, shader),
                Draw::Rect { rect, shader } => frame.rect(*rect, shader),
                Draw::Glyphs {
                    glyphs: placed,
                    shader,
                } => {
                    let (paint, image) = frame.paint(shader);
                    for glyph in placed {
                        let Some(&at) = self.atlas.placed.get(&glyph.bitmap.id) else {
                            continue;
                        };
                        let mut floats = [0.0; COVER_FLOATS];
                        floats[..4].copy_from_slice(&[
                            glyph.x as f32,
                            glyph.y as f32,
                            (glyph.x + glyph.bitmap.width as i32) as f32,
                            (glyph.y + glyph.bitmap.height as i32) as f32,
                        ]);
                        floats[4..8].copy_from_slice(&[
                            at.0 as f32,
                            at.1 as f32,
                            FROM_GLYPH,
                            paint,
                        ]);
                        frame.cover(floats, image);
                    }
                }
                Draw::Cells {
                    x,
                    y,
                    grid,
                    cell,
                    foreground,
                } => {
                    let tiles = tiles.next().unwrap_or_default();
                    for ((column, row, _, wide, fg, bg), tile) in grid.painted().zip(tiles) {
                        let wide = wide.clamp(1, 2) as i32;
                        let left = x + i32::from(column) * i32::from(cell.0);
                        let top = y + i32::from(row) * i32::from(cell.1);
                        let at = tile
                            .and_then(|tile| self.atlas.placed.get(&tile.id))
                            .map_or((-1.0, -1.0), |at| (at.0 as f32, at.1 as f32));
                        let mut floats = [0.0; COVER_FLOATS];
                        floats[..4].copy_from_slice(&[
                            left as f32,
                            top as f32,
                            (left + i32::from(cell.0) * wide) as f32,
                            (top + i32::from(cell.1)) as f32,
                        ]);
                        floats[4..8].copy_from_slice(&[at.0, at.1, CELL, 0.0]);
                        floats[8..12]
                            .copy_from_slice(&fg.map_or(*foreground, unpack_premultiplied));
                        floats[12..16].copy_from_slice(&bg.map_or([0.0; 4], unpack_premultiplied));
                        frame.cover(floats, None);
                    }
                }
                Draw::PushClip { polygons, window } => frame.push_clip(polygons, *window),
                Draw::PopClip => frame.pop_clip(),
            }
        }
        frame.end_segment(Onto::Picture);
        frame
    }

    /// Draw `draws` at `size` and read the picture back. `fail_readback`
    /// fails the readback, as a fault would (GFX-007).
    pub fn render(
        &mut self,
        draws: &[Draw],
        size: (u32, u32),
        glyphs: &mut Glyphs,
        fail_readback: bool,
    ) -> Result<Drawn, GraphicsError> {
        let started = Instant::now();
        std::thread::sleep(Duration::from_millis(20));
        self.check()?;
        self.images
            .retain(|_, texture| texture.image.upgrade().is_some());
        self.targets(size);
        let frame = self.build(draws, size, glyphs);
        let image_keys: Vec<usize> = frame
            .images
            .iter()
            .map(|image| self.image_texture(image))
            .collect();
        let (kept_edges, kept_covers, kept_paints) =
            (self.edges.take(), self.covers.take(), self.paints.take());
        let edges = self.upload(
            kept_edges,
            &floats_to_bytes(&frame.edges),
            "canvas edges",
            wgpu::BufferUsages::VERTEX,
        );
        let covers = self.upload(
            kept_covers,
            &floats_to_bytes(&frame.covers),
            "canvas covers",
            wgpu::BufferUsages::VERTEX,
        );
        let paints = self.upload(
            kept_paints,
            &floats_to_bytes(frame.paints.as_flattened()),
            "canvas paints",
            wgpu::BufferUsages::STORAGE,
        );
        let globals = self.upload(
            None,
            &floats_to_bytes(&[size.0 as f32, size.1 as f32, 0.0, 0.0]),
            "canvas globals",
            wgpu::BufferUsages::UNIFORM,
        );
        let mask_usage =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        while self
            .targets
            .first()
            .is_some_and(|targets| targets.masks.len() < frame.deepest)
        {
            let mask = self.texture("canvas clip", size, MASK_FORMAT, mask_usage);
            if let Some(targets) = self.targets.first_mut() {
                targets.masks.push(mask.create_view(&Default::default()));
            }
        }
        let targets = self.targets.first().ok_or_else(|| {
            GraphicsError::Readback("the canvas has no texture to draw into".into())
        })?;
        let shared = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("canvas shared"),
            layout: &self.shared_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: paints.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&targets.coverage_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                },
            ],
        });
        // The coverage atlas cannot be read while edges are drawn into it.
        let blank_shared = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("canvas shared for edges"),
            layout: &self.shared_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: paints.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.blank_mask),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                },
            ],
        });
        let sources = |image: Option<usize>, mask: Option<usize>| {
            let image = image
                .and_then(|index| image_keys.get(index))
                .and_then(|key| self.images.get(key))
                .map_or(&self.blank_color, |texture| &texture.view);
            let mask = mask
                .and_then(|depth| targets.masks.get(depth))
                .unwrap_or(&self.blank_mask);
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("canvas sources"),
                layout: &self.sources_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(image),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(mask),
                    },
                ],
            })
        };
        let no_sources = sources(None, None);
        let mut draw_calls = 0u32;
        let mut encoder = self.device.create_command_encoder(&Default::default());
        for pass in &frame.passes {
            let (view, clear, pipeline, group) = match pass {
                Pass::Accumulate(_) => (
                    &targets.coverage_view,
                    true,
                    &self.edge_pipeline,
                    &blank_shared,
                ),
                Pass::Cover {
                    onto: Onto::Picture,
                    clear,
                    ..
                } => (&targets.picture_view, *clear, &self.cover_pipeline, &shared),
                Pass::Cover {
                    onto: Onto::Mask(depth),
                    clear,
                    ..
                } => match targets.masks.get(*depth) {
                    Some(mask) => (mask, *clear, &self.mask_pipeline, &shared),
                    None => continue,
                },
            };
            let mut drawing = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("canvas"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            drawing.set_pipeline(pipeline);
            drawing.set_bind_group(0, group, &[]);
            match pass {
                Pass::Accumulate(range) => {
                    drawing.set_bind_group(1, &no_sources, &[]);
                    drawing.set_vertex_buffer(0, edges.slice(..));
                    drawing.draw(0..4, range.clone());
                    draw_calls += 1;
                }
                Pass::Cover { onto, batches, .. } => {
                    drawing.set_vertex_buffer(0, covers.slice(..));
                    for batch in batches {
                        // A mask is drawn from the mask of the clip around
                        // it, never from itself.
                        let mask = match onto {
                            Onto::Mask(depth) => batch.mask.filter(|mask| mask < depth),
                            Onto::Picture => batch.mask,
                        };
                        drawing.set_bind_group(1, &sources(batch.image, mask), &[]);
                        drawing.draw(0..4, batch.covers.clone());
                        draw_calls += 1;
                    }
                }
            }
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &targets.picture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &targets.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(targets.padded_row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let render = started.elapsed();
        let pixels = if fail_readback {
            Err(GraphicsError::Readback("injected readback failure".into()))
        } else {
            self.read_back(targets, size)
        };
        self.edges = Some(edges);
        self.covers = Some(covers);
        self.paints = Some(paints);
        pixels.map(|(bytes, wait, readback)| Drawn {
            bytes,
            draw_calls,
            render,
            wait,
            readback,
        })
    }

    /// The picture as straight RGBA bytes, with how long the GPU took to
    /// finish it and how long copying it out took.
    fn read_back(
        &self,
        targets: &Targets,
        size: (u32, u32),
    ) -> Result<(Vec<u8>, Duration, Duration), GraphicsError> {
        let started = Instant::now();
        let slice = targets.readback.slice(..);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
        let deadline = Instant::now() + DEADLINE;
        loop {
            self.check()?;
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
                        "the readback was dropped before it finished".into(),
                    ));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
            if Instant::now() >= deadline {
                return Err(GraphicsError::Readback(
                    "the picture was not read back in time".into(),
                ));
            }
        }
        let wait = started.elapsed();
        let started = Instant::now();
        let row = size.0 as usize * 4;
        let mut bytes = Vec::with_capacity(row * size.1 as usize);
        {
            let mapped = slice
                .get_mapped_range()
                .map_err(|error| GraphicsError::Readback(error.to_string()))?;
            if targets.padded_row as usize == row {
                bytes.extend_from_slice(&mapped[..row * size.1 as usize]);
            } else {
                for padded in mapped.chunks_exact(targets.padded_row as usize) {
                    bytes.extend_from_slice(&padded[..row]);
                }
            }
        }
        targets.readback.unmap();
        unpremultiply(&mut bytes);
        self.check()?;
        Ok((bytes, wait, started.elapsed()))
    }
}
