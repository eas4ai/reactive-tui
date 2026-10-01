//! canvas-output mechanism, GFX-005 (docs/spec/canvas.md): the canvas shows
//! its frames as Kitty graphics, else Sixel, else block glyphs, as the host
//! allows or an override says; pixel frames replace each other in place; a
//! Kitty host that accepts shared memory gets frames that way; and a slow
//! terminal never gets a backlog. GFX-006, the startup queries that fill the
//! capability report, is in src/backend/suprtui/input_pty.rs.

mod canvas_support;
mod common;

use canvas_support::reference_options;
use common::app_input;
use reactive_tui::app::{App, RootComponent, RootUpdate};
use reactive_tui::backend::{ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::graphics::{
    Canvas, CanvasOutput, CanvasProps, Color, GraphicsOptions, Paint, Path, Scene,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KITTY: &str = "\x1b_G";
/// How the crate's Sixel pictures start: the device control string with a
/// transparent background.
const SIXEL: &str = "\x1bP0;1q";

/// What the backend writes before a picture of any kind: it saves the cursor.
const PICTURE: &str = "\x1b7";

/// A glyph the half-block blitter draws for a cell whose halves differ.
const BLOCK: &str = "▄";

/// Draw block glyphs as half blocks, so a picture with an edge in it holds
/// [`BLOCK`]. Every test sets the same blitter, so their order does not
/// matter.
fn half_blocks() {
    reactive_tui::widgets::display::set_image_blitter(Some(
        reactive_tui::widgets::display::Blitter::HalfBlock,
    ));
}

fn canvas(scene: Scene, options: GraphicsOptions) -> Element {
    Element::typed::<Canvas>(CanvasProps::new(Arc::new(scene)).options(options))
}

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// The output of the frames up to the first that holds `needle`.
fn output_until(options: GraphicsOptions, images: ImageOutputOptions, needle: &str) -> String {
    half_blocks();
    let frames = app_input::run_when_output(
        Root(canvas(canvas_support::shapes(), options)),
        (40, 12),
        images,
        vec![(needle.to_owned(), 1, None)],
    );
    frames
        .iter()
        .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
        .collect()
}

#[test]
fn gfx_005_the_host_decides_kitty_then_sixel_then_blocks() {
    let both = ImageOutputOptions {
        kitty_graphics: true,
        sixel: true,
        ..Default::default()
    };
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    // The backend saves the cursor before any picture, whatever its kind, so
    // a host that gets the wrong kind is seen at once and named.
    let on_both = output_until(reference_options(true), both, PICTURE);
    let on_sixel = output_until(reference_options(true), sixel, PICTURE);
    let on_neither = output_until(
        reference_options(true),
        ImageOutputOptions::default(),
        BLOCK,
    );
    let kinds = |output: &str| (output.contains(KITTY), output.contains(SIXEL));
    assert!(
        kinds(&on_both) == (true, false)
            && kinds(&on_sixel) == (false, true)
            && kinds(&on_neither) == (false, false),
        "GFX-005: (Kitty, Sixel) sent to a host with both: {:?}, to a host with Sixel only: {:?}, to a host with neither: {:?}",
        kinds(&on_both),
        kinds(&on_sixel),
        kinds(&on_neither)
    );
}

#[test]
fn gfx_005_an_override_replaces_the_choice() {
    let kitty = ImageOutputOptions {
        kitty_graphics: true,
        sixel: true,
        ..Default::default()
    };
    let forced = output_until(
        GraphicsOptions {
            output: Some(CanvasOutput::Blocks),
            ..reference_options(true)
        },
        kitty,
        BLOCK,
    );
    assert!(
        !forced.contains(KITTY) && !forced.contains(SIXEL),
        "GFX-005: an override to block glyphs on a Kitty host still sent pixels"
    );
}

/// The variable that tells [`environment_child`] which output to expect.
const EXPECTED: &str = "CANVAS_TEST_EXPECTS";

#[test]
fn gfx_005_the_environment_replaces_the_choice() {
    // Every canvas of a process reads the variable, so each App runs in a
    // process of its own: this test binary, asked for the one test below.
    let run = |value: &str, expected: &str| {
        let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
            .args(["--exact", "environment_child", "--ignored", "--nocapture"])
            .env(CanvasOutput::ENV, value)
            .env(EXPECTED, expected)
            .output()
            .expect("the test binary runs");
        let said = String::from_utf8_lossy(&child.stdout).into_owned()
            + &String::from_utf8_lossy(&child.stderr);
        if child.status.success() && said.contains("1 passed") {
            None
        } else {
            let from = said.find("GFX-005").unwrap_or(0);
            Some(format!("{value:?}: {}", &said[from..]))
        }
    };
    // A name wins over the application's choice, in any case and with
    // space around it. A value that names no output changes nothing.
    let failures: Vec<String> = [
        (" Blocks ", "blocks"),
        ("sixel", "sixel"),
        ("pixels", "kitty"),
    ]
    .into_iter()
    .filter_map(|(value, expected)| run(value, expected))
    .collect();
    assert!(
        failures.is_empty(),
        "GFX-005: the environment's override was not followed: {failures:?}"
    );
}

/// One App on a host that takes Kitty graphics and Sixel, whose application
/// asks for Kitty graphics, under the environment its parent test set.
#[test]
#[ignore = "run by gfx_005_the_environment_replaces_the_choice, which sets the variable"]
fn environment_child() {
    let expected = std::env::var(EXPECTED).expect("the parent test names the output");
    let host = ImageOutputOptions {
        kitty_graphics: true,
        sixel: true,
        ..Default::default()
    };
    let application = GraphicsOptions {
        output: Some(CanvasOutput::Kitty),
        ..reference_options(true)
    };
    let needle = if expected == "blocks" { BLOCK } else { PICTURE };
    let shown = output_until(application, host, needle);
    let wanted = match expected.as_str() {
        "kitty" => (true, false),
        "sixel" => (false, true),
        _ => (false, false),
    };
    assert_eq!(
        (shown.contains(KITTY), shown.contains(SIXEL)),
        wanted,
        "GFX-005: (Kitty, Sixel) sent with {} set to {:?}",
        CanvasOutput::ENV,
        std::env::var(CanvasOutput::ENV)
    );
}

/// The Kitty commands that start a picture, on a host that takes Kitty
/// graphics through shared memory, each cut to its first 40 characters.
fn pictures_with_shared_memory_accepted() -> Vec<String> {
    let shared = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        ..Default::default()
    };
    let output = output_until(reference_options(true), shared, KITTY);
    output
        .split(KITTY)
        .skip(1)
        .filter(|command| command.contains("a=T"))
        .map(|command| command.chars().take(40).collect())
        .collect()
}

// The crate's shared memory is POSIX shared memory.
#[cfg(unix)]
#[test]
fn gfx_005_kitty_frames_travel_through_shared_memory_when_accepted() {
    let pictures = pictures_with_shared_memory_accepted();
    assert!(
        !pictures.is_empty() && pictures.iter().all(|picture| picture.contains("t=s")),
        "GFX-005: with shared memory accepted the Kitty frames were sent as {pictures:?}"
    );
}

// Windows has no POSIX shared memory, and its startup asks no terminal for
// it (GFX-006), so an application that names it still gets its pictures.
#[cfg(not(unix))]
#[test]
fn gfx_005_kitty_frames_are_sent_in_the_command_without_posix_shared_memory() {
    let pictures = pictures_with_shared_memory_accepted();
    assert!(
        !pictures.is_empty() && pictures.iter().all(|picture| !picture.contains("t=s")),
        "GFX-005: on a system without POSIX shared memory the Kitty frames were sent as {pictures:?}"
    );
}

/// The cube at the angle of `frame`, over its background.
fn spinning_cube(frame: usize) -> Scene {
    let angle = frame as f32 * 0.1;
    canvas_support::cube(angle, angle)
}

/// A square that moves to the right with `frame`, in a picture that is
/// transparent everywhere else.
fn moving_square(frame: usize) -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(8.0 * frame as f32, 40.0, 48.0, 48.0),
        &Paint::solid(Color::rgba(200, 40, 40, 255)),
    );
    scene
}

