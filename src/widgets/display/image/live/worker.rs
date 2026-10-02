//! One active decode and one replaceable pending source per retained image.
use super::super::{ExternalRenderer, Image, ImageDisplayMode, ImageFormat, ImageProcessor};
use super::animation::{self, Animation};
use crate::reactive::ThreadSafeSignal;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Instant;

struct Request {
    id: u64,
    image: Arc<Image>,
    format: Option<ImageFormat>,
    size: Option<(u32, u32)>,
    pixels: Option<Arc<image::RgbaImage>>,
}
pub(super) struct Response {
    pub id: u64,
    pub cells: Option<super::Cells>,
    pub result: Result<Arc<image::RgbaImage>, String>,
}
#[derive(Default)]
struct Slots {
    request: Option<Request>,
    response: Option<Response>,
}
struct Shared {
    slots: Mutex<Slots>,
    ready: Condvar,
    closed: AtomicBool,
    generation: AtomicU64,
    changed: ThreadSafeSignal<u64>,
}
pub(super) struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn new() -> std::io::Result<Self> {
        let shared = Arc::new(Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
            generation: AtomicU64::new(0),
            changed: ThreadSafeSignal::new(0),
        });
        let owner = shared.clone();
        let thread = thread::Builder::new()
            .name("image-loader".into())
            .spawn(move || run(owner))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }
    pub fn observe(&self) {
        self.shared.changed.get();
    }
    #[cfg(test)]
    pub fn submit(&self, image: Arc<Image>, format: Option<ImageFormat>) -> u64 {
        self.submit_at(image, format, None, None)
    }
    pub fn submit_at(
        &self,
        image: Arc<Image>,
        format: Option<ImageFormat>,
        size: Option<(u32, u32)>,
        pixels: Option<Arc<image::RgbaImage>>,
    ) -> u64 {
        let id = self.shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let mut slots = self.shared.slots.lock().unwrap();
        slots.response = None;
        slots.request = Some(Request {
            id,
            image,
            format,
            size,
            pixels,
        });
        drop(slots);
        self.shared.ready.notify_one();
        id
    }
    pub fn take(&self) -> Option<Response> {
        self.shared.slots.lock().unwrap().response.take()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared.slots.lock().unwrap().request = None;
        self.shared.ready.notify_one();
        if self
            .thread
            .take()
            .is_some_and(|thread| thread.join().is_err())
        {
            log::error!("Image loading worker panicked during shutdown");
        }
    }
}
struct Active {
    request: Request,
    animation: Arc<Animation>,
    started: Instant,
    deadline: Option<Instant>,
    index: Option<usize>,
    mode: ImageDisplayMode,
}
enum Wake {
    Request(Request),
    Frame,
    Closed,
}
fn wait(shared: &Shared, deadline: Option<Instant>) -> Wake {
    let mut slots = shared.slots.lock().unwrap();
    loop {
        if shared.closed.load(Ordering::Acquire) {
            return Wake::Closed;
        }
        if let Some(request) = slots.request.take() {
            return Wake::Request(request);
        }
        slots = if let Some(deadline) = deadline {
            let now = Instant::now();
            if now >= deadline {
                return Wake::Frame;
            }
            shared.ready.wait_timeout(slots, deadline - now).unwrap().0
        } else {
            shared.ready.wait(slots).unwrap()
        };
    }
}
fn cancelled(shared: &Shared, id: u64) -> bool {
    shared.closed.load(Ordering::Acquire) || shared.generation.load(Ordering::Acquire) != id
}
fn publish(shared: &Shared, response: Response) {
    let mut slots = shared.slots.lock().unwrap();
    if cancelled(shared, response.id) {
        return;
    }
    slots.response = Some(response);
    drop(slots);
    shared
        .changed
        .update(|revision| *revision = revision.wrapping_add(1));
}
fn activate(
    request: Request,
    old: Option<Active>,
    shared: &Shared,
    automatic: &mut Option<ImageDisplayMode>,
) -> Result<Active, String> {
    let reusable = old.filter(|old| {
        request.pixels.is_some()
            || (old.request.image.source == request.image.source
                && old.request.format == request.format)
    });
    let (animation, started) = if let Some(old) = reusable {
        (old.animation, old.started)
    } else if let Some(pixels) = &request.pixels {
        (Arc::new(Animation::still(pixels.clone())), Instant::now())
    } else {
        (
            animation::load(&request.image.source, request.format, || {
                cancelled(shared, request.id)
            })
            .map_err(|error| error.to_string())?,
            Instant::now(),
        )
    };
    let mode = if request.image.display_mode == ImageDisplayMode::Auto {
        if let Some(mode) = *automatic {
            mode
        } else {
            let mode = automatic_mode(|| cancelled(shared, request.id));
            if !cancelled(shared, request.id) {
                *automatic = Some(mode);
            }
            mode
        }
    } else {
        request.image.display_mode
    };
    Ok(Active {
        request,
        animation,
        started,
        deadline: None,
        index: None,
        mode,
    })
}
fn render_cells(
    active: &Active,
    pixels: &image::RgbaImage,
    shared: &Shared,
) -> Result<Option<super::Cells>, String> {
    let request = &active.request;
    let Some((width, height)) = request.size.filter(|(w, h)| *w > 0 && *h > 0) else {
        return Ok(None);
    };
    match active.mode {
        ImageDisplayMode::Chafa | ImageDisplayMode::Viu => {}
        // Auto without an installed tool draws with the renderer's
        // blitters, chosen by tier (BLT-002).
        ImageDisplayMode::Auto => {
            let (width, height) = request
                .image
                .size_constraints
                .map_or((width, height), |(w, h)| (w.min(width), h.min(height)));
            if width == 0 || height == 0 {
                return Ok(None);
            }
            let grid = super::blocks::grid(
                pixels,
                &request.image,
                (width, height),
                super::super::image_blitter(),
            )?;
            return Ok(Some(super::Cells::Blitted(Arc::new(grid))));
        }
        // The fallback text is fixed; the widget shows it without pixels.
        ImageDisplayMode::Fallback => return Ok(None),
        _ => return ascii(pixels, request, (width, height)).map(Some),
    }
    let config = Image {
        source: super::super::super::ImageSource::FilePath(Default::default()),
        display_mode: active.mode,
        size_constraints: Some(
            request
                .image
                .size_constraints
                .map_or((width, height), |(w, h)| (w.min(width), h.min(height))),
        ),
        preserve_aspect: request.image.preserve_aspect,
        fallback_text: None,
        background_color: request.image.background_color,
        quality: request.image.quality,
    };
    super::cells::validate_size(width, height)?;
    let output = ExternalRenderer::new()
        .render_pixels(pixels, &config, || cancelled(shared, request.id))
        .map_err(|error| error.to_string())?;
    Ok(Some(super::Cells::Captured(Arc::new(super::cells::parse(
        &output, width, height,
    )?))))
}
/// ASCII art of `pixels` filling `size` cells: the picture in AsciiArt mode,
/// and the text a pixel protocol falls back to where the host cannot show
/// it.
fn ascii(
    pixels: &image::RgbaImage,
    request: &Request,
    size: (u32, u32),
) -> Result<super::Cells, String> {
    let config = Image {
        source: super::super::super::ImageSource::FilePath(Default::default()),
        display_mode: ImageDisplayMode::AsciiArt,
        size_constraints: Some(size),
        preserve_aspect: request.image.preserve_aspect,
        background_color: request.image.background_color,
        quality: request.image.quality,
        ..Image::default()
    };
    ImageProcessor::new()
        .ascii_from_pixels(pixels, &config)
        .map(|text| super::Cells::Text(Arc::from(text)))
        .map_err(|error| error.to_string())
}
fn run(shared: Arc<Shared>) {
    let mut automatic = None;
    let mut active: Option<Active> = None;
    loop {
        match wait(&shared, active.as_ref().and_then(|active| active.deadline)) {
            Wake::Closed => return,
            Wake::Frame => {}
            Wake::Request(request) => {
                let id = request.id;
                match activate(request, active.take(), &shared, &mut automatic) {
                    Ok(next) => active = Some(next),
                    Err(error) => {
                        publish(
                            &shared,
                            Response {
                                id,
                                cells: None,
                                result: Err(error),
                            },
                        );
                        continue;
                    }
                }
            }
        }
        let Some(active) = &mut active else {
            continue;
        };
        let (index, next) = active.animation.at(active.started.elapsed());
        active.deadline = if active.request.size.is_some_and(|(w, h)| w > 0 && h > 0)
            && active.mode != ImageDisplayMode::Fallback
            && !active.request.image.has_empty_size()
        {
            next.and_then(|delay| active.started.checked_add(delay))
        } else {
            None
        };
        if active.index == Some(index) {
            continue;
        }
        let pixels = active.animation.frames[index].pixels.clone();
        let response = match render_cells(active, &pixels, &shared) {
            Ok(cells) => {
                active.index = Some(index);
                Response {
                    id: active.request.id,
                    cells,
                    result: Ok(pixels),
                }
            }
            Err(error) => {
                active.deadline = None;
                Response {
                    id: active.request.id,
                    cells: None,
                    result: Err(error),
                }
            }
        };
        publish(&shared, response);
    }
}

