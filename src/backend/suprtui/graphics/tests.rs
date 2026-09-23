use crate::{
    backend::{Backend, ImageOutputOptions, SuprTuiBackend},
    builder::ElementBuilder,
    component::{Element, ElementType, LayoutType},
    widgets::{Image, ImageDisplayMode, ImageFormat},
};
use base64::Engine;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Bytes {
    bytes: Vec<u8>,
    flushes: usize,
    fail: bool,
}
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Bytes>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut state = self.0.lock().unwrap();
        state.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.flushes += 1;
        if state.fail {
            state.fail = false;
            Err(io::Error::other("image flush failed"))
        } else {
            Ok(())
        }
    }
}
impl Capture {
    fn take(&self) -> String {
        String::from_utf8(std::mem::take(&mut self.0.lock().unwrap().bytes)).unwrap()
    }
}
fn backend(capture: Capture) -> SuprTuiBackend {
    SuprTuiBackend::with_writer_and_images(
        8,
        4,
        capture,
        ImageOutputOptions {
            kitty_graphics: true,
            cell_pixels: (1, 1),
            ..Default::default()
        },
    )
    .unwrap()
}
fn picture(id: u32, color: [u8; 4], class: &str) -> Element {
    let config = Image::from_raw_bytes(color.repeat(8), 4, 2, ImageFormat::RGBA8888)
        .with_display_mode(ImageDisplayMode::KittyGraphics);
    let pixels = Arc::new(image::RgbaImage::from_raw(4, 2, color.repeat(8)).unwrap());
    let mut element = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .class(class)
        .build();
    element.metadata.image = Some(Arc::new(
        crate::widgets::display::image::paint::ImagePaint::new(id, pixels, &config),
    ));
    let mut fallback = Element::text("fallback");
    fallback.metadata.image_fallback = Some(id);
    element.children.push(fallback);
    element
}
fn frame(children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .class("w-full h-full")
        .children(children)
        .build()
}
fn payloads(output: &str) -> Vec<(String, Vec<u8>)> {
    let mut result = Vec::new();
    let mut control = String::new();
    let mut data = String::new();
    for part in output.split("\x1b_G").skip(1) {
        let part = part.split_once("\x1b\\").unwrap().0;
        let (header, payload) = part.split_once(';').unwrap();
        if header.contains("a=d") {
            continue;
        }
        if header.contains("a=T") {
            control = header.into();
            data.clear();
        }
        data.push_str(payload);
        if header.split(',').any(|field| field == "m=0") {
            result.push((
                control.clone(),
                base64::engine::general_purpose::STANDARD
                    .decode(&data)
                    .unwrap(),
            ));
        }
    }
    result
}
fn present(backend: &mut SuprTuiBackend, root: &Element) {
    backend.render_frame(root).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
}

fn protocol_picture(id: u32, color: [u8; 4], class: &str, mode: ImageDisplayMode) -> Element {
    let mut element = picture(id, color, class);
    Arc::make_mut(element.metadata.image.as_mut().unwrap()).mode = mode;
    element
}

#[test]
fn api_image_graphics_sixel_absolute_offsets_keep_preceding_translucent_pixels() {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer_and_images(
        8,
        4,
        capture.clone(),
        ImageOutputOptions {
            sixel: true,
            cell_pixels: (2, 3),
            ..Default::default()
        },
    )
    .unwrap();
    let mode = ImageDisplayMode::Sixel;
    let root = frame(vec![
        protocol_picture(101, [255, 0, 0, 128], "absolute w-4 h-2", mode),
        protocol_picture(102, [0, 0, 255, 255], "absolute left-4 top-2 w-4 h-2", mode),
    ])
    .with_class("w-full h-full bg-black");
    present(&mut backend, &root);
    let decoded = protocol_pixels(&capture.take(), mode);
    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[1].dimensions(), (16, 12));
    let prior = decoded[1].get_pixel(0, 0).0;
    assert!(prior[0].abs_diff(128) <= 2, "{prior:?}");
    assert_eq!(&prior[1..], &[0, 0, 255]);
    assert_eq!(decoded[1].get_pixel(8, 6).0, [0, 0, 255, 255]);
    assert_eq!(decoded[1].get_pixel(8, 0)[3], 0);
}

