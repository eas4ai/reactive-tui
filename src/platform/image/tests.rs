use super::*;
use ::base64::Engine;

fn encoded(format: image::ImageFormat) -> Vec<u8> {
    let mut data = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_raw(2, 1, vec![255, 0, 0, 0, 0, 255])
        .unwrap()
        .write_to(&mut data, format)
        .unwrap();
    data.into_inner()
}

fn kitty(image: &Image, options: &DrawOptions) -> (String, image::RgbaImage) {
    let mut output = Vec::new();
    image.render_kitty(&mut output, options).unwrap();
    let output = String::from_utf8(output).unwrap();
    let commands: Vec<_> = output
        .split("\x1b_G")
        .skip(1)
        .map(|part| {
            part.split_once("\x1b\\")
                .unwrap()
                .0
                .split_once(';')
                .unwrap()
        })
        .collect();
    assert!(
        commands[0].0.split(',').any(|part| part == "a=T"),
        "{output}"
    );
    assert!(commands
        .last()
        .unwrap()
        .0
        .split(',')
        .any(|part| part == "m=0"));
    let dimension = |name: &str| {
        commands[0]
            .0
            .split(',')
            .find_map(|part| part.strip_prefix(name))
            .unwrap()
            .parse::<u32>()
            .unwrap()
    };
    let bytes = ::base64::engine::general_purpose::STANDARD
        .decode(commands.iter().map(|(_, data)| *data).collect::<String>())
        .unwrap();
    let pixels = image::RgbaImage::from_raw(dimension("s="), dimension("v="), bytes).unwrap();
    (output, pixels)
}

#[test]
fn api_image_platform_file_and_memory_decode_actual_pixels() {
    let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    std::fs::write(file.path(), encoded(image::ImageFormat::Png)).unwrap();
    let from_file = Image::from_file(file.path().to_str().unwrap()).unwrap();
    assert_eq!((from_file.width, from_file.height), (2, 1));
    let from_memory =
        Image::from_memory(encoded(image::ImageFormat::Png), ImageFormat::Png).unwrap();
    for image in [from_file, from_memory] {
        let (_, pixels) = kitty(&image, &DrawOptions::default());
        assert_eq!(pixels.into_raw(), [255, 0, 0, 255, 0, 0, 255, 255]);
    }
    assert!(file.path().exists());
}

#[test]
fn api_image_platform_rejects_invalid_encoded_and_raw_data_without_output() {
    for (data, format) in [
        (vec![255, 216, 0, 0], ImageFormat::Jpeg),
        (b"GIF89a\x02\0\x01\0".to_vec(), ImageFormat::Gif),
        (encoded(image::ImageFormat::Png), ImageFormat::Jpeg),
    ] {
        assert!(Image::from_memory(data, format).is_err(), "{format:?}");
    }
    let invalid = Image::from_pixels(vec![0; 3], 1, 1, PixelFormat::Rgba);
    let mut bytes = Vec::new();
    assert!(invalid
        .render_kitty(&mut bytes, &DrawOptions::default())
        .is_err());
    assert!(bytes.is_empty());
}

#[test]
fn api_image_platform_iterm_uses_encoded_file_and_byte_count() {
    let image = Image::from_pixels(vec![255, 0, 0, 128], 1, 1, PixelFormat::Rgba);
    let mut output = Vec::new();
    image
        .render_iterm2(&mut output, &DrawOptions::default())
        .unwrap();
    let output = String::from_utf8(output).unwrap();
    let (header, payload) = output
        .strip_prefix("\x1b]1337;File=")
        .unwrap()
        .strip_suffix('\x07')
        .unwrap()
        .split_once(':')
        .unwrap();
    assert!(header.split(';').any(|field| field == "inline=1"));
    let bytes = ::base64::engine::general_purpose::STANDARD
        .decode(payload)
        .unwrap();
    assert!(header
        .split(';')
        .any(|field| field == format!("size={}", bytes.len())));
    assert_eq!(
        image::load_from_memory(&bytes)
            .unwrap()
            .to_rgba8()
            .into_raw(),
        [255, 0, 0, 128]
    );
}

#[test]
fn api_image_platform_kitty_crops_and_retains_placement_options() {
    let image = Image::from_pixels(vec![255, 0, 0, 0, 0, 255], 2, 1, PixelFormat::Rgb);
    let (output, pixels) = kitty(
        &image,
        &DrawOptions {
            clip_region: Some(ClipRegion {
                x: Some(1),
                y: None,
                width: Some(1),
                height: Some(1),
            }),
            pixel_offset: Some((2, 3)),
            z_index: Some(-4),
            ..Default::default()
        },
    );
    assert_eq!(pixels.dimensions(), (1, 1));
    assert_eq!(pixels.into_raw(), [0, 0, 255, 255]);
    assert!(output.contains(",X=2,Y=3"));
    assert!(output.contains(",z=-4"));
}

