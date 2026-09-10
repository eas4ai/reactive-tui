//! Image rendering support for terminal graphics
//!
//! Based on libvaxis Image.zig with support for multiple protocols

use super::ImageFormat;
use crate::error::Result;
use crate::widgets::display::image::{
    decoded, Image as WidgetImage, ImageFormat as WidgetFormat, ProtocolRenderer, SixelRenderer,
};
use std::io::Write;

mod sixel_decode;

/// Image source data
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// File path
    Path(String),
    /// Encoded image file bytes in memory
    Memory(Vec<u8>),
    /// Raw RGB/RGBA pixel data
    Pixels {
        /// Raw pixel data bytes
        data: Vec<u8>,
        /// Image width in pixels
        width: u32,
        /// Image height in pixels
        height: u32,
        /// Pixel format specification
        format: PixelFormat,
    },
}

/// Pixel data format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    /// RGB format (3 bytes per pixel: red, green, blue)
    Rgb,
    /// RGBA format (4 bytes per pixel: red, green, blue, alpha)
    Rgba,
}

/// Image transmission format for Kitty protocol
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransmitFormat {
    /// Raw RGB data transmission
    Rgb,
    /// Raw RGBA data transmission
    Rgba,
    /// PNG compressed image transmission
    Png,
}

/// Image transmission medium
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransmitMedium {
    /// Transmit via file path
    File,
    /// Transmit via temporary file
    TempFile,
    /// Transmit via shared memory
    SharedMem,
    /// Transmit data directly in escape sequence
    Direct,
}

/// Image scaling modes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ScaleMode {
    /// No scaling applied
    #[default]
    None,
    /// Stretch/shrink to fill the window
    Fill,
    /// Scale to fit window, maintaining aspect ratio
    Fit,
    /// Scale to fit window, only if needed
    Contain,
}

/// Image placement options
#[derive(Debug, Clone, Default)]
pub struct DrawOptions {
    /// Pixel offset within the top-left cell
    pub pixel_offset: Option<(u16, u16)>,

    /// Vertical stacking order (z-index)
    /// < 0: Below text
    /// < -1_073_741_824: Below default background
    pub z_index: Option<i32>,

    /// Clip region of source image
    pub clip_region: Option<ClipRegion>,

    /// Scaling mode
    pub scale: ScaleMode,

    /// Drawing box in terminal cells. None scaling clips; other modes scale to fit this box.
    pub size: Option<(u16, u16)>,
}

/// Image clipping region
#[derive(Debug, Clone)]
pub struct ClipRegion {
    /// X coordinate of the clipping region (None = no clipping)
    pub x: Option<u32>,
    /// Y coordinate of the clipping region (None = no clipping)
    pub y: Option<u32>,
    /// Width of the clipping region (None = no clipping)
    pub width: Option<u32>,
    /// Height of the clipping region (None = no clipping)
    pub height: Option<u32>,
}

/// Image placement in terminal
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// Unique identifier for the image
    pub img_id: u32,
    /// Drawing options for the image placement
    pub options: DrawOptions,
}

/// Main image structure
#[derive(Debug)]
pub struct Image {
    /// Unique identifier
    pub id: u32,

    /// Image dimensions in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,

    /// Image source data
    pub source: ImageSource,

    /// Detected or specified format
    pub format: ImageFormat,
}

impl Image {
    /// Load and validate an encoded image from a regular file.
    pub fn from_file(path: &str) -> Result<Self> {
        let format = detect_format_from_path(path)?;
        let bytes = decoded::read_file(std::path::Path::new(path))?;
        let pixels = decode(&bytes, format)?;
        Ok(Self {
            id: ProtocolRenderer::new().generate_image_id(),
            width: pixels.width(),
            height: pixels.height(),
            source: ImageSource::Path(path.to_owned()),
            format,
        })
    }

    /// Decode encoded file bytes. Raw data requires `from_pixels` and dimensions.
    pub fn from_memory(data: Vec<u8>, format: ImageFormat) -> Result<Self> {
        let pixels = decode(&data, format)?;
        Ok(Self {
            id: ProtocolRenderer::new().generate_image_id(),
            width: pixels.width(),
            height: pixels.height(),
            source: ImageSource::Memory(data),
            format,
        })
    }

