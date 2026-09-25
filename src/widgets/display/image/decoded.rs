//! Shared checked decoding for every image renderer.

use super::{ImageFormat, ImageSource};
use crate::error::{ReactiveError, Result};
use base64::Engine;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbaImage};
use std::borrow::Cow;
use std::io::{Cursor, Read};
use std::path::Path;

const MAX_ENCODED: usize = 64 * 1024 * 1024;
const MAX_RGBA: u64 = 256 * 1024 * 1024;

fn error(message: impl std::fmt::Display) -> ReactiveError {
    ReactiveError::ImageProcessing(message.to_string())
}

pub(super) fn load(source: &ImageSource) -> Result<RgbaImage> {
    load_with_hint(source, None)
}

pub(super) fn load_with_hint(source: &ImageSource, hint: Option<ImageFormat>) -> Result<RgbaImage> {
    match source_data(source, hint)? {
        SourceData::Raw {
            data,
            width,
            height,
            format,
        } => raw(data, width, height, format),
        SourceData::Encoded(data) => encoded(&data, hint),
    }
}

pub(super) enum SourceData<'a> {
    Raw {
        data: &'a [u8],
        width: u32,
        height: u32,
        format: ImageFormat,
    },
    Encoded(Cow<'a, [u8]>),
}

pub(super) fn source_data(
    source: &ImageSource,
    hint: Option<ImageFormat>,
) -> Result<SourceData<'_>> {
    let bytes = match source {
        ImageSource::FilePath(path) => Cow::Owned(read_file(path)?),
        ImageSource::Base64Data(data) => Cow::Owned(base64_bytes(data)?),
        ImageSource::RawBytes {
            data,
            width,
            height,
            format,
        } => {
            let format = hint.unwrap_or(*format);
            if matches!(format, ImageFormat::RGB888 | ImageFormat::RGBA8888) {
                return Ok(SourceData::Raw {
                    data,
                    width: *width,
                    height: *height,
                    format,
                });
            }
            Cow::Borrowed(data.as_slice())
        }
        ImageSource::Url(url) => {
            if url.starts_with("data:") {
                let (header, data) = url
                    .split_once(',')
                    .ok_or_else(|| error("Image data URL has no payload"))?;
                if !header.starts_with("data:image/") || !header.ends_with(";base64") {
                    return Err(error("Image data URL must contain a base64 image payload"));
                }
                Cow::Owned(base64_bytes(data)?)
            } else if url.starts_with("http://") || url.starts_with("https://") {
                return Err(error("HTTP image loading is not supported; use a local file or base64 image data URL"));
            } else {
                Cow::Owned(read_file(Path::new(url))?)
            }
        }
    };
    if bytes.len() > MAX_ENCODED {
        return Err(error("Encoded image exceeds the 64 MiB limit"));
    }
    Ok(SourceData::Encoded(bytes))
}

pub(crate) fn dimensions(width: u32, height: u32) -> Result<usize> {
    let bytes = u64::from(width) * u64::from(height);
    let bytes = bytes.checked_mul(4).filter(|bytes| *bytes <= MAX_RGBA);
    if width == 0 || height == 0 {
        return Err(error("Image dimensions must be nonzero"));
    }
    bytes
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| error("Decoded image exceeds the 256 MiB RGBA limit"))
}

pub(crate) fn raw(data: &[u8], width: u32, height: u32, format: ImageFormat) -> Result<RgbaImage> {
    let rgba_len = dimensions(width, height)?;
    let channels = if format == ImageFormat::RGB888 { 3 } else { 4 };
    let expected = rgba_len / 4 * channels;
    if data.len() != expected {
        return Err(error(format!(
            "Raw image byte count mismatch: expected {expected}, got {}",
            data.len()
        )));
    }
    let bytes = if channels == 4 {
        data.to_vec()
    } else {
        data.chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect()
    };
    RgbaImage::from_raw(width, height, bytes).ok_or_else(|| error("Invalid RGBA image buffer"))
}

