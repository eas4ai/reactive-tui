//! Terminal protocol renderer for Kitty graphics and iTerm2 inline images
//!
//! Provides image rendering using terminal-specific protocols that support
//! high-quality image display directly in the terminal.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::Image;
#[cfg(test)]
use crate::widgets::display::image::{ImageFormat, ImageSource};

/// Protocol-based renderer for terminal-specific image protocols
pub struct ProtocolRenderer;

impl ProtocolRenderer {
    /// Create a new protocol renderer
    pub fn new() -> Self {
        Self
    }

    /// Render image using Kitty graphics protocol
    pub fn render_kitty_graphics(&self, image: &Image) -> Result<String> {
        if image.has_empty_size() {
            return Ok(String::new());
        }
        let (image_data, width, height) = self.load_and_process_image(image)?;

        // Calculate display dimensions
        let (display_width, display_height) = self.calculate_display_size(
            width,
            height,
            image.size_constraints,
            image.preserve_aspect,
        )?;

        // Resize if needed
        let (final_data, final_width, final_height) =
            if display_width != width || display_height != height {
                self.resize_image(
                    &image_data,
                    (width, height),
                    (display_width, display_height),
                    image.quality,
                )?
            } else {
                (image_data, width, height)
            };

        let pixels = image::RgbaImage::from_raw(final_width, final_height, final_data)
            .ok_or_else(|| ReactiveError::ImageProcessing("Invalid RGBA image buffer".into()))?;
        Ok(Self::kitty_pixels(
            &pixels,
            self.generate_image_id(),
            0,
            false,
        ))
    }

    pub(crate) fn kitty_pixels(
        pixels: &image::RgbaImage,
        image_id: u32,
        z: i32,
        keep_cursor: bool,
    ) -> String {
        Self::kitty_pixels_at(pixels, image_id, z, keep_cursor, None)
    }

    pub(crate) fn kitty_pixels_at(
        pixels: &image::RgbaImage,
        image_id: u32,
        z: i32,
        keep_cursor: bool,
        offset: Option<(u16, u16)>,
    ) -> String {
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(pixels.as_raw());
        let (width, height) = pixels.dimensions();
        let cursor = if keep_cursor { ",C=1" } else { "" };
        let offset = offset
            .map(|(x, y)| format!(",X={x},Y={y}"))
            .unwrap_or_default();
        let mut sequence = String::new();
        let mut chunks = base64_data.as_bytes().chunks(4096).peekable();
        let mut first = true;
        while let Some(chunk) = chunks.next() {
            let more = u8::from(chunks.peek().is_some());
            if first {
                sequence.push_str(&format!(
                    "\x1b_Ga=T,f=32,s={width},v={height},i={image_id},q=2{cursor},z={z}{offset},m={more};"
                ));
                first = false;
            } else {
                sequence.push_str(&format!("\x1b_Gm={more};"));
            }
            sequence.extend(chunk.iter().map(|byte| char::from(*byte)));
            sequence.push_str("\x1b\\");
        }
        sequence
    }

    /// Render image using iTerm2 inline images protocol
    pub fn render_iterm2_inline(&self, image: &Image) -> Result<String> {
        if image.has_empty_size() {
            return Ok(String::new());
        }
        let (image_data, width, height) = self.load_and_process_image(image)?;

        // Calculate display dimensions
        let (display_width, display_height) = self.calculate_display_size(
            width,
            height,
            image.size_constraints,
            image.preserve_aspect,
        )?;

        // Resize if needed
        let (final_data, final_width, final_height) =
            if display_width != width || display_height != height {
                self.resize_image(
                    &image_data,
                    (width, height),
                    (display_width, display_height),
                    image.quality,
                )?
            } else {
                (image_data, width, height)
            };

        // Inline images carry an encoded file, not raw pixel bytes.
        let pixels = image::RgbaImage::from_raw(final_width, final_height, final_data)
            .ok_or_else(|| ReactiveError::ImageProcessing("Invalid RGBA image buffer".into()))?;
        Self::iterm_pixels(&pixels, image.preserve_aspect)
    }

    pub(crate) fn iterm_pixels(pixels: &image::RgbaImage, preserve_aspect: bool) -> Result<String> {
        let (final_width, final_height) = pixels.dimensions();
        let mut png = std::io::Cursor::new(Vec::new());
        pixels
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|error| {
                ReactiveError::ImageProcessing(format!("Cannot encode inline image: {error}"))
            })?;
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(png.get_ref());
        let preserve = u8::from(preserve_aspect);
        let sequence = format!(
            "\x1b]1337;File=size={};width={final_width}px;height={final_height}px;inline=1;preserveAspectRatio={preserve}:{base64_data}\x07",
            png.get_ref().len()
        );

