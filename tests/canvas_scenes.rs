//! canvas-scenes mechanism: GFX-001, GFX-003, GFX-007 and GFX-008
//! (docs/spec/canvas.md). The reference scenes, drawn on the software
//! renderer, must be the same picture as their checked-in images; theme
//! tokens resolve through the active theme; Linux text takes fontconfig's
//! first loadable monospace font; a CellGrid is one instanced draw; the
//! worker is named, never blocks the caller and keeps only the newest scene;
//! faults fall back to the software renderer or show a message; and the
//! demos draw through the canvas.

mod canvas_support;
mod common;

use canvas_support::{check_reference, reference_options, references, SIZE};
use common::app_input;
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::graphics::{
    fonts::{self, FontSource},
    Canvas, CanvasProps, Color, GraphicsFault, GraphicsMode, GraphicsOptions, GraphicsWorker,
    HybridRenderer, Paint, Path, Scene,
};
use reactive_tui::theme::Theme;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A scene that fills the picture with one color.
fn solid(color: Color) -> Scene {
    let mut scene = Scene::new();
    scene.fill(&Path::rect(0.0, 0.0, 4096.0, 4096.0), &Paint::solid(color));
    scene
}

#[test]
fn gfx_001_reference_scenes_are_their_checked_in_images() {
    let mut renderer = HybridRenderer::new(reference_options(true));
    let mut failures = Vec::new();
    for reference in references() {
        let frame = renderer
            .render(&reference.scene, reference.size.0, reference.size.1)
            .unwrap_or_else(|error| panic!("GFX-001: {} did not render: {error}", reference.name));
        if let Some(why) = check_reference(reference.name, &frame) {
            failures.push(format!("{}: {why}", reference.name));
        }
    }
    assert!(
        failures.is_empty(),
        "GFX-001: on the software renderer these reference scenes are not their images: {failures:?}"
    );
}