/// How long a root holds its last frame before it stops, so the canvas
/// pictures still being made ready reach the terminal (GFX-009).
const HOLD: Duration = Duration::from_secs(1);

/// Once a root has shown its last frame: hold it, then stop.
fn hold(held: &mut Option<std::time::Instant>) -> RootUpdate {
    if held.get_or_insert_with(std::time::Instant::now).elapsed() >= HOLD {
        RootUpdate::Exit
    } else {
        RootUpdate::Unchanged
    }
}

/// A root that draws the scene of each frame beside a fixed label for
/// `frames` frames, holds the last, and stops.
struct Spinner {
    frame: usize,
    frames: usize,
    held: Option<std::time::Instant>,
    options: GraphicsOptions,
    scene: fn(usize) -> Scene,
}
impl RootComponent for Spinner {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        div()
            .class("flex flex-row w-full h-full")
            .children(vec![
                Element::text("left side").with_class("w-20 h-full"),
                div()
                    .class("w-40 h-full")
                    .children(vec![canvas((self.scene)(self.frame), self.options.clone())])
                    .build(),
            ])
            .build()
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if self.frame + 1 < self.frames {
            self.frame += 1;
            return Ok(RootUpdate::Redraw);
        }
        Ok(hold(&mut self.held))
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// A terminal that takes `delay` per flush and keeps each flush apart.
#[derive(Clone)]
struct SlowTerminal {
    flushes: Arc<Mutex<Vec<Vec<u8>>>>,
    pending: Arc<Mutex<Vec<u8>>>,
    delay: Duration,
}
impl Write for SlowTerminal {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pending.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        std::thread::sleep(self.delay);
        let chunk = std::mem::take(&mut *self.pending.lock().unwrap());
        if !chunk.is_empty() {
            self.flushes.lock().unwrap().push(chunk);
        }
        Ok(())
    }
}

