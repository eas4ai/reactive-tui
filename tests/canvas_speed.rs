//! canvas-hosts mechanism, GFX-004 (docs/spec/canvas.md): the speed floor
//! on the Windows test tablet's Intel Iris Xe. The mechanism runs this in a
//! release build on the tablet with `--ignored`; elsewhere it does not run,
//! because the bound belongs to that hardware. The scene is fitted to the
//! picture and covers all of it, so every pixel and every cell is drawn.

mod canvas_support;

use canvas_support::reference_options;
use reactive_tui::graphics::{GraphicsMode, HybridRenderer};
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