#[test]
fn api_image_platform_webp_and_sixel_files_preserve_pixels() {
    let webp = Image::from_memory(encoded(image::ImageFormat::WebP), ImageFormat::WebP).unwrap();
    assert_eq!(
        kitty(&webp, &DrawOptions::default()).1.into_raw(),
        [255, 0, 0, 255, 0, 0, 255, 255]
    );
    let sixel = b"\x1bP0;1q\"1;1;4;8#1;2;100;0;0!2~#2;2;0;0;100!2~-#3;1;240;50;100!2B\x1b\\";
    let file = tempfile::Builder::new()
        .suffix(".sixel")
        .tempfile()
        .unwrap();
    std::fs::write(file.path(), sixel).unwrap();
    for image in [
        Image::from_memory(sixel.to_vec(), ImageFormat::Sixel).unwrap(),
        Image::from_file(file.path().to_str().unwrap()).unwrap(),
    ] {
        assert_eq!((image.width, image.height), (4, 8));
        let pixels = kitty(&image, &DrawOptions::default()).1;
        for y in 0..8 {
            for x in 0..4 {
                let expected = match (y < 6, x < 2) {
                    (true, true) => [255, 0, 0, 255],
                    (true, false) => [0, 0, 255, 255],
                    (false, true) => [0, 255, 0, 255],
                    (false, false) => [0; 4],
                };
                assert_eq!(pixels.get_pixel(x, y).0, expected, "at {x},{y}");
            }
        }
    }
}

#[test]
fn api_image_platform_sixel_encoder_transmits_colors_and_decoded_sources() {
    for image in [
        Image::from_memory(encoded(image::ImageFormat::Png), ImageFormat::Png).unwrap(),
        Image::from_pixels(vec![255, 0, 0, 0, 0, 255], 2, 1, PixelFormat::Rgb),
    ] {
        let mut output = Vec::new();
        image
            .render_sixel(&mut output, &DrawOptions::default())
            .unwrap();
        let decoded = sixel_decode::decode(&output).unwrap();
        assert_eq!(
            decoded.dimensions(),
            (2, 1),
            "{}",
            String::from_utf8_lossy(&output).escape_debug()
        );
        assert_eq!(decoded.into_raw(), [255, 0, 0, 255, 0, 0, 255, 255]);
    }
}

#[test]
fn api_image_platform_sixel_rejects_overflow_truncation_and_non_image_controls() {
    for bytes in [
        b"\x1bPq!4294967296~\x1b\\".as_slice(),
        b"\x1bPq!64000000~\x1b\\",
        b"\x1bPq~",
        b"\x1bPq!2\x1b\\",
        b"\x1bPq#1024~\x1b\\",
        b"\x1bPq#1;2;101;0;0~\x1b\\",
        b"\x1bPq\"1;1;4294967295;4294967295~\x1b\\",
        b"\x1bPq\x1b[2J~\x1b\\",
        b"\x1bPq~\x1b\\\x1b]52;steal\x07",
    ] {
        assert!(
            Image::from_memory(bytes.to_vec(), ImageFormat::Sixel).is_err(),
            "{bytes:?}"
        );
    }
    let c1 = Image::from_memory(b"\x90q#1;1;120;50;100@\x9c".to_vec(), ImageFormat::Sixel).unwrap();
    assert_eq!(
        kitty(&c1, &DrawOptions::default()).1.into_raw(),
        [255, 0, 0, 255]
    );
}

#[test]
fn api_image_platform_empty_clipping_and_unsupported_z_order_write_nothing() {
    let image = Image::from_pixels(vec![255, 0, 0, 128], 1, 1, PixelFormat::Rgba);
    let options = DrawOptions {
        clip_region: Some(ClipRegion {
            x: Some(u32::MAX),
            y: None,
            width: None,
            height: None,
        }),
        ..Default::default()
    };
    let mut output = Vec::new();
    image.render_kitty(&mut output, &options).unwrap();
    image.render_iterm2(&mut output, &options).unwrap();
    image.render_sixel(&mut output, &options).unwrap();
    assert!(output.is_empty());
    let options = DrawOptions {
        z_index: Some(1),
        ..Default::default()
    };
    assert!(image.render_iterm2(&mut output, &options).is_err());
    assert!(image.render_sixel(&mut output, &options).is_err());
    assert!(output.is_empty());
}

