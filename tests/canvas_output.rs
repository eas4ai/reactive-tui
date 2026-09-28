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
use reactive_tui::graphics::{Canvas, CanvasOutput, CanvasProps, GraphicsOptions, Scene};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KITTY: &str = "\x1b_G";
/// How the crate's Sixel pictures start: the device control string with a
/// transparent background.
const SIXEL: &str = "\x1bP0;1q";

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
    let on_both = output_until(reference_options(true), both, KITTY);
    let on_sixel = output_until(reference_options(true), sixel, SIXEL);
    let on_neither = output_until(
        reference_options(true),
        ImageOutputOptions::default(),
        BLOCK,
    );
    assert!(
        !on_both.contains(SIXEL)
            && !on_sixel.contains(KITTY)
            && !on_neither.contains(KITTY)
            && !on_neither.contains(SIXEL),
        "GFX-005: a Kitty and Sixel host, a Sixel host and a host with neither did not get Kitty, Sixel and block glyphs"
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

#[test]
fn gfx_005_kitty_frames_travel_through_shared_memory_when_accepted() {
    let shared = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: true,
        ..Default::default()
    };
    let output = output_until(reference_options(true), shared, KITTY);
    let commands: Vec<&str> = output.split(KITTY).skip(1).collect();
    assert!(
        !commands.is_empty()
            && commands
                .iter()
                .filter(|command| command.contains("a=T"))
                .all(|command| command.contains("t=s")),
        "GFX-005: with shared memory accepted the Kitty frames were sent as {:?}",
        commands
            .iter()
            .map(|c| &c[..c.len().min(40)])
            .collect::<Vec<_>>()
    );
}

/// A root that draws a new cube angle each frame beside a fixed label, and
/// stops after `frames`.
struct Spinner {
    frame: usize,
    frames: usize,
    options: GraphicsOptions,
}
impl RootComponent for Spinner {
    fn render(&self) -> Element {
        use reactive_tui::builder::core::div;
        let angle = self.frame as f32 * 0.1;
        div()
            .class("flex flex-row w-full h-full")
            .children(vec![
                Element::text("left side").with_class("w-20 h-full"),
                div()
                    .class("w-40 h-full")
                    .children(vec![canvas(
                        canvas_support::cube(angle, angle),
                        self.options.clone(),
                    )])
                    .build(),
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

fn run_spinner(images: ImageOutputOptions, delay: Duration, frames: usize) -> Vec<String> {
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
            options: reference_options(true),
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
    let flushes = run_spinner(sixel, Duration::ZERO, 12);
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

#[test]
fn gfx_005_a_slow_terminal_gets_no_backlog() {
    let kitty = ImageOutputOptions {
        kitty_graphics: true,
        ..Default::default()
    };
    let flushes = run_spinner(kitty, Duration::from_millis(60), 20);
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
