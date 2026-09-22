//! sys-core commitment tests (SYS-001, SYS-002, SYS-004 … SYS-008).

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::link::{LinkPool, MAX_URL_LENGTH};
use suprtui::sys::{
    EventSink, LogLevel, Logger, SpanError, SpanFeed, SpanFeedOptions, SplitScrollback, emit,
};

// ---------------------------------------------------------------------------
// SYS-001: instance isolation (plus review of `src/` for globals).
// ---------------------------------------------------------------------------

#[test]
fn req_001_instance_isolation() {
    // Sinks observe only their own events.
    let first_seen = Rc::new(RefCell::new(Vec::new()));
    let second_seen = Rc::new(RefCell::new(Vec::new()));
    let mut first = EventSink::new({
        let seen = Rc::clone(&first_seen);
        move |name: &str, _data: &[u8]| seen.borrow_mut().push(name.to_string())
    });
    let mut second = EventSink::new({
        let seen = Rc::clone(&second_seen);
        move |name: &str, _data: &[u8]| seen.borrow_mut().push(name.to_string())
    });
    emit(Some(&mut first), "a", b"1");
    assert_eq!(first_seen.borrow().as_slice(), ["a"]);
    assert!(second_seen.borrow().is_empty());
    emit(Some(&mut second), "b", b"2");
    assert_eq!(second_seen.borrow().as_slice(), ["b"]);
    assert_eq!(first_seen.borrow().as_slice(), ["a"]);

    // Loggers keep separate levels and sinks.
    let mut quiet = Logger::new(LogLevel::Err);
    let mut loud = Logger::new(LogLevel::Debug);
    let quiet_count = Rc::new(RefCell::new(0));
    let loud_count = Rc::new(RefCell::new(0));
    quiet.set_sink({
        let count = Rc::clone(&quiet_count);
        move |_, _| *count.borrow_mut() += 1
    });
    loud.set_sink({
        let count = Rc::clone(&loud_count);
        move |_, _| *count.borrow_mut() += 1
    });
    loud.debug("hello");
    assert_eq!(*loud_count.borrow(), 1);
    assert_eq!(*quiet_count.borrow(), 0);

    // Feeds drain independently.
    let mut feed_a = SpanFeed::new(SpanFeedOptions::default()).unwrap();
    let mut feed_b = SpanFeed::new(SpanFeedOptions::default()).unwrap();
    feed_a.write(b"a").unwrap();
    feed_a.commit().unwrap();
    assert_eq!(feed_a.drain(8).len(), 1);
    assert!(feed_b.drain(8).is_empty());

    // Link pools scope ids to their pool.
    let mut pool_a = LinkPool::new();
    let pool_b = LinkPool::new();
    let id = pool_a.alloc(b"https://example.com").unwrap();
    assert!(pool_b.get(id).is_err());
    assert!(pool_a.get(id).is_ok());
}

// ---------------------------------------------------------------------------
// SYS-002: ordered delivery, destroy stops delivery.
// ---------------------------------------------------------------------------

#[test]
fn req_002_event_bus() {
    let received = Rc::new(RefCell::new(Vec::new()));
    let mut sink = EventSink::new({
        let received = Rc::clone(&received);
        move |name: &str, data: &[u8]| {
            received
                .borrow_mut()
                .push((name.to_string(), data.to_vec()));
        }
    });
    assert!(sink.is_alive());
    emit(Some(&mut sink), "first", b"1");
    emit(Some(&mut sink), "second", b"2");
    emit(Some(&mut sink), "third", b"3");
    assert_eq!(
        received.borrow().as_slice(),
        [
            ("first".to_string(), b"1".to_vec()),
            ("second".to_string(), b"2".to_vec()),
            ("third".to_string(), b"3".to_vec()),
        ]
    );

    sink.destroy();
    assert!(!sink.is_alive());
    emit(Some(&mut sink), "fourth", b"4");
    assert_eq!(received.borrow().len(), 3);

    // Missing sinks and callbacks are silent no-ops.
    emit(None, "ghost", b"0");
}

