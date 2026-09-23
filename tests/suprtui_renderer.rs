use reactive_tui::backend::{Backend, SuprTuiBackend};
use reactive_tui::component::{Element, LayoutType};
use reactive_tui::error::ReactiveError;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    flushes: usize,
    remaining: Option<usize>,
    fail_flush: bool,
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Output>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut out = self.0.lock().unwrap();
        if out.remaining == Some(0) {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "controlled write failure",
            ));
        }
        let count = out.remaining.unwrap_or(bytes.len()).min(bytes.len());
        if let Some(left) = &mut out.remaining {
            *left -= count;
        }
        out.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut out = self.0.lock().unwrap();
        out.flushes += 1;
        if out.fail_flush {
            return Err(io::Error::other("controlled flush failure"));
        }
        Ok(())
    }
}

impl Capture {
    fn take(&self) -> Vec<u8> {
        std::mem::take(&mut self.0.lock().unwrap().bytes)
    }
}

fn frame(text: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full p-0.5 bg-blue-500")
        .with_children(vec![
            Element::text(text).with_class("w-full h-1 text-red-500 font-bold")
        ])
}

fn show(backend: &mut SuprTuiBackend, element: &Element) {
    assert!(backend.render_frame(element).unwrap());
    backend.present().unwrap();
}

#[test]
fn rnd_001_styled_layout_update_removal_and_unchanged_screen() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(12, 5, out.clone()).unwrap();
    let mut terminal = vt100::Parser::new(5, 12, 0);
    show(&mut backend, &frame("hello"));
    terminal.process(&out.take());
    let cell = terminal.screen().cell(2, 2).unwrap();
    assert_eq!(
        cell.contents(),
        "h",
        "screen: {:?}",
        terminal.screen().contents()
    );
    assert!(cell.bold());
    assert_eq!(cell.fgcolor(), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(
        terminal.screen().cell(0, 0).unwrap().bgcolor(),
        vt100::Color::Rgb(59, 130, 246)
    );
    show(&mut backend, &frame("hi"));
    terminal.process(&out.take());
    assert_eq!(terminal.screen().cell(2, 3).unwrap().contents(), "i");
    assert_eq!(terminal.screen().cell(2, 4).unwrap().contents(), " ");
    show(&mut backend, &frame("hi"));
    assert!(out.take().is_empty(), "unchanged frame emitted bytes");
    show(&mut backend, &Element::empty());
    terminal.process(&out.take());
    assert!(terminal.screen().contents().trim().is_empty());
}

#[test]
fn rnd_002_preserves_clusters_and_clips_at_cell_boundaries() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(8, 2, out.clone()).unwrap();
    let text = "e\u{301}界👩‍💻";
    show(&mut backend, &Element::text(text).with_class("w-full h-1"));
    let bytes = out.take();
    let output = String::from_utf8(bytes.clone()).unwrap();
    for cluster in ["e\u{301}", "界", "👩‍💻"] {
        assert_eq!(output.matches(cluster).count(), 1, "{output:?}");
    }
    let mut terminal = vt100::Parser::new(2, 8, 0);
    terminal.process(&bytes);
    assert_eq!(terminal.screen().cell(0, 0).unwrap().contents(), "e\u{301}");
    assert_eq!(terminal.screen().cell(0, 1).unwrap().contents(), "界");
    assert!(terminal.screen().cell(0, 2).unwrap().is_wide_continuation());
    show(&mut backend, &Element::text(text).with_class("w-full h-1"));
    assert!(
        out.take().is_empty(),
        "unchanged Unicode frame emitted bytes"
    );
    show(&mut backend, &Element::text("ab界").with_class("w-3 h-1"));
    let clipped = String::from_utf8(out.take()).unwrap();
    assert!(
        !clipped.contains('界'),
        "wide glyph straddled the clip boundary"
    );
    show(
        &mut backend,
        &Element::text("\x1b[31m").with_class("w-full h-1"),
    );
    assert!(!out.take().windows(5).any(|w| w == b"\x1b[31m"));
}

