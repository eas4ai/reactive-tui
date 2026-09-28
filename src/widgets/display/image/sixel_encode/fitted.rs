//! A palette fitted to one picture, by median cut: the picture's colors,
//! counted in cells of 8 by 8 by 8, are split again and again along their
//! longest side at the middle of their pixels until there are as many
//! groups as Sixel color registers. Each group's mean is a palette color.

use image::RgbaImage;

/// Color registers a Sixel picture uses.
const COLORS: usize = 256;
/// Cells along each channel, and in all.
const SIDE: usize = 32;
const CELLS: usize = SIDE * SIDE * SIDE;

/// The threshold of each pixel of an 8 by 8 tile, 0 to 63, spread so that
/// neighbours differ most (Bayer's ordered dither).
const THRESHOLDS: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

/// The colors of one cell: how many pixels and the sum of each channel.
#[derive(Clone, Copy, Default)]
struct Cell {
    count: u64,
    sums: [u64; 3],
}

fn cell_of([r, g, b]: [u8; 3]) -> usize {
    (usize::from(r) >> 3) * SIDE * SIDE + (usize::from(g) >> 3) * SIDE + (usize::from(b) >> 3)
}

/// The place of cell `index` along `channel`, 0 to 31.
fn place(index: usize, channel: usize) -> usize {
    (index / SIDE.pow(2 - channel as u32)) % SIDE
}

/// The cells of `group` span this far along each channel.
fn spans(group: &[usize]) -> [usize; 3] {
    [0, 1, 2].map(|channel| {
        let places = group.iter().map(|&index| place(index, channel));
        let (low, high) = places.fold((SIDE, 0), |(low, high), at| (low.min(at), high.max(at)));
        high.saturating_sub(low)
    })
}

