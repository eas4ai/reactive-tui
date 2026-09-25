//! Bounded decoded GIF frames and an elapsed-time playback clock.
use super::super::{decoded, ImageFormat, ImageSource};
use crate::error::{ReactiveError, Result};
use image::{AnimationDecoder, ImageDecoder};
use std::{io::Cursor, sync::Arc, time::Duration};

const MAX_FRAMES: usize = 4096;
const MAX_BYTES: usize = 256 * 1024 * 1024;

pub(super) struct Frame {
    pub pixels: Arc<image::RgbaImage>,
    end_ms: u64,
}
pub(super) struct Animation {
    pub frames: Vec<Frame>,
    cycles: Option<u32>,
}
impl Animation {
    pub fn still(pixels: Arc<image::RgbaImage>) -> Self {
        Self {
            frames: vec![Frame { pixels, end_ms: 0 }],
            cycles: Some(1),
        }
    }
    /// Index and deadline relative to playback start; expired frames are skipped.
    pub fn at(&self, elapsed: Duration) -> (usize, Option<Duration>) {
        let last = self.frames.len() - 1;
        let duration = u128::from(self.frames[last].end_ms);
        if last == 0 || duration == 0 {
            return (0, None);
        }
        let elapsed = elapsed.as_millis();
        let cycle = elapsed / duration;
        if self.cycles.is_some_and(|count| cycle >= u128::from(count)) {
            return (last, None);
        }
        let within = elapsed % duration;
        let index = self
            .frames
            .partition_point(|frame| u128::from(frame.end_ms) <= within);
        let deadline = cycle * duration + u128::from(self.frames[index].end_ms);
        (
            index,
            u64::try_from(deadline).ok().map(Duration::from_millis),
        )
    }
}
fn failure(message: impl std::fmt::Display) -> ReactiveError {
    ReactiveError::ImageProcessing(message.to_string())
}