// ---------------------------------------------------------------------------
// SYS-004: link pool interning, limits, scoping.
// ---------------------------------------------------------------------------

#[test]
fn req_004_link_pool() {
    let mut pool = LinkPool::new();

    // Exactly 512 bytes intern; 513 are rejected.
    let max_url = vec![b'u'; MAX_URL_LENGTH];
    assert_eq!(MAX_URL_LENGTH, 512);
    let id = pool.alloc(&max_url).unwrap();
    assert_eq!(pool.get(id).unwrap(), max_url.as_slice());
    let long_url = vec![b'u'; MAX_URL_LENGTH + 1];
    assert!(pool.alloc(&long_url).is_err());

    // Reference counting tracks holders.
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    assert_eq!(pool.get_refcount(id).unwrap(), 2);
    pool.decref(id).unwrap();
    assert_eq!(pool.get_refcount(id).unwrap(), 1);

    // Re-allocating a live URL returns the same id.
    let same = pool.alloc(&max_url).unwrap();
    assert_eq!(same, id);

    // Ids never resolve in another pool.
    let mut other = LinkPool::new();
    assert!(other.get(id).is_err());
    assert!(other.incref(id).is_err());
    assert!(other.decref(id).is_err());
}

// ---------------------------------------------------------------------------
// SYS-005: ordered spans, atomic contiguity, exactly-once drain.
// ---------------------------------------------------------------------------

fn make_feed(chunk_size: usize, auto_commit: bool) -> SpanFeed {
    SpanFeed::new(SpanFeedOptions {
        chunk_size,
        auto_commit,
    })
    .unwrap()
}

#[test]
fn req_005_span_feed() {
    assert!(
        SpanFeed::new(SpanFeedOptions {
            chunk_size: 0,
            auto_commit: false,
        })
        .is_err()
    );

    // Ordered spans with explicit commits.
    let mut feed = make_feed(8, false);
    feed.write(b"ab").unwrap();
    feed.write(b"cd").unwrap();
    feed.commit().unwrap();
    feed.write(b"efgh").unwrap();
    feed.commit().unwrap();
    let spans = feed.drain(8);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].data, b"abcd");
    assert_eq!(spans[1].data, b"efgh");
    // Drained spans never reappear.
    assert!(feed.drain(8).is_empty());

    // Atomic writes stay contiguous across chunk boundaries.
    let mut feed = make_feed(4, false);
    feed.write_atomic(b"0123456789").unwrap();
    let spans = feed.drain(8);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].data, b"0123456789");

    // Atomic writes flush earlier pending bytes first, preserving order.
    let mut feed = make_feed(4, false);
    feed.write(b"ab").unwrap();
    feed.write_atomic(b"CD").unwrap();
    let spans = feed.drain(8);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].data, b"ab");
    assert_eq!(spans[1].data, b"CD");

    // A failed atomic write publishes nothing.
    let mut feed = make_feed(4, false);
    feed.close().unwrap();
    assert_eq!(feed.write_atomic(b"nope"), Err(SpanError::Closed));
    assert!(feed.drain(8).is_empty());

    // Without auto-commit, over-chunk writes fail cleanly.
    let mut feed = make_feed(4, false);
    assert_eq!(feed.write(b"abcde"), Err(SpanError::NoSpace));
    assert!(feed.drain(8).is_empty());
    feed.write(b"abcd").unwrap();
    feed.commit().unwrap();
    assert_eq!(feed.drain(8).len(), 1);

    // Auto-commit fills chunks and publishes each.
    let mut feed = make_feed(4, true);
    feed.write(b"abcdefghij").unwrap();
    let spans = feed.drain(8);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].data, b"abcd");
    assert_eq!(spans[1].data, b"efgh");
    feed.commit().unwrap();
    let rest = feed.drain(8);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].data, b"ij");

    // Reservations gate interleaving; close ends everything.
    let mut feed = make_feed(8, false);
    feed.reserve().unwrap();
    assert_eq!(feed.reserve(), Err(SpanError::Busy));
    assert_eq!(feed.write(b"x"), Err(SpanError::Busy));
    feed.commit_reserved().unwrap();
    assert_eq!(feed.commit_reserved(), Err(SpanError::InvalidArgument));
    feed.write(b"y").unwrap();
    assert_eq!(feed.reserve(), Err(SpanError::Busy));
    feed.commit().unwrap();
    feed.close().unwrap();
    assert_eq!(feed.close(), Ok(()));
    assert_eq!(feed.write(b"z"), Err(SpanError::Closed));
    assert_eq!(feed.commit(), Err(SpanError::Closed));
    let spans = feed.drain(8);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].data, b"y");

    // Stats account the session.
    let stats = feed.stats();
    assert_eq!(stats.bytes_written, 1);
    assert_eq!(stats.spans_published, 1);
    assert_eq!(stats.spans_drained, 1);
    assert!(!feed.has_pending());
}

