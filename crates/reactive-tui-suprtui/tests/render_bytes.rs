//! render-bytes mechanism: RAS-001, RAS-002, RAS-005 and RAS-008 on the
//! rasterizer's byte stream and timings (docs/spec/rasterizer.md).

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use suprtui::ansi::{self, CellDecoration, TextAttributes, UnderlineStyle};
use suprtui::render::{Backend, MemoryBackend, RenderStatus, Renderer, WriteStatus};
use suprtui::uni::pool::GraphemePool;

fn renderer(width: u32, height: u32) -> Renderer<'static, MemoryBackend> {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    Renderer::new(width, height, pool, MemoryBackend::new()).unwrap()
}

fn draw(r: &mut Renderer<'static, MemoryBackend>, text: &str, x: u32, y: u32) {
    r.next_buffer()
        .draw_text(
            text,
            x,
            y,
            ansi::rgb_color(255, 255, 255, 255),
            Some(ansi::rgb_color(0, 0, 0, 255)),
            0,
        )
        .unwrap();
}

fn last_frame(r: &Renderer<'static, MemoryBackend>) -> Vec<u8> {
    r.backend().frames().last().cloned().unwrap_or_default()
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| w == &needle)
        .count()
}

/// Cursor-move sequences: CUP (`ESC [ r ; c H`) and CHA (`ESC [ c G`).
fn moves(frame: &[u8]) -> (usize, usize) {
    let mut cups = 0;
    let mut chas = 0;
    let mut i = 0;
    let bytes = frame;
    while i + 2 < bytes.len() {
        if bytes[i] == 0x1b && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b';') {
                j += 1;
            }
            if j < bytes.len() {
                match bytes[j] {
                    b'H' => cups += 1,
                    b'G' => chas += 1,
                    _ => {}
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    (cups, chas)
}

/// RAS-001: adjacent changed cells emit one move; a same-row jump uses CHA;
/// a new row uses CUP.
#[test]
fn ras_001_adjacent_cells_share_one_move_and_same_row_jumps_use_cha() {
    let mut r = renderer(20, 4);
    draw(&mut r, "ab", 2, 1);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    let (cups, chas) = moves(&frame);
    assert_eq!((cups, chas), (1, 0), "two adjacent cells: {frame:?}");
    assert_eq!(1, r.stats().moves_emitted);
    assert_eq!(1, r.stats().moves_elided);

    // Same row, a gap: CHA, no CUP for the second run.
    draw(&mut r, "c", 2, 1);
    draw(&mut r, "d", 9, 1);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    let (cups, chas) = moves(&frame);
    assert_eq!((cups, chas), (1, 1), "same-row jump must be CHA: {frame:?}");
    assert!(
        count(&frame, b"\x1b[10G") == 1,
        "CHA to column 10: {frame:?}"
    );

    // A new row: CUP.
    draw(&mut r, "e", 2, 1);
    draw(&mut r, "f", 2, 3);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    let (cups, chas) = moves(&frame);
    assert_eq!((cups, chas), (2, 0), "a row change must be CUP: {frame:?}");
}

/// RAS-001: a wide glyph advances the tracked cursor by two columns, so the
/// cell after its continuation emits no move.
#[test]
fn ras_001_wide_glyphs_advance_the_cursor_by_their_width() {
    let mut r = renderer(10, 2);
    draw(&mut r, "界x", 0, 0);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    let (cups, chas) = moves(&frame);
    assert_eq!((cups, chas), (1, 0), "wide glyph then x: {frame:?}");
    assert!(count(&frame, "界x".as_bytes()) == 1, "{frame:?}");
}

/// RAS-002: one style across many cells emits one foreground and one
/// background sequence; no per-cell reset; attributes turn off with their
/// own codes.
#[test]
fn ras_002_style_is_emitted_once_per_run_and_reset_at_most_twice_per_frame() {
    let mut r = renderer(40, 3);
    draw(&mut r, "the quick brown fox", 0, 0);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    assert_eq!(1, count(&frame, b"\x1b[38;2;255;255;255m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[48;2;0;0;0m"), "{frame:?}");
    assert_eq!(2, count(&frame, b"\x1b[0m"), "{frame:?}");
    assert_eq!(1, r.stats().fg_emitted);
    assert_eq!(15, r.stats().fg_elided, "sixteen non-space cells changed");

    // Bold then plain on one row: SGR 1 once, SGR 22 once, no reset between.
    r.next_buffer()
        .draw_text(
            "bold",
            0,
            1,
            ansi::rgb_color(255, 255, 255, 255),
            Some(ansi::rgb_color(0, 0, 0, 255)),
            u32::from(TextAttributes::BOLD),
        )
        .unwrap();
    draw(&mut r, "plain", 4, 1);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    assert_eq!(1, count(&frame, b"\x1b[1m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[22m"), "{frame:?}");
    assert_eq!(2, count(&frame, b"\x1b[0m"), "{frame:?}");

    // Underline style and color end with 24 and 59, overline with 55.
    let styled = CellDecoration {
        underline: UnderlineStyle::Curly,
        underline_color: Some([1, 2, 3]),
        overline: true,
    };
    let mut cell = suprtui::buffer::make_cell(
        u32::from(b'u'),
        ansi::rgb_color(255, 255, 255, 255),
        ansi::rgb_color(0, 0, 0, 255),
        0,
    );
    cell.decoration = styled;
    r.next_buffer().set(0, 2, cell);
    draw(&mut r, "v", 1, 2);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    let frame = last_frame(&r);
    assert_eq!(1, count(&frame, b"\x1b[4:3m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[58:2::1:2:3m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[53m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[24m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[59m"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[55m"), "{frame:?}");
}

/// RAS-002: every attribute that ends between two adjacent cells turns off
/// with its own SGR code, emitted once, and the frame holds no reset beyond
/// the one after its sync-set and the one before its sync-reset.
#[test]
fn ras_002_every_attribute_turns_off_with_its_own_code() {
    for (name, attribute, on, off) in [
        (
            "bold",
            TextAttributes::BOLD,
            &b"\x1b[1m"[..],
            &b"\x1b[22m"[..],
        ),
        ("dim", TextAttributes::DIM, b"\x1b[2m", b"\x1b[22m"),
        ("italic", TextAttributes::ITALIC, b"\x1b[3m", b"\x1b[23m"),
        (
            "underline",
            TextAttributes::UNDERLINE,
            b"\x1b[4m",
            b"\x1b[24m",
        ),
        ("blink", TextAttributes::BLINK, b"\x1b[5m", b"\x1b[25m"),
        ("inverse", TextAttributes::INVERSE, b"\x1b[7m", b"\x1b[27m"),
        ("hidden", TextAttributes::HIDDEN, b"\x1b[8m", b"\x1b[28m"),
        (
            "strikethrough",
            TextAttributes::STRIKETHROUGH,
            b"\x1b[9m",
            b"\x1b[29m",
        ),
    ] {
        let mut r = renderer(12, 1);
        r.next_buffer()
            .draw_text(
                "on",
                0,
                0,
                ansi::rgb_color(255, 255, 255, 255),
                Some(ansi::rgb_color(0, 0, 0, 255)),
                u32::from(attribute),
            )
            .unwrap();
        draw(&mut r, "off", 2, 0);
        assert_eq!(RenderStatus::Rendered, r.render(false));
        let frame = last_frame(&r);
        assert_eq!(1, count(&frame, on), "{name} on: {frame:?}");
        assert_eq!(1, count(&frame, off), "{name} off: {frame:?}");
        assert_eq!(
            2,
            count(&frame, b"\x1b[0m"),
            "{name}: a reset beyond the frame's two: {frame:?}"
        );
    }
}

/// A backend that keeps every frame's bytes in one reused buffer.
struct Sink {
    bytes: Vec<u8>,
    frames: usize,
}

impl Backend for Sink {
    fn prepare_frame(&mut self) -> WriteStatus {
        WriteStatus::Ok
    }
    fn begin_frame(&mut self) {
        self.bytes.clear();
    }
    fn write_bytes(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }
    fn write_out(&mut self, _: &[u8]) {}
    fn fail_frame(&mut self) {}
    fn end_frame(&mut self) -> WriteStatus {
        if !self.bytes.is_empty() {
            self.frames += 1;
        }
        WriteStatus::Ok
    }
}

fn sink_renderer(width: u32, height: u32) -> Renderer<'static, Sink> {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    Renderer::new(
        width,
        height,
        pool,
        Sink {
            bytes: Vec::with_capacity(1 << 24),
            frames: 0,
        },
    )
    .unwrap()
}

fn fill_text<B: Backend>(r: &mut Renderer<'static, B>, width: u32, height: u32, seed: u8) {
    let line: String = (0..width)
        .map(|x| char::from(b'a' + ((x as u8).wrapping_add(seed)) % 26))
        .collect();
    for y in 0..height {
        r.next_buffer()
            .draw_text(
                &line,
                0,
                y,
                ansi::rgb_color(220, 220, 220, 255),
                Some(ansi::rgb_color(10, 10, 30, 255)),
                0,
            )
            .unwrap();
    }
}

/// Text in style runs, as a highlighted listing or a log view draws it: the
/// foreground changes every eight cells and every other run is bold.
fn fill_styled_text<B: Backend>(r: &mut Renderer<'static, B>, width: u32, height: u32) {
    for y in 0..height {
        for run in 0..width.div_ceil(8) {
            let x = run * 8;
            let text: String = (x..(x + 8).min(width))
                .map(|column| char::from(b'a' + (column % 26) as u8))
                .collect();
            let shade = ((run * 37 + y * 11) % 200) as u8;
            r.next_buffer()
                .draw_text(
                    &text,
                    x,
                    y,
                    ansi::rgb_color(55 + shade, 200 - shade / 2, 120, 255),
                    Some(ansi::rgb_color(10, 10, 30, 255)),
                    if run % 2 == 0 {
                        0
                    } else {
                        u32::from(TextAttributes::BOLD)
                    },
                )
                .unwrap();
        }
    }
}

/// RAS-005's bound on the render call for an unchanged full-size frame.
const UNCHANGED_BOUND_US: u64 = 1000;

fn best_of_three(mut run: impl FnMut() -> Duration) -> Duration {
    (0..3).map(|_| run()).min().unwrap()
}

/// The host's one-minute load average, for a timing failure's message.
/// The unchanged frame is bound by memory reads, so other builds on the
/// host slow it: at load average 4.7 it took 293 to 554 microseconds on
/// this host's performance cores and up to 1.14 milliseconds on its
/// efficiency cores, and with eight memory-copy loops running it took
/// 4.4 milliseconds. The test runs wherever the scheduler puts it, as the
/// render worker does.
fn load_average() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|text| text.split_whitespace().next().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string())
}

/// RAS-005: a full 200 by 50 text repaint is at most 12 bytes per cell, in
/// one style and in style runs; an unchanged 262,144-cell frame costs at
/// most 1 millisecond and a full repaint of it at most 15 milliseconds, the
/// best of three runs, in a release build. A debug build checks the byte
/// bound and a scaled time bound. The mechanism runs this test once, so a
/// time bound fails when all three runs miss it, and a failure names the
/// load average.
#[test]
fn ras_005_byte_and_time_bounds_hold_best_of_three() {
    let scale: u32 = if cfg!(debug_assertions) { 40 } else { 1 };
    for (workload, styled) in [("one-style", false), ("styled", true)] {
        let mut r = sink_renderer(200, 50);
        if styled {
            fill_styled_text(&mut r, 200, 50);
        } else {
            fill_text(&mut r, 200, 50, 0);
        }
        assert_eq!(RenderStatus::Rendered, r.render(true));
        let bytes = r.backend().bytes.len();
        let per_cell = bytes as f64 / (200.0 * 50.0);
        eprintln!("{workload} 200x50 text repaint: {bytes} bytes, {per_cell:.2} per cell");
        assert!(
            per_cell <= 12.0,
            "{workload} 200x50 text repaint emitted {bytes} bytes, {per_cell:.2} per cell"
        );
    }

    let mut big = sink_renderer(512, 512);
    fill_text(&mut big, 512, 512, 3);
    assert_eq!(RenderStatus::Rendered, big.render(true));
    let unchanged = best_of_three(|| {
        // The painter repaints the same frame between renders, which is
        // what evicts the buffers from cache; time the render call after
        // that repaint, as the loop pays it.
        fill_text(&mut big, 512, 512, 3);
        let started = Instant::now();
        let status = big.render(false);
        let took = started.elapsed();
        assert_eq!(RenderStatus::Skipped, status);
        took
    });
    eprintln!("unchanged 512x512 frame, best of three: {unchanged:?}");
    assert!(
        unchanged <= Duration::from_micros(UNCHANGED_BOUND_US * u64::from(scale)),
        "unchanged 512x512 frame took {unchanged:?} at load average {}",
        load_average()
    );
    let repaint = best_of_three(|| {
        fill_text(&mut big, 512, 512, 3);
        let started = Instant::now();
        let status = big.render(true);
        let took = started.elapsed();
        assert_eq!(RenderStatus::Rendered, status);
        took
    });
    eprintln!("full 512x512 repaint, best of three: {repaint:?}");
    assert!(
        repaint <= Duration::from_millis(15 * u64::from(scale)),
        "full 512x512 repaint took {repaint:?} at load average {}",
        load_average()
    );
}

/// RAS-008: the diff finds the first changed column of a row from the
/// column arrays, so a change to only a decoration in the last column of the
/// last row is published and nothing else is.
#[test]
fn ras_008_row_diff_finds_the_first_changed_column_in_any_array() {
    let mut r = renderer(30, 6);
    fill_text(&mut r, 30, 6, 0);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    fill_text(&mut r, 30, 6, 0);
    assert_eq!(RenderStatus::Skipped, r.render(false));

    fill_text(&mut r, 30, 6, 0);
    let mut cell = r.next_buffer().get(29, 5).unwrap();
    cell.decoration.overline = true;
    r.next_buffer().set(29, 5, cell);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    assert_eq!(1, r.stats().cells_updated);
    let frame = last_frame(&r);
    assert_eq!(1, count(&frame, b"\x1b[6;30H"), "{frame:?}");
    assert_eq!(1, count(&frame, b"\x1b[53m"), "{frame:?}");
}
