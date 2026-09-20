//! Image decoding and PNG re-encoding (MED-001, MED-002).
//!
//! Ports the `image.zig` decode surface on the decided `image` crate
//! (see the decoder decision in `docs/decisions/`). Images are
//! caller-owned RGBA buffers; nothing is cached process-wide.

use image::ImageFormat;
use std::io::Cursor;

/// Decoded image: dimensions with row-major RGBA pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Image failure. Corrupt input reports; it never panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaError {
    /// Not one of PNG, JPEG, GIF, or WebP.
    UnsupportedFormat,
    /// Recognized container that does not decode.
    CorruptData,
    /// Decoded size exceeds the engine limits.
    TooLarge,
    /// PNG re-encoding failed.
    EncodeFailed,
}

impl DecodedImage {
    fn new(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        if pixels.len() == width as usize * height as usize * 4 {
            Some(Self {
                width,
                height,
                pixels,
            })
        } else {
            None
        }
    }

    /// Re-encode as PNG; decoding the result must reproduce this image
    /// (MED-002).
    pub fn encode_png(&self) -> Result<Vec<u8>, MediaError> {
        let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            self.width,
            self.height,
            self.pixels.clone(),
        )
        .ok_or(MediaError::EncodeFailed)?;
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(buffer)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .map_err(|_| MediaError::EncodeFailed)?;
        Ok(out)
    }
}

/// Decode PNG, JPEG, GIF (first frame), or WebP bytes to RGBA
/// (MED-001).
pub fn decode(bytes: &[u8]) -> Result<DecodedImage, MediaError> {
    let format = image::guess_format(bytes).map_err(|_| MediaError::UnsupportedFormat)?;
    match format {
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP => {}
        _ => return Err(MediaError::UnsupportedFormat),
    }
    let image = image::load_from_memory(bytes).map_err(|err| match err {
        image::ImageError::Limits(_) => MediaError::TooLarge,
        _ => MediaError::CorruptData,
    })?;
    let rgba = image.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    DecodedImage::new(width, height, rgba.into_raw()).ok_or(MediaError::CorruptData)
}
