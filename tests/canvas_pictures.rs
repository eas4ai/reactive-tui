//! canvas-pictures mechanism, GFX-009 (docs/spec/canvas.md): on a terminal
//! that takes pixels, the App does not wait in `present` while a canvas's
//! picture is made ready. Pictures are made ready on a picture thread or on
//! the canvas's own worker, at most one picture of a canvas waits, a picture
//! is never written over cells that something now covers, and in a release
//! build the App's wait in `present` stays within a frame while a new picture
//! of 1920 by 960 pixels arrives every frame. The mechanism runs every test
//! here in a release build, ignored ones included, on this host, the macOS
//! host and the Windows tablet; the two ignored tests are for release builds
//! only.

mod canvas_support;

use canvas_support::reference_options;
use reactive_tui::app::{App, AppWaker, RootComponent, RootUpdate};
use reactive_tui::backend::{Backend, FrameLayout, ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::event::types::Event;
use reactive_tui::graphics::{Canvas, CanvasProps};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One frame at 60 frames a second.
const BOUND: Duration = Duration::from_micros(16_600);
/// How the crate's Sixel pictures start: the device control string with a
/// transparent background.
const SIXEL: &str = "\x1bP0;1q";
/// The pixels of one cell, as the backend draws them when no terminal
/// reports its own.
const CELL: (usize, usize) = (8, 16);

/// The pixel outputs this host's build offers: Kitty graphics in the
/// command and Sixel everywhere, Kitty through POSIX shared memory on Unix.
fn outputs() -> Vec<(&'static str, ImageOutputOptions)> {
    let kitty = ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    };
    let mut outputs = vec![("Kitty, in the command", kitty)];
    if cfg!(unix) {
        outputs.push((
            "Kitty, shared memory",
            ImageOutputOptions {
                kitty_shared_memory: true,
                ..kitty
            },
        ));
    }
    outputs.push((
        "Sixel",
        ImageOutputOptions {
            sixel: true,
            ..Default::default()
        },
    ));
    outputs
}

