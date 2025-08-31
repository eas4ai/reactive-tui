//! Image rendering support for terminal graphics
//!
//! Based on libvaxis Image.zig with support for multiple protocols

use super::ImageFormat;
use crate::error::Result;
use std::io::Write;

/// Image source data
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// File path
    Path(String),
    /// Raw image data in memory
    Memory(Vec<u8>),
    /// Raw RGB/RGBA pixel data
    Pixels {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: PixelFormat,
    },
}

/// Pixel data format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb,
    Rgba,
}

/// Image transmission format for Kitty protocol
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransmitFormat {
    Rgb,
    Rgba,
    Png,
}

/// Image transmission medium
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransmitMedium {
    File,
    TempFile,
    SharedMem,
    Direct,
}

/// Image scaling modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleMode {
    /// No scaling applied
    None,
    /// Stretch/shrink to fill the window
    Fill,
    /// Scale to fit window, maintaining aspect ratio
    Fit,
    /// Scale to fit window, only if needed
    Contain,
}

impl Default for ScaleMode {
    fn default() -> Self {
        ScaleMode::None
    }
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

    /// Explicit size in terminal cells
    pub size: Option<(u16, u16)>,
}

/// Image clipping region
#[derive(Debug, Clone)]
pub struct ClipRegion {
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Image placement in terminal
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    pub img_id: u32,
    pub options: DrawOptions,
}

/// Main image structure
#[derive(Debug)]
pub struct Image {
    /// Unique identifier
    pub id: u32,

    /// Image dimensions in pixels
    pub width: u32,
    pub height: u32,

    /// Image source data
    pub source: ImageSource,

    /// Detected or specified format
    pub format: ImageFormat,
}

impl Image {
    /// Create image from file path
    pub fn from_file(path: &str) -> Result<Self> {
        let format = detect_format_from_path(path)?;

        Ok(Self {
            id: generate_image_id(),
            width: 0, // Will be detected when loaded
            height: 0,
            source: ImageSource::Path(path.to_string()),
            format,
        })
    }

    /// Create image from memory data
    pub fn from_memory(data: Vec<u8>, format: ImageFormat) -> Result<Self> {
        let (width, height) = detect_dimensions(&data, format)?;

        Ok(Self {
            id: generate_image_id(),
            width,
            height,
            source: ImageSource::Memory(data),
            format,
        })
    }

