//! Controlled real-host image fixture. The driver writes stages 0..=3 to a file.
use reactive_tui::{
    app::{App, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    builder,
    component::Element,
    error::{ReactiveError, Result},
    widgets::{ImageDisplayMode, ImageFormat, ImageQuality},
};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

fn pixels(stage: u8) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(128 * 64 * 4);
    for _y in 0..64 {
        for x in 0..128 {
            pixels.extend_from_slice(match (stage, x < 64) {
                (0, true) => &[255, 0, 0, 255],
                (0, false) => &[0, 0, 255, 255],
                (_, true) => &[0, 255, 0, 255],
                (_, false) => &[255, 255, 0, 255],
            });
        }
    }
    pixels
}

struct Probe {
    command: PathBuf,
    stage: u8,
    started: Instant,
    mode: ImageDisplayMode,
    full: bool,
    animated: bool,
}

impl Probe {
    fn data(&self) -> Vec<u8> {
        if !self.animated {
            return pixels(self.stage);
        }
        let mut bytes = Vec::new();
        {
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
            encoder
                .set_repeat(image::codecs::gif::Repeat::Infinite)
                .unwrap();
            for stage in 0..2 {
                encoder
                    .encode_frame(image::Frame::from_parts(
                        image::RgbaImage::from_raw(128, 64, pixels(stage)).unwrap(),
                        0,
                        0,
                        image::Delay::from_numer_denom_ms(6000, 1),
                    ))
                    .unwrap();
            }
        }
        bytes
    }
    fn format(&self) -> ImageFormat {
        if self.animated {
            ImageFormat::GIF
        } else {
            ImageFormat::RGBA8888
        }
    }
}

impl RootComponent for Probe {
    fn render(&self) -> Element {
        let mut root = builder::div().class("w-full h-full bg-black");
        if self.stage < 2 && self.full {
            root = root.child(
                reactive_tui::widgets::Image::from_raw_bytes(self.data(), 128, 64, self.format())
                    .with_display_mode(self.mode)
                    .with_preserve_aspect(false)
                    .with_quality(
                        if matches!(self.mode, ImageDisplayMode::Chafa | ImageDisplayMode::Viu) {
                            ImageQuality::High
                        } else {
                            ImageQuality::Fast
                        },
                    )
                    .into_element()
                    .with_class("absolute w-full h-full")
                    .with_key("host-picture"),
            );
        } else if self.stage < 2 {
            root = root.child(
                builder::image()
                    .source_raw_bytes(self.data(), 128, 64, self.format())
                    .display_mode(self.mode)
                    .quality(
                        if matches!(self.mode, ImageDisplayMode::Chafa | ImageDisplayMode::Viu) {
                            ImageQuality::High
                        } else {
                            ImageQuality::Fast
                        },
                    )
                    .class(if self.stage == 0 {
                        "absolute left-2 top-2 w-8 h-4"
                    } else if matches!(self.mode, ImageDisplayMode::Chafa | ImageDisplayMode::Viu) {
                        "absolute left-12 top-6 w-12 h-6"
                    } else {
                        "absolute left-12 top-6 w-8 h-4"
                    })
                    .build()
                    .with_key("host-picture"),
            );
        }
        root.child(
            Element::text(format!("IMAGE HOST STAGE {}", self.stage))
                .with_class("absolute left-0 top-0 w-20 h-1 z-10"),
        )
        .build()
    }

    fn update(&mut self) -> Result<RootUpdate> {
        if self.started.elapsed() > Duration::from_secs(45) {
            return Err(ReactiveError::invalid_state(
                "image host fixture watchdog expired",
            ));
        }
        let stage: u8 = std::fs::read_to_string(&self.command)?
            .trim()
            .parse()
            .map_err(|_| ReactiveError::invalid_state("image host stage must be 0..=3"))?;
        match stage {
            3 => Ok(RootUpdate::Exit),
            0..=2 if stage != self.stage => {
                self.stage = stage;
                Ok(RootUpdate::Redraw)
            }
            0..=2 => Ok(RootUpdate::Unchanged),
            _ => Err(ReactiveError::invalid_state(
                "image host stage must be 0..=3",
            )),
        }
    }
}

