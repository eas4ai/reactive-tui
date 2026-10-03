//! charts-pictures mechanism: CHT-037, CHT-038 and CHT-039 (docs/spec/charts.md),
//! and the pixel clauses of CHT-012, CHT-013, CHT-021, CHT-025 and CHT-027, which
//! charts-goldens and frame-budget run from here. Built with `wgpu-graphics`.
//!
//! A line, area, scatter, bar or candlestick chart on a terminal that takes
//! Kitty graphics or Sixel draws its plot area as one pixel picture. The tests
//! run an App on a backend that writes to memory, decode the Kitty pictures the
//! backend sends, and compare them with the reference pictures under
//! tests/snapshots/charts/pictures, which `REGENERATE=1` refreshes and nothing
//! else writes (BAR-004's rule).

mod canvas_support;

use reactive_tui::app::{App, AppWaker, RootComponent, RootUpdate};
use reactive_tui::backend::{Backend, FrameLayout, ImageOutputOptions, SuprTuiBackend};
use reactive_tui::builder::core::div;
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
use reactive_tui::graphics::{CanvasOutput, GraphicsFrame, GraphicsOptions};
use reactive_tui::widgets::display::{
    AreaChartBuilder, BarChartBuilder, BarGrowth, CandlestickChartBuilder, Chart, ChartAxis,
    ChartLegend, ChartProps, ChartType, DataPoint, DataSeries, FillStyle, LineChartBuilder,
    ScatterChartBuilder,
};
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The pixels of one cell, as the tests' terminals report them.
const CELL: (u16, u16) = (8, 16);
/// How a Kitty graphics command starts.
const KITTY: &str = "\x1b_G";
/// How the crate's Sixel pictures start: the device control string with a
/// transparent background.
const SIXEL: &str = "\x1bP0;1q";
/// How long a run may take before it ends on its own. A hang guard, not a
/// bound: a slow host fails a test on what it shows, not on how long it took.
const GUARD: Duration = Duration::from_secs(20);
/// One frame at 60 frames a second (CHT-039).
const BOUND: Duration = Duration::from_micros(16_600);

/// A host that takes Kitty graphics in the command, so the picture's pixels
/// are in the bytes.
fn kitty() -> ImageOutputOptions {
    ImageOutputOptions {
        kitty_graphics: true,
        cell_pixels: CELL,
        ..Default::default()
    }
}

/// A host that takes Sixel and nothing else.
fn sixel() -> ImageOutputOptions {
    ImageOutputOptions {
        sixel: true,
        cell_pixels: CELL,
        ..Default::default()
    }
}

