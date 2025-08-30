//! Sixel graphics renderer for native terminal image display
//!
//! Provides high-quality image rendering using the sixel graphics protocol,
//! which is supported by many modern terminals including xterm, wezterm, and mlterm.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageFormat, ImageQuality, ImageSource};
use sixel_rs::{optflags::DiffusionMethod, pixelformat::PixelFormat, sixel_string};
use std::path::Path;

/// Sixel graphics renderer
pub struct SixelRenderer;

impl SixelRenderer {
    /// Create a new sixel renderer
    pub fn new() -> Self {
        Self
    }

    /// Render an image using sixel graphics
    pub fn render_image(&self, image: &Image) -> Result<String> {
        // Load and process the image data
        let (image_data, width, height) = self.load_image_data(&image.source)?;

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
                self.resize_image(&image_data, width, height, display_width, display_height)?
            } else {
                (image_data, width, height)
            };

        // Convert to RGB888 format for sixel
        let rgb_data = self.ensure_rgb888(&final_data, final_width, final_height)?;

        // Select diffusion method based on quality setting
        let diffusion_method = match image.quality {
            ImageQuality::Fast => DiffusionMethod::None,
            ImageQuality::Balanced => DiffusionMethod::Atkinson,
            ImageQuality::High => DiffusionMethod::Stucki,
        };

        // Generate sixel string
        let sixel_output = sixel_string(
            &rgb_data,
            final_width as i32,
            final_height as i32,
            PixelFormat::RGB888,
            diffusion_method,
        )
        .map_err(|e| ReactiveError::ImageProcessing(format!("Sixel encoding failed: {:?}", e)))?;

        Ok(sixel_output)
    }

    /// Load image data from various sources
    fn load_image_data(&self, source: &ImageSource) -> Result<(Vec<u8>, u32, u32)> {
        match source {
            ImageSource::FilePath(path) => self.load_from_file(path),
            ImageSource::Base64Data(data) => self.load_from_base64(data),
            ImageSource::RawBytes {
                data,
                width,
                height,
                format,
            } => self.load_from_raw_bytes(data, *width, *height, *format),
            ImageSource::Url(_) => Err(ReactiveError::ImageProcessing(
                "URL loading not yet implemented".to_string(),
            )),
        }
    }

    /// Load image from file path
    fn load_from_file(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let img = image::open(path)
            .map_err(|e| ReactiveError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let data = rgb_img.into_raw();

        Ok((data, width, height))
    }

    /// Load image from base64 data
    fn load_from_base64(&self, base64_data: &str) -> Result<(Vec<u8>, u32, u32)> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| ReactiveError::ImageProcessing(format!("Invalid base64 data: {}", e)))?;

        let img = image::load_from_memory(&decoded).map_err(|e| {
            ReactiveError::ImageProcessing(format!("Failed to decode image: {}", e))
        })?;

        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let data = rgb_img.into_raw();

        Ok((data, width, height))
    }

    /// Load image from raw bytes
    fn load_from_raw_bytes(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(Vec<u8>, u32, u32)> {
        match format {
            ImageFormat::RGB888 => Ok((data.to_vec(), width, height)),
            ImageFormat::RGBA8888 => {
                // Convert RGBA to RGB by dropping alpha channel
                let rgb_data: Vec<u8> = data
                    .chunks_exact(4)
                    .flat_map(|rgba| [rgba[0], rgba[1], rgba[2]])
                    .collect();
                Ok((rgb_data, width, height))
            }
            _ => {
                // For compressed formats, decode using image crate
                let img = image::load_from_memory(data).map_err(|e| {
                    ReactiveError::ImageProcessing(format!("Failed to decode image: {}", e))
                })?;

                let rgb_img = img.to_rgb8();
                let (w, h) = rgb_img.dimensions();
                let rgb_data = rgb_img.into_raw();

                Ok((rgb_data, w, h))
            }
        }
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
        width: u32,
        height: u32,
        new_width: u32,
        new_height: u32,
    ) -> Result<(Vec<u8>, u32, u32)> {
        use image::{ImageBuffer, Rgb};

        let img_buffer = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data)
            .ok_or_else(|| ReactiveError::ImageProcessing("Invalid image buffer".to_string()))?;

        let resized = image::imageops::resize(
            &img_buffer,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        Ok((resized.into_raw(), new_width, new_height))
    }

    /// Ensure image data is in RGB888 format
    fn ensure_rgb888(&self, data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        // Validate data size
        let expected_size = (width * height * 3) as usize;
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