// ---------------------------------------------------------------------------
// SYS-006: split-scrollback formulas.
// ---------------------------------------------------------------------------

#[test]
fn req_006_split_scrollback() {
    // Starts empty.
    let scrollback = SplitScrollback::default();
    assert_eq!(scrollback.published_rows, 0);
    assert_eq!(scrollback.tail_column, 0);
    assert_eq!(scrollback.render_offset(6), 0);

    // Reset seeds pinned rows.
    let mut scrollback = SplitScrollback::default();
    scrollback.reset(6);
    assert_eq!(scrollback.published_rows, 6);
    assert_eq!(scrollback.tail_column, 0);
    assert_eq!(scrollback.render_offset(6), 6);
    scrollback.publish_snapshot_rows(1, 1, 40, false);
    assert_eq!(scrollback.render_offset(6), 6);
    assert_eq!(scrollback.tail_column, 1);

    // The offset preserves the growth-created gap until output fills it.
    let mut scrollback = SplitScrollback::default();
    scrollback.reset(7);
    scrollback.note_viewport_scroll(5);
    assert_eq!(scrollback.render_offset(2), 2);
    assert_eq!(scrollback.render_offset(7), 2);
    scrollback.note_newline();
    scrollback.note_newline();
    assert_eq!(scrollback.render_offset(7), 4);

    // Snapshot rows start at a line boundary.
    let mut scrollback = SplitScrollback::default();
    scrollback.publish_snapshot_rows(1, 4, 20, false);
    assert_eq!(scrollback.published_rows, 1);
    assert_eq!(scrollback.tail_column, 4);
    scrollback.note_newline();
    scrollback.publish_snapshot_rows(2, 8, 20, true);
    assert_eq!(scrollback.published_rows, 4);
    assert_eq!(scrollback.tail_column, 0);

    // Columns wrap against the terminal width.
    let mut scrollback = SplitScrollback::default();
    scrollback.publish_snapshot_rows(1, 6, 4, true);
    assert_eq!(scrollback.published_rows, 3);
    assert_eq!(scrollback.tail_column, 0);

    // A missing trailing newline leaves the tail open.
    let mut scrollback = SplitScrollback::default();
    scrollback.publish_snapshot_rows(1, 6, 4, false);
    assert_eq!(scrollback.published_rows, 2);
    assert_eq!(scrollback.tail_column, 2);
}

// ---------------------------------------------------------------------------
// SYS-007: level gating and sinkless silence.
// ---------------------------------------------------------------------------