/// A host that takes no pixels.
fn no_pixels() -> ImageOutputOptions {
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
struct Terminal {
    bytes: Arc<Mutex<Vec<u8>>>,
    pictures: Arc<AtomicUsize>,
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
struct Frame {
    output: Vec<u8>,
    work: Duration,
    waited: Duration,
    began: Instant,
    pictures: usize,
    threads: BTreeSet<String>,
    /// How many screen-reader nodes of the Image role the frame's tree held.
    image_nodes: usize,
    /// The texts of the frame's live regions.
    live: Vec<String>,
    /// Whether an Image node of the frame's tree read busy.
    busy: bool,
}

/// What the frame's element tree tells a screen reader: its Image nodes,
/// and the texts of its live regions.
fn described(element: &Element, images: &mut usize, live: &mut Vec<String>, busy: &mut bool) {
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
fn thread_names() -> BTreeSet<String> {
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
type Script = Box<dyn FnMut(usize, usize) -> Option<Event> + Send + Sync>;
/// When a run ends: asked after every frame with the frames presented, the
/// pictures sent and the time since the start.
type Stop = Box<dyn Fn(usize, usize, Duration) -> bool + Send + Sync>;

/// The default backend with each present timed and each frame's bytes kept.
struct Timed {
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
struct Shown {
    element: Element,
    frames: Arc<Mutex<Vec<Frame>>>,
    stop: Stop,
    pictures: Arc<AtomicUsize>,
    started: Instant,
}

impl RootComponent for Shown {
    fn render(&self) -> Element {
        self.element.clone()
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
struct Run {
    size: (u16, u16),
    frames: Vec<Frame>,
}

/// One run at a time in this process: the hover tests count pictures per
/// frame of a 150 ms motion, and twenty Apps drawing at once on the one
/// drawing thread would starve them when a gate runs the suite on every
/// test thread (BAR-010).
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Run `element` on a terminal of `size` cells that takes `images`, feeding
/// it `script`'s events, until `stop` says so.
fn run(
    element: Element,
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
fn silent() -> Script {
    Box::new(|_, _| None)
}

/// Events at frames: each is handed to the App at the first poll after
/// that many frames were presented.
fn at_frames(mut events: Vec<(usize, Event)>) -> Script {
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
fn after_pictures(count: usize, settle: usize) -> Stop {
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
fn after_frames(frames: usize) -> Stop {
    Box::new(move |frame, _, elapsed| frame >= frames || elapsed >= GUARD)
}

fn hover(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent::new(MouseEventKind::Move, Position::cell(x, y)))
}

// ---------------------------------------------------------------------------
// What the bytes hold: Kitty pictures with their pixels, Sixel rasters, and
// the screen's cells after any frame.

/// The zero-based row and column of the last cursor move in `text`.
fn last_cursor_move(text: &str) -> Option<(usize, usize)> {
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
struct Picture {
    /// The frame it was sent in.
    frame: usize,
    /// Its pixels, `s` by `v`.
    size: (u32, u32),
    /// The cells its placement names (`c`, `r`).
    cells: Option<(u32, u32)>,
    /// The cell its placement starts at: the cursor's row and column.
    at: Option<(usize, usize)>,
    /// Sent through shared memory, so its pixels are not in the bytes.
    shared: bool,
    /// Straight RGBA, row by row; empty when shared.
    pixels: Vec<[u8; 4]>,
}

impl Picture {
    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels
            .get((y * self.size.0 + x) as usize)
            .copied()
            .unwrap_or([0; 4])
    }
    /// The picture as a frame, for comparison with a reference.
    fn frame(&self) -> GraphicsFrame {
        GraphicsFrame::from_rgba(self.size.0, self.size.1, self.pixels.clone())
            .expect("a picture whose pixels fill its size")
    }
    /// Where the picture has anything: alpha above zero.
    fn opaque(&self) -> Vec<bool> {
        self.pixels.iter().map(|p| p[3] > 0).collect()
    }
    /// The rectangle of cells the placement covers: row, column, rows, columns.
    fn rect(&self) -> (usize, usize, usize, usize) {
        let (row, column) = self.at.expect("a cursor move before the picture");
        let (columns, rows) = self.cells.expect("a placement that names its cells");
        (row, column, rows as usize, columns as usize)
    }
}

/// Every Kitty picture in `frames`, in the order they were sent.
fn kitty_pictures(frames: &[Frame]) -> Vec<Picture> {
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

/// A Sixel raster the backend wrote: its stated size, which of its pixels
/// it sets, and the cell it starts at.
struct Raster {
    size: (usize, usize),
    /// The color of every pixel the data sets, none where it sets nothing.
    colors: Vec<Option<[u8; 3]>>,
    at: Option<(usize, usize)>,
}

/// Every Sixel raster in `frames`, in order.
fn sixel_rasters(frames: &[Frame]) -> Vec<Raster> {
    let mut rasters = Vec::new();
    for frame in frames {
        let text = String::from_utf8_lossy(&frame.output).into_owned();
        let mut from = 0;
        while let Some(found) = text[from..].find(SIXEL) {
            let start = from + found;
            let body = &text[start + SIXEL.len()..];
            let (size, colors) = canvas_support::sixel_colors(body);
            rasters.push(Raster {
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
    fn pictures(&self) -> Vec<Picture> {
        kitty_pictures(&self.frames)
    }
    fn rasters(&self) -> Vec<Raster> {
        sixel_rasters(&self.frames)
    }
    /// The screen as a terminal shows it after frame `frame`.
    fn screen(&self, frame: usize) -> vt100::Screen {
        let mut parser = vt100::Parser::new(self.size.1, self.size.0, 0);
        for frame in &self.frames[..=frame.min(self.frames.len().saturating_sub(1))] {
            parser.process(&frame.output);
        }
        parser.screen().clone()
    }
    /// How long the run took.
    fn span(&self) -> Duration {
        self.frames
            .last()
            .zip(self.frames.first())
            .map_or(Duration::ZERO, |(last, first)| last.began - first.began)
    }
    /// The first picture, or a failure that says none came.
    fn first_picture(&self, what: &str) -> Picture {
        self.pictures().into_iter().next().unwrap_or_else(|| {
            panic!(
                "CHT-037: {what} on a host that takes Kitty graphics sent no picture in {} frames and {:.1} s",
                self.frames.len(),
                self.span().as_secs_f64()
            )
        })
    }
}

/// The text of the cell at `row`, `column`.
fn cell_text(screen: &vt100::Screen, row: usize, column: usize) -> String {
    screen
        .cell(row as u16, column as u16)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

/// Whether `text` is a glyph the mask canvas draws shapes with: braille, a
/// block element, or an ASCII shape glyph.
fn shape_glyph(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c) || ('\u{2580}'..='\u{259F}').contains(&c))
}

/// The cells inside `rect` whose text is not blank, with their text.
fn marked_cells(
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
fn role(token: &str) -> [u8; 3] {
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color(token)
        .unwrap_or_else(|| panic!("the theme resolves {token}"));
    [r, g, b].map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Whether `pixel`'s color is within `tolerance` of `color` in every channel.
fn near(pixel: [u8; 4], color: [u8; 3], tolerance: u8) -> bool {
    (0..3).all(|i| pixel[i].abs_diff(color[i]) <= tolerance)
}

// ---------------------------------------------------------------------------
// The charts.

/// One row of the catalog's chart data.
struct Sample {
    label: &'static str,
    value: f64,
    open: f64,
    close: f64,
}

/// The catalog's Charts page data.
fn samples() -> Vec<Sample> {
    let labels = ["mon", "tue", "wed", "thu", "fri", "sat"];
    let values = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    labels
        .iter()
        .zip(values)
        .enumerate()
        .map(|(i, (label, value))| Sample {
            label,
            value,
            open: if i == 0 { value } else { values[i - 1] },
            close: value,
        })
        .collect()
}

fn series(name: &str, values: &[f64]) -> DataSeries {
    DataSeries::new(
        name,
        values
            .iter()
            .enumerate()
            .map(|(i, v)| DataPoint::with_label(*v, format!("p{i}")))
            .collect(),
    )
}

/// A chart of `kind` whose plot is its whole rectangle: no title, no axis
/// labels, no legend, no animation, values 0 to 10.
fn bare(kind: ChartType, size: (u16, u16), values: &[f64]) -> ChartProps {
    ChartProps {
        chart_type: kind,
        width: size.0,
        height: size.1,
        series: vec![series("series", values)],
        x_axis: ChartAxis {
            show_labels: false,
            show_grid: false,
            ..Default::default()
        },
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(10.0),
            show_labels: false,
            show_grid: false,
            ..Default::default()
        },
        legend: ChartLegend {
            visible: false,
            ..Default::default()
        },
        animated: false,
        ..Default::default()
    }
}

/// A chart of `kind` with its chrome: a title, axis labels, a grid and a
/// legend, so the plot is smaller than the chart.
fn dressed(kind: ChartType, size: (u16, u16), values: &[f64]) -> ChartProps {
    let mut props = bare(kind, size, values);
    props.title = Some("Sales".into());
    props.x_axis.show_labels = true;
    props.y_axis.show_labels = true;
    props.y_axis.show_grid = true;
    props.legend.visible = true;
    props
}

fn chart(props: ChartProps) -> Element {
    Element::typed::<Chart>(props)
}

const VALUES: [f64; 6] = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];

// ---------------------------------------------------------------------------
// CHT-037: the plot as a picture.

#[test]
fn cht_037_a_line_chart_on_a_kitty_host_sends_its_plot_as_a_picture() {
    let run = run(
        chart(dressed(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line chart of 80 by 24 cells");
    let (columns, rows) = picture
        .cells
        .expect("CHT-037: the picture's placement names the cells it covers");
    assert_eq!(
        picture.size,
        (columns * u32::from(CELL.0), rows * u32::from(CELL.1)),
        "CHT-037: the picture is the plot's cells times the cell size"
    );
    let (row, column) = picture
        .at
        .expect("CHT-037: the picture is placed at the plot, with a cursor move before it");
    assert!(
        row >= 1 && column >= 1 && row + rows as usize <= 23 && column + columns as usize <= 80,
        "CHT-037: the picture covers the plot inside the title row, the label column and the x-label row, found rows {row}..{} and columns {column}..{}",
        row + rows as usize,
        column + columns as usize
    );
    // The chrome stays cell text around it.
    let screen = run.screen(picture.frame);
    let text = screen.contents();
    assert!(
        text.contains("Sales") && text.contains("10") && text.contains("series"),
        "CHT-037: the title, a tick label and the legend stay cell text:\n{text}"
    );
}

#[test]
fn cht_037_the_plot_is_blank_until_its_first_picture_and_never_braille() {
    let run = run(
        chart(dressed(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line chart of 80 by 24 cells");
    let rect = picture.rect();
    for frame in 0..=picture.frame {
        let shapes = marked_cells(&run.screen(frame), rect, shape_glyph);
        assert!(
            shapes.is_empty(),
            "CHT-037: frame {frame}, before or with the first picture, holds shape glyphs inside the plot: {:?}",
            &shapes[..shapes.len().min(6)]
        );
    }
}

#[test]
fn cht_037_the_text_stays_in_cells_and_reads_over_the_picture() {
    // Value labels sit inside the plot, above the bars (CHT-013): they stay
    // cell text and read over the picture.
    let values = [2.5, 8.5, 5.5, 9.5, 3.5, 7.5];
    let mut props = dressed(ChartType::BarVertical, (80, 24), &values);
    props.value_labels = Some(true);
    let run = run(
        chart(props),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a bar chart with value labels");
    let screen = run.screen(run.frames.len() - 1);
    let inside = marked_cells(&screen, picture.rect(), |_| true);
    let labels: String = inside.iter().map(|(_, _, text)| text.as_str()).collect();
    assert!(
        labels.contains("9.5") && labels.contains("2.5"),
        "CHT-037: the value labels inside the plot read in the cells over the picture, found {labels:?}"
    );
    assert!(
        screen.contents().contains("10"),
        "CHT-037: the tick labels stay cell text"
    );
}

#[test]
fn cht_037_the_picture_is_drawn_at_the_terminals_cell_size() {
    let host = ImageOutputOptions {
        cell_pixels: (9, 18),
        ..kitty()
    };
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        host,
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line chart on a terminal of 9 by 18 pixel cells");
    let (columns, rows) = picture.cells.expect("a placement that names its cells");
    assert_eq!(
        (picture.size, (columns, rows)),
        ((720, 432), (80, 24)),
        "CHT-037: a bare 80 by 24 chart on 9 by 18 pixel cells is a 720 by 432 picture placed over 80 by 24 cells"
    );
}

#[test]
fn cht_037_on_sixel_the_raster_is_the_plots_cells_at_the_cells_pixels() {
    let props = bare(ChartType::BarVertical, (80, 24), &VALUES);
    let on_kitty = run(
        chart(props.clone()),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = on_kitty.first_picture("a bar chart");
    let on_sixel = run(
        chart(props),
        (80, 24),
        sixel(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let rasters = on_sixel.rasters();
    let raster = rasters.first().unwrap_or_else(|| {
        panic!(
            "CHT-037: a bar chart on a host that takes Sixel wrote no raster in {} frames",
            on_sixel.frames.len()
        )
    });
    assert_eq!(
        raster.size,
        (640, 384),
        "CHT-037: the Sixel raster is the plot's 80 by 24 cells at 8 by 16 pixels"
    );
    assert_eq!(raster.at, Some((0, 0)), "the raster starts at the plot");
    // Sixel cannot leave a pixel to the screen, so the plot's background
    // is painted in the cells' background color: a pixel in that color
    // is unpainted, any other is painted.
    let opaque = picture.opaque();
    let background = raster
        .colors
        .iter()
        .zip(&opaque)
        .find(|(_, opaque)| !**opaque)
        .and_then(|(color, _)| *color);
    let disagree = raster
        .colors
        .iter()
        .zip(&opaque)
        .filter(|(color, opaque)| (color.is_some() && **color != background) != **opaque)
        .count();
    let total = (640 * 384) as f64;
    assert!(
        (disagree as f64) / total <= 0.01,
        "CHT-037: the Sixel raster's painted pixels and the Kitty picture's opaque pixels disagree on {:.2} percent of the plot",
        100.0 * disagree as f64 / total
    );
}

#[test]
fn cht_037_without_pixels_the_cells_are_the_fallbacks() {
    // The golden the feature-off build records for this chart on the debug
    // backend (tests/charts_goldens.rs, line_medium): without pixels the
    // same cells are what a terminal gets.
    let golden = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/charts/line_medium.ansi");
    let golden = std::fs::read_to_string(&golden)
        .unwrap_or_else(|error| panic!("the golden {}: {error}", golden.display()));
    let expected: String = golden
        .lines()
        .take_while(|line| !line.starts_with("colors:"))
        .collect::<Vec<_>>()
        .join("\n");
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        no_pixels(),
        silent(),
        after_frames(6),
        true,
    );
    assert!(
        run.pictures().is_empty() && run.rasters().is_empty(),
        "a host without pixels is sent none"
    );
    let text = run.screen(run.frames.len() - 1).contents();
    assert_eq!(
        text.trim_end(),
        expected.trim_end(),
        "CHT-037: without pixels the chart's cells are the fallback's golden"
    );
}

#[test]
fn cht_037_a_pie_chart_sends_no_picture() {
    let run = run(
        chart(bare(ChartType::Pie, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_frames(8),
        true,
    );
    assert!(
        run.pictures().is_empty(),
        "CHT-037: a pie chart draws in cells and sends no picture"
    );
    let text = run.screen(run.frames.len() - 1).contents();
    assert!(
        text.chars().any(|c| ('\u{2580}'..='\u{259F}').contains(&c)),
        "the pie is drawn in cells:\n{text}"
    );
}

#[test]
fn cht_037_the_chart_is_one_screen_reader_node_and_its_picture_says_nothing() {
    let run = run(
        chart(dressed(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line chart");
    let frame = &run.frames[picture.frame];
    assert_eq!(
        frame.image_nodes, 1,
        "CHT-037: the chart is one Image node for the screen reader; its picture adds none"
    );
    assert!(
        !frame.live.iter().any(|text| text.starts_with("Canvas:")),
        "CHT-037: the picture's canvas announces nothing of its own, found {:?}",
        frame.live
    );
}

#[test]
fn cht_036_the_chart_reads_busy_until_its_first_picture_is_shown() {
    let run = run(
        chart(dressed(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 3),
        true,
    );
    let picture = run.first_picture("a line chart");
    // Every frame from the chart's first layout up to and including the one
    // that carries the first picture reads busy: the chart's node is laid
    // out before its canvas reports the picture shown. Frame 0 is drawn
    // before the chart has a size, so nothing is awaited in it. The frame
    // after the picture's does not read busy.
    let not_busy: Vec<usize> = (1..=picture.frame)
        .filter(|&frame| !run.frames[frame].busy)
        .collect();
    assert!(
        not_busy.is_empty(),
        "CHT-036: frames {not_busy:?} before the first picture (frame {}) did not read busy",
        picture.frame
    );
    let after = picture.frame + 1;
    assert!(
        run.frames.get(after).is_some_and(|frame| !frame.busy),
        "CHT-036: the frame after the first picture still reads busy ({} frames)",
        run.frames.len()
    );
}

/// The most common opaque color of a picture: a line chart's stroke.
fn dominant(picture: &Picture) -> [u8; 3] {
    let mut counts: std::collections::HashMap<[u8; 3], usize> = std::collections::HashMap::new();
    for pixel in &picture.pixels {
        if pixel[3] > 200 {
            *counts.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map_or([0; 3], |(color, _)| color)
}

/// A line chart whose theme switches to light after its first picture:
/// no later frame carries a picture in the old theme's colors. Run in a
/// process of its own, since the theme is the process's.
#[test]
#[ignore = "run by thm_003_after_a_theme_change_no_frame_shows_the_old_theme_picture"]
fn theme_change_child() {
    use reactive_tui::theme::{dark_theme, light_theme, Theme};
    Theme::set_active(dark_theme());
    let old = role("chart-1");
    let switched = Arc::new(Mutex::new(None::<usize>));
    let flag = switched.clone();
    let script: Script = Box::new(move |presented, pictures| {
        let mut at = flag.lock().unwrap();
        if at.is_none() && pictures >= 1 && presented >= 4 {
            Theme::set_active(light_theme());
            *at = Some(presented);
        }
        None
    });
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        script,
        after_pictures(2, 8),
        true,
    );
    let at = switched
        .lock()
        .unwrap()
        .expect("the theme switched during the run");
    let new = role("chart-1");
    assert!(
        !near([old[0], old[1], old[2], 255], new, 12),
        "the two themes differ in chart-1"
    );
    let pictures = run.pictures();
    let after: Vec<&Picture> = pictures.iter().filter(|p| p.frame >= at).collect();
    let old_colored: Vec<usize> = after
        .iter()
        .filter(|p| {
            let c = dominant(p);
            near([c[0], c[1], c[2], 255], old, 12)
        })
        .map(|p| p.frame)
        .collect();
    assert!(
        old_colored.is_empty(),
        "THM-003: frames {old_colored:?} after the theme change at frame {at} carried a picture in the old theme's colors"
    );
    assert!(
        after.iter().any(|p| {
            let c = dominant(p);
            near([c[0], c[1], c[2], 255], new, 12)
        }),
        "THM-003: a picture in the new theme's colors followed the change ({} pictures after frame {at})",
        after.len()
    );
    Theme::set_active(dark_theme());
}

#[test]
fn thm_003_after_a_theme_change_no_frame_shows_the_old_theme_picture() {
    let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", "theme_change_child", "--ignored", "--nocapture"])
        .output()
        .expect("the test binary runs");
    let said = String::from_utf8_lossy(&child.stdout).into_owned()
        + &String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success() && said.contains("1 passed"),
        "THM-003: {}",
        &said[said.find("THM-003").unwrap_or(0)..]
    );
}

/// A line chart on a Kitty host with the plots switched back to cells, by the
/// environment the parent set or by the application's graphics options. Run
/// in a process of its own, since both choices are made once per process.
#[test]
#[ignore = "run by cht_037_the_environment_and_the_options_switch_the_plots_back_to_cells"]
fn plots_in_cells_child() {
    if std::env::var("CHART_PICTURES_SWITCH").as_deref() == Ok("options") {
        reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions {
            output: Some(CanvasOutput::Blocks),
            ..Default::default()
        });
    }
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_frames(8),
        true,
    );
    assert!(
        run.pictures().is_empty(),
        "CHT-037: with the plots switched back to cells a Kitty host is sent no picture"
    );
    let text = run.screen(run.frames.len() - 1).contents();
    assert!(
        text.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "the plot is drawn in cells:\n{text}"
    );
}

#[test]
fn cht_037_the_environment_and_the_options_switch_the_plots_back_to_cells() {
    for (name, value) in [
        ("REACTIVE_TUI_CANVAS", "blocks"),
        ("CHART_PICTURES_SWITCH", "options"),
    ] {
        let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
            .args([
                "--exact",
                "plots_in_cells_child",
                "--ignored",
                "--nocapture",
            ])
            .env(name, value)
            .output()
            .expect("the test binary runs");
        let said = String::from_utf8_lossy(&child.stdout).into_owned()
            + &String::from_utf8_lossy(&child.stderr);
        assert!(
            child.status.success() && said.contains("1 passed"),
            "CHT-037 with {name}={value}: {}",
            &said[said.find("CHT-037").unwrap_or(0)..]
        );
    }
}

// ---------------------------------------------------------------------------
// Reference pictures (CHT-012, CHT-013, CHT-037): a chart's first picture on
// a Kitty host against tests/snapshots/charts/pictures/<name>.png, within
// GFX-002's tolerance.

/// `tests/snapshots/charts/pictures`, or `charts/pictures` under
/// `REACTIVE_TUI_SNAPSHOTS`: the charts-pictures check points that at a copy
/// with one reference altered, and this test must then fail.
fn reference_path(name: &str) -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("charts/pictures")
        .join(format!("{name}.png"))
}

/// The process's graphics options while a reference picture is drawn: the
/// software renderer, which the references are drawn by on every host
/// (CHT-012, CHT-013, CHT-037). Held across the run, so another test's App
/// that starts meanwhile draws on it too, consistently, and never sees the
/// options change under it.
static SOFTWARE: Mutex<()> = Mutex::new(());

/// The difference between `element`'s first picture and its reference, or
/// none; with REGENERATE=1 the reference is written instead (BAR-004).
fn reference_problem(name: &str, element: Element) -> Option<String> {
    let _software = SOFTWARE.lock().unwrap_or_else(|e| e.into_inner());
    reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions {
        force_cpu: true,
        ..Default::default()
    });
    let run = run(
        element,
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions::default());
    let Some(picture) = run.pictures().into_iter().next() else {
        return Some(format!("{name}: no picture was sent"));
    };
    assert_eq!(
        picture.pixels.len(),
        (picture.size.0 * picture.size.1) as usize,
        "{name}: the picture's pixels fill its size"
    );
    let frame = picture.frame();
    let path = reference_path(name);
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        let flat: Vec<u8> = frame.pixels().iter().flatten().copied().collect();
        let image = image::RgbaImage::from_raw(frame.width(), frame.height(), flat)
            .expect("a picture of its size");
        let mut bytes = Vec::new();
        image
            .write_to(&mut io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("a PNG of the picture");
        std::fs::create_dir_all(path.parent().unwrap()).expect("the pictures directory");
        std::fs::write(&path, &bytes).expect("the reference picture written");
        return None;
    }
    let reference = match image::open(&path) {
        Ok(reference) => reference.to_rgba8(),
        Err(error) => {
            return Some(format!(
                "{name}: no reference picture {}: {error}",
                path.display()
            ))
        }
    };
    let pixels = reference.pixels().map(|pixel| pixel.0).collect();
    let reference =
        GraphicsFrame::from_rgba(reference.width(), reference.height(), pixels).unwrap();
    canvas_support::difference(&frame, &reference).map(|why| format!("{name}: {why}"))
}

/// Every variant's problem, as one failure.
fn assert_references(requirement: &str, variants: Vec<(&str, Element)>) {
    let problems: Vec<String> = variants
        .into_iter()
        .filter_map(|(name, element)| reference_problem(name, element))
        .collect();
    assert!(
        problems.is_empty(),
        "{requirement}: a chart's picture differs from its reference, or none came: {problems:?}"
    );
}

/// The alpha of the picture at (`x`, `y`).
fn alpha_at(picture: &Picture, x: f64, y: f64) -> u8 {
    picture.pixel(x as u32, y as u32)[3]
}

/// A gradient case: the values, the value axis's limits, and probes as
/// (column, stroke y, baseline y) in fractions of the picture.
type GradientCase = ([f64; 6], f64, f64, Vec<(f64, f64, f64)>);

#[test]
fn cht_012_a_gradient_area_below_or_across_its_baseline_fades_toward_it() {
    // A bare 80 by 24 area is a 640 by 384 picture with its six points at
    // x = 0, 128, 256, 384, 512, 640. Negative values put the area below its
    // baseline, mixed values across it: on each side the fill is strongest
    // at the stroke and transparent at the baseline.
    let cases: [GradientCase; 2] = [
        // (column, stroke y, baseline y) per probe, in fractions of the picture
        (
            [-2.0, -8.0, -5.0, -9.0, -3.0, -7.0],
            -10.0,
            0.0,
            vec![(0.6, 0.9, 0.0)],
        ),
        (
            [-8.0, 8.0, -8.0, 8.0, -8.0, 8.0],
            -10.0,
            10.0,
            vec![(0.2, 0.1, 0.5), (0.4, 0.9, 0.5)],
        ),
    ];
    for (values, min, max, probes) in cases {
        let mut props = bare(ChartType::Area, (80, 24), &values);
        props.series[0].fill_style = FillStyle::Gradient;
        props.y_axis.min = Some(min);
        props.y_axis.max = Some(max);
        let picture = run(
            chart(props),
            (80, 24),
            kitty(),
            silent(),
            after_pictures(1, 2),
            true,
        )
        .first_picture("a gradient area");
        let (w, h) = (f64::from(picture.size.0), f64::from(picture.size.1));
        for (column, stroke, baseline) in probes {
            let x = column * w;
            // Four pixels inside the area from the stroke, and three from the baseline.
            let toward = if stroke > baseline { -4.0 } else { 4.0 };
            let near_stroke = alpha_at(&picture, x, stroke * h + toward);
            let near_base = alpha_at(&picture, x, baseline * h - toward * 0.75);
            assert!(
                near_stroke > 60 && near_base < 25 && near_stroke > 3 * near_base,
                "CHT-012: with values {values:?} the fill at column {column} is strongest at the stroke (alpha {near_stroke}) and fades to transparent at the baseline (alpha {near_base})"
            );
        }
    }
}

#[test]
fn cht_014_narrow_candle_bodies_keep_their_band_ratio_in_a_picture() {
    // Forty candles in a 40-cell plot: a band is eight pixels and the
    // default body is 0.8 of it, so every body is at most seven pixels wide
    // and no two touch. Every body spans 4 to 6, so one row crosses them all.
    let samples: Vec<Sample> = (0..40)
        .map(|_| Sample {
            label: "c",
            value: 6.0,
            open: 4.0,
            close: 6.0,
        })
        .collect();
    let mut props = CandlestickChartBuilder::new(samples)
        .x(|s| s.label)
        .open(|s| s.open)
        .close(|s| s.close)
        .high(|_| 7.0)
        .low(|_| 3.0)
        .size(40, 12)
        .build();
    props.x_axis.show_labels = false;
    props.y_axis.show_labels = false;
    props.legend.visible = false;
    let picture = run(
        chart(props),
        (40, 12),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    )
    .first_picture("forty candles");
    let runs_of = |y: u32| -> Vec<(u32, u32)> {
        let mut runs = Vec::new();
        let mut start = None;
        for x in 0..=picture.size.0 {
            let on = x < picture.size.0 && picture.pixel(x, y)[3] > 128;
            match (on, start) {
                (true, None) => start = Some(x),
                (false, Some(s)) => {
                    runs.push((s, x));
                    start = None;
                }
                _ => {}
            }
        }
        runs
    };
    let (row, runs) = (0..picture.size.1)
        .map(|y| (y, runs_of(y)))
        .max_by_key(|(_, runs)| runs.len())
        .expect("rows");
    let band = f64::from(picture.size.0) / 40.0;
    let widest = runs.iter().map(|(a, b)| b - a).max().unwrap_or(0);
    assert!(
        runs.len() == 40 && f64::from(widest) <= band * 0.8 + 1.0,
        "CHT-014: forty bodies of at most {:.1} pixels in a {} pixel wide picture; row {row} has {} runs, the widest {widest} pixels: {:?}",
        band * 0.8,
        picture.size.0,
        runs.len(),
        &runs[..runs.len().min(8)]
    );
}

#[test]
fn cht_037_a_candlestick_matches_its_reference_picture() {
    let candlestick = CandlestickChartBuilder::new(samples())
        .x(|s| s.label)
        .open(|s| s.open)
        .close(|s| s.close)
        .high(|s| s.open.max(s.close) + 1.0)
        .low(|s| s.open.min(s.close) - 1.0)
        .size(80, 24)
        .render();
    assert_references("CHT-037", vec![("candlestick", candlestick)]);
}

#[test]
fn cht_012_line_area_and_scatter_pictures_match_their_references() {
    let line = LineChartBuilder::new(samples())
        .x(|s| s.label)
        .y(|s| s.value)
        .name("value")
        .y(|s| s.open)
        .name("open")
        .natural()
        .size(80, 24)
        .render();
    // Two series overlaid, not stacked: the second's steps cross the first's.
    let area = AreaChartBuilder::new(samples())
        .x(|s| s.label)
        .y(|s| s.value)
        .name("value")
        .fill("chart-2")
        .y(|s| s.open)
        .name("open")
        .step_after()
        .size(80, 24)
        .render();
    let stacked = AreaChartBuilder::new(samples())
        .x(|s| s.label)
        .y(|s| s.value)
        .name("value")
        .y(|s| s.open)
        .name("open")
        .stacked(true)
        .size(80, 24)
        .render();
    let mut gradient = dressed(ChartType::Area, (80, 24), &VALUES);
    gradient.series[0].fill_style = FillStyle::Gradient;
    let mut pattern = dressed(ChartType::Area, (80, 24), &VALUES);
    pattern.series[0].fill_style = FillStyle::Pattern("diagonal".into());
    // Two series at distinct points: open differs from value after the
    // first sample, so neither series hides the other.
    let scatter = ScatterChartBuilder::new(samples())
        .x(|s| s.open)
        .y(|s| s.value)
        .name("value by open")
        .y(|s| s.open)
        .name("open by open")
        .size(80, 24)
        .render();
    assert_references(
        "CHT-012",
        vec![
            ("line_two_series", line),
            ("area_overlaid", area),
            ("area_stacked", stacked),
            ("area_gradient", chart(gradient)),
            ("area_pattern", chart(pattern)),
            ("scatter_two_series", scatter),
        ],
    );
}

#[test]
fn cht_013_bar_pictures_match_their_references() {
    let vertical = BarChartBuilder::new(samples())
        .band(|s| s.label)
        .value(|s| s.value)
        .name("value")
        .fill("chart-3")
        .size(80, 24)
        .render();
    let horizontal = BarChartBuilder::new(samples())
        .band(|s| s.label)
        .value(|s| s.value)
        .name("value")
        .alignment(BarGrowth::Left)
        .size(80, 24)
        .render();
    let grouped = BarChartBuilder::new(samples())
        .band(|s| s.label)
        .value(|s| s.value)
        .name("value")
        .value(|s| s.open)
        .name("open")
        .size(80, 24)
        .render();
    let stacked = BarChartBuilder::new(samples())
        .band(|s| s.label)
        .value(|s| s.value)
        .name("value")
        .value(|s| s.open)
        .name("open")
        .stacked(true)
        .size(80, 24)
        .render();
    let gradient_rounded = BarChartBuilder::new(samples())
        .band(|s| s.label)
        .value(|s| s.value)
        .name("value")
        .fill_gradient(|_, _, _| vec![(0.0, "chart-1"), (1.0, "chart-2")])
        .corner_radius(0.5)
        .size(80, 24)
        .render();
    assert_references(
        "CHT-013",
        vec![
            ("bar_vertical", vertical),
            ("bar_horizontal", horizontal),
            ("bar_grouped", grouped),
            ("bar_stacked", stacked),
            ("bar_gradient_rounded", gradient_rounded),
        ],
    );
}

#[test]
fn cht_013_a_corner_radius_rounds_the_bars() {
    // One bar of 8 out of 10 in a bare 40 by 12 chart, square and with
    // corners of one cell: the rounded bar's top left pixel is empty while
    // the top edge a cell in is still there.
    let mut square = bare(ChartType::BarVertical, (40, 12), &[8.0]);
    square.max_band_width = Some(20);
    let mut rounded = square.clone();
    rounded.corner_radius = 1.0;
    let square = run(
        chart(square),
        (40, 12),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    )
    .first_picture("a square bar");
    let rounded = run(
        chart(rounded),
        (40, 12),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    )
    .first_picture("a rounded bar");
    let bars = bar_columns(&square);
    let (left, right) = *bars.first().expect("one bar");
    let top = (0..square.size.1)
        .find(|&y| square.pixel((left + right) / 2, y)[3] > 128)
        .expect("the bar's top row");
    let corner = (square.pixel(left, top)[3], rounded.pixel(left, top)[3]);
    let edge = (
        square.pixel(left + 16, top)[3],
        rounded.pixel(left + 16, top)[3],
    );
    assert!(
        corner.0 > 200 && corner.1 < 100 && edge.0 > 200 && edge.1 > 200,
        "CHT-013: a corner radius of one cell empties the bar's corner pixel and keeps its edge: corner alpha square {} rounded {}, edge alpha square {} rounded {}",
        corner.0,
        corner.1,
        edge.0,
        edge.1
    );
}

#[test]
fn cht_013_a_bar_tip_ends_at_its_exact_pixel() {
    // One bar of 3, 3.5 and 4 out of 10 in a bare 40 by 12 chart: a 320 by
    // 192 picture whose bar top is at its exact pixel row, so 3.5 ends
    // between 3 and 4.
    let mut tops = Vec::new();
    for value in [3.0, 3.5, 4.0] {
        let run = run(
            chart(bare(ChartType::BarVertical, (40, 12), &[value])),
            (40, 12),
            kitty(),
            silent(),
            after_pictures(1, 2),
            true,
        );
        let picture = run.first_picture(&format!("a bar of {value}"));
        let column = picture.size.0 / 2;
        let top = (0..picture.size.1)
            .find(|&y| picture.pixel(column, y)[3] > 128)
            .unwrap_or_else(|| panic!("the bar of {value} has pixels in its middle column"));
        tops.push(top);
    }
    assert!(
        tops[0] > tops[1] && tops[1] > tops[2],
        "CHT-013: the bar tops of 3, 3.5 and 4 out of 10 are at distinct pixel rows, found {tops:?} (rows from the top)"
    );
}

// ---------------------------------------------------------------------------
// CHT-021, CHT-025, CHT-027: the picture's thread, the mask inside the plot,
// thinning.

#[test]
fn cht_021_the_plot_picture_is_drawn_on_the_drawing_thread() {
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line chart");
    let threads = &run.frames[picture.frame].threads;
    if cfg!(target_os = "linux") {
        let named = |prefix: &str| threads.iter().any(|name| name.starts_with(prefix));
        assert!(
            named("rtui-canvas") && named("rtui-chart"),
            "CHT-021: the plot picture is drawn on the canvas's drawing thread and the cells on the chart worker; threads alive with the picture: {threads:?}"
        );
    } else {
        println!("SKIP: thread names are read from /proc, which only Linux has");
    }
}

#[test]
fn cht_025_a_plot_drawn_as_a_picture_leaves_its_cells_blank() {
    let run = run(
        chart(dressed(ChartType::Area, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("an area chart");
    let marked = marked_cells(&run.screen(run.frames.len() - 1), picture.rect(), |_| true);
    assert!(
        marked.is_empty(),
        "CHT-025: the mask canvas writes nothing inside a plot that is a picture, found {:?}",
        &marked[..marked.len().min(8)]
    );
}

#[test]
fn cht_027_a_ten_thousand_point_line_is_drawn_as_a_picture() {
    let values: Vec<f64> = (0..10_000)
        .map(|i| 5.0 + 4.0 * ((i as f64) * 0.013).sin())
        .collect();
    let run = run(
        chart(bare(ChartType::Line, (40, 12), &values)),
        (40, 12),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let picture = run.first_picture("a line of 10,000 points");
    assert_eq!(
        picture.size,
        (320, 192),
        "CHT-027: the 10,000-point line's plot is a 320 by 192 picture, thinned per pixel column"
    );
}

// ---------------------------------------------------------------------------
// CHT-038: the hover in the picture.

/// The columns of the bars in a picture: runs of opaque pixels along its
/// bottom row, as (first, last) pixel columns.
fn bar_columns(picture: &Picture) -> Vec<(u32, u32)> {
    let bottom = picture.size.1 - 1;
    let mut bars = Vec::new();
    let mut start = None;
    for x in 0..=picture.size.0 {
        let on = x < picture.size.0 && picture.pixel(x, bottom)[3] > 200;
        match (on, start) {
            (true, None) => start = Some(x),
            (false, Some(from)) => {
                bars.push((from, x - 1));
                start = None;
            }
            _ => {}
        }
    }
    bars
}

/// The center of the hover band in a picture: the mean column of the pixels
/// in its top rows, where no bar reaches.
fn band_center(picture: &Picture) -> Option<f64> {
    let (mut sum, mut count) = (0.0, 0usize);
    for y in 0..8 {
        for x in 0..picture.size.0 {
            if picture.pixel(x, y)[3] > 0 {
                sum += x as f64;
                count += 1;
            }
        }
    }
    (count > 0).then(|| sum / count as f64)
}

#[test]
fn cht_038_a_hovered_line_chart_draws_the_crosshair_and_dots_in_the_picture() {
    // A bare 80 by 24 line: its six points sit at columns 0, 16, 32, 48, 64
    // and 80 of 80, so column 32 hovers index 2, whose value 5 is halfway up.
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        at_frames(vec![(6, hover(32, 12))]),
        after_pictures(2, 12),
        true,
    );
    let pictures = run.pictures();
    let before = pictures
        .first()
        .unwrap_or_else(|| panic!("CHT-038: the line chart sent no picture"));
    let after = pictures
        .last()
        .filter(|p| p.frame > before.frame)
        .unwrap_or_else(|| {
            panic!(
            "CHT-038: a hover change sends a new picture; {} pictures came, none after the hover",
            pictures.len()
        )
        });
    // The crosshair: a column whose text-muted pixels grew by most of the
    // plot's height.
    let muted = role("text-muted");
    let column_count = |p: &Picture, x: u32| {
        (0..p.size.1)
            .filter(|&y| {
                let pixel = p.pixel(x, y);
                pixel[3] > 100 && near(pixel, muted, 40)
            })
            .count()
    };
    let crosshair = (0..after.size.0).find(|&x| {
        column_count(after, x) >= column_count(before, x) + (after.size.1 as usize * 6 / 10)
    });
    assert!(
        crosshair.is_some(),
        "CHT-038: a hovered line chart's picture has a one-pixel crosshair in text-muted at the hovered index"
    );
    // The dot's halo: translucent pixels around the point of index 2 that
    // the picture before the hover did not have.
    let (cx, cy) = (after.size.0 as i64 * 2 / 5, after.size.1 as i64 / 2);
    let halo = |p: &Picture| {
        let mut count = 0;
        for y in (cy - 14).max(0)..(cy + 14).min(p.size.1 as i64) {
            for x in (cx - 14).max(0)..(cx + 14).min(p.size.0 as i64) {
                let alpha = p.pixel(x as u32, y as u32)[3];
                if (20..=140).contains(&alpha) {
                    count += 1;
                }
            }
        }
        count
    };
    assert!(
        halo(after) >= halo(before) + 40,
        "CHT-038: the hovered point has a dot inside a translucent halo; translucent pixels near the point went from {} to {}",
        halo(before),
        halo(after)
    );
    // Nothing of the hover is patched into the cells, apart from the
    // tooltip, which stays cell text in its rounded box (CHT-037).
    let screen = run.screen(run.frames.len() - 1);
    let whole = (0, 0, run.size.1 as usize, run.size.0 as usize);
    let boxes: Vec<((usize, usize), (usize, usize))> =
        marked_cells(&screen, whole, |text| text == "╭")
            .into_iter()
            .zip(marked_cells(&screen, whole, |text| text == "╯"))
            .map(|((r0, c0, _), (r1, c1, _))| ((r0, c0), (r1, c1)))
            .collect();
    assert!(
        !boxes.is_empty(),
        "CHT-037: the hovered chart shows its tooltip in a box of cells"
    );
    let in_box = |row: usize, column: usize| {
        boxes
            .iter()
            .any(|((r0, c0), (r1, c1))| (*r0..=*r1).contains(&row) && (*c0..=*c1).contains(&column))
    };
    let marked: Vec<(usize, usize, String)> = marked_cells(&screen, after.rect(), |text| {
        ["│", "─", "◌", "·", "|", "-"].contains(&text)
    })
    .into_iter()
    .filter(|(row, column, _)| !in_box(*row, *column))
    .collect();
    assert!(
        marked.is_empty(),
        "CHT-038: the crosshair is drawn in the picture, not the cells, found {marked:?}"
    );
}

/// A bar chart hovered on its second bar, then on its fifth from frame 36:
/// the pictures after the second hover.
fn hovered_bars(class: Option<&str>) -> (Vec<Picture>, Vec<(u32, u32)>) {
    // The bars' columns come from a first run, so the hovers land on them.
    let mut props = bare(ChartType::BarVertical, (80, 24), &VALUES);
    props.class = class.map(str::to_owned);
    let plain = run(
        chart(props.clone()),
        (80, 24),
        kitty(),
        silent(),
        after_pictures(1, 2),
        true,
    );
    let bars = bar_columns(&plain.first_picture("a bar chart"));
    assert_eq!(bars.len(), 6, "six bars in the picture: {bars:?}");
    let cell_of = |bar: (u32, u32)| ((bar.0 + bar.1) / 2 / u32::from(CELL.0)) as u16;
    let run = run(
        chart(props),
        (80, 24),
        kitty(),
        at_frames(vec![
            (6, hover(cell_of(bars[1]), 20)),
            (36, hover(cell_of(bars[4]), 20)),
        ]),
        after_frames(76),
        true,
    );
    let pictures: Vec<Picture> = run
        .pictures()
        .into_iter()
        .filter(|picture| picture.frame > 36)
        .collect();
    assert!(
        !pictures.is_empty(),
        "CHT-038: hovering another bar sends a new picture; none came after the second hover ({} pictures in all)",
        run.pictures().len()
    );
    (pictures, bars)
}

#[test]
fn cht_038_a_hovered_bar_chart_fades_the_others_and_glides_the_band() {
    let (pictures, bars) = hovered_bars(None);
    let centers: Vec<f64> = pictures.iter().filter_map(band_center).collect();
    // The band moves toward the fifth bar and never back, through at least
    // three positions; once it has settled a later picture may repeat the
    // final center.
    let mut distinct = centers.clone();
    distinct.dedup();
    assert!(
        distinct.len() >= 3 && centers.windows(2).all(|pair| pair[1] >= pair[0]),
        "CHT-038: the band glides from the second bar to the fifth over several pictures, found centers {centers:?}"
    );
    let target = f64::from(bars[4].0 + bars[4].1) / 2.0;
    assert!(
        centers
            .last()
            .is_some_and(|last| (last - target).abs() <= 8.0),
        "CHT-038: the band settles on the fifth bar at {target}, found {centers:?}"
    );
    let last = pictures.last().unwrap();
    let series = role("chart-1");
    let y = last.size.1 - 10;
    let full_in = |picture: &Picture, bar: (u32, u32)| {
        let pixel = picture.pixel((bar.0 + bar.1) / 2, y);
        pixel[3] >= 240 && near(pixel, series, 12)
    };
    let full = |bar: (u32, u32)| full_in(last, bar);
    // From the first picture of the hover, while the band still glides
    // from the second bar, the fifth bar is in its full color.
    let faded_hovered: Vec<usize> = pictures
        .iter()
        .enumerate()
        .filter(|(_, picture)| !full_in(picture, bars[4]))
        .map(|(i, _)| i)
        .collect();
    assert!(
        faded_hovered.is_empty(),
        "CHT-038: the hovered bar keeps its full color in every picture of the glide; pictures {faded_hovered:?} of {} faded it",
        pictures.len()
    );
    assert!(
        full(bars[4]),
        "CHT-038: the hovered bar keeps its full color, found {:?}",
        last.pixel((bars[4].0 + bars[4].1) / 2, y)
    );
    let unfaded: Vec<usize> = (0..6).filter(|&i| i != 4 && full(bars[i])).collect();
    assert!(
        unfaded.is_empty(),
        "CHT-038: the other bars fade toward the background while one is hovered; bars {unfaded:?} kept their full color"
    );
}

#[test]
fn cht_038_reduced_motion_snaps_the_band() {
    let (pictures, bars) = hovered_bars(Some("reduced-motion"));
    assert_eq!(
        pictures.len(),
        1,
        "CHT-038: under reduced-motion a hover change is one picture, found {}",
        pictures.len()
    );
    let target = f64::from(bars[4].0 + bars[4].1) / 2.0;
    let center = band_center(&pictures[0]);
    assert!(
        center.is_some_and(|center| (center - target).abs() <= 8.0),
        "CHT-038: the band is at the fifth bar at once, found {center:?} against {target}"
    );
}

#[test]
fn cht_038_a_hovered_scatter_rings_its_point() {
    let run = run(
        chart(bare(ChartType::Scatter, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        at_frames(vec![(6, hover(32, 12))]),
        after_pictures(2, 12),
        true,
    );
    let pictures = run.pictures();
    let before = pictures
        .first()
        .unwrap_or_else(|| panic!("CHT-038: the scatter sent no picture"));
    let after = pictures.last().filter(|p| p.frame > before.frame).unwrap_or_else(|| {
        panic!("CHT-038: hovering a scatter's point sends a new picture; none came after the hover")
    });
    let ring = role("ring");
    let ringed = |p: &Picture| {
        p.pixels
            .iter()
            .filter(|pixel| pixel[3] > 100 && near(**pixel, ring, 30))
            .count()
    };
    assert!(
        ringed(after) >= ringed(before) + 20,
        "CHT-038: the selected point has a one-pixel ring in the ring role; ring-colored pixels went from {} to {}",
        ringed(before),
        ringed(after)
    );
}

#[test]
fn cht_038_a_same_index_move_sends_no_picture() {
    let run = run(
        chart(bare(ChartType::Line, (80, 24), &VALUES)),
        (80, 24),
        kitty(),
        at_frames(vec![(6, hover(32, 12)), (40, hover(33, 13))]),
        after_frames(70),
        true,
    );
    let pictures = run.pictures();
    let after_first = pictures.iter().filter(|p| p.frame > 6).count();
    let after_second = pictures.iter().filter(|p| p.frame > 40).count();
    assert!(
        after_first >= 1,
        "CHT-038: the first hover sends a picture ({} pictures in all)",
        pictures.len()
    );
    assert_eq!(
        after_second, 0,
        "CHT-038: a move that keeps the hovered index sends no picture (CHT-019)"
    );
}

// ---------------------------------------------------------------------------
// CHT-039: the nine cartesian charts of the Charts page, all on screen.

/// The catalog's nine cartesian charts at `size` cells each.
fn cartesian_charts(size: (u16, u16)) -> Vec<Element> {
    let (w, h) = size;
    vec![
        LineChartBuilder::new(samples())
            .x(|s| s.label)
            .y(|s| s.value)
            .name("value")
            .y(|s| s.open)
            .name("open")
            .natural()
            .size(w, h)
            .render(),
        AreaChartBuilder::new(samples())
            .x(|s| s.label)
            .y(|s| s.value)
            .name("value")
            .fill("chart-2")
            .step_after()
            .size(w, h)
            .render(),
        AreaChartBuilder::new(samples())
            .x(|s| s.label)
            .y(|s| s.value)
            .name("value")
            .y(|s| s.open)
            .name("open")
            .stacked(true)
            .size(w, h)
            .render(),
        ScatterChartBuilder::new(samples())
            .x(|s| s.open)
            .y(|s| s.close)
            .name("close by open")
            .y(|s| s.value)
            .name("value by open")
            .size(w, h)
            .render(),
        BarChartBuilder::new(samples())
            .band(|s| s.label)
            .value(|s| s.value)
            .name("value")
            .fill("chart-3")
            .size(w, h)
            .render(),
        BarChartBuilder::new(samples())
            .band(|s| s.label)
            .value(|s| s.value)
            .name("value")
            .alignment(BarGrowth::Left)
            .size(w, h)
            .render(),
        BarChartBuilder::new(samples())
            .band(|s| s.label)
            .value(|s| s.value)
            .name("value")
            .value(|s| s.open)
            .name("open")
            .size(w, h)
            .render(),
        BarChartBuilder::new(samples())
            .band(|s| s.label)
            .value(|s| s.value)
            .name("value")
            .value(|s| s.open)
            .name("open")
            .stacked(true)
            .size(w, h)
            .render(),
        CandlestickChartBuilder::new(samples())
            .x(|s| s.label)
            .open(|s| s.open)
            .close(|s| s.close)
            .high(|s| s.open.max(s.close) + 1.0)
            .low(|s| s.open.min(s.close) - 1.0)
            .size(w, h)
            .render(),
    ]
}

/// The nine charts in three rows of three, each 80 by 20 cells.
fn charts_grid() -> Element {
    let mut charts = cartesian_charts((80, 20)).into_iter();
    let mut page = div().class("flex-col w-full h-full");
    for _ in 0..3 {
        let mut row = div().class("flex-row w-full h-20");
        for _ in 0..3 {
            row = row.child(charts.next().unwrap());
        }
        page = page.child(row.build());
    }
    page.build()
}

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() * 95 / 100]
}

#[test]
#[ignore = "a release-build bound on the Linux host; charts-pictures runs it there and records the other hosts"]
fn cht_039_the_nine_charts_keep_the_frame_budget_with_every_plot_a_picture() {
    const CHARTS: usize = 9;
    const MEASURED: usize = 60;
    let host = ImageOutputOptions {
        kitty_graphics: true,
        kitty_shared_memory: cfg!(unix),
        cell_pixels: CELL,
        ..Default::default()
    };
    // The frame at which every chart had sent a picture, shared between the
    // script, which hovers from then on, and the stop rule.
    let ready: Arc<Mutex<Option<usize>>> = Arc::default();
    let script_ready = Arc::clone(&ready);
    let stop_ready = Arc::clone(&ready);
    // The vertical bar chart is the middle of the second row: cells 80 to
    // 159 across, 20 to 39 down; the pointer moves to another of its six
    // bands every frame.
    let mut last_frame = None;
    let script: Script = Box::new(move |presented, pictures| {
        let mut ready = script_ready.lock().unwrap();
        if ready.is_none() && pictures >= CHARTS {
            *ready = Some(presented);
        }
        let from = (*ready)?;
        if presented <= from || last_frame == Some(presented) {
            return None;
        }
        last_frame = Some(presented);
        let band = (presented - from) % 6;
        Some(hover(86 + 12 * band as u16, 32))
    });
    let stop: Stop = Box::new(move |frame, _, elapsed| {
        let ready = stop_ready.lock().unwrap();
        ready.is_some_and(|from| frame >= from + MEASURED + 2) || elapsed >= GUARD
    });
    let run = run(charts_grid(), (240, 60), host, script, stop, false);
    let first = run.frames.first().expect("a first frame");
    // Every chart has sent its first picture when pictures at nine distinct
    // placements have been seen: a chart's repeated pictures share one.
    let placed = run.pictures();
    let all_sent = (0..run.frames.len())
        .find(|&frame| {
            placed
                .iter()
                .filter(|picture| picture.frame <= frame)
                .filter_map(|picture| picture.at)
                .collect::<BTreeSet<_>>()
                .len()
                >= CHARTS
        })
        .map(|frame| run.frames[frame].began - first.began);
    let from = ready.lock().unwrap().unwrap_or(run.frames.len());
    let measured: Vec<&Frame> = run.frames.iter().skip(from).take(MEASURED).collect();
    // No frame to measure, because the pictures never all came, counts as
    // over the bound.
    let (work, waited) = if measured.is_empty() {
        (Duration::MAX, Duration::MAX)
    } else {
        (
            p95(measured.iter().map(|frame| frame.work).collect::<Vec<_>>()),
            p95(measured
                .iter()
                .map(|frame| frame.waited)
                .collect::<Vec<_>>()),
        )
    };
    let pictures = run.frames.last().map_or(0, |frame| frame.pictures);
    println!(
        "CHT-039 nine charts at 240 by 60 as pictures: all {CHARTS} first pictures after {}, then over {} frames of hovering a bar every frame the App's work per frame p95 {:.2} ms and its wait in present p95 {:.2} ms; {pictures} pictures in {:.2} s",
        all_sent.map_or("never".to_owned(), |t| format!("{:.0} ms", t.as_secs_f64() * 1e3)),
        measured.len(),
        work.as_secs_f64() * 1e3,
        waited.as_secs_f64() * 1e3,
        run.span().as_secs_f64()
    );
    let in_time = all_sent.is_some_and(|t| t <= Duration::from_secs(1));
    // The bound binds on the Linux development host; elsewhere the numbers
    // are recorded (CHT-039).
    let bound = !cfg!(target_os = "linux")
        || (measured.len() == MEASURED && work <= BOUND && waited <= BOUND);
    assert!(
        in_time && bound,
        "CHT-039: every chart sent its first picture within a second ({in_time}) and, on the Linux host, the App's work per frame and its wait in present stayed under 16.6 ms at the 95th percentile over {} frames: work p95 {:.2} ms, wait p95 {:.2} ms",
        measured.len(),
        work.as_secs_f64() * 1e3,
        waited.as_secs_f64() * 1e3
    );
}
