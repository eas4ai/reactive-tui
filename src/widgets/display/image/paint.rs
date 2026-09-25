//! Immutable decoded image content carried through the ordinary paint tree.

use super::{Image, ImageDisplayMode, ImageQuality};
use crate::core::surface::Rgba;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageProtocol {
    Kitty,
    Sixel,
    Inline,
}

#[derive(Clone)]
pub(crate) struct ImagePaint {
    pub id: u32,
    pub pixels: Arc<image::RgbaImage>,
    pub mode: ImageDisplayMode,
    pub preserve_aspect: bool,
    pub background: Option<Rgba>,
    pub quality: ImageQuality,
    pub max_size: Option<(u32, u32)>,
}
impl ImagePaint {
    pub fn new(id: u32, pixels: Arc<image::RgbaImage>, config: &Image) -> Self {
        Self {
            id,
            pixels,
            mode: config.display_mode,
            preserve_aspect: config.preserve_aspect,
            background: config.background_color,
            quality: config.quality,
            max_size: config.size_constraints,
        }
    }
}
impl PartialEq for ImagePaint {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && Arc::ptr_eq(&self.pixels, &other.pixels)
            && self.mode == other.mode
            && self.preserve_aspect == other.preserve_aspect
            && self.background == other.background
            && self.quality == other.quality
            && self.max_size == other.max_size
    }
}
