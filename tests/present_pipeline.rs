//! present-pipeline mechanism: PIP-001 and PIP-002 (docs/spec/presentation.md).
//!
//! A writer with a controllable flush delay and failure observes when
//! `present` returns relative to the terminal write, that one frame is in
//! flight, and how a flush failure is reported.

use reactive_tui::backend::{Backend, SuprTuiBackend};
use reactive_tui::component::{Element, LayoutType};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
struct State {
    bytes: Vec<u8>,
    flush_delay: Duration,
    fail_flush: bool,
    flushes: usize,
    in_flight: usize,
    max_in_flight: usize,
}

#[derive(Clone, Default)]
struct Writer(Arc<Mutex<State>>);

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let (delay, fail) = {
            let mut state = self.0.lock().unwrap();
            state.in_flight += 1;
            state.max_in_flight = state.max_in_flight.max(state.in_flight);
            (state.flush_delay, state.fail_flush)
        };
        // A slow terminal: each flush takes the configured delay.
        thread::sleep(delay);
        let mut state = self.0.lock().unwrap();
        state.in_flight -= 1;
        state.flushes += 1;
        if fail {
            return Err(io::Error::other("controlled flush failure"));
        }
        Ok(())
    }
}

impl Writer {
    fn with_delay(delay: Duration) -> Self {
        let writer = Self::default();
        writer.0.lock().unwrap().flush_delay = delay;
        writer
    }

    fn take(&self) -> Vec<u8> {
        std::mem::take(&mut self.0.lock().unwrap().bytes)
    }

    fn flushes(&self) -> usize {
        self.0.lock().unwrap().flushes
    }

    fn set_fail_flush(&self, fail: bool) {
        self.0.lock().unwrap().fail_flush = fail;
    }

    /// Block until the worker has flushed `count` times. The deadline is a
    /// hang guard, generous so a busy machine cannot fail a correct test.
    fn wait_for_flushes(&self, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while self.flushes() < count {
            assert!(Instant::now() < deadline, "the worker never flushed");
            thread::sleep(Duration::from_millis(1));
        }
    }
}

fn frame(text: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full p-0.5 bg-blue-500")
        .with_children(vec![
            Element::text(text).with_class("w-full h-1 text-red-500 font-bold")
        ])
}

/// A frame whose child is narrower, so its painted geometry differs.
fn narrow_frame(text: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full p-0.5 bg-blue-500")
        .with_children(vec![
            Element::text(text).with_class("w-6 h-1 text-red-500 font-bold")
        ])
}

fn show(backend: &mut SuprTuiBackend, element: &Element) {
    assert!(backend.render_frame(element).unwrap());
    backend.present().unwrap();
}

/// Rows addressed by absolute cursor moves in a byte stream.
fn rows_moved_to(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);
    let mut rows = std::collections::BTreeSet::new();
    for piece in text.split("\x1b[").skip(1) {
        if let Some(end) = piece.find('H') {
            if let Some((row, _)) = piece[..end].split_once(';') {
                if let Ok(row) = row.parse::<u32>() {
                    rows.insert(row);
                }
            }
        }
    }
    rows.len()
}

/// PIP-001: present returns before the flush of the frame it submitted and
/// the next present waits for that flush, so one frame is in flight.
#[test]
fn pip_001_present_returns_before_its_flush_and_one_frame_is_in_flight() {
    // Long enough that a busy machine's stalls stay well inside it.
    let delay = Duration::from_millis(400);
    let out = Writer::with_delay(delay);
    let mut backend = SuprTuiBackend::with_writer(20, 4, out.clone()).unwrap();
    out.wait_for_flushes(out.flushes());

    backend.render_frame(&frame("one")).unwrap();
    let first_started = Instant::now();
    backend.present().unwrap();
    let first = first_started.elapsed();
    // The behavior under test: present hands the frame to the flush worker
    // and returns well inside the flush's delay.
    assert!(
        first < delay / 2,
        "present waited for its own flush: {first:?} with a {delay:?} flush"
    );

    backend.render_frame(&frame("two")).unwrap();
    backend.present().unwrap();
    // The behavior under test: the second present returns only once the first
    // frame's flush, which takes `delay` from the first present, is done. A
    // test thread that stalls past it only makes this weaker, never false.
    let since_first = first_started.elapsed();
    assert!(
        since_first >= delay,
        "the second present did not wait for the first frame's flush: {since_first:?}"
    );
    assert_eq!(
        out.0.lock().unwrap().max_in_flight,
        1,
        "two frames were in flight"
    );
    backend.shutdown().unwrap();
}

