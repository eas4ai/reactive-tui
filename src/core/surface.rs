//! Terminal surface buffer for rendering
//!
//! This module provides the core surface abstraction for terminal rendering,
//! including cell-based buffers, text attributes, colors, and Unicode handling.

use super::geometry::{Point, Rect, Size};
use crate::layout::css::gradients::{Gradient, GradientBorder};
use std::env;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

mod image_output;

// === Unicode Handling Utilities ===

/// Terminal emoji handling detection
use std::sync::OnceLock;

static HANDLES_VS16_INCORRECTLY: OnceLock<bool> = OnceLock::new();

/// Some terminals incorrectly only advance the cursor one space for emoji with VS16
/// This detects those terminals and compensates with additional whitespace
///
/// Based on: https://www.jeffquast.com/post/ucs-detect-test-results/
/// and: https://darrenburns.net/posts/emoji-in-the-terminal/
pub(crate) fn handles_vs16_incorrectly() -> bool {
    *HANDLES_VS16_INCORRECTLY.get_or_init(|| {
        env::var("TERM_PROGRAM")
            .map(|s| s == "Apple_Terminal")
            .unwrap_or(false)
            || env::var("GNOME_TERMINAL_SCREEN").is_ok_and(|v| !v.is_empty())
    })
}

/// Calculate the required padding for emoji with VS16 in problematic terminals
pub(crate) fn emoji_padding_required(text: &str) -> usize {
    if text.contains('\u{fe0f}') && handles_vs16_incorrectly() {
        text.width().saturating_sub(1)
    } else {
        0
    }
}

/// Calculate the display width of text, accounting for grapheme clusters
pub fn text_display_width(text: &str) -> usize {
    text.graphemes(true).map(UnicodeWidthStr::width).sum()
}

/// Split text into grapheme clusters for proper Unicode handling
pub fn text_to_graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

// === Border Characters ===

/// Characters used for drawing borders
///
/// Provides different border styles including ASCII, rounded, and double-line borders.
#[derive(Clone, Copy, Debug)]
pub struct BorderChars {
    /// Top-left corner character
    pub top_left: char,
    /// Top-right corner character
    pub top_right: char,
    /// Bottom-left corner character
    pub bottom_left: char,
    /// Bottom-right corner character
    pub bottom_right: char,
    /// Horizontal line character
    pub horizontal: char,
    /// Vertical line character
    pub vertical: char,
}

impl Default for BorderChars {
    fn default() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }
}

impl BorderChars {
    /// ASCII border characters for compatibility
    pub fn ascii() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
        }
    }

    /// Rounded border characters
    pub fn rounded() -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
        }
    }

    /// Double-line border characters
    pub fn double() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
        }
    }

    /// Thick border characters
    pub fn thick() -> Self {
        Self {
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            horizontal: '━',
            vertical: '┃',
        }
    }
}

// === Enhanced Text Style ===

/// Enhanced text style for Canvas-like functionality
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextStyle {
    /// Foreground color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
    /// Whether to use emoji-aware rendering
    pub emoji_aware: bool,
}

impl TextStyle {
    /// Create a new text style
    pub fn new(fg: Rgba, bg: Rgba, attr: Attr) -> Self {
        Self {
            fg,
            bg,
            attr,
            emoji_aware: true,
        }
    }

    /// Create a simple text style with just foreground color
    pub fn fg(fg: Rgba) -> Self {
        Self {
            fg,
            bg: Rgba::transparent(),
            attr: Attr::empty(),
            emoji_aware: true,
        }
    }

    /// Create a text style with foreground and background
    pub fn fg_bg(fg: Rgba, bg: Rgba) -> Self {
        Self {
            fg,
            bg,
            attr: Attr::empty(),
            emoji_aware: true,
        }
    }

    /// Add bold attribute
    pub fn bold(mut self) -> Self {
        self.attr |= Attr::BOLD;
        self
    }

    /// Add italic attribute
    pub fn italic(mut self) -> Self {
        self.attr |= Attr::ITALIC;
        self
    }

    /// Add underline attribute
    pub fn underline(mut self) -> Self {
        self.attr |= Attr::UNDERLINE;
        self
    }

    /// Disable emoji-aware rendering
    pub fn no_emoji_handling(mut self) -> Self {
        self.emoji_aware = false;
        self
    }
}

// === Surface Subview for Clipped Drawing ===

/// A clipped view into a surface for Canvas-like drawing operations
pub struct SurfaceSubview<'a> {
    surface: &'a mut Surface,
    /// Offset from surface origin
    offset_x: isize,
    offset_y: isize,
    /// Clipping rectangle in surface coordinates
    clip_rect: Rect,
}

impl<'a> SurfaceSubview<'a> {
    /// Create a new subview with clipping
    pub(crate) fn new(
        surface: &'a mut Surface,
        offset_x: isize,
        offset_y: isize,
        clip_rect: Rect,
    ) -> Self {
        Self {
            surface,
            offset_x,
            offset_y,
            clip_rect,
        }
    }

    /// Get the clipped bounds of this subview
    pub fn bounds(&self) -> Rect {
        self.clip_rect
    }

    /// Write text with style and clipping
    pub fn write_text(&mut self, x: isize, y: isize, text: &str, style: TextStyle) {
        let abs_x = self.offset_x + x;
        let abs_y = self.offset_y + y;

        // Check if position is within clipping bounds
        if abs_x < self.clip_rect.left() as isize
            || abs_y < self.clip_rect.top() as isize
            || abs_y >= self.clip_rect.bottom() as isize
        {
            return;
        }

        // Calculate how much text fits within the clipping bounds
        let max_width = (self.clip_rect.right() as isize - abs_x).max(0) as usize;

        if max_width == 0 {
            return;
        }

        // Use the enhanced text writing method
        self.surface
            .write_text_enhanced(abs_x as usize, abs_y as usize, text, max_width, style);
    }

    /// Fill a rectangle with background color
    pub fn fill_background(
        &mut self,
        x: isize,
        y: isize,
        width: usize,
        height: usize,
        color: Rgba,
    ) {
        let abs_x = self.offset_x + x;
        let abs_y = self.offset_y + y;

        // Calculate intersection with clipping bounds
        let left = abs_x.max(self.clip_rect.left() as isize) as usize;
        let top = abs_y.max(self.clip_rect.top() as isize) as usize;
        let right = (abs_x + width as isize).min(self.clip_rect.right() as isize) as usize;
        let bottom = (abs_y + height as isize).min(self.clip_rect.bottom() as isize) as usize;

        if left >= right || top >= bottom {
            return;
        }

        let fill_rect = Rect::from_coords(left, top, right - left, bottom - top);
        self.surface.fill_background_rect(fill_rect, color);
    }

    /// Clear text in a rectangular area
    pub fn clear_text(&mut self, x: isize, y: isize, width: usize, height: usize) {
        self.fill_background(x, y, width, height, Rgba::transparent());

        // Also clear the character content
        let abs_x = self.offset_x + x;
        let abs_y = self.offset_y + y;

        let left = abs_x.max(self.clip_rect.left() as isize) as usize;
        let top = abs_y.max(self.clip_rect.top() as isize) as usize;
        let right = (abs_x + width as isize).min(self.clip_rect.right() as isize) as usize;
        let bottom = (abs_y + height as isize).min(self.clip_rect.bottom() as isize) as usize;

        for y in top..bottom {
            for x in left..right {
                if x < self.surface.w && y < self.surface.h {
                    let idx = self.surface.idx(x, y);
                    self.surface.clear_grapheme_at(idx);
                    self.surface.buf[idx].ch = ' ';
                }
            }
        }
    }
}

/// RGBA color representation with floating-point components
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rgba {
    /// Red component (0.0 to 1.0)
    pub r: f32,
    /// Green component (0.0 to 1.0)
    pub g: f32,
    /// Blue component (0.0 to 1.0)
    pub b: f32,
    /// Alpha (transparency) component (0.0 to 1.0)
    pub a: f32,
}

impl Rgba {
    /// Default epsilon for color comparisons (1/255 for 8-bit precision)
    pub const DEFAULT_EPSILON: f32 = 1.0 / 255.0;

    /// Create a new RGBA color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a transparent color
    pub fn transparent() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    /// Create white color
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Create black color
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Compare colors with epsilon tolerance for floating-point precision
    pub fn equals_epsilon(self, other: Self, epsilon: f32) -> bool {
        (self.r - other.r).abs() < epsilon
            && (self.g - other.g).abs() < epsilon
            && (self.b - other.b).abs() < epsilon
            && (self.a - other.a).abs() < epsilon
    }

    /// Compare colors with default epsilon
    pub fn approx_eq(self, other: Self) -> bool {
        self.equals_epsilon(other, Self::DEFAULT_EPSILON)
    }