#[test]
fn req_007_logging() {
    // Without a sink every level drops silently.
    let mut logger = Logger::new(LogLevel::Debug);
    logger.err("boom");
    logger.warn("careful");
    logger.info("note");
    logger.debug("detail");

    // The gate honors err < warn < info < debug.
    let received = Rc::new(RefCell::new(Vec::new()));
    let mut logger = Logger::new(LogLevel::Err);
    logger.set_sink({
        let received = Rc::clone(&received);
        move |level: LogLevel, message: &str| {
            received.borrow_mut().push((level, message.to_string()));
        }
    });
    logger.debug("hidden");
    logger.info("hidden");
    logger.warn("hidden");
    assert!(received.borrow().is_empty());
    logger.err("shown");
    assert_eq!(
        received.borrow().as_slice(),
        [(LogLevel::Err, "shown".to_string())]
    );

    logger.set_level(LogLevel::Debug);
    assert_eq!(logger.level(), LogLevel::Debug);
    logger.debug("shown now");
    assert_eq!(received.borrow().len(), 2);

    // Clearing the sink restores silence.
    logger.clear_sink();
    logger.err("dropped");
    assert_eq!(received.borrow().len(), 2);
}

// ---------------------------------------------------------------------------
// SYS-008: malformed inputs return typed errors, never panic.
// ---------------------------------------------------------------------------

#[test]
fn req_008_error_discipline() {
    use suprtui::audio::{AudioEngine, EngineOptions, TestBackend};
    use suprtui::layout::LayoutTree;
    use suprtui::media::decode as decode_image;
    use suprtui::term::ClipboardPassthrough;
    use suprtui::term::clipboard_sequence_size;
    use suprtui::term_embedded::{EmbeddedOptions, EmbeddedTerminal};

    // Every arm returns Err (a panic would fail the harness first).
    assert!(decode_image(&[]).is_err());
    assert!(decode_image(b"\x89PNG trailing junk without structure").is_err());
    assert!(decode_image(&[0u8; 128]).is_err());

    let mut audio = AudioEngine::new(EngineOptions::default(), TestBackend::standard()).unwrap();
    assert!(audio.load_wav(b"RIFFxxxx").is_err());
    assert!(audio.open_capture(0, 4).is_err());
    assert!(audio.open_capture(2, 0).is_err());
    assert!(audio.select_playback(usize::MAX).is_err());
    assert!(audio.play(u32::MAX, Default::default()).is_err());
    assert!(audio.stop_voice(u32::MAX).is_err());

    let mut tree = LayoutTree::new();
    let doomed = tree.new_leaf(&Default::default()).unwrap();
    tree.remove(doomed).unwrap();
    assert!(tree.computed(doomed).is_err());
    assert!(tree.mark_dirty(doomed).is_err());

    assert!(EmbeddedTerminal::new(EmbeddedOptions::new(0, 24)).is_err());
    let mut embedded = EmbeddedTerminal::new(EmbeddedOptions::new(8, 2)).unwrap();
    assert!(embedded.resize(0, 2).is_err());
    assert!(
        embedded
            .set_selection(
                suprtui::term_embedded::Point { x: 99, y: 0 },
                suprtui::term_embedded::Point { x: 0, y: 0 }
            )
            .is_err()
    );
    assert!(embedded.drain_responses(&mut []).is_ok());

    assert!(clipboard_sequence_size(usize::MAX, ClipboardPassthrough::Direct).is_err());

    let mut pool = LinkPool::new();
    assert!(pool.alloc(&vec![b'x'; 513]).is_err());
    assert!(pool.get(0x00FF_FFFF).is_err());

    let mut feed = SpanFeed::new(SpanFeedOptions::default()).unwrap();
    feed.close().unwrap();
    assert!(feed.write(b"late").is_err());

    let mut engine = AudioEngine::new(EngineOptions::default(), TestBackend::standard()).unwrap();
    engine.open_capture(2, 4).unwrap();
    assert!(engine.read_capture(&mut [0.0f32; 1]).is_err());
}
