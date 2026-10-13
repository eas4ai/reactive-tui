//! Image fallback blitters (docs/spec/blitters.md, BLT-001 and BLT-002).
//!
//! A blitter draws a block of pixels as one cell: a glyph whose set parts
//! take the foreground color and whose clear parts take the background.
//! For each block the glyph and its two colors come from the split of the
//! block's pixels into two sets with the least total squared color error,
//! found by trying every split. A transparent pixel makes the background
//! transparent: the opaque pixels form the glyph and the rest of the cell
//! shows what is below it. The tiers and the exhaustive split follow the
//! notcurses blitters as ideas, not code (see UPSTREAM.md).

/// One way to draw pixel blocks as cells, in tier order from the most
/// pixels per cell to the fewest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Blitter {
    /// 2 by 4 braille dots, U+2800 to U+28FF.
    Braille,
    /// 2 by 4 octants, U+1CD00 to U+1CDE5 with the block elements that
    /// already drew the other patterns.
    Octant,
    /// 2 by 3 sextants, U+1FB00 to U+1FB3B with the half and full blocks.
    Sextant,
    /// 2 by 2 quadrants with the half and full blocks.
    Quadrant,
    /// Upper and lower half blocks, one pixel above the other.
    HalfBlock,
    /// One pixel per cell, drawn as a space on the pixel's color.
    Ascii,
}

/// A pixel whose alpha is below this is transparent.
pub const OPAQUE_ALPHA: u8 = 128;

/// The environment variable that overrides the blitter choice with a
/// blitter name (see [`Blitter::from_name`]).
pub const BLITTER_ENV: &str = "REACTIVE_TUI_BLITTER";

impl Blitter {
    /// Every blitter in tier order.
    pub const TIERS: [Blitter; 6] = [
        Blitter::Braille,
        Blitter::Octant,
        Blitter::Sextant,
        Blitter::Quadrant,
        Blitter::HalfBlock,
        Blitter::Ascii,
    ];

    /// Pixels per cell as (columns, rows).
    pub const fn cell_pixels(self) -> (u32, u32) {
        match self {
            Blitter::Braille | Blitter::Octant => (2, 4),
            Blitter::Sextant => (2, 3),
            Blitter::Quadrant => (2, 2),
            Blitter::HalfBlock => (1, 2),
            Blitter::Ascii => (1, 1),
        }
    }

    /// The name [`Blitter::from_name`] reads.
    pub const fn name(self) -> &'static str {
        match self {
            Blitter::Braille => "braille",
            Blitter::Octant => "octant",
            Blitter::Sextant => "sextant",
            Blitter::Quadrant => "quadrant",
            Blitter::HalfBlock => "half-block",
            Blitter::Ascii => "ascii",
        }
    }

    /// The blitter a name selects, ignoring case and surrounding space.
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::TIERS
            .into_iter()
            .find(|blitter| blitter.name().eq_ignore_ascii_case(name))
    }

    /// The glyph for a pattern of set pixels: bit `i` is pixel `i` in
    /// reading order, left to right and then top to bottom. Bits past the
    /// cell's pixel count are ignored; the empty pattern is a space.
    pub fn glyph(self, pattern: u8) -> char {
        let code = match self {
            Blitter::Braille if pattern == 0 => 0x20,
            Blitter::Braille => 0x2800 + braille_dots(pattern),
            Blitter::Octant => OCTANT[usize::from(pattern)],
            Blitter::Sextant => SEXTANT[usize::from(pattern & 0x3F)],
            Blitter::Quadrant => QUADRANT[usize::from(pattern & 0x0F)],
            Blitter::HalfBlock => HALF[usize::from(pattern & 0x03)],
            Blitter::Ascii => 0x20,
        };
        char::from_u32(code).unwrap_or(' ')
    }
}

/// Braille numbers its dots down the left column (1, 2, 3, then 7) and
/// then down the right one (4, 5, 6, then 8); dot `n` is bit `n - 1`.
fn braille_dots(pattern: u8) -> u32 {
    const DOT_BIT: [u8; 8] = [0, 3, 1, 4, 2, 5, 6, 7];
    (0..8)
        .filter(|pixel| pattern & (1 << pixel) != 0)
        .fold(0, |dots, pixel| dots | 1 << DOT_BIT[pixel])
}