fn main() -> Result<()> {
    let command = std::env::args_os()
        .nth(1)
        .ok_or_else(|| ReactiveError::invalid_state("expected stage command path"))?;
    let protocol = std::env::args().nth(2);
    if let Some(protocol) = protocol.as_deref().filter(|p| p.starts_with("surface-")) {
        return surface_stages(PathBuf::from(command), protocol);
    }
    if let Some(protocol) = protocol.as_deref().filter(|p| !p.starts_with("app-")) {
        return standalone(PathBuf::from(command), protocol);
    }
    let (backend, mode) = match protocol.as_deref() {
        Some("app-sixel" | "app-sixel-full" | "app-gif-sixel") => (
            SuprTuiBackend::new_with_images(reactive_tui::backend::ImageOutputOptions {
                sixel: true,
                ..Default::default()
            })?,
            ImageDisplayMode::Sixel,
        ),
        Some("app-iterm" | "app-iterm-full" | "app-gif-iterm") => (
            SuprTuiBackend::new_with_images(reactive_tui::backend::ImageOutputOptions {
                iterm2_inline: true,
                ..Default::default()
            })?,
            ImageDisplayMode::ITerm2Inline,
        ),
        Some("app-chafa" | "app-gif-chafa") => (SuprTuiBackend::new()?, ImageDisplayMode::Chafa),
        Some("app-viu" | "app-gif-viu") => (SuprTuiBackend::new()?, ImageDisplayMode::Viu),
        Some("app-gif") => (SuprTuiBackend::new()?, ImageDisplayMode::KittyGraphics),
        Some("app-auto") => (SuprTuiBackend::new()?, ImageDisplayMode::Auto),
        Some("app-ascii") => (SuprTuiBackend::new()?, ImageDisplayMode::AsciiArt),
        None => (SuprTuiBackend::new()?, ImageDisplayMode::Auto),
        _ => return Err(ReactiveError::invalid_state("unknown App image protocol")),
    };
    App::builder()
        .backend(backend)
        .root(Probe {
            command: command.into(),
            stage: 0,
            started: Instant::now(),
            mode,
            full: protocol.as_deref().is_some_and(|p| p.ends_with("-full")),
            animated: protocol
                .as_deref()
                .is_some_and(|p| p.starts_with("app-gif")),
        })
        .screen_reader(false)
        .build()?
        .run()
}

