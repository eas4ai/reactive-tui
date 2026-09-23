//! Image widget with multi-backend rendering support
//!
//! Provides comprehensive image display capabilities for terminal applications,
//! supporting multiple rendering backends including sixel, external tools, and
//! terminal-specific protocols.

pub(crate) mod decoded;
mod external_renderer;
mod image_processor;
mod live;
pub(crate) mod paint;
mod protocol_renderer;
mod sixel_encode;
mod sixel_renderer;

pub use ::suprtui::blit::Blitter;
pub use external_renderer::ExternalRenderer;
pub use image_processor::ImageProcessor;
pub use protocol_renderer::ProtocolRenderer;
pub use sixel_renderer::SixelRenderer;

/// The application's blitter override: 0 for none, otherwise one more than
/// the blitter's place in [`Blitter::TIERS`].
static BLITTER_OVERRIDE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Choose the blitter image cell fallback draws with, or return to the
/// choice by tier with `None` (docs/spec/blitters.md, BLT-002). The
/// `REACTIVE_TUI_BLITTER` environment variable still wins, so a user can
/// correct a terminal the per-terminal table gets wrong.
pub fn set_image_blitter(blitter: Option<Blitter>) {
    let value = blitter
        .and_then(|blitter| Blitter::TIERS.iter().position(|tier| *tier == blitter))
        .map_or(0, |index| index as u8 + 1);
    BLITTER_OVERRIDE.store(value, std::sync::atomic::Ordering::Relaxed);
}

/// The blitter the environment or the application named, if either did,
/// the `REACTIVE_TUI_BLITTER` environment variable first. With one named,
/// `Auto` draws with the renderer's blitters instead of an installed
/// external tool, which would ignore the choice (BLT-002).
pub(crate) fn named_blitter() -> Option<Blitter> {
    let application = match BLITTER_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        value => Blitter::TIERS.get(usize::from(value) - 1).copied(),
    };
    ::suprtui::blit::environment_override()
        .ok()
        .flatten()
        .or(application)
}

/// The blitter image cell fallback draws with: the `REACTIVE_TUI_BLITTER`
/// environment variable, then [`set_image_blitter`], and otherwise by
/// tier: ASCII when the terminal answered that it has no unicode, the
/// per-terminal table's entry for the host's identity, or sextant.
pub fn image_blitter() -> Blitter {
    let application = match BLITTER_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        value => Blitter::TIERS.get(usize::from(value) - 1).copied(),
    };
    let environment = ::suprtui::blit::environment_override().unwrap_or_else(|message| {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| log::warn!("{message}; the blitter is chosen by tier"));
        None
    });
    ::suprtui::blit::choose(
        crate::widgets::display::charts::glyph_support(),
        crate::core::capabilities::TerminalQuery::host_identity().as_deref(),
        application,
        environment,
    )
}

use crate::core::surface::Rgba;
use crate::error::Result;
use std::path::PathBuf;

/// Image widget for displaying images in terminal applications
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    /// Source of the image data
    pub source: ImageSource,
    /// How the image should be displayed
    pub display_mode: ImageDisplayMode,
    /// Optional size constraints (width, height)
    pub size_constraints: Option<(u32, u32)>,
    /// Whether to preserve aspect ratio
    pub preserve_aspect: bool,
    /// Fallback text if image cannot be displayed
    pub fallback_text: Option<String>,
    /// Background color for transparent images
    pub background_color: Option<Rgba>,
    /// Image quality settings
    pub quality: ImageQuality,
}

/// Source of image data
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSource {
    /// Load image from file path
    FilePath(PathBuf),
    /// Image data as base64 encoded string
    Base64Data(String),
    /// Raw image bytes with format information
    RawBytes {
        /// Raw image data bytes
        data: Vec<u8>,
        /// Image width in pixels
        width: u32,
        /// Image height in pixels
        height: u32,
        /// Image format specification
        format: ImageFormat,
    },
    /// URL for future HTTP support
    Url(String),
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// RGB format with 8 bits per channel
    RGB888,
    /// RGBA format with 8 bits per channel
    RGBA8888,
    /// PNG compressed format
    PNG,
    /// JPEG compressed format
    JPEG,
    /// GIF animated format
    GIF,
    /// BMP bitmap format
    BMP,
    /// TIFF format
    TIFF,
}

/// Image display rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageDisplayMode {
    /// Automatically select the best available method
    Auto,
    /// Use sixel graphics protocol (native terminal graphics)
    Sixel,
    /// Use external chafa renderer
    Chafa,
    /// Use external viu renderer
    Viu,
    /// Use Kitty graphics protocol
    KittyGraphics,
    /// Use iTerm2 inline images protocol
    ITerm2Inline,
    /// Convert to ASCII art
    AsciiArt,
    /// Show fallback text only
    Fallback,
}