#[test]
fn rnd_003_resize_matches_fresh_frame_and_ignores_zero() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(16, 6, out.clone()).unwrap();
    let mut resized_terminal = vt100::Parser::new(6, 16, 0);
    show(&mut backend, &frame("resize me"));
    resized_terminal.process(&out.take());
    backend.resize(0, 0);
    assert_eq!(backend.size(), (16, 6));
    show(&mut backend, &frame("resize me"));
    assert!(out.take().is_empty());
    for (width, height) in [(8, 4), (20, 7)] {
        backend.resize(width, height);
        show(&mut backend, &frame("resize me"));
        let resized = out.take();
        let fresh_out = Capture::default();
        let mut fresh =
            SuprTuiBackend::with_writer(width as u16, height as u16, fresh_out.clone()).unwrap();
        show(&mut fresh, &frame("resize me"));
        resized_terminal
            .screen_mut()
            .set_size(height as u16, width as u16);
        let mut fresh_terminal = vt100::Parser::new(height as u16, width as u16, 0);
        resized_terminal.process(&resized);
        fresh_terminal.process(&fresh_out.take());
        assert_eq!(
            resized_terminal.screen().contents_formatted(),
            fresh_terminal.screen().contents_formatted()
        );
    }
}

#[test]
fn rnd_001_grid_and_overlapping_layers() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(10, 4, out.clone()).unwrap();
    let grid = Element::layout(LayoutType::Grid)
        .with_class("grid grid-cols-2 w-full h-1")
        .with_children(vec![Element::text("A"), Element::text("B")]);
    show(&mut backend, &grid);
    let mut terminal = vt100::Parser::new(4, 10, 0);
    terminal.process(&out.take());
    assert_eq!(terminal.screen().cell(0, 0).unwrap().contents(), "A");
    assert_eq!(terminal.screen().cell(0, 5).unwrap().contents(), "B");
    let layers = Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(vec![
            Element::text("X").with_class("absolute top-0 left-0 w-1 h-1 z-20"),
            Element::text("lower").with_class("absolute top-0 left-0 w-5 h-1 z-10"),
        ]);
    show(&mut backend, &layers);
    terminal.process(&out.take());
    assert!(terminal.screen().contents().starts_with("Xower"));
}

#[test]
fn rnd_002_nested_absolute_text_obeys_ancestor_clip() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(12, 8, out.clone()).unwrap();
    let root = Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full p-0.5")
        .with_children(vec![Element::layout(LayoutType::Flex)
            .with_class("relative w-3 h-1 overflow-hidden")
            .with_children(vec![
                Element::text("ab界").with_class("absolute top-0 left-0 w-4 h-1")
            ])]);
    show(&mut backend, &root);
    let bytes = out.take();
    assert!(!String::from_utf8_lossy(&bytes).contains('界'));
    let mut terminal = vt100::Parser::new(8, 12, 0);
    terminal.process(&bytes);
    assert_eq!(terminal.screen().cell(2, 2).unwrap().contents(), "a");
    assert_eq!(terminal.screen().cell(2, 3).unwrap().contents(), "b");
    assert_eq!(terminal.screen().cell(2, 4).unwrap().contents(), " ");
}

#[test]
fn rnd_004_flushes_and_repaints_after_partial_write_or_flush_failure() {
    for fail_flush in [false, true] {
        let out = Capture::default();
        let mut backend = SuprTuiBackend::with_writer(12, 5, out.clone()).unwrap();
        let mut terminal = vt100::Parser::new(5, 12, 0);
        show(&mut backend, &frame("original"));
        terminal.process(&out.take());
        assert!(out.0.lock().unwrap().flushes > 0);
        {
            let mut state = out.0.lock().unwrap();
            state.fail_flush = fail_flush;
            state.remaining = if fail_flush { None } else { Some(73) };
        }
        backend.render_frame(&frame("new")).unwrap();
        let error = backend.present().unwrap_err();
        assert!(matches!(error, ReactiveError::Io(_)), "{error:?}");
        assert!(error.to_string().contains(if fail_flush {
            "controlled flush failure"
        } else {
            "controlled write failure"
        }));
        terminal.process(&out.take());
        {
            let mut state = out.0.lock().unwrap();
            state.fail_flush = false;
            state.remaining = None;
        }
        backend.present().unwrap();
        let retry = out.take();
        assert!(!retry.is_empty(), "retry was incorrectly skipped");
        terminal.process(&retry);
        let fresh_out = Capture::default();
        let mut fresh = SuprTuiBackend::with_writer(12, 5, fresh_out.clone()).unwrap();
        show(&mut fresh, &frame("new"));
        let mut expected = vt100::Parser::new(5, 12, 0);
        expected.process(&fresh_out.take());
        assert_eq!(
            terminal.screen().contents_formatted(),
            expected.screen().contents_formatted()
        );
    }
}

