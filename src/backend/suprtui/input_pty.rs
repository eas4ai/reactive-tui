//! INP-001 to INP-006: an App on the default backend receives real terminal
//! input on a pseudo-terminal.
//!
//! Each test runs a copy of this test binary on a new pseudo-terminal of
//! [`COLUMNS`] by [`ROWS`], where `smoke_input_pty_child` builds an App on
//! [`SuprTuiBackend::new`] for one scenario. The test writes SGR mouse
//! reports, bracketed pastes and keys to the terminal, reads everything the
//! App writes, and reads the log in which the scenario's elements record each
//! event they receive. Without the scenario variable the child returns at
//! once, so a plain `cargo test` run passes it.

use super::SuprTuiBackend;
use crate::app::{App, RootComponent};
use crate::builder::core::div;
use crate::component::{Component, Element, Props};
use crate::error::{ReactiveError, Result};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyCode, KeyModifiers, Position, WheelDelta};
use crate::terminal::owned_pty::PtyChild;
use crate::widgets::display::table::{Table, TableColumn, TableProps, TableRow};
use crate::widgets::display::{TreeBuilder, TreeNode};
use crate::widgets::layout::ScrollViewBuilder;
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Names the scenario the copy runs.
const SCENARIO: &str = "REACTIVE_TUI_INPUT_PTY_SCENARIO";
/// The file the copy's elements append their events to.
const LOG: &str = "REACTIVE_TUI_INPUT_PTY_LOG";
/// The child test's path as libtest prints it.
const CHILD: &str = "backend::suprtui::input_pty::smoke_input_pty_child";
const COLUMNS: u16 = 80;
const ROWS: u16 = 24;
/// Bounds a hung copy; a copy starts and answers in well under a second.
const DEADLINE: Duration = Duration::from_secs(30);
const CURSOR_QUERY: &[u8] = b"\x1b[6n";
const CURSOR_REPORT: &[u8] = b"\x1b[1;1R";
/// The DECSET modes INP-001 names, in the order crossterm writes them.
const MODES: [&str; 6] = ["1000", "1002", "1003", "1015", "1006", "2004"];

// ---------------------------------------------------------------------------
// The copy: an App on the default backend for one scenario.

/// Runs one scenario when the pseudo-terminal test starts this binary.
#[test]
fn smoke_input_pty_child() {
    let Some(scenario) = std::env::var_os(SCENARIO) else {
        eprintln!("SKIP: run by the INP pseudo-terminal tests with {SCENARIO} set");
        return;
    };
    let app = App::builder()
        .backend(SuprTuiBackend::new().expect("the default backend on the pseudo-terminal"))
        .root(Scenario(scenario.to_string_lossy().into_owned()))
        .quit_key(KeyCode::Char('q'), KeyModifiers::empty())
        .build()
        .expect("the scenario's App");
    // An error exit is one of the scenarios; its bytes are what the parent checks.
    let _ = app.run();
}

fn log(line: String) {
    static FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
    let file = FILE.get_or_init(|| {
        let path = std::env::var_os(LOG).expect("the log path");
        Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("the event log"),
        )
    });
    let mut file = file.lock().unwrap();
    writeln!(file, "{line}").expect("an event log line");
}