        Ok(sequence)
    }

    /// Load and process image data from various sources
    fn load_and_process_image(&self, image: &Image) -> Result<(Vec<u8>, u32, u32)> {
        let mut pixels = super::decoded::load(&image.source)?;
        if let Some(color) = image.background_color {
            let background = image::Rgba([
                (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]);
            for pixel in pixels.pixels_mut() {
                let mut composite = background;
                super::decoded::blend_pixel(&mut composite, pixel);
                *pixel = composite;
            }
        }
        let (width, height) = pixels.dimensions();
        Ok((pixels.into_raw(), width, height))
    }

    /// Calculate display size based on constraints
    fn calculate_display_size(
        &self,
        orig_width: u32,
        orig_height: u32,
        size_constraints: Option<(u32, u32)>,
        preserve_aspect: bool,
    ) -> Result<(u32, u32)> {
        let (max_width, max_height) = if let Some((w, h)) = size_constraints {
            (w, h)
        } else {
            // Use terminal size as default constraint
            let (term_cols, term_rows) =
                crate::core::terminal::Terminal::get_size().unwrap_or((80, 24));
            // Convert terminal characters to approximate pixel size
            (term_cols as u32 * 10, term_rows as u32 * 20)
        };

        if !preserve_aspect {
            return Ok((max_width.min(orig_width), max_height.min(orig_height)));
        }

        // Calculate aspect-preserving dimensions
        let width_ratio = max_width as f32 / orig_width as f32;
        let height_ratio = max_height as f32 / orig_height as f32;
        let ratio = width_ratio.min(height_ratio).min(1.0); // Don't upscale

        let new_width = (orig_width as f32 * ratio) as u32;
        let new_height = (orig_height as f32 * ratio) as u32;

        Ok((new_width.max(1), new_height.max(1)))
    }

    /// Resize image data
    fn resize_image(
        &self,
        data: &[u8],
        (width, height): (u32, u32),
        (new_width, new_height): (u32, u32),
        quality: super::ImageQuality,
    ) -> Result<(Vec<u8>, u32, u32)> {
        use image::{ImageBuffer, Rgba};

        let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data)
            .ok_or_else(|| ReactiveError::ImageProcessing("Invalid image buffer".to_string()))?;

        let resized = image::imageops::resize(
            &img_buffer,
            new_width,
            new_height,
            match quality {
                super::ImageQuality::Fast => image::imageops::FilterType::Nearest,
                super::ImageQuality::Balanced => image::imageops::FilterType::Triangle,
                super::ImageQuality::High => image::imageops::FilterType::Lanczos3,
            },
        );

        Ok((resized.into_raw(), new_width, new_height))
    }

    /// Generate a unique image ID for protocols that require it
    pub(crate) fn generate_image_id(&self) -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                Some(id.checked_add(1).unwrap_or(1))
            })
            .expect("image ID update always returns a value")
    }
}