    /// Blend two colors using alpha blending
    pub fn blend(self, other: Self, alpha: f32) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * alpha,
            g: self.g + (other.g - self.g) * alpha,
            b: self.b + (other.b - self.b) * alpha,
            a: self.a + (other.a - self.a) * alpha,
        }
    }

    /// Multiply color by alpha (for transparency effects)
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a * alpha.clamp(0.0, 1.0),
        }
    }

    /// Premultiply alpha (for efficient blending)
    pub fn premultiply_alpha(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Convert to linear color space (gamma correction)
    pub fn to_linear(self) -> Self {
        fn srgb_to_linear(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        Self {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    /// Convert from linear to sRGB color space
    pub fn to_srgb(self) -> Self {
        fn linear_to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }

        Self {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }

    /// Get luminance (perceived brightness) using Rec. 709 coefficients
    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Calculate contrast ratio between two colors (WCAG standard)
    pub fn contrast_ratio(self, other: Self) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }
}

// SIMD-optimized color operations (when portable_simd is available)
#[cfg(feature = "simd")]
impl Rgba {
    /// SIMD-optimized epsilon comparison
    pub fn equals_epsilon_simd(self, other: Self, epsilon: f32) -> bool {
        use std::simd::{f32x4, prelude::SimdFloat, prelude::SimdPartialOrd};

        let a = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let b = f32x4::from_array([other.r, other.g, other.b, other.a]);
        let diff = (a - b).abs();
        let eps = f32x4::splat(epsilon);

        diff.simd_lt(eps).all()
    }

    /// SIMD-optimized color blending
    pub fn blend_simd(self, other: Self, alpha: f32) -> Self {
        use std::simd::f32x4;

        let alpha = alpha.clamp(0.0, 1.0);
        let a = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let b = f32x4::from_array([other.r, other.g, other.b, other.a]);
        let alpha_vec = f32x4::splat(alpha);

        // result = a + (b - a) * alpha
        let result = a + (b - a) * alpha_vec;
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// SIMD-optimized alpha multiplication
    pub fn with_alpha_simd(self, alpha: f32) -> Self {
        use std::simd::f32x4;

        let alpha = alpha.clamp(0.0, 1.0);
        let color = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let alpha_vec = f32x4::from_array([1.0, 1.0, 1.0, alpha]);
        let result = color * alpha_vec;
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// SIMD-optimized premultiply alpha
    pub fn premultiply_alpha_simd(self) -> Self {
        use std::simd::f32x4;

        let color = f32x4::from_array([self.r, self.g, self.b, self.a]);
        let alpha_vec = f32x4::splat(self.a);
        let alpha_mask = f32x4::from_array([1.0, 1.0, 1.0, 0.0]); // Don't multiply alpha by itself
        let result = color * (alpha_vec * alpha_mask + f32x4::from_array([0.0, 0.0, 0.0, 1.0]));
        let arr = result.to_array();

        Self::new(arr[0], arr[1], arr[2], self.a)
    }
}

// Fallback implementations that use SIMD when available
impl Rgba {
    /// Optimized epsilon comparison (uses SIMD when available)
    #[inline]
    pub fn equals_epsilon_fast(self, other: Self, epsilon: f32) -> bool {
        #[cfg(feature = "simd")]
        {
            self.equals_epsilon_simd(other, epsilon)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.equals_epsilon(other, epsilon)
        }
    }

    /// Optimized color blending (uses SIMD when available)
    #[inline]
    pub fn blend_fast(self, other: Self, alpha: f32) -> Self {
        #[cfg(feature = "simd")]
        {
            self.blend_simd(other, alpha)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.blend(other, alpha)
        }
    }

    /// Optimized alpha multiplication (uses SIMD when available)
    #[inline]
    pub fn with_alpha_fast(self, alpha: f32) -> Self {
        #[cfg(feature = "simd")]
        {
            self.with_alpha_simd(alpha)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.with_alpha(alpha)
        }
    }
}

use crate::error::{ReactiveError, Result};
use bitflags::bitflags;
use std::collections::HashMap;
bitflags! {
    /// Text attributes for terminal cells
    ///
    /// Bitflags representing various text formatting options that can be
    /// combined together for rich text display in the terminal.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Attr: u8 {
        /// Bold text formatting
        const BOLD=1<<0;
        /// Italic text formatting
        const ITALIC=1<<1;
        /// Underlined text
        const UNDERLINE=1<<2;
        /// Reverse video (swap foreground/background colors)
        const REVERSE=1<<3;
        /// Strikethrough text
        const STRIKE=1<<4;
    }
}

impl Attr {
    /// Create text attributes from individual boolean flags
    ///
    /// # Arguments
    /// * `bold` - Enable bold text
    /// * `italic` - Enable italic text
    /// * `underline` - Enable underlined text
    /// * `reverse` - Enable reverse video
    /// * `strike` - Enable strikethrough
    ///
    /// # Returns
    /// Combined `Attr` flags
    #[inline]
    pub fn from_flags(
        bold: bool,
        italic: bool,
        underline: bool,
        reverse: bool,
        strike: bool,
    ) -> Self {
        let mut a = Attr::empty();
        if bold {
            a |= Attr::BOLD;
        }
        if italic {
            a |= Attr::ITALIC;
        }
        if underline {
            a |= Attr::UNDERLINE;
        }
        if reverse {
            a |= Attr::REVERSE;
        }
        if strike {
            a |= Attr::STRIKE;
        }
        a
    }
}

/// Image placement information for a cell
///
/// Defines how an image should be displayed within a terminal cell,
/// including the source region and display properties.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageCellPlacement {
    /// Source X coordinate in the image (pixels)
    pub source_x: u16,
    /// Source Y coordinate in the image (pixels)
    pub source_y: u16,
    /// Width of the image region to display (pixels); zero uses the remaining width.
    pub source_width: u16,
    /// Height of the image region to display (pixels); zero uses the remaining height.
    pub source_height: u16,
    /// Z-index for layering (negative = behind text, positive = in front)
    pub z_index: i8,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: f32,
}

impl Default for ImageCellPlacement {
    fn default() -> Self {
        Self {
            source_x: 0,
            source_y: 0,
            source_width: 0,
            source_height: 0,
            z_index: -1, // Behind text by default
            opacity: 1.0,
        }
    }
}

/// A single cell in the terminal surface
///
/// Represents one character position in the terminal with its
/// associated styling, colors, and optional image content.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Cell {
    /// The character to display
    pub ch: char,
    /// Foreground color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
    /// Optional reference to an image in the image registry
    pub image_id: Option<u32>,
    /// Image placement information if image_id is Some
    pub image_placement: Option<ImageCellPlacement>,
}

impl Cell {
    /// Create a new cell with image placement
    pub fn with_image(image_id: u32, placement: ImageCellPlacement) -> Self {
        Self {
            image_id: Some(image_id),
            image_placement: Some(placement),
            ..Default::default()
        }
    }

    /// Set image placement for this cell
    pub fn set_image(&mut self, image_id: u32, placement: ImageCellPlacement) {
        self.image_id = Some(image_id);
        self.image_placement = Some(placement);
    }

    /// Clear image placement from this cell
    pub fn clear_image(&mut self) {
        self.image_id = None;
        self.image_placement = None;
    }

    /// Check if this cell has an image
    pub fn has_image(&self) -> bool {
        self.image_id.is_some()
    }
}

/// Image data stored in the registry
#[derive(Debug, Clone)]
/// Image data for rendering in the terminal
pub struct ImageData {
    /// Image dimensions in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Raw RGBA pixel data
    pub pixels: Vec<u8>,
    /// Optional metadata
    pub metadata: ImageMetadata,
}

/// Image metadata
#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    /// Original file path or source
    pub source: Option<String>,
    /// Image format
    pub format: Option<String>,
    /// Creation timestamp
    pub created_at: Option<std::time::SystemTime>,
}

/// Registry for managing images referenced by cells
#[derive(Debug)]
pub struct ImageRegistry {
    images: HashMap<u32, ImageData>,
    next_id: u32,
}

impl Default for ImageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageRegistry {
    /// Create a new image registry
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new image. Returns the reserved ID zero for invalid data or
    /// exhausted IDs. Use `try_register_image` to obtain the error.
    pub fn register_image(&mut self, image: ImageData) -> u32 {
        self.try_register_image(image).unwrap_or(0)
    }

    /// Validate raw RGBA data before retaining it. IDs are never reused.
    pub fn try_register_image(&mut self, image: ImageData) -> Result<u32> {
        let expected =
            crate::widgets::display::image::decoded::dimensions(image.width, image.height)?;
        if image.pixels.len() != expected {
            return Err(ReactiveError::invalid_parameter(format!(
                "Surface image needs {expected} RGBA bytes, got {}",
                image.pixels.len()
            )));
        }
        let id = self.next_id;
        if id == 0 {
            return Err(ReactiveError::resource("Surface image IDs exhausted"));
        }
        self.next_id = id.checked_add(1).unwrap_or(0);
        self.images.insert(id, image);
        Ok(id)
    }

    /// Get image data by ID
    pub fn get_image(&self, id: u32) -> Option<&ImageData> {
        self.images.get(&id)
    }

    /// Remove an image from the registry
    pub fn remove_image(&mut self, id: u32) -> Option<ImageData> {
        self.images.remove(&id)
    }

    /// Get all registered image IDs
    pub fn image_ids(&self) -> Vec<u32> {
        self.images.keys().copied().collect()
    }

    /// Clear all images
    pub fn clear(&mut self) {
        self.images.clear();
    }