#[test]
fn gfx_001_a_theme_token_paints_the_active_theme_color() {
    let expected = Theme::active()
        .resolve_color("blue-500")
        .expect("blue-500 resolves");
    let expected = [expected.0, expected.1, expected.2].map(|c| (c * 255.0).round() as i32);
    let mut renderer = HybridRenderer::new(reference_options(true));
    let frame = renderer
        .render(&solid(Color::token("blue-500")), 16, 16)
        .expect("a 16 by 16 picture");
    let pixel = frame.pixels()[8 * 16 + 8];
    let got = [pixel[0] as i32, pixel[1] as i32, pixel[2] as i32];
    assert!(
        got.iter().zip(expected).all(|(g, e)| (g - e).abs() <= 1),
        "GFX-001: blue-500 painted {got:?}, the active theme resolves it to {expected:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn gfx_001_linux_text_takes_fontconfigs_first_loadable_monospace() {
    let listed = std::process::Command::new("fc-match")
        .args(["-s", "-f", "%{file}\\n", "monospace"])
        .output()
        .expect("fc-match on the Linux host");
    let candidates: Vec<std::path::PathBuf> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(Into::into)
        .collect();
    let first = candidates.iter().find(|path| fonts::loads(path)).cloned();
    let chosen = fonts::choose(None, &candidates);
    let expected = first.map_or(FontSource::Bundled, FontSource::System);
    assert!(
        chosen == expected && fonts::choose(None, &[]) == FontSource::Bundled,
        "GFX-001: fontconfig lists {:?}; the canvas chose {chosen:?}, not {expected:?}, or with no candidate it chose {:?}",
        &candidates[..candidates.len().min(4)],
        fonts::choose(None, &[])
    );
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_001_a_cell_grid_is_one_instanced_draw() {
    let mut grid = reactive_tui::layout::CellGrid::new(40, 12);
    for y in 0..12 {
        for x in 0..40 {
            grid.set(x, y, if (x + y) % 2 == 0 { "▀" } else { "a" }, None);
        }
    }
    let mut scene = Scene::new();
    scene.cells((0.0, 0.0), Arc::new(grid), (8, 16));
    let mut renderer = HybridRenderer::new(reference_options(false));
    let frame = renderer
        .render(&scene, SIZE.0, SIZE.1)
        .expect("a scene of one cell grid renders");
    assert!(
        matches!(frame.mode(), GraphicsMode::Gpu(_)) && frame.draw_calls() == 1,
        "GFX-001: a scene of one 40 by 12 cell grid drew with {:?} in {} draw calls",
        frame.mode(),
        frame.draw_calls()
    );
}

/// Waits up to 30 s for the worker's next picture.
fn next_frame(worker: &GraphicsWorker) -> Arc<reactive_tui::graphics::GraphicsFrame> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(frame) = worker.take_latest() {
            return frame;
        }
        assert!(
            Instant::now() < deadline,
            "GFX-003: the worker drew no picture in 30 s"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn gfx_003_the_worker_is_named_and_the_caller_never_waits() {
    let worker = GraphicsWorker::spawn(reference_options(true)).expect("a worker");
    // A large scene on the software renderer takes far longer than a submit may.
    let heavy = Arc::new(canvas_support::cube(0.3, 0.2));
    let started = Instant::now();
    for _ in 0..10 {
        worker.submit(heavy.clone(), (4096, 4096));
    }
    let submitting = started.elapsed();
    let frame = next_frame(&worker);
    let caller = std::thread::current().name().unwrap_or("").to_owned();
    assert!(
        frame.thread().starts_with("rtui-canvas-")
            && frame.thread() != caller
            && submitting < Duration::from_millis(50),
        "GFX-003: the picture was drawn on {:?} (the caller is {caller:?}), and ten submits took {submitting:?}",
        frame.thread()
    );
}

#[test]
fn gfx_003_a_busy_worker_keeps_only_the_newest_scene() {
    let worker = GraphicsWorker::spawn(reference_options(true)).expect("a worker");
    worker.submit(Arc::new(canvas_support::cube(0.1, 0.1)), (4096, 4096));
    for shade in 0..20u8 {
        worker.submit(Arc::new(solid(Color::rgba(shade, 0, 0, 255))), (64, 64));
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    let last = loop {
        let frame = next_frame(&worker);
        if frame.width() == 64 {
            break frame;
        }
        assert!(
            Instant::now() < deadline,
            "GFX-003: the newest scene never rendered"
        );
    };
    let stats = worker.stats();
    assert!(
        stats.waiting_max <= 1 && last.pixels()[0][0] == 19,
        "GFX-003: {} scenes waited at once, and the last picture drawn was of shade {}, not the newest (19)",
        stats.waiting_max,
        last.pixels()[0][0]
    );
}

#[test]
fn gfx_003_the_worker_keeps_no_interval_of_its_own() {
    let worker = GraphicsWorker::spawn(reference_options(true)).expect("a worker");
    let scene = Arc::new(canvas_support::shapes());
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(1) {
        worker.submit(scene.clone(), SIZE);
        std::thread::sleep(Duration::from_millis(16));
        let _ = worker.take_latest();
    }
    let rendered = worker.stats().rendered;
    assert!(
        rendered >= 50,
        "GFX-003: scenes submitted every 16 ms for one second at 40 by 12 cells rendered {rendered} frames"
    );
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_007_faults_switch_to_the_software_renderer_for_good() {
    let scene = canvas_support::shapes();
    let mut problems = Vec::new();
    for fault in [
        GraphicsFault::Adapter,
        GraphicsFault::DeviceLoss,
        GraphicsFault::Readback,
    ] {
        let mut renderer = HybridRenderer::new(GraphicsOptions {
            fault: Some(fault),
            ..reference_options(false)
        });
        for attempt in 0..3 {
            match renderer.render(&scene, SIZE.0, SIZE.1) {
                Ok(frame) => match frame.mode() {
                    GraphicsMode::CpuFallback(reason) if reason.contains(fault.label()) => {}
                    other => problems.push(format!("{fault:?} render {attempt}: {other:?}")),
                },
                Err(error) => problems.push(format!("{fault:?} render {attempt} failed: {error}")),
            }
        }
    }
    assert!(
        problems.is_empty(),
        "GFX-007: after an injected fault the canvas did not stay on the software renderer naming the fault: {problems:?}"
    );
}

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

#[test]
fn gfx_007_a_failing_software_renderer_shows_a_message_and_the_app_goes_on() {
    let props = CanvasProps::new(Arc::new(canvas_support::shapes())).options(GraphicsOptions {
        force_cpu: true,
        fault: Some(GraphicsFault::Software),
        ..reference_options(true)
    });
    let frames = app_input::run_when(
        Root(Element::typed::<Canvas>(props)),
        (40, 12),
        vec![("Canvas", None), ("Canvas", None)],
    );
    let last = frames.last().expect("frames");
    assert!(
        frames.len() >= 2 && last.text.contains("software"),
        "GFX-007: with both renderers failing the canvas area read {:?}",
        last.text
    );
}

/// The demo sources, which must draw through the canvas (GFX-008).
fn source(path: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn gfx_008_the_demos_and_src_graphics_keep_only_the_canvas() {
    let mut problems = Vec::new();
    for page in [
        "examples/widget_catalog/catalog.rs",
        "examples/animation_showcase/showcase.rs",
    ] {
        let text = source(page);
        if !text.contains("Canvas")
            || text.contains("CubeRenderer")
            || text.contains("GraphicsCanvas")
        {
            problems.push(format!(
                "{page} does not draw only through the Canvas widget"
            ));
        }
    }
    let graphics = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/graphics");
    let mut stack = vec![graphics];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            if name == "cube.wgsl" || name == "torus.wgsl" || text.contains("CubeRenderer") {
                problems.push(format!("{} holds another renderer", path.display()));
            }
        }
    }
    assert!(problems.is_empty(), "GFX-008: {problems:?}");
}