fn run_spinner(
    images: ImageOutputOptions,
    delay: Duration,
    frames: usize,
    scene: fn(usize) -> Scene,
) -> Vec<String> {
    let terminal = SlowTerminal {
        flushes: Arc::default(),
        pending: Arc::default(),
        delay,
    };
    let backend = SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), images).unwrap();
    App::builder()
        .backend(backend)
        .root(Spinner {
            frame: 0,
            frames,
            held: None,
            options: reference_options(true),
            scene,
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
    let flushes = terminal.flushes.lock().unwrap();
    flushes
        .iter()
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

/// A root that draws `canvases` canvases side by side, each a new cube
/// angle every frame for `frames` frames, holds the last, and stops.
#[cfg(target_os = "linux")]
struct Row {
    frame: usize,
    frames: usize,
    held: Option<std::time::Instant>,
    canvases: usize,
}
#[cfg(target_os = "linux")]
impl RootComponent for Row {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        div()
            .class("flex flex-row w-full h-full")
            .children(
                (0..self.canvases)
                    .map(|n| {
                        let angle = (self.frame + n) as f32 * 0.1;
                        div()
                            .class("w-10 h-full")
                            .children(vec![canvas(
                                canvas_support::cube(angle, angle),
                                reference_options(true),
                            )])
                            .build()
                    })
                    .collect::<Vec<_>>(),
            )
            .build()
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if self.frame + 1 < self.frames {
            self.frame += 1;
            return Ok(RootUpdate::Redraw);
        }
        Ok(hold(&mut self.held))
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// A terminal that notes, when output reaches it, whether the shared-memory
/// object each Kitty picture names is there to be read.
#[cfg(target_os = "linux")]
#[derive(Clone, Default)]
struct SharedReader {
    pending: Arc<Mutex<Vec<u8>>>,
    /// For each picture, in the order they came, its image id and whether
    /// its object was there.
    found: Arc<Mutex<Vec<(String, bool)>>>,
}
#[cfg(target_os = "linux")]
impl Write for SharedReader {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pending.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        use base64::Engine;
        let chunk = std::mem::take(&mut *self.pending.lock().unwrap());
        let chunk = String::from_utf8_lossy(&chunk);
        let found = chunk
            .split(KITTY)
            .skip(1)
            .filter(|command| command.contains("a=T") && command.contains("t=s"))
            .filter_map(|command| {
                let (control, rest) = command.split_once(';')?;
                let id = control
                    .split(',')
                    .find_map(|key| key.strip_prefix("i="))?
                    .to_owned();
                let name = rest.split_once('\x1b')?.0;
                let name = base64::engine::general_purpose::STANDARD
                    .decode(name)
                    .ok()?;
                let name = String::from_utf8(name).ok()?;
                let there = std::path::Path::new("/dev/shm")
                    .join(name.trim_start_matches('/'))
                    .exists();
                Some((id, there))
            });
        self.found.lock().unwrap().extend(found);
        Ok(())
    }
}

// The objects are looked for where Linux keeps them, in /dev/shm.
#[cfg(target_os = "linux")]
#[test]
fn gfx_005_every_canvas_of_a_frame_keeps_its_shared_picture() {
    const CANVASES: usize = 6;
    let shared = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        ..Default::default()
    };
    let terminal = SharedReader::default();
    let backend = SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), shared).unwrap();
    App::builder()
        .backend(backend)
        .root(Row {
            frame: 0,
            frames: 12,
            held: None,
            canvases: CANVASES,
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
    // Each canvas's pictures are written as they are made ready
    // (GFX-009), so the pictures of all six are made while the objects of
    // the others still wait to be read.
    let found = terminal.found.lock().unwrap();
    let canvases: std::collections::BTreeSet<&String> = found.iter().map(|(id, _)| id).collect();
    let missing: Vec<&(String, bool)> = found.iter().filter(|(_, there)| !there).collect();
    assert!(
        canvases.len() == CANVASES && missing.is_empty(),
        "GFX-005: of {} pictures for {} canvases of {CANVASES}, these had no shared memory when they were written: {:?}",
        found.len(),
        canvases.len(),
        &missing[..missing.len().min(6)]
    );
}

/// The (row, column) of every cursor position a chunk of output sets, 1-based.
fn cursor_moves(chunk: &str) -> Vec<(u16, u16)> {
    let mut moves = Vec::new();
    for part in chunk.split("\x1b[").skip(1) {
        let digits: String = part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ';')
            .collect();
        if part[digits.len()..].starts_with('H') {
            let mut numbers = digits.split(';').map(|n| n.parse::<u16>().unwrap_or(1));
            moves.push((numbers.next().unwrap_or(1), numbers.next().unwrap_or(1)));
        }
    }
    moves
}