    /// Get the number of registered images
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

/// Terminal rendering surface
///
/// A 2D grid of cells that represents the terminal display buffer.
/// Each cell contains a character, colors, attributes, and optional image data.
pub struct Surface {
    /// Width in terminal cells
    w: usize,
    /// Height in terminal cells
    h: usize,
    /// Buffer of cells (row-major order)
    buf: Vec<Cell>,
    /// Image registry for managing cell-referenced images
    image_registry: ImageRegistry,
    /// Complete multi-scalar or wide graphemes, keyed by their leading cell.
    graphemes: std::collections::BTreeMap<usize, (String, usize)>,
}

impl Surface {
    /// Create a new surface with the given dimensions
    ///
    /// # Arguments
    /// * `w` - Width in terminal cells
    /// * `h` - Height in terminal cells
    ///
    /// # Returns
    /// A new `Surface` initialized with default cells
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            buf: vec![Cell::default(); w * h],
            image_registry: ImageRegistry::new(),
            graphemes: Default::default(),
        }
    }

    /// A `w` by `h` surface holding `cells` in row-major order, written once
    /// rather than defaulted and then set. The caller stores U+FFFD for a
    /// control character, as [`Surface::set`] does. Panics when the cell
    /// count is not `w * h`.
    pub(crate) fn from_cells(w: usize, h: usize, cells: Vec<Cell>) -> Self {
        assert_eq!(cells.len(), w * h, "a surface holds exactly w * h cells");
        debug_assert!(cells.iter().all(|cell| !cell.ch.is_control()));
        Self {
            w,
            h,
            buf: cells,
            image_registry: ImageRegistry::new(),
            graphemes: Default::default(),
        }
    }

    /// The surface's cells, row-major, so their allocation can hold the
    /// next frame.
    pub(crate) fn into_cells(self) -> Vec<Cell> {
        self.buf
    }

    /// Create a new surface with validation
    pub fn new_validated(w: usize, h: usize) -> Result<Self> {
        if w == 0 || h == 0 {
            return Err(ReactiveError::invalid_parameter(
                "Surface dimensions must be greater than 0",
            ));
        }
        if w > 10000 || h > 10000 {
            return Err(ReactiveError::invalid_parameter(
                "Surface dimensions too large (max 10000x10000)",
            ));
        }

        // Check for multiplication overflow
        if let Some(buffer_size) = w.checked_mul(h) {
            // Additional safety check for reasonable memory usage (100MB limit for cells)
            const MAX_CELLS: usize = 100_000_000 / std::mem::size_of::<Cell>();
            if buffer_size > MAX_CELLS {
                return Err(ReactiveError::invalid_parameter(format!(
                    "Surface buffer size {} exceeds maximum allowed {}",
                    buffer_size, MAX_CELLS
                )));
            }
        } else {
            return Err(ReactiveError::invalid_parameter(format!(
                "Surface dimensions {}x{} would cause integer overflow",
                w, h
            )));
        }

        Ok(Self::new(w, h))
    }
    /// Calculate buffer index from coordinates
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        // Saturating operations prevent overflow panics
        y.saturating_mul(self.w).saturating_add(x)
    }
    /// Get the dimensions of the surface
    ///
    /// # Returns
    /// Tuple of (width, height) in terminal cells
    pub fn dims(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    /// Reinitialize the surface to the given dimensions, reusing allocation when possible.
    pub fn reinit(&mut self, w: usize, h: usize) {
        self.graphemes.clear();
        self.w = w;
        self.h = h;
        let needed = w * h;
        if self.buf.capacity() >= needed {
            self.buf.resize(needed, Cell::default());
        } else {
            self.buf = vec![Cell::default(); needed];
        }
        // Note: Image registry is preserved across reinit
    }
    /// Clear the surface with the given background color
    ///
    /// # Arguments
    /// * `bg` - Background color to fill the surface with
    pub fn clear(&mut self, bg: Rgba) {
        self.graphemes.clear();
        for c in &mut self.buf {
            *c = Cell {
                ch: ' ',
                fg: Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                bg,
                attr: Attr::empty(),
                image_id: None,
                image_placement: None,
            };
        }
    }
    /// Set a cell at the specified coordinates.
    ///
    /// Host control characters are stored as the Unicode replacement character.
    pub fn set(&mut self, x: usize, y: usize, mut cell: Cell) {
        if x < self.w && y < self.h {
            if cell.ch.is_control() {
                cell.ch = '\u{fffd}';
            }
            let i = self.idx(x, y);
            self.clear_grapheme_at(i);
            self.buf[i] = cell;
        }
    }
    /// Get a cell at the specified coordinates
    pub fn get(&self, x: usize, y: usize) -> Cell {
        if x >= self.w || y >= self.h {
            return Cell::default();
        }
        self.buf[self.idx(x, y)]
    }

    /// Store one complete printable grapheme without changing the legacy Cell layout.
    /// Returns false for multiple graphemes, control text, zero width or clipping.
    /// `get` retains the first scalar; `grapheme` and DiffWriter retain full text.
    pub fn set_grapheme(&mut self, x: usize, y: usize, text: &str, mut cell: Cell) -> bool {
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;
        let width = text.width();
        if y >= self.h
            || width == 0
            || x >= self.w
            || width > self.w - x
            || text.graphemes(true).count() != 1
            || text.chars().any(char::is_control)
        {
            return false;
        }
        let start = self.idx(x, y);
        for index in start..start + width {
            self.clear_grapheme_at(index);
        }
        cell.ch = text.chars().next().expect("nonempty grapheme");
        self.buf[start] = cell;
        for index in start + 1..start + width {
            self.buf[index] = Cell { ch: ' ', ..cell };
        }
        if width > 1 || text.chars().count() > 1 {
            self.graphemes.insert(start, (text.to_owned(), width));
        }
        true
    }

    /// Complete text for a leading cell, empty text for a continuation cell.
    pub fn grapheme(&self, x: usize, y: usize) -> std::borrow::Cow<'_, str> {
        if x >= self.w || y >= self.h {
            return std::borrow::Cow::Borrowed("");
        }
        let index = self.idx(x, y);
        if let Some((text, _)) = self.graphemes.get(&index) {
            std::borrow::Cow::Borrowed(text)
        } else if self.grapheme_continuation(index) {
            std::borrow::Cow::Borrowed("")
        } else {
            std::borrow::Cow::Owned(self.buf[index].ch.to_string())
        }
    }

    fn grapheme_continuation(&self, index: usize) -> bool {
        self.graphemes
            .range(..index)
            .next_back()
            .is_some_and(|(&start, (_, width))| index < start + width)
    }

    fn cell_text_matches(&self, other: &Self, index: usize) -> bool {
        self.graphemes.get(&index) == other.graphemes.get(&index)
            && !self.grapheme_continuation(index)
    }

    fn append_cell_text(&self, index: usize, output: &mut String) {
        if let Some((text, _)) = self.graphemes.get(&index) {
            output.push_str(text);
        } else {
            let ch = self.buf[index].ch;
            output.push(ch);
            if unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) == 2 {
                output.push(' ');
            }
        }
    }

    fn clear_grapheme_at(&mut self, index: usize) {
        let covered = self
            .graphemes
            .range(..=index)
            .next_back()
            .filter(|(start, (_, width))| index < **start + width)
            .map(|(&start, (_, width))| (start, *width));
        if let Some((start, width)) = covered {
            self.graphemes.remove(&start);
            for cell in &mut self.buf[start..start + width] {
                cell.ch = ' ';
            }
        }
    }

    /// Write a string at the specified position with styling
    pub fn write_str(&mut self, mut x: usize, y: usize, s: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        if y >= self.h {
            return;
        }
        for ch in s.chars() {
            if x >= self.w {
                break;
            }
            let cell = Cell {
                ch,
                fg,
                bg,
                attr,
                image_id: None,
                image_placement: None,
            };
            self.set(x, y, cell);
            x += 1;
            if unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) == 2 && x < self.w {
                self.set(
                    x,
                    y,
                    Cell {
                        ch: ' ',
                        fg,
                        bg,
                        attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
                x += 1;
            }
        }
    }

    /// Write a string at the given point
    pub fn write_str_at(&mut self, point: Point, s: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        self.write_str(point.x, point.y, s, fg, bg, attr)
    }

    /// Set a cell at the given point
    pub fn set_at(&mut self, point: Point, cell: Cell) {
        if point.x < self.w && point.y < self.h {
            self.set(point.x, point.y, cell);
        }
    }

    /// Get a cell at the given point
    pub fn get_at(&self, point: Point) -> Cell {
        self.get(point.x, point.y)
    }

    /// Fill a rectangle with a character
    pub fn fill_rect(&mut self, rect: Rect, ch: char, fg: Rgba, bg: Rgba, attr: Attr) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set(
                    x,
                    y,
                    Cell {
                        ch,
                        fg,
                        bg,
                        attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
            }
        }
    }

    /// Get the size of this surface
    pub fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }

    /// Get the bounds of this surface as a Rect
    pub fn bounds(&self) -> Rect {
        Rect::from_coords(0, 0, self.w, self.h)
    }

    /// Write a string with width clipping
    #[allow(clippy::too_many_arguments)]
    pub fn write_str_clipped(
        &mut self,
        x: usize,
        y: usize,
        s: &str,
        max_width: usize,
        fg: Rgba,
        bg: Rgba,
        attr: Attr,
    ) {
        if y >= self.h {
            return;
        }

        // If max_width is 0, use surface width as limit
        let limit = if max_width == 0 {
            self.w.saturating_sub(x)
        } else {
            max_width
        };
        let mut used = 0usize;

        for ch in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            if used + char_width > limit || x + used >= self.w {
                break;
            }

            let cell = Cell {
                ch,
                fg,
                bg,
                attr,
                image_id: None,
                image_placement: None,
            };
            self.set(x + used, y, cell);
            used += char_width;

            // For wide characters, add continuation cell
            if char_width == 2 && x + used < self.w {
                self.set(
                    x + used,
                    y,
                    Cell {
                        ch: ' ',
                        fg,
                        bg,
                        attr,
                        image_id: None,
                        image_placement: None,
                    },
                );
                used += 1;
            }
        }
    }

    /// Write text with a PaintStyle
    pub fn write_text_styled(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        style: &crate::ui::paint::PaintStyle,
    ) {
        self.write_str(x, y, text, style.fg, style.bg, style.attr);
    }

    /// Write clipped text with a PaintStyle
    pub fn write_text_styled_clipped(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        max_width: usize,
        style: &crate::ui::paint::PaintStyle,
    ) {
        self.write_str_clipped(x, y, text, max_width, style.fg, style.bg, style.attr);
    }

    // === Enhanced Canvas-like Methods ===

    /// Enhanced text writing with Unicode awareness and emoji handling
    pub fn write_text_enhanced(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        max_width: usize,
        style: TextStyle,
    ) {
        if y >= self.h {
            return;
        }

        let limit = if max_width == 0 {
            self.w.saturating_sub(x)
        } else {
            max_width
        };

        let mut used_width = 0;
        let mut current_x = x;

        // Handle emoji padding if needed
        let emoji_padding = if style.emoji_aware {
            emoji_padding_required(text)
        } else {
            0
        };

        // Process text as grapheme clusters for proper Unicode handling
        for grapheme in text.graphemes(true) {
            let grapheme_width = UnicodeWidthStr::width(grapheme);

            // Check if we have space for this grapheme
            if used_width + grapheme_width > limit || current_x >= self.w {
                break;
            }

            // Write the grapheme
            if let Some(ch) = grapheme.chars().next() {
                let cell = Cell {
                    ch,
                    fg: style.fg,
                    bg: style.bg,
                    attr: style.attr,
                    image_id: None,
                    image_placement: None,
                };
                self.set(current_x, y, cell);
            }

            current_x += grapheme_width;
            used_width += grapheme_width;

            // Add emoji padding if needed
            if emoji_padding > 0 && grapheme.contains('\u{fe0f}') {
                for _ in 0..emoji_padding {
                    if current_x < self.w && used_width < limit {
                        let padding_cell = Cell {
                            ch: ' ',
                            fg: style.fg,
                            bg: style.bg,
                            attr: style.attr,
                            image_id: None,
                            image_placement: None,
                        };
                        self.set(current_x, y, padding_cell);
                        current_x += 1;
                        used_width += 1;
                    }
                }
            }
        }
    }

    /// Fill a rectangle with background color only
    pub fn fill_background_rect(&mut self, rect: Rect, color: Rgba) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                if x < self.w && y < self.h {
                    let idx = self.idx(x, y);
                    self.buf[idx].bg = color;
                }
            }
        }
    }

    /// Fill a rectangle with a gradient background
    pub fn fill_gradient_rect(&mut self, rect: Rect, gradient: &Gradient) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        let width = rect.width();
        let height = rect.height();

        // Generate gradient colors based on direction
        use crate::layout::css::gradients::GradientDirection;

        match gradient.direction {
            GradientDirection::ToRight | GradientDirection::ToLeft => {
                // Horizontal gradient
                let colors = gradient.render(width);
                let colors = if matches!(gradient.direction, GradientDirection::ToLeft) {
                    colors.into_iter().rev().collect()
                } else {
                    colors
                };

                for y in rect.top()..rect.bottom() {
                    for (i, x) in (rect.left()..rect.right()).enumerate() {
                        if x < self.w && y < self.h && i < colors.len() {
                            let idx = self.idx(x, y);
                            let color = colors[i];
                            self.buf[idx].bg = Rgba::new(
                                color.0 as f32 / 255.0,
                                color.1 as f32 / 255.0,
                                color.2 as f32 / 255.0,
                                color.3,
                            );
                        }
                    }
                }
            }
            GradientDirection::ToBottom | GradientDirection::ToTop => {
                // Vertical gradient
                let colors = gradient.render(height);
                let colors = if matches!(gradient.direction, GradientDirection::ToTop) {
                    colors.into_iter().rev().collect()
                } else {
                    colors
                };

                for (i, y) in (rect.top()..rect.bottom()).enumerate() {
                    for x in rect.left()..rect.right() {
                        if x < self.w && y < self.h && i < colors.len() {
                            let idx = self.idx(x, y);
                            let color = colors[i];
                            self.buf[idx].bg = Rgba::new(
                                color.0 as f32 / 255.0,
                                color.1 as f32 / 255.0,
                                color.2 as f32 / 255.0,
                                color.3,
                            );
                        }
                    }
                }
            }
            _ => {
                // Diagonal gradients - interpolate based on position
                for y in rect.top()..rect.bottom() {
                    for x in rect.left()..rect.right() {
                        if x < self.w && y < self.h {
                            let rel_x = (x - rect.left()) as f32 / width as f32;
                            let rel_y = (y - rect.top()) as f32 / height as f32;

                            let pos = match gradient.direction {
                                GradientDirection::ToTopRight => (rel_x + (1.0 - rel_y)) / 2.0,
                                GradientDirection::ToTopLeft => {
                                    ((1.0 - rel_x) + (1.0 - rel_y)) / 2.0
                                }
                                GradientDirection::ToBottomRight => (rel_x + rel_y) / 2.0,
                                GradientDirection::ToBottomLeft => ((1.0 - rel_x) + rel_y) / 2.0,
                                _ => 0.5,
                            };

                            if let Some(color) = gradient.color_at(pos) {
                                let idx = self.idx(x, y);
                                self.buf[idx].bg = Rgba::new(
                                    color.0 as f32 / 255.0,
                                    color.1 as f32 / 255.0,
                                    color.2 as f32 / 255.0,
                                    color.3,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw a gradient border around a rectangle
    pub fn draw_gradient_border(&mut self, rect: Rect, gradient_border: &GradientBorder) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        let width = rect.width();
        let height = rect.height();

        if width < 2 || height < 2 {
            return; // Too small for a border
        }

        // Get border colors for all sides
        let border_colors = gradient_border.render_border(width, height);

        if border_colors.len() >= 4 {
            let top_colors = &border_colors[0];
            let right_colors = &border_colors[1];
            let bottom_colors = &border_colors[2];
            let left_colors = &border_colors[3];

            // Draw top border
            for (i, x) in (rect.left()..rect.right()).enumerate() {
                if x < self.w && i < top_colors.len() {
                    let idx = self.idx(x, rect.top());
                    let color = top_colors[i];
                    self.buf[idx].bg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                    self.buf[idx].ch = '─';
                }
            }

            // Draw bottom border
            for (i, x) in (rect.left()..rect.right()).enumerate() {
                if x < self.w && rect.bottom() > 0 && i < bottom_colors.len() {
                    let idx = self.idx(x, rect.bottom() - 1);
                    let color = bottom_colors[i];
                    self.buf[idx].bg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                    self.buf[idx].ch = '─';
                }
            }

            // Draw left border
            for (i, y) in (rect.top()..rect.bottom()).enumerate() {
                if y < self.h && i < left_colors.len() {
                    let idx = self.idx(rect.left(), y);
                    let color = left_colors[i];
                    self.buf[idx].bg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                    self.buf[idx].ch = if i == 0 || i == left_colors.len() - 1 {
                        ' '
                    } else {
                        '│'
                    };
                }
            }

            // Draw right border
            for (i, y) in (rect.top()..rect.bottom()).enumerate() {
                if y < self.h && rect.right() > 0 && i < right_colors.len() {
                    let idx = self.idx(rect.right() - 1, y);
                    let color = right_colors[i];
                    self.buf[idx].bg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                    self.buf[idx].ch = if i == 0 || i == right_colors.len() - 1 {
                        ' '
                    } else {
                        '│'
                    };
                }
            }

            // Draw corners with special characters
            if let Some(color) = top_colors.first() {
                let idx = self.idx(rect.left(), rect.top());
                self.buf[idx].ch = '╭';
                self.buf[idx].fg = Rgba::new(
                    color.0 as f32 / 255.0,
                    color.1 as f32 / 255.0,
                    color.2 as f32 / 255.0,
                    color.3,
                );
            }

            if let Some(color) = top_colors.last() {
                if rect.right() > 0 {
                    let idx = self.idx(rect.right() - 1, rect.top());
                    self.buf[idx].ch = '╮';
                    self.buf[idx].fg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                }
            }

            if let Some(color) = bottom_colors.first() {
                if rect.bottom() > 0 {
                    let idx = self.idx(rect.left(), rect.bottom() - 1);
                    self.buf[idx].ch = '╰';
                    self.buf[idx].fg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                }
            }

            if let Some(color) = bottom_colors.last() {
                if rect.right() > 0 && rect.bottom() > 0 {
                    let idx = self.idx(rect.right() - 1, rect.bottom() - 1);
                    self.buf[idx].ch = '╯';
                    self.buf[idx].fg = Rgba::new(
                        color.0 as f32 / 255.0,
                        color.1 as f32 / 255.0,
                        color.2 as f32 / 255.0,
                        color.3,
                    );
                }
            }
        }
    }

    /// Create a clipped subview for Canvas-like drawing
    pub fn subview_mut(
        &mut self,
        offset_x: isize,
        offset_y: isize,
        clip_x: isize,
        clip_y: isize,
        clip_width: usize,
        clip_height: usize,
    ) -> SurfaceSubview<'_> {
        let clip_rect = Rect::from_coords(
            clip_x.max(0) as usize,
            clip_y.max(0) as usize,
            clip_width,
            clip_height,
        );

        SurfaceSubview::new(self, offset_x, offset_y, clip_rect)
    }

    /// Calculate the display width of text using Unicode-aware methods
    pub fn text_width(&self, text: &str) -> usize {
        text_display_width(text)
    }

    /// Write text with automatic line wrapping
    pub fn write_text_wrapped(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        max_width: usize,
        style: TextStyle,
    ) -> usize {
        let mut current_y = y;
        let mut lines_written = 0;

        for line in text.lines() {
            if current_y >= self.h {
                break;
            }

            // Simple word wrapping
            let mut remaining = line;
            let mut line_x = x;

            while !remaining.is_empty() && current_y < self.h {
                let available_width = max_width.min(self.w.saturating_sub(line_x));

                if available_width == 0 {
                    current_y += 1;
                    line_x = x;
                    lines_written += 1;
                    continue;
                }

                // Find how much text fits
                let mut fit_width = 0;
                let mut fit_chars = 0;

                for (i, grapheme) in remaining.graphemes(true).enumerate() {
                    let grapheme_width = UnicodeWidthStr::width(grapheme);
                    if fit_width + grapheme_width > available_width {
                        break;
                    }
                    fit_width += grapheme_width;
                    fit_chars = i + 1;
                }

                if fit_chars == 0 {
                    // Can't fit even one character, move to next line
                    current_y += 1;
                    line_x = x;
                    lines_written += 1;
                    continue;
                }

                // Extract the text that fits
                let fit_text: String = remaining.graphemes(true).take(fit_chars).collect();

                // Write the text
                self.write_text_enhanced(line_x, current_y, &fit_text, available_width, style);

                // Update remaining text
                remaining = &remaining[fit_text.len()..];

                if remaining.is_empty() {
                    current_y += 1;
                    lines_written += 1;
                } else {
                    // Continue on same line if there's space, otherwise wrap
                    line_x += fit_width;
                    if line_x >= self.w {
                        current_y += 1;
                        line_x = x;
                        lines_written += 1;
                    }
                }
            }
        }

        lines_written
    }

    /// Write styled text with automatic color and attribute application
    pub fn write_styled_text(&mut self, x: usize, y: usize, text: &str, style: TextStyle) {
        self.write_text_enhanced(x, y, text, 0, style);
    }

    /// Fill a rectangular area with a character and style
    pub fn fill_char_rect(&mut self, rect: Rect, ch: char, style: TextStyle) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        let cell = Cell {
            ch,
            fg: style.fg,
            bg: style.bg,
            attr: style.attr,
            image_id: None,
            image_placement: None,
        };

        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                if x < self.w && y < self.h {
                    self.set(x, y, cell);
                }
            }
        }
    }

    /// Draw a border around a rectangle
    pub fn draw_border(&mut self, rect: Rect, style: TextStyle, border_chars: Option<BorderChars>) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        if rect.width() < 2 || rect.height() < 2 {
            return;
        }

        let chars = border_chars.unwrap_or_default();

        // Top and bottom borders
        for x in rect.left()..rect.right() {
            if x < self.w {
                // Top border
                if rect.top() < self.h {
                    let ch = if x == rect.left() {
                        chars.top_left
                    } else if x == rect.right() - 1 {
                        chars.top_right
                    } else {
                        chars.horizontal
                    };
                    self.set(
                        x,
                        rect.top(),
                        Cell {
                            ch,
                            fg: style.fg,
                            bg: style.bg,
                            attr: style.attr,
                            image_id: None,
                            image_placement: None,
                        },
                    );
                }

                // Bottom border
                if rect.bottom() > 0 && rect.bottom() - 1 < self.h {
                    let ch = if x == rect.left() {
                        chars.bottom_left
                    } else if x == rect.right() - 1 {
                        chars.bottom_right
                    } else {
                        chars.horizontal
                    };
                    self.set(
                        x,
                        rect.bottom() - 1,
                        Cell {
                            ch,
                            fg: style.fg,
                            bg: style.bg,
                            attr: style.attr,
                            image_id: None,
                            image_placement: None,
                        },
                    );
                }
            }
        }

        // Left and right borders
        for y in (rect.top() + 1)..(rect.bottom() - 1) {
            if y < self.h {
                // Left border
                if rect.left() < self.w {
                    self.set(
                        rect.left(),
                        y,
                        Cell {
                            ch: chars.vertical,
                            fg: style.fg,
                            bg: style.bg,
                            attr: style.attr,
                            image_id: None,
                            image_placement: None,
                        },
                    );
                }

                // Right border
                if rect.right() > 0 && rect.right() - 1 < self.w {
                    self.set(
                        rect.right() - 1,
                        y,
                        Cell {
                            ch: chars.vertical,
                            fg: style.fg,
                            bg: style.bg,
                            attr: style.attr,
                            image_id: None,
                            image_placement: None,
                        },
                    );
                }
            }
        }
    }

    /// Clear a rectangular area (set to spaces with transparent background)
    pub fn clear_rect(&mut self, rect: Rect) {
        let bounds = Rect::from_coords(0, 0, self.w, self.h);
        let rect = rect.clamp(&bounds);

        let clear_cell = Cell {
            ch: ' ',
            fg: Rgba::transparent(),
            bg: Rgba::transparent(),
            attr: Attr::empty(),
            image_id: None,
            image_placement: None,
        };

        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                if x < self.w && y < self.h {
                    self.set(x, y, clear_cell);
                }
            }
        }
    }

    /// Draw a box with optional background color
    pub fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, bg: Option<Rgba>) {
        let (ww, hh) = self.dims();
        for yy in y..y.saturating_add(h).min(hh) {
            for xx in x..x.saturating_add(w).min(ww) {
                let mut c = self.get(xx, yy);
                if let Some(bg) = bg {
                    c.bg = bg;
                }
                self.set(xx, yy, c);
            }
        }
    }

    // === Image Management Methods ===

    /// Register a new image; zero indicates invalid data or exhausted IDs.
    pub fn register_image(&mut self, image: ImageData) -> u32 {
        self.image_registry.register_image(image)
    }

    /// Register validated raw RGBA data, reporting invalid input.
    pub fn try_register_image(&mut self, image: ImageData) -> Result<u32> {
        self.image_registry.try_register_image(image)
    }

    /// Get image data by ID
    pub fn get_image(&self, id: u32) -> Option<&ImageData> {
        self.image_registry.get_image(id)
    }

    /// Remove an image from the registry
    pub fn remove_image(&mut self, id: u32) -> Option<ImageData> {
        let removed = self.image_registry.remove_image(id)?;
        for cell in &mut self.buf {
            if cell.image_id == Some(id) {
                cell.clear_image();
            }
        }
        Some(removed)
    }

    /// Clear all images from the registry
    pub fn clear_images(&mut self) {
        self.image_registry.clear();
        self.clear_all_image_placements();
    }

    /// Get the number of registered images
    pub fn image_count(&self) -> usize {
        self.image_registry.len()
    }

    /// Set image placement for a cell
    pub fn set_cell_image(
        &mut self,
        x: usize,
        y: usize,
        image_id: u32,
        placement: ImageCellPlacement,
    ) {
        if x < self.w && y < self.h {
            let idx = self.idx(x, y);
            self.buf[idx].set_image(image_id, placement);
        }
    }

    /// Clear image placement from a cell
    pub fn clear_cell_image(&mut self, x: usize, y: usize) {
        if x < self.w && y < self.h {
            let idx = self.idx(x, y);
            self.buf[idx].clear_image();
        }
    }

    /// Place an image across multiple cells. Invalid regions leave the surface
    /// unchanged; use `try_place_image_region` to obtain the error.
    #[allow(clippy::too_many_arguments)]
    pub fn place_image_region(
        &mut self,
        start_x: usize,
        start_y: usize,
        cell_width: usize,
        cell_height: usize,
        image_id: u32,
        image_width: u32,
        image_height: u32,
        z_index: i8,
        opacity: f32,
    ) {
        let _ = self.try_place_image_region(
            start_x,
            start_y,
            cell_width,
            cell_height,
            image_id,
            image_width,
            image_height,
            z_index,
            opacity,
        );
    }

    /// Place a source region across a clipped destination. Source dimensions
    /// must fit the public placement's 16-bit coordinates and the registered image.
    #[allow(clippy::too_many_arguments)]
    pub fn try_place_image_region(
        &mut self,
        start_x: usize,
        start_y: usize,
        cell_width: usize,
        cell_height: usize,
        image_id: u32,
        image_width: u32,
        image_height: u32,
        z_index: i8,
        opacity: f32,
    ) -> Result<()> {
        if cell_width == 0
            || cell_height == 0
            || image_width == 0
            || image_height == 0
            || image_width > u32::from(u16::MAX)
            || image_height > u32::from(u16::MAX)
            || !opacity.is_finite()
            || !(0.0..=1.0).contains(&opacity)
            || self
                .get_image(image_id)
                .is_none_or(|image| image_width > image.width || image_height > image.height)
        {
            return Err(ReactiveError::invalid_parameter(
                "Invalid Surface image region, image ID or opacity",
            ));
        }
        // Partition the entire source, including its remainder. When upscaling,
        // adjacent cells may sample the same pixel instead of an empty region.
        let region = |index: usize, cells: usize, pixels: u32| {
            let start = (index as u128 * u128::from(pixels) / cells as u128) as u16;
            let end =
                (((index as u128 + 1) * u128::from(pixels) / cells as u128) as u16).max(start + 1);
            (start, end - start)
        };
        for cell_y in 0..cell_height.min(self.h.saturating_sub(start_y)) {
            for cell_x in 0..cell_width.min(self.w.saturating_sub(start_x)) {
                let x = start_x + cell_x;
                let y = start_y + cell_y;

                if x < self.w && y < self.h {
                    let (source_x, source_width) = region(cell_x, cell_width, image_width);
                    let (source_y, source_height) = region(cell_y, cell_height, image_height);

                    let placement = ImageCellPlacement {
                        source_x,
                        source_y,
                        source_width,
                        source_height,
                        z_index,
                        opacity,
                    };

                    self.set_cell_image(x, y, image_id, placement);
                }
            }
        }
        Ok(())
    }

    /// Get all cells that have image placements
    pub fn get_image_cells(&self) -> Vec<(usize, usize, u32, ImageCellPlacement)> {
        let mut result = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                let cell = self.get(x, y);
                if let (Some(image_id), Some(placement)) = (cell.image_id, cell.image_placement) {
                    result.push((x, y, image_id, placement));
                }
            }
        }
        result
    }

    /// Get direct access to the underlying cell buffer for FFI
    ///
    /// # Safety
    /// This provides raw access to the internal buffer. The caller must ensure:
    /// - The buffer is not modified while other operations are in progress
    /// - The buffer size matches width * height
    /// - No out-of-bounds access occurs
    ///
    /// Obtaining mutable cell access discards complete-grapheme metadata.
    pub unsafe fn raw_buffer_ptr(&mut self) -> *mut Cell {
        self.graphemes.clear();
        self.buf.as_mut_ptr()
    }

    /// Get the buffer size (width * height)
    pub fn buffer_size(&self) -> usize {
        self.buf.len()
    }

    /// Get direct read-only access to the cell buffer
    pub fn cells(&self) -> &[Cell] {
        &self.buf
    }

    /// Get direct mutable access to the cell buffer
    ///
    /// # Safety
    /// The caller must ensure no concurrent access occurs
    /// Obtaining mutable cell access discards complete-grapheme metadata.
    pub unsafe fn cells_mut(&mut self) -> &mut [Cell] {
        self.graphemes.clear();
        &mut self.buf
    }

    /// Create an image from raw RGBA pixel data
    pub fn create_image_from_rgba(&mut self, width: u32, height: u32, pixels: Vec<u8>) -> u32 {
        let image_data = ImageData {
            width,
            height,
            pixels,
            metadata: ImageMetadata {
                source: Some("raw_rgba".to_string()),
                format: Some("RGBA".to_string()),
                created_at: Some(std::time::SystemTime::now()),
            },
        };
        self.register_image(image_data)
    }

    /// Create a simple colored image for testing.
    #[cfg(test)]
    pub fn create_test_image(&mut self, width: u32, height: u32, r: u8, g: u8, b: u8) -> u32 {
        let Ok(bytes) = crate::widgets::display::image::decoded::dimensions(width, height) else {
            return 0;
        };
        let mut pixels = Vec::with_capacity(bytes);
        for _ in 0..bytes / 4 {
            pixels.extend_from_slice(&[r, g, b, 255]); // RGBA
        }
        self.create_image_from_rgba(width, height, pixels)
    }

    /// Place an image as a simple overlay (fills entire cell)
    pub fn place_image_simple(&mut self, x: usize, y: usize, image_id: u32, z_index: i8) {
        let placement = ImageCellPlacement {
            source_x: 0,
            source_y: 0,
            source_width: 0,
            source_height: 0,
            z_index,
            opacity: 1.0,
        };
        self.set_cell_image(x, y, image_id, placement);
    }

    /// Place an image as a background (behind text)
    pub fn place_image_background(&mut self, x: usize, y: usize, image_id: u32) {
        self.place_image_simple(x, y, image_id, -1);
    }

    /// Place an image as a foreground overlay (in front of text)
    pub fn place_image_foreground(&mut self, x: usize, y: usize, image_id: u32) {
        self.place_image_simple(x, y, image_id, 1);
    }

    /// Clear all image placements from the surface
    pub fn clear_all_image_placements(&mut self) {
        for y in 0..self.h {
            for x in 0..self.w {
                self.clear_cell_image(x, y);
            }
        }
    }

    /// Clone this surface into a new surface
    pub fn clone_into_new(&self) -> Surface {
        let mut s = Surface::new(self.w, self.h);
        s.buf.copy_from_slice(&self.buf);
        s.graphemes.clone_from(&self.graphemes);
        // Clone the image registry
        s.image_registry = ImageRegistry {
            images: self.image_registry.images.clone(),
            next_id: self.image_registry.next_id,
        };
        s
    }

    /// Copy data from another surface
    pub fn copy_from(&mut self, other: &Surface) {
        assert_eq!(self.dims(), other.dims());
        self.buf.copy_from_slice(&other.buf);
        self.graphemes.clone_from(&other.graphemes);
        // Copy the image registry
        self.image_registry = ImageRegistry {
            images: other.image_registry.images.clone(),
            next_id: other.image_registry.next_id,
        };
    }
}

