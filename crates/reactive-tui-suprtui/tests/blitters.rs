//! blitters mechanism: BLT-001 and BLT-002 (docs/spec/blitters.md).
//!
//! Every block's split is checked against every other split of the same
//! pixels, transparent pixels must leave the background transparent, each
//! tier's glyphs must cover its patterns inside its Unicode range, and the
//! tier choice must follow the capability report, the host table and the
//! overrides.

use suprtui::blit::{self, Blitter};
use suprtui::buffer::draw::QUADRANT_CHARS;

const BLOCK_TIERS: [Blitter; 5] = [
    Blitter::Octant,
    Blitter::Sextant,
    Blitter::Quadrant,
    Blitter::HalfBlock,
    Blitter::Ascii,
];

fn count(blitter: Blitter) -> usize {
    let (columns, rows) = blitter.cell_pixels();
    (columns * rows) as usize
}

/// Squared error of a split: each pixel's squared distance to its set's
/// exact mean, summed. Exact in `f64` for eight 8-bit pixels.
fn split_error(block: &[[u8; 4]], pattern: u8) -> f64 {
    let mut error = 0.0;
    for set in [true, false] {
        let members: Vec<_> = block
            .iter()
            .enumerate()
            .filter(|(i, _)| (pattern & (1 << i) != 0) == set)
            .map(|(_, pixel)| pixel)
            .collect();
        if members.is_empty() {
            continue;
        }
        for channel in 0..3 {
            let mean =
                members.iter().map(|p| f64::from(p[channel])).sum::<f64>() / members.len() as f64;
            error += members
                .iter()
                .map(|p| (f64::from(p[channel]) - mean).powi(2))
                .sum::<f64>();
        }
    }
    error
}

fn rounded_mean(block: &[[u8; 4]], pattern: u8) -> Option<[u8; 3]> {
    let members: Vec<_> = block
        .iter()
        .enumerate()
        .filter(|(i, _)| pattern & (1 << i) != 0)
        .map(|(_, pixel)| pixel)
        .collect();
    let n = members.len() as u32;
    (n > 0).then(|| {
        [0, 1, 2].map(|c| {
            let sum: u32 = members.iter().map(|p| u32::from(p[c])).sum();
            ((sum + n / 2) / n) as u8
        })
    })
}

/// A fixed pseudo-random sequence, so every run checks the same blocks.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u8 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 56) as u8
    }
}

/// Fixed blocks for a tier: uniform, two-tone, gradients, one odd pixel,
/// near ties and random pixels, all opaque.
fn fixed_blocks(n: usize) -> Vec<Vec<[u8; 4]>> {
    let mut blocks = vec![
        vec![[40, 80, 120, 255]; n],
        (0..n)
            .map(|i| {
                if i % 2 == 0 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                }
            })
            .collect(),
        (0..n)
            .map(|i| {
                let v = (i * 255 / n.max(2).saturating_sub(1).max(1)) as u8;
                [v, v, v, 255]
            })
            .collect(),
        (0..n)
            .map(|i| {
                if i == n - 1 {
                    [250, 250, 10, 255]
                } else {
                    [10, 10, 10, 255]
                }
            })
            .collect(),
        (0..n)
            .map(|i| [100 + (i % 2) as u8, 100, 100, 255])
            .collect(),
    ];
    let mut random = Lcg(0x5EED ^ n as u64);
    for _ in 0..300 {
        blocks.push(
            (0..n)
                .map(|_| [random.next(), random.next(), random.next(), 255])
                .collect(),
        );
    }
    blocks
}

/// BLT-001: every opaque block takes the split with the least total error,
/// found by trying every split, with the rounded means as its colors.
#[test]
fn blt_001_every_block_takes_the_split_with_the_least_error() {
    for blitter in Blitter::TIERS {
        let n = count(blitter);
        for block in fixed_blocks(n) {
            let cell = blit::blit_block(blitter, &block);
            let chosen = split_error(&block, cell.pattern);
            let least = (0..1u16 << n)
                .map(|pattern| split_error(&block, pattern as u8))
                .fold(f64::INFINITY, f64::min);
            assert!(
                chosen <= least + 1e-6,
                "{blitter:?} {block:?}: chose {:#b} with error {chosen}, a split has {least}",
                cell.pattern
            );
            assert!(
                u16::from(cell.pattern) < 1 << n,
                "{blitter:?}: pattern past the cell"
            );
            assert_eq!(cell.glyph, blitter.glyph(cell.pattern), "{blitter:?}");
            assert_eq!(
                cell.fg,
                rounded_mean(&block, cell.pattern),
                "{blitter:?} fg"
            );
            let clear = ((1u16 << n) - 1) as u8 & !cell.pattern;
            assert_eq!(cell.bg, rounded_mean(&block, clear), "{blitter:?} bg");
            assert!(
                cell.bg.is_some(),
                "{blitter:?}: an opaque block has a background"
            );
        }
    }
}