#[test]
fn gfx_005_pixel_frames_replace_each_other_in_place() {
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    let flushes = run_spinner(sixel, Duration::ZERO, 12, spinning_cube);
    let pictures: Vec<&String> = flushes
        .iter()
        .filter(|chunk| chunk.contains(SIXEL))
        .collect();
    let later = &pictures[pictures.len().min(1)..];
    let outside: Vec<(u16, u16)> = later
        .iter()
        .flat_map(|chunk| cursor_moves(chunk))
        .filter(|&(_, column)| column <= 20)
        .collect();
    assert!(
        pictures.len() >= 2
            && later.iter().all(|chunk| !chunk.contains("\x1b[2J"))
            && outside.is_empty(),
        "GFX-005: of {} Sixel frames the later ones cleared the screen or moved to cells outside the canvas: {:?}",
        pictures.len(),
        &outside[..outside.len().min(5)]
    );
}

/// A root that draws the spinning cube beside a fixed label and, from
/// frame `covered` to frame `gone`, a note over part of the canvas; from
/// frame `gone` on it draws no canvas.
struct Covered {
    frame: usize,
    frames: usize,
    covered: usize,
    gone: usize,
}
impl RootComponent for Covered {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let mut right = Vec::new();
        if self.frame < self.gone {
            right.push(canvas(spinning_cube(self.frame), reference_options(true)));
        }
        if self.frame >= self.covered {
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
                div().class("relative w-40 h-full").children(right).build(),
            ])
            .build()
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