/// Enhanced diff statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    /// Number of rows that changed between frames
    pub rows_changed: usize,
    /// Number of text spans written to terminal
    pub spans_written: usize,
    /// Number of individual cells that changed
    pub cells_changed: usize,
    /// Total number of cells in the surface
    pub cells_total: usize,
    /// Total bytes written to terminal output
    pub bytes_written: usize,
    /// Number of color change operations
    pub color_changes: usize,
    /// Number of text attribute changes
    pub attr_changes: usize,
    /// Number of cursor movement operations
    pub cursor_moves: usize,
    /// Efficiency ratio (changed cells / total cells)
    pub efficiency_ratio: f32,
}

/// Writer for generating terminal escape sequences from surface diffs
pub struct DiffWriter {
    /// Output buffer for escape sequences
    out: Vec<u8>,
    /// Current foreground color state
    cur_fg: Option<Rgba>,
    /// Current background color state
    cur_bg: Option<Rgba>,
    /// Current text attributes state
    cur_attr: Attr,
    /// Number of rows changed in last diff operation
    last_rows_changed: usize,
    /// Number of text spans written in last diff operation
    last_spans_written: usize,
    /// Enhanced statistics for performance tracking
    stats: DiffStats,
    /// Whether to use epsilon comparison for colors
    use_epsilon_comparison: bool,
    /// Whether to skip identical color changes
    skip_identical_colors: bool,
    image_options: crate::backend::ImageOutputOptions,
    graphics: crate::backend::suprtui::graphics::Graphics<image_output::Raster>,
    image_ids: [u32; 2],
    image_error: Option<String>,
    pending_images: Option<Vec<image_output::Raster>>,
}

