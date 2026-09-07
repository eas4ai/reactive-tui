//! Packed color model and text attributes, ported from the reference `ansi.zig`.
//!
//! Only the color/attribute core needed by the buffer domain lives here
//! (BUF-001, BUF-002, BUF-003): the packed `Rgba` layout with intent
//! metadata, the 16-entry palette, the 256-color cube, and the attribute
//! word with style flags plus link id. Terminal escape writers, HSV
//! conversion, and color distance arrive with the render/terminal and
//! media commitments that need them.

/// Packed color with embedded metadata.
///
/// Each of the four `u16` components stores an 8-bit channel value in its
/// low byte and one byte of a 32-bit metadata word in its high byte:
/// `component[i] = channel_byte | (meta_byte << 8)`. The metadata encodes
/// a [`ColorIntent`] and an optional palette slot, so color + intent stay
/// one comparable 64-bit value.
pub type Rgba = [u16; 4];

/// Original color intent: literal RGB, indexed palette, or terminal default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorIntent {
    Rgb = 0,
    Indexed = 1,
    Default = 2,
}

/// Standard ANSI 16-color palette RGB values (colors 0-15).
pub const ANSI16_RGB: [[u8; 3]; 16] = [
    [0x00, 0x00, 0x00],
    [0x80, 0x00, 0x00],
    [0x00, 0x80, 0x00],
    [0x80, 0x80, 0x00],
    [0x00, 0x00, 0x80],
    [0x80, 0x00, 0x80],
    [0x00, 0x80, 0x80],
    [0xc0, 0xc0, 0xc0],
    [0x80, 0x80, 0x80],
    [0xff, 0x00, 0x00],
    [0x00, 0xff, 0x00],
    [0xff, 0xff, 0x00],
    [0x00, 0x00, 0xff],
    [0xff, 0x00, 0xff],
    [0x00, 0xff, 0xff],
    [0xff, 0xff, 0xff],
];