impl Default for ProtocolRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_image_protocol_background_and_fast_resize_reach_encoded_pixels() {
        use base64::Engine;
        let mut input = Image::from_raw_bytes(
            vec![255, 0, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0, 0, 0, 255, 0],
            4,
            1,
            ImageFormat::RGBA8888,
        )
        .with_max_size(2, 1)
        .with_preserve_aspect(false);
        input.background_color = Some(crate::core::surface::Rgba {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        });
        input.quality = super::super::ImageQuality::Fast;
        let renderer = ProtocolRenderer::new();
        let output = renderer.render_kitty_graphics(&input).unwrap();
        let payload = output
            .split_once(';')
            .unwrap()
            .1
            .strip_suffix("\x1b\\")
            .unwrap();
        let pixels = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .unwrap();
        assert_eq!(pixels, [0, 255, 0, 255, 0, 255, 0, 255]);
        input.background_color = None;
        input.source = super::super::ImageSource::RawBytes {
            data: vec![
                255, 0, 0, 255, 0, 0, 255, 255, 255, 0, 0, 255, 0, 0, 255, 255,
            ],
            width: 4,
            height: 1,
            format: ImageFormat::RGBA8888,
        };
        let output = renderer.render_kitty_graphics(&input).unwrap();
        let payload = output
            .split_once(';')
            .unwrap()
            .1
            .strip_suffix("\x1b\\")
            .unwrap();
        let pixels = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .unwrap();
        assert_eq!(pixels, [0, 0, 255, 255, 0, 0, 255, 255]);
    }

    #[test]
    fn api_image_protocol_kitty_transmits_one_chunked_rgba_image() {
        use base64::Engine;
        let pixels: Vec<u8> = (0..4096).map(|index| (index % 251) as u8).collect();
        let image = Image::from_raw_bytes(pixels.clone(), 32, 32, ImageFormat::RGBA8888)
            .with_max_size(32, 32);
        let output = ProtocolRenderer::new()
            .render_kitty_graphics(&image)
            .unwrap();
        let mut payload = String::new();
        let commands: Vec<_> = output.split("\x1b_G").skip(1).collect();
        assert!(commands.len() > 1, "image must be chunked");
        for (index, command) in commands.iter().enumerate() {
            let command = command.strip_suffix("\x1b\\").unwrap();
            let (control, data) = command.split_once(';').unwrap();
            if index == 0 {
                assert!(control.split(',').any(|field| field == "f=32"), "{control}");
                assert!(control.split(',').any(|field| field == "a=T"));
                assert!(control.split(',').any(|field| field == "s=32"));
                assert!(control.split(',').any(|field| field == "v=32"));
            } else {
                assert!(!control.contains("a="), "one transmit-and-display action");
            }
            assert!(data.len() <= 4096);
            let last = index + 1 == commands.len();
            assert!(control
                .split(',')
                .any(|field| field == if last { "m=0" } else { "m=1" }));
            if !last {
                assert_eq!(data.len() % 4, 0);
            }
            payload.push_str(data);
        }
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(payload)
                .unwrap(),
            pixels
        );
    }

    #[test]
    fn api_image_protocol_iterm_encodes_an_image_file_with_byte_size() {
        use base64::Engine;
        let pixels = vec![255, 0, 0, 128, 0, 255, 0, 255];
        let image =
            Image::from_raw_bytes(pixels.clone(), 2, 1, ImageFormat::RGBA8888).with_max_size(2, 1);
        let output = ProtocolRenderer::new()
            .render_iterm2_inline(&image)
            .unwrap();
        let sequence = output
            .strip_prefix("\x1b]1337;File=")
            .unwrap()
            .strip_suffix('\x07')
            .unwrap();
        let (control, payload) = sequence.split_once(':').unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .unwrap();
        let size: usize = control
            .split(';')
            .find_map(|field| field.strip_prefix("size="))
            .unwrap()
            .parse()
            .expect("size counts bytes");
        assert_eq!(size, bytes.len());
        assert!(control.split(';').any(|field| field == "width=2px"));
        assert!(control.split(';').any(|field| field == "height=1px"));
        let decoded = image::load_from_memory(&bytes)
            .expect("inline payload is an encoded file")
            .to_rgba8();
        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.into_raw(), pixels);
    }

    #[test]
    fn api_image_protocol_rejects_invalid_raw_extents() {
        let renderer = ProtocolRenderer::new();
        for (bytes, width, height) in [
            (vec![0; 3], 1, 1),
            (vec![0; 5], 1, 1),
            (vec![], 0, 0),
            (vec![], u32::MAX, u32::MAX),
        ] {
            let image = Image::from_raw_bytes(bytes, width, height, ImageFormat::RGBA8888)
                .with_max_size(1, 1);
            assert!(
                renderer.render_kitty_graphics(&image).is_err(),
                "{width}x{height}"
            );
        }
    }

    #[test]
    fn test_calculate_display_size_preserve_aspect() {
        let renderer = ProtocolRenderer::new();

        let (width, height) = renderer
            .calculate_display_size(100, 200, Some((50, 150)), true)
            .unwrap();

        // Should scale to fit within constraints while preserving aspect ratio
        // Width ratio: 50/100 = 0.5, Height ratio: 150/200 = 0.75
        // Min ratio: 0.5, so: width = 100 * 0.5 = 50, height = 200 * 0.5 = 100
        assert_eq!(width, 50); // 100 * 0.5
        assert_eq!(height, 100); // 200 * 0.5
    }

    #[test]
    fn test_generate_image_id() {
        let renderer = ProtocolRenderer::new();

        let id1 = renderer.generate_image_id();
        let id2 = renderer.generate_image_id();

        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn test_process_raw_bytes_rgb_to_rgba() {
        let renderer = ProtocolRenderer::new();

        // RGB data: red, green, blue pixels
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];

        let (rgba_data, width, height) = renderer
            .load_and_process_image(&Image {
                source: ImageSource::RawBytes {
                    data: rgb_data,
                    width: 3,
                    height: 1,
                    format: ImageFormat::RGB888,
                },
                ..Image::default()
            })
            .unwrap();

        assert_eq!(width, 3);
        assert_eq!(height, 1);
        assert_eq!(rgba_data.len(), 12); // 3 pixels * 4 bytes

        // Check that alpha channel was added
        assert_eq!(
            rgba_data,
            vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]
        );
    }
}
