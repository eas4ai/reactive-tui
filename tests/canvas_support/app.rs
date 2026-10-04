//! The App-on-a-memory-terminal harness the picture suites share
//! (charts_pictures.rs, pixel_looks.rs): a backend that writes to memory and
//! takes Kitty graphics, Sixel or no pixels, a script of events, a stop rule,
//! every frame's bytes kept apart with the App's work and its wait in
//! present, and decoders for the Kitty pictures and Sixel rasters the bytes
//! hold.
#![allow(dead_code)]

use reactive_tui::app::{App, AppWaker, RootComponent, RootUpdate};
use reactive_tui::backend::{Backend, FrameLayout, ImageOutputOptions, SuprTuiBackend};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
use reactive_tui::graphics::GraphicsFrame;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The pixels of one cell, as the tests' terminals report them.
pub const CELL: (u16, u16) = (8, 16);
/// How a Kitty graphics command starts.
pub const KITTY: &str = "\x1b_G";
/// How the crate's Sixel pictures start: the device control string with a
/// transparent background.
pub const SIXEL: &str = "\x1bP0;1q";
/// How long a run may take before it ends on its own. A hang guard, not a
/// bound: a slow host fails a test on what it shows, not on how long it took.
pub const GUARD: Duration = Duration::from_secs(20);
/// One frame at 60 frames a second (CHT-039).
pub const BOUND: Duration = Duration::from_micros(16_600);

/// A host that takes Kitty graphics in the command, so the picture's pixels
/// are in the bytes.
pub fn kitty() -> ImageOutputOptions {
    ImageOutputOptions {
        kitty_graphics: true,
        cell_pixels: CELL,
        ..Default::default()
    }
}

/// A host that takes Sixel and nothing else.
pub fn sixel() -> ImageOutputOptions {
    ImageOutputOptions {
        sixel: true,
        cell_pixels: CELL,
        ..Default::default()
    }
}