/// Image rendering quality settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageQuality {
    /// Fast rendering with lower quality
    Fast,
    /// Balanced quality and performance
    Balanced,
    /// High quality rendering (slower)
    High,
}

/// Terminal image rendering capabilities
#[derive(Debug, Clone)]
pub struct ImageCapabilities {
    /// Whether Sixel graphics are supported
    pub sixel: bool,
    /// Whether Kitty graphics protocol is supported
    pub kitty_graphics: bool,
    /// Whether iTerm2 inline images are supported
    pub iterm2_inline: bool,
    /// Whether chafa tool is available for ASCII art
    pub chafa_available: bool,
    /// Whether viu tool is available for image display
    pub viu_available: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            source: ImageSource::FilePath(PathBuf::new()),
            display_mode: ImageDisplayMode::Auto,
            size_constraints: None,
            preserve_aspect: true,
            fallback_text: Some("📷 [Image]".to_string()),
            background_color: None,
            quality: ImageQuality::Balanced,
        }
    }
}

impl Image {
    /// Construct a retained image component for App.
    pub fn into_element(self) -> crate::component::Element {
        self.into_element_with_hint(None)
    }

    pub(crate) fn into_element_with_hint(
        self,
        format: Option<ImageFormat>,
    ) -> crate::component::Element {
        crate::component::Element::typed::<live::LiveImage>(live::LiveProps {
            image: std::sync::Arc::new(self),
            format,
        })
    }

