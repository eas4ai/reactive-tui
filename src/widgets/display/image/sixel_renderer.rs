//! Sixel graphics renderer for native terminal image display
//!
//! Provides high-quality image rendering using the sixel graphics protocol,
//! which is supported by many modern terminals including xterm, wezterm, and mlterm.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageQuality};

/// Sixel graphics renderer
pub struct SixelRenderer;

impl SixelRenderer {
    /// Create a new sixel renderer
    pub fn new() -> Self {
        Self
    }

    /// Render an image using sixel graphics
    pub fn render_image(&self, image: &Image) -> Result<String> {
        if image.has_empty_size() {
            return Ok(String::new());
        }
        // Load and process the image data
        let (image_data, width, height) = self.load_image_data(image)?;

        // Calculate display dimensions
        let (display_width, display_height) = self.calculate_display_size(
            width,
            height,
            image.size_constraints,
            image.preserve_aspect,
        )?;

        // Resize image if needed
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

        // Convert to RGB888 format for sixel
        let rgb_data = self.ensure_rgb888(&final_data, final_width, final_height)?;

        let pixels = image::RgbaImage::from_fn(final_width, final_height, |x, y| {
            let offset = (y as usize * final_width as usize + x as usize) * 3;
            image::Rgba([
                rgb_data[offset],
                rgb_data[offset + 1],
                rgb_data[offset + 2],
                255,
            ])
        });
        super::sixel_encode::encode(&pixels, image.quality)
    }

    pub(crate) fn encode_pixels(
        pixels: &image::RgbaImage,
        quality: ImageQuality,
    ) -> Result<String> {
        // Sixel has binary transparency. Blend partial alpha against black;
        // preserve fully transparent pixels for clipping and pixel offsets.
        let rgb = super::decoded::rgb_pixels(pixels, None);
        let pixels = image::RgbaImage::from_fn(pixels.width(), pixels.height(), |x, y| {
            let color = rgb.get_pixel(x, y);
            image::Rgba([
                color[0],
                color[1],
                color[2],
                if pixels.get_pixel(x, y)[3] == 0 {
                    0
                } else {
                    255
                },
            ])
        });
        super::sixel_encode::encode(&pixels, quality)
    }

    /// Load image data from various sources
    fn load_image_data(&self, image: &Image) -> Result<(Vec<u8>, u32, u32)> {
        let pixels = super::decoded::rgb(image)?;
        let (width, height) = pixels.dimensions();
        Ok((pixels.into_raw(), width, height))
    }

    /// Calculate display size based on constraints and terminal size
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
            // Assuming roughly 2:1 character aspect ratio
            (term_cols as u32 * 8, term_rows as u32 * 16)
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
        use image::{ImageBuffer, Rgb};

        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data)
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

    /// Ensure image data is in RGB888 format
    fn ensure_rgb888(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        // Validate data size with overflow protection
        let expected_size = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(3))
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or_else(|| {
                ReactiveError::ImageProcessing(format!(
                    "Image dimensions too large: {}x{} would overflow",
                    width, height
                ))
            })?;

        if data.len() != expected_size {
            return Err(ReactiveError::ImageProcessing(format!(
                "Invalid data size: expected {}, got {}",
                expected_size,
                data.len()
            )));
        }

        // Data is already RGB888
        Ok(data.to_vec())
    }
}

impl Default for SixelRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_display_size_preserve_aspect() {
        let renderer = SixelRenderer::new();

        // Test aspect ratio preservation
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
    fn test_calculate_display_size_no_upscale() {
        let renderer = SixelRenderer::new();

        // Test that images are not upscaled
        let (width, height) = renderer
            .calculate_display_size(10, 20, Some((100, 200)), true)
            .unwrap();

        assert_eq!(width, 10); // Original size maintained
        assert_eq!(height, 20);
    }

    #[test]
    fn test_ensure_rgb888_validation() {
        let renderer = SixelRenderer::new();

        // Test with correct size
        let data = vec![255u8; 300]; // 10x10x3
        let result = renderer.ensure_rgb888(&data, 10, 10);
        assert!(result.is_ok());

        // Test with incorrect size
        let data = vec![255u8; 299]; // Wrong size
        let result = renderer.ensure_rgb888(&data, 10, 10);
        assert!(result.is_err());
    }
}