/// A host that takes no pixels.
pub fn no_pixels() -> ImageOutputOptions {
    ImageOutputOptions {
        cell_pixels: CELL,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// The harness: an App on a backend that writes to memory, with a script of
// events, a stop rule, and every frame's bytes kept apart.

/// The App's terminal in memory: every byte, and a count of the pictures
/// among them, counted as they are written so the stop rule can read it.
#[derive(Clone, Default)]
pub struct Terminal {
    pub bytes: Arc<Mutex<Vec<u8>>>,
    pub pictures: Arc<AtomicUsize>,
}

impl Write for Terminal {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let marks = memchr::memchr_iter(0x1b, bytes)
            .filter(|&at| {
                [&b"\x1b_Ga=T"[..], SIXEL.as_bytes()]
                    .iter()
                    .any(|mark| bytes[at..].starts_with(mark))
            })
            .count();
        self.pictures.fetch_add(marks, Ordering::SeqCst);
        self.bytes.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// One presented frame: the bytes it added to the terminal, the App's work
/// before it (from the input wait returning to present, BAR-005's measure),
/// its wait in present (GFX-009's), when it began, the pictures the terminal
/// had been sent by then, and the threads alive in the process.
pub struct Frame {
    pub output: Vec<u8>,
    pub work: Duration,
    pub waited: Duration,
    pub began: Instant,
    pub pictures: usize,
    pub threads: BTreeSet<String>,
    /// How many screen-reader nodes of the Image role the frame's tree held.
    pub image_nodes: usize,
    /// The texts of the frame's live regions.
    pub live: Vec<String>,
    /// Whether an Image node of the frame's tree read busy.
    pub busy: bool,
}

/// What the frame's element tree tells a screen reader: its Image nodes,
/// and the texts of its live regions.
pub fn described(element: &Element, images: &mut usize, live: &mut Vec<String>, busy: &mut bool) {
    if let Some(node) = element
        .metadata
        .accessibility
        .as_ref()
        .filter(|node| node.role() == reactive_tui::accessibility::Role::Image)
    {
        *images += 1;
        *busy |= node.is_busy();
    }
    let polite = element
        .class
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|c| c == "aria-live-polite"));
    if let (true, reactive_tui::component::ElementType::Text(text)) =
        (polite, &element.element_type)
    {
        live.push(text.clone());
    }
    for child in &element.children {
        described(child, images, live, busy);
    }
}

/// The names of the process's threads, where the host tells them.
pub fn thread_names() -> BTreeSet<String> {
    std::fs::read_dir("/proc/self/task")
        .map(|tasks| {
            tasks
                .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("comm")).ok())
                .map(|name| name.trim().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The script of a run: asked at every input poll with the frames presented
/// so far and the pictures sent, it hands the App an event or nothing.
pub type Script = Box<dyn FnMut(usize, usize) -> Option<Event> + Send + Sync>;
/// When a run ends: asked after every frame with the frames presented, the
/// pictures sent and the time since the start.
pub type Stop = Box<dyn Fn(usize, usize, Duration) -> bool + Send + Sync>;

/// The default backend with each present timed and each frame's bytes kept.
pub struct Timed {
    inner: SuprTuiBackend,
    terminal: Terminal,
    frames: Arc<Mutex<Vec<Frame>>>,
    script: Script,
    /// When the last input poll returned, so the frame's work is measured
    /// from there.
    wait_returned: Option<Instant>,
    /// How many of the terminal's bytes earlier frames already took.
    read: usize,
    /// Whether to wait for the frame's bytes, pictures included, before
    /// taking them: the content tests do, the timing test does not, since
    /// the App never waits for a picture to be made ready (GFX-009).
    sync: bool,
    /// What the tree being rendered tells a screen reader.
    tree: (usize, Vec<String>, bool),
}

impl Backend for Timed {
    fn image_output(&self) -> Option<ImageOutputOptions> {
        self.inner.image_output()
    }
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
        let (mut images, mut live, mut busy) = (0, Vec::new(), false);
        described(element, &mut images, &mut live, &mut busy);
        self.tree = (images, live, busy);
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
        let began = Instant::now();
        self.inner.present()?;
        let waited = began.elapsed();
        // The App's work for the frame runs from the input wait returning
        // through the frame being presented (BAR-005), so it includes the
        // wait in present, which is also measured on its own (CHT-039).
        let work = self
            .wait_returned
            .map_or(Duration::ZERO, |returned| returned.elapsed());
        if self.sync {
            self.inner.sync()?;
        }
        let output = {
            let bytes = self.terminal.bytes.lock().unwrap();
            let from = self.read.min(bytes.len());
            self.read = bytes.len();
            bytes[from..].to_vec()
        };
        self.frames.lock().unwrap().push(Frame {
            output,
            work,
            waited,
            began,
            pictures: self.terminal.pictures.load(Ordering::SeqCst),
            threads: thread_names(),
            image_nodes: self.tree.0,
            live: self.tree.1.clone(),
            busy: self.tree.2,
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
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        panic!("wake-aware input required")
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        let presented = self.frames.lock().unwrap().len();
        let pictures = self.terminal.pictures.load(Ordering::SeqCst);
        if let Some(event) = (self.script)(presented, pictures) {
            self.wait_returned = Some(Instant::now());
            return Ok(Some(event));
        }
        wake.wait(Some(Duration::from_millis(1)));
        self.wait_returned = Some(Instant::now());
        Ok(None)
    }
}

/// A root that shows one element and asks for a frame every turn of the
/// loop until the stop rule says the run is over. The loop turns more
/// often than it presents, so the rule reads the frames presented.
pub struct Shown {
    element: Element,
    /// Another element shown from that many presented frames on, once the
    /// first picture was sent: a look leaves or arrives after it was shown,
    /// however long its first picture took.
    swap: Option<(usize, Element)>,
    frames: Arc<Mutex<Vec<Frame>>>,
    stop: Stop,
    pictures: Arc<AtomicUsize>,
    started: Instant,
}

impl RootComponent for Shown {
    fn render(&self) -> Element {
        let presented = self.frames.lock().unwrap().len();
        let pictures = self.pictures.load(Ordering::SeqCst);
        match &self.swap {
            Some((at, element)) if presented >= *at && pictures >= 1 => element.clone(),
            _ => self.element.clone(),
        }
    }
    fn update(&mut self) -> Result<RootUpdate> {
        let presented = self.frames.lock().unwrap().len();
        let pictures = self.pictures.load(Ordering::SeqCst);
        if (self.stop)(presented, pictures, self.started.elapsed()) {
            return Ok(RootUpdate::Exit);
        }
        Ok(RootUpdate::Redraw)
    }
}

/// What a run left behind.
pub struct Run {
    pub size: (u16, u16),
    pub frames: Vec<Frame>,
}

/// One run at a time in this process: the hover tests count pictures per
/// frame of a 150 ms motion, and twenty Apps drawing at once on the one
/// drawing thread would starve them when a gate runs the suite on every
/// test thread (BAR-010).
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Run `element` on a terminal of `size` cells that takes `images`, feeding
/// it `script`'s events, until `stop` says so.
pub fn run(
    element: Element,
    size: (u16, u16),
    images: ImageOutputOptions,
    script: Script,
    stop: Stop,
    sync: bool,
) -> Run {
    run_swapping(element, None, size, images, script, stop, sync)
}

/// As `run`, with `swap` naming another element the root shows from that
/// many presented frames on, once the first picture was sent, so a look can
/// leave or arrive mid-run after it was shown.
#[allow(clippy::too_many_arguments)]
pub fn run_swapping(
    element: Element,
    swap: Option<(usize, Element)>,
    size: (u16, u16),
    images: ImageOutputOptions,
    script: Script,
    stop: Stop,
    sync: bool,
) -> Run {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let terminal = Terminal::default();
    let frames = Arc::new(Mutex::new(Vec::new()));
    let inner = SuprTuiBackend::with_writer_and_images(size.0, size.1, terminal.clone(), images)
        .expect("a backend that writes to memory");
    App::builder()
        .backend(Timed {
            inner,
            terminal: terminal.clone(),
            frames: Arc::clone(&frames),
            script,
            wait_returned: None,
            read: 0,
            sync,
            tree: (0, Vec::new(), false),
        })
        .root(Shown {
            element,
            swap,
            frames: Arc::clone(&frames),
            stop,
            pictures: Arc::clone(&terminal.pictures),
            started: Instant::now(),
        })
        .build()
        .expect("an App")
        .run()
        .expect("the App runs to its end");
    let mut frames = frames.lock().unwrap();
    Run {
        size,
        frames: std::mem::take(&mut *frames),
    }
}

/// No events.
pub fn silent() -> Script {
    Box::new(|_, _| None)
}

/// Events at frames: each is handed to the App at the first poll after
/// that many frames were presented.
pub fn at_frames(mut events: Vec<(usize, Event)>) -> Script {
    events.reverse();
    Box::new(move |presented, _| {
        if events.last().is_some_and(|(frame, _)| presented >= *frame) {
            events.pop().map(|(_, event)| event)
        } else {
            None
        }
    })
}

/// The run ends `settle` frames after the terminal was sent its `count`th
/// picture, or at the guard.
pub fn after_pictures(count: usize, settle: usize) -> Stop {
    let reached = Mutex::new(None);
    Box::new(move |frame, pictures, elapsed| {
        let mut reached = reached.lock().unwrap();
        if pictures >= count && reached.is_none() {
            *reached = Some(frame);
        }
        reached.is_some_and(|at| frame >= at + settle) || elapsed >= GUARD
    })
}

/// The run ends after `frames` frames, or at the guard.
pub fn after_frames(frames: usize) -> Stop {
    Box::new(move |frame, _, elapsed| frame >= frames || elapsed >= GUARD)
}

pub fn hover(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent::new(MouseEventKind::Move, Position::cell(x, y)))
}

// ---------------------------------------------------------------------------
// What the bytes hold: Kitty pictures with their pixels, Sixel rasters, and
// the screen's cells after any frame.

/// The zero-based row and column of the last cursor move in `text`.
pub fn last_cursor_move(text: &str) -> Option<(usize, usize)> {
    text.rmatch_indices("\x1b[").find_map(|(at, _)| {
        let part = &text[at + 2..];
        let digits: String = part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ';')
            .collect();
        if !part[digits.len()..].starts_with('H') {
            return None;
        }
        let mut numbers = digits.split(';').map(|n| n.parse::<usize>().unwrap_or(1));
        let row = numbers.next().unwrap_or(1);
        let column = numbers.next().unwrap_or(1);
        Some((row.saturating_sub(1), column.saturating_sub(1)))
    })
}

/// A Kitty picture the backend sent, decoded.
#[derive(Clone)]
pub struct Picture {
    /// The frame it was sent in.
    pub frame: usize,
    /// Its pixels, `s` by `v`.
    pub size: (u32, u32),
    /// The cells its placement names (`c`, `r`).
    pub cells: Option<(u32, u32)>,
    /// The cell its placement starts at: the cursor's row and column.
    pub at: Option<(usize, usize)>,
    /// The image id its command names (`i`).
    pub id: Option<u32>,
    /// Sent through shared memory, so its pixels are not in the bytes.
    pub shared: bool,
    /// Straight RGBA, row by row; empty when shared.
    pub pixels: Vec<[u8; 4]>,
}

impl Picture {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels
            .get((y * self.size.0 + x) as usize)
            .copied()
            .unwrap_or([0; 4])
    }
    /// The picture as a frame, for comparison with a reference.
    pub fn frame(&self) -> GraphicsFrame {
        GraphicsFrame::from_rgba(self.size.0, self.size.1, self.pixels.clone())
            .expect("a picture whose pixels fill its size")
    }
    /// Where the picture has anything: alpha above zero.
    pub fn opaque(&self) -> Vec<bool> {
        self.pixels.iter().map(|p| p[3] > 0).collect()
    }
    /// The rectangle of cells the placement covers: row, column, rows, columns.
    pub fn rect(&self) -> (usize, usize, usize, usize) {
        let (row, column) = self.at.expect("a cursor move before the picture");
        let (columns, rows) = self.cells.expect("a placement that names its cells");
        (row, column, rows as usize, columns as usize)
    }
}

/// Every Kitty picture in `frames`, in the order they were sent.
pub fn kitty_pictures(frames: &[Frame]) -> Vec<Picture> {
    use base64::Engine;
    let mut pictures = Vec::new();
    // The picture being transmitted, and its base64 so far.
    let mut open: Option<(Picture, String)> = None;
    for (index, frame) in frames.iter().enumerate() {
        let text = String::from_utf8_lossy(&frame.output).into_owned();
        let mut from = 0;
        while let Some(found) = text[from..].find(KITTY) {
            let start = from + found;
            let command = &text[start + KITTY.len()..];
            let end = command.find("\x1b\\").unwrap_or(command.len());
            let command = &command[..end];
            let (control, payload) = command.split_once(';').unwrap_or((command, ""));
            let keys: Vec<(&str, &str)> = control
                .split(',')
                .filter_map(|pair| pair.split_once('='))
                .collect();
            let key = |name: &str| keys.iter().find(|(k, _)| *k == name).map(|(_, v)| *v);
            let number = |name: &str| key(name).and_then(|v| v.parse::<u32>().ok());
            match key("a") {
                Some("T") => {
                    open = Some((
                        Picture {
                            frame: index,
                            size: (number("s").unwrap_or(0), number("v").unwrap_or(0)),
                            cells: number("c").zip(number("r")),
                            at: last_cursor_move(&text[..start]),
                            id: number("i"),
                            shared: key("t") == Some("s"),
                            pixels: Vec::new(),
                        },
                        String::new(),
                    ));
                }
                Some(_) => {
                    from = start + KITTY.len() + end;
                    continue;
                }
                None => {}
            }
            if let Some((_, data)) = &mut open {
                data.push_str(payload);
            }
            if key("m") != Some("1") {
                if let Some((mut picture, data)) = open.take() {
                    if !picture.shared {
                        let bytes = base64::engine::general_purpose::STANDARD
                            .decode(data.as_bytes())
                            .unwrap_or_default();
                        picture.pixels = bytes
                            .chunks_exact(4)
                            .map(|p| [p[0], p[1], p[2], p[3]])
                            .collect();
                    }
                    pictures.push(picture);
                }
            }
            from = start + KITTY.len() + end;
        }
    }
    pictures
}

/// One Kitty graphics command the backend sent: its action, the image id it
/// names and every key, with the frame it was sent in and the cell the
/// cursor stood at.
#[derive(Clone, Debug)]
pub struct KittyCommand {
    pub frame: usize,
    pub action: String,
    pub id: Option<u32>,
    pub keys: Vec<(String, String)>,
    pub at: Option<(usize, usize)>,
    /// Whether the command carried any payload.
    pub payload: bool,
}

impl KittyCommand {
    pub fn key(&self, name: &str) -> Option<&str> {
        self.keys
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Every Kitty graphics command in `frames`, in the order they were sent;
/// a chunked transmission is one command per chunk.
pub fn kitty_commands(frames: &[Frame]) -> Vec<KittyCommand> {
    let mut commands = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        let text = String::from_utf8_lossy(&frame.output).into_owned();
        let mut from = 0;
        while let Some(found) = text[from..].find(KITTY) {
            let start = from + found;
            let command = &text[start + KITTY.len()..];
            let end = command.find("\x1b\\").unwrap_or(command.len());
            let command = &command[..end];
            let (control, payload) = command.split_once(';').unwrap_or((command, ""));
            let keys: Vec<(String, String)> = control
                .split(',')
                .filter_map(|pair| pair.split_once('='))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();
            let key = |name: &str| keys.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone());
            commands.push(KittyCommand {
                frame: index,
                action: key("a").unwrap_or_default(),
                id: key("i").and_then(|v| v.parse().ok()),
                keys: keys.clone(),
                at: last_cursor_move(&text[..start]),
                payload: !payload.is_empty(),
            });
            from = start + KITTY.len() + end;
        }
    }
    commands
}

/// A Sixel raster the backend wrote: its stated size, which of its pixels
/// it sets, and the cell it starts at.
pub struct Raster {
    /// The frame whose output carried it.
    pub frame: usize,
    pub size: (usize, usize),
    /// The color of every pixel the data sets, none where it sets nothing.
    pub colors: Vec<Option<[u8; 3]>>,
    pub at: Option<(usize, usize)>,
}

/// Every Sixel raster in `frames`, in order.
pub fn sixel_rasters(frames: &[Frame]) -> Vec<Raster> {
    let mut rasters = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        let text = String::from_utf8_lossy(&frame.output).into_owned();
        let mut from = 0;
        while let Some(found) = text[from..].find(SIXEL) {
            let start = from + found;
            let body = &text[start + SIXEL.len()..];
            let (size, colors) = super::sixel_colors(body);
            rasters.push(Raster {
                frame: index,
                size,
                colors,
                at: last_cursor_move(&text[..start]),
            });
            from = start + SIXEL.len();
        }
    }
    rasters
}