/// The palette of `pixels` and the palette index of each pixel, row by row.
/// A pixel with no alpha takes index 0 and no part in the palette.
pub(super) fn palette(pixels: &RgbaImage) -> (Vec<[u8; 3]>, Vec<u8>) {
    let mut cells = vec![Cell::default(); CELLS];
    for pixel in pixels.pixels().filter(|pixel| pixel[3] != 0) {
        let cell = &mut cells[cell_of([pixel[0], pixel[1], pixel[2]])];
        cell.count += 1;
        for (sum, channel) in cell.sums.iter_mut().zip(&pixel.0[..3]) {
            *sum += u64::from(*channel);
        }
    }
    let used: Vec<usize> = (0..CELLS).filter(|&index| cells[index].count > 0).collect();
    let mut groups: Vec<Vec<usize>> = vec![used];
    while groups.len() < COLORS {
        // The group to split: the one whose colors lie furthest apart,
        // weighed by its pixels, among those with more than one cell.
        let widest = groups
            .iter()
            .enumerate()
            .filter(|(_, group)| group.len() > 1)
            .max_by_key(|(_, group)| {
                let pixels: u64 = group.iter().map(|&index| cells[index].count).sum();
                let span = spans(group).into_iter().max().unwrap_or(0) as u64;
                (span * span) * pixels.min(1 << 20)
            })
            .map(|(at, _)| at);
        let Some(at) = widest else {
            break;
        };
        let mut group = std::mem::take(&mut groups[at]);
        let span = spans(&group);
        let channel = (0..3).max_by_key(|&channel| span[channel]).unwrap_or(0);
        group.sort_unstable_by_key(|&index| place(index, channel));
        let pixels: u64 = group.iter().map(|&index| cells[index].count).sum();
        let mut seen = 0;
        let mut cut = 1;
        for (position, &index) in group.iter().enumerate() {
            seen += cells[index].count;
            if seen * 2 >= pixels {
                cut = (position + 1).clamp(1, group.len() - 1);
                break;
            }
        }
        let upper = group.split_off(cut);
        groups[at] = group;
        groups.push(upper);
    }
    groups.retain(|group| !group.is_empty());
    let colors: Vec<[u8; 3]> = groups
        .iter()
        .map(|group| {
            let count: u64 = group.iter().map(|&index| cells[index].count).sum();
            [0, 1, 2].map(|channel| {
                let sum: u64 = group.iter().map(|&index| cells[index].sums[channel]).sum();
                ((sum + count / 2) / count.max(1)) as u8
            })
        })
        .collect();
    // The palette index of each cell: its group's, or for a cell the
    // dither reaches that no pixel lies in, the nearest color's, found
    // when it is first needed.
    const UNKNOWN: u16 = u16::MAX;
    let mut index_of = vec![UNKNOWN; CELLS];
    for (index, group) in groups.iter().enumerate() {
        for &cell in group {
            index_of[cell] = index as u16;
        }
    }
    let nearest = |rgb: [u8; 3]| -> u16 {
        colors
            .iter()
            .enumerate()
            .min_by_key(|(_, color)| {
                color
                    .iter()
                    .zip(rgb)
                    .map(|(a, b)| (i32::from(*a) - i32::from(b)).pow(2))
                    .sum::<i32>()
            })
            .map_or(0, |(index, _)| index as u16)
    };
    let mut indices = Vec::with_capacity(pixels.width() as usize * pixels.height() as usize);
    for (x, y, pixel) in pixels.enumerate_pixels() {
        if pixel[3] == 0 || colors.is_empty() {
            indices.push(0);
            continue;
        }
        // Up to half a cell either way, by the pixel's place in the tile.
        let nudge = i16::from(THRESHOLDS[(y & 7) as usize][(x & 7) as usize]) / 8 - 4;
        let rgb = [0, 1, 2].map(|channel| (i16::from(pixel[channel]) + nudge).clamp(0, 255) as u8);
        let cell = cell_of(rgb);
        if index_of[cell] == UNKNOWN {
            index_of[cell] = nearest(rgb);
        }
        indices.push(index_of[cell] as u8);
    }
    (colors, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_of_few_colors_keeps_each_of_them() {
        let pixels = RgbaImage::from_fn(4, 2, |x, _| {
            image::Rgba(if x < 2 {
                [200, 40, 40, 255]
            } else {
                [40, 40, 200, 255]
            })
        });
        let (colors, indices) = palette(&pixels);
        assert_eq!(colors.len(), 2);
        assert!(colors.contains(&[200, 40, 40]) && colors.contains(&[40, 40, 200]));
        assert_eq!(colors[usize::from(indices[0])], [200, 40, 40]);
        assert_eq!(colors[usize::from(indices[3])], [40, 40, 200]);
    }

    #[test]
    fn a_gradient_of_many_colors_fits_the_registers_and_stays_close() {
        // A dark blue gradient: 600 colors that a fixed color cube draws
        // as three or four grays.
        let pixels = RgbaImage::from_fn(600, 8, |x, _| {
            let t = x as f32 / 599.0;
            image::Rgba([
                (8.0 + 30.0 * t) as u8,
                (8.0 + 36.0 * t) as u8,
                (14.0 + 60.0 * t) as u8,
                255,
            ])
        });
        let (colors, indices) = palette(&pixels);
        assert!(colors.len() <= COLORS, "{} colors", colors.len());
        assert_eq!(indices.len(), 600 * 8);
        let worst = pixels
            .pixels()
            .zip(&indices)
            .map(|(pixel, index)| {
                let color = colors[usize::from(*index)];
                (0..3)
                    .map(|c| (i32::from(pixel[c]) - i32::from(color[c])).abs())
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);
        assert!(worst <= 12, "a pixel is {worst} of 255 from its color");
        let distinct: std::collections::HashSet<u8> = indices.iter().copied().collect();
        assert!(distinct.len() > 16, "only {} colors drawn", distinct.len());
    }

    #[test]
    fn a_pixel_with_no_alpha_takes_no_part() {
        let pixels = RgbaImage::from_fn(2, 1, |x, _| {
            image::Rgba(if x == 0 {
                [255, 255, 255, 0]
            } else {
                [10, 20, 30, 255]
            })
        });
        let (colors, indices) = palette(&pixels);
        assert_eq!(colors, vec![[10, 20, 30]]);
        assert_eq!(indices, vec![0, 0]);
    }
}