    /// Store raw pixels. Rendering validates their dimensions and exact byte count.
    pub fn from_pixels(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            id: ProtocolRenderer::new().generate_image_id(),
            width,
            height,
            source: ImageSource::Pixels {
                data,
                width,
                height,
                format,
            },
            format: ImageFormat::Raw,
        }
    }

    fn pixels(&self) -> Result<image::RgbaImage> {
        match &self.source {
            ImageSource::Path(path) => decode(
                &decoded::read_file(std::path::Path::new(path))?,
                self.format,
            ),
            ImageSource::Memory(bytes) => decode(bytes, self.format),
            ImageSource::Pixels {
                data,
                width,
                height,
                format,
            } => decoded::raw(
                data,
                *width,
                *height,
                match format {
                    PixelFormat::Rgb => WidgetFormat::RGB888,
                    PixelFormat::Rgba => WidgetFormat::RGBA8888,
                },
            ),
        }
    }

    /// Transmit and display one decoded, cropped image at the current cursor.
    pub fn render_kitty<W: Write>(&self, writer: &mut W, opts: &DrawOptions) -> Result<()> {
        if self.id == 0 {
            return Err(image_error("Kitty image ID must be nonzero"));
        }
        if let Some(pixels) = self.prepared(opts, false)? {
            let output = ProtocolRenderer::kitty_pixels_at(
                &pixels,
                self.id,
                opts.z_index.unwrap_or(0),
                false,
                opts.pixel_offset,
            );
            writer.write_all(output.as_bytes())?;
        }
        Ok(())
    }

    /// Encode decoded pixels as Sixel. Nonzero z-order is Kitty-specific.
    pub fn render_sixel<W: Write>(&self, writer: &mut W, opts: &DrawOptions) -> Result<()> {
        reject_z_order(opts)?;
        if let Some(pixels) = self.prepared(opts, true)? {
            let output =
                SixelRenderer::encode_pixels(&pixels, crate::widgets::ImageQuality::Balanced)?;
            writer.write_all(output.as_bytes())?;
        }
        Ok(())
    }

    /// Encode a PNG file for iTerm2 inline display. Nonzero z-order is Kitty-specific.
    pub fn render_iterm2<W: Write>(&self, writer: &mut W, opts: &DrawOptions) -> Result<()> {
        reject_z_order(opts)?;
        if let Some(pixels) = self.prepared(opts, true)? {
            let image = widget_image(pixels);
            let output = ProtocolRenderer::new().render_iterm2_inline(&image)?;
            writer.write_all(output.as_bytes())?;
        }
        Ok(())
    }

    fn prepared(&self, opts: &DrawOptions, pad_offset: bool) -> Result<Option<image::RgbaImage>> {
        let mut pixels = self.pixels()?;
        if let Some(clip) = &opts.clip_region {
            let x = clip.x.unwrap_or(0).min(pixels.width());
            let y = clip.y.unwrap_or(0).min(pixels.height());
            let width = clip.width.unwrap_or(pixels.width()).min(pixels.width() - x);
            let height = clip
                .height
                .unwrap_or(pixels.height())
                .min(pixels.height() - y);
            if width == 0 || height == 0 {
                return Ok(None);
            }
            pixels = image::imageops::crop_imm(&pixels, x, y, width, height).to_image();
        }
        if opts.size.is_some() || opts.scale != ScaleMode::None {
            let (width, height) = drawing_box(opts.size);
            if width == 0 || height == 0 {
                return Ok(None);
            }
            if opts.scale == ScaleMode::None {
                pixels = image::imageops::crop_imm(
                    &pixels,
                    0,
                    0,
                    pixels.width().min(width),
                    pixels.height().min(height),
                )
                .to_image();
            } else {
                let (width, height) = if opts.scale == ScaleMode::Fill {
                    (width, height)
                } else {
                    let ratio = (width as f64 / pixels.width() as f64)
                        .min(height as f64 / pixels.height() as f64);
                    let ratio = if opts.scale == ScaleMode::Contain {
                        ratio.min(1.0)
                    } else {
                        ratio
                    };
                    (
                        ((pixels.width() as f64 * ratio) as u32).max(1),
                        ((pixels.height() as f64 * ratio) as u32).max(1),
                    )
                };
                decoded::dimensions(width, height)?;
                if pixels.dimensions() != (width, height) {
                    pixels = image::imageops::resize(
                        &pixels,
                        width,
                        height,
                        image::imageops::FilterType::Lanczos3,
                    );
                }
            }
        }
        if let Some((x, y)) = opts.pixel_offset.filter(|_| pad_offset) {
            let width = pixels
                .width()
                .checked_add(u32::from(x))
                .ok_or_else(|| image_error("Image offset overflow"))?;
            let height = pixels
                .height()
                .checked_add(u32::from(y))
                .ok_or_else(|| image_error("Image offset overflow"))?;
            decoded::dimensions(width, height)?;
            let mut padded = image::RgbaImage::new(width, height);
            image::imageops::replace(&mut padded, &pixels, i64::from(x), i64::from(y));
            pixels = padded;
        }
        Ok(Some(pixels))
    }
}

fn image_error(message: &str) -> crate::error::ReactiveError {
    crate::error::ReactiveError::ImageProcessing(message.into())
}

fn reject_z_order(opts: &DrawOptions) -> Result<()> {
    if opts.z_index.is_some_and(|z| z != 0) {
        return Err(image_error("Nonzero image z-order requires Kitty graphics"));
    }
    Ok(())
}

fn widget_image(pixels: image::RgbaImage) -> WidgetImage {
    let (width, height) = pixels.dimensions();
    WidgetImage::from_raw_bytes(pixels.into_raw(), width, height, WidgetFormat::RGBA8888)
        .with_max_size(width, height)
        .with_preserve_aspect(false)
}

fn drawing_box(size: Option<(u16, u16)>) -> (u32, u32) {
    let window = crossterm::terminal::window_size().ok();
    let cell = window
        .filter(|s| s.columns > 0 && s.rows > 0 && s.width >= s.columns && s.height >= s.rows)
        .map(|s| (s.width / s.columns, s.height / s.rows))
        .unwrap_or((8, 16));
    let (cols, rows) = size
        .or_else(|| crossterm::terminal::size().ok())
        .unwrap_or((80, 24));
    (
        u32::from(cols) * u32::from(cell.0),
        u32::from(rows) * u32::from(cell.1),
    )
}

fn decode(bytes: &[u8], format: ImageFormat) -> Result<image::RgbaImage> {
    let format = match format {
        ImageFormat::Png => image::ImageFormat::Png,
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::Gif => image::ImageFormat::Gif,
        ImageFormat::WebP => image::ImageFormat::WebP,
        ImageFormat::Raw => {
            return Err(image_error(
                "Raw image data requires explicit dimensions and pixel format",
            ))
        }
        ImageFormat::Sixel => return sixel_decode::decode(bytes),
    };
    decoded::encoded_format(bytes, Some(format))
}

fn detect_format_from_path(path: &str) -> Result<ImageFormat> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "webp" => Ok(ImageFormat::WebP),
        "six" | "sixel" => Ok(ImageFormat::Sixel),
        _ => Err(image_error("Unsupported image file extension")),
    }
}

#[cfg(test)]
mod tests;
