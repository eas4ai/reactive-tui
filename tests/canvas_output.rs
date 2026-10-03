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
/// How an iTerm2 inline picture starts.
const INLINE: &str = "\x1b]1337;File=";

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

/// A terminal that takes `delay` per flush, keeps each flush apart and
/// counts the pictures it is sent.
#[derive(Clone)]
struct SlowTerminal {
    flushes: Arc<Mutex<Vec<Vec<u8>>>>,
    pending: Arc<Mutex<Vec<u8>>>,
    delay: Duration,
    pictures: Arc<std::sync::atomic::AtomicUsize>,
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
            let text = String::from_utf8_lossy(&chunk);
            let pictures = text.matches(SIXEL).count()
                + text.matches("\x1b_Ga=T").count()
                + text.matches(INLINE).count();
            self.pictures
                .fetch_add(pictures, std::sync::atomic::Ordering::SeqCst);
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
        pictures: Arc::default(),
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

/// A terminal that notes, when output reaches it and `delay` later, whether
/// the shared-memory object each Kitty picture names is there to be read.
#[cfg(target_os = "linux")]
#[derive(Clone, Default)]
struct SharedReader {
    delay: Duration,
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
        std::thread::sleep(self.delay);
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

// The objects are looked for where Linux keeps them, in /dev/shm.
#[cfg(target_os = "linux")]
#[test]
fn gfx_005_a_slow_terminal_finds_the_shared_memory_of_every_picture_it_is_sent() {
    // The terminal reads each picture's object 150 ms after it was sent,
    // while the canvas's next pictures are made ready apart (GFX-009); an
    // object that the making of later pictures removes is not there.
    let shared = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        ..Default::default()
    };
    let terminal = SharedReader {
        delay: Duration::from_millis(150),
        ..Default::default()
    };
    let backend = SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), shared).unwrap();
    App::builder()
        .backend(backend)
        .root(Spinner {
            frame: 0,
            frames: 40,
            held: None,
            options: reference_options(true),
            scene: spinning_cube,
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
    let found = terminal.found.lock().unwrap();
    let missing = found.iter().filter(|(_, there)| !there).count();
    assert!(
        found.len() >= 3 && missing == 0,
        "GFX-005: of {} pictures sent through shared memory to a terminal that reads 150 ms later, {missing} had no object by then",
        found.len()
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

/// Where a [`Covered`] root is: the canvas alone, then a note over part of
/// it from the pictures the terminal had been sent then, then the note
/// alone from a moment on.
#[derive(Clone, Copy)]
enum Cover {
    Bare,
    Noted(usize),
    Gone(std::time::Instant),
}

/// A root that draws the spinning cube beside a fixed label until the
/// terminal has been sent a picture, then a note over part of the canvas
/// until two more pictures came, then the note without the canvas for a
/// moment, and stops. Pictures reach the terminal after their frames
/// (GFX-009), so the root goes on by the pictures, not by frames.
struct Covered {
    frame: usize,
    cover: Cover,
    pictures: Arc<std::sync::atomic::AtomicUsize>,
    deadline: std::time::Instant,
}
impl RootComponent for Covered {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let mut right = Vec::new();
        if !matches!(self.cover, Cover::Gone(_)) {
            right.push(canvas(spinning_cube(self.frame), reference_options(true)));
        }
        if !matches!(self.cover, Cover::Bare) {
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
        let sent = self.pictures.load(std::sync::atomic::Ordering::SeqCst);
        let late = std::time::Instant::now() >= self.deadline;
        self.cover = match self.cover {
            Cover::Bare if sent >= 1 || late => Cover::Noted(sent),
            // Two more, so that one was made for the note's cells.
            Cover::Noted(before) if sent >= before + 2 || late => {
                Cover::Gone(std::time::Instant::now())
            }
            Cover::Gone(at) if at.elapsed() >= Duration::from_millis(300) => {
                return Ok(RootUpdate::Exit)
            }
            cover => cover,
        };
        Ok(RootUpdate::Redraw)
    }
    /// The backend writes to memory and has no terminal to read keys from.
    fn accepts_input(&self) -> bool {
        false
    }
}

/// What a terminal that takes `images` is sent, flush by flush, while a
/// note comes over the canvas and then the canvas goes.
fn run_covered(images: ImageOutputOptions) -> Vec<String> {
    let terminal = SlowTerminal {
        flushes: Arc::default(),
        pending: Arc::default(),
        delay: Duration::ZERO,
        pictures: Arc::default(),
    };
    let backend = SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), images).unwrap();
    App::builder()
        .backend(backend)
        .root(Covered {
            frame: 0,
            cover: Cover::Bare,
            pictures: Arc::clone(&terminal.pictures),
            deadline: std::time::Instant::now() + Duration::from_secs(20),
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
        "GFX-005: with a note drawn over the canvas and then the canvas gone, {cleared} later flushes cleared the screen, cells outside the canvas were set at {:?}, and after the last picture the rows written were {rows_after:?}",
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
        "GFX-005: with a note drawn over the canvas and then the canvas gone, a Kitty host was sent {} deletions while the canvas showed and {} after it, and cells outside the canvas were set at {:?}",
        deleted(&flushes[first + 1..=last]),
        deleted(&flushes[last + 1..]),
        &outside[..outside.len().min(5)]
    );
}

/// What lies over the lower canvas in [`Stacked`].
#[derive(Clone, Copy)]
enum Over {
    /// A canvas of green at half alpha, Sixel like the one below.
    Canvas,
    /// An image of green at half alpha, Sixel or inline.
    Image(reactive_tui::widgets::display::ImageDisplayMode),
}

/// A root with a canvas in `lower` filling a four by one terminal, a
/// translucent green canvas or image over its middle two cells, and a black
/// background. Once both pictures reached the terminal the lower canvas
/// takes `then` when given, and the root stops after two more pictures or
/// at its deadline.
struct Stacked {
    lower: Color,
    then: Option<Color>,
    over: Over,
    mark: Option<usize>,
    /// When the pictures the root waited for had come: it stops a moment
    /// later, so a picture still being made ready arrives too.
    hold: Option<std::time::Instant>,
    pictures: Arc<std::sync::atomic::AtomicUsize>,
    deadline: std::time::Instant,
}
impl RootComponent for Stacked {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let solid = |color: Color| {
            let mut scene = Scene::new();
            scene.fill(&Path::rect(0.0, 0.0, 1000.0, 1000.0), &Paint::solid(color));
            scene
        };
        let sixel = || GraphicsOptions {
            output: Some(CanvasOutput::Sixel),
            ..reference_options(true)
        };
        let green = Color::rgba(0, 255, 0, 128);
        let over = match self.over {
            Over::Canvas => canvas(solid(green), sixel()),
            Over::Image(mode) => Element::from(
                reactive_tui::widgets::display::Image::from_raw_bytes(
                    [0u8, 255, 0, 128].repeat(8),
                    4,
                    2,
                    reactive_tui::widgets::display::ImageFormat::RGBA8888,
                )
                .with_display_mode(mode),
            )
            .with_class("w-2 h-1"),
        };
        div()
            .class("relative w-full h-full bg-black")
            .children(vec![
                canvas(solid(self.lower.clone()), sixel()),
                div()
                    .class("absolute left-1 top-0 w-2 h-1")
                    .child(over)
                    .build(),
            ])
            .build()
    }
    fn update(&mut self) -> Result<RootUpdate> {
        let sent = self.pictures.load(std::sync::atomic::Ordering::SeqCst);
        if std::time::Instant::now() >= self.deadline {
            return Ok(RootUpdate::Exit);
        }
        if let Some(since) = self.hold {
            return Ok(if since.elapsed() >= Duration::from_millis(300) {
                RootUpdate::Exit
            } else {
                RootUpdate::Redraw
            });
        }
        match (self.mark, self.then.clone()) {
            (None, Some(next)) if sent >= 2 => {
                self.lower = next;
                self.then = None;
                self.mark = Some(sent);
            }
            (None, None) if sent >= 2 => self.hold = Some(std::time::Instant::now()),
            (Some(mark), _) if sent >= mark + 2 => self.hold = Some(std::time::Instant::now()),
            _ => {}
        }
        Ok(RootUpdate::Redraw)
    }
    fn accepts_input(&self) -> bool {
        false
    }
}

/// What a terminal taking `images`, `delay` per flush, is sent for
/// [`Stacked`], flush by flush.
fn run_stacked(
    images: ImageOutputOptions,
    over: Over,
    then: Option<Color>,
    delay: Duration,
) -> Vec<String> {
    let terminal = SlowTerminal {
        flushes: Arc::default(),
        pending: Arc::default(),
        delay,
        pictures: Arc::default(),
    };
    let backend = SuprTuiBackend::with_writer_and_images(4, 1, terminal.clone(), images).unwrap();
    App::builder()
        .backend(backend)
        .root(Stacked {
            lower: Color::rgba(255, 0, 0, 255),
            then,
            over,
            mark: None,
            hold: None,
            pictures: Arc::clone(&terminal.pictures),
            deadline: std::time::Instant::now() + Duration::from_secs(10),
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

/// A color as Sixel states it: red, green and blue in percent.
type Percent = (usize, usize, usize);
/// A Sixel picture: its declared width, its palette colors and its text.
type SixelPicture = (usize, Vec<Percent>, String);

/// The Sixel pictures in `flushes`, in order.
fn sixel_pictures(flushes: &[String]) -> Vec<SixelPicture> {
    flushes
        .iter()
        .flat_map(|chunk| chunk.split(SIXEL).skip(1).map(str::to_owned))
        .map(|picture| {
            let (size, _) = canvas_support::sixel_pixels(&picture);
            let colors = picture
                .split('#')
                .skip(1)
                .filter_map(|entry| {
                    let numbers: Vec<usize> = entry
                        .split(|c: char| !c.is_ascii_digit())
                        .take_while(|n| !n.is_empty())
                        .filter_map(|n| n.parse().ok())
                        .collect();
                    (numbers.len() >= 5 && numbers[1] == 2)
                        .then(|| (numbers[2], numbers[3], numbers[4]))
                })
                .collect();
            (size.0, colors, picture)
        })
        .collect()
}

fn percent_near(a: Percent, b: Percent) -> bool {
    a.0.abs_diff(b.0) <= 2 && a.1.abs_diff(b.1) <= 2 && a.2.abs_diff(b.2) <= 2
}

/// GFX-009: a translucent canvas over another canvas, both made ready on
/// the picture thread, is composed over the lower canvas's pixels, not over
/// the cell background: green at half alpha over red is sent as half red,
/// half green. When the lower canvas changes, the picture over it is made
/// again over the new pixels. The lower picture leaves the covered cells
/// unset, so whichever picture arrives later, the composite stands.
#[test]
fn gfx_009_a_translucent_canvas_composes_over_the_canvas_below_it() {
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    for delay in [Duration::ZERO, Duration::from_millis(20)] {
        let flushes = run_stacked(
            sixel,
            Over::Canvas,
            Some(Color::rgba(0, 0, 255, 255)),
            delay,
        );
        let pictures = sixel_pictures(&flushes);
        let uppers: Vec<_> = pictures.iter().filter(|(w, _, _)| *w == 16).collect();
        let lowers: Vec<_> = pictures.iter().filter(|(w, _, _)| *w == 32).collect();
        assert!(
            uppers.len() >= 2 && !lowers.is_empty(),
            "GFX-009: with {delay:?} per flush, both canvases are sent and the upper one again after the lower changed: {} upper and {} lower pictures of {:?}",
            uppers.len(),
            lowers.len(),
            pictures.iter().map(|(w, c, _)| (*w, c.clone())).collect::<Vec<_>>()
        );
        assert!(
            uppers[0].1.iter().any(|c| percent_near(*c, (50, 50, 0))),
            "GFX-009 ({delay:?}): green at half alpha over the red canvas is half red, half green, found {:?}",
            uppers[0].1
        );
        assert!(
            uppers
                .last()
                .unwrap()
                .1
                .iter()
                .any(|c| percent_near(*c, (0, 50, 50))),
            "GFX-009 ({delay:?}): after the lower canvas turned blue the picture over it is made again over blue, found {:?}",
            uppers.last().unwrap().1
        );
        for (_, _, picture) in &lowers {
            let (size, set) = canvas_support::sixel_pixels(picture);
            let covered: Vec<bool> = (0..size.1)
                .flat_map(|y| (8..24).map(move |x| (x, y)))
                .map(|(x, y)| set[y * size.0 + x])
                .collect();
            assert!(
                covered.iter().all(|pixel| !*pixel),
                "GFX-009: the lower canvas leaves the cells under the upper one unset:\n{picture}"
            );
        }
    }
}

/// GFX-009: a translucent image over a canvas made ready on the picture
/// thread is composed over the canvas's pixels too, as Sixel and as an
/// inline picture.
#[test]
fn gfx_009_a_translucent_image_composes_over_the_canvas_below_it() {
    use reactive_tui::widgets::display::ImageDisplayMode;
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    let flushes = run_stacked(
        sixel,
        Over::Image(ImageDisplayMode::Sixel),
        None,
        Duration::ZERO,
    );
    // An image's Sixel picture is drawn from the screen's corner, so it is
    // wider than the image's cells and carries the canvas's pixels beside it.
    let pictures = sixel_pictures(&flushes);
    let uppers: Vec<_> = pictures.iter().filter(|(w, _, _)| *w != 32).collect();
    assert!(
        !uppers.is_empty()
            && uppers
                .last()
                .unwrap()
                .1
                .iter()
                .any(|c| percent_near(*c, (50, 50, 0))),
        "GFX-009: a Sixel image of green at half alpha over the red canvas is half red, half green: {:?}\n{}",
        pictures.iter().map(|(w, c, _)| (*w, c.clone())).collect::<Vec<_>>(),
        flushes.concat().replace('\x1b', "<ESC>")
    );
    let both = ImageOutputOptions {
        sixel: true,
        iterm2_inline: true,
        ..Default::default()
    };
    let flushes = run_stacked(
        both,
        Over::Image(ImageDisplayMode::ITerm2Inline),
        None,
        Duration::ZERO,
    );
    use base64::Engine;
    let inline: Vec<image::RgbaImage> = flushes
        .iter()
        .flat_map(|chunk| chunk.split(INLINE).skip(1).map(str::to_owned))
        .filter_map(|payload| {
            let encoded = payload.split(':').nth(1)?.split('\x07').next()?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .ok()?;
            image::load_from_memory(&bytes)
                .ok()
                .map(|img| img.to_rgba8())
        })
        .collect();
    let pixel = inline
        .last()
        .map(|picture| *picture.get_pixel(0, 0))
        .expect("an inline picture of the image over the canvas");
    assert!(
        pixel[0].abs_diff(127) <= 2 && pixel[1].abs_diff(128) <= 2 && pixel[2] <= 2,
        "GFX-009: an inline image of green at half alpha over the red canvas is half red, half green, found {pixel:?}"
    );
}

/// The size a Sixel picture states for itself and how many of its pixels
/// its data leaves unset, which a terminal shows as they were before.
/// `picture` is what follows [`SIXEL`].
fn sixel_unset(picture: &str) -> ((usize, usize), usize) {
    let (size, set) = canvas_support::sixel_pixels(picture);
    (size, set.iter().filter(|pixel| !**pixel).count())
}

/// GFX-005: a canvas whose pixel height is not a multiple of six (one cell
/// row is 16 pixels, two are 32) is sent whole as Sixel: the picture states
/// its true height, every pixel is set, and the partial last band sets no
/// pixel below the canvas.
#[test]
fn gfx_005_a_sixel_canvas_keeps_its_last_partial_band() {
    let sixel = ImageOutputOptions {
        sixel: true,
        ..Default::default()
    };
    for rows in [1u16, 2, 3] {
        let mut scene = Scene::new();
        scene.fill(
            &Path::rect(0.0, 0.0, 1000.0, 1000.0),
            &Paint::solid(Color::rgba(255, 0, 0, 255)),
        );
        let options = GraphicsOptions {
            output: Some(CanvasOutput::Sixel),
            ..reference_options(true)
        };
        let frames = app_input::run_when_output(
            Root(canvas(scene, options)),
            (4, rows),
            sixel,
            vec![(SIXEL.to_owned(), 1, None)],
        );
        let output: String = frames
            .iter()
            .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
            .collect();
        let picture = output.split(SIXEL).nth(1).expect("a Sixel picture");
        let (size, set) = canvas_support::sixel_pixels(picture);
        let expected = (32usize, 16 * usize::from(rows));
        assert_eq!(
            size, expected,
            "GFX-005: a {rows}-row canvas states its whole picture, not whole bands only"
        );
        assert!(
            set.iter().all(|pixel| *pixel),
            "GFX-005: every pixel of the {rows}-row canvas is sent ({} of {} unset)",
            set.iter().filter(|p| !**p).count(),
            set.len()
        );
        assert_eq!(
            canvas_support::sixel_outside(picture),
            0,
            "GFX-005: the partial last band sets no pixel below the {rows}-row canvas"
        );
    }
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

/// The control keys of the first Kitty command in `output` that starts a
/// picture, each as (key, value).
fn first_picture_controls(output: &str) -> Vec<(String, String)> {
    output
        .split(KITTY)
        .skip(1)
        .find(|command| command.contains("a=T"))
        .map(|command| {
            command
                .split(';')
                .next()
                .unwrap_or("")
                .split(',')
                .filter_map(|pair| pair.split_once('='))
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

/// The number `key` has among `controls`, if it has one.
fn control(controls: &[(String, String)], key: &str) -> Option<u32> {
    controls
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| value.parse().ok())
}

/// What a terminal of `size` cells that takes `images` is sent by an App
/// whose one canvas, of the shapes scene on the software renderer, fills
/// the screen, up to the first frame whose output holds `needle`.
fn canvas_output_at(size: (u16, u16), images: ImageOutputOptions, needle: &str) -> String {
    half_blocks();
    app_input::run_when_output(
        Root(canvas(canvas_support::shapes(), reference_options(true))),
        size,
        images,
        vec![(needle.to_owned(), 1, None)],
    )
    .iter()
    .map(|frame| String::from_utf8_lossy(&frame.output).into_owned())
    .collect()
}

#[test]
fn gfx_010_a_wide_terminal_gets_its_picture_one_pixel_per_screen_pixel() {
    // 520 columns of 9-pixel cells are 4680 pixels, wider than the 4096 the
    // canvas once drew at most; the placement names the cells it covers.
    let host = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        cell_pixels: (9, 18),
        ..Default::default()
    };
    let controls = first_picture_controls(&canvas_output_at((520, 60), host, "a=T"));
    let got = ["s", "v", "c", "r"].map(|key| control(&controls, key));
    assert!(
        got == [Some(4680), Some(1080), Some(520), Some(60)],
        "GFX-010: a canvas of 520 by 60 cells of 9 by 18 pixels was sent a picture whose s, v, c and r are {got:?}, not 4680, 1080, 520 and 60"
    );
}

#[test]
fn gfx_010_a_picture_too_large_for_the_command_keeps_whole_pixels_per_cell() {
    // 520 by 260 cells of 8 by 16 pixels are 17.3 million pixels, and a
    // picture sent in the command itself has room for 12 million: the
    // canvas draws the largest whole pixels per cell that fit, and the
    // placement names the cells, which the terminal scales the picture to.
    let host = ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    };
    let controls = first_picture_controls(&canvas_output_at((520, 260), host, "a=T"));
    let got = ["s", "v", "c", "r"].map(|key| control(&controls, key));
    let [width, height, columns, rows] = got;
    let whole = width.zip(height).is_some_and(|(width, height)| {
        width > 0
            && height > 0
            && width % 520 == 0
            && height % 260 == 0
            && u64::from(width) * u64::from(height) <= 12_000_000
    });
    assert!(
        whole && columns == Some(520) && rows == Some(260),
        "GFX-010: a canvas of 520 by 260 cells of 8 by 16 pixels sent in the command was given a picture whose s, v, c and r are {got:?}: not whole pixels per cell within 12 million pixels, or a placement that does not name its 520 by 260 cells"
    );
}

#[test]
fn gfx_010_on_sixel_the_picture_is_the_cells_pixels() {
    // The same 520 by 60 cells of 9 by 18 pixels as Sixel: the raster is
    // 4680 by 1080 pixels, one per screen pixel.
    let host = ImageOutputOptions {
        sixel: true,
        cell_pixels: (9, 18),
        ..Default::default()
    };
    let output = canvas_output_at((520, 60), host, SIXEL);
    let picture = output.split(SIXEL).nth(1).expect("a Sixel picture");
    let (size, _) = canvas_support::sixel_pixels(picture);
    assert_eq!(
        size,
        (4680, 1080),
        "GFX-010: a Sixel canvas of 520 by 60 cells of 9 by 18 pixels was sent a raster of {size:?}"
    );
}