impl Default for DiffWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffWriter {
    /// Create a new diff writer for computing terminal updates
    pub fn new() -> Self {
        Self::with_options(true, true)
    }

    /// Create a new DiffWriter with optimization settings
    pub fn with_options(use_epsilon_comparison: bool, skip_identical_colors: bool) -> Self {
        Self {
            out: Vec::with_capacity(1 << 20),
            cur_fg: None,
            cur_bg: None,
            cur_attr: Attr::empty(),
            last_rows_changed: 0,
            last_spans_written: 0,
            stats: DiffStats::default(),
            use_epsilon_comparison,
            skip_identical_colors,
            image_options: crate::backend::ImageOutputOptions::default(),
            graphics: Default::default(),
            image_ids: std::array::from_fn(|_| {
                crate::widgets::display::image::ProtocolRenderer::new().generate_image_id()
            }),
            image_error: None,
            pending_images: None,
        }
    }

    /// Select confirmed host graphics capabilities. The default uses decoded
    /// half-block cells. Call `try_diff` to report malformed placements.
    pub fn set_image_options(&mut self, options: crate::backend::ImageOutputOptions) {
        self.image_options = options;
    }

    pub(crate) fn refresh_image_cell_pixels(&mut self) {
        self.image_options.refresh_cell_pixels();
    }