/// PIP-002: a flush failure is reported by the next present, the geometry
/// falls back to the last acknowledged frame, the frame after the failure is
/// a full repaint, and a failure with no present after it is reported by
/// shutdown.
#[test]
fn pip_002_flush_failure_is_reported_next_and_forces_a_full_repaint() {
    let out = Writer::default();
    let mut backend = SuprTuiBackend::with_writer(16, 4, out.clone()).unwrap();
    show(&mut backend, &frame("first"));
    backend.sync().unwrap();
    let first_flushes = out.flushes();
    let first_geometry = format!("{:?}", backend.painted_nodes().unwrap());

    out.set_fail_flush(true);
    show(&mut backend, &narrow_frame("second"));
    out.wait_for_flushes(first_flushes + 1);
    out.set_fail_flush(false);
    let second_geometry = format!("{:?}", backend.painted_nodes().unwrap());
    assert_ne!(first_geometry, second_geometry, "the frames must differ");

    backend.render_frame(&frame("third")).unwrap();
    let error = backend.present().unwrap_err();
    assert!(
        error.to_string().contains("controlled flush failure"),
        "{error:?}"
    );
    assert_eq!(
        format!("{:?}", backend.painted_nodes().unwrap()),
        first_geometry,
        "geometry must fall back to the last acknowledged frame"
    );

    out.take();
    backend.present().unwrap();
    out.wait_for_flushes(first_flushes + 2);
    let repaint = out.take();
    let text = String::from_utf8_lossy(&repaint);
    assert!(text.contains("third"), "{text:?}");
    assert_eq!(
        rows_moved_to(&repaint),
        4,
        "the frame after a failure must repaint every row: {text:?}"
    );
    // The repainted frame's geometry is the one a fresh backend reports for
    // the same frame.
    let mut fresh = SuprTuiBackend::with_writer(16, 4, Writer::default()).unwrap();
    show(&mut fresh, &frame("third"));
    assert_eq!(
        format!("{:?}", backend.painted_nodes().unwrap()),
        format!("{:?}", fresh.painted_nodes().unwrap()),
        "after the repaint the geometry is the repainted frame's"
    );

    out.set_fail_flush(true);
    show(&mut backend, &frame("fourth"));
    out.wait_for_flushes(first_flushes + 3);
    assert!(
        backend.shutdown().is_err(),
        "shutdown must report the unreported flush failure"
    );
}

/// PIP-002: a flush failure that `sync` reports falls back to the last
/// acknowledged frame's geometry, as a failure the next present reports
/// does; the frame that failed never becomes the fallback for a later
/// failure.
#[test]
fn pip_002_a_failure_reported_by_sync_restores_the_acknowledged_geometry() {
    let out = Writer::default();
    let mut backend = SuprTuiBackend::with_writer(16, 4, out.clone()).unwrap();
    show(&mut backend, &frame("first"));
    backend.sync().unwrap();
    let first = format!("{:?}", backend.painted_nodes().unwrap());
    let flushed = out.flushes();

    out.set_fail_flush(true);
    show(&mut backend, &narrow_frame("second"));
    out.wait_for_flushes(flushed + 1);
    out.set_fail_flush(false);
    let second = format!("{:?}", backend.painted_nodes().unwrap());
    assert_ne!(first, second, "the frames must differ");
    let error = backend.sync().unwrap_err();
    assert!(
        error.to_string().contains("controlled flush failure"),
        "{error:?}"
    );
    assert_eq!(
        format!("{:?}", backend.painted_nodes().unwrap()),
        first,
        "after sync reports the failure the geometry is the acknowledged frame's"
    );

    // The next frame presents; its own flush fails, and the next present
    // reports that. The fallback is still the first frame, the last one
    // whose flush succeeded, never the failed second frame.
    out.set_fail_flush(true);
    show(&mut backend, &narrow_frame("third"));
    out.wait_for_flushes(flushed + 2);
    out.set_fail_flush(false);
    backend.render_frame(&frame("fourth")).unwrap();
    backend.present().unwrap_err();
    assert_eq!(
        format!("{:?}", backend.painted_nodes().unwrap()),
        first,
        "the fallback must be the last frame whose flush was acknowledged"
    );
}
