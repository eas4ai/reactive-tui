use super::{app_input, Control};
use reactive_tui::{
    builder,
    widgets::{ImageDisplayMode, ImageFormat, ImageQuality},
};

#[test]
fn image_builder_paints_decoded_pixels_with_authored_bounds() {
    for size in [(24, 8), (48, 14)] {
        let image = builder::image()
            .source_raw_bytes(vec![0; 12], 2, 2, ImageFormat::RGB888)
            .display_mode(ImageDisplayMode::AsciiArt)
            .quality(ImageQuality::Fast)
            .class("w-4 h-2 ml-2 mt-1")
            .build();
        let root = builder::div().class("w-full h-full").child(image).build();
        let frames = app_input::run_when(Control(root), size, vec![("@@@@", None)]);
        let frame = frames.last().unwrap();
        // Margin utilities use the existing four-cell spacing scale.
        for x in 8..12 {
            assert_eq!(
                frame.screen.cell(4, x).unwrap().contents(),
                "@",
                "{}\n{:?}",
                frame.text,
                frame.geometry
            );
            assert_eq!(
                frame.screen.cell(5, x).unwrap().contents(),
                "@",
                "{}\n{:?}",
                frame.text,
                frame.geometry
            );
        }
        assert!(frame
            .screen
            .cell(1, 1)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
        assert!(!frame.text.contains("Image (mode:"));
    }
}

#[test]
fn image_builder_loads_encoded_file_contents() {
    let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    image::RgbImage::from_pixel(2, 2, image::Rgb([0, 0, 0]))
        .save(file.path())
        .unwrap();
    for size in [(24, 8), (48, 14)] {
        let control = builder::image()
            .source_file(file.path())
            .display_mode(ImageDisplayMode::AsciiArt)
            .quality(ImageQuality::Fast)
            .class("w-4 h-2")
            .build();
        app_input::run_when(Control(control), size, vec![("@@@@", None)]);
        assert!(file.path().exists());
    }
}

fn black_png() -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(2, 2, image::Rgb([0, 0, 0]))
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

#[test]
fn image_builder_retains_base64_data_url_and_format_hint() {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD.encode(black_png());
    for size in [(24, 8), (48, 14)] {
        for image in [
            builder::image()
                .source_base64(data.clone())
                .format(ImageFormat::PNG),
            builder::image().source_url(format!("data:image/png;base64,{data}")),
            builder::image()
                .source_raw_bytes(vec![0, 0, 0, 255], 1, 1, ImageFormat::RGB888)
                .format(ImageFormat::RGBA8888),
        ] {
            let frames = app_input::run_when(
                Control(
                    image
                        .display_mode(ImageDisplayMode::AsciiArt)
                        .quality(ImageQuality::Fast)
                        .class("w-4 h-2")
                        .build(),
                ),
                size,
                vec![("@@@@", None)],
            );
            let last = &frames.last().unwrap().text;
            assert!(
                last.contains("@@@@") && !last.contains("Image error"),
                "black image paints as ASCII art at {size:?}:\n{last}"
            );
        }
    }
}

#[test]
fn image_builder_reports_invalid_sources_and_empty_state() {
    for size in [(32, 8), (48, 14)] {
        for image in [
            builder::image().source_base64("%%%".into()),
            builder::image().source_url("https://example.invalid/image.png".into()),
            builder::image().source_raw_bytes(vec![0; 3], 1, 1, ImageFormat::RGBA8888),
            builder::image()
                .source_raw_bytes(black_png(), 1, 1, ImageFormat::PNG)
                .format(ImageFormat::JPEG),
        ] {
            let frames = app_input::run_when(
                Control(image.class("w-full h-3").build()),
                size,
                vec![("Image error:", None)],
            );
            let last = &frames.last().unwrap().text;
            assert!(
                last.contains("Image error:"),
                "invalid source reports an error at {size:?}:\n{last}"
            );
        }
        let frames = app_input::run_when(
            Control(builder::image().class("w-full h-2").build()),
            size,
            vec![("Image has no source", None)],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("Image has no source") && !last.contains("Image error"),
            "empty image reports its empty state at {size:?}:\n{last}"
        );
    }
}

struct ChangingImage {
    gray: bool,
    removed: bool,
}
impl reactive_tui::app::RootComponent for ChangingImage {
    fn render(&self) -> reactive_tui::component::Element {
        if self.removed {
            return builder::div()
                .class("w-full h-full")
                .child(reactive_tui::component::Element::text("REMOVED"))
                .build();
        }
        let image = builder::image()
            .source_raw_bytes(
                vec![if self.gray { 160 } else { 0 }; 3],
                1,
                1,
                ImageFormat::RGB888,
            )
            .display_mode(ImageDisplayMode::AsciiArt)
            .quality(ImageQuality::Fast)
            .class("w-4 h-2")
            .build()
            .with_key("picture");
        builder::div().class("w-full h-full").child(image).build()
    }
    fn try_handle_event(
        &mut self,
        event: &reactive_tui::event::types::Event,
    ) -> reactive_tui::error::Result<reactive_tui::event::router::EventResult> {
        use reactive_tui::event::{
            router::EventResult,
            types::{Event, KeyCode},
        };
        Ok(match event {
            Event::Key(key) if key.code == KeyCode::F(1) => {
                self.gray = true;
                EventResult::Handled
            }
            Event::Key(key) if key.code == KeyCode::F(2) => {
                self.removed = true;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        })
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn image_builder_updates_a_keyed_source_and_clears_on_removal() {
    use reactive_tui::event::types::KeyCode;
    for size in [(24, 8), (48, 14)] {
        let frames = app_input::run_when(
            ChangingImage {
                gray: false,
                removed: false,
            },
            size,
            vec![
                ("@@@@", app_input::key(KeyCode::F(1))),
                ("----", app_input::key(KeyCode::F(2))),
                ("REMOVED", None),
            ],
        );
        let last = &frames.last().unwrap().text;
        assert!(!last.contains("@@@@"));
        assert!(!last.contains("----"));
        assert!(!last.contains("Loading"));
    }
}

#[test]
fn image_builder_resizes_to_its_presented_content_area() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    let control = builder::image()
        .source_raw_bytes(vec![0; 3], 1, 1, ImageFormat::RGB888)
        .display_mode(ImageDisplayMode::AsciiArt)
        .quality(ImageQuality::Fast)
        .class("w-1/2 h-1/2")
        .build();
    let frames = app_input::run_when(
        Control(builder::div().class("w-full h-full").child(control).build()),
        (24, 8),
        vec![
            ("@@@@@@@@", Some(Event::Resize(ResizeEvent::new(48, 14)))),
            ("@@@@@@@@@@@@@@", None),
        ],
    );
    assert!(frames.last().unwrap().text.contains("@@@@@@@@@@@@@@"));
}

#[test]
fn image_builder_clips_to_its_parent_and_keeps_other_images_independent() {
    for size in [(24, 8), (48, 14)] {
        let image = |value, class: &str| {
            builder::image()
                .source_raw_bytes(vec![value; 3], 1, 1, ImageFormat::RGB888)
                .display_mode(ImageDisplayMode::AsciiArt)
                .quality(ImageQuality::Fast)
                .class(class)
                .build()
        };
        let root = builder::div()
            .class("flex flex-row w-full h-full")
            .child(
                builder::div()
                    .class("w-4 h-2 overflow-hidden shrink-0")
                    .child(image(0, "w-8 h-4 shrink-0"))
                    .build(),
            )
            .child(image(160, "w-4 h-2 shrink-0"))
            .build();
        let frames = app_input::run_when(Control(root), size, vec![("@@@@----", None)]);
        let frame = frames.last().unwrap();
        assert!(!frame.text.contains("@@@@@@@@"));
        assert!(frame
            .screen
            .cell(2, 0)
            .unwrap()
            .contents()
            .trim()
            .is_empty());
    }
}

#[test]
fn image_native_element_honors_alpha_background_quality_and_text_mode() {
    use reactive_tui::{core::surface::Rgba, widgets::Image};
    for size in [(24, 8), (48, 14)] {
        let image = Image::from_raw_bytes(vec![255, 255, 255, 0], 1, 1, ImageFormat::RGBA8888)
            .with_background_color(Rgba {
                r: 160.0 / 255.0,
                g: 160.0 / 255.0,
                b: 160.0 / 255.0,
                a: 1.0,
            })
            .with_display_mode(ImageDisplayMode::AsciiArt)
            .with_quality(ImageQuality::Fast);
        let frames = app_input::run_when(
            Control(image.into_element().with_class("w-4 h-2")),
            size,
            vec![("----", None)],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("----"),
            "transparent pixel over a grey background paints mid-tone at {size:?}:\n{last}"
        );
        let detailed = builder::image()
            .source_raw_bytes(vec![0; 3], 1, 1, ImageFormat::RGB888)
            .display_mode(ImageDisplayMode::AsciiArt)
            .quality(ImageQuality::High)
            .class("w-4 h-2")
            .build();
        let frames = app_input::run_when(Control(detailed), size, vec![("$$$$", None)]);
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("$$$$"),
            "high quality black pixel paints the densest glyph at {size:?}:\n{last}"
        );
        let text: reactive_tui::component::Element = Image::default()
            .with_display_mode(ImageDisplayMode::Fallback)
            .with_fallback_text("CUSTOM IMAGE")
            .into();
        let frames = app_input::run_when(
            Control(text.with_class("w-full h-2")),
            size,
            vec![("CUSTOM IMAGE", None)],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("CUSTOM IMAGE"),
            "fallback text mode paints the custom text at {size:?}:\n{last}"
        );
    }
}

struct GraphicImage {
    green: bool,
    removed: bool,
    mode: ImageDisplayMode,
}
impl reactive_tui::app::RootComponent for GraphicImage {
    fn render(&self) -> reactive_tui::component::Element {
        let mut root = builder::div().class("w-full h-full");
        if !self.removed {
            let color = if self.green {
                [0, 255, 0, 255]
            } else {
                [255, 0, 0, 255]
            };
            root = root.child(
                builder::image()
                    .source_raw_bytes(color.repeat(8), 4, 2, ImageFormat::RGBA8888)
                    .display_mode(self.mode)
                    .class("absolute left-2 top-1 w-4 h-2")
                    .build()
                    .with_key("image"),
            );
        }
        root.build()
    }
    fn try_handle_event(
        &mut self,
        event: &reactive_tui::event::types::Event,
    ) -> reactive_tui::error::Result<reactive_tui::event::router::EventResult> {
        use reactive_tui::event::{
            router::EventResult,
            types::{Event, KeyCode},
        };
        Ok(match event {
            Event::Key(key) if key.code == KeyCode::Char('g') => {
                self.green = true;
                EventResult::Handled
            }
            Event::Key(key) if key.code == KeyCode::Char('x') => {
                self.removed = true;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        })
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[test]
fn image_builder_presents_decoded_kitty_pixels_and_releases_updated_and_removed_resources() {
    use base64::Engine;
    use reactive_tui::{backend::ImageOutputOptions, event::types::KeyCode};
    for size in [(16, 6), (32, 10)] {
        let red = base64::engine::general_purpose::STANDARD.encode([255, 0, 0, 255].repeat(8));
        let green = base64::engine::general_purpose::STANDARD.encode([0, 255, 0, 255].repeat(8));
        let frames = app_input::run_when_output(
            GraphicImage {
                green: false,
                removed: false,
                mode: ImageDisplayMode::KittyGraphics,
            },
            size,
            ImageOutputOptions {
                kitty_graphics: true,
                cell_pixels: (1, 1),
                ..Default::default()
            },
            vec![
                (red.clone(), 1, app_input::key(KeyCode::Char('g'))),
                (green.clone(), 1, app_input::key(KeyCode::Char('x'))),
                ("a=d,d=I".into(), 2, None),
            ],
        );
        let output = String::from_utf8(frames.last().unwrap().output.clone()).unwrap();
        assert!(output.contains(&red) && output.contains(&green));
        assert!(output.contains("\x1b[2;3H\x1b_Ga=T"));
        assert!(output.contains("s=4,v=2"));
        assert!(!frames.last().unwrap().text.contains("Loading image"));
        let latest_image = output.rfind("a=T").unwrap();
        let latest_delete = output.rfind("a=d,d=I").unwrap();
        assert!(
            latest_delete > latest_image,
            "removal must delete the final placement"
        );
    }
}

#[test]
fn image_file_and_encoded_memory_share_kitty_payloads_in_app() {
    use base64::Engine;
    let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    let pixels = image::RgbaImage::from_pixel(4, 2, image::Rgba([19, 83, 201, 128]));
    pixels.save(file.path()).unwrap();
    let encoded = std::fs::read(file.path()).unwrap();
    let expected = base64::engine::general_purpose::STANDARD.encode(pixels.as_raw());
    for size in [(16, 6), (32, 10)] {
        for image in [
            builder::image().source_file(file.path()),
            builder::image()
                .source_base64(base64::engine::general_purpose::STANDARD.encode(&encoded))
                .format(ImageFormat::PNG),
        ] {
            let frames = app_input::run_when_output(
                Control(
                    image
                        .display_mode(ImageDisplayMode::KittyGraphics)
                        .class("w-4 h-2")
                        .build(),
                ),
                size,
                reactive_tui::backend::ImageOutputOptions {
                    kitty_graphics: true,
                    cell_pixels: (1, 1),
                    ..Default::default()
                },
                vec![(expected.clone(), 1, None)],
            );
            assert!(String::from_utf8_lossy(&frames.last().unwrap().output).contains(&expected));
            assert!(file.path().exists());
        }
    }
}

#[test]
fn image_builder_presents_sixel_and_inline_updates_and_removal() {
    use base64::Engine;
    use reactive_tui::{backend::ImageOutputOptions, event::types::KeyCode};
    for size in [(16, 6), (32, 10)] {
        for mode in [ImageDisplayMode::Sixel, ImageDisplayMode::ITerm2Inline] {
            let markers: Vec<String> = if mode == ImageDisplayMode::Sixel {
                vec!["#0;2;100;0;0".into(), "#0;2;0;100;0".into()]
            } else {
                [[255, 0, 0, 255], [0, 255, 0, 255]]
                    .into_iter()
                    .map(|pixel| {
                        let mut bytes = std::io::Cursor::new(Vec::new());
                        image::RgbaImage::from_pixel(4, 2, image::Rgba(pixel))
                            .write_to(&mut bytes, image::ImageFormat::Png)
                            .unwrap();
                        base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
                    })
                    .collect()
            };
            let frames = app_input::run_when_output(
                GraphicImage {
                    green: false,
                    removed: false,
                    mode,
                },
                size,
                ImageOutputOptions {
                    sixel: mode == ImageDisplayMode::Sixel,
                    iterm2_inline: mode == ImageDisplayMode::ITerm2Inline,
                    cell_pixels: (1, 1),
                    ..Default::default()
                },
                vec![
                    (markers[0].clone(), 1, app_input::key(KeyCode::Char('g'))),
                    (markers[1].clone(), 1, app_input::key(KeyCode::Char('x'))),
                    ("\x1b[2J".into(), 2, None),
                ],
            );
            let last = frames.last().unwrap();
            let output = String::from_utf8_lossy(&last.output);
            assert!(output.rfind("\x1b[2J").unwrap() > output.rfind(&markers[1]).unwrap());
            assert!(!last.text.contains("Loading"));
        }
    }
}

#[test]
fn image_animated_gif_advances_without_input_and_repeats_through_app() {
    use base64::Engine;
    use image::{
        codecs::gif::{GifEncoder, Repeat},
        Delay, Frame,
    };
    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut bytes);
        encoder.set_repeat(Repeat::Infinite).unwrap();
        for color in [[255, 0, 0, 255], [0, 255, 0, 255]] {
            encoder
                .encode_frame(Frame::from_parts(
                    image::RgbaImage::from_pixel(4, 2, image::Rgba(color)),
                    0,
                    0,
                    Delay::from_numer_denom_ms(100, 1),
                ))
                .unwrap();
        }
    }
    let red = base64::engine::general_purpose::STANDARD.encode([255, 0, 0, 255].repeat(8));
    let green = base64::engine::general_purpose::STANDARD.encode([0, 255, 0, 255].repeat(8));
    for size in [(16, 6), (32, 10)] {
        let frames = app_input::run_when_output(
            Control(
                builder::image()
                    .source_raw_bytes(bytes.clone(), 0, 0, ImageFormat::GIF)
                    .display_mode(ImageDisplayMode::KittyGraphics)
                    .class("w-4 h-2")
                    .build(),
            ),
            size,
            reactive_tui::backend::ImageOutputOptions {
                kitty_graphics: true,
                cell_pixels: (1, 1),
                ..Default::default()
            },
            vec![(red.clone(), 2, None)],
        );
        let output = String::from_utf8_lossy(&frames.last().unwrap().output);
        assert!(output.find(&red).unwrap() < output.find(&green).unwrap());
        assert!(output.rfind(&red).unwrap() > output.find(&green).unwrap());
    }
}

/// Pin the blitter the block fallback draws with, so what the widget draws
/// does not depend on the host's terminal or an installed chafa or viu (a
/// named blitter wins over both). False, after printing SKIP, when the
/// REACTIVE_TUI_BLITTER environment variable overrides the pin.
fn pin_sextant() -> bool {
    if std::env::var_os("REACTIVE_TUI_BLITTER").is_some() {
        eprintln!("SKIP: REACTIVE_TUI_BLITTER overrides the pinned blitter");
        return false;
    }
    reactive_tui::widgets::display::set_image_blitter(Some(
        reactive_tui::widgets::display::Blitter::Sextant,
    ));
    true
}

/// A 96 by 32 picture, three times as wide as tall: a diagonal between two
/// gradients, so the block fallback draws partial blocks in many colors. No
/// pixel is black, the color the painter clears the screen to.
fn picture() -> reactive_tui::component::Element {
    let (width, height) = (96u32, 32u32);
    let mut bytes = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            let shade = (40 + x * 2) as u8;
            bytes.extend_from_slice(&if x > y * 3 {
                [220, shade, 40]
            } else {
                [40, shade, 220]
            });
        }
    }
    builder::image()
        .source_raw_bytes(bytes, width, height, ImageFormat::RGB888)
        .display_mode(ImageDisplayMode::Auto)
        .quality(ImageQuality::Fast)
        .build()
}

/// Columns of the widest row the picture reaches: cells whose background
/// is not the cleared screen's black.
fn drawn_width(frame: &super::common::app_input::Snapshot) -> u16 {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .map(|row| {
            (0..columns)
                .filter(|&column| {
                    frame.screen.cell(row, column).is_some_and(|cell| {
                        cell.bgcolor() != vt100::Color::Rgb(0, 0, 0)
                            && cell.bgcolor() != vt100::Color::Default
                    })
                })
                .count() as u16
        })
        .max()
        .unwrap_or(0)
}

/// BAR-003: an image with no size of its own fills the rectangle its parent
/// allots. The picture is wider than the rectangle's shape, so it spans
/// every column.
#[test]
fn bar_003_an_image_fills_the_rectangle_its_parent_allots() {
    if !pin_sextant() {
        return;
    }
    for size in [(40u16, 12u16), (72, 20)] {
        let root = builder::div()
            .class("w-full h-full")
            .child(picture())
            .build();
        // The first frame that is not busy holds the drawn picture.
        let frames = app_input::run(Control(root), size, vec![(1, None)]);
        let frame = frames.last().unwrap();
        assert_eq!(
            drawn_width(frame),
            size.0,
            "at {size:?} the image must span its parent:\n{}",
            frame.text
        );
    }
}

/// BAR-003: after a resize the image draws at the new size and never shows
/// the cells it drew for the old one; while the worker draws, the frame
/// shows the image area empty.
#[test]
fn bar_003_a_resized_image_never_shows_the_cells_drawn_for_the_old_size() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    if !pin_sextant() {
        return;
    }
    let (old, new) = ((40u16, 12u16), (100u16, 30u16));
    let root = builder::div()
        .class("w-full h-full")
        .child(picture())
        .build();
    let frames = app_input::run(
        Control(root),
        old,
        vec![
            (1, Some(Event::Resize(ResizeEvent::new(new.0, new.1)))),
            (2, None),
        ],
    );
    let widths: Vec<u16> = frames
        .iter()
        .filter(|frame| frame.screen.size() == (new.1, new.0))
        .map(drawn_width)
        .collect();
    assert!(!widths.is_empty(), "no frame at the new size");
    assert!(
        widths.iter().all(|&width| width == 0 || width == new.0),
        "a frame after the resize drew the picture at another width: {widths:?}"
    );
    assert_eq!(widths.last(), Some(&new.0), "{widths:?}");
}