/// BLT-001: a two-colored block splits along its colors: left red and
/// right blue draws the right half in blue on red.
#[test]
fn blt_001_two_colors_split_along_the_edge_between_them() {
    let red = [255, 0, 0, 255];
    let blue = [0, 0, 255, 255];
    for (blitter, right_half) in [
        (Blitter::Octant, '\u{2590}'),
        (Blitter::Braille, '\u{28B8}'),
        (Blitter::Sextant, '\u{2590}'),
        (Blitter::Quadrant, '\u{2590}'),
    ] {
        let block: Vec<_> = (0..count(blitter))
            .map(|i| if i % 2 == 0 { red } else { blue })
            .collect();
        let cell = blit::blit_block(blitter, &block);
        assert_eq!(cell.glyph, right_half, "{blitter:?}");
        assert_eq!((cell.fg, cell.bg), (Some([0, 0, 255]), Some([255, 0, 0])));
    }
    let cell = blit::blit_block(Blitter::HalfBlock, &[red, blue]);
    assert_eq!(
        (cell.glyph, cell.fg, cell.bg),
        ('\u{2584}', Some([0, 0, 255]), Some([255, 0, 0]))
    );
}

/// BLT-001: any transparent pixel leaves the background transparent; the
/// opaque pixels form the glyph in their mean color.
#[test]
fn blt_001_a_transparent_pixel_makes_the_background_transparent() {
    let mut random = Lcg(7);
    for blitter in Blitter::TIERS {
        let n = count(blitter);
        for hole in 0..n {
            let block: Vec<[u8; 4]> = (0..n)
                .map(|i| {
                    let alpha = if i == hole {
                        random.next() % blit::OPAQUE_ALPHA
                    } else {
                        255
                    };
                    [random.next(), random.next(), random.next(), alpha]
                })
                .collect();
            let cell = blit::blit_block(blitter, &block);
            assert_eq!(cell.bg, None, "{blitter:?} hole {hole}: opaque background");
            let opaque = ((1u16 << n) - 1) as u8 & !(1 << hole);
            assert_eq!(cell.pattern, opaque, "{blitter:?} hole {hole}");
            assert_eq!(
                cell.fg,
                rounded_mean(&block, opaque),
                "{blitter:?} hole {hole}"
            );
            assert_eq!(cell.glyph, blitter.glyph(opaque));
        }
        let clear = vec![[9, 9, 9, 0]; n];
        let cell = blit::blit_block(blitter, &clear);
        assert_eq!(
            (cell.glyph, cell.fg, cell.bg),
            (' ', None, None),
            "{blitter:?}"
        );
    }
    // Past the image's edge is transparent too.
    let image = image::RgbaImage::from_pixel(3, 5, image::Rgba([200, 10, 10, 255]));
    let grid = blit::blit_image(Blitter::Sextant, &image);
    assert_eq!((grid.columns, grid.rows), (2, 2));
    assert!(grid.cells[0].bg.is_some(), "a whole opaque block");
    for edge in [1, 2, 3] {
        assert_eq!(
            grid.cells[edge].bg, None,
            "cell {edge} reaches past the edge"
        );
    }
}