    /// Create image from raw pixel data
    pub fn from_pixels(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            id: generate_image_id(),
            width,
            height,
            source: ImageSource::Pixels {
                data,
                width,
                height,
                format,
            },
            format: match format {
                PixelFormat::Rgb => ImageFormat::Raw,
                PixelFormat::Rgba => ImageFormat::Raw,
            },
        }
    }

    /// Render image using Kitty graphics protocol
    pub fn render_kitty<W: Write>(&self, writer: &mut W, opts: &DrawOptions) -> Result<()> {
        let _transmit_format = match &self.source {
            ImageSource::Pixels {
                format: PixelFormat::Rgb,
                ..
            } => TransmitFormat::Rgb,
            ImageSource::Pixels {
                format: PixelFormat::Rgba,
                ..
            } => TransmitFormat::Rgba,
            _ => TransmitFormat::Png,
        };

        // Kitty graphics transmission sequence
        let opener = format!(
            "\x1b_Gf=32,i={},s={},v={},m=1;",
            self.id, self.width, self.height
        );

        writer.write_all(opener.as_bytes())?;

        // Encode and transmit image data
        match &self.source {
            ImageSource::Memory(data) => {
                let encoded = base64::encode(data);
                writer.write_all(encoded.as_bytes())?;
            }
            ImageSource::Pixels { data, .. } => {
                let encoded = base64::encode(data);
                writer.write_all(encoded.as_bytes())?;
            }
            ImageSource::Path(path) => {
                // For file paths, we could either read and encode,
                // or use Kitty's file transmission mode
                let data = std::fs::read(path)?;
                let encoded = base64::encode(&data);
                writer.write_all(encoded.as_bytes())?;
            }
        }

        // End transmission
        writer.write_all(b"\x1b\\")?;

        // Placement command
        if opts.pixel_offset.is_some() || opts.z_index.is_some() || opts.size.is_some() {
            let mut placement = format!("\x1b_Ga=p,i={}", self.id);

            if let Some((x, y)) = opts.pixel_offset {
                placement.push_str(&format!(",X={},Y={}", x, y));
            }

            if let Some(z) = opts.z_index {
                placement.push_str(&format!(",z={}", z));
            }

            if let Some((cols, rows)) = opts.size {
                placement.push_str(&format!(",c={},r={}", cols, rows));
            }

            placement.push_str(";\x1b\\");
            writer.write_all(placement.as_bytes())?;
        }

        Ok(())
    }

    /// Render image using Sixel protocol
    pub fn render_sixel<W: Write>(&self, writer: &mut W, _opts: &DrawOptions) -> Result<()> {
        // Basic sixel implementation
        // In a full implementation, this would convert the image to sixel format
        writer.write_all(b"\x1bPq")?; // Start sixel sequence

        // For now, just a placeholder - real implementation would:
        // 1. Convert image to 6-pixel high bands
        // 2. Quantize colors to sixel palette
        // 3. Encode each band as sixel data

        writer.write_all(b"\x1b\\")?; // End sixel sequence
        Ok(())
    }

    /// Render image using iTerm2 inline images
    pub fn render_iterm2<W: Write>(&self, writer: &mut W, opts: &DrawOptions) -> Result<()> {
        let mut sequence = String::from("\x1b]1337;File=");

        // Add parameters
        sequence.push_str(&format!("width={}px;height={}px", self.width, self.height));

        if let Some((cols, rows)) = opts.size {
            sequence.push_str(&format!(";size={}x{}", cols, rows));
        }

        sequence.push_str(":"); // End parameters

        // Add base64-encoded image data
        match &self.source {
            ImageSource::Memory(data) => {
                sequence.push_str(&base64::encode(data));
            }
            ImageSource::Path(path) => {
                let data = std::fs::read(path)?;
                sequence.push_str(&base64::encode(&data));
            }
            ImageSource::Pixels { data, .. } => {
                sequence.push_str(&base64::encode(data));
            }
        }

        sequence.push('\x07'); // End sequence
        writer.write_all(sequence.as_bytes())?;

        Ok(())
    }
}

/// Detect image format from file extension
fn detect_format_from_path(path: &str) -> Result<ImageFormat> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "webp" => Ok(ImageFormat::WebP),
        "six" | "sixel" => Ok(ImageFormat::Sixel),
        _ => Err(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported image format").into(),
        ),
    }
}

/// Detect image dimensions from data
fn detect_dimensions(data: &[u8], format: ImageFormat) -> Result<(u32, u32)> {
    match format {
        ImageFormat::Png => detect_png_dimensions(data),
        ImageFormat::Jpeg => detect_jpeg_dimensions(data),
        ImageFormat::Gif => detect_gif_dimensions(data),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cannot detect dimensions for this format",
        )
        .into()),
    }
}

/// Detect PNG dimensions (simplified)
fn detect_png_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() < 24 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid PNG header").into(),
        );
    }

    // IHDR chunk starts at offset 8
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

    Ok((width, height))
}

/// Detect JPEG dimensions (simplified)
fn detect_jpeg_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() < 4 || &data[0..2] != b"\xff\xd8" {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JPEG header").into(),
        );
    }

    // This is a simplified parser - real implementation would parse JPEG segments
    // For now, return a placeholder
    Ok((800, 600))
}

/// Detect GIF dimensions (simplified)
fn detect_gif_dimensions(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() < 10 || (&data[0..6] != b"GIF87a" && &data[0..6] != b"GIF89a") {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid GIF header").into(),
        );
    }

    let width = u16::from_le_bytes([data[6], data[7]]) as u32;
    let height = u16::from_le_bytes([data[8], data[9]]) as u32;

    Ok((width, height))
}

/// Generate unique image ID
fn generate_image_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Base64 encoding (simplified - in production use a proper base64 crate)
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        // This is a placeholder - use a proper base64 implementation
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();

        for chunk in data.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);

            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

            result.push(CHARS[((n >> 18) & 63) as usize] as char);
            result.push(CHARS[((n >> 12) & 63) as usize] as char);
            result.push(if chunk.len() > 1 {
                CHARS[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            result.push(if chunk.len() > 2 {
                CHARS[(n & 63) as usize] as char
            } else {
                '='
            });
        }

        result
    }
}
