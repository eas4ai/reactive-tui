//! Integration tests for the sys-small commitment (SYS-010, SYS-011).
//!
//! Vectors ported from the reference event-emitter suite; the file
//! logger has no reference unit file, so its vectors follow the
//! SYS-011 falsifier directly. Spec-literal unit tests live in
//! `src/sys.rs`.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::sys::{EventEmitter, FileLogger, LogLevel};

// ---- SYS-010: emitter ----

/// Shared firing log plus a listener factory over it.
struct Recorder {
    fired: Rc<RefCell<Vec<String>>>,
}

impl Recorder {
    fn new() -> Self {
        Self {
            fired: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn listener(&self, tag: &str) -> Box<dyn FnMut()> {
        let fired = Rc::clone(&self.fired);
        let tag = tag.to_string();
        Box::new(move || fired.borrow_mut().push(tag.clone()))
    }

    fn fired(&self) -> Vec<String> {
        self.fired.borrow().clone()
    }
}

#[test]
fn req_010_attach_fire_order() {
    let rec = Recorder::new();
    let mut emitter = EventEmitter::new();
    emitter.on("data", rec.listener("first"));
    emitter.on("data", rec.listener("second"));
    emitter.on("other", rec.listener("wrong"));
    emitter.emit("data");
    assert_eq!(rec.fired(), vec!["first".to_string(), "second".to_string()]);
}

#[test]
fn req_010_emit_without_listeners_is_silent() {
    let mut emitter = EventEmitter::new();
    emitter.emit("nothing");
    assert_eq!(0, emitter.listener_count());
}

#[test]
fn req_010_detach_never_fires_again() {
    let rec = Recorder::new();
    let mut emitter = EventEmitter::new();
    let keep = emitter.on("data", rec.listener("keep"));
    let drop = emitter.on("data", rec.listener("drop"));
    assert!(emitter.off(drop));
    assert!(!emitter.off(drop));
    emitter.emit("data");
    assert_eq!(rec.fired(), vec!["keep".to_string()]);
    assert_eq!(1, emitter.listener_count());
    assert!(emitter.off(keep));
    assert_eq!(0, emitter.listener_count());
}

#[test]
fn req_010_separate_events_stay_separate() {
    let rec = Recorder::new();
    let mut emitter = EventEmitter::new();
    emitter.on("a", rec.listener("a"));
    emitter.on("b", rec.listener("b"));
    emitter.emit("b");
    emitter.emit("a");
    assert_eq!(rec.fired(), vec!["b".to_string(), "a".to_string()]);
}

// ---- SYS-011: file logger ----

fn temp_log(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("suprtui-{name}-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn req_011_level_gating() {
    let path = temp_log("gating");
    let logger = FileLogger::new(&path, LogLevel::Warn);
    logger.debug("d1").unwrap();
    logger.info("i1").unwrap();
    logger.warn("w1").unwrap();
    logger.err("e1").unwrap();
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(!body.contains("d1"));
    assert!(!body.contains("i1"));
    assert!(body.contains("w1"));
    assert!(body.contains("e1"));
    assert_eq!(2, body.lines().count());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn req_011_appends_across_calls() {
    let path = temp_log("append");
    let logger = FileLogger::new(&path, LogLevel::Debug);
    logger.info("one").unwrap();
    logger.info("two").unwrap();
    let second = FileLogger::new(&path, LogLevel::Debug);
    second.info("three").unwrap();
    let body = std::fs::read_to_string(&path).unwrap();
    assert_eq!(3, body.lines().count());
    assert!(body.contains("three"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn req_011_io_failure_reports() {
    // A directory is not an appendable file: must report, not panic.
    let dir = std::env::temp_dir().join(format!("suprtui-dir-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let logger = FileLogger::new(&dir, LogLevel::Debug);
    assert!(logger.info("lost").is_err());
    let _ = std::fs::remove_dir(&dir);
}