/// Auto's fallback: an installed chafa, then viu, and otherwise `Auto`
/// itself, which the worker draws with the renderer's blitters. A host
/// with a pixel protocol gets the blitters too: the backend remains the
/// authority for the graphics and the fallback shows where it does not.
/// A blitter the application or the environment named also means the
/// blitters: an external tool would ignore the choice (BLT-002).
fn automatic_mode(cancelled: impl Fn() -> bool) -> ImageDisplayMode {
    if super::super::named_blitter().is_some() {
        return ImageDisplayMode::Auto;
    }
    let caps = crate::core::capabilities::TerminalQuery::detect_from_env();
    if caps.sixel || caps.kitty_graphics || caps.iterm2_graphics {
        return ImageDisplayMode::Auto;
    }
    if ExternalRenderer::available("chafa", &cancelled) {
        ImageDisplayMode::Chafa
    } else if !cancelled() && ExternalRenderer::available("viu", cancelled) {
        ImageDisplayMode::Viu
    } else {
        ImageDisplayMode::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn receive(worker: &Worker) -> Response {
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(response) = worker.take() {
                return response;
            }
            assert!(Instant::now() < deadline, "image worker did not respond");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn api_image_gif_worker_stops_finite_playback_and_replaces_infinite_source() {
        use image::{
            codecs::gif::{GifEncoder, Repeat},
            Delay, Frame,
        };
        for mode in ["finite", "replace", "hidden"] {
            let mut bytes = Vec::new();
            {
                let mut encoder = GifEncoder::new(&mut bytes);
                if mode != "finite" {
                    encoder.set_repeat(Repeat::Infinite).unwrap();
                }
                for color in [[255, 0, 0, 255], [0, 255, 0, 255]] {
                    encoder
                        .encode_frame(Frame::from_parts(
                            image::RgbaImage::from_pixel(1, 1, image::Rgba(color)),
                            0,
                            0,
                            Delay::from_numer_denom_ms(300, 1),
                        ))
                        .unwrap();
                }
            }
            let worker = Worker::new().unwrap();
            let owner = Arc::downgrade(&worker.shared);
            let id = worker.submit_at(
                Arc::new(
                    Image::from_raw_bytes(bytes, 0, 0, ImageFormat::GIF)
                        .with_display_mode(ImageDisplayMode::AsciiArt)
                        .with_max_size(if mode == "hidden" { 0 } else { 4 }, 2),
                ),
                None,
                Some((4, 2)),
                None,
            );
            let first = receive(&worker);
            assert_eq!(first.id, id);
            assert_eq!(first.result.unwrap().as_raw(), &[255, 0, 0, 255]);
            if mode == "hidden" {
                // The behavior under test: a hidden image sends no more frames.
                thread::sleep(Duration::from_millis(400));
                assert!(worker.take().is_none(), "zero-sized image kept animating");
                drop(worker);
                assert!(owner.upgrade().is_none());
                continue;
            }
            let second = receive(&worker);
            let second = second.result.unwrap();
            assert_eq!(second.as_raw(), &[0, 255, 0, 255]);
            let pixels = Arc::downgrade(&second);
            drop(second);
            if mode == "replace" {
                let id = worker.submit_at(
                    Arc::new(
                        Image::from_raw_bytes(vec![0, 0, 255, 255], 1, 1, ImageFormat::RGBA8888)
                            .with_display_mode(ImageDisplayMode::AsciiArt),
                    ),
                    None,
                    Some((4, 2)),
                    None,
                );
                let response = receive(&worker);
                assert_eq!(response.id, id);
                assert_eq!(response.result.unwrap().as_raw(), &[0, 0, 255, 255]);
                assert!(
                    pixels.upgrade().is_none(),
                    "replaced GIF retained its frame pixels"
                );
            }
            // The behavior under test: the replaced image sends no more frames.
            thread::sleep(Duration::from_millis(400));
            assert!(
                worker.take().is_none(),
                "finished/replaced GIF kept publishing frames"
            );
            drop(worker);
            assert!(owner.upgrade().is_none());
        }
    }

    #[test]
    fn api_image_worker_publishes_latest_source_and_releases_ownership() {
        let worker = Worker::new().unwrap();
        let owner = Arc::downgrade(&worker.shared);
        for value in 0..100 {
            worker.submit(
                Arc::new(Image::from_raw_bytes(
                    vec![value; 3],
                    1,
                    1,
                    ImageFormat::RGB888,
                )),
                None,
            );
        }
        let id = worker.submit(
            Arc::new(Image::from_raw_bytes(
                vec![128; 3],
                1,
                1,
                ImageFormat::RGB888,
            )),
            None,
        );
        let response = receive(&worker);
        assert_eq!(response.id, id);
        assert_eq!(response.result.unwrap().as_raw(), &[128, 128, 128, 255]);
        drop(worker);
        assert!(
            owner.upgrade().is_none(),
            "image worker still owns its state after removal"
        );
    }

    #[test]
    fn api_image_worker_reports_current_errors_and_joins_pending_work() {
        let worker = Worker::new().unwrap();
        let owner = Arc::downgrade(&worker.shared);
        let id = worker.submit(
            Arc::new(Image::from_raw_bytes(
                vec![0; 3],
                1,
                1,
                ImageFormat::RGBA8888,
            )),
            None,
        );
        assert_eq!(receive(&worker).id, id);
        let id = worker.submit(Arc::new(Image::from_base64("%%%")), None);
        let response = receive(&worker);
        assert_eq!(response.id, id);
        assert!(response.result.unwrap_err().contains("base64"));
        worker.submit(
            Arc::new(Image::from_raw_bytes(
                vec![0; 3 * 1024 * 1024],
                1024,
                1024,
                ImageFormat::RGB888,
            )),
            None,
        );
        drop(worker);
        assert!(owner.upgrade().is_none());
    }
}

#[cfg(all(test, unix))]
mod external_tests {
    use super::super::super::ImageDisplayMode;
    use super::*;
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        process::Command,
        time::{Duration, Instant},
    };

    fn wait_for(mut ready: impl FnMut() -> bool) {
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let end = Instant::now() + Duration::from_secs(30);
        while !ready() {
            assert!(
                Instant::now() < end,
                "external image worker did not make progress"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    /// Draw `pixels` through the widget's block fallback into 12 by 6 cells.
    fn blitted(pixels: Arc<image::RgbaImage>) -> Arc<crate::layout::paint_tree::cells::CellGrid> {
        let shared = Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
            generation: AtomicU64::new(1),
            changed: ThreadSafeSignal::new(0),
        };
        let image = Image {
            display_mode: ImageDisplayMode::Auto,
            // Nearest scaling keeps the edge sharp, so partial blocks remain.
            quality: crate::widgets::display::image::ImageQuality::Fast,
            ..Image::default()
        };
        let active = Active {
            request: Request {
                id: 1,
                image: Arc::new(image),
                format: None,
                size: Some((12, 6)),
                pixels: Some(pixels.clone()),
            },
            animation: Arc::new(Animation::still(pixels.clone())),
            started: Instant::now(),
            deadline: None,
            index: None,
            mode: ImageDisplayMode::Auto,
        };
        let Some(super::super::Cells::Blitted(grid)) =
            render_cells(&active, &pixels, &shared).unwrap()
        else {
            panic!("Auto without a tool must draw with the blitters");
        };
        grid
    }

    /// BLT-002 where images are drawn: the widget's block fallback draws with
    /// the blitter the application chose. A diagonal two-color split leaves
    /// partial blocks along the edge, which each tier draws with its own
    /// glyphs, so an ignored override shows in the cells.
    #[test]
    fn blt_002_the_widget_draws_with_the_blitter_the_application_chose() {
        let _serial = crate::widgets::display::charts::GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if std::env::var_os(::suprtui::blit::BLITTER_ENV).is_some() {
            eprintln!("SKIP: the environment override wins over the application's");
            return;
        }
        /// Clears the override even if an assertion fails.
        struct Cleared;
        impl Drop for Cleared {
            fn drop(&mut self) {
                crate::widgets::display::set_image_blitter(None);
            }
        }
        let _cleared = Cleared;
        let pixels = Arc::new(image::RgbaImage::from_fn(96, 96, |x, y| {
            image::Rgba(if x > y {
                [220, 30, 30, 255]
            } else {
                [30, 30, 220, 255]
            })
        }));
        let within = |glyph: &str, low: u32, high: u32| {
            glyph
                .chars()
                .next()
                .is_some_and(|c| (low..=high).contains(&u32::from(c)))
        };
        for tier in crate::widgets::display::Blitter::TIERS {
            crate::widgets::display::set_image_blitter(Some(tier));
            let grid = blitted(pixels.clone());
            let glyphs: Vec<String> = grid
                .iter()
                .map(|(_, _, glyph, _, _)| glyph.to_string())
                .filter(|glyph| !glyph.trim().is_empty())
                .collect();
            use crate::widgets::display::Blitter::*;
            let drawn_with_the_tier = match tier {
                Braille => !glyphs.is_empty() && glyphs.iter().all(|g| within(g, 0x2800, 0x28FF)),
                Octant => glyphs.iter().any(|g| within(g, 0x1CD00, 0x1CDE5)),
                Sextant => glyphs.iter().any(|g| within(g, 0x1FB00, 0x1FB3B)),
                Quadrant => glyphs.iter().any(|g| within(g, 0x2596, 0x259F)),
                HalfBlock => {
                    glyphs.iter().any(|g| g == "▀" || g == "▄")
                        && glyphs.iter().all(|g| ["▀", "▄", "█"].contains(&g.as_str()))
                }
                Ascii => glyphs.iter().all(|g| g.is_ascii()),
            };
            assert!(
                drawn_with_the_tier,
                "{tier:?} chosen, the widget drew: {glyphs:?}"
            );
        }
    }

    #[test]
    fn blt_001_auto_without_a_tool_draws_blitted_cells() {
        let pixels = Arc::new(image::RgbaImage::from_pixel(
            8,
            8,
            image::Rgba([0, 200, 0, 255]),
        ));
        let shared = Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
            generation: AtomicU64::new(1),
            changed: ThreadSafeSignal::new(0),
        };
        let image = Image {
            display_mode: ImageDisplayMode::Auto,
            ..Image::default()
        };
        let active = Active {
            request: Request {
                id: 1,
                image: Arc::new(image),
                format: None,
                size: Some((6, 3)),
                pixels: Some(pixels.clone()),
            },
            animation: Arc::new(Animation::still(pixels.clone())),
            started: Instant::now(),
            deadline: None,
            index: None,
            mode: ImageDisplayMode::Auto,
        };
        let Some(super::super::Cells::Blitted(grid)) =
            render_cells(&active, &pixels, &shared).unwrap()
        else {
            panic!("Auto without a tool must draw with the blitters");
        };
        assert!(grid.width() > 0 && grid.width() <= 6 && grid.height() <= 3);
        assert_eq!(
            grid.background(0, 0),
            Some((0.0, 200.0 / 255.0, 0.0, 1.0)),
            "a uniform block is its color as background"
        );
    }

    /// BLT-002: a blitter the environment or the application named wins over
    /// an installed chafa or viu, which would ignore it. Each case runs in a
    /// child process whose PATH holds working fake tools.
    #[test]
    fn blt_002_a_named_blitter_wins_over_an_installed_tool() {
        const CHILD: &str = "REACTIVE_IMAGE_NAMED_BLITTER_CHILD";
        if let Ok(case) = std::env::var(CHILD) {
            if case == "application" {
                crate::widgets::display::set_image_blitter(Some(
                    crate::widgets::display::Blitter::Braille,
                ));
            }
            assert_eq!(
                automatic_mode(|| false),
                if case == "none" {
                    ImageDisplayMode::Chafa
                } else {
                    ImageDisplayMode::Auto
                },
                "{case}"
            );
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        for name in ["chafa", "viu"] {
            let program = temp.path().join(name);
            fs::write(&program, "#!/bin/sh\nprintf 'version 1'\n").unwrap();
            fs::set_permissions(program, fs::Permissions::from_mode(0o700)).unwrap();
        }
        for case in ["none", "environment", "application"] {
            let mut child = Command::new(std::env::current_exe().unwrap());
            child
                .args(["--exact", "widgets::display::image::live::worker::external_tests::blt_002_a_named_blitter_wins_over_an_installed_tool", "--nocapture"])
                .env(CHILD, case)
                .env("PATH", temp.path())
                .env("TERM", "dumb")
                .env("TERM_PROGRAM", "")
                .env_remove("KITTY_WINDOW_ID")
                .env_remove("ITERM_SESSION_ID")
                .env_remove(::suprtui::blit::BLITTER_ENV);
            if case == "environment" {
                child.env(::suprtui::blit::BLITTER_ENV, "braille");
            }
            let output = child.output().unwrap();
            assert!(
                output.status.success(),
                "{case}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn api_image_external_auto_keeps_graphics_priority_and_falls_back_between_tools() {
        const CHILD: &str = "REACTIVE_IMAGE_AUTO_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            assert_eq!(
                automatic_mode(|| false),
                match mode.as_str() {
                    "chafa" => ImageDisplayMode::Chafa,
                    "viu" => ImageDisplayMode::Viu,
                    _ => ImageDisplayMode::Auto,
                }
            );
            return;
        }
        for mode in ["chafa", "viu", "missing", "graphics"] {
            let temp = tempfile::tempdir().unwrap();
            if mode != "missing" {
                for name in ["chafa", "viu"] {
                    let program = temp.path().join(name);
                    fs::write(
                        &program,
                        if mode == "viu" && name == "chafa" {
                            "#!/bin/sh\nexit 1\n"
                        } else {
                            "#!/bin/sh\nprintf 'version 1'\n"
                        },
                    )
                    .unwrap();
                    fs::set_permissions(program, fs::Permissions::from_mode(0o700)).unwrap();
                }
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "widgets::display::image::live::worker::external_tests::api_image_external_auto_keeps_graphics_priority_and_falls_back_between_tools", "--nocapture"])
                .env(CHILD,mode).env("PATH",temp.path()).env("TERM","dumb")
                .env("TERM_PROGRAM", if mode == "graphics" { "kitty" } else { "" })
                .env_remove("KITTY_WINDOW_ID").env_remove("ITERM_SESSION_ID")
                .output().unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn api_image_external_worker_reuses_decoded_source_and_cancels_owned_child() {
        const CHILD: &str = "REACTIVE_IMAGE_WORKER_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let source = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
            image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]))
                .save(source.path())
                .unwrap();
            let config = Arc::new(
                Image::from_file(source.path()).with_display_mode(ImageDisplayMode::Chafa),
            );
            let worker = Worker::new().unwrap();
            let id = worker.submit_at(config.clone(), None, Some((4, 2)), None);
            if mode == "cancel" {
                let marker = std::env::var("REACTIVE_IMAGE_CHILD_PID").unwrap();
                wait_for(|| fs::read_to_string(&marker).is_ok_and(|s| !s.is_empty()));
                let pid: i32 = fs::read_to_string(marker).unwrap().trim().parse().unwrap();
                let start = Instant::now();
                drop(worker);
                // The behavior under test: removal kills the tool at once
                // instead of waiting out its timeout.
                assert!(
                    start.elapsed() < Duration::from_secs(1),
                    "removal waited for the tool timeout"
                );
                // kill(0) observes only; the worker must have reaped its owned process.
                assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
                return;
            }
            let mut response = None;
            wait_for(|| {
                response = worker.take();
                response.is_some()
            });
            let response = response.unwrap();
            assert_eq!(response.id, id);
            let Some(super::super::Cells::Captured(screen)) = response.cells else {
                panic!("chafa output was not captured");
            };
            assert_eq!(screen.cell(0, 0).unwrap().contents(), "R");
            let pixels = response.result.unwrap();
            source.close().unwrap();
            let id = worker.submit_at(config, None, Some((8, 3)), Some(pixels));
            let mut response = None;
            wait_for(|| {
                response = worker.take();
                response.is_some()
            });
            let response = response.unwrap();
            assert_eq!(response.id, id);
            assert!(
                response.result.is_ok(),
                "resize reloaded the deleted source"
            );
            let Some(super::super::Cells::Captured(screen)) = response.cells else {
                panic!("chafa output was not captured");
            };
            assert_eq!(screen.size(), (4, 8));
            return;
        }
        for mode in ["render", "cancel"] {
            let temp = tempfile::tempdir().unwrap();
            let program = temp.path().join("chafa");
            fs::write(&program, if mode == "cancel" {
                "#!/bin/sh\nprintf '%s' \"$$\" > \"$REACTIVE_IMAGE_CHILD_PID\"\nexec /bin/sleep 30\n"
            } else {
                "#!/bin/sh\nprintf '\\033[38;2;255;0;0mRED\\n'\n"
            }).unwrap();
            fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "widgets::display::image::live::worker::external_tests::api_image_external_worker_reuses_decoded_source_and_cancels_owned_child", "--nocapture"])
                .env(CHILD, mode).env("PATH", temp.path())
                .env("REACTIVE_IMAGE_CHILD_PID", temp.path().join("pid"))
                .output().unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
