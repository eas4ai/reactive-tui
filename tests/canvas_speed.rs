//! canvas-hosts mechanism, GFX-004 (docs/spec/canvas.md): the speed floor
//! on the Windows test tablet's Intel Iris Xe. The mechanism runs this in a
//! release build on the tablet with `--ignored`; elsewhere it does not run,
//! because the bound belongs to that hardware.

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

#[test]
#[ignore = "GFX-004's bound is for the Windows tablet; canvas-hosts runs it there in release"]
fn gfx_004_the_tablet_draws_the_animation_scene_within_a_frame() {
    let mut renderer = HybridRenderer::new(reference_options(false));
    let mut pixels = Vec::with_capacity(FRAMES);
    let mut blocks = Vec::with_capacity(FRAMES);
    let mut mode = None;
    for frame in 0..FRAMES + 20 {
        let angle = frame as f32 * 0.02;
        let scene = canvas_support::cube(angle, angle * 0.7);
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
    assert!(
        matches!(mode, GraphicsMode::Gpu(_)) && pixels <= BOUND && blocks <= BOUND,
        "GFX-004: with {mode:?} the 95th percentiles of {FRAMES} frames were {pixels:?} for 1920 by 960 pixels and {blocks:?} for 240 by 60 cells of block glyphs"
    );
}