    /// Create a new image widget from a file path
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            source: ImageSource::FilePath(path.into()),
            ..Default::default()
        }
    }

    /// Create a new image widget from base64 data
    pub fn from_base64<S: Into<String>>(data: S) -> Self {
        Self {
            source: ImageSource::Base64Data(data.into()),
            ..Default::default()
        }
    }

    /// Create a new image widget from raw bytes
    pub fn from_raw_bytes(data: Vec<u8>, width: u32, height: u32, format: ImageFormat) -> Self {
        Self {
            source: ImageSource::RawBytes {
                data,
                width,
                height,
                format,
            },
            ..Default::default()
        }
    }

    /// Set the display mode
    pub fn with_display_mode(mut self, mode: ImageDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Set size constraints (max width, max height)
    pub fn with_max_size(mut self, width: u32, height: u32) -> Self {
        self.size_constraints = Some((width, height));
        self
    }

    /// Set whether to preserve aspect ratio
    pub fn with_preserve_aspect(mut self, preserve: bool) -> Self {
        self.preserve_aspect = preserve;
        self
    }

    /// Set fallback text when image cannot be displayed
    pub fn with_fallback_text<S: Into<String>>(mut self, text: S) -> Self {
        self.fallback_text = Some(text.into());
        self
    }

    /// Set background color
    pub fn with_background_color(mut self, color: Rgba) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set rendering quality
    pub fn with_quality(mut self, quality: ImageQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Render the image using the best available method
    pub fn render(&self, capabilities: &ImageCapabilities) -> Result<String> {
        if self.has_empty_size() {
            return Ok(String::new());
        }
        let mode = match self.display_mode {
            ImageDisplayMode::Auto => self.select_best_mode(capabilities),
            mode => mode,
        };

        match mode {
            ImageDisplayMode::Sixel if capabilities.sixel => self.render_sixel(),
            ImageDisplayMode::Chafa if capabilities.chafa_available => self.render_chafa(),
            ImageDisplayMode::Viu if capabilities.viu_available => self.render_viu(),
            ImageDisplayMode::KittyGraphics if capabilities.kitty_graphics => self.render_kitty(),
            ImageDisplayMode::ITerm2Inline if capabilities.iterm2_inline => self.render_iterm2(),
            ImageDisplayMode::AsciiArt => self.render_ascii(),
            ImageDisplayMode::Fallback => self.render_fallback(),
            _ => self.render_ascii(),
        }
    }

    fn has_empty_size(&self) -> bool {
        self.size_constraints
            .is_some_and(|(width, height)| width == 0 || height == 0)
    }

    fn select_best_mode(&self, capabilities: &ImageCapabilities) -> ImageDisplayMode {
        // Priority order based on quality and compatibility
        if capabilities.sixel {
            ImageDisplayMode::Sixel
        } else if capabilities.kitty_graphics {
            ImageDisplayMode::KittyGraphics
        } else if capabilities.iterm2_inline {
            ImageDisplayMode::ITerm2Inline
        } else if capabilities.chafa_available {
            ImageDisplayMode::Chafa
        } else if capabilities.viu_available {
            ImageDisplayMode::Viu
        } else {
            ImageDisplayMode::AsciiArt
        }
    }

    /// Render fallback text when image cannot be displayed
    pub fn render_fallback(&self) -> Result<String> {
        Ok(self
            .fallback_text
            .as_deref()
            .unwrap_or("📷 [Image not supported]")
            .to_string())
    }

    fn render_sixel(&self) -> Result<String> {
        let renderer = SixelRenderer::new();
        renderer.render_image(self)
    }

    fn render_chafa(&self) -> Result<String> {
        let renderer = ExternalRenderer::new();
        renderer.render_with_chafa(self)
    }

    fn render_viu(&self) -> Result<String> {
        let renderer = ExternalRenderer::new();
        renderer.render_with_viu(self)
    }

    fn render_kitty(&self) -> Result<String> {
        let renderer = ProtocolRenderer::new();
        renderer.render_kitty_graphics(self)
    }

    fn render_iterm2(&self) -> Result<String> {
        let renderer = ProtocolRenderer::new();
        renderer.render_iterm2_inline(self)
    }

    fn render_ascii(&self) -> Result<String> {
        let processor = ImageProcessor::new();
        processor.to_ascii_art(self)
    }
}

impl From<Image> for crate::component::Element {
    fn from(image: Image) -> Self {
        image.into_element()
    }
}

impl ImageFormat {
    /// Get the number of bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            ImageFormat::RGB888 => 3,
            ImageFormat::RGBA8888 => 4,
            ImageFormat::PNG
            | ImageFormat::JPEG
            | ImageFormat::GIF
            | ImageFormat::BMP
            | ImageFormat::TIFF => {
                // These are compressed formats, actual bytes per pixel varies
                4 // Assume RGBA when decompressed
            }
        }
    }

    /// Check if this format supports transparency
    pub fn has_alpha(&self) -> bool {
        matches!(
            self,
            ImageFormat::RGBA8888 | ImageFormat::PNG | ImageFormat::GIF
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn blt_002_the_application_override_replaces_the_choice_until_cleared() {
        // The override, the glyph report and the host identity are all
        // process-wide; other tests change them under this lock.
        let _serial = crate::widgets::display::charts::GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if std::env::var_os(::suprtui::blit::BLITTER_ENV).is_some() {
            eprintln!("SKIP: the environment override wins over the application's");
            return;
        }
        for tier in super::Blitter::TIERS {
            super::set_image_blitter(Some(tier));
            assert_eq!(super::image_blitter(), tier);
        }
        super::set_image_blitter(None);
        assert_eq!(
            super::image_blitter(),
            ::suprtui::blit::choose(
                crate::widgets::display::charts::glyph_support(),
                crate::core::capabilities::TerminalQuery::host_identity().as_deref(),
                None,
                None,
            ),
            "clearing returns to the choice by tier"
        );
    }

    use super::*;

    fn capabilities(available: bool) -> ImageCapabilities {
        ImageCapabilities {
            sixel: available,
            kitty_graphics: available,
            iterm2_inline: available,
            chafa_available: available,
            viu_available: available,
        }
    }

    #[test]
    fn api_image_unavailable_protocols_fall_back_to_decoded_image() {
        let image = Image::from_raw_bytes(vec![0; 3], 1, 1, ImageFormat::RGB888)
            .with_max_size(2, 1)
            .with_preserve_aspect(false)
            .with_quality(ImageQuality::Fast);
        for mode in [
            ImageDisplayMode::Auto,
            ImageDisplayMode::Sixel,
            ImageDisplayMode::KittyGraphics,
            ImageDisplayMode::ITerm2Inline,
            ImageDisplayMode::Chafa,
            ImageDisplayMode::Viu,
        ] {
            assert_eq!(
                image
                    .clone()
                    .with_display_mode(mode)
                    .render(&capabilities(false))
                    .unwrap(),
                "@@\n",
                "{mode:?}"
            );
        }
        assert_eq!(
            image
                .with_display_mode(ImageDisplayMode::Fallback)
                .with_fallback_text("No image")
                .render(&capabilities(false))
                .unwrap(),
            "No image"
        );
    }

    #[test]
    fn api_image_empty_bounds_emit_no_protocol_or_placeholder() {
        let image =
            Image::from_raw_bytes(vec![0; 3], 1, 1, ImageFormat::RGB888).with_max_size(0, 1);
        for mode in [
            ImageDisplayMode::Auto,
            ImageDisplayMode::Sixel,
            ImageDisplayMode::KittyGraphics,
            ImageDisplayMode::ITerm2Inline,
            ImageDisplayMode::Chafa,
            ImageDisplayMode::Viu,
            ImageDisplayMode::AsciiArt,
            ImageDisplayMode::Fallback,
        ] {
            assert_eq!(
                image
                    .clone()
                    .with_display_mode(mode)
                    .render(&capabilities(true))
                    .unwrap(),
                "",
                "{mode:?}"
            );
        }
    }
}