fn surface_stages(command: PathBuf, protocol: &str) -> Result<()> {
    use reactive_tui::{
        backend::ImageOutputOptions,
        core::{
            renderer::Renderer,
            surface::{Cell, Rgba},
        },
    };
    let (width, height) = crossterm::terminal::size()?;
    let mut renderer = Renderer::new(usize::from(width), usize::from(height))?;
    renderer.set_image_options(ImageOutputOptions {
        kitty_graphics: protocol.contains("kitty"),
        sixel: protocol.contains("sixel"),
        iterm2_inline: protocol.contains("iterm"),
        ..Default::default()
    });
    renderer.enable_high_performance_mode()?;
    let started = Instant::now();
    let mut shown = None;
    while started.elapsed() < Duration::from_secs(45) {
        let stage: u8 = std::fs::read_to_string(&command)?
            .trim()
            .parse()
            .map_err(|_| ReactiveError::invalid_state("invalid Surface fixture stage"))?;
        if stage == 3 {
            return renderer.shutdown();
        }
        if shown != Some(stage) {
            renderer.begin_frame()?;
            let surface = renderer.surface_mut();
            surface.clear(Rgba::black());
            surface.clear_images();
            if stage < 2 {
                let id = surface.create_image_from_rgba(128, 64, pixels(stage));
                let (x, y, w, h) = if protocol.ends_with("-full") {
                    (0, 0, usize::from(width), usize::from(height))
                } else if stage == 0 {
                    (2, 2, 8, 4)
                } else {
                    (12, 6, 8, 4)
                };
                let behind = protocol.contains("behind");
                surface.try_place_image_region(
                    x,
                    y,
                    w,
                    h,
                    id,
                    128,
                    64,
                    if behind { -1 } else { 1 },
                    1.0,
                )?;
                if behind {
                    let mut cell = surface.get(x + 1, y + 1);
                    cell.ch = 'A';
                    cell.fg = Rgba::white();
                    surface.set(x + 1, y + 1, cell);
                }
            }
            for (x, ch) in format!("IMAGE SURFACE STAGE {stage}").chars().enumerate() {
                surface.set(
                    x,
                    0,
                    Cell {
                        ch,
                        fg: Rgba::white(),
                        bg: Rgba::black(),
                        ..Default::default()
                    },
                );
            }
            #[cfg(target_os = "linux")]
            if protocol.contains("retry") && stage == 0 {
                fail_surface_frame(&mut renderer)?;
                renderer.begin_frame()?;
            }
            renderer.end_frame()?;
            shown = Some(stage);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(ReactiveError::invalid_state(
        "Surface fixture watchdog expired",
    ))
}

#[cfg(target_os = "linux")]
fn fail_surface_frame(renderer: &mut reactive_tui::core::renderer::Renderer) -> Result<()> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    // This fixture is an isolated child inside the owned host. Redirect only its
    // stdout to a device that rejects writes, then restore the terminal FD.
    std::io::stdout().flush()?;
    let saved = unsafe { libc::dup(libc::STDOUT_FILENO) };
    if saved < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    // SAFETY: dup returned a new owned descriptor above.
    let saved = unsafe { OwnedFd::from_raw_fd(saved) };
    let failed = std::fs::OpenOptions::new().write(true).open("/dev/full")?;
    // SAFETY: both descriptors remain alive during the call.
    if unsafe { libc::dup2(failed.as_raw_fd(), libc::STDOUT_FILENO) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let result = renderer.end_frame();
    // SAFETY: restore the saved stdout descriptor before inspecting the result.
    if unsafe { libc::dup2(saved.as_raw_fd(), libc::STDOUT_FILENO) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    if result.is_ok() {
        return Err(ReactiveError::invalid_state(
            "Surface frame ignored failed output",
        ));
    }
    eprintln!("Surface buffered output failure was reported; retrying the same frame");
    Ok(())
}

fn standalone(command: PathBuf, protocol: &str) -> Result<()> {
    let mut out = std::io::stdout();
    out.write_all(b"\x1b[?1049h\x1b[?25l\x1b[40m")?;
    let result = standalone_stages(&command, protocol, &mut out);
    let restore = out
        .write_all(b"\x1b[0m\x1b[?25h\x1b[?1049l")
        .and_then(|()| out.flush());
    result?;
    restore?;
    Ok(())
}

fn standalone_stages(
    command: &std::path::Path,
    protocol: &str,
    out: &mut impl Write,
) -> Result<()> {
    use reactive_tui::platform::{
        image::{DrawOptions, Image},
        ImageFormat,
    };
    let started = Instant::now();
    let mut shown = None;
    while started.elapsed() < Duration::from_secs(45) {
        let stage: u8 = std::fs::read_to_string(command)?
            .trim()
            .parse()
            .map_err(|_| ReactiveError::invalid_state("invalid image fixture stage"))?;
        if stage == 3 {
            return Ok(());
        }
        if shown != Some(stage) {
            out.write_all(b"\x1b[2J\x1b[H")?;
            write!(out, "IMAGE PROTOCOL STAGE {stage}")?;
            if stage < 2 {
                let pixels =
                    image::RgbaImage::from_raw(128, 64, pixels(stage)).expect("fixture extent");
                let mut png = std::io::Cursor::new(Vec::new());
                pixels
                    .write_to(&mut png, image::ImageFormat::Png)
                    .map_err(|e| ReactiveError::ImageProcessing(e.to_string()))?;
                let image = Image::from_memory(png.into_inner(), ImageFormat::Png)?;
                out.write_all(if stage == 0 {
                    b"\x1b[3;3H"
                } else {
                    b"\x1b[9;25H"
                })?;
                match protocol {
                    "kitty" => image.render_kitty(out, &DrawOptions::default())?,
                    "sixel" => image.render_sixel(out, &DrawOptions::default())?,
                    "iterm" => image.render_iterm2(out, &DrawOptions::default())?,
                    _ => return Err(ReactiveError::invalid_state("unknown fixture protocol")),
                }
            }
            out.flush()?;
            shown = Some(stage);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(ReactiveError::invalid_state(
        "image protocol fixture watchdog expired",
    ))
}
