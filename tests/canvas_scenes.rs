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
use reactive_tui::app::{App, RootComponent, RootUpdate};
use reactive_tui::backend::{ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::event::router::EventResult;
use reactive_tui::event::types::{Event, KeyCode, KeyEvent};
use reactive_tui::graphics::{
    Canvas, CanvasProps, Color, GraphicsFault, GraphicsMode, GraphicsOptions, GraphicsWorker,
    HybridRenderer, Paint, Path, PathBuilder, Scene, Stroke,
};
use reactive_tui::theme::{Theme, ThemeVariables};
use std::sync::{Arc, Mutex};
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

/// A white line 4 pixels wide across a picture of 320 by 16 pixels.
fn line(dash: &[f32]) -> Scene {
    let path = PathBuilder::new()
        .move_to(10.0, 8.0)
        .line_to(310.0, 8.0)
        .build();
    let mut scene = Scene::new();
    scene.stroke(
        &path,
        &Stroke::new(4.0).dash(dash),
        &Paint::solid(Color::rgba(255, 255, 255, 255)),
    );
    scene
}

#[test]
fn gfx_001_a_dash_pattern_near_zero_draws_a_solid_line() {
    let mut renderer = HybridRenderer::new(reference_options(true));
    let mut draw = |dash: &[f32]| {
        renderer
            .render(&line(dash), 320, 16)
            .expect("a 320 by 16 picture")
            .pixels()
            .to_vec()
    };
    let solid = draw(&[]);
    // f32 cannot take a dash of 0.00001 from a line of 300 pixels.
    let near_zero = draw(&[0.000_01, 0.000_01]);
    let dashed = draw(&[8.0, 8.0]);
    let middle = solid[8 * 320 + 160];
    assert!(
        near_zero == solid && dashed != solid && middle == [255; 4],
        "GFX-001: a dash pattern of 0.00001 and 0.00001 drew {} pixels unlike the solid line's, \
         a pattern of 8 and 8 drew {}, and the solid line's middle is {middle:?}",
        unlike(&near_zero, &solid),
        unlike(&dashed, &solid)
    );
}

/// How many pixels of `a` differ from `b`'s.
fn unlike(a: &[[u8; 4]], b: &[[u8; 4]]) -> usize {
    a.iter().zip(b).filter(|(a, b)| a != b).count()
}

/// `base` with another `primary`. Only `primary` differs, so a test that
/// runs meanwhile and reads another color of the theme sees no change.
fn with_primary(base: &Theme, hex: &str) -> Theme {
    Theme::new("canvas-test")
        .with_variables(ThemeVariables::new().set("--color-primary", hex))
        .extend(base.clone())
}

/// The two colors the theme tests give `primary`, one after the other.
const PRIMARIES: [(&str, [i32; 3]); 2] = [("#c81e28", [200, 30, 40]), ("#1e3cc8", [30, 60, 200])];

fn near(got: [i32; 3], expected: [i32; 3]) -> bool {
    got.iter().zip(expected).all(|(g, e)| (g - e).abs() <= 1)
}

#[test]
// Tests that change the active theme take turns.
#[serial_test::serial(theme)]
fn gfx_001_a_theme_token_paints_the_active_theme_color() {
    let before = Theme::active();
    let mut renderer = HybridRenderer::new(reference_options(true));
    let mut painted = Vec::new();
    for (hex, _) in PRIMARIES {
        Theme::set_active(with_primary(&before, hex));
        let frame = renderer
            .render(&solid(Color::token("primary")), 16, 16)
            .expect("a 16 by 16 picture");
        let pixel = frame.pixels()[8 * 16 + 8];
        painted.push([pixel[0] as i32, pixel[1] as i32, pixel[2] as i32]);
    }
    Theme::set_active((*before).clone());
    assert!(
        near(painted[0], PRIMARIES[0].1) && near(painted[1], PRIMARIES[1].1),
        "GFX-001: `primary` painted {painted:?} under themes that give it {:?} and {:?}",
        PRIMARIES[0].1,
        PRIMARIES[1].1
    );
}

/// A root that shows one canvas of a scene that never changes, and gives
/// the theme another `primary` when a key arrives.
struct Themed {
    scene: Arc<Scene>,
    base: Arc<Theme>,
}
impl RootComponent for Themed {
    fn render(&self) -> Element {
        Element::typed::<Canvas>(
            CanvasProps::new(self.scene.clone()).options(reference_options(true)),
        )
    }
    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
        if matches!(event, Event::Key(_)) {
            Theme::set_active(with_primary(&self.base, PRIMARIES[1].0));
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        Ok(RootUpdate::Unchanged)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// The colors of the cell in the middle of `frame`: its glyph's and its
/// background's.
fn middle_colors(frame: &app_input::Snapshot) -> Vec<[i32; 3]> {
    let (rows, columns) = frame.screen.size();
    let cell = frame.screen.cell(rows / 2, columns / 2).expect("a cell");
    [cell.fgcolor(), cell.bgcolor()]
        .into_iter()
        .filter_map(|color| match color {
            vt100::Color::Rgb(r, g, b) => Some([r as i32, g as i32, b as i32]),
            _ => None,
        })
        .collect()
}

#[test]
// Tests that change the active theme take turns.
#[serial_test::serial(theme)]
fn gfx_001_a_canvas_follows_a_change_of_theme() {
    let before = Theme::active();
    Theme::set_active(with_primary(&before, PRIMARIES[0].0));
    let frames = app_input::run(
        Themed {
            scene: Arc::new(solid(Color::token("primary"))),
            base: before.clone(),
        },
        (40, 12),
        // Frames that are not busy: the first, before the canvas knows its
        // size; the first that shows the picture, after which the key
        // changes the theme; and the first that shows the picture in the
        // new theme's color.
        vec![
            (2, Some(Event::Key(KeyEvent::new(KeyCode::Char('t'))))),
            (3, None),
        ],
    );
    Theme::set_active((*before).clone());
    let settled: Vec<&app_input::Snapshot> = frames.iter().filter(|f| !f.busy).collect();
    let first = settled.get(1).map(|frame| middle_colors(frame));
    let last = settled.last().map(|frame| middle_colors(frame));
    assert!(
        first
            .as_ref()
            .is_some_and(|colors| colors.iter().any(|c| near(*c, PRIMARIES[0].1)))
            && last
                .as_ref()
                .is_some_and(|colors| colors.iter().any(|c| near(*c, PRIMARIES[1].1))),
        "GFX-001: a canvas of `primary` showed {first:?} under the first theme ({:?}) and {last:?} after the theme changed ({:?})",
        PRIMARIES[0].1,
        PRIMARIES[1].1
    );
}

#[cfg(target_os = "linux")]
#[test]
fn gfx_001_linux_text_takes_fontconfigs_first_loadable_monospace() {
    use reactive_tui::graphics::fonts::{self, FontSource};
    // Each line is the number of the face in the file, then the file.
    let listed = std::process::Command::new("fc-match")
        .args(["-s", "-f", "%{index} %{file}\\n", "monospace"])
        .output()
        .expect("fc-match on the Linux host");
    let candidates: Vec<(std::path::PathBuf, u32)> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter_map(|line| {
            let (index, file) = line.split_once(' ')?;
            Some((file.into(), index.parse::<u32>().ok()? & 0xFFFF))
        })
        .collect();
    let first = candidates
        .iter()
        .find(|(file, face)| fonts::loads_face(file, *face))
        .cloned();
    let chosen = fonts::choose(None, &candidates);
    let expected = first.map_or(FontSource::Bundled, |(file, face)| {
        FontSource::System(file, face)
    });
    assert!(
        chosen == expected && fonts::choose(None, &[]) == FontSource::Bundled,
        "GFX-001: fontconfig lists {:?}; the canvas chose {chosen:?}, not {expected:?}, or with no candidate it chose {:?}",
        &candidates[..candidates.len().min(4)],
        fonts::choose(None, &[])
    );
    // The canvas's own way to its font: a renderer of an application that
    // names none. The software renderer takes its font as the hardware one does.
    let with_font = |font| GraphicsOptions {
        force_cpu: true,
        font,
        ..Default::default()
    };
    let mut scene = Scene::new();
    let white = Paint::solid(Color::rgba(255, 255, 255, 255));
    scene.text((4.0, 4.0), 24.0, "Canvas 0O1l", &white);
    let mut own = HybridRenderer::new(with_font(FontSource::Auto));
    let mut named = HybridRenderer::new(with_font(expected.clone()));
    let drawn = own.render(&scene, 320, 48).expect("a line of text");
    let wanted = named.render(&scene, 320, 48).expect("a line of text");
    let inked = drawn.pixels().iter().filter(|pixel| pixel[3] > 0).count();
    assert!(
        *own.font() == expected && drawn.pixels() == wanted.pixels() && inked > 0,
        "GFX-001: fontconfig's first loadable font is {expected:?}; a canvas with no application \
         font draws in {:?}, {} of its pixels differ from the text drawn in that font, and {inked} \
         pixels hold ink",
        own.font(),
        unlike(drawn.pixels(), wanted.pixels())
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
    // The large scene keeps the worker busy while the 20 small ones
    // arrive. Had they waited in a queue, the worker would have drawn all
    // 21; it draws the large one, or the scene that replaced it, and then
    // the newest. One more is allowed for a host so loaded that the large
    // scene is finished before the last small one arrives.
    assert!(
        stats.rendered <= 3 && stats.replaced >= 18 && last.pixels()[0][0] == 19,
        "GFX-003: of 21 scenes submitted to a busy worker {} were drawn and {} were replaced \
         before they were drawn, and the last picture drawn was of shade {}, not the newest (19)",
        stats.rendered,
        stats.replaced,
        last.pixels()[0][0]
    );
}

#[test]
fn gfx_003_the_worker_keeps_no_interval_of_its_own() {
    let worker = GraphicsWorker::spawn(reference_options(true)).expect("a worker");
    let scene = Arc::new(canvas_support::shapes());
    let started = Instant::now();
    // Scenes are submitted by the clock, 16 ms apart, so that a sleep that
    // takes longer than asked, as on macOS, costs no scene.
    let mut submitted = 0;
    while started.elapsed() < Duration::from_secs(1) {
        worker.submit(scene.clone(), SIZE);
        submitted += 1;
        let next = started + Duration::from_millis(16) * submitted;
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
        let _ = worker.take_latest();
    }
    let rendered = worker.stats().rendered;
    println!("GFX-003 frames: {rendered} rendered of {submitted} submitted in one second");
    assert!(
        rendered >= 50,
        "GFX-003: of {submitted} scenes submitted every 16 ms for one second at 40 by 12 cells {rendered} were rendered"
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

/// GFX-007: a frame whose glyphs do not all fit the largest atlas is drawn
/// whole in software for that frame and says so, and the adapter draws the
/// next frame that fits; a cell outside the picture takes no room in the
/// atlas, so a grid with a row below the picture draws every visible glyph
/// on the adapter, the same picture as the software renderer's.
#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_007_a_full_glyph_atlas_draws_the_frame_in_software_and_offscreen_cells_take_no_room() {
    use reactive_tui::layout::CellGrid;
    // `columns` by `rows` distinct braille glyphs from `first`.
    let braille = |columns: u16, rows: u16, first: u32| {
        let mut grid = CellGrid::new(columns, rows);
        for y in 0..rows {
            for x in 0..columns {
                let code = first + u32::from(y) * u32::from(columns) + u32::from(x);
                let glyph = char::from_u32(code).unwrap().to_string();
                grid.set(x, y, &glyph, Some((1.0, 1.0, 1.0, 1.0)));
            }
        }
        Arc::new(grid)
    };
    let cells_scene = |grids: &[Arc<CellGrid>]| {
        let mut scene = Scene::new();
        for grid in grids {
            scene.cells((0.0, 0.0), Arc::clone(grid), (512, 512));
        }
        scene
    };
    // The visible 8 by 8 cells of 512 pixels that hold no ink.
    let empty_cells = |frame: &reactive_tui::graphics::GraphicsFrame| -> Vec<(usize, usize)> {
        let pixels = frame.pixels();
        (0..8)
            .flat_map(|cy| (0..8).map(move |cx| (cx, cy)))
            .filter(|(cx, cy)| {
                !(cy * 512..(cy + 1) * 512)
                    .any(|y| (cx * 512..(cx + 1) * 512).any(|x| pixels[y * 4096 + x][3] > 0))
            })
            .collect()
    };
    let mut gpu = HybridRenderer::new(reference_options(false));
    let mut cpu = HybridRenderer::new(reference_options(true));
    assert!(
        matches!(gpu.mode(), GraphicsMode::Gpu(_)),
        "GFX-007: this host's adapter draws, found {:?}",
        gpu.mode()
    );

    // A ninth row below the picture: its glyphs take no atlas room, so the
    // 64 visible ones fit and every visible cell is drawn on the adapter.
    let below = cells_scene(&[braille(8, 9, 0x2801)]);
    let on_gpu = gpu.render(&below, 4096, 4096).unwrap();
    let on_cpu = cpu.render(&below, 4096, 4096).unwrap();
    assert!(
        matches!(on_gpu.mode(), GraphicsMode::Gpu(_)),
        "GFX-007: a grid whose ninth row lies below the picture is drawn on the adapter, found {:?}",
        on_gpu.mode()
    );
    assert_eq!(
        empty_cells(&on_gpu),
        Vec::<(usize, usize)>::new(),
        "GFX-007: every visible cell of the 8 by 9 grid holds its glyph on the adapter"
    );
    assert!(
        on_gpu.pixels() == on_cpu.pixels(),
        "GFX-007: the adapter's picture of the 8 by 9 grid equals the software renderer's ({} pixels differ)",
        on_gpu
            .pixels()
            .iter()
            .zip(on_cpu.pixels())
            .filter(|(a, b)| a != b)
            .count()
    );

    // Two grids of 64 distinct glyphs each over the same cells: 128 glyphs
    // of 512 pixels do not fit the 4096 atlas, so this frame is drawn in
    // software, whole, and the frame says why; the adapter is kept.
    let crowded = cells_scene(&[braille(8, 8, 0x2801), braille(8, 8, 0x2841)]);
    let on_gpu = gpu.render(&crowded, 4096, 4096).unwrap();
    let on_cpu = cpu.render(&crowded, 4096, 4096).unwrap();
    assert!(
        matches!(on_gpu.mode(), GraphicsMode::CpuFallback(reason) if reason.contains("atlas")),
        "GFX-007: a frame the atlas cannot hold is drawn in software and says so, found {:?}",
        on_gpu.mode()
    );
    assert!(
        on_gpu.pixels() == on_cpu.pixels(),
        "GFX-007: the frame the atlas could not hold equals the software renderer's picture"
    );
    assert!(
        matches!(gpu.mode(), GraphicsMode::Gpu(_)),
        "GFX-007: the adapter is kept after a frame its atlas could not hold, found {:?}",
        gpu.mode()
    );
    let fits = cells_scene(&[braille(8, 8, 0x2801)]);
    let next = gpu.render(&fits, 4096, 4096).unwrap();
    assert!(
        matches!(next.mode(), GraphicsMode::Gpu(_)),
        "GFX-007: the next frame that fits is drawn on the adapter again, found {:?}",
        next.mode()
    );
}

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_007_a_software_fault_fails_both_renderers() {
    let scene = canvas_support::shapes();
    let mut renderer = HybridRenderer::new(GraphicsOptions {
        fault: Some(GraphicsFault::Software),
        ..reference_options(false)
    });
    let began = renderer.mode().clone();
    let drawn: Vec<String> = (0..3)
        .map(|_| match renderer.render(&scene, SIZE.0, SIZE.1) {
            Ok(frame) => format!("a picture by {:?}", frame.mode()),
            Err(error) => format!("error: {error}"),
        })
        .collect();
    assert!(
        matches!(began, GraphicsMode::Gpu(_))
            && matches!(renderer.mode(), GraphicsMode::CpuFallback(_))
            && drawn
                .iter()
                .all(|drawn| drawn.starts_with("error") && drawn.contains("software")),
        "GFX-007: a renderer that began as {began:?} with a software fault injected drew {drawn:?} \
         and ended as {:?}",
        renderer.mode()
    );
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_007_a_failing_software_renderer_shows_a_message_and_the_app_goes_on() {
    // The hardware renderer is tried first and fails too.
    let props = CanvasProps::new(Arc::new(canvas_support::shapes())).options(GraphicsOptions {
        fault: Some(GraphicsFault::Software),
        ..reference_options(false)
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

/// One canvas of the shapes scene, on the software renderer.
fn shapes_canvas() -> Element {
    let props =
        CanvasProps::new(Arc::new(canvas_support::shapes())).options(reference_options(true));
    Element::typed::<Canvas>(props)
}

/// What a terminal of `size` cells that takes `images` is sent by an App
/// that shows `root`. Each step waits for a finished frame whose output
/// holds its text and then sends its event, if it has one.
fn sent_until(
    root: Element,
    size: (u16, u16),
    images: ImageOutputOptions,
    steps: Vec<(&str, Option<Event>)>,
) -> String {
    let frames = app_input::run_when_output(
        Root(root),
        size,
        images,
        steps
            .into_iter()
            .map(|(needle, event)| (needle.to_owned(), 1, event))
            .collect(),
    );
    frames
        .iter()
        .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
        .collect()
}

/// The width and height each Kitty picture in `output` states for itself.
fn kitty_sizes(output: &str) -> Vec<(u32, u32)> {
    output
        .split("\x1b_G")
        .skip(1)
        .filter(|command| command.contains("a=T"))
        .filter_map(|command| {
            let controls = command.split(';').next()?;
            let number = |key: &str| {
                controls
                    .split(',')
                    .find_map(|pair| pair.strip_prefix(key))
                    .and_then(|value| value.parse::<u32>().ok())
            };
            Some((number("s=")?, number("v=")?))
        })
        .collect()
}

#[test]
fn gfx_001_an_area_wider_than_4096_pixels_is_drawn_whole() {
    // 600 by 200 cells of 8 by 16 pixels are 4800 by 3200 pixels: wider
    // than the 4096 the canvas once drew at most, and within the 64 MiB a
    // frame holds for one picture (GFX-010).
    let shared = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        ..Default::default()
    };
    let sizes = kitty_sizes(&sent_until(
        shapes_canvas(),
        (600, 200),
        shared,
        vec![("a=T", None)],
    ));
    assert!(
        sizes.first() == Some(&(4800, 3200)),
        "GFX-001: an area of 600 by 200 cells of 8 by 16 pixels was sent pictures of {sizes:?}, not one of 4800 by 3200"
    );
}

/// A terminal that keeps the screen the App's output paints, so a root can
/// look at what each canvas shows.
#[derive(Clone)]
struct ScreenTerminal {
    parser: Arc<Mutex<vt100::Parser>>,
}
impl ScreenTerminal {
    fn new(columns: u16, rows: u16) -> Self {
        Self {
            parser: Arc::new(Mutex::new(vt100::Parser::new(rows, columns, 0))),
        }
    }
    /// The RGB colors of the cell at `column`, `row`: its glyph's and its
    /// background's, those that are RGB.
    fn colors(&self, column: u16, row: u16) -> Vec<[u8; 3]> {
        let parser = self.parser.lock().unwrap();
        let Some(cell) = parser.screen().cell(row, column) else {
            return Vec::new();
        };
        [cell.fgcolor(), cell.bgcolor()]
            .into_iter()
            .filter_map(|color| match color {
                vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
                _ => None,
            })
            .collect()
    }
}
impl std::io::Write for ScreenTerminal {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.parser.lock().unwrap().process(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The canvases of GFX-003's sharing tests, side by side.
const MANY: usize = 15;

/// The color canvas `n` of the fifteen draws: no two alike, and none a
/// part of another's name in the output.
fn own_rgb(n: usize) -> [u8; 3] {
    [(16 * n + 15) as u8, 90, 60]
}

/// Whether each of the fifteen canvases, 10 cells wide from the left,
/// shows its own color in its middle.
fn shows_own_color(terminal: &ScreenTerminal) -> Vec<bool> {
    (0..MANY)
        .map(|n| terminal.colors(10 * n as u16 + 5, 6).contains(&own_rgb(n)))
        .collect()
}

/// Fifteen canvases side by side, each a solid color of its own, all
/// drawing on one worker the application started. The root looks at the
/// screen each frame and stops once every canvas shows its color, or at
/// its deadline.
struct Fifteen {
    worker: Arc<GraphicsWorker>,
    terminal: ScreenTerminal,
    deadline: Instant,
}
impl RootComponent for Fifteen {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        div()
            .class("flex flex-row w-full h-full")
            .children(
                (0..MANY)
                    .map(|n| {
                        let [r, g, b] = own_rgb(n);
                        let props = CanvasProps::new(Arc::new(solid(Color::rgba(r, g, b, 255))))
                            .options(reference_options(true))
                            .worker(self.worker.clone());
                        div()
                            .class("w-10 h-full")
                            .children(vec![Element::typed::<Canvas>(props)])
                            .build()
                    })
                    .collect::<Vec<_>>(),
            )
            .build()
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        let shown = shows_own_color(&self.terminal);
        if shown.iter().all(|shown| *shown) || Instant::now() >= self.deadline {
            return Ok(RootUpdate::Exit);
        }
        Ok(RootUpdate::Redraw)
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

#[test]
fn gfx_003_canvases_sharing_a_worker_each_show_their_own_picture() {
    let worker = Arc::new(GraphicsWorker::spawn(reference_options(true)).expect("a worker"));
    let terminal = ScreenTerminal::new(150, 12);
    let backend = SuprTuiBackend::with_writer_and_images(
        150,
        12,
        terminal.clone(),
        ImageOutputOptions::default(),
    )
    .expect("a backend that writes to memory");
    App::builder()
        .backend(backend)
        .root(Fifteen {
            worker,
            terminal: terminal.clone(),
            deadline: Instant::now() + Duration::from_secs(20),
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let shown = shows_own_color(&terminal);
    let wrong: Vec<(usize, Vec<[u8; 3]>)> = (0..MANY)
        .filter(|&n| !shown[n])
        .map(|n| (n, terminal.colors(10 * n as u16 + 5, 6)))
        .collect();
    assert!(
        wrong.is_empty(),
        "GFX-003: of fifteen canvases on one worker, these did not show their own color (canvas, colors shown): {wrong:?}"
    );
}

/// How many threads of this process are named `rtui-canvas-*`, read from
/// /proc, so Linux counts.
#[cfg(target_os = "linux")]
fn drawing_threads() -> usize {
    std::fs::read_dir("/proc/self/task")
        .map(|tasks| {
            tasks
                .flatten()
                .filter(|task| {
                    std::fs::read_to_string(task.path().join("comm"))
                        .unwrap_or_default()
                        .starts_with("rtui-canvas")
                })
                .count()
        })
        .unwrap_or(0)
}

/// Fifteen canvases of the shapes scene in a grid of five by three, each
/// on the worker it starts itself; after a few frames the root counts the
/// drawing threads and stops.
#[cfg(target_os = "linux")]
struct Counting {
    frame: usize,
    threads: Arc<Mutex<Option<usize>>>,
}
#[cfg(target_os = "linux")]
impl RootComponent for Counting {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        div()
            .class("flex flex-col w-full h-full")
            .children(
                (0..3)
                    .map(|_| {
                        div()
                            .class("flex flex-row w-full h-12")
                            .children(
                                (0..5)
                                    .map(|_| {
                                        div()
                                            .class("w-30 h-12")
                                            .children(vec![shapes_canvas()])
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
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        self.frame += 1;
        if self.frame < 4 {
            return Ok(RootUpdate::Redraw);
        }
        // Counted while the canvases are shown, so their threads are alive.
        *self.threads.lock().unwrap() = Some(drawing_threads());
        Ok(RootUpdate::Exit)
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// One App of fifteen canvases, in a process of its own: the other tests of
/// this binary start workers too, which would be counted.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "run by gfx_003_fifteen_canvases_share_one_drawing_thread in a process of its own"]
fn fifteen_canvases_child() {
    let threads = Arc::new(Mutex::new(None));
    let backend = SuprTuiBackend::with_writer_and_images(
        150,
        36,
        ScreenTerminal::new(150, 36),
        ImageOutputOptions::default(),
    )
    .expect("a backend that writes to memory");
    App::builder()
        .backend(backend)
        .root(Counting {
            frame: 0,
            threads: Arc::clone(&threads),
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let threads = threads
        .lock()
        .unwrap()
        .expect("the root counted the threads");
    println!("GFX-003 drawing threads for fifteen canvases: {threads}");
    assert!(
        threads == 1,
        "GFX-003: fifteen canvases drawn with the same options started {threads} drawing threads, not one"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn gfx_003_fifteen_canvases_share_one_drawing_thread() {
    let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
        .args([
            "--exact",
            "fifteen_canvases_child",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .expect("the test binary runs");
    let said = String::from_utf8_lossy(&child.stdout).into_owned()
        + &String::from_utf8_lossy(&child.stderr);
    let from = said.find("GFX-003").unwrap_or(0);
    assert!(
        child.status.success() && said.contains("1 passed"),
        "GFX-003: {}",
        &said[from..]
    );
}

/// Two canvases side by side, each on a worker of its own, that trade
/// places when a key arrives: both pictures are then placed anew in one
/// frame.
struct Swapping {
    halves: [Element; 2],
    swapped: bool,
}
impl RootComponent for Swapping {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let [left, right] = &self.halves;
        let children = if self.swapped {
            vec![right.clone(), left.clone()]
        } else {
            vec![left.clone(), right.clone()]
        };
        div()
            .class("flex flex-row w-full h-full")
            .children(children)
            .build()
    }
    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
        if matches!(event, Event::Key(_)) {
            self.swapped = true;
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        Ok(RootUpdate::Unchanged)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn gfx_007_a_picture_the_frame_cannot_hold_shows_a_message_and_the_app_goes_on() {
    use reactive_tui::builder::core::div;
    // Two canvases side by side, each 260 by 260 cells of 16 by 32 pixels.
    // One pixel per screen pixel would be 4160 by 8320, more than the 64
    // MiB a frame holds for one picture, so each is drawn with the most
    // whole pixels per cell that fit, 8 by 31, as 2080 by 8060 pixels
    // (GFX-010); a frame holds one such picture and not two. The pictures
    // come one after the other and each frame shows the one that is new;
    // when the canvases trade places, one frame has to show both.
    let large_cells = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        cell_pixels: (16, 32),
        ..Default::default()
    };
    let half = |key: &str| {
        let worker = Arc::new(GraphicsWorker::spawn(reference_options(true)).expect("a worker"));
        let props = CanvasProps::new(Arc::new(canvas_support::shapes()))
            .options(reference_options(true))
            .worker(worker);
        div()
            .class("w-1/2 h-full")
            .children(vec![Element::typed::<Canvas>(props)])
            .build()
            .with_key(key)
    };
    let frames = app_input::run_when_output(
        Swapping {
            halves: [half("left"), half("right")],
            swapped: false,
        },
        (520, 260),
        large_cells,
        // The renderer steps over blank cells, so the message's words
        // are apart in the output: its first word is what is waited for.
        vec![
            ("a=T".to_owned(), 1, app_input::key(KeyCode::Enter)),
            ("Canvas:".to_owned(), 1, None),
        ],
    );
    let output: String = frames
        .iter()
        .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
        .collect();
    let sizes = kitty_sizes(&output);
    assert!(
        output.contains("limit") && sizes.len() >= 3 && sizes.contains(&(2080, 8060)),
        "GFX-007: of two canvases whose pictures one frame cannot hold, the pictures sent were {sizes:?}"
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