#[test]
fn api_image_platform_scaling_and_pixel_offsets_use_real_raster_bounds() {
    let image = Image::from_pixels([255, 0, 0, 255].repeat(8), 4, 2, PixelFormat::Rgba);
    let mut options = DrawOptions {
        size: Some((1, 1)),
        ..Default::default()
    };
    let native = image.prepared(&options, false).unwrap().unwrap();
    assert_eq!(native.dimensions(), (4, 2));
    options.scale = ScaleMode::Contain;
    assert_eq!(
        image
            .prepared(&options, false)
            .unwrap()
            .unwrap()
            .dimensions(),
        (4, 2)
    );
    let (width, height) = drawing_box(options.size);
    options.scale = ScaleMode::Fill;
    assert_eq!(
        image
            .prepared(&options, false)
            .unwrap()
            .unwrap()
            .dimensions(),
        (width, height)
    );
    options.scale = ScaleMode::Fit;
    let fit = image.prepared(&options, false).unwrap().unwrap();
    assert!(fit.width() <= width && fit.height() <= height);
    assert_eq!(fit.width(), fit.height() * 2);
    options.scale = ScaleMode::None;
    options.pixel_offset = Some((2, 3));
    let padded = image.prepared(&options, true).unwrap().unwrap();
    assert_eq!(padded.dimensions(), (6, 5));
    assert_eq!(padded.get_pixel(0, 0).0, [0; 4]);
    assert_eq!(padded.get_pixel(2, 3).0, [255, 0, 0, 255]);
}

#[test]
fn api_image_platform_sixel_quality_handles_more_than_256_colors() {
    use crate::widgets::{Image as Widget, ImageFormat as Format, ImageQuality};
    let pixels = image::RgbaImage::from_fn(64, 24, |x, y| {
        image::Rgba([
            (x * 4) as u8,
            (y * 11) as u8,
            ((x * 7 + y * 3) % 256) as u8,
            255,
        ])
    });
    let mut outputs = Vec::new();
    for quality in [
        ImageQuality::Fast,
        ImageQuality::Balanced,
        ImageQuality::High,
    ] {
        let config = Widget::from_raw_bytes(pixels.as_raw().clone(), 64, 24, Format::RGBA8888)
            .with_max_size(64, 24)
            .with_quality(quality);
        let output = SixelRenderer::new().render_image(&config).unwrap();
        let decoded = sixel_decode::decode(output.as_bytes()).unwrap();
        assert_eq!(decoded.dimensions(), pixels.dimensions());
        let mut sums = [0i64; 3];
        let mut square_error = 0u64;
        for (actual, expected) in decoded.pixels().zip(pixels.pixels()) {
            for channel in 0..3 {
                let delta = i64::from(actual[channel]) - i64::from(expected[channel]);
                sums[channel] += delta;
                square_error += (delta * delta) as u64;
            }
        }
        assert!(
            sums.iter().all(|sum| sum.abs() < 64 * 24 * 4),
            "{quality:?}: {sums:?}"
        );
        assert!(
            square_error < 64 * 24 * 3 * 35 * 35,
            "{quality:?}: {square_error}"
        );
        outputs.push(output);
    }
    assert_ne!(
        outputs[0], outputs[1],
        "Balanced must diffuse quantization error"
    );
    assert_ne!(
        outputs[1], outputs[2],
        "High must use its distinct diffusion weights"
    );
}

#[test]
fn api_image_platform_sixel_offsets_preserve_transparent_pixels() {
    let image = Image::from_pixels(vec![255, 0, 0, 255], 1, 1, PixelFormat::Rgba);
    let mut bytes = Vec::new();
    image
        .render_sixel(
            &mut bytes,
            &DrawOptions {
                pixel_offset: Some((2, 3)),
                ..Default::default()
            },
        )
        .unwrap();
    let decoded = sixel_decode::decode(&bytes).unwrap();
    assert_eq!(decoded.dimensions(), (3, 4));
    for (x, y, pixel) in decoded.enumerate_pixels() {
        assert_eq!(
            pixel.0,
            if (x, y) == (2, 3) {
                [255, 0, 0, 255]
            } else {
                [0; 4]
            }
        );
    }
}

proptest::proptest! {
    #[test]
    fn smoke_api_image_platform_sixel_malformed_bodies_do_not_panic(body in proptest::collection::vec(0u8..=127, 0..256)) {
        let mut bytes = b"\x1bP0;1q".to_vec();
        bytes.extend(body);
        bytes.extend_from_slice(b"\x1b\\");
        let _ = Image::from_memory(bytes, ImageFormat::Sixel);
    }
}
