//! Bounded decoding of one Sixel file into its pixel grid.
//! Reference: Xterm control sequences, Graphics / Sixel Graphics, and DEC VT340.
use super::{decoded, image_error};
use crate::error::Result;
use image::{Rgba, RgbaImage};

const MAX_INPUT: usize = 64 * 1024 * 1024;
const MAX_WORK: u64 = 64 * 1024 * 1024;

pub(super) fn decode(input: &[u8]) -> Result<RgbaImage> {
    if input.len() > MAX_INPUT {
        return Err(image_error("Encoded Sixel exceeds the 64 MiB limit"));
    }
    let input = input.trim_ascii();
    let input = input
        .strip_prefix(b"\x1bP")
        .or_else(|| input.strip_prefix(b"\x90"))
        .ok_or_else(|| image_error("Sixel file must start with DCS"))?;
    let (prefix, data) = input.split_at(
        input
            .iter()
            .position(|c| *c == b'q')
            .ok_or_else(|| image_error("Sixel file has no data introducer"))?,
    );
    let mut pos = 0;
    let args = parameters(prefix, &mut pos)?;
    if pos != prefix.len() || args.len() > 3 {
        return Err(image_error("Invalid Sixel DCS parameters"));
    }
    let data = data[1..]
        .strip_suffix(b"\x1b\\")
        .or_else(|| data[1..].strip_suffix(b"\x9c"))
        .ok_or_else(|| image_error("Sixel file is missing its terminator"))?;
    // Inspect every coordinate and repeat before allocating the output. A second
    // pass paints in source order, preserving overwritten colors without a command list.
    let (width, height) = scan(data, None)?;
    decoded::dimensions(width, height)?;
    let background = if args.get(1) == Some(&1) {
        [0; 4]
    } else {
        [0, 0, 0, 255]
    };
    let mut pixels = RgbaImage::from_pixel(width, height, Rgba(background));
    scan(data, Some(&mut pixels))?;
    Ok(pixels)
}

fn parameters(data: &[u8], pos: &mut usize) -> Result<Vec<u32>> {
    let mut result = Vec::new();
    loop {
        let mut value = 0u32;
        let start = *pos;
        while let Some(digit @ b'0'..=b'9') = data.get(*pos) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(u32::from(*digit - b'0')))
                .ok_or_else(|| image_error("Sixel numeric parameter overflow"))?;
            *pos += 1;
        }
        if *pos == start && result.is_empty() && data.get(*pos) != Some(&b';') {
            break;
        }
        result.push(value);
        if result.len() > 5 {
            return Err(image_error("Too many Sixel parameters"));
        }
        if data.get(*pos) != Some(&b';') {
            break;
        }
        *pos += 1;
    }
    Ok(result)
}

fn scan(data: &[u8], mut pixels: Option<&mut RgbaImage>) -> Result<(u32, u32)> {
    let mut palette = [[0, 0, 0, 255]; 1024];
    for (entry, rgb) in palette.iter_mut().zip([
        [0, 0, 0],
        [51, 51, 204],
        [204, 33, 33],
        [51, 204, 51],
        [204, 51, 204],
        [51, 204, 204],
        [204, 204, 51],
        [135, 135, 135],
        [66, 66, 66],
        [84, 84, 153],
        [153, 66, 66],
        [84, 153, 84],
        [153, 84, 153],
        [84, 153, 153],
        [153, 153, 84],
        [204, 204, 204],
    ]) {
        *entry = [rgb[0], rgb[1], rgb[2], 255];
    }
    let (mut pos, mut color) = (0, 0);
    let (mut x, mut y, mut width, mut height) = (0u32, 0u32, 0u32, 0u32);
    let mut work = 0u64;
    while let Some(command) = data.get(pos).copied() {
        pos += 1;
        match command {
            b'#' => {
                let args = parameters(data, &mut pos)?;
                color = *args
                    .first()
                    .ok_or_else(|| image_error("Sixel color register is missing"))?
                    as usize;
                if color >= palette.len() {
                    return Err(image_error("Sixel color register exceeds 1023"));
                }
                if args.len() > 1 {
                    if args.len() != 5 {
                        return Err(image_error("Sixel color definition needs five parameters"));
                    }
                    let rgb = match args[1] {
                        2 if args[2..].iter().all(|v| *v <= 100) => {
                            [percent(args[2]), percent(args[3]), percent(args[4])]
                        }
                        1 if args[2] <= 360 && args[3] <= 100 && args[4] <= 100 => {
                            hls(args[2], args[3], args[4])
                        }
                        _ => return Err(image_error("Invalid Sixel color space or components")),
                    };
                    palette[color] = [rgb[0], rgb[1], rgb[2], 255];
                }
            }
            b'"' => {
                let args = parameters(data, &mut pos)?;
                if args.len() != 2 && args.len() != 4 {
                    return Err(image_error("Invalid Sixel raster attributes"));
                }
                if args.len() == 4 {
                    width = width.max(args[2]);
                    height = height.max(args[3]);
                    decoded::dimensions(width.max(1), height.max(1))?;
                }
            }
            b'$' => x = 0,
            b'-' => {
                x = 0;
                y = y
                    .checked_add(6)
                    .ok_or_else(|| image_error("Sixel row overflow"))?;
            }
            b'!' | b'?'..=b'~' => {
                let (count, bits) = if command == b'!' {
                    let args = parameters(data, &mut pos)?;
                    if args.len() != 1 {
                        return Err(image_error("Sixel repeat needs one count"));
                    }
                    let byte = *data
                        .get(pos)
                        .filter(|c| (b'?'..=b'~').contains(c))
                        .ok_or_else(|| image_error("Sixel repeat has no pixel data"))?;
                    pos += 1;
                    (args[0].max(1), byte - b'?')
                } else {
                    (1, command - b'?')
                };
                work += u64::from(count) * 6;
                if work > MAX_WORK {
                    return Err(image_error(
                        "Sixel exceeds the 64 million pixel-operation limit",
                    ));
                }
                let right = x
                    .checked_add(count)
                    .ok_or_else(|| image_error("Sixel column overflow"))?;
                width = width.max(right);
                let band = (8 - bits.leading_zeros()).max(1);
                height = height.max(
                    y.checked_add(band)
                        .ok_or_else(|| image_error("Sixel row overflow"))?,
                );
                decoded::dimensions(width, height)?;
                if let Some(image) = pixels.as_deref_mut() {
                    for dy in 0..6 {
                        if bits & (1 << dy) != 0 {
                            for px in x..right {
                                image.put_pixel(px, y + dy, Rgba(palette[color]));
                            }
                        }
                    }
                }
                x = right;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {}
            _ => return Err(image_error("Invalid byte in Sixel image data")),
        }
    }
    Ok((width, height))
}

fn percent(value: u32) -> u8 {
    ((value * 255 + 50) / 100) as u8
}

fn hls(hue: u32, light: u32, saturation: u32) -> [u8; 3] {
    // DEC hue starts at blue; conventional HSL starts at red.
    let hue = ((hue + 240) % 360) as f64 / 60.0;
    let light = light as f64 / 100.0;
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * saturation as f64 / 100.0;
    let second = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
    let rgb = match hue as u8 {
        0 => [chroma, second, 0.0],
        1 => [second, chroma, 0.0],
        2 => [0.0, chroma, second],
        3 => [0.0, second, chroma],
        4 => [second, 0.0, chroma],
        _ => [chroma, 0.0, second],
    };
    rgb.map(|v| {
        ((v + light - chroma / 2.0) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}
