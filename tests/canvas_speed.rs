//! canvas-hosts mechanism, GFX-004 (docs/spec/canvas.md): the speed floor
//! on the Windows test tablet's Intel Iris Xe. The mechanism runs this in a
//! release build on the tablet with `--ignored`; elsewhere it does not run,
//! because the bound belongs to that hardware. The scene is fitted to the
//! picture and covers all of it, so every pixel and every cell is drawn.
//!
//! The second test measures what no requirement bounds: how long the App's
//! thread waits in `present` while a picture of 240 by 60 cells is made
//! ready for a terminal that takes pixels. It prints the times and fails
//! only when no pictures were sent.

mod canvas_support;

use canvas_support::reference_options;
use reactive_tui::app::{App, AppWaker, RootComponent, RootUpdate};
use reactive_tui::backend::{Backend, FrameLayout, ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::event::types::Event;
use reactive_tui::graphics::{Canvas, CanvasProps, GraphicsMode, HybridRenderer};
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// A terminal that counts the bytes and the pictures it is sent and keeps
/// nothing.
#[derive(Clone, Default)]
struct Counting {
    bytes: Arc<AtomicUsize>,
    pictures: Arc<AtomicUsize>,
}
impl Write for Counting {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A Kitty picture begins with its action, a Sixel picture with the
        // device control string the crate writes.
        let begins = |at: usize| {
            [&b"\x1b_Ga=T"[..], &b"\x1bP0;1q"[..]]
                .iter()
                .any(|mark| bytes[at..].starts_with(mark))
        };
        let pictures = bytes
            .iter()
            .enumerate()
            .filter(|(at, byte)| **byte == 0x1b && begins(*at))
            .count();
        self.pictures.fetch_add(pictures, Ordering::SeqCst);
        self.bytes.fetch_add(bytes.len(), Ordering::SeqCst);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// One present: how long the App's thread waited, and the pictures and
/// bytes the terminal was sent for it.
struct Present {
    waited: Duration,
    pictures: usize,
    bytes: usize,
}

/// The default backend with each present timed.
struct Timed {
    inner: SuprTuiBackend,
    terminal: Counting,
    presents: Arc<Mutex<Vec<Present>>>,
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
        let before = (
            self.terminal.pictures.load(Ordering::SeqCst),
            self.terminal.bytes.load(Ordering::SeqCst),
        );
        let started = Instant::now();
        self.inner.present()?;
        let waited = started.elapsed();
        // The bytes are written after present returns; they are counted,
        // not timed.
        self.inner.sync()?;
        self.presents.lock().unwrap().push(Present {
            waited,
            pictures: self.terminal.pictures.load(Ordering::SeqCst) - before.0,
            bytes: self.terminal.bytes.load(Ordering::SeqCst) - before.1,
        });
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

/// A root that shows the cube at a new angle with every frame, over all of
/// its area, and stops after `frames`.
struct Turning {
    frame: usize,
    frames: usize,
}
impl RootComponent for Turning {
    fn render(&self) -> Element {
        let angle = self.frame as f32 * 0.02;
        let scene = canvas_support::cube_in_view(angle, angle * 0.7);
        let props = CanvasProps::new(Arc::new(scene))
            .view(canvas_support::SIZE.0 as f32, canvas_support::SIZE.1 as f32)
            .options(reference_options(false));
        Element::typed::<Canvas>(props)
    }
    fn update(&mut self) -> Result<RootUpdate> {
        self.frame += 1;
        Ok(if self.frame >= self.frames {
            RootUpdate::Exit
        } else {
            RootUpdate::Redraw
        })
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// The presents of 400 frames of the turning cube at 240 by 60 cells that
/// sent a picture, without the first 20 of them.
fn presents_with_a_picture(images: ImageOutputOptions) -> Vec<Present> {
    let terminal = Counting::default();
    let presents = Arc::new(Mutex::new(Vec::new()));
    let inner = SuprTuiBackend::with_writer_and_images(240, 60, terminal.clone(), images)
        .expect("a backend that writes to memory");
    App::builder()
        .backend(Timed {
            inner,
            terminal,
            presents: presents.clone(),
        })
        .root(Turning {
            frame: 0,
            frames: 400,
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let mut presents = std::mem::take(&mut *presents.lock().unwrap());
    presents.retain(|present| present.pictures > 0);
    presents.drain(..presents.len().min(20));
    presents
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
#[ignore = "a measurement for the Windows tablet; canvas-hosts runs it there in release"]
fn gfx_003_the_apps_wait_for_a_picture_of_240_by_60_cells_is_measured() {
    let kitty = ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    };
    let mut outputs = vec![("Kitty, in the command", kitty)];
    // The crate's shared memory is POSIX shared memory.
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
    let mut sent = Vec::new();
    for (name, images) in outputs {
        let presents = presents_with_a_picture(images);
        let bytes =
            presents.iter().map(|present| present.bytes).sum::<usize>() / presents.len().max(1);
        let waits: Vec<Duration> = presents.iter().map(|present| present.waited).collect();
        if !waits.is_empty() {
            println!(
                "GFX-003 present with a picture of 1920 by 960 pixels as {name}: median {:.2} ms, \
                 p95 {:.2} ms over {} pictures of {bytes} bytes",
                median(waits.clone()),
                p95(waits).as_secs_f64() * 1e3,
                presents.len()
            );
        }
        sent.push((name, presents.len()));
    }
    assert!(
        sent.iter().all(|(_, pictures)| *pictures >= 30),
        "GFX-003: too few pictures were sent to measure the App's wait: {sent:?}"
    );
}