    /// Last error from the compatibility `diff` method, which emits no output
    /// on failure. New output owners should use `try_diff` and propagate errors.
    pub fn image_error(&self) -> Option<&str> {
        self.image_error.as_deref()
    }

    /// Bytes to remove every graphics ID this writer may have emitted.
    /// The output owner must write these before relinquishing the terminal.
    pub fn image_cleanup(&self) -> Vec<u8> {
        self.graphics.cleanup()
    }

    /// Confirm that the latest prepared output was written and flushed.
    /// Without acknowledgment, subsequent diffs conservatively resend images.
    pub fn acknowledge_output(&mut self) {
        if let Some(images) = self.pending_images.take() {
            self.graphics.acknowledge(images);
        }
    }

    /// Get the latest diff statistics
    pub fn stats(&self) -> &DiffStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DiffStats::default();
    }
    #[inline]
    fn push(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes());
    }

    fn sgr_color(&mut self, r: u8, g: u8, b: u8, is_fg: bool) {
        if is_fg {
            self.push(&format!("\x1b[38;2;{r};{g};{b}m"));
        } else {
            self.push(&format!("\x1b[48;2;{r};{g};{b}m"));
        }
    }

    fn apply_attr_delta(&mut self, want: Attr) {
        let cur = self.cur_attr;
        // Turn off bits no longer needed
        if cur.contains(Attr::BOLD) && !want.contains(Attr::BOLD) {
            self.push("\x1b[22m");
        }
        if cur.contains(Attr::ITALIC) && !want.contains(Attr::ITALIC) {
            self.push("\x1b[23m");
        }
        if cur.contains(Attr::UNDERLINE) && !want.contains(Attr::UNDERLINE) {
            self.push("\x1b[24m");
        }
        if cur.contains(Attr::REVERSE) && !want.contains(Attr::REVERSE) {
            self.push("\x1b[27m");
        }
        if cur.contains(Attr::STRIKE) && !want.contains(Attr::STRIKE) {
            self.push("\x1b[29m");
        }
        // Turn on bits newly required
        if !cur.contains(Attr::BOLD) && want.contains(Attr::BOLD) {
            self.push("\x1b[1m");
        }
        if !cur.contains(Attr::ITALIC) && want.contains(Attr::ITALIC) {
            self.push("\x1b[3m");
        }
        if !cur.contains(Attr::UNDERLINE) && want.contains(Attr::UNDERLINE) {
            self.push("\x1b[4m");
        }
        if !cur.contains(Attr::REVERSE) && want.contains(Attr::REVERSE) {
            self.push("\x1b[7m");
        }
        if !cur.contains(Attr::STRIKE) && want.contains(Attr::STRIKE) {
            self.push("\x1b[9m");
        }
        self.cur_attr = want;
    }

    /// Compute the diff between two surfaces and generate terminal output
    pub fn diff(&mut self, cur: &Surface, next: &Surface, force: bool) {
        if let Err(error) = self.try_diff(cur, next, force) {
            self.image_error = Some(error.to_string());
            self.out.clear();
        }
    }

    /// Prepare complete output without writing it. Advance the caller's current
    /// surface only after successful write and flush; retry with `force` after
    /// output failure. Graphics cleanup includes possibly transmitted IDs.
    pub fn try_diff(&mut self, cur: &Surface, next: &Surface, force: bool) -> Result<()> {
        self.out.clear();
        self.image_error = None;
        self.pending_images = None;
        let planes = image_output::project(next, self.image_options, self.image_ids)?;
        let fallback = self
            .image_options
            .protocol(crate::widgets::ImageDisplayMode::Auto)
            .is_none();
        let resolve = if fallback {
            image_output::fallback
        } else {
            image_output::legacy_text
        };
        let resolve_cells = self
            .image_options
            .protocol(crate::widgets::ImageDisplayMode::Auto)
            != Some(crate::widgets::display::image::paint::ImageProtocol::Kitty);
        let current_cells = if resolve_cells {
            Some(
                (0..cur.buf.len())
                    .map(|i| resolve(cur, i))
                    .collect::<Result<Vec<_>>>()?,
            )
        } else {
            None
        };
        let next_cells = if resolve_cells {
            Some(
                (0..next.buf.len())
                    .map(|i| resolve(next, i))
                    .collect::<Result<Vec<_>>>()?,
            )
        } else {
            None
        };
        // An earlier legacy graphics image may be erased by clearing the screen;
        // repaint text whenever graphics change. Caller owns delivery and retry.
        let graphics = self
            .graphics
            .prepare(&planes, self.image_options.cell_pixels, force)?;
        let force = force || graphics.is_some();
        if let Some((before, _)) = &graphics {
            self.out.extend_from_slice(before);
        }
        self.push("\x1b[?25l");
        let (w, h) = cur.dims();
        assert_eq!((w, h), next.dims());
        self.cur_fg = None;
        self.cur_bg = None;
        self.cur_attr = Attr::empty();
        self.last_rows_changed = 0;
        self.last_spans_written = 0;

        // Reset statistics
        self.stats = DiffStats::default();
        self.stats.cells_total = w * h;
        for y in 0..h {
            let mut run_buf = String::with_capacity(w);
            let mut run_start_col: Option<usize> = None;
            let mut wrote_row = false;
            for x in 0..w {
                let index = next.idx(x, y);
                let replace_text = fallback && image_output::replaces_grapheme(next, index);
                if next.grapheme_continuation(index) && !replace_text {
                    continue;
                }
                let a = current_cells
                    .as_ref()
                    .map_or_else(|| cur.get(x, y), |cells| cells[index]);
                let b = next_cells
                    .as_ref()
                    .map_or_else(|| next.get(x, y), |cells| cells[index]);

                // Enhanced cell comparison with epsilon-based color comparison and image support
                let cells_equal = if !force {
                    if self.use_epsilon_comparison {
                        a.ch == b.ch
                            && a.fg.approx_eq(b.fg)
                            && a.bg.approx_eq(b.bg)
                            && a.attr == b.attr
                            && a.image_id == b.image_id
                            && a.image_placement == b.image_placement
                    } else {
                        a == b
                    }
                } else {
                    false
                };

                let cells_equal = cells_equal && cur.cell_text_matches(next, index);
                if cells_equal {
                    if let Some(start) = run_start_col {
                        self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                        self.push(&run_buf);
                        self.last_spans_written += 1;
                        self.stats.cursor_moves += 1;
                        wrote_row = true;
                        run_buf.clear();
                        run_start_col = None;
                    }
                    continue;
                }
                if run_start_col.is_none() {
                    run_start_col = Some(x);
                }
                // Track that this cell changed
                self.stats.cells_changed += 1;

                // Check for color/attribute changes regardless of run buffer state
                let fg_changed = if self.use_epsilon_comparison {
                    self.cur_fg.is_none_or(|cur| !cur.approx_eq(b.fg))
                } else {
                    self.cur_fg != Some(b.fg)
                };

                let bg_changed = if self.use_epsilon_comparison {
                    self.cur_bg.is_none_or(|cur| !cur.approx_eq(b.bg))
                } else {
                    self.cur_bg != Some(b.bg)
                };

                let attr_changed = self.cur_attr != b.attr;

                // If any style changed, flush the current run and start a new one
                if fg_changed || bg_changed || attr_changed {
                    // Flush current run if it has content
                    if !run_buf.is_empty() {
                        if let Some(start) = run_start_col {
                            self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                            self.push(&run_buf);
                            self.last_spans_written += 1;
                            self.stats.cursor_moves += 1;
                        }
                        run_buf.clear();
                        run_start_col = None;
                    }

                    // Apply new colors/attributes
                    if fg_changed {
                        let (r, g, bv) = (
                            (b.fg.r * 255.0) as u8,
                            (b.fg.g * 255.0) as u8,
                            (b.fg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, true);
                        self.cur_fg = Some(b.fg);
                        self.stats.color_changes += 1;
                    }

                    if bg_changed {
                        let (r, g, bv) = (
                            (b.bg.r * 255.0) as u8,
                            (b.bg.g * 255.0) as u8,
                            (b.bg.b * 255.0) as u8,
                        );
                        self.sgr_color(r, g, bv, false);
                        self.cur_bg = Some(b.bg);
                        self.stats.color_changes += 1;
                    }

                    if attr_changed {
                        self.apply_attr_delta(b.attr);
                        self.stats.attr_changes += 1;
                    }
                }

                // Start new run if needed
                if run_start_col.is_none() {
                    run_start_col = Some(x);
                }

                // Add character to current run
                if replace_text || b.ch != next.buf[index].ch {
                    run_buf.push(b.ch);
                } else {
                    next.append_cell_text(index, &mut run_buf);
                }
            }

            // Flush any remaining run at end of line
            if !run_buf.is_empty() {
                if let Some(start) = run_start_col {
                    self.push(&format!("\x1b[{y1};{x1}H", y1 = y + 1, x1 = start + 1));
                    self.push(&run_buf);
                    self.last_spans_written += 1;
                    self.stats.cursor_moves += 1;
                    wrote_row = true;
                }
            }
            if wrote_row {
                self.last_rows_changed += 1;
            }
        }
        self.push("\x1b[?25h");

        if let Some((_, after)) = graphics {
            self.out.extend_from_slice(&after);
        }
        self.pending_images = Some(planes);

        // Finalize statistics
        self.stats.rows_changed = self.last_rows_changed;
        self.stats.spans_written = self.last_spans_written;
        self.stats.bytes_written = self.out.len();
        self.stats.efficiency_ratio = if self.stats.cells_total > 0 {
            self.stats.cells_changed as f32 / self.stats.cells_total as f32
        } else {
            0.0
        };
        Ok(())
    }

    /// Get the generated terminal output
    pub fn output(&self) -> &[u8] {
        &self.out
    }
}

