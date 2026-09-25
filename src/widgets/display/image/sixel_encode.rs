//! Sixel palette encoding with owned, bounded buffers and no terminal I/O.
use super::{decoded, ImageQuality};
use crate::error::{ReactiveError, Result};
use image::RgbaImage;
use std::collections::HashMap;
use std::fmt::Write;

const MAX_OUTPUT: usize = 64 * 1024 * 1024;
const MAX_SCRATCH: usize = 32 * 1024 * 1024;

fn error(message: &str) -> ReactiveError {
    ReactiveError::ImageProcessing(message.into())
}

pub(super) fn encode(pixels: &RgbaImage, quality: ImageQuality) -> Result<String> {
    let (width, height) = pixels.dimensions();
    decoded::dimensions(width, height)?;
    let width = width as usize;
    // A six-row band has at most six differently colored entries per column.
    if width
        .checked_mul(6 * std::mem::size_of::<(u32, u8)>())
        .is_none_or(|n| n > MAX_SCRATCH)
    {
        return Err(error("Sixel row exceeds the 32 MiB workspace limit"));
    }
    let palette = Palette::new(pixels);
    let indices = palette.indices(pixels, quality);
    let mut output = format!("\x1bP0;1q\"1;1;{width};{height}");
    for (index, [r, g, b]) in palette.colors.iter().enumerate() {
        let percent = |channel: &u8| (u32::from(*channel) * 100 + 127) / 255;
        write!(
            output,
            "#{index};2;{};{};{}",
            percent(r),
            percent(g),
            percent(b)
        )
        .expect("String write");
    }
    for row in (0..height as usize).step_by(6) {
        if row != 0 {
            output.push('-');
        }
        let mut bands = vec![Vec::<(u32, u8)>::new(); palette.colors.len()];
        for x in 0..width {
            let mut column = [(0u8, 0u8); 6];
            let mut used = 0;
            for dy in 0..6.min(height as usize - row) {
                if pixels.get_pixel(x as u32, (row + dy) as u32)[3] == 0 {
                    continue;
                }
                let index = indices[(row + dy) * width + x];
                if let Some((_, bits)) = column[..used].iter_mut().find(|(i, _)| *i == index) {
                    *bits |= 1 << dy;
                } else {
                    column[used] = (index, 1 << dy);
                    used += 1;
                }
            }
            for (index, bits) in &column[..used] {
                bands[usize::from(*index)].push((x as u32, *bits));
            }
        }
        let mut first = true;
        for (index, band) in bands
            .iter()
            .enumerate()
            .filter(|(_, band)| !band.is_empty())
        {
            if !first {
                output.push('$');
            }
            first = false;
            write!(output, "#{index}").expect("String write");
            let mut column = 0;
            let mut pos = 0;
            while let Some(&(x, bits)) = band.get(pos) {
                if x > column {
                    append_run(&mut output, b'?', x - column);
                }
                let mut count = 1;
                while band.get(pos + count as usize) == Some(&(x + count, bits)) {
                    count += 1;
                }
                append_run(&mut output, b'?' + bits, count);
                column = x + count;
                pos += count as usize;
                if output.len() > MAX_OUTPUT {
                    return Err(error("Sixel output exceeds the 64 MiB limit"));
                }
            }
        }
    }
    output.push_str("\x1b\\");
    Ok(output)
}

fn append_run(output: &mut String, byte: u8, count: u32) {
    if count > 3 {
        write!(output, "!{count}{}", char::from(byte)).expect("String write");
    } else {
        output.extend(std::iter::repeat_n(char::from(byte), count as usize));
    }
}