/// The six intensity levels of the ANSI 256-color cube (indices 16-231).
pub const ANSI_256_CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Quantize a 0.0-1.0 float channel to a byte. Clamps out-of-range and
/// non-finite inputs. Reference `rgbaComponentToU8`.
pub fn component_to_u8(component: f32) -> u8 {
    if !component.is_finite() {
        return 0;
    }
    (component.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Convert a byte channel back to a 0.0-1.0 float.
pub fn u8_to_component(component: u8) -> f32 {
    f32::from(component) / 255.0
}

/// Pack an intent and palette slot into a 32-bit metadata word:
/// bits 0-7 = slot, bits 8-9 = intent.
pub fn pack_meta(intent: ColorIntent, slot: u8) -> u32 {
    u32::from(slot) | ((intent as u32) << 8)
}

/// Build an RGBA value from 8-bit channels and a metadata word.
pub fn pack_rgba8(r: u8, g: u8, b: u8, a: u8, meta: u32) -> Rgba {
    [
        u16::from(r) | (((meta & 0xFF) as u16) << 8),
        u16::from(g) | ((((meta >> 8) & 0xFF) as u16) << 8),
        u16::from(b) | ((((meta >> 16) & 0xFF) as u16) << 8),
        u16::from(a) | ((((meta >> 24) & 0xFF) as u16) << 8),
    ]
}

pub fn red(c: Rgba) -> u8 {
    (c[0] & 0xFF) as u8
}

pub fn green(c: Rgba) -> u8 {
    (c[1] & 0xFF) as u8
}

pub fn blue(c: Rgba) -> u8 {
    (c[2] & 0xFF) as u8
}

pub fn alpha(c: Rgba) -> u8 {
    (c[3] & 0xFF) as u8
}

pub fn red_f(c: Rgba) -> f32 {
    u8_to_component(red(c))
}

pub fn green_f(c: Rgba) -> f32 {
    u8_to_component(green(c))
}

pub fn blue_f(c: Rgba) -> f32 {
    u8_to_component(blue(c))
}

pub fn alpha_f(c: Rgba) -> f32 {
    u8_to_component(alpha(c))
}

/// Reassemble the 32-bit metadata word from the high bytes.
pub fn get_meta(c: Rgba) -> u32 {
    u32::from(c[0] >> 8)
        | (u32::from(c[1] >> 8) << 8)
        | (u32::from(c[2] >> 8) << 16)
        | (u32::from(c[3] >> 8) << 24)
}

/// Color intent stored in the metadata.
pub fn intent(c: Rgba) -> ColorIntent {
    match ((get_meta(c) >> 8) & 0xFF) as u8 {
        1 => ColorIntent::Indexed,
        2 => ColorIntent::Default,
        _ => ColorIntent::Rgb,
    }
}

/// Palette slot stored in the metadata. Meaningful for indexed intent.
pub fn slot(c: Rgba) -> u8 {
    (get_meta(c) & 0xFF) as u8
}

/// Copy a color with different metadata, preserving channels.
pub fn with_meta(c: Rgba, meta: u32) -> Rgba {
    pack_rgba8(red(c), green(c), blue(c), alpha(c), meta)
}

/// Literal RGB color. Reference `rgbColor`.
pub fn rgb_color(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    pack_rgba8(r, g, b, a, pack_meta(ColorIntent::Rgb, 0))
}

/// Literal RGB color from 0.0-1.0 float channels.
pub fn rgba_from_floats(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    rgb_color(
        component_to_u8(r),
        component_to_u8(g),
        component_to_u8(b),
        component_to_u8(a),
    )
}

/// Indexed ANSI color with an RGB snapshot for blending. Reference
/// `indexedColor`.
pub fn indexed_color(index: u8, r: u8, g: u8, b: u8) -> Rgba {
    pack_rgba8(r, g, b, 255, pack_meta(ColorIntent::Indexed, index))
}

/// Terminal-default color. Reference `defaultColor`.
pub fn default_color(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    pack_rgba8(r, g, b, a, pack_meta(ColorIntent::Default, 0))
}

/// Opaque literal RGB. Reference `u8RgbToRgba`.
pub fn u8_rgb_to_rgba(r: u8, g: u8, b: u8) -> Rgba {
    rgb_color(r, g, b, 255)
}

/// ANSI 256-color palette index to RGB: base 16, 6x6x6 cube, gray ramp.
pub fn fallback_ansi256_color(index: usize) -> Rgba {
    if index < ANSI16_RGB.len() {
        let base = ANSI16_RGB[index];
        return u8_rgb_to_rgba(base[0], base[1], base[2]);
    }
    if index < 232 {
        let cube = index - 16;
        return u8_rgb_to_rgba(
            ANSI_256_CUBE_LEVELS[(cube / 36) % 6],
            ANSI_256_CUBE_LEVELS[(cube / 6) % 6],
            ANSI_256_CUBE_LEVELS[cube % 6],
        );
    }
    let gray: u8 = (8 + (index - 232) * 10) as u8;
    u8_rgb_to_rgba(gray, gray, gray)
}

/// Style flags (low 8 bits of the attribute word) and link-id packing
/// (upper 24 bits). Reference `TextAttributes`.
pub struct TextAttributes;

impl TextAttributes {
    pub const NONE: u8 = 0;
    pub const BOLD: u8 = 1 << 0;
    pub const DIM: u8 = 1 << 1;
    pub const ITALIC: u8 = 1 << 2;
    pub const UNDERLINE: u8 = 1 << 3;
    pub const BLINK: u8 = 1 << 4;
    pub const INVERSE: u8 = 1 << 5;
    pub const HIDDEN: u8 = 1 << 6;
    pub const STRIKETHROUGH: u8 = 1 << 7;

    pub const ATTRIBUTE_BASE_BITS: u32 = 8;
    pub const ATTRIBUTE_BASE_MASK: u32 = 0xFF;
    pub const LINK_ID_BITS: u32 = 24;
    pub const LINK_ID_SHIFT: u32 = Self::ATTRIBUTE_BASE_BITS;
    pub const LINK_ID_PAYLOAD_MASK: u32 = (1 << Self::LINK_ID_BITS) - 1;
    pub const LINK_ID_MASK: u32 = Self::LINK_ID_PAYLOAD_MASK << Self::LINK_ID_SHIFT;

    /// Low 8 style bits of an attribute word.
    pub fn base_attributes(attr: u32) -> u8 {
        (attr & Self::ATTRIBUTE_BASE_MASK) as u8
    }

    /// Link id from bits 8-31.
    pub fn link_id(attr: u32) -> u32 {
        (attr & Self::LINK_ID_MASK) >> Self::LINK_ID_SHIFT
    }

    /// Set the link id, preserving style flags.
    pub fn set_link_id(attr: u32, link_id: u32) -> u32 {
        (attr & Self::ATTRIBUTE_BASE_MASK)
            | ((link_id & Self::LINK_ID_PAYLOAD_MASK) << Self::LINK_ID_SHIFT)
    }

    /// Whether an attribute word carries a link.
    pub fn has_link(attr: u32) -> bool {
        Self::link_id(attr) != 0
    }
}