/// One cell a blitter drew. A `None` color is transparent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlitCell {
    /// The glyph for `pattern`.
    pub glyph: char,
    /// The set pixels: bit `i` is pixel `i` in reading order.
    pub pattern: u8,
    /// The mean color of the set pixels; `None` when no pixel is set.
    pub fg: Option<[u8; 3]>,
    /// The mean color of the clear pixels; `None` when the block holds a
    /// transparent pixel, so the cell shows what is below it.
    pub bg: Option<[u8; 3]>,
}

/// Draw one block of pixels, given in reading order, as one cell. Pixels
/// missing from `pixels` are transparent and extra ones are ignored.
pub fn blit_block(blitter: Blitter, pixels: &[[u8; 4]]) -> BlitCell {
    let (columns, rows) = blitter.cell_pixels();
    let count = (columns * rows) as usize;
    let mut block = [[0u8; 4]; 8];
    for (slot, pixel) in block.iter_mut().zip(pixels.iter().take(count)) {
        *slot = *pixel;
    }
    let block = &block[..count];
    let opaque = block
        .iter()
        .enumerate()
        .filter(|(_, pixel)| pixel[3] >= OPAQUE_ALPHA)
        .fold(0u8, |mask, (i, _)| mask | 1 << i);
    let full = if count == 8 { 0xFF } else { (1u8 << count) - 1 };
    if opaque != full {
        // A transparent pixel: the opaque ones are the glyph and the
        // background stays transparent.
        return BlitCell {
            glyph: blitter.glyph(opaque),
            pattern: opaque,
            fg: mean(block, opaque),
            bg: None,
        };
    }
    let pattern = best_split(block);
    BlitCell {
        glyph: blitter.glyph(pattern),
        pattern,
        fg: mean(block, pattern),
        bg: mean(block, full & !pattern),
    }
}

/// The split with the least total squared error. A split's error is
/// the sum of each pixel's squared distance to its set's mean, which is
/// `sum |p|^2 - |S1|^2 / n1 - |S0|^2 / n0` for set sums `S` and sizes `n`,
/// so the best split has the largest `|S1|^2 / n1 + |S0|^2 / n0`. A split
/// and its complement are the same two sets with the colors swapped, so
/// only splits that leave pixel 0 clear are tried; the first of equal
/// splits wins. The comparison is exact in integers.
fn best_split(block: &[[u8; 4]]) -> u8 {
    let count = block.len() as u64;
    let total = block.iter().fold([0u64; 3], |sum, pixel| {
        [
            sum[0] + u64::from(pixel[0]),
            sum[1] + u64::from(pixel[1]),
            sum[2] + u64::from(pixel[2]),
        ]
    });
    let norm = |s: [u64; 3]| s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
    // (numerator, denominator) of the score for the empty split.
    let mut best = (0u8, norm(total), count);
    for pattern in 1..(1u16 << (block.len() - 1)) {
        let pattern = (pattern << 1) as u8;
        let mut set = [0u64; 3];
        let mut n1 = 0u64;
        for (i, pixel) in block.iter().enumerate() {
            if pattern & (1 << i) != 0 {
                set[0] += u64::from(pixel[0]);
                set[1] += u64::from(pixel[1]);
                set[2] += u64::from(pixel[2]);
                n1 += 1;
            }
        }
        let clear = [total[0] - set[0], total[1] - set[1], total[2] - set[2]];
        let n0 = count - n1;
        let numerator = norm(set) * n0 + norm(clear) * n1;
        let denominator = n1 * n0;
        if numerator * best.2 > best.1 * denominator {
            best = (pattern, numerator, denominator);
        }
    }
    best.0
}

