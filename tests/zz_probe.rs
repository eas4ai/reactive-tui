//! A scratch probe, never committed: where a picture's time goes on this
//! host, by size and by scene.
mod canvas_support;

use canvas_support::reference_options;
use reactive_tui::graphics::{Color, HybridRenderer, Paint, Path, Scene};
use std::time::{Duration, Instant};

/// Ask Windows not to slow this thread or this process down to save power,
/// and say which processor the thread is on.
#[cfg(windows)]
fn full_speed(ask: bool) -> u32 {
    #[repr(C)]
    struct Throttling {
        version: u32,
        control_mask: u32,
        state_mask: u32,
    }
    extern "system" {
        fn GetCurrentThread() -> isize;
        fn GetCurrentProcessorNumber() -> u32;
        fn SetThreadInformation(
            thread: isize,
            class: i32,
            info: *const Throttling,
            size: u32,
        ) -> i32;
        fn SetProcessInformation(
            process: isize,
            class: i32,
            info: *const Throttling,
            size: u32,
        ) -> i32;
    }
    // Execution speed is controlled (mask 1) and not throttled (state 0).
    let state = Throttling {
        version: 1,
        control_mask: 1,
        state_mask: 0,
    };
    unsafe {
        if ask {
            let thread = SetThreadInformation(GetCurrentThread(), 3, &state, 12);
            println!("PROBE asked for full speed: thread {thread} only");
        }
        GetCurrentProcessorNumber()
    }
}
#[cfg(not(windows))]
fn full_speed(_: bool) -> u32 {
    0
}

fn median(mut samples: Vec<Duration>) -> f64 {
    samples.sort();
    samples[samples.len() / 2].as_secs_f64() * 1e3
}

#[test]
#[ignore = "a probe"]
fn probe_gpu_parts() {
    full_speed(false);
    let mut renderer = HybridRenderer::new(reference_options(false));
    let empty = |_: f32, _: (u32, u32)| Scene::new();
    let cube =
        |angle: f32, size: (u32, u32)| canvas_support::cube_filling(angle, angle * 0.7, size);
    let small = |angle: f32, _: (u32, u32)| canvas_support::cube(angle, angle * 0.7);
    // The background and `count` turned squares of `side` pixels.
    fn squares(count: usize, side: f32, angle: f32, size: (u32, u32)) -> Scene {
        let mut scene = Scene::new();
        scene.fill(
            &Path::rect(0.0, 0.0, size.0 as f32, size.1 as f32),
            &Paint::solid(Color::rgba(40, 44, 70, 255)),
        );
        for n in 0..count {
            let (cx, cy) = (400.0 + 220.0 * n as f32, 480.0);
            let (sin, cos) = (angle + n as f32).sin_cos();
            let half = side / 2.0;
            let corner = |x: f32, y: f32| (cx + x * cos - y * sin, cy + x * sin + y * cos);
            let mut path = reactive_tui::graphics::PathBuilder::new();
            for (i, (x, y)) in [(-half, -half), (half, -half), (half, half), (-half, half)]
                .into_iter()
                .enumerate()
            {
                let (x, y) = corner(x, y);
                path = if i == 0 {
                    path.move_to(x, y)
                } else {
                    path.line_to(x, y)
                };
            }
            scene.fill(
                &path.close().build(),
                &Paint::solid(Color::rgba(90, 120 + 20 * n as u8, 255, 255)),
            );
        }
        scene
    }
    let one = |angle: f32, size: (u32, u32)| squares(1, 600.0, angle, size);
    let three = |angle: f32, size: (u32, u32)| squares(3, 600.0, angle, size);
    let six_small = |angle: f32, size: (u32, u32)| squares(6, 200.0, angle, size);
    let scenes: [(&str, &dyn Fn(f32, (u32, u32)) -> Scene); 9] = [
        ("6 squares of 200", &six_small),
        ("cube not fitted", &small),
        ("1 square of 600", &one),
        ("empty", &empty),
        ("3 squares of 600", &three),
        ("cube not fitted", &small),
        ("empty", &empty),
        ("1 square of 600", &one),
        ("cube fitted", &cube),
    ];
    for size in [(1920u32, 960u32)] {
        for (name, scene) in &scenes {
            let mut total = Vec::new();
            let mut parts: [Vec<Duration>; 4] = Default::default();
            let mut calls = 0;
            let plain = vec![255u8; size.0 as usize * size.1 as usize * 4];
            let mut copies = Vec::new();
            let mut processors = std::collections::BTreeSet::new();
            for frame in 0..80 {
                processors.insert(full_speed(false));
                let scene = scene(frame as f32 * 0.02, size);
                let started = Instant::now();
                let picture = renderer.render(&scene, size.0, size.1).expect("a picture");
                let took = started.elapsed();
                // The same number of bytes copied from ordinary memory,
                // right after the picture: how fast the processor is then.
                let started = Instant::now();
                let mut bytes = Vec::with_capacity(plain.len());
                bytes.extend_from_slice(&plain);
                std::hint::black_box(&bytes);
                copies.push(started.elapsed());
                if frame >= 20 {
                    total.push(took);
                    let t = picture.timings();
                    for (part, time) in parts
                        .iter_mut()
                        .zip([t.prepare, t.render, t.wait, t.readback])
                    {
                        part.push(time);
                    }
                }
                calls = picture.draw_calls();
            }
            let [prepare, render, wait, readback] = parts.map(median);
            println!(
                "PROBE {}x{} {name}: total {:.2} ms = prepare {prepare:.2} + hand over {render:.2} + wait {wait:.2} + copy out {readback:.2}; {calls} draw calls; a plain copy of as many bytes {:.2} ms; on processors {processors:?}",
                size.0,
                size.1,
                median(total),
                median(copies),
            );
        }
    }
}