#[test]
fn backend_remains_send_and_sync_and_shutdown_is_idempotent() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SuprTuiBackend>();
    let mut backend = SuprTuiBackend::with_writer(4, 2, Capture::default()).unwrap();
    backend.shutdown().unwrap();
    backend.shutdown().unwrap();
    assert!(backend.present().is_err());
    assert!(SuprTuiBackend::with_writer(0, 0, Capture::default()).is_err());
}

#[test]
fn rnd_001_nested_layout_keeps_children_above_their_parent_background() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(20, 10, out.clone()).unwrap();
    let root = Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full p-0.5")
        .with_children(vec![Element::layout(LayoutType::Flex)
            .with_class("relative flex flex-col w-8 h-3 bg-green-500")
            .with_children(vec![Element::text("child").with_class("w-full h-1")])]);
    show(&mut backend, &root);
    let mut terminal = vt100::Parser::new(10, 20, 0);
    terminal.process(&out.take());
    assert_eq!(terminal.screen().cell(2, 2).unwrap().contents(), "c");
    assert_eq!(
        terminal.screen().cell(3, 2).unwrap().bgcolor(),
        vt100::Color::Rgb(34, 197, 94)
    );
}

#[test]
fn rnd_001_explicit_black_background_covers_lower_content() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(8, 4, out.clone()).unwrap();
    let root = Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full bg-blue-500")
        .with_children(vec![
            Element::layout(LayoutType::Flex).with_class("w-4 h-2 bg-black")
        ]);
    show(&mut backend, &root);
    let mut terminal = vt100::Parser::new(4, 8, 0);
    terminal.process(&out.take());
    assert_eq!(
        terminal.screen().cell(0, 0).unwrap().bgcolor(),
        vt100::Color::Rgb(0, 0, 0)
    );
    assert_eq!(
        terminal.screen().cell(3, 7).unwrap().bgcolor(),
        vt100::Color::Rgb(59, 130, 246)
    );
}

/// RAS-006: the debug overlay shows bytes, elisions and the four times.
#[test]
fn ras_006_debug_overlay_shows_bytes_elisions_and_times() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(160, 6, out.clone()).unwrap();
    backend.set_debug_overlay(true);
    let mut terminal = vt100::Parser::new(6, 160, 0);
    show(&mut backend, &frame("first"));
    terminal.process(&out.take());
    show(&mut backend, &frame("second"));
    terminal.process(&out.take());
    let overlay: String = (0..160)
        .map(|x| {
            terminal
                .screen()
                .cell(5, x)
                .map(|c| c.contents())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("");
    for needle in [
        "bytes:", "moves:", "fg:", "bg:", "attr:", "layout:", "diff:", "emit:", "write:",
    ] {
        assert!(
            overlay.contains(needle),
            "overlay lacks {needle}: {overlay:?}"
        );
    }
    let bytes: usize = overlay
        .split("bytes: ")
        .nth(1)
        .and_then(|rest| rest.split(' ').next())
        .and_then(|n| n.parse().ok())
        .expect("bytes figure");
    assert!(
        bytes > 0,
        "a rendered frame reported zero bytes: {overlay:?}"
    );
}

/// RAS-007: the painter clears the next buffer once per frame, so a smaller
/// root after a larger one leaves no stale cells.
#[test]
fn ras_007_a_smaller_frame_leaves_no_stale_cells() {
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(12, 4, out.clone()).unwrap();
    let mut terminal = vt100::Parser::new(4, 12, 0);
    show(
        &mut backend,
        &Element::text("wide wide wi").with_class("w-full h-1"),
    );
    terminal.process(&out.take());
    assert_eq!(terminal.screen().cell(0, 11).unwrap().contents(), "i");
    show(&mut backend, &Element::text("ab").with_class("w-2 h-1"));
    terminal.process(&out.take());
    assert_eq!(terminal.screen().cell(0, 0).unwrap().contents(), "a");
    assert_eq!(
        terminal.screen().cell(0, 11).unwrap().contents(),
        " ",
        "stale cell survived: {:?}",
        terminal.screen().contents()
    );
}