fn protocol_pixels(output: &str, mode: ImageDisplayMode) -> Vec<image::RgbaImage> {
    if mode == ImageDisplayMode::ITerm2Inline {
        return output
            .split("\x1b]1337;File=")
            .skip(1)
            .map(|part| {
                let payload = part
                    .split_once('\x07')
                    .unwrap()
                    .0
                    .split_once(':')
                    .unwrap()
                    .1;
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(payload)
                    .unwrap();
                image::load_from_memory(&bytes).unwrap().to_rgba8()
            })
            .collect();
    }
    output
        .split("\x1bP")
        .skip(1)
        .map(|part| {
            let sequence = format!("\x1bP{}\x1b\\", part.split_once("\x1b\\").unwrap().0);
            let image = crate::platform::image::Image::from_memory(
                sequence.into_bytes(),
                crate::platform::ImageFormat::Sixel,
            )
            .unwrap();
            let mut bytes = Vec::new();
            image
                .render_kitty(&mut bytes, &crate::platform::image::DrawOptions::default())
                .unwrap();
            let decoded = payloads(std::str::from_utf8(&bytes).unwrap());
            image::RgbaImage::from_raw(image.width, image.height, decoded[0].1.clone()).unwrap()
        })
        .collect()
}

#[test]
fn api_image_graphics_sixel_and_inline_keep_masks_cache_and_clear_owned_placements() {
    for mode in [ImageDisplayMode::Sixel, ImageDisplayMode::ITerm2Inline] {
        let capture = Capture::default();
        let mut backend = SuprTuiBackend::with_writer_and_images(
            8,
            4,
            capture.clone(),
            ImageOutputOptions {
                sixel: mode == ImageDisplayMode::Sixel,
                iterm2_inline: mode == ImageDisplayMode::ITerm2Inline,
                cell_pixels: (1, 1),
                ..Default::default()
            },
        )
        .unwrap();
        let root = frame(vec![
            protocol_picture(81, [255, 0, 0, 255], "absolute w-4 h-2", mode),
            protocol_picture(82, [0, 0, 255, 128], "absolute left-1 w-4 h-2", mode),
            Element::text("X").with_class("absolute left-2 top-0 w-1 h-1"),
        ]);
        present(&mut backend, &root);
        let output = capture.take();
        let pixels = protocol_pixels(&output, mode);
        assert_eq!(pixels.len(), 2, "{mode:?}");
        assert_eq!(pixels[0].get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(
            pixels[0].get_pixel(2, 0)[3],
            0,
            "text masks the lower image"
        );
        assert_eq!(
            pixels[1].get_pixel(
                if mode == ImageDisplayMode::Sixel {
                    2
                } else {
                    1
                },
                0
            )[3],
            0,
            "text masks the upper image"
        );
        if mode == ImageDisplayMode::Sixel {
            assert_eq!(pixels[1].get_pixel(0, 0).0, [255, 0, 0, 255]);
            assert!(output.contains("\x1b[?80s\x1b[?80h"));
            assert!(output.contains("\x1b[?80r"));
            for (actual, expected) in pixels[1]
                .get_pixel(1, 0)
                .0
                .into_iter()
                .zip([127u8, 0, 128, 255])
            {
                assert!(
                    actual.abs_diff(expected) <= 2,
                    "{:?}",
                    pixels[1].get_pixel(1, 0)
                );
            }
            for (actual, expected) in pixels[1]
                .get_pixel(4, 0)
                .0
                .into_iter()
                .zip([0u8, 0, 128, 255])
            {
                assert!(
                    actual.abs_diff(expected) <= 2,
                    "{:?}",
                    pixels[1].get_pixel(4, 0)
                );
            }
        } else {
            assert_eq!(pixels[1].get_pixel(0, 0).0, [127, 0, 128, 255]);
            assert_eq!(pixels[1].get_pixel(3, 0).0, [0, 0, 255, 128]);
        }
        assert!(!output.contains("fallback"));
        present(&mut backend, &root);
        assert!(
            protocol_pixels(&capture.take(), mode).is_empty(),
            "unchanged pixels stay cached"
        );
        backend.resize(10, 5);
        present(&mut backend, &root);
        let output = capture.take();
        assert!(output.contains("\x1b[2J"));
        assert_eq!(protocol_pixels(&output, mode).len(), 2);
        present(&mut backend, &frame(Vec::new()));
        let output = capture.take();
        assert!(output.contains("\x1b[2J"));
        assert!(protocol_pixels(&output, mode).is_empty());
        assert!(output.find("\x1b[?2026h").unwrap() < output.find("\x1b[2J").unwrap());
        present(&mut backend, &root);
        capture.take();
        backend.shutdown().unwrap();
        assert!(capture.take().contains("\x1b[2J"));
    }
}

#[test]
fn api_image_graphics_failed_sixel_flush_keeps_cleanup_for_retry() {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer_and_images(
        8,
        4,
        capture.clone(),
        ImageOutputOptions {
            sixel: true,
            cell_pixels: (1, 1),
            ..Default::default()
        },
    )
    .unwrap();
    let root = frame(vec![protocol_picture(
        83,
        [255, 0, 0, 255],
        "w-4 h-2",
        ImageDisplayMode::Sixel,
    )]);
    capture.0.lock().unwrap().fail = true;
    backend.render_frame(&root).unwrap();
    backend.present().unwrap();
    assert!(backend.sync().is_err());
    capture.take();
    present(&mut backend, &root);
    let output = capture.take();
    assert!(output.contains("\x1b[2J"));
    assert_eq!(protocol_pixels(&output, ImageDisplayMode::Sixel).len(), 1);
}
#[test]
fn api_image_graphics_output_is_owned_cached_moved_and_removed() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let image = picture(41, [255, 0, 0, 255], "absolute left-1 top-1 w-4 h-2");
    let root = frame(vec![image.clone()]);
    present(&mut backend, &root);
    let output = capture.take();
    let images = payloads(&output);
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].1, [255, 0, 0, 255].repeat(8));
    assert!(output.contains("\x1b[2;2H\x1b_Ga=T"));
    assert!(!output.contains("fallback"));
    assert_eq!(output.matches("\x1b[?2026h").count(), 1);
    let start = output.find("\x1b[?2026h").unwrap();
    let end = output.rfind("\x1b[?2026l").unwrap();
    assert!(start < output.find("\x1b_Ga=T").unwrap() && end > output.find("\x1b_Ga=T").unwrap());
    assert_eq!(capture.0.lock().unwrap().flushes, 1);
    present(&mut backend, &root);
    assert_eq!(capture.take(), "");
    let mut moved = image;
    moved.class = Some("absolute left-3 top-0 w-4 h-2".into());
    present(&mut backend, &frame(vec![moved]));
    let output = capture.take();
    assert!(output.contains("a=d,d=I,i=41"));
    assert!(output.contains("\x1b[1;4H\x1b_Ga=T"));
    present(&mut backend, &Element::empty());
    let output = capture.take();
    assert!(output.contains("a=d,d=I,i=41"));
    assert!(payloads(&output).is_empty());
    backend.shutdown().unwrap();
    assert!(capture.take().is_empty());
}
#[test]
fn api_image_graphics_actual_coverage_masks_text_and_opaque_backgrounds() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let image = picture(42, [200, 20, 10, 128], "absolute w-4 h-2");
    let text = Element::text("X").with_class("absolute left-1 top-0 w-1 h-1");
    let block = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .class("absolute left-2 top-1 w-1 h-1 bg-black")
        .build();
    present(&mut backend, &frame(vec![image, text, block]));
    let images = payloads(&capture.take());
    let mut expected = [200, 20, 10, 128].repeat(8);
    expected[4..8].fill(0);
    expected[24..28].fill(0);
    assert_eq!(images[0].1, expected);
    backend.shutdown().unwrap();
    assert!(capture.take().contains("a=d,d=I,i=42"));
}
#[test]
fn api_image_graphics_clip_and_output_failure_retry_delete_possible_placements() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let image = picture(43, [0, 255, 0, 255], "absolute left-6 top-3 w-4 h-2");
    capture.0.lock().unwrap().fail = true;
    backend.render_frame(&frame(vec![image])).unwrap();
    backend.present().unwrap();
    assert!(backend.sync().is_err());
    let failed = payloads(&capture.take());
    assert_eq!(failed[0].1, [0, 255, 0, 255].repeat(2));
    backend.present().unwrap();
    backend.sync().unwrap();
    let retry = capture.take();
    assert!(retry.contains("a=d,d=I,i=43"));
    assert_eq!(payloads(&retry)[0].1, failed[0].1);
    backend.shutdown().unwrap();
    assert!(capture.take().contains("a=d,d=I,i=43"));
}
#[test]
fn api_image_graphics_writer_requires_explicit_capability_and_valid_cell_pixels() {
    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(8, 4, capture.clone()).unwrap();
    present(
        &mut backend,
        &frame(vec![picture(44, [1, 2, 3, 255], "w-8 h-2")]),
    );
    let output = capture.take();
    assert!(payloads(&output).is_empty());
    let mut parser = vt100::Parser::new(4, 8, 0);
    parser.process(output.as_bytes());
    assert!(
        parser.screen().contents().contains("fallback"),
        "{}",
        parser.screen().contents()
    );
    for cell_pixels in [(0, 16), (8, 0), (257, 16)] {
        assert!(SuprTuiBackend::with_writer_and_images(
            8,
            4,
            Capture::default(),
            ImageOutputOptions {
                kitty_graphics: true,
                cell_pixels,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn api_image_graphics_cell_pixels_padding_tint_and_layer_order_follow_the_painter() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let low = picture(900, [255, 0, 0, 255], "absolute w-4 h-2");
    let high = picture(100, [0, 0, 255, 128], "absolute left-1 top-0 w-4 h-2");
    let tint = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(
            crate::layout::style::StyleBuilder::new()
                .position_absolute()
                .inset_left(0.0)
                .inset_top(0.0)
                .width_px(1.0)
                .height_px(1.0)
                .bg_rgba(0.0, 1.0, 0.0, 0.5),
        )
        .build();
    present(&mut backend, &frame(vec![low, high, tint]));
    let images = payloads(&capture.take());
    assert_eq!(images.len(), 2);
    assert!(images[0].0.contains("z=0") && images[1].0.contains("z=1"));
    assert_eq!(&images[0].1[..4], &[127, 128, 0, 255]);
    assert_eq!(&images[1].1[..4], &[0, 0, 255, 128]);
    backend.shutdown().unwrap();

    let capture = Capture::default();
    let mut backend = SuprTuiBackend::with_writer_and_images(
        8,
        4,
        capture.clone(),
        ImageOutputOptions {
            kitty_graphics: true,
            cell_pixels: (2, 2),
            ..Default::default()
        },
    )
    .unwrap();
    let mut image = picture(45, [0, 255, 255, 255], "absolute w-4 h-2 pl-1");
    // Padding utility uses four cells; an empty content box emits no graphics.
    present(&mut backend, &frame(vec![image.clone()]));
    assert!(payloads(&capture.take()).is_empty());
    image.class = Some("absolute w-4 h-2".into());
    present(&mut backend, &frame(vec![image]));
    let images = payloads(&capture.take());
    assert!(images[0].0.contains("s=8,v=4"));
    // Natural 4x2 pixels occupy two by one cells when cells are 2x2 pixels.
    let raster = image::RgbaImage::from_raw(8, 4, images[0].1.clone()).unwrap();
    assert_eq!(raster.get_pixel(3, 1).0, [0, 255, 255, 255]);
    assert_eq!(raster.get_pixel(4, 1).0, [0, 0, 0, 0]);
}

#[test]
fn api_image_graphics_refreshes_cell_metrics_and_clears_when_switching_to_cell_frames() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let root = frame(vec![picture(
        46,
        [255, 255, 0, 255],
        "absolute left-1 top-0 w-4 h-2",
    )]);
    present(&mut backend, &root);
    assert!(payloads(&capture.take())[0].0.contains("s=4,v=2"));
    backend.images.cell_pixels = (2, 2);
    present(&mut backend, &root);
    let output = capture.take();
    assert!(output.contains("a=d,d=I,i=46"));
    assert!(payloads(&output)[0].0.contains("s=8,v=4"));
    backend.resize(3, 2);
    present(&mut backend, &root);
    let output = capture.take();
    assert!(output.contains("a=d,d=I,i=46"));
    assert!(payloads(&output)[0].0.contains("s=4,v=4"));
    backend
        .render_cells(Arc::new(
            crate::backend::CellFrame::new(
                3,
                2,
                vec![
                    crate::backend::FrameCell {
                        text: " ".into(),
                        width: 1,
                        foreground: [255; 3],
                        background: [0; 3],
                        attributes: 0,
                        decoration: Default::default()
                    };
                    6
                ],
                None,
            )
            .unwrap(),
        ))
        .unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    let output = capture.take();
    assert!(output.contains("a=d,d=I,i=46"));
    assert!(payloads(&output).is_empty());
}

#[test]
fn native_cursor_respects_image_pixels_and_retains_coverage_on_cached_frames() {
    let capture = Capture::default();
    let mut backend = backend(capture.clone());
    let mut cursor = Element::text("R").with_class("absolute left-1 top-0 w-1 h-1");
    cursor.metadata.text_cursor = Some(crate::component::element::TextCursor {
        column: 0,
        style: ::suprtui::render::CursorStyle::Line,
    });
    for alpha in [255, 0] {
        let image = picture(91, [255, 0, 0, alpha], "absolute left-1 top-0 w-4 h-2");
        present(&mut backend, &frame(vec![cursor.clone(), image.clone()]));
        let output = capture.take();
        assert_eq!(
            output.contains("\x1b[?25h"),
            alpha == 0,
            "image alpha {alpha}: {output:?}"
        );
        // A text update away from the cursor keeps image geometry unchanged.
        present(
            &mut backend,
            &frame(vec![
                cursor.clone(),
                image.clone(),
                Element::text("X").with_class("absolute left-7 top-3 w-1 h-1"),
            ]),
        );
        let output = capture.take();
        assert_eq!(
            output.contains("\x1b[?25h"),
            alpha == 0,
            "cached image alpha {alpha}: {output:?}"
        );
        present(&mut backend, &frame(vec![image, cursor.clone()]));
        assert!(
            capture.take().contains("\x1b[?25h"),
            "image below text hid cursor"
        );
    }
}
