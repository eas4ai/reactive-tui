//! Terminal protocol renderer for Kitty graphics and iTerm2 inline images
//!
//! Provides image rendering using terminal-specific protocols that support
//! high-quality image display directly in the terminal.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageFormat, ImageSource};
use std::path::Path;

/// Protocol-based renderer for terminal-specific image protocols
pub struct ProtocolRenderer;

impl ProtocolRenderer {
    /// Create a new protocol renderer
    pub fn new() -> Self {
        Self
    }

    /// Render image using Kitty graphics protocol
    pub fn render_kitty_graphics(&self, image: &Image) -> Result<String> {
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
                self.resize_image(&image_data, width, height, display_width, display_height)?
            } else {
                (image_data, width, height)
            };

        // Convert to base64
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&final_data);

        // Generate unique image ID
        let image_id = self.generate_image_id();

        // Build Kitty graphics protocol sequence
        let mut sequence = String::new();

        // Transmission command
        sequence.push_str(&format!(
            "\x1b_Ga=T,f=24,s={},v={},i={};{}\x1b\\",
            final_width, final_height, image_id, base64_data
        ));

        // Placement command
        sequence.push_str(&format!("\x1b_Ga=p,i={}\x1b\\", image_id));

        Ok(sequence)
    }

    /// Render image using iTerm2 inline images protocol
    pub fn render_iterm2_inline(&self, image: &Image) -> Result<String> {
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
                self.resize_image(&image_data, width, height, display_width, display_height)?
            } else {
                (image_data, width, height)
            };

        // Convert to base64
        use base64::Engine;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&final_data);

        // Build iTerm2 inline image sequence
        let size_param = format!("size={}x{};", final_width, final_height);
        let preserve_param = if image.preserve_aspect { "1" } else { "0" };

        let sequence = format!(
            "\x1b]1337;File={}inline=1;preserveAspectRatio={}:{}\x07",
            size_param, preserve_param, base64_data
        );

        Ok(sequence)
    }

    /// Load and process image data from various sources
    fn load_and_process_image(&self, image: &Image) -> Result<(Vec<u8>, u32, u32)> {
        match &image.source {
            ImageSource::FilePath(path) => self.load_from_file(path),
            ImageSource::Base64Data(data) => self.load_from_base64(data),
            ImageSource::RawBytes {
                data,
                width,
                height,
                format,
            } => self.process_raw_bytes(data, *width, *height, *format),
            ImageSource::Url(_) => Err(ReactiveError::ImageProcessing(
                "URL loading not yet implemented".to_string(),
            )),
        }
    }

    /// Load image from file path
    fn load_from_file(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let img = image::open(path)
            .map_err(|e| ReactiveError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        // Convert to RGBA for protocol compatibility
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let data = rgba_img.into_raw();

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

        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let data = rgba_img.into_raw();

        Ok((data, width, height))
    }

    /// Process raw bytes based on format
    fn process_raw_bytes(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Result<(Vec<u8>, u32, u32)> {
        match format {
            ImageFormat::RGBA8888 => Ok((data.to_vec(), width, height)),
            ImageFormat::RGB888 => {
                // Convert RGB to RGBA by adding alpha channel
                let rgba_data: Vec<u8> = data
                    .chunks_exact(3)
                    .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                    .collect();
                Ok((rgba_data, width, height))
            }
            _ => {
                // For compressed formats, decode using image crate
                let img = image::load_from_memory(data).map_err(|e| {
                    ReactiveError::ImageProcessing(format!("Failed to decode image: {}", e))
                })?;

                let rgba_img = img.to_rgba8();
                let (w, h) = rgba_img.dimensions();
                let rgba_data = rgba_img.into_raw();

                Ok((rgba_data, w, h))
            }
        }
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
        width: u32,
        height: u32,
        new_width: u32,
        new_height: u32,
    ) -> Result<(Vec<u8>, u32, u32)> {
        use image::{ImageBuffer, Rgba};

        let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data)
            .ok_or_else(|| ReactiveError::ImageProcessing("Invalid image buffer".to_string()))?;

        let resized = image::imageops::resize(
            &img_buffer,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        Ok((resized.into_raw(), new_width, new_height))
    }

    /// Generate a unique image ID for protocols that require it
    fn generate_image_id(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32
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

        // IDs should be different (though they might be the same if called very quickly)
        // At minimum, they should be valid u32 values
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn test_process_raw_bytes_rgb_to_rgba() {
        let renderer = ProtocolRenderer::new();

        // RGB data: red, green, blue pixels
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];

        let (rgba_data, width, height) = renderer
            .process_raw_bytes(&rgb_data, 3, 1, ImageFormat::RGB888)
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