impl Run {
    pub fn pictures(&self) -> Vec<Picture> {
        kitty_pictures(&self.frames)
    }
    pub fn rasters(&self) -> Vec<Raster> {
        sixel_rasters(&self.frames)
    }
    /// The screen as a terminal shows it after frame `frame`.
    pub fn screen(&self, frame: usize) -> vt100::Screen {
        let mut parser = vt100::Parser::new(self.size.1, self.size.0, 0);
        for frame in &self.frames[..=frame.min(self.frames.len().saturating_sub(1))] {
            parser.process(&frame.output);
        }
        parser.screen().clone()
    }
    /// How long the run took.
    pub fn span(&self) -> Duration {
        self.frames
            .last()
            .zip(self.frames.first())
            .map_or(Duration::ZERO, |(last, first)| last.began - first.began)
    }
    /// The first picture, or a failure that says none came.
    pub fn first_picture(&self, what: &str) -> Picture {
        self.pictures().into_iter().next().unwrap_or_else(|| {
            panic!(
                "{what} on a host that takes Kitty graphics sent no picture in {} frames and {:.1} s",
                self.frames.len(),
                self.span().as_secs_f64()
            )
        })
    }
}

/// The text of the cell at `row`, `column`.
pub fn cell_text(screen: &vt100::Screen, row: usize, column: usize) -> String {
    screen
        .cell(row as u16, column as u16)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

/// Whether `text` is a glyph the mask canvas draws shapes with: braille, a
/// block element, or an ASCII shape glyph.
pub fn shape_glyph(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c) || ('\u{2580}'..='\u{259F}').contains(&c))
}

/// The cells inside `rect` whose text is not blank, with their text.
pub fn marked_cells(
    screen: &vt100::Screen,
    rect: (usize, usize, usize, usize),
    keep: impl Fn(&str) -> bool,
) -> Vec<(usize, usize, String)> {
    let (row, column, rows, columns) = rect;
    let mut cells = Vec::new();
    for r in row..row + rows {
        for c in column..column + columns {
            let text = cell_text(screen, r, c);
            if !text.trim().is_empty() && keep(&text) {
                cells.push((r, c, text));
            }
        }
    }
    cells
}

/// The active theme's color for `token`, as the picture draws it.
pub fn role(token: &str) -> [u8; 3] {
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color(token)
        .unwrap_or_else(|| panic!("the theme resolves {token}"));
    [r, g, b].map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Whether `pixel`'s color is within `tolerance` of `color` in every channel.
pub fn near(pixel: [u8; 4], color: [u8; 3], tolerance: u8) -> bool {
    (0..3).all(|i| pixel[i].abs_diff(color[i]) <= tolerance)
}
