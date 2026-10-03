//! canvas-hosts mechanism, GFX-004 (docs/spec/canvas.md): the speed floor
//! on the Windows test tablet's Intel Iris Xe. The mechanism runs this in a
//! release build on the tablet with `--ignored`; elsewhere it does not run,
//! because the bound belongs to that hardware. The scene is fitted to the
//! picture and covers all of it, so every pixel and every cell is drawn.
//!
//! The App's wait in `present` while a picture is made ready for a terminal
//! that takes pixels is GFX-009's, in tests/canvas_pictures.rs.
//!
//! GFX-011: fifteen canvases of 80 by 24 cells in one App, each given a new
//! scene every frame for two seconds, draw on one thread and each receive
//! at least 30 pictures a second. The bound binds on the Linux development
//! host, in release on the hardware adapter; on the Windows tablet and the
//! macOS host the run prints its numbers and nothing binds.

mod canvas_support;

use canvas_support::reference_options;
use reactive_tui::app::{App, RootComponent, RootUpdate};
use reactive_tui::backend::{ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::graphics::{Canvas, CanvasProps, GraphicsMode, HybridRenderer};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const FRAMES: usize = 300;
const BOUND: Duration = Duration::from_micros(16_600);

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() * 95 / 100]
}

/// The middle one of `samples`, in milliseconds.
fn median(mut samples: Vec<Duration>) -> f64 {
    samples.sort();
    samples[samples.len() / 2].as_secs_f64() * 1e3
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
#[ignore = "GFX-004's bound is for the Windows tablet; canvas-hosts runs it there in release"]
fn gfx_004_the_tablet_draws_the_animation_scene_within_a_frame() {
    let mut renderer = HybridRenderer::new(reference_options(false));
    let mut pixels = Vec::with_capacity(FRAMES);
    let mut blocks = Vec::with_capacity(FRAMES);
    // Where a picture's time goes: preparing the draws, handing them to
    // the GPU, waiting for it, and copying the picture out.
    let mut parts: [Vec<Duration>; 4] = Default::default();
    let mut mode = None;
    for frame in 0..FRAMES + 20 {
        let angle = frame as f32 * 0.02;
        let scene = canvas_support::cube_filling(angle, angle * 0.7, (1920, 960));
        let started = Instant::now();
        let picture = renderer
            .render(&scene, 1920, 960)
            .expect("a 1920 by 960 picture");
        let drew = started.elapsed();
        let started = Instant::now();
        let grid = renderer
            .render_cells(&scene, 240, 60)
            .expect("240 by 60 cells");
        let blitted = started.elapsed();
        assert_eq!((grid.width(), grid.height()), (240, 60));
        // The first frames build pipelines and atlases; they are not counted.
        if frame >= 20 {
            pixels.push(drew);
            blocks.push(blitted);
            let timings = picture.timings();
            for (part, time) in parts.iter_mut().zip([
                timings.prepare,
                timings.render,
                timings.wait,
                timings.readback,
            ]) {
                part.push(time);
            }
        }
        mode = Some(picture.mode().clone());
    }
    let mode = mode.unwrap();
    let (pixels, blocks) = (p95(pixels), p95(blocks));
    println!(
        "GFX-004 {} p95 pixels {:.2} ms blocks {:.2} ms",
        mode.label(),
        pixels.as_secs_f64() * 1e3,
        blocks.as_secs_f64() * 1e3
    );
    let [prepare, render, wait, readback] = parts.map(median);
    println!(
        "GFX-004 a picture's median time in ms: prepare {prepare:.2}, hand to the GPU {render:.2}, wait for the GPU {wait:.2}, copy out {readback:.2}"
    );
    assert!(
        matches!(mode, GraphicsMode::Gpu(_)) && pixels <= BOUND && blocks <= BOUND,
        "GFX-004: with {mode:?} the 95th percentiles of {FRAMES} frames were {pixels:?} for 1920 by 960 pixels and {blocks:?} for 240 by 60 cells of block glyphs"
    );
}

/// The canvases of GFX-011's run, in a grid of five by three.
const CANVASES: usize = 15;
/// How long the run lasts after the warm-up.
const RUN: Duration = Duration::from_secs(2);
/// The first pictures start the drawing thread and the terminal's shared
/// memory; they are not counted.
const WARM: Duration = Duration::from_millis(500);
/// The fewest new pictures a second each canvas must receive on the Linux
/// host.
const EACH: f64 = 30.0;

/// How many threads of this process are named `rtui-canvas-*`, read from
/// /proc, so Linux counts; elsewhere `None`.
fn drawing_threads() -> Option<usize> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let tasks = std::fs::read_dir("/proc/self/task").ok()?;
    Some(
        tasks
            .flatten()
            .filter(|task| {
                std::fs::read_to_string(task.path().join("comm"))
                    .unwrap_or_default()
                    .starts_with("rtui-canvas")
            })
            .count(),
    )
}