impl DiffWriter {
    /// Get the number of rows changed in the last diff
    pub fn last_rows_changed(&self) -> usize {
        self.last_rows_changed
    }
    /// Get the number of spans written in the last diff
    pub fn last_spans_written(&self) -> usize {
        self.last_spans_written
    }

    /// Enable or disable epsilon-based color comparison
    pub fn set_epsilon_comparison(&mut self, enabled: bool) {
        self.use_epsilon_comparison = enabled;
    }

    /// Enable or disable skipping identical colors
    pub fn set_skip_identical_colors(&mut self, enabled: bool) {
        self.skip_identical_colors = enabled;
    }

    /// Get detailed diff statistics
    pub fn detailed_stats(&self) -> DiffStats {
        self.stats.clone()
    }

    /// Check if the last diff was efficient (low change ratio)
    pub fn is_efficient(&self) -> bool {
        self.stats.efficiency_ratio < 0.3 // Less than 30% of cells changed
    }

    /// Get a performance assessment of the last diff
    pub fn performance_assessment(&self) -> &'static str {
        match self.stats.efficiency_ratio {
            r if r < 0.1 => "Excellent - Very few changes",
            r if r < 0.3 => "Good - Moderate changes",
            r if r < 0.6 => "Fair - Many changes",
            _ => "Poor - Most cells changed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffwriter_counts_increase_on_changes() {
        let a = Surface::new(10, 3);
        let mut b = Surface::new(10, 3);
        let fg = Rgba::white();
        let bg = Rgba::black();
        b.write_str(0, 0, "hello", fg, bg, Attr::empty());
        b.write_str(0, 1, "世界", fg, bg, Attr::empty());

        let mut diff = DiffWriter::new();
        diff.diff(&a, &b, false);
        assert!(diff.last_rows_changed() > 0);
        assert!(diff.last_spans_written() > 0);
        assert!(!diff.output().is_empty());
    }

    #[test]
    fn surface_reinit_reuses_capacity() {
        let mut s = Surface::new(40, 10);
        let cap_initial = s.buf.capacity();
        s.reinit(40, 10);
        let cap_same = s.buf.capacity();
        assert_eq!(
            cap_same, cap_initial,
            "capacity should not change on same-size reinit"
        );
        // shrink
        s.reinit(10, 5);
        let cap_shrink = s.buf.capacity();
        assert_eq!(
            cap_shrink, cap_initial,
            "capacity should be reused on shrink when sufficient"
        );
        // grow beyond
        s.reinit(100, 50);
        let cap_grow = s.buf.capacity();
        assert!(cap_grow >= 100 * 50);
    }

    #[test]
    fn test_image_registry_basic_operations() {
        let mut registry = ImageRegistry::new();

        // Test empty registry
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(registry.image_ids().is_empty());

        // Create test image data
        let mut pixels = Vec::new();
        for _ in 0..(16 * 16) {
            pixels.extend_from_slice(&[255, 0, 0, 255]); // Red pixels (RGBA)
        }
        let image_data = ImageData {
            width: 16,
            height: 16,
            pixels,
            metadata: ImageMetadata::default(),
        };

        // Register image
        let id = registry.register_image(image_data.clone());
        assert_eq!(id, 1);
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert_eq!(registry.image_ids(), vec![1]);

        // Get image
        let retrieved = registry.get_image(id).unwrap();
        assert_eq!(retrieved.width, 16);
        assert_eq!(retrieved.height, 16);
        assert_eq!(retrieved.pixels.len(), 16 * 16 * 4);

        // Remove image
        let removed = registry.remove_image(id).unwrap();
        assert_eq!(removed.width, 16);
        assert!(registry.is_empty());
        assert!(registry.get_image(id).is_none());
    }

    #[test]
    fn test_cell_image_placement() {
        let mut cell = Cell::default();

        // Test initial state
        assert!(!cell.has_image());
        assert_eq!(cell.image_id, None);
        assert_eq!(cell.image_placement, None);

        // Set image
        let placement = ImageCellPlacement {
            source_x: 10,
            source_y: 20,
            source_width: 16,
            source_height: 16,
            z_index: 1,
            opacity: 0.8,
        };

        cell.set_image(42, placement);
        assert!(cell.has_image());
        assert_eq!(cell.image_id, Some(42));
        assert_eq!(cell.image_placement, Some(placement));

        // Clear image
        cell.clear_image();
        assert!(!cell.has_image());
        assert_eq!(cell.image_id, None);
        assert_eq!(cell.image_placement, None);
    }