/// The rounded mean color of the pixels `mask` selects.
fn mean(block: &[[u8; 4]], mask: u8) -> Option<[u8; 3]> {
    let (sum, n) = block
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .fold(([0u32; 3], 0u32), |(sum, n), (_, pixel)| {
            (
                [
                    sum[0] + u32::from(pixel[0]),
                    sum[1] + u32::from(pixel[1]),
                    sum[2] + u32::from(pixel[2]),
                ],
                n + 1,
            )
        });
    (n > 0).then(|| sum.map(|channel| ((channel + n / 2) / n) as u8))
}

/// An image drawn as cells, row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlitGrid {
    /// Width in cells.
    pub columns: u32,
    /// Height in cells.
    pub rows: u32,
    /// `columns * rows` cells in reading order.
    pub cells: Vec<BlitCell>,
}

/// Draw an image as cells, one block of [`Blitter::cell_pixels`] per cell.
/// Pixels past the image's right and bottom edges are transparent.
pub fn blit_image(blitter: Blitter, image: &image::RgbaImage) -> BlitGrid {
    let (width, height) = image.dimensions();
    let (block_width, block_height) = blitter.cell_pixels();
    let columns = width.div_ceil(block_width);
    let rows = height.div_ceil(block_height);
    let mut cells = Vec::with_capacity(columns as usize * rows as usize);
    let mut block = [[0u8; 4]; 8];
    for row in 0..rows {
        for column in 0..columns {
            for dy in 0..block_height {
                for dx in 0..block_width {
                    let (x, y) = (column * block_width + dx, row * block_height + dy);
                    block[(dy * block_width + dx) as usize] = if x < width && y < height {
                        image.get_pixel(x, y).0
                    } else {
                        [0; 4]
                    };
                }
            }
            cells.push(blit_block(
                blitter,
                &block[..(block_width * block_height) as usize],
            ));
        }
    }
    BlitGrid {
        columns,
        rows,
        cells,
    }
}

/// Hosts known to draw a tier's glyphs themselves rather than from a
/// font, or known to lack them, by the identity their environment gives
/// (`TERM_PROGRAM`, or `TERM` when that is unset). No terminal reports its
/// glyph coverage, so this table is the only knowledge the choice has.
const HOST_TIERS: [(&str, Blitter); 9] = [
    ("kitty", Blitter::Octant),
    ("xterm-kitty", Blitter::Octant),
    ("ghostty", Blitter::Octant),
    ("xterm-ghostty", Blitter::Octant),
    ("foot", Blitter::Octant),
    ("foot-extra", Blitter::Octant),
    ("Apple_Terminal", Blitter::Quadrant),
    ("vscode", Blitter::Quadrant),
    ("linux", Blitter::HalfBlock),
];

/// The tier the per-terminal table gives a host identity, ignoring case,
/// or `None` for a host not in the table.
pub fn table_tier(identity: &str) -> Option<Blitter> {
    let identity = identity.trim();
    HOST_TIERS
        .iter()
        .find(|(host, _)| host.eq_ignore_ascii_case(identity))
        .map(|(_, blitter)| *blitter)
}

/// Choose the blitter by tier (BLT-002). An environment override, and
/// then an application override, replaces the choice. Otherwise a host
/// whose capability report says it has no unicode gets ASCII, a host in
/// the table gets its entry, and any other host gets sextant.
pub fn choose(
    unicode: bool,
    identity: Option<&str>,
    application: Option<Blitter>,
    environment: Option<Blitter>,
) -> Blitter {
    environment.or(application).unwrap_or_else(|| {
        if unicode {
            identity.and_then(table_tier).unwrap_or(Blitter::Sextant)
        } else {
            Blitter::Ascii
        }
    })
}

/// The override a value of [`BLITTER_ENV`] names: `None` for an empty
/// value, and an error naming a value that is not a blitter name.
pub fn parse_override(value: &str) -> Result<Option<Blitter>, String> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    Blitter::from_name(value).map(Some).ok_or_else(|| {
        format!(
            "{BLITTER_ENV}={value:?} names no blitter; use braille, octant, sextant, quadrant, \
             half-block or ascii"
        )
    })
}

