//! Image processing utilities for format conversion and ASCII art generation
//!
//! Provides various image processing capabilities including ASCII art conversion,
//! format conversion, and image manipulation utilities.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageFormat, ImageQuality, ImageSource};
use std::path::Path;

/// Image processor for various image manipulation tasks
pub struct ImageProcessor;

/// Configuration for ASCII art generation
#[derive(Debug, Clone)]
pub struct AsciiConfig {
    pub max_width: u32,
    pub contrast_boost: f32,
    pub invert: bool,
    pub detailed: bool,
    pub dither: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            contrast_boost: 1.2,
            invert: false,
            detailed: false,
            dither: true,
        }
    }
}

impl ImageProcessor {
    /// Create a new image processor
    pub fn new() -> Self {
        Self
    }

    /// Convert image to ASCII art
    pub fn to_ascii_art(&self, image: &Image) -> Result<String> {
        let config = self.create_ascii_config(image);
        let (image_data, width, height) = self.load_image_data(&image.source)?;

        self.convert_to_ascii(&image_data, width, height, &config)
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
            ImageSource::Url(url) => {
                // Load image from URL
                self.load_from_url(url)
            }
        }
    }

    /// Load image from URL
    fn load_from_url(&self, url: &str) -> Result<(Vec<u8>, u32, u32)> {
        // Check if it's a data URL (base64 encoded)
        if url.starts_with("data:image/") {
            if let Some(base64_start) = url.find("base64,") {
                let base64_data = &url[base64_start + 7..];
                return self.load_from_base64(base64_data);
            }
        }

        // For HTTP/HTTPS URLs, we'd need an HTTP client
        if url.starts_with("http://") || url.starts_with("https://") {
            return Err(ReactiveError::ImageProcessing(
                "HTTP URL loading requires adding reqwest dependency. Use data URLs or local files for now.".to_string()
            ));
        }

        // Try to treat as local file path
        let path = Path::new(url);
        self.load_from_file(path)
    }

    /// Load image from file path
    fn load_from_file(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let img = image::open(path)
            .map_err(|e| ReactiveError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        let gray_img = img.to_luma8();
        let (width, height) = gray_img.dimensions();
        let data = gray_img.into_raw();

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

        let gray_img = img.to_luma8();
        let (width, height) = gray_img.dimensions();
        let data = gray_img.into_raw();

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
            ImageFormat::RGB888 => {
                // Convert RGB to grayscale
                let gray_data: Vec<u8> = data
                    .chunks_exact(3)
                    .map(|rgb| {
                        // Use standard luminance formula
                        (0.299 * rgb[0] as f32 + 0.587 * rgb[1] as f32 + 0.114 * rgb[2] as f32)
                            as u8
                    })
                    .collect();
                Ok((gray_data, width, height))
            }
            ImageFormat::RGBA8888 => {
                // Convert RGBA to grayscale (ignore alpha)
                let gray_data: Vec<u8> = data
                    .chunks_exact(4)
                    .map(|rgba| {
                        (0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32)
                            as u8
                    })
                    .collect();
                Ok((gray_data, width, height))
            }
            _ => {
                // For compressed formats, decode using image crate
                let img = image::load_from_memory(data).map_err(|e| {
                    ReactiveError::ImageProcessing(format!("Failed to decode image: {}", e))
                })?;

                let gray_img = img.to_luma8();
                let (w, h) = gray_img.dimensions();
                let gray_data = gray_img.into_raw();

                Ok((gray_data, w, h))
            }
        }
    }

    /// Create ASCII configuration based on image settings
    fn create_ascii_config(&self, image: &Image) -> AsciiConfig {
        let max_width = image.size_constraints.map(|(w, _)| w).unwrap_or_else(|| {
            crate::core::terminal::Terminal::get_size()
                .map(|(cols, _)| cols as u32)
                .unwrap_or(80)
        });

        AsciiConfig {
            max_width,
            detailed: matches!(image.quality, ImageQuality::High),
            dither: !matches!(image.quality, ImageQuality::Fast),
            ..Default::default()
        }
    }

    /// Convert grayscale image data to ASCII art
    fn convert_to_ascii(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        config: &AsciiConfig,
    ) -> Result<String> {
        // ASCII character ramps
        let ascii_chars_detailed =
            "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
        let ascii_chars_simple = "@%#*+=-:. ";

        let char_ramp: Vec<char> = if config.detailed {
            ascii_chars_detailed.chars().collect()
        } else {
            ascii_chars_simple.chars().collect()
        };
        let ramp_len = char_ramp.len() as f32;

        // Calculate new dimensions
        let aspect_ratio = height as f32 / width as f32;
        let new_width = config.max_width;
        let new_height = (new_width as f32 * aspect_ratio * 0.5) as u32; // 0.5 for character aspect ratio

        // Resize image
        let resized_data = self.resize_grayscale(data, width, height, new_width, new_height)?;

        // Apply dithering if enabled
        let final_data = if config.dither {
            self.apply_floyd_steinberg_dithering(&resized_data, new_width, new_height, ramp_len)?
        } else {
            resized_data
        };

        // Convert to ASCII
        let mut ascii_art = String::with_capacity((new_width * new_height + new_height) as usize);

        for (i, &pixel) in final_data.iter().enumerate() {
            let mut brightness = pixel as f32;

            // Apply contrast boost
            brightness = ((brightness / 255.0 - 0.5) * config.contrast_boost + 0.5) * 255.0;
            brightness = brightness.clamp(0.0, 255.0);

            // Apply inversion if requested
            if config.invert {
                brightness = 255.0 - brightness;
            }

            // Map to character
            let char_index = ((brightness / 255.0) * (ramp_len - 1.0)).round() as usize;
            let char_index = char_index.min(char_ramp.len() - 1);
            ascii_art.push(char_ramp[char_index]);

            // Add newline at end of each row
            if (i + 1) % new_width as usize == 0 {
                ascii_art.push('\n');
            }
        }

        Ok(ascii_art)
    }

    /// Resize grayscale image data
    fn resize_grayscale(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        new_width: u32,
        new_height: u32,
    ) -> Result<Vec<u8>> {
        use image::{ImageBuffer, Luma};

        let img_buffer =
            ImageBuffer::<Luma<u8>, _>::from_raw(width, height, data).ok_or_else(|| {
                ReactiveError::ImageProcessing("Invalid grayscale image buffer".to_string())
            })?;

        let resized = image::imageops::resize(
            &img_buffer,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        Ok(resized.into_raw())
    }

    /// Apply Floyd-Steinberg dithering
    fn apply_floyd_steinberg_dithering(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        levels: f32,
    ) -> Result<Vec<u8>> {
        let mut f32_buffer: Vec<f32> = data.iter().map(|&p| p as f32).collect();

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let old_pixel = f32_buffer[idx];

                // Quantize pixel
                let new_pixel =
                    ((old_pixel / 255.0 * (levels - 1.0)).round() / (levels - 1.0)) * 255.0;
                f32_buffer[idx] = new_pixel;

                let quant_error = old_pixel - new_pixel;

                // Distribute error to neighboring pixels
                if x + 1 < width {
                    f32_buffer[idx + 1] += quant_error * 7.0 / 16.0;
                }
                if x > 0 && y + 1 < height {
                    f32_buffer[idx + width as usize - 1] += quant_error * 3.0 / 16.0;
                }
                if y + 1 < height {
                    f32_buffer[idx + width as usize] += quant_error * 5.0 / 16.0;
                }
                if x + 1 < width && y + 1 < height {
                    f32_buffer[idx + width as usize + 1] += quant_error * 1.0 / 16.0;
                }
            }
        }

        let final_pixels: Vec<u8> = f32_buffer
            .iter()
            .map(|&p| p.clamp(0.0, 255.0) as u8)
            .collect();

        Ok(final_pixels)
    }
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_config_default() {
        let config = AsciiConfig::default();
        assert_eq!(config.max_width, 80);
        assert_eq!(config.contrast_boost, 1.2);
        assert!(!config.invert);
        assert!(!config.detailed);
        assert!(config.dither);
    }

    #[test]
    fn test_rgb_to_grayscale_conversion() {
        let processor = ImageProcessor::new();

        // RGB data: pure red, pure green, pure blue
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];

        let (gray_data, width, height) = processor
            .load_from_raw_bytes(&rgb_data, 3, 1, ImageFormat::RGB888)
            .unwrap();

        assert_eq!(width, 3);
        assert_eq!(height, 1);
        assert_eq!(gray_data.len(), 3);

        // Check luminance conversion (approximate values)
        assert!(gray_data[0] > 70 && gray_data[0] < 80); // Red luminance ~76
        assert!(gray_data[1] > 140 && gray_data[1] < 150); // Green luminance ~149
        assert!(gray_data[2] > 25 && gray_data[2] < 35); // Blue luminance ~29
    }

    #[test]
    fn test_floyd_steinberg_dithering() {
        let processor = ImageProcessor::new();

        // Create a gradient
        let data: Vec<u8> = (0..100).map(|i| (i * 255 / 99) as u8).collect();

        let result = processor
            .apply_floyd_steinberg_dithering(&data, 10, 10, 4.0)
            .unwrap();

        assert_eq!(result.len(), 100);
        // Dithered values should be quantized to specific levels
        for &pixel in &result {
            let normalized = pixel as f32 / 255.0 * 3.0;
            let rounded = normalized.round();
            assert!((normalized - rounded).abs() < 0.1);
        }
    }
}