/// One log line: kind, button, cell, Shift Alt Ctrl, and the wheel's lines.
fn describe(event: &Event) -> Option<String> {
    match event {
        Event::Mouse(mouse) => {
            let cell = match mouse.position {
                Position::Cell { x, y } => format!("{x},{y}"),
                other => format!("{other:?}"),
            };
            let mods = mouse.modifiers;
            let flags = format!(
                "{}{}{}",
                if mods.shift { 'S' } else { '-' },
                if mods.alt { 'A' } else { '-' },
                if mods.ctrl { 'C' } else { '-' }
            );
            let wheel = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                Some(WheelDelta::Lines { x, y }) => format!(" wheel {x},{y}"),
                Some(other) => format!(" wheel {other:?}"),
                None => String::new(),
            };
            Some(format!(
                "{:?} {:?} {cell} {flags}{wheel}",
                mouse.kind, mouse.button
            ))
        }
        Event::Paste(paste) => Some(format!("Paste {:?}", paste.content)),
        Event::Key(key) => Some(format!("Key {:?}", key.code)),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ProbeProps {
    name: String,
    class: String,
}

impl Props for ProbeProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// An element that logs every mouse event it receives, in its own cells.
struct Probe;

impl Component for Probe {
    type Props = ProbeProps;
    type State = ();

    fn new(_props: ProbeProps) -> Self {
        Probe
    }

    fn render(&self, props: &ProbeProps, _state: &()) -> Element {
        div().class(&props.class).text(&props.name).build()
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut ProbeProps,
        _state: &mut (),
    ) -> EventResult {
        match (event, describe(event)) {
            (Event::Mouse(_), Some(line)) => {
                log(format!("{} {line}", props.name));
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

fn probe(name: &str, class: &str) -> Element {
    Element::typed::<Probe>(ProbeProps {
        name: name.into(),
        class: class.into(),
    })
}

fn rows(prefix: &str, count: usize) -> Vec<Element> {
    (0..count)
        .map(|index| {
            div()
                .class("h-1")
                .text(&format!("{prefix}-{index}"))
                .build()
        })
        .collect()
}

/// An outer scroll view of twelve rows whose second to fifth rows hold a
/// scroll view, a table or a tree of twelve rows shown four at a time, or a
/// one-line text field.
fn wheel_scene(kind: &str) -> Element {
    let inner = match kind {
        "wheel-view" => ScrollViewBuilder::new(
            div()
                .class("flex flex-col")
                .children(rows("inner", 12))
                .build(),
        )
        .scroll_y(true)
        .viewport_height(4)
        .smooth_scroll(false)
        .show_scrollbars(false)
        .render(),
        "wheel-table" => {
            let columns = vec![TableColumn::new("Row", "row")];
            let body = (0..12)
                .map(|index| {
                    TableRow::new(&format!("r{index}")).with_cell("row", &format!("inner-{index}"))
                })
                .collect();
            Table::with_props(TableProps {
                scrollable: true,
                max_height: Some(4),
                ..Table::with_rows(Table::with_columns(columns), body)
            })
        }
        // A one-line text field, which has nothing to scroll.
        "wheel-input" => crate::builder::text_input().value("field").build(),
        _ => {
            let mut root = TreeNode::new("root", "inner-root").expanded(true);
            for index in 0..12 {
                root = root.add_child(TreeNode::new(format!("n{index}"), format!("inner-{index}")));
            }
            TreeBuilder::new()
                .root(root)
                .scrollable(true)
                .max_height(4)
                .render()
        }
    };
    let mut content = vec![
        div().class("h-1").text("OUTER-TOP").build(),
        div().class("h-4").children(vec![inner]).build(),
    ];
    content.extend(rows("outer", 30));
    ScrollViewBuilder::new(div().class("flex flex-col").children(content).build())
        .scroll_y(true)
        .viewport_height(12)
        .smooth_scroll(false)
        .show_scrollbars(false)
        .render()
}

struct Scenario(String);

impl RootComponent for Scenario {
    fn render(&self) -> Element {
        match self.0.as_str() {
            "pair" => div()
                .class("flex flex-row w-full h-full")
                .children(vec![
                    probe("LEFT", "flex-1 h-full"),
                    probe("RIGHT", "flex-1 h-full"),
                ])
                .build(),
            "wheel-view" | "wheel-table" | "wheel-tree" | "wheel-input" => wheel_scene(&self.0),
            _ => probe("PROBE", "w-full h-full"),
        }
    }

    fn try_handle_event(&mut self, event: &Event) -> Result<EventResult> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('e') => {
                    return Err(ReactiveError::invalid_state("the error-exit scenario"))
                }
                KeyCode::Char('p') => panic!("the panic-exit scenario"),
                _ => {}
            }
        }
        if let Some(line) = describe(event) {
            log(format!("root {line}"));
        }
        Ok(EventResult::Ignored)
    }
}

// ---------------------------------------------------------------------------
// The test side: a copy on a pseudo-terminal, its output and its event log.

struct Session {
    child: PtyChild,
    output: Vec<u8>,
    answered: usize,
    log: PathBuf,
    status: Option<ExitStatus>,
}

impl Session {
    /// Starts `scenario` and waits for `marker` in its first frame.
    fn start(scenario: &str, marker: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let log = std::env::temp_dir().join(format!(
            "reactive-tui-input-pty-{}-{scenario}-{}.log",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&log);
        std::fs::write(&log, "").expect("an empty event log");
        let mut command = Command::new(std::env::current_exe().expect("the test binary's path"));
        command
            .args([
                CHILD,
                "--exact",
                "--nocapture",
                "--test-threads=1",
                "--color=never",
            ])
            .env(SCENARIO, scenario)
            .env(LOG, &log);
        let child = PtyChild::spawn(command, COLUMNS, ROWS).expect("a pseudo-terminal");
        let mut session = Session {
            child,
            output: Vec::new(),
            answered: 0,
            log,
            status: None,
        };
        session.until(
            |session| session.text().contains(marker),
            &format!("{scenario}'s first frame"),
        );
        session
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }

    /// Reads what the copy wrote and answers its cursor queries.
    fn pump(&mut self) {
        if self.status.is_none() {
            self.status = self.child.try_wait().expect("the copy's status");
        }
        let mut buffer = [0; 8192];
        loop {
            match self.child.master.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => self.output.extend_from_slice(&buffer[..read]),
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock
                        || error.raw_os_error() == Some(libc::EIO) =>
                {
                    break
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("reading the pseudo-terminal failed: {error}"),
            }
        }
        let queries = self
            .output
            .windows(CURSOR_QUERY.len())
            .filter(|window| *window == CURSOR_QUERY)
            .count();
        while self.answered < queries && self.status.is_none() {
            self.send(CURSOR_REPORT);
            self.answered += 1;
        }
    }

    fn until(&mut self, done: impl Fn(&Self) -> bool, what: &str) {
        let deadline = Instant::now() + DEADLINE;
        loop {
            self.pump();
            if done(self) {
                return;
            }
            if self.status.is_some() && !done(self) {
                self.pump();
                if done(self) {
                    return;
                }
                panic!(
                    "the copy exited before {what} ({:?}):\n{}",
                    self.status,
                    self.text()
                );
            }
            if Instant::now() > deadline {
                panic!("no {what} within {DEADLINE:?}:\n{}", self.text());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Reads for `time`, so the copy handles what it was sent and repaints.
    fn settle(&mut self, time: Duration) {
        let end = Instant::now() + time;
        while Instant::now() < end {
            self.pump();
            std::thread::sleep(Duration::from_millis(5));
        }
        self.pump();
    }

    fn send(&mut self, mut bytes: &[u8]) {
        let deadline = Instant::now() + DEADLINE;
        while !bytes.is_empty() {
            match self.child.master.write(bytes) {
                Ok(written) => bytes = &bytes[written..],
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "the pseudo-terminal stopped accepting input"
                    );
                    self.pump();
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("writing the pseudo-terminal failed: {error}"),
            }
        }
    }

    fn events(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .expect("the event log")
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// Waits for the copy to exit.
    fn exit(&mut self) {
        self.until(|session| session.status.is_some(), "the copy's exit");
        self.settle(Duration::from_millis(50));
    }

    /// The screen the copy's output leaves on a terminal of this size.
    fn screen(&self) -> Vec<String> {
        let mut parser = vt100::Parser::new(ROWS, COLUMNS, 0);
        parser.process(&self.output);
        parser
            .screen()
            .contents()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.status.is_none() {
            let _ = self.child.stop();
        }
        let _ = std::fs::remove_file(&self.log);
    }
}

/// An SGR mouse report at a 0-based cell: `press` ends it with `M`, a release with `m`.
fn sgr(code: u8, x: u16, y: u16, press: bool) -> Vec<u8> {
    format!(
        "\x1b[<{code};{};{}{}",
        x + 1,
        y + 1,
        if press { 'M' } else { 'm' }
    )
    .into_bytes()
}

fn click(session: &mut Session, x: u16, y: u16) {
    session.send(&sgr(0, x, y, true));
    session.send(&sgr(0, x, y, false));
}

fn set(mode: &str) -> String {
    format!("\x1b[?{mode}h")
}

fn reset(mode: &str) -> String {
    format!("\x1b[?{mode}l")
}

// ---------------------------------------------------------------------------
// INP-001: the modes are set before the first frame and reset on every exit.

fn check_modes(exit_key: char, how: &str) {
    let mut session = Session::start("modes", "PROBE");
    let text = session.text();
    let frame = text.find("PROBE").unwrap();
    let missing: Vec<&str> = MODES
        .iter()
        .copied()
        .filter(|mode| text.find(&set(mode)).is_none_or(|at| at > frame))
        .collect();
    assert!(
        missing.is_empty(),
        "INP-001: before its first frame the default backend set none of the modes {missing:?}"
    );
    session.send(exit_key.to_string().as_bytes());
    session.exit();
    let text = session.text();
    let left_set: Vec<&str> = MODES
        .iter()
        .copied()
        .filter(
            |mode| match (text.rfind(&set(mode)), text.rfind(&reset(mode))) {
                (Some(on), Some(off)) => on > off,
                (Some(_), None) => true,
                _ => false,
            },
        )
        .collect();
    assert!(
        left_set.is_empty(),
        "INP-001: after {how} the last write of the modes {left_set:?} set them"
    );
}

#[test]
fn inp_001_modes_on_and_off_after_a_normal_exit() {
    check_modes('q', "a normal exit");
}

#[test]
fn inp_001_modes_on_and_off_after_an_error_exit() {
    check_modes('e', "an error exit");
}

#[test]
fn inp_001_modes_on_and_off_after_a_panic() {
    check_modes('p', "a panic");
}

// ---------------------------------------------------------------------------
// INP-002: every report reaches the App as one event that keeps what it says.

#[test]
fn inp_002_reports_and_pastes_keep_what_they_say() {
    let mut session = Session::start("full", "PROBE");
    let mut expected = Vec::new();
    let mut at = 0u16;
    let mut next = || {
        at += 1;
        (at % 70, 1 + at / 70)
    };
    for (bits, flags) in [(0u8, "---"), (4, "S--"), (8, "-A-"), (16, "--C")] {
        for (button, name) in [(0u8, "Left"), (1, "Middle"), (2, "Right")] {
            let (x, y) = next();
            session.send(&sgr(button | bits, x, y, true));
            expected.push(format!("PROBE Down {name} {x},{y} {flags}"));
            let (x, y) = next();
            session.send(&sgr(32 | button | bits, x, y, true));
            expected.push(format!("PROBE Drag {name} {x},{y} {flags}"));
            session.send(&sgr(button | bits, x, y, false));
            expected.push(format!("PROBE Up {name} {x},{y} {flags}"));
            session.settle(Duration::from_millis(20));
        }
        let (x, y) = next();
        session.send(&sgr(35 | bits, x, y, true));
        expected.push(format!("PROBE Move None {x},{y} {flags}"));
        session.settle(Duration::from_millis(20));
        for (code, delta) in [(64u8, "0,-1"), (65, "0,1"), (66, "-1,0"), (67, "1,0")] {
            let (x, y) = next();
            session.send(&sgr(code | bits, x, y, true));
            expected.push(format!("PROBE Wheel None {x},{y} {flags} wheel {delta}"));
            session.settle(Duration::from_millis(20));
        }
    }
    session.send(b"\x1b[200~first line\nsecond line\x1b[201~");
    session.settle(Duration::from_millis(300));
    let events = session.events();
    let missing: Vec<&String> = expected
        .iter()
        .filter(|line| !events.contains(line))
        .collect();
    assert!(
        missing.is_empty(),
        "INP-002: {} of {} reports did not arrive as sent, for example {:?}; received {:?}",
        missing.len(),
        expected.len(),
        missing.iter().take(3).collect::<Vec<_>>(),
        events.iter().take(8).collect::<Vec<_>>()
    );
    let pastes: Vec<&String> = events
        .iter()
        .filter(|line| line.starts_with("root Paste"))
        .collect();
    let keys: Vec<&String> = events
        .iter()
        .filter(|line| line.starts_with("root Key"))
        .collect();
    assert!(
        pastes == [&"root Paste \"first line\\nsecond line\"".to_owned()] && keys.is_empty(),
        "INP-002: the paste arrived as {pastes:?} and the keys {keys:?}"
    );
}

// ---------------------------------------------------------------------------
// INP-003: a drag and its release stay with the element that got the press.

#[test]
fn inp_003_drags_and_the_release_stay_with_the_pressed_element() {
    let mut session = Session::start("pair", "RIGHT");
    session.send(&sgr(0, 5, 2, true));
    for x in [20, 45, 60] {
        session.settle(Duration::from_millis(40));
        session.send(&sgr(32, x, 2, true));
    }
    session.settle(Duration::from_millis(40));
    session.send(&sgr(0, 60, 2, false));
    session.settle(Duration::from_millis(300));
    let events = session.events();
    let to_right: Vec<&String> = events
        .iter()
        .filter(|line| line.starts_with("RIGHT Drag") || line.starts_with("RIGHT Up"))
        .collect();
    let left_drags = events
        .iter()
        .filter(|line| line.starts_with("LEFT Drag"))
        .count();
    let left_release = events.iter().any(|line| line.starts_with("LEFT Up"));
    assert!(
        to_right.is_empty() && left_drags == 3 && left_release,
        "INP-003: the right element received {to_right:?}; the pressed left element received {left_drags} of 3 drags and {} release; log {events:?}",
        if left_release { "its" } else { "no" }
    );
}

// ---------------------------------------------------------------------------
// INP-004: releases make Click, DoubleClick and TripleClick events.

fn clicks(events: &[String], name: &str, x: u16) -> Vec<String> {
    events
        .iter()
        .filter(|line| line.starts_with(&format!("{name} ")) && line.contains(&format!(" {x},")))
        .filter_map(|line| line.split(' ').nth(1).map(str::to_owned))
        .filter(|kind| kind.ends_with("Click"))
        .collect()
}

#[test]
fn inp_004_releases_make_counted_clicks() {
    let mut session = Session::start("pair", "RIGHT");
    click(&mut session, 5, 2);
    session.settle(Duration::from_millis(700));
    for _ in 0..2 {
        click(&mut session, 10, 2);
        session.settle(Duration::from_millis(100));
    }
    session.settle(Duration::from_millis(700));
    for _ in 0..3 {
        click(&mut session, 15, 2);
        session.settle(Duration::from_millis(100));
    }
    session.settle(Duration::from_millis(700));
    click(&mut session, 20, 2);
    session.settle(Duration::from_millis(600));
    click(&mut session, 20, 2);
    session.settle(Duration::from_millis(700));
    session.send(&sgr(0, 25, 2, true));
    session.send(&sgr(0, 45, 2, false));
    session.settle(Duration::from_millis(300));
    let events = session.events();
    let single = clicks(&events, "LEFT", 5);
    let double = clicks(&events, "LEFT", 10);
    let triple = clicks(&events, "LEFT", 15);
    let slow = clicks(&events, "LEFT", 20);
    let across: Vec<String> = [("LEFT", 25), ("LEFT", 45), ("RIGHT", 5)]
        .into_iter()
        .flat_map(|(name, x)| clicks(&events, name, x))
        .collect();
    assert!(
        single == ["Click"]
            && double.last().map(String::as_str) == Some("DoubleClick")
            && triple.last().map(String::as_str) == Some("TripleClick")
            && !slow.iter().any(|kind| kind == "DoubleClick")
            && slow.iter().filter(|kind| *kind == "Click").count() == 2
            && across.is_empty(),
        "INP-004: one click gave {single:?}, two quick clicks {double:?}, three {triple:?}, two slow ones {slow:?}, and a press released over another element {across:?}"
    );
}

// ---------------------------------------------------------------------------
// INP-005: the wheel goes to what is under the pointer and passes on at an edge.

fn outer_top(session: &Session) -> bool {
    session
        .screen()
        .first()
        .is_some_and(|row| row.contains("OUTER-TOP"))
}

fn check_wheel(scene: &str) {
    let mut session = Session::start(scene, "OUTER-TOP");
    session.send(&sgr(65, 5, 2, true));
    session.settle(Duration::from_millis(300));
    assert!(
        outer_top(&session),
        "INP-005: in {scene}, one wheel step over an inner widget that can still scroll scrolled the outer view:\n{}",
        session.screen().join("\n")
    );
    for _ in 0..20 {
        session.send(&sgr(65, 5, 2, true));
        session.settle(Duration::from_millis(30));
    }
    session.settle(Duration::from_millis(300));
    assert!(
        !outer_top(&session),
        "INP-005: in {scene}, wheel steps past the inner widget's last row left the outer view unscrolled:\n{}",
        session.screen().join("\n")
    );
}

#[test]
fn inp_005_the_wheel_passes_from_a_scroll_view_at_its_edge() {
    check_wheel("wheel-view");
}

#[test]
fn inp_005_the_wheel_passes_from_a_table_at_its_edge() {
    check_wheel("wheel-table");
}

#[test]
fn inp_005_the_wheel_passes_from_a_tree_at_its_edge() {
    check_wheel("wheel-tree");
}

/// The wheel over an element that cannot scroll, a one-line text field,
/// bubbles to the view around it.
#[test]
fn inp_005_the_wheel_passes_over_a_text_field() {
    let mut session = Session::start("wheel-input", "OUTER-TOP");
    for _ in 0..4 {
        session.send(&sgr(65, 5, 2, true));
        session.settle(Duration::from_millis(30));
    }
    session.settle(Duration::from_millis(300));
    assert!(
        !outer_top(&session),
        "INP-005: wheel steps over a text field left the outer view unscrolled:\n{}",
        session.screen().join("\n")
    );
}

#[test]
fn inp_005_shift_with_the_wheel_scrolls_sideways() {
    let mut session = Session::start("wheel-view", "OUTER-TOP");
    for _ in 0..3 {
        session.send(&sgr(65 | 4, 5, 8, true));
        session.settle(Duration::from_millis(30));
    }
    session.settle(Duration::from_millis(300));
    assert!(
        outer_top(&session),
        "INP-005: Shift with the wheel down scrolled the outer view vertically:\n{}",
        session.screen().join("\n")
    );
}

// ---------------------------------------------------------------------------
// INP-006: waiting motion reports are merged; presses keep their place.

#[test]
fn inp_006_waiting_motion_is_merged() {
    let mut session = Session::start("full", "PROBE");
    let mut burst = Vec::new();
    for index in 0..100u16 {
        burst.extend(sgr(35, index % 70, 3, true));
    }
    burst.extend(sgr(0, 5, 6, true));
    burst.extend(sgr(0, 5, 6, false));
    session.send(&burst);
    session.settle(Duration::from_millis(500));
    let events = session.events();
    let press = events
        .iter()
        .position(|line| line.starts_with("PROBE Down Left 5,6"));
    let moves = events
        .iter()
        .take(press.unwrap_or(events.len()))
        .filter(|line| line.starts_with("PROBE Move"))
        .count();
    let late_move = press.is_some_and(|press| {
        events[press..]
            .iter()
            .any(|line| line.starts_with("PROBE Move None 29,3"))
    });
    assert!(
        press.is_some() && moves <= 2 && !late_move,
        "INP-006: 100 waiting motion reports reached the App as {moves} Move events before the press ({})",
        if press.is_some() { "the press arrived" } else { "the press was lost" }
    );
}