/// BLT-001: a grid cell is the block of pixels under it.
#[test]
fn blt_001_an_image_grid_blits_the_block_under_each_cell() {
    let mut random = Lcg(11);
    let image = image::RgbaImage::from_fn(6, 8, |_, _| {
        image::Rgba([random.next(), random.next(), random.next(), 255])
    });
    for blitter in Blitter::TIERS {
        let (w, h) = blitter.cell_pixels();
        let grid = blit::blit_image(blitter, &image);
        assert_eq!(
            (grid.columns, grid.rows),
            (6u32.div_ceil(w), 8u32.div_ceil(h))
        );
        assert_eq!(grid.cells.len(), (grid.columns * grid.rows) as usize);
        for row in 0..grid.rows {
            for column in 0..grid.columns {
                let block: Vec<_> = (0..h)
                    .flat_map(|dy| (0..w).map(move |dx| (column * w + dx, row * h + dy)))
                    .map(|(x, y)| {
                        if x < 6 && y < 8 {
                            image.get_pixel(x, y).0
                        } else {
                            [0; 4]
                        }
                    })
                    .collect();
                assert_eq!(
                    grid.cells[(row * grid.columns + column) as usize],
                    blit::blit_block(blitter, &block),
                    "{blitter:?} cell ({column}, {row})"
                );
            }
        }
    }
}

/// BLT-001: each tier draws every pattern with a distinct glyph from its
/// own range; the empty pattern is a space and the full one a full block.
#[test]
fn blt_001_each_tier_draws_every_pattern_from_its_range() {
    let blocks = 0x2580..=0x259F;
    for blitter in BLOCK_TIERS.into_iter().chain([Blitter::Braille]) {
        let n = count(blitter);
        let glyphs: Vec<char> = (0..1u16 << n).map(|p| blitter.glyph(p as u8)).collect();
        let distinct: std::collections::HashSet<_> = glyphs.iter().collect();
        if blitter == Blitter::Ascii {
            // One pixel per cell, always drawn as the background: a space.
            assert_eq!(distinct.len(), 1, "Ascii draws only spaces");
        } else {
            assert_eq!(
                distinct.len(),
                glyphs.len(),
                "{blitter:?}: two patterns share a glyph"
            );
        }
        assert_eq!(glyphs[0], ' ', "{blitter:?}: empty pattern");
        if blitter != Blitter::Ascii && blitter != Blitter::Braille {
            assert_eq!(
                *glyphs.last().unwrap(),
                '\u{2588}',
                "{blitter:?}: full pattern"
            );
        }
        for (pattern, glyph) in glyphs.iter().enumerate().skip(1) {
            let code = u32::from(*glyph);
            let in_range = match blitter {
                Blitter::Braille => (0x2801..=0x28FF).contains(&code),
                Blitter::Octant => {
                    (0x1CD00..=0x1CDE5).contains(&code)
                        || blocks.contains(&code)
                        || [
                            0x1CEA0, 0x1CEA3, 0x1CEA8, 0x1CEAB, 0x1FB82, 0x1FB85, 0x1FBE6, 0x1FBE7,
                        ]
                        .contains(&code)
                }
                Blitter::Sextant => (0x1FB00..=0x1FB3B).contains(&code) || blocks.contains(&code),
                Blitter::Quadrant | Blitter::HalfBlock => blocks.contains(&code),
                Blitter::Ascii => code == 0x20,
            };
            assert!(in_range, "{blitter:?} pattern {pattern:#b}: U+{code:04X}");
        }
    }
    // Code points named for their pattern (Unicode 16 names).
    for (blitter, pattern, code) in [
        (Blitter::Octant, 0b0000_0100, 0x1CD00), // BLOCK OCTANT-3
        (Blitter::Octant, 0b1111_1110, 0x1CDE5), // BLOCK OCTANT-2345678
        (Blitter::Octant, 0b0000_0001, 0x1CEA8), // LEFT HALF UPPER ONE QUARTER BLOCK
        (Blitter::Octant, 0b1100_0000, 0x2582),  // LOWER ONE QUARTER BLOCK
        (Blitter::Octant, 0b0101_0101, 0x258C),  // LEFT HALF BLOCK
        (Blitter::Sextant, 0b00_0001, 0x1FB00),  // BLOCK SEXTANT-1
        (Blitter::Sextant, 0b11_1110, 0x1FB3B),  // BLOCK SEXTANT-23456
        (Blitter::Sextant, 0b10_1010, 0x2590),   // RIGHT HALF BLOCK
        (Blitter::Quadrant, 0b1000, 0x2597),     // QUADRANT LOWER RIGHT
        (Blitter::Quadrant, 0b0110, 0x259E),     // QUADRANT UPPER RIGHT AND LOWER LEFT
        (Blitter::HalfBlock, 0b01, 0x2580),      // UPPER HALF BLOCK
        (Blitter::Braille, 0b0000_0001, 0x2801), // BRAILLE PATTERN DOTS-1
        (Blitter::Braille, 0b0000_0010, 0x2808), // BRAILLE PATTERN DOTS-4
        (Blitter::Braille, 0b0100_0000, 0x2840), // BRAILLE PATTERN DOTS-7
        (Blitter::Braille, 0b1000_0000, 0x2880), // BRAILLE PATTERN DOTS-8
    ] {
        assert_eq!(
            u32::from(blitter.glyph(pattern)),
            code,
            "{blitter:?} {pattern:#b}"
        );
    }
    assert_eq!(Blitter::Ascii.glyph(1), ' ');
}