/// A terminal that counts the pictures it is sent and, when asked to,
/// keeps every byte.
#[derive(Clone, Default)]
struct Terminal {
    pictures: Arc<AtomicUsize>,
    kept: Option<Arc<Mutex<Vec<u8>>>>,
}
impl Terminal {
    fn keeping() -> Self {
        Self {
            kept: Some(Arc::default()),
            ..Self::default()
        }
    }
}
impl Write for Terminal {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A Kitty picture begins with its action, a Sixel picture with the
        // device control string the crate writes. A picture is megabytes,
        // so the marks are found the fast way: this terminal is the App's
        // memory, not a slow link.
        let pictures = memchr::memchr_iter(0x1b, bytes)
            .filter(|&at| {
                [&b"\x1b_Ga=T"[..], SIXEL.as_bytes()]
                    .iter()
                    .any(|mark| bytes[at..].starts_with(mark))
            })
            .count();
        self.pictures.fetch_add(pictures, Ordering::SeqCst);
        if let Some(kept) = &self.kept {
            kept.lock().unwrap().extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// One present: when it began, how long the App waited in it, the pictures
/// the terminal had been sent when it returned, and the frame the root had
/// rendered.
struct Present {
    began: Instant,
    waited: Duration,
    pictures: usize,
    frame: usize,
}

/// What the timed backend saw: every present, and the threads that made
/// the canvas pictures ready.
#[derive(Default)]
struct Log {
    presents: Vec<Present>,
    threads: BTreeMap<String, u64>,
}

/// The default backend with each present timed and logged.
struct Timed {
    inner: SuprTuiBackend,
    pictures: Arc<AtomicUsize>,
    frame: Arc<AtomicUsize>,
    log: Arc<Mutex<Log>>,
}
impl Backend for Timed {
    fn painted_nodes(&self) -> Option<&[reactive_tui::backend::PaintedNode]> {
        self.inner.painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[reactive_tui::backend::PresentedLayout]> {
        self.inner.component_layouts()
    }
    fn hit_cells(&self) -> Option<&[u32]> {
        self.inner.hit_cells()
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn layout_frame(&mut self, element: Arc<Element>) -> Result<Option<FrameLayout>> {
        self.inner.layout_frame(element)
    }
    fn apply_patches(
        &mut self,
        patches: &[reactive_tui::render::reconcile::PatchOp],
        tree: &reactive_tui::render::RenderTree,
    ) -> Result<()> {
        self.inner.apply_patches(patches, tree)
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn present(&mut self) -> Result<()> {
        let started = Instant::now();
        self.inner.present()?;
        let waited = started.elapsed();
        // No sync: it waits for the pictures being made ready, which the
        // App never does.
        let mut log = self.log.lock().unwrap();
        log.presents.push(Present {
            began: started,
            waited,
            pictures: self.pictures.load(Ordering::SeqCst),
            frame: self.frame.load(Ordering::SeqCst),
        });
        log.threads = self.inner.picture_threads().clone();
        Ok(())
    }
    fn sync(&mut self) -> Result<()> {
        self.inner.sync()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<Event>> {
        self.inner.poll_event(timeout_ms)
    }
    fn poll_event_with_wake(
        &mut self,
        timeout: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        self.inner.poll_event_with_wake(timeout, wake)
    }
}

/// A root that shows the cube at a new angle every frame while `turn`
/// says so, then holds the last angle for `hold` and stops. Without a
/// note the canvas fills the screen; with one it sits beside a label, and
/// from frame `note` on a note covers part of it.
struct Turning {
    frame: usize,
    turn: Box<dyn Fn(usize) -> bool + Send + Sync>,
    hold: Duration,
    held: Option<Instant>,
    note: Option<usize>,
    /// The frame the root last rendered, for the log.
    shown: Arc<AtomicUsize>,
}
impl RootComponent for Turning {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        self.shown.store(self.frame, Ordering::SeqCst);
        let angle = self.frame as f32 * 0.02;
        let scene = canvas_support::cube_in_view(angle, angle * 0.7);
        let props = CanvasProps::new(Arc::new(scene))
            .view(canvas_support::SIZE.0 as f32, canvas_support::SIZE.1 as f32)
            .options(reference_options(false));
        let canvas = Element::typed::<Canvas>(props);
        let Some(note) = self.note else {
            return canvas;
        };
        let mut right = vec![canvas];
        if self.frame >= note {
            right.push(
                div()
                    .class("absolute left-4 top-2 w-12 h-3 bg-blue-900")
                    .text("a note")
                    .build(),
            );
        }
        div()
            .class("flex flex-row w-full h-full")
            .children(vec![
                Element::text("left side").with_class("w-20 h-full"),
                div().class("relative w-60 h-full").children(right).build(),
            ])
            .build()
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if (self.turn)(self.frame + 1) {
            self.frame += 1;
            return Ok(RootUpdate::Redraw);
        }
        let held = *self.held.get_or_insert_with(Instant::now);
        Ok(if held.elapsed() >= self.hold {
            RootUpdate::Exit
        } else {
            RootUpdate::Unchanged
        })
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// Run `root` on a backend of `size` cells that writes to `terminal` and
/// takes `images`, and return what the timed backend saw.
fn run(
    size: (u16, u16),
    images: ImageOutputOptions,
    terminal: &Terminal,
    turn: Box<dyn Fn(usize) -> bool + Send + Sync>,
    hold: Duration,
    note: Option<usize>,
) -> Log {
    let log = Arc::new(Mutex::new(Log::default()));
    let shown = Arc::new(AtomicUsize::new(0));
    let inner = SuprTuiBackend::with_writer_and_images(size.0, size.1, terminal.clone(), images)
        .expect("a backend that writes to memory");
    App::builder()
        .backend(Timed {
            inner,
            pictures: Arc::clone(&terminal.pictures),
            frame: Arc::clone(&shown),
            log: Arc::clone(&log),
        })
        .root(Turning {
            frame: 0,
            turn,
            hold,
            held: None,
            note,
            shown,
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let mut seen = log.lock().unwrap();
    std::mem::take(&mut *seen)
}

/// The middle one of `samples`, in milliseconds.
fn median(mut samples: Vec<Duration>) -> f64 {
    samples.sort();
    samples[samples.len() / 2].as_secs_f64() * 1e3
}

/// The 95th percentile of `samples`.
fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() * 95 / 100]
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_009_a_canvas_picture_is_made_ready_on_a_picture_or_canvas_thread() {
    let mut seen = Vec::new();
    for (name, images) in outputs() {
        let terminal = Terminal::default();
        let pictures = Arc::clone(&terminal.pictures);
        let deadline = Instant::now() + Duration::from_secs(60);
        let log = run(
            (40, 12),
            images,
            &terminal,
            Box::new(move |_| pictures.load(Ordering::SeqCst) < 5 && Instant::now() < deadline),
            Duration::from_millis(500),
            None,
        );
        seen.push((name, log.threads));
    }
    let named =
        |thread: &str| thread.starts_with("rtui-picture-") || thread.starts_with("rtui-canvas-");
    assert!(
        seen.iter()
            .all(|(_, threads)| !threads.is_empty() && threads.keys().all(|thread| named(thread))),
        "GFX-009: canvas pictures were made ready on these threads, by output: {seen:?}"
    );
}

#[test]
#[serial_test::serial(gpu)]
#[ignore = "a release-build test; canvas-pictures runs it"]
fn gfx_009_at_most_one_picture_of_a_canvas_waits() {
    // Sixel pictures of 1920 by 960 pixels take longer to make ready than
    // the canvas takes to draw them, so pictures arrive while one is made
    // ready. After the root's last angle is presented, the terminal may
    // still be sent the picture being made ready, the one picture that
    // waits, and the canvas's own two: the scene it draws and the one that
    // waits for it (GFX-003). A queue of waiting pictures sends more.
    let terminal = Terminal::default();
    let turns = 60;
    let log = run(
        (240, 60),
        ImageOutputOptions {
            sixel: true,
            ..Default::default()
        },
        &terminal,
        Box::new(move |frame| frame < turns),
        Duration::from_secs(3),
        None,
    );
    let total = terminal.pictures.load(Ordering::SeqCst);
    let last = log
        .presents
        .iter()
        .find(|present| present.frame == turns - 1)
        .map(|present| present.pictures);
    let after = last.map(|sent| total - sent);
    assert!(
        last.is_some_and(|sent| sent > 0) && after.is_some_and(|after| after <= 4),
        "GFX-009: of {total} Sixel pictures, {last:?} had reached the terminal when the root's last angle was presented and {after:?} came after it"
    );
}

#[test]
#[serial_test::serial(gpu)]
fn gfx_009_a_picture_is_never_written_over_cells_that_cover_its_canvas() {
    // The canvas is 60 by 24 cells beside a label of 20; from frame 10 on, a
    // note covers the canvas's cells 4 to 15 of rows 2 to 4.
    let terminal = Terminal::keeping();
    let covered = 10;
    // The root turns on until three pictures came after the note did, so
    // the pictures of a busy host are waited for, not counted in frames.
    let pictures = Arc::clone(&terminal.pictures);
    let noted = Arc::new(AtomicUsize::new(usize::MAX));
    let deadline = Instant::now() + Duration::from_secs(30);
    run(
        (80, 24),
        ImageOutputOptions {
            sixel: true,
            ..Default::default()
        },
        &terminal,
        Box::new(move |frame| {
            if frame <= covered {
                return true;
            }
            let sent = pictures.load(Ordering::SeqCst);
            let _ = noted.compare_exchange(usize::MAX, sent, Ordering::SeqCst, Ordering::SeqCst);
            sent < noted.load(Ordering::SeqCst).saturating_add(3) && Instant::now() < deadline
        }),
        Duration::from_secs(1),
        Some(covered),
    );
    let kept = terminal.kept.as_ref().unwrap().lock().unwrap();
    let output = String::from_utf8_lossy(&kept);
    let noted = output
        .find("a note")
        .expect("the note reaches the terminal");
    let later = &output[noted..];
    let mut pictures = 0;
    let mut over = Vec::new();
    for (index, _) in later.match_indices(SIXEL) {
        pictures += 1;
        // The picture is drawn from the cursor, which the last cursor move
        // before it put on the canvas's corner.
        let before = &later[..index];
        let Some(origin) = last_cursor_move(before) else {
            over.push((index, "no cursor move before the picture".to_owned()));
            continue;
        };
        let (size, set) = canvas_support::sixel_pixels(&later[index + SIXEL.len()..]);
        // The note's cells, columns 24 to 35 and rows 2 to 4 of the screen.
        let note = |x: usize, y: usize| {
            (24..36).contains(&(origin.1 + x / CELL.0)) && (2..5).contains(&(origin.0 + y / CELL.1))
        };
        let inside = (0..size.1)
            .flat_map(|y| (0..size.0).map(move |x| (x, y)))
            .filter(|&(x, y)| set[y * size.0 + x] && note(x, y))
            .count();
        if inside > 0 {
            over.push((index, format!("{inside} pixels set over the note")));
        }
    }
    assert!(
        pictures >= 2 && over.is_empty(),
        "GFX-009: of {pictures} Sixel pictures written after the note, these drew over it: {:?}",
        &over[..over.len().min(5)]
    );
}

/// The zero-based row and column of the last cursor move in `text`.
fn last_cursor_move(text: &str) -> Option<(usize, usize)> {
    text.rmatch_indices("\x1b[").find_map(|(at, _)| {
        let part = &text[at + 2..];
        let digits: String = part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ';')
            .collect();
        if !part[digits.len()..].starts_with('H') {
            return None;
        }
        let mut numbers = digits.split(';').map(|n| n.parse::<usize>().unwrap_or(1));
        let row = numbers.next().unwrap_or(1);
        let column = numbers.next().unwrap_or(1);
        Some((row.saturating_sub(1), column.saturating_sub(1)))
    })
}

#[test]
#[serial_test::serial(gpu)]
#[ignore = "a release-build bound; canvas-pictures runs it on the three hosts"]
fn gfx_009_the_apps_wait_stays_within_a_frame_with_a_new_picture_every_frame() {
    // The first five pictures warm the backend and the terminal's shared
    // memory up; the next 30 are measured.
    const WARM: usize = 5;
    const MEASURED: usize = 30;
    let mut failures = Vec::new();
    for (name, images) in outputs() {
        let terminal = Terminal::default();
        let pictures = Arc::clone(&terminal.pictures);
        let started = Instant::now();
        let deadline = started + Duration::from_secs(180);
        // The root turns until the pictures have come, then holds its last
        // angle while those being made ready arrive.
        let log = run(
            (240, 60),
            images,
            &terminal,
            Box::new(move |_| {
                pictures.load(Ordering::SeqCst) < WARM + MEASURED && Instant::now() < deadline
            }),
            Duration::from_millis(500),
            None,
        );
        let total = terminal.pictures.load(Ordering::SeqCst);
        let measured: Vec<&Present> = log
            .presents
            .iter()
            .skip_while(|present| present.pictures < WARM)
            .collect();
        let sent = total.saturating_sub(measured.first().map_or(total, |first| first.pictures));
        if sent < MEASURED {
            failures.push(format!(
                "{name}: {sent} pictures reached the terminal after the first {WARM} in {:.0} s, fewer than {MEASURED}",
                started.elapsed().as_secs_f64()
            ));
            continue;
        }
        let waits: Vec<Duration> = measured.iter().map(|present| present.waited).collect();
        let worst = p95(waits.clone());
        let span = measured
            .last()
            .zip(measured.first())
            .map_or(Duration::ZERO, |(last, first)| last.began - first.began);
        println!(
            "GFX-009 the App's wait in present with a new picture of 1920 by 960 pixels every frame, as {name}: median {:.2} ms, p95 {:.2} ms over {} presents and {sent} pictures in {:.2} s",
            median(waits.clone()),
            worst.as_secs_f64() * 1e3,
            waits.len(),
            span.as_secs_f64()
        );
        if worst > BOUND {
            failures.push(format!("{name}: p95 {:.2} ms", worst.as_secs_f64() * 1e3));
        }
    }
    assert!(
        failures.is_empty(),
        "GFX-009: the App's wait in present exceeded 16.6 ms at the 95th percentile, or too few pictures came: {failures:?}"
    );
}