/// The override the environment names through [`BLITTER_ENV`]; `None`
/// when the variable is unset.
pub fn environment_override() -> Result<Option<Blitter>, String> {
    match std::env::var(BLITTER_ENV) {
        Ok(value) => parse_override(&value),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(format!("{BLITTER_ENV} is not Unicode")),
    }
}

/// Half blocks: bit 0 is the upper pixel, bit 1 the lower.
const HALF: [u32; 4] = [0x20, 0x2580, 0x2584, 0x2588];

/// Quadrants: bits 0 to 3 are upper left, upper right, lower left and
/// lower right.
const QUADRANT: [u32; 16] = [
    0x20, 0x2598, 0x259D, 0x2580, 0x2596, 0x258C, 0x259E, 0x259B, 0x2597, 0x259A, 0x2590, 0x259C,
    0x2584, 0x2599, 0x259F, 0x2588,
];

/// Octants: bit `i` is octant `i + 1` in reading order. Patterns that the
/// block elements already drew use those code points. Derived from the
/// Unicode 16 character names.
const OCTANT: [u32; 256] = [
    0x00020, 0x1CEA8, 0x1CEAB, 0x1FB82, 0x1CD00, 0x02598, 0x1CD01, 0x1CD02, 0x1CD03, 0x1CD04,
    0x0259D, 0x1CD05, 0x1CD06, 0x1CD07, 0x1CD08, 0x02580, 0x1CD09, 0x1CD0A, 0x1CD0B, 0x1CD0C,
    0x1FBE6, 0x1CD0D, 0x1CD0E, 0x1CD0F, 0x1CD10, 0x1CD11, 0x1CD12, 0x1CD13, 0x1CD14, 0x1CD15,
    0x1CD16, 0x1CD17, 0x1CD18, 0x1CD19, 0x1CD1A, 0x1CD1B, 0x1CD1C, 0x1CD1D, 0x1CD1E, 0x1CD1F,
    0x1FBE7, 0x1CD20, 0x1CD21, 0x1CD22, 0x1CD23, 0x1CD24, 0x1CD25, 0x1CD26, 0x1CD27, 0x1CD28,
    0x1CD29, 0x1CD2A, 0x1CD2B, 0x1CD2C, 0x1CD2D, 0x1CD2E, 0x1CD2F, 0x1CD30, 0x1CD31, 0x1CD32,
    0x1CD33, 0x1CD34, 0x1CD35, 0x1FB85, 0x1CEA3, 0x1CD36, 0x1CD37, 0x1CD38, 0x1CD39, 0x1CD3A,
    0x1CD3B, 0x1CD3C, 0x1CD3D, 0x1CD3E, 0x1CD3F, 0x1CD40, 0x1CD41, 0x1CD42, 0x1CD43, 0x1CD44,
    0x02596, 0x1CD45, 0x1CD46, 0x1CD47, 0x1CD48, 0x0258C, 0x1CD49, 0x1CD4A, 0x1CD4B, 0x1CD4C,
    0x0259E, 0x1CD4D, 0x1CD4E, 0x1CD4F, 0x1CD50, 0x0259B, 0x1CD51, 0x1CD52, 0x1CD53, 0x1CD54,
    0x1CD55, 0x1CD56, 0x1CD57, 0x1CD58, 0x1CD59, 0x1CD5A, 0x1CD5B, 0x1CD5C, 0x1CD5D, 0x1CD5E,
    0x1CD5F, 0x1CD60, 0x1CD61, 0x1CD62, 0x1CD63, 0x1CD64, 0x1CD65, 0x1CD66, 0x1CD67, 0x1CD68,
    0x1CD69, 0x1CD6A, 0x1CD6B, 0x1CD6C, 0x1CD6D, 0x1CD6E, 0x1CD6F, 0x1CD70, 0x1CEA0, 0x1CD71,
    0x1CD72, 0x1CD73, 0x1CD74, 0x1CD75, 0x1CD76, 0x1CD77, 0x1CD78, 0x1CD79, 0x1CD7A, 0x1CD7B,
    0x1CD7C, 0x1CD7D, 0x1CD7E, 0x1CD7F, 0x1CD80, 0x1CD81, 0x1CD82, 0x1CD83, 0x1CD84, 0x1CD85,
    0x1CD86, 0x1CD87, 0x1CD88, 0x1CD89, 0x1CD8A, 0x1CD8B, 0x1CD8C, 0x1CD8D, 0x1CD8E, 0x1CD8F,
    0x02597, 0x1CD90, 0x1CD91, 0x1CD92, 0x1CD93, 0x0259A, 0x1CD94, 0x1CD95, 0x1CD96, 0x1CD97,
    0x02590, 0x1CD98, 0x1CD99, 0x1CD9A, 0x1CD9B, 0x0259C, 0x1CD9C, 0x1CD9D, 0x1CD9E, 0x1CD9F,
    0x1CDA0, 0x1CDA1, 0x1CDA2, 0x1CDA3, 0x1CDA4, 0x1CDA5, 0x1CDA6, 0x1CDA7, 0x1CDA8, 0x1CDA9,
    0x1CDAA, 0x1CDAB, 0x02582, 0x1CDAC, 0x1CDAD, 0x1CDAE, 0x1CDAF, 0x1CDB0, 0x1CDB1, 0x1CDB2,
    0x1CDB3, 0x1CDB4, 0x1CDB5, 0x1CDB6, 0x1CDB7, 0x1CDB8, 0x1CDB9, 0x1CDBA, 0x1CDBB, 0x1CDBC,
    0x1CDBD, 0x1CDBE, 0x1CDBF, 0x1CDC0, 0x1CDC1, 0x1CDC2, 0x1CDC3, 0x1CDC4, 0x1CDC5, 0x1CDC6,
    0x1CDC7, 0x1CDC8, 0x1CDC9, 0x1CDCA, 0x1CDCB, 0x1CDCC, 0x1CDCD, 0x1CDCE, 0x1CDCF, 0x1CDD0,
    0x1CDD1, 0x1CDD2, 0x1CDD3, 0x1CDD4, 0x1CDD5, 0x1CDD6, 0x1CDD7, 0x1CDD8, 0x1CDD9, 0x1CDDA,
    0x02584, 0x1CDDB, 0x1CDDC, 0x1CDDD, 0x1CDDE, 0x02599, 0x1CDDF, 0x1CDE0, 0x1CDE1, 0x1CDE2,
    0x0259F, 0x1CDE3, 0x02586, 0x1CDE4, 0x1CDE5, 0x02588,
];