#[test]
#[ignore = "a probe"]
fn probe_block_glyphs() {
    let mut renderer = HybridRenderer::new(reference_options(false));
    println!("PROBE {}", renderer.mode().label());
    for (name, fitted) in [("cube not fitted", false), ("cube fitted", true)] {
        let mut total = Vec::new();
        for frame in 0..80 {
            let angle = frame as f32 * 0.02;
            let scene = if fitted {
                canvas_support::cube_filling(angle, angle * 0.7, (1920, 960))
            } else {
                canvas_support::cube(angle, angle * 0.7)
            };
            let started = Instant::now();
            let grid = renderer.render_cells(&scene, 240, 60).expect("cells");
            let took = started.elapsed();
            assert_eq!(grid.width(), 240);
            if frame >= 20 {
                total.push(took);
            }
        }
        println!(
            "PROBE block glyphs 240x60 {name}: total {:.2} ms",
            median(total)
        );
    }
}

/// The fitted cube drawn without a pause and then at 60 pictures a second,
/// by windows of 50 pictures: does the host slow down under steady load?
#[test]
#[ignore = "a probe"]
fn probe_pacing() {
    let mut renderer = HybridRenderer::new(reference_options(false));
    for (name, frame_time) in [
        ("without a pause", Duration::ZERO),
        ("at 60 a second", Duration::from_micros(16_667)),
        ("at 30 a second", Duration::from_micros(33_333)),
    ] {
        std::thread::sleep(Duration::from_secs(20));
        let mut windows = Vec::new();
        let mut window = Vec::new();
        for frame in 0..400 {
            let angle = frame as f32 * 0.02;
            let scene = canvas_support::cube_filling(angle, angle * 0.7, (1920, 960));
            let started = Instant::now();
            let picture = renderer.render(&scene, 1920, 960).expect("a picture");
            let took = started.elapsed();
            std::hint::black_box(&picture);
            window.push(took);
            if window.len() == 50 {
                windows.push(format!("{:.1}", median(std::mem::take(&mut window))));
            }
            if let Some(rest) = frame_time.checked_sub(started.elapsed()) {
                std::thread::sleep(rest);
            }
        }
        println!(
            "PROBE cube fitted at 1920x960 {name}: median ms of each 50 pictures: {}",
            windows.join(" ")
        );
    }
}
