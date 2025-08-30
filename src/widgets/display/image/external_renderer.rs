//! External tool renderer for chafa and viu
//!
//! Provides image rendering using external command-line tools like chafa and viu.
//! Based on moggu's approach with enhanced error handling and configuration.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageQuality, ImageSource};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// External tool renderer for chafa and viu
pub struct ExternalRenderer;

impl ExternalRenderer {
    /// Create a new external renderer
    pub fn new() -> Self {
        Self
    }

    /// Render image using chafa
    pub fn render_with_chafa(&self, image: &Image) -> Result<String> {
        let temp_file = self.prepare_temp_file(image)?;
        let result = self.render_chafa_file(&temp_file, image);

        // Clean up temp file
        let _ = fs::remove_file(&temp_file);

        result
    }

    /// Render image using viu
    pub fn render_with_viu(&self, image: &Image) -> Result<String> {
        let temp_file = self.prepare_temp_file(image)?;
        let result = self.render_viu_file(&temp_file, image);

        // Clean up temp file
        let _ = fs::remove_file(&temp_file);

        result
    }

    /// Prepare a temporary file for external tools
    fn prepare_temp_file(&self, image: &Image) -> Result<std::path::PathBuf> {
        match &image.source {
            ImageSource::FilePath(path) => {
                // Use the file directly if it exists
                if path.exists() {
                    Ok(path.clone())
                } else {
                    Err(ReactiveError::ImageProcessing(format!(
                        "Image file not found: {}",
                        path.display()
                    )))
                }
            }
            ImageSource::Base64Data(data) => {
                // Decode and write to temp file
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|e| {
                        ReactiveError::ImageProcessing(format!("Invalid base64 data: {}", e))
                    })?;

                self.write_temp_file(&decoded, "png")
            }
            ImageSource::RawBytes { data, format, .. } => {
                let extension = match format {
                    crate::widgets::display::image::ImageFormat::PNG => "png",
                    crate::widgets::display::image::ImageFormat::JPEG => "jpg",
                    crate::widgets::display::image::ImageFormat::GIF => "gif",
                    crate::widgets::display::image::ImageFormat::BMP => "bmp",
                    crate::widgets::display::image::ImageFormat::TIFF => "tiff",
                    _ => "png", // Default to PNG for raw formats
                };

                self.write_temp_file(data, extension)
            }
            ImageSource::Url(_) => Err(ReactiveError::ImageProcessing(
                "URL loading not yet implemented".to_string(),
            )),
        }
    }

    /// Write data to a temporary file
    fn write_temp_file(&self, data: &[u8], extension: &str) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "reactive_tui_image_{}.{}",
            std::process::id(),
            extension
        ));

        let mut file = fs::File::create(&temp_file).map_err(|e| {
            ReactiveError::ImageProcessing(format!("Failed to create temp file: {}", e))
        })?;

        file.write_all(data).map_err(|e| {
            ReactiveError::ImageProcessing(format!("Failed to write temp file: {}", e))
        })?;

        Ok(temp_file)
    }

    /// Render file using chafa
    fn render_chafa_file(&self, file_path: &Path, image: &Image) -> Result<String> {
        let (max_width, max_height) = self.get_display_constraints(image)?;

        let mut cmd = Command::new("chafa");

        // Size constraints
        cmd.arg("--size")
            .arg(format!("{}x{}", max_width, max_height));

        // Quality settings based on image quality
        match image.quality {
            ImageQuality::Fast => {
                cmd.arg("--colors=16");
                cmd.arg("--symbols=ascii");
            }
            ImageQuality::Balanced => {
                cmd.arg("--colors=256");
                cmd.arg("--symbols=block");
            }
            ImageQuality::High => {
                cmd.arg("--colors=256");
                cmd.arg("--symbols=block+border+space");
                cmd.arg("--dither=bayer");
            }
        }

        // Additional options
        cmd.arg("--stretch");
        cmd.arg("--animate=off");

        // Background color if specified
        if let Some(bg_color) = &image.background_color {
            let color_hex = format!(
                "#{:02x}{:02x}{:02x}",
                (bg_color.r * 255.0) as u8,
                (bg_color.g * 255.0) as u8,
                (bg_color.b * 255.0) as u8
            );
            cmd.arg("--bg").arg(color_hex);
        }

        cmd.arg(file_path);

        let output = cmd
            .output()
            .map_err(|e| ReactiveError::ExternalTool(format!("Failed to execute chafa: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(ReactiveError::ExternalTool(format!(
                "Chafa failed: {}",
                error_msg
            )))
        }
    }

    /// Render file using viu
    fn render_viu_file(&self, file_path: &Path, image: &Image) -> Result<String> {
        let (max_width, max_height) = self.get_display_constraints(image)?;

        let mut cmd = Command::new("viu");

        // Size constraints
        cmd.arg("-w").arg(max_width.to_string());
        cmd.arg("-h").arg(max_height.to_string());

        // Quality settings
        match image.quality {
            ImageQuality::Fast => {
                cmd.arg("--static");
            }
            ImageQuality::Balanced => {
                cmd.arg("--static");
            }
            ImageQuality::High => {
                cmd.arg("--static");
                // viu doesn't have as many quality options as chafa
            }
        }

        cmd.arg(file_path);

        let output = cmd
            .output()
            .map_err(|e| ReactiveError::ExternalTool(format!("Failed to execute viu: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(ReactiveError::ExternalTool(format!(
                "Viu failed: {}",
                error_msg
            )))
        }
    }

    /// Get display size constraints
    fn get_display_constraints(&self, image: &Image) -> Result<(u32, u32)> {
        if let Some((width, height)) = image.size_constraints {
            Ok((width, height))
        } else {
            // Use terminal size as default
            let (term_cols, term_rows) =
                crate::core::terminal::Terminal::get_size().unwrap_or((80, 24));

            // Leave some margin for UI elements
            let max_width = (term_cols as u32).saturating_sub(4).max(20);
            let max_height = (term_rows as u32).saturating_sub(2).max(10);

            Ok((max_width, max_height))
        }
    }

    /// Check if chafa is available
    pub fn is_chafa_available() -> bool {
        Command::new("chafa")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if viu is available
    pub fn is_viu_available() -> bool {
        Command::new("viu")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get installation instructions for missing tools
    pub fn get_installation_help() -> String {
        let mut help = String::new();

        help.push_str("📷 Image display tools not found.\n\n");

        if !Self::is_chafa_available() {
            help.push_str("For the best image preview experience, install chafa:\n\n");
            help.push_str("• Arch Linux / Manjaro:     sudo pacman -S chafa\n");
            help.push_str("• Ubuntu / Debian:          sudo apt install chafa\n");
            help.push_str("• Fedora / RHEL / CentOS:   sudo dnf install chafa\n");
            help.push_str("• macOS (Homebrew):         brew install chafa\n");
            help.push_str("• Windows (WSL):            sudo apt install chafa\n\n");
        }

        if !Self::is_viu_available() {
            help.push_str("Alternative: Install viu (Rust-based):\n");
            help.push_str("• cargo install viu\n\n");
        }

        help.push_str("After installation, restart the application for best results!");

        help
    }
}

impl Default for ExternalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::display::image::Image;

    #[test]
    fn test_get_display_constraints_with_size() {
        let renderer = ExternalRenderer::new();
        let image = Image::default().with_max_size(100, 50);

        let (width, height) = renderer.get_display_constraints(&image).unwrap();
        assert_eq!(width, 100);
        assert_eq!(height, 50);
    }

    #[test]
    fn test_get_display_constraints_default() {
        let renderer = ExternalRenderer::new();
        let image = Image::default();

        let (width, height) = renderer.get_display_constraints(&image).unwrap();
        // Should use terminal size with margins
        assert!(width >= 20);
        assert!(height >= 10);
    }

    #[test]
    fn test_write_temp_file() {
        let renderer = ExternalRenderer::new();
        let data = b"test data";

        let temp_file = renderer.write_temp_file(data, "txt").unwrap();
        assert!(temp_file.exists());

        let content = fs::read(&temp_file).unwrap();
        assert_eq!(content, data);

        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}