struct Palette {
    colors: Vec<[u8; 3]>,
    exact: HashMap<[u8; 3], u8>,
}
impl Palette {
    fn new(pixels: &RgbaImage) -> Self {
        let mut palette = Self {
            colors: Vec::new(),
            exact: HashMap::new(),
        };
        for pixel in pixels.pixels().filter(|p| p[3] != 0) {
            let rgb = [pixel[0], pixel[1], pixel[2]];
            if !palette.exact.contains_key(&rgb) {
                if palette.colors.len() == 256 {
                    return Self::quantized();
                }
                palette.exact.insert(rgb, palette.colors.len() as u8);
                palette.colors.push(rgb);
            }
        }
        palette
    }
    fn quantized() -> Self {
        let mut colors = Vec::with_capacity(256);
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    colors.push([r * 51, g * 51, b * 51]);
                }
            }
        }
        for gray in 0..40 {
            colors.push([((gray * 255 + 19) / 39) as u8; 3]);
        }
        Self {
            colors,
            exact: HashMap::new(),
        }
    }
    fn nearest(&self, rgb: [u8; 3]) -> u8 {
        let cube = rgb.map(|channel| ((u16::from(channel) + 25) / 51).min(5) as usize);
        let cube = cube[0] * 36 + cube[1] * 6 + cube[2];
        let sum: u32 = rgb.iter().map(|c| u32::from(*c)).sum();
        let gray = 216 + ((sum * 39 + 382) / 765) as usize;
        let distance = |index: usize| {
            self.colors[index]
                .iter()
                .zip(rgb)
                .map(|(a, b)| {
                    let diff = i32::from(*a) - i32::from(b);
                    diff * diff
                })
                .sum::<i32>()
        };
        if distance(gray) < distance(cube) {
            gray as u8
        } else {
            cube as u8
        }
    }
    fn indices(&self, pixels: &RgbaImage, quality: ImageQuality) -> Vec<u8> {
        let width = pixels.width() as usize;
        let mut indices = Vec::with_capacity(width * pixels.height() as usize);
        if !self.exact.is_empty() || self.colors.is_empty() {
            for pixel in pixels.pixels() {
                indices.push(
                    *self
                        .exact
                        .get(&[pixel[0], pixel[1], pixel[2]])
                        .unwrap_or(&0),
                );
            }
            return indices;
        }
        let weights: &[(isize, usize, f32)] = match quality {
            ImageQuality::Fast => &[],
            ImageQuality::Balanced => &[
                (1, 0, 0.125),
                (2, 0, 0.125),
                (-1, 1, 0.125),
                (0, 1, 0.125),
                (1, 1, 0.125),
                (0, 2, 0.125),
            ],
            ImageQuality::High => &[
                (1, 0, 8.0 / 42.0),
                (2, 0, 4.0 / 42.0),
                (-2, 1, 2.0 / 42.0),
                (-1, 1, 4.0 / 42.0),
                (0, 1, 8.0 / 42.0),
                (1, 1, 4.0 / 42.0),
                (2, 1, 2.0 / 42.0),
                (-2, 2, 1.0 / 42.0),
                (-1, 2, 2.0 / 42.0),
                (0, 2, 4.0 / 42.0),
                (1, 2, 2.0 / 42.0),
                (2, 2, 1.0 / 42.0),
            ],
        };
        let mut errors = if weights.is_empty() {
            Vec::new()
        } else {
            vec![[0.0f32; 3]; width * 3]
        };
        for y in 0..pixels.height() as usize {
            for x in 0..width {
                let pixel = pixels.get_pixel(x as u32, y as u32);
                let slot = (y % 3) * width + x;
                let rgb = std::array::from_fn(|c| {
                    (f32::from(pixel[c]) + errors.get(slot).map_or(0.0, |e| e[c]))
                        .round()
                        .clamp(0.0, 255.0) as u8
                });
                let index = self.nearest(rgb);
                indices.push(index);
                if !weights.is_empty() {
                    errors[slot] = [0.0; 3];
                }
                if pixel[3] == 0 {
                    continue;
                }
                for &(dx, dy, weight) in weights {
                    let Some(next_x) = x.checked_add_signed(dx).filter(|x| *x < width) else {
                        continue;
                    };
                    let next = ((y + dy) % 3) * width + next_x;
                    for c in 0..3 {
                        errors[next][c] += (f32::from(rgb[c])
                            - f32::from(self.colors[usize::from(index)][c]))
                            * weight;
                    }
                }
            }
        }
        indices
    }
}