/// A terminal that counts, per image id, the Kitty pictures it is sent
/// from `from` on.
#[derive(Clone)]
struct Counter {
    counts: Arc<Mutex<BTreeMap<u32, usize>>>,
    from: Instant,
}
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if Instant::now() >= self.from {
            let text = String::from_utf8_lossy(bytes);
            let mut counts = self.counts.lock().unwrap();
            for command in text.split("\x1b_G").skip(1) {
                let controls = command.split(';').next().unwrap_or("");
                if !controls.starts_with("a=T") {
                    continue;
                }
                let id = controls
                    .split(',')
                    .find_map(|pair| pair.strip_prefix("i="))
                    .and_then(|value| value.parse::<u32>().ok());
                if let Some(id) = id {
                    *counts.entry(id).or_insert(0) += 1;
                }
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Fifteen canvases of 80 by 24 cells, each a new cube angle every frame,
/// for the warm-up and the run; then the root counts the drawing threads
/// and stops.
struct Grid {
    started: Instant,
    frame: usize,
    threads: Arc<Mutex<Option<Option<usize>>>>,
}
impl RootComponent for Grid {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let angle = self.frame as f32 * 0.02;
        div()
            .class("flex flex-col w-full h-full")
            .children(
                (0..3)
                    .map(|row| {
                        div()
                            .class("flex flex-row w-full h-24")
                            .children(
                                (0..5)
                                    .map(|column| {
                                        let n = (row * 5 + column) as f32;
                                        let scene = canvas_support::cube_filling(
                                            angle + n * 0.3,
                                            angle * 0.7 + n * 0.1,
                                            (640, 384),
                                        );
                                        let props = CanvasProps::new(Arc::new(scene))
                                            .options(reference_options(false));
                                        div()
                                            .class("w-80 h-24")
                                            .children(vec![Element::typed::<Canvas>(props)])
                                            .build()
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .build()
                    })
                    .collect::<Vec<_>>(),
            )
            .build()
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if self.started.elapsed() < WARM + RUN {
            self.frame += 1;
            return Ok(RootUpdate::Redraw);
        }
        // Counted while the canvases are still shown, so their threads are
        // alive.
        *self.threads.lock().unwrap() = Some(drawing_threads());
        Ok(RootUpdate::Exit)
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
#[ignore = "GFX-011's run is a release-build measurement; canvas-hosts runs it on the three hosts and binds the bound on the Linux host"]
fn gfx_011_fifteen_canvases_on_one_thread_each_get_thirty_pictures_a_second() {
    let images = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: cfg!(unix),
        cell_pixels: (8, 16),
        ..Default::default()
    };
    let started = Instant::now();
    let counter = Counter {
        counts: Arc::default(),
        from: started + WARM,
    };
    let threads = Arc::new(Mutex::new(None));
    let backend = SuprTuiBackend::with_writer_and_images(400, 72, counter.clone(), images)
        .expect("a backend that writes to memory");
    App::builder()
        .backend(backend)
        .root(Grid {
            started,
            frame: 0,
            threads: Arc::clone(&threads),
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let counts = counter.counts.lock().unwrap().clone();
    let threads = threads.lock().unwrap().flatten();
    let seconds = RUN.as_secs_f64();
    let rates: Vec<f64> = counts.values().map(|&n| n as f64 / seconds).collect();
    let fewest = rates.iter().copied().fold(f64::INFINITY, f64::min);
    let most = rates.iter().copied().fold(0.0, f64::max);
    println!(
        "GFX-011 fifteen canvases of 80 by 24 cells with a new scene every frame for {seconds:.0} s: {} canvases received pictures, the fewest {fewest:.1} a second and the most {most:.1}; drawing threads: {}",
        counts.len(),
        threads.map_or_else(|| "not counted".to_owned(), |threads| threads.to_string())
    );
    if cfg!(target_os = "linux") {
        assert!(
            counts.len() == CANVASES && fewest >= EACH && threads == Some(1),
            "GFX-011: of {CANVASES} canvases {} received pictures, the fewest {fewest:.1} a second against {EACH}, drawn on {} threads instead of one",
            counts.len(),
            threads.map_or_else(|| "an uncounted number of".to_owned(), |threads| threads.to_string())
        );
    }
}