/// BLT-002: a host that reports no unicode gets ASCII, whatever its
/// identity.
#[test]
fn blt_002_no_unicode_gets_ascii() {
    for identity in [None, Some("kitty"), Some("linux"), Some("unknown-term")] {
        assert_eq!(
            blit::choose(false, identity, None, None),
            Blitter::Ascii,
            "{identity:?}"
        );
    }
}

/// BLT-002: a host in the table gets its entry, ignoring case; any other
/// host, or none, gets sextant.
#[test]
fn blt_002_table_hosts_get_their_entry_and_others_sextant() {
    for (identity, tier) in [
        ("kitty", Blitter::Octant),
        ("xterm-kitty", Blitter::Octant),
        ("ghostty", Blitter::Octant),
        ("xterm-ghostty", Blitter::Octant),
        ("foot", Blitter::Octant),
        ("foot-extra", Blitter::Octant),
        ("Apple_Terminal", Blitter::Quadrant),
        ("vscode", Blitter::Quadrant),
        ("linux", Blitter::HalfBlock),
    ] {
        assert_eq!(blit::table_tier(identity), Some(tier), "{identity}");
        assert_eq!(
            blit::choose(true, Some(identity), None, None),
            tier,
            "{identity}"
        );
        let shouted = identity.to_ascii_uppercase();
        assert_eq!(
            blit::choose(true, Some(&shouted), None, None),
            tier,
            "{shouted}"
        );
    }
    for identity in [None, Some("xterm-256color"), Some("tmux"), Some("")] {
        assert_eq!(
            blit::choose(true, identity, None, None),
            Blitter::Sextant,
            "{identity:?}"
        );
    }
}

/// BLT-002: an application or environment override replaces the choice,
/// including the ASCII rule and the table; the environment's wins.
#[test]
fn blt_002_overrides_replace_the_choice() {
    for tier in Blitter::TIERS {
        for (unicode, identity) in [(false, None), (true, Some("kitty")), (true, None)] {
            assert_eq!(blit::choose(unicode, identity, Some(tier), None), tier);
            assert_eq!(blit::choose(unicode, identity, None, Some(tier)), tier);
        }
    }
    assert_eq!(
        blit::choose(
            true,
            Some("kitty"),
            Some(Blitter::Braille),
            Some(Blitter::HalfBlock)
        ),
        Blitter::HalfBlock
    );
    for tier in Blitter::TIERS {
        assert_eq!(Blitter::from_name(tier.name()), Some(tier));
        assert_eq!(
            blit::parse_override(&format!(" {} ", tier.name().to_uppercase())),
            Ok(Some(tier))
        );
    }
    assert_eq!(blit::parse_override("  "), Ok(None));
    let error = blit::parse_override("hexagon").unwrap_err();
    assert!(
        error.contains("hexagon") && error.contains(blit::BLITTER_ENV),
        "{error}"
    );
}

/// BLT-001: the quadrant fallback for packed image cells draws the quadrant
/// blitter's glyph for the same four pixels.
#[test]
fn blt_001_packed_image_cells_use_the_quadrant_glyphs() {
    for nibble in 0..16u8 {
        // Nibble bits: 8 upper left, 4 upper right, 2 lower left, 1 lower right.
        let pattern = (nibble >> 3 & 1) | (nibble >> 1 & 2) | (nibble << 1 & 4) | (nibble << 3 & 8);
        assert_eq!(
            QUADRANT_CHARS[usize::from(nibble)],
            u32::from(Blitter::Quadrant.glyph(pattern)),
            "nibble {nibble:04b}"
        );
    }
}