pub(super) fn load(
    source: &ImageSource,
    hint: Option<ImageFormat>,
    cancelled: impl Fn() -> bool,
) -> Result<Arc<Animation>> {
    let data = match decoded::source_data(source, hint)? {
        decoded::SourceData::Raw {
            data,
            width,
            height,
            format,
        } => {
            return Ok(Arc::new(Animation::still(Arc::new(decoded::raw(
                data, width, height, format,
            )?))));
        }
        decoded::SourceData::Encoded(data) => data,
    };
    let format = decoded::format_hint(hint)?.or_else(|| image::guess_format(&data).ok());
    if format != Some(image::ImageFormat::Gif) {
        return Ok(Arc::new(Animation::still(Arc::new(
            decoded::encoded_format(&data, format)?,
        ))));
    }
    let mut metadata_options = gif::DecodeOptions::new();
    metadata_options.skip_frame_decoding(true);
    metadata_options.check_frame_consistency(true);
    let mut metadata = metadata_options
        .read_info(Cursor::new(data.as_ref()))
        .map_err(failure)?;
    let bytes_per_frame =
        decoded::dimensions(u32::from(metadata.width()), u32::from(metadata.height()))?;
    let cycles = match metadata.repeat() {
        gif::Repeat::Infinite => None,
        gif::Repeat::Finite(count) => Some(u32::from(count) + 1),
    };
    let mut count = 0usize;
    while metadata.next_frame_info().map_err(failure)?.is_some() {
        if cancelled() {
            return Err(failure("Image loading cancelled"));
        }
        count += 1;
        if count > MAX_FRAMES
            || count
                .checked_mul(bytes_per_frame)
                .is_none_or(|bytes| bytes > MAX_BYTES)
        {
            return Err(failure(
                "GIF exceeds the 4096-frame or 256 MiB decoded animation limit",
            ));
        }
    }
    if count == 0 {
        return Err(failure("GIF contains no frames"));
    }
    let mut decoder =
        image::codecs::gif::GifDecoder::new(Cursor::new(data.as_ref())).map_err(failure)?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_BYTES as u64);
    decoder.set_limits(limits).map_err(failure)?;
    let mut frames = Vec::with_capacity(count);
    let mut end_ms = 0u64;
    for frame in decoder.into_frames() {
        if cancelled() {
            return Err(failure("Image loading cancelled"));
        }
        let frame = frame.map_err(failure)?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay = (u64::from(numerator) / u64::from(denominator.max(1))).max(10);
        end_ms = end_ms
            .checked_add(delay)
            .ok_or_else(|| failure("GIF duration overflow"))?;
        frames.push(Frame {
            pixels: Arc::new(frame.into_buffer()),
            end_ms,
        });
    }
    if frames.len() != count {
        return Err(failure("GIF frame count changed during decoding"));
    }
    Ok(Arc::new(Animation { frames, cycles }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn source(repeat: gif::Repeat) -> ImageSource {
        let mut data = Vec::new();
        {
            let mut encoder =
                gif::Encoder::new(&mut data, 2, 1, &[255, 0, 0, 0, 0, 255, 0, 255, 0]).unwrap();
            encoder.set_repeat(repeat).unwrap();
            for (left, width, indices, delay, dispose) in [
                (0, 2, vec![0, 0], 2, gif::DisposalMethod::Keep),
                (1, 1, vec![1], 3, gif::DisposalMethod::Background),
                (0, 1, vec![2], 4, gif::DisposalMethod::Previous),
                (1, 1, vec![1], 1, gif::DisposalMethod::Keep),
            ] {
                encoder
                    .write_frame(&gif::Frame {
                        left,
                        width,
                        height: 1,
                        buffer: Cow::Owned(indices),
                        delay,
                        dispose,
                        ..Default::default()
                    })
                    .unwrap();
            }
        }
        ImageSource::RawBytes {
            data,
            width: 0,
            height: 0,
            format: ImageFormat::GIF,
        }
    }

    #[test]
    fn api_image_gif_composites_disposal_and_obeys_finite_and_infinite_timing() {
        let animation = load(&source(gif::Repeat::Finite(1)), None, || false).unwrap();
        let pixels: Vec<_> = animation
            .frames
            .iter()
            .map(|f| f.pixels.as_raw().clone())
            .collect();
        assert_eq!(
            pixels,
            vec![
                vec![255, 0, 0, 255, 255, 0, 0, 255],
                vec![255, 0, 0, 255, 0, 0, 255, 255],
                vec![0, 255, 0, 255, 0, 0, 0, 0],
                vec![255, 0, 0, 255, 0, 0, 255, 255],
            ]
        );
        for (time, index, deadline) in [
            (0, 0, Some(20)),
            (20, 1, Some(50)),
            (50, 2, Some(90)),
            (90, 3, Some(100)),
            (100, 0, Some(120)),
            (200, 3, None),
            (9999, 3, None),
        ] {
            assert_eq!(
                animation.at(Duration::from_millis(time)),
                (index, deadline.map(Duration::from_millis))
            );
        }
        let animation = load(&source(gif::Repeat::Infinite), None, || false).unwrap();
        assert_eq!(
            animation.at(Duration::from_millis(250)),
            (2, Some(Duration::from_millis(290)))
        );
    }

    #[test]
    fn api_image_gif_bounds_zero_delay_and_rejects_excess_tiny_frames() {
        for count in [2, MAX_FRAMES + 1] {
            let mut data = Vec::new();
            {
                let mut encoder =
                    gif::Encoder::new(&mut data, 1, 1, &[0, 0, 0, 255, 255, 255]).unwrap();
                for index in 0..count {
                    encoder
                        .write_frame(&gif::Frame {
                            width: 1,
                            height: 1,
                            buffer: Cow::Owned(vec![(index % 2) as u8]),
                            delay: 0,
                            ..Default::default()
                        })
                        .unwrap();
                }
            }
            let source = ImageSource::RawBytes {
                data,
                width: 0,
                height: 0,
                format: ImageFormat::GIF,
            };
            let result = load(&source, None, || false);
            if count == 2 {
                let animation = result.unwrap();
                assert_eq!(
                    animation.at(Duration::ZERO),
                    (0, Some(Duration::from_millis(10)))
                );
                assert_eq!(
                    animation.at(Duration::from_millis(10)),
                    (1, Some(Duration::from_millis(20)))
                );
                assert_eq!(animation.at(Duration::from_millis(20)), (1, None));
            } else {
                assert!(result.err().unwrap().to_string().contains("4096-frame"));
            }
        }
    }

    #[test]
    fn api_image_gif_rejects_cumulative_storage_before_allocating_frames_and_cancels() {
        let mut data = Vec::new();
        {
            let mut encoder =
                gif::Encoder::new(&mut data, 8192, 8192, &[0, 0, 0, 255, 255, 255]).unwrap();
            for _ in 0..2 {
                encoder
                    .write_frame(&gif::Frame {
                        width: 1,
                        height: 1,
                        buffer: Cow::Borrowed(&[0]),
                        ..Default::default()
                    })
                    .unwrap();
            }
        }
        let source = ImageSource::RawBytes {
            data,
            width: 0,
            height: 0,
            format: ImageFormat::GIF,
        };
        let error = load(&source, None, || false).err().unwrap().to_string();
        assert!(error.contains("decoded animation limit"), "{error}");
        assert!(load(&self::source(gif::Repeat::Infinite), None, || true)
            .err()
            .unwrap()
            .to_string()
            .contains("cancelled"));
    }
}