    #[test]
    fn test_surface_image_operations() {
        let mut surface = Surface::new(10, 10);

        // Create test image
        let image_id = surface.create_test_image(32, 32, 255, 0, 0); // Red image
        assert_eq!(surface.image_count(), 1);

        // Test image data
        let image_data = surface.get_image(image_id).unwrap();
        assert_eq!(image_data.width, 32);
        assert_eq!(image_data.height, 32);
        assert_eq!(image_data.pixels[0], 255); // Red channel
        assert_eq!(image_data.pixels[1], 0); // Green channel
        assert_eq!(image_data.pixels[2], 0); // Blue channel
        assert_eq!(image_data.pixels[3], 255); // Alpha channel

        // Place image in cell
        surface.place_image_background(5, 5, image_id);

        // Check cell has image
        let cell = surface.get(5, 5);
        assert!(cell.has_image());
        assert_eq!(cell.image_id, Some(image_id));
        assert!(cell.image_placement.is_some());

        // Check placement details
        let placement = cell.image_placement.unwrap();
        assert_eq!(placement.z_index, -1); // Background
        assert_eq!(placement.opacity, 1.0);

        // Get all image cells
        let image_cells = surface.get_image_cells();
        assert_eq!(image_cells.len(), 1);
        assert_eq!(image_cells[0], (5, 5, image_id, placement));

        // Clear image placement
        surface.clear_cell_image(5, 5);
        let cell = surface.get(5, 5);
        assert!(!cell.has_image());
        assert!(surface.get_image_cells().is_empty());
    }

    #[test]
    fn test_image_region_placement() {
        let mut surface = Surface::new(20, 20);

        // Create test image
        let image_id = surface.create_test_image(64, 64, 0, 255, 0); // Green image

        // Place image across a 4x3 region
        surface.place_image_region(5, 5, 4, 3, image_id, 64, 64, 0, 1.0);

        // Check that all cells in the region have the image
        let image_cells = surface.get_image_cells();
        assert_eq!(image_cells.len(), 4 * 3);

        // Verify each cell has correct placement
        for y in 5..8 {
            for x in 5..9 {
                let cell = surface.get(x, y);
                assert!(cell.has_image());
                assert_eq!(cell.image_id, Some(image_id));

                let placement = cell.image_placement.unwrap();
                assert_eq!(placement.z_index, 0);
                assert_eq!(placement.opacity, 1.0);

                // Check source coordinates are distributed across the image
                let expected_source_x = ((x - 5) as u32 * 16) as u16; // 64/4 = 16 pixels per cell
                let expected_source_y = ((y - 5) as u32 * 21) as u16; // 64/3 ≈ 21 pixels per cell
                assert_eq!(placement.source_x, expected_source_x);
                assert_eq!(placement.source_y, expected_source_y);
            }
        }
        let last = surface.get(8, 7).image_placement.unwrap();
        assert_eq!(u32::from(last.source_y) + u32::from(last.source_height), 64);
    }

    #[test]
    fn api_surface_image_small_source_and_removal() {
        let mut surface = Surface::new(4, 2);
        let id = surface.create_test_image(1, 1, 255, 0, 0);
        surface.place_image_region(0, 0, 4, 2, id, 1, 1, 1, 1.0);
        for (_, _, _, placement) in surface.get_image_cells() {
            assert_eq!(placement.source_width, 1);
            assert_eq!(placement.source_height, 1);
        }
        surface.remove_image(id).unwrap();
        assert!(surface.get_image_cells().is_empty());
    }

    #[test]
    fn api_surface_image_validation_and_clipped_work() {
        let mut surface = Surface::new(2, 1);
        assert_eq!(surface.create_image_from_rgba(2, 1, vec![0; 7]), 0);
        assert_eq!(surface.create_test_image(u32::MAX, u32::MAX, 0, 0, 0), 0);
        assert_eq!(surface.image_count(), 0);
        let id = surface.create_test_image(1, 1, 255, 0, 0);
        surface
            .try_place_image_region(0, 0, usize::MAX, usize::MAX, id, 1, 1, 1, 1.0)
            .unwrap();
        assert_eq!(surface.get_image_cells().len(), 2);
        surface
            .try_place_image_region(
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
                id,
                1,
                1,
                1,
                1.0,
            )
            .unwrap();
        assert!(surface
            .try_place_image_region(0, 0, 1, 1, id, 1, 1, 1, f32::NAN)
            .is_err());
        surface.clear_images();
        assert!(surface.get_image_cells().is_empty());
        let new_id = surface.create_test_image(1, 1, 0, 255, 0);
        assert_ne!(id, new_id);
        let mut registry = ImageRegistry::default();
        let data = surface.get_image(new_id).unwrap().clone();
        assert_ne!(registry.try_register_image(data.clone()).unwrap(), 0);
        registry.next_id = u32::MAX;
        assert_eq!(registry.try_register_image(data.clone()).unwrap(), u32::MAX);
        assert!(registry.try_register_image(data).is_err());
    }

    #[test]
    fn test_surface_copy_preserves_images() {
        let mut surface1 = Surface::new(5, 5);

        // Create and place image
        let image_id = surface1.create_test_image(16, 16, 0, 0, 255); // Blue image
        surface1.place_image_foreground(2, 2, image_id);

        // Test clone_into_new
        let surface2 = surface1.clone_into_new();
        assert_eq!(surface2.image_count(), 1);
        assert!(surface2.get(2, 2).has_image());

        // Test copy_from
        let mut surface3 = Surface::new(5, 5);
        surface3.copy_from(&surface1);
        assert_eq!(surface3.image_count(), 1);
        assert!(surface3.get(2, 2).has_image());

        // Verify image data is preserved
        let original_image = surface1.get_image(image_id).unwrap();
        let copied_image = surface3.get_image(image_id).unwrap();
        assert_eq!(original_image.width, copied_image.width);
        assert_eq!(original_image.height, copied_image.height);
        assert_eq!(original_image.pixels, copied_image.pixels);
    }

    #[test]
    fn test_clear_all_image_placements() {
        let mut surface = Surface::new(10, 10);

        // Create multiple images and place them
        let image1 = surface.create_test_image(16, 16, 255, 0, 0);
        let image2 = surface.create_test_image(16, 16, 0, 255, 0);

        surface.place_image_background(1, 1, image1);
        surface.place_image_foreground(5, 5, image2);
        surface.place_image_simple(8, 8, image1, 2);

        // Verify placements exist
        assert_eq!(surface.get_image_cells().len(), 3);

        // Clear all placements
        surface.clear_all_image_placements();

        // Verify all placements are gone
        assert!(surface.get_image_cells().is_empty());
        assert!(!surface.get(1, 1).has_image());
        assert!(!surface.get(5, 5).has_image());
        assert!(!surface.get(8, 8).has_image());

        // But images should still be in registry
        assert_eq!(surface.image_count(), 2);
    }

    #[test]
    fn test_enhanced_surface_features() {
        let mut surface = Surface::new(80, 24);

        // Test Unicode text width calculation
        assert_eq!(text_display_width("Hello"), 5);
        assert_eq!(text_display_width("世界"), 4); // 2 wide chars
        assert_eq!(text_display_width("🌍"), 2); // emoji

        // Test enhanced text writing
        let style = TextStyle::new(
            Rgba::new(1.0, 1.0, 1.0, 1.0),
            Rgba::new(0.0, 0.0, 0.0, 1.0),
            Attr::BOLD,
        );

        surface.write_text_enhanced(5, 2, "Hello, 世界!", 20, style);

        // Test subview clipping
        {
            let mut subview = surface.subview_mut(10, 5, 10, 5, 20, 10);
            subview.write_text(0, 0, "Clipped text", style);
        }

        // Test border drawing
        let border_rect = Rect::from_coords(30, 8, 20, 8);
        surface.draw_border(border_rect, style, Some(BorderChars::rounded()));

        // Test background filling
        let fill_rect = Rect::from_coords(55, 10, 15, 5);
        surface.fill_background_rect(fill_rect, Rgba::new(0.8, 0.4, 0.4, 1.0));

        // Test character filling
        let char_rect = Rect::from_coords(60, 2, 10, 3);
        surface.fill_char_rect(char_rect, '█', TextStyle::fg(Rgba::new(0.0, 1.0, 0.0, 1.0)));

        // Test clearing
        let clear_rect = Rect::from_coords(2, 20, 20, 3);
        surface.clear_rect(clear_rect);

        // Verify surface dimensions
        assert_eq!(surface.w, 80);
        assert_eq!(surface.h, 24);
    }

    #[test]
    fn test_border_chars() {
        let default_chars = BorderChars::default();
        assert_eq!(default_chars.top_left, '┌');
        assert_eq!(default_chars.horizontal, '─');

        let ascii_chars = BorderChars::ascii();
        assert_eq!(ascii_chars.top_left, '+');
        assert_eq!(ascii_chars.horizontal, '-');

        let rounded_chars = BorderChars::rounded();
        assert_eq!(rounded_chars.top_left, '╭');

        let double_chars = BorderChars::double();
        assert_eq!(double_chars.horizontal, '═');
    }

    #[test]
    fn test_text_style() {
        let style = TextStyle::new(
            Rgba::new(1.0, 0.0, 0.0, 1.0),
            Rgba::new(0.0, 1.0, 0.0, 1.0),
            Attr::BOLD,
        );

        assert_eq!(style.fg, Rgba::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(style.bg, Rgba::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(style.attr, Attr::BOLD);
        assert!(style.emoji_aware);

        let fg_style = TextStyle::fg(Rgba::new(0.4, 0.4, 0.4, 1.0));
        assert_eq!(fg_style.fg, Rgba::new(0.4, 0.4, 0.4, 1.0));
        assert_eq!(fg_style.bg, Rgba::transparent());

        let bold_style = TextStyle::fg(Rgba::white()).bold();
        assert!(bold_style.attr.contains(Attr::BOLD));

        let italic_style = TextStyle::fg(Rgba::white()).italic();
        assert!(italic_style.attr.contains(Attr::ITALIC));

        let underline_style = TextStyle::fg(Rgba::white()).underline();
        assert!(underline_style.attr.contains(Attr::UNDERLINE));
    }
}