pub(crate) fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|err| error(format!("Cannot open image {}: {err}", path.display())))?;
    let metadata = file.metadata().map_err(error)?;
    if !metadata.is_file() {
        return Err(error("Image source must be a regular file"));
    }
    if metadata.len() > MAX_ENCODED as u64 {
        return Err(error("Encoded image exceeds the 64 MiB limit"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_ENCODED as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() > MAX_ENCODED {
        return Err(error("Encoded image exceeds the 64 MiB limit"));
    }
    Ok(bytes)
}

fn base64_bytes(data: &str) -> Result<Vec<u8>> {
    if data.len() > MAX_ENCODED.div_ceil(3) * 4 {
        return Err(error("Encoded image exceeds the 64 MiB limit"));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|err| error(format!("Invalid image base64: {err}")))?;
    Ok(bytes)
}

fn encoded(data: &[u8], hint: Option<ImageFormat>) -> Result<RgbaImage> {
    encoded_format(data, format_hint(hint)?)
}

pub(super) fn format_hint(hint: Option<ImageFormat>) -> Result<Option<image::ImageFormat>> {
    hint.map(|format| {
        Ok(match format {
            ImageFormat::PNG => image::ImageFormat::Png,
            ImageFormat::JPEG => image::ImageFormat::Jpeg,
            ImageFormat::GIF => image::ImageFormat::Gif,
            ImageFormat::BMP => image::ImageFormat::Bmp,
            ImageFormat::TIFF => image::ImageFormat::Tiff,
            ImageFormat::RGB888 | ImageFormat::RGBA8888 => {
                return Err(error(
                    "Raw format hints require a raw source with explicit dimensions",
                ))
            }
        })
    })
    .transpose()
}

pub(crate) fn encoded_format(data: &[u8], hint: Option<image::ImageFormat>) -> Result<RgbaImage> {
    if data.len() > MAX_ENCODED {
        return Err(error("Encoded image exceeds the 64 MiB limit"));
    }
    let mut reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(error)?;
    if let Some(format) = hint {
        reader.set_format(format);
    }
    let mut decoder = reader.into_decoder().map_err(error)?;
    let (width, height) = decoder.dimensions();
    let rgba_bytes = dimensions(width, height)? as u64;
    let conversion_bytes = (decoder.color_type() != image::ColorType::Rgba8).then_some(rgba_bytes);
    let remaining = MAX_RGBA
        .checked_sub(data.len() as u64)
        .and_then(|bytes| bytes.checked_sub(conversion_bytes.unwrap_or(0)))
        .filter(|bytes| decoder.total_bytes() <= *bytes)
        .ok_or_else(|| error("Image exceeds the 256 MiB total memory budget"))?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(remaining);
    decoder
        .set_limits(limits)
        .map_err(|_| error("Image exceeds the 256 MiB total memory budget"))?;
    DynamicImage::from_decoder(decoder)
        .map(DynamicImage::into_rgba8)
        .map_err(error)
}

/// Opaque renderers composite transparent pixels over the selected background.
/// Without a background, black is the terminal-independent fallback.
pub(super) fn rgb(image: &super::Image) -> Result<image::RgbImage> {
    let pixels = load(&image.source)?;
    Ok(rgb_pixels(&pixels, image.background_color))
}

pub(super) fn rgb_pixels(
    pixels: &RgbaImage,
    background_color: Option<crate::core::surface::Rgba>,
) -> image::RgbImage {
    let background = background_color
        .map(|color| {
            [
                (color.r.clamp(0.0, 1.0) * color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.g.clamp(0.0, 1.0) * color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
                (color.b.clamp(0.0, 1.0) * color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]
        })
        .unwrap_or([0; 3]);
    image::RgbImage::from_fn(pixels.width(), pixels.height(), |x, y| {
        let pixel = pixels.get_pixel(x, y);
        let alpha = u32::from(pixel[3]);
        image::Rgb(std::array::from_fn(|index| {
            ((u32::from(pixel[index]) * alpha + u32::from(background[index]) * (255 - alpha) + 127)
                / 255) as u8
        }))
    })
}

/// Source-over for byte pixels. Integer rounding keeps opaque alpha exactly 255.
pub(crate) fn blend_pixel(below: &mut image::Rgba<u8>, above: &image::Rgba<u8>) {
    let top = u32::from(above[3]);
    let bottom = u32::from(below[3]);
    let alpha = top * 255 + bottom * (255 - top);
    if alpha == 0 {
        *below = image::Rgba([0; 4]);
        return;
    }
    for channel in 0..3 {
        below[channel] = ((u32::from(above[channel]) * top * 255
            + u32::from(below[channel]) * bottom * (255 - top)
            + alpha / 2)
            / alpha) as u8;
    }
    below[3] = ((alpha + 127) / 255) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(pixels: RgbaImage) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        pixels
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn api_image_decode_sources_preserve_pixels_and_encoded_dimensions() {
        let expected = vec![
            255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 255, 0, 255, 255, 255, 255,
        ];
        let bytes = png(RgbaImage::from_raw(2, 2, expected.clone()).unwrap());
        let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), &bytes).unwrap();
        for source in [
            ImageSource::FilePath(file.path().to_owned()),
            ImageSource::Base64Data(base64.clone()),
            ImageSource::Url(format!("data:image/png;base64,{base64}")),
            ImageSource::Url(file.path().to_string_lossy().into_owned()),
            ImageSource::RawBytes {
                data: bytes,
                width: u32::MAX,
                height: 0,
                format: ImageFormat::PNG,
            },
            ImageSource::RawBytes {
                data: expected.clone(),
                width: 2,
                height: 2,
                format: ImageFormat::RGBA8888,
            },
        ] {
            let pixels = load(&source).unwrap();
            assert_eq!(pixels.dimensions(), (2, 2));
            assert_eq!(pixels.into_raw(), expected);
        }
        assert!(file.path().exists());
    }

    #[test]
    fn api_image_decode_all_advertised_encoded_formats() {
        for (format, source_format, tolerance) in [
            (image::ImageFormat::Png, ImageFormat::PNG, 0),
            (image::ImageFormat::Jpeg, ImageFormat::JPEG, 3),
            (image::ImageFormat::Gif, ImageFormat::GIF, 0),
            (image::ImageFormat::Bmp, ImageFormat::BMP, 0),
            (image::ImageFormat::Tiff, ImageFormat::TIFF, 0),
        ] {
            let mut bytes = Cursor::new(Vec::new());
            image::RgbImage::from_pixel(8, 8, image::Rgb([255, 0, 0]))
                .write_to(&mut bytes, format)
                .unwrap();
            let pixels = load(&ImageSource::RawBytes {
                data: bytes.into_inner(),
                width: 1,
                height: 1,
                format: source_format,
            })
            .unwrap();
            assert_eq!(pixels.dimensions(), (8, 8), "{format:?}");
            for pixel in pixels.pixels() {
                for (actual, expected) in pixel.0.into_iter().zip([255u8, 0, 0, 255]) {
                    assert!(
                        actual.abs_diff(expected) <= tolerance,
                        "{format:?}: {pixel:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn api_image_decode_rejects_malformed_and_oversized_sources() {
        for source in [
            ImageSource::Base64Data("%%%".into()),
            ImageSource::Url("data:image/png,not-base64".into()),
            ImageSource::Url("data:image/png;base64".into()),
            ImageSource::Url("https://example.invalid/image.png".into()),
            ImageSource::RawBytes {
                data: vec![0; 4],
                width: 1,
                height: 1,
                format: ImageFormat::RGB888,
            },
            ImageSource::RawBytes {
                data: vec![0; 3],
                width: 1,
                height: 1,
                format: ImageFormat::RGBA8888,
            },
            ImageSource::RawBytes {
                data: Vec::new(),
                width: 0,
                height: 1,
                format: ImageFormat::RGBA8888,
            },
            ImageSource::RawBytes {
                data: Vec::new(),
                width: u32::MAX,
                height: u32::MAX,
                format: ImageFormat::RGBA8888,
            },
            ImageSource::RawBytes {
                data: vec![1, 2, 3],
                width: 1,
                height: 1,
                format: ImageFormat::PNG,
            },
        ] {
            assert!(load(&source).is_err());
        }
        let large = tempfile::NamedTempFile::new().unwrap();
        large.as_file().set_len(MAX_ENCODED as u64 + 1).unwrap();
        assert!(load(&ImageSource::FilePath(large.path().to_owned()))
            .unwrap_err()
            .to_string()
            .contains("64 MiB"));
        let directory = tempfile::tempdir().unwrap();
        assert!(load(&ImageSource::FilePath(directory.path().to_owned())).is_err());
    }

    #[test]
    fn api_image_decode_opaque_renderers_composite_alpha() {
        let image =
            super::super::Image::from_raw_bytes(vec![255, 0, 0, 128], 1, 1, ImageFormat::RGBA8888);
        assert_eq!(rgb(&image).unwrap().into_raw(), [128, 0, 0]);
        let white = crate::core::surface::Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        assert_eq!(
            rgb(&image.clone().with_background_color(white))
                .unwrap()
                .into_raw(),
            [255, 127, 127]
        );
        assert_eq!(
            rgb(&image.with_background_color(crate::core::surface::Rgba { a: 0.0, ..white }))
                .unwrap()
                .into_raw(),
            [128, 0, 0]
        );
    }

    #[test]
    fn xis_002_decode_rejects_combined_storage_over_budget() {
        const SIDE: u32 = 6_300;
        let pixels = image::RgbImage::from_pixel(SIDE, SIDE, image::Rgb([7, 11, 13]));
        let mut encoded = Cursor::new(Vec::new());
        pixels
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        drop(pixels);
        let encoded = encoded.into_inner();
        assert!(encoded.len() < MAX_ENCODED);

        let decoder = ImageReader::new(Cursor::new(&encoded))
            .with_guessed_format()
            .unwrap()
            .into_decoder()
            .unwrap();
        let decoder_storage = decoder.total_bytes();
        let output_storage = dimensions(SIDE, SIDE).unwrap() as u64;
        let complete_storage = encoded.len() as u64 + decoder_storage + output_storage;
        assert!(
            complete_storage > MAX_RGBA,
            "fixture must exceed the total budget: {complete_storage}"
        );
        drop(decoder);

        let error = match encoded_format(&encoded, Some(image::ImageFormat::Png)) {
            Ok(_) => panic!(
                "decoder accepted {complete_storage} bytes of required storage under a {MAX_RGBA}-byte budget"
            ),
            Err(error) => error,
        };
        assert!(error.to_string().contains("total memory"), "{error}");
    }
}