/// Text grid plus a stable digest of every cell's colors.
fn golden_bytes(frame: &super::common::app_input::Snapshot) -> Vec<u8> {
    let (rows, columns) = frame.screen.size();
    let mut hasher = super::common::digest::Digest::default();
    for row in 0..rows {
        for column in 0..columns {
            if let Some(cell) = frame.screen.cell(row, column) {
                hasher.field(format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).as_bytes());
            }
        }
    }
    format!("{}\ncolors: {:016x}\n", frame.text, hasher.finish()).into_bytes()
}

/// BAR-004: the image widget renders on the debug backend to checked-in
/// goldens at two sizes, the wide one 400 columns, since its layout scales
/// with width. REGENERATE=1 rewrites them after an intended change.
#[test]
fn bar_004_image_goldens_at_two_sizes_on_the_debug_backend() {
    if !pin_sextant() {
        return;
    }
    // REACTIVE_TUI_SNAPSHOTS: the charts-goldens check points it at a copy
    // with one golden changed, and this test must then fail.
    let dir = std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("image");
    for (name, size) in [("image_medium", (80u16, 24u16)), ("image_wide", (400, 100))] {
        let root = builder::div()
            .class("w-full h-full")
            .child(picture())
            .build();
        let frames = app_input::run_on_debug(Control(root), size, vec![(1, None)]);
        let frame = frames.last().unwrap();
        assert_eq!(drawn_width(frame), size.0, "{name} must span the screen");
        let path = dir.join(format!("{name}.ansi"));
        let bytes = golden_bytes(frame);
        if std::env::var("REGENERATE").as_deref() == Ok("1") {
            std::fs::create_dir_all(&dir).expect("snapshot dir");
            std::fs::write(&path, &bytes).expect("write golden");
            continue;
        }
        let expected = std::fs::read(&path).unwrap_or_else(|_| {
            panic!(
                "missing golden {path:?}; run with REGENERATE=1, review the diff, then commit it"
            )
        });
        assert_eq!(
            String::from_utf8_lossy(&bytes),
            String::from_utf8_lossy(&expected),
            "golden mismatch for {name}"
        );
    }
}