/// What a terminal that takes `images` is sent, flush by flush, while a
/// note comes over the canvas at frame 12 and the canvas goes at frame 28.
fn run_covered(images: ImageOutputOptions) -> Vec<String> {
    let terminal = SlowTerminal {
        flushes: Arc::default(),
        pending: Arc::default(),
        delay: Duration::ZERO,
    };
    let backend = SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), images).unwrap();
    App::builder()
        .backend(backend)
        .root(Covered {
            frame: 0,
            frames: 40,
            covered: 12,
            gone: 28,
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
    let flushes = terminal.flushes.lock().unwrap();
    flushes
        .iter()
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

/// Where in `flushes` the first picture, the note and the last picture are,
/// when they come in that order.
fn covered_order(flushes: &[String], picture: &str) -> Option<(usize, usize, usize)> {
    let first = flushes.iter().position(|chunk| chunk.contains(picture))?;
    let noted = flushes.iter().position(|chunk| chunk.contains("a note"))?;
    let last = flushes.iter().rposition(|chunk| chunk.contains(picture))?;
    (first < noted && noted < last).then_some((first, noted, last))
}

#[test]
fn gfx_005_cells_over_a_canvas_change_without_clearing_the_screen() {
    let flushes = run_covered(ImageOutputOptions {
        sixel: true,
        ..Default::default()
    });
    let (first, _, last) =
        covered_order(&flushes, SIXEL).expect("a picture, the note, and a picture after it");
    let later = &flushes[first + 1..];
    let cleared = later
        .iter()
        .filter(|chunk| chunk.contains("\x1b[2J"))
        .count();
    let outside: Vec<(u16, u16)> = later
        .iter()
        .flat_map(|chunk| cursor_moves(chunk))
        .filter(|&(_, column)| column <= 20)
        .collect();
    // Once the canvas is gone, the cells it covered are written again, so
    // nothing of its last picture stays: the rows of its area are set.
    let rows_after: std::collections::BTreeSet<u16> = flushes[last + 1..]
        .iter()
        .flat_map(|chunk| cursor_moves(chunk))
        .map(|(row, _)| row)
        .collect();
    assert!(
        cleared == 0 && outside.is_empty() && (1..=20).all(|row| rows_after.contains(&row)),
        "GFX-005: with a note drawn over the canvas from frame 12 and the canvas gone from frame 28, {cleared} later flushes cleared the screen, cells outside the canvas were set at {:?}, and after the last picture the rows written were {rows_after:?}",
        &outside[..outside.len().min(5)]
    );
}

#[test]
fn gfx_005_cells_over_a_kitty_canvas_change_without_sending_it_away() {
    let flushes = run_covered(ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    });
    let (first, _, last) =
        covered_order(&flushes, "a=T").expect("a picture, the note, and a picture after it");
    let deleted = |chunks: &[String]| chunks.iter().filter(|chunk| chunk.contains("a=d")).count();
    let outside: Vec<(u16, u16)> = flushes[first + 1..]
        .iter()
        .flat_map(|chunk| cursor_moves(chunk))
        .filter(|&(_, column)| column <= 20)
        .collect();
    assert!(
        deleted(&flushes[first + 1..=last]) == 0
            && deleted(&flushes[last + 1..]) == 1
            && outside.is_empty(),
        "GFX-005: with a note drawn over the canvas from frame 12 and the canvas gone from frame 28, a Kitty host was sent {} deletions while the canvas showed and {} after it, and cells outside the canvas were set at {:?}",
        deleted(&flushes[first + 1..=last]),
        deleted(&flushes[last + 1..]),
        &outside[..outside.len().min(5)]
    );
}

/// The size a Sixel picture states for itself and how many of its pixels
/// its data leaves unset, which a terminal shows as they were before.
/// `picture` is what follows [`SIXEL`].
fn sixel_unset(picture: &str) -> ((usize, usize), usize) {
    let (size, set) = canvas_support::sixel_pixels(picture);
    (size, set.iter().filter(|pixel| !**pixel).count())
}

#[test]
fn gfx_005_a_sixel_frame_leaves_nothing_of_the_last() {
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    let flushes = run_spinner(sixel, Duration::ZERO, 12, moving_square);
    let pictures: Vec<((usize, usize), usize)> = flushes
        .iter()
        .flat_map(|chunk| {
            chunk
                .split(SIXEL)
                .skip(1)
                .map(sixel_unset)
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        pictures.len() >= 2
            && pictures
                .iter()
                .all(|(size, unset)| size.0 > 0 && size.1 > 0 && *unset == 0),
        "GFX-005: a square that moves over a transparent picture was sent as Sixel pictures of (size, pixels left as the screen had them): {:?}",
        &pictures[..pictures.len().min(6)]
    );
}

#[test]
fn gfx_005_a_slow_terminal_gets_no_backlog() {
    let kitty = ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    };
    let flushes = run_spinner(kitty, Duration::from_millis(60), 20, spinning_cube);
    let per_flush: Vec<usize> = flushes
        .iter()
        .map(|chunk| {
            chunk
                .split(KITTY)
                .skip(1)
                .filter(|c| c.contains("a=T"))
                .count()
        })
        .collect();
    assert!(
        per_flush.iter().any(|&n| n > 0) && per_flush.iter().all(|&n| n <= 1),
        "GFX-005: with a terminal taking 60 ms per flush, canvas frames per flush were {per_flush:?}"
    );
}