/// Sextants: bit `i` is sextant `i + 1` in reading order; the empty, left,
/// right and full patterns are the space and the half and full blocks.
const SEXTANT: [u32; 64] = [
    0x00020, 0x1FB00, 0x1FB01, 0x1FB02, 0x1FB03, 0x1FB04, 0x1FB05, 0x1FB06, 0x1FB07, 0x1FB08,
    0x1FB09, 0x1FB0A, 0x1FB0B, 0x1FB0C, 0x1FB0D, 0x1FB0E, 0x1FB0F, 0x1FB10, 0x1FB11, 0x1FB12,
    0x1FB13, 0x0258C, 0x1FB14, 0x1FB15, 0x1FB16, 0x1FB17, 0x1FB18, 0x1FB19, 0x1FB1A, 0x1FB1B,
    0x1FB1C, 0x1FB1D, 0x1FB1E, 0x1FB1F, 0x1FB20, 0x1FB21, 0x1FB22, 0x1FB23, 0x1FB24, 0x1FB25,
    0x1FB26, 0x1FB27, 0x02590, 0x1FB28, 0x1FB29, 0x1FB2A, 0x1FB2B, 0x1FB2C, 0x1FB2D, 0x1FB2E,
    0x1FB2F, 0x1FB30, 0x1FB31, 0x1FB32, 0x1FB33, 0x1FB34, 0x1FB35, 0x1FB36, 0x1FB37, 0x1FB38,
    0x1FB39, 0x1FB3A, 0x1FB3B, 0x02588,
];
