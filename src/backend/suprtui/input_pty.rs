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
/// Set in the job the suspend scenario's shell starts.
const INNER: &str = "REACTIVE_TUI_INPUT_PTY_INNER";
/// The child test's path as libtest prints it.
const CHILD: &str = "backend::suprtui::input_pty::smoke_input_pty_child";
const COLUMNS: u16 = 80;
const ROWS: u16 = 24;
/// Bounds a hung copy; a copy starts and answers in well under a second.
const DEADLINE: Duration = Duration::from_secs(30);
const CURSOR_QUERY: &[u8] = b"\x1b[6n";
const CURSOR_REPORT: &[u8] = b"\x1b[1;1R";
/// The Kitty keyboard protocol query and a reply that reports it.
const KEYBOARD_QUERY: &[u8] = b"\x1b[?u";
const KEYBOARD_REPLY: &[u8] = b"\x1b[?0u";
/// The background color query, terminated by BEL or ST.
const BACKGROUND_QUERY: &[u8] = b"\x1b]11;?";
/// The primary device attributes query and a VT220-style reply.
const ATTRIBUTES_QUERY: &[u8] = b"\x1b[c";
const ATTRIBUTES_REPLY: &[u8] = b"\x1b[?62;22c";
/// The Kitty keyboard flags INP-007 names: 1, 8 and 16.
const KITTY_PUSH: &str = "\x1b[>25u";
const KITTY_POP: &str = "\x1b[<1u";
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
    run_scenario(&scenario.to_string_lossy());
}

/// Runs one scenario: the suspend scenario as a shell around a job that runs
/// it again, every other one as an App on this terminal.
fn run_scenario(scenario: &str) {
    if scenario == "suspend" && std::env::var_os(INNER).is_none() {
        return job_shell();
    }
    if scenario == "theme-set" {
        crate::theme::Theme::set_active(crate::theme::high_contrast_theme());
    }
    let app = App::builder()
        .backend(SuprTuiBackend::new().expect("the default backend on the pseudo-terminal"))
        .root(Scenario(scenario.to_owned()))
        .quit_key(KeyCode::Char('q'), KeyModifiers::empty())
        .build()
        .expect("the scenario's App");
    // An error exit is one of the scenarios; its bytes are what the parent checks.
    let _ = app.run();
    // What the shell would read next; mouse_reports_queued_at_exit_are_discarded
    // checks it after its exit.
    log(format!("LEFTOVER {}", queued_input_bytes()));
}

/// Runs the scenario again as a job in its own foreground process group, as
/// a shell does: the copy leads its own session, and a stop signal sent to
/// the orphaned group of a session leader is discarded. When the job stops
/// this logs STOPPED with the terminal's line mode, takes the terminal back,
/// and after 100 ms hands it over again, continues the job and logs
/// CONTINUED.
fn job_shell() {
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;
    let tty = std::fs::File::open("/dev/tty").expect("the controlling terminal");
    let fd = tty.as_raw_fd();
    let mut command = Command::new(std::env::current_exe().expect("the test binary's path"));
    command
        .args([
            CHILD,
            "--exact",
            "--nocapture",
            "--test-threads=1",
            "--color=never",
        ])
        .env(INNER, "1");
    // SAFETY: only async-signal-safe calls between fork and exec; the job
    // makes its own group the terminal's foreground before it reads.
    unsafe {
        command.pre_exec(|| {
            libc::signal(libc::SIGTTOU, libc::SIG_IGN);
            if libc::setpgid(0, 0) < 0 || libc::tcsetpgrp(0, libc::getpid()) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            libc::signal(libc::SIGTTOU, libc::SIG_DFL);
            Ok(())
        });
    }
    #[allow(clippy::zombie_processes)] // waitpid below reaps it
    let job = command.spawn().expect("the job");
    let pid = job.id() as libc::pid_t;
    // SAFETY: waitpid, tcsetpgrp, tcgetattr and kill on this process's own
    // job and terminal; SIGTTOU is ignored so the shell may take the terminal.
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        loop {
            let mut status = 0;
            if libc::waitpid(pid, &mut status, libc::WUNTRACED) < 0 || !libc::WIFSTOPPED(status) {
                break;
            }
            libc::tcsetpgrp(fd, libc::getpgrp());
            let mut termios = std::mem::zeroed::<libc::termios>();
            libc::tcgetattr(fd, &mut termios);
            log(format!(
                "STOPPED canonical={}",
                termios.c_lflag & libc::ICANON != 0
            ));
            std::thread::sleep(Duration::from_millis(100));
            libc::tcsetpgrp(fd, pid);
            libc::kill(-pid, libc::SIGCONT);
            log("CONTINUED".to_owned());
        }
    }
}

/// The bytes still queued on the controlling terminal, which a shell would
/// read next; read without waiting, with the line discipline off meanwhile.
fn queued_input_bytes() -> usize {
    use std::os::fd::AsRawFd;
    let tty = std::fs::File::open("/dev/tty").expect("the controlling terminal");
    let fd = tty.as_raw_fd();
    // SAFETY: termios calls and reads on a descriptor that stays open; the
    // saved settings are put back before returning.
    unsafe {
        let mut saved = std::mem::zeroed::<libc::termios>();
        assert_eq!(libc::tcgetattr(fd, &mut saved), 0, "the terminal settings");
        let mut raw = saved;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 0;
        assert_eq!(libc::tcsetattr(fd, libc::TCSANOW, &raw), 0, "raw reads");
        let mut count = 0;
        let mut buffer = [0u8; 4096];
        loop {
            let read = libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len());
            if read <= 0 {
                break;
            }
            count += read as usize;
        }
        libc::tcsetattr(fd, libc::TCSANOW, &saved);
        count
    }
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
        Event::Key(key) => {
            let mods = key.modifiers;
            Some(format!(
                "Key {:?} {}{}{}{}",
                key.code,
                if mods.shift { 'S' } else { '-' },
                if mods.ctrl { 'C' } else { '-' },
                if mods.alt { 'A' } else { '-' },
                if mods.meta { 'M' } else { '-' }
            ))
        }
        Event::Focus(focus) => Some(format!("Focus {:?}", focus.kind)),
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

/// An element that logs every mouse event it receives, in its own cells, and
/// the keys and focus events that reach it, which it passes on.
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
            (Event::Key(_) | Event::Focus(_), Some(line)) => {
                log(format!("{} {line}", props.name));
                EventResult::Ignored
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

/// A scroll view that scrolls both ways, whose first row is 300 cells wide.
fn wide_scene() -> Element {
    let mut content = vec![div()
        .class("w-300 h-1")
        .text(&format!("WIDE-START{}", "-".repeat(290)))
        .build()];
    content.extend(rows("outer", 30));
    ScrollViewBuilder::new(div().class("flex flex-col").children(content).build())
        .scroll_x(true)
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
            "wheel-wide" => wide_scene(),
            "focus" => probe("FOCUSED", "w-full h-full").auto_focus(),
            "theme" | "theme-set" => {
                log(format!("THEME {}", crate::theme::Theme::active().name));
                probe("PROBE", "w-full h-full")
            }
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
                // Stop reading input for a while, then exit with an error.
                KeyCode::Char('s') => {
                    std::thread::sleep(Duration::from_millis(300));
                    return Err(ReactiveError::invalid_state("the slow error-exit scenario"));
                }
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

/// What the fake terminal answers besides cursor position queries: the
/// device attributes unless it is silent, the keyboard protocol and the
/// background color when set, each `hold` after it first sees a query.
#[derive(Clone, Copy)]
struct Terminal {
    kitty: bool,
    background: Option<&'static str>,
    silent: bool,
    hold: Duration,
}

impl Terminal {
    /// Answers only the device attributes query, at once.
    const PLAIN: Terminal = Terminal {
        kitty: false,
        background: None,
        silent: false,
        hold: Duration::ZERO,
    };
    /// Also reports the Kitty keyboard protocol.
    const KITTY: Terminal = Terminal {
        kitty: true,
        ..Terminal::PLAIN
    };
}

struct Session {
    child: PtyChild,
    output: Vec<u8>,
    answered: usize,
    log: PathBuf,
    status: Option<ExitStatus>,
    terminal: Terminal,
    /// Replies sent to the keyboard, background and attributes queries.
    replied: [usize; 3],
    /// When the copy's first startup query was seen.
    queried_at: Option<Instant>,
}

impl Session {
    /// Starts `scenario` on a terminal that answers only the device
    /// attributes query, and waits for `marker` in its first frame.
    fn start(scenario: &str, marker: &str) -> Self {
        Session::start_with(scenario, marker, Terminal::PLAIN)
    }

    /// Starts `scenario` on `terminal` and waits for `marker` in its first frame.
    fn start_with(scenario: &str, marker: &str, terminal: Terminal) -> Self {
        let mut session = Session::spawn(scenario, terminal);
        session.until(
            |session| session.text().contains(marker),
            &format!("{scenario}'s first frame"),
        );
        session
    }

    /// Starts `scenario` on `terminal` without waiting.
    fn spawn(scenario: &str, terminal: Terminal) -> Self {
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
        Session {
            child,
            output: Vec::new(),
            answered: 0,
            log,
            status: None,
            terminal,
            replied: [0; 3],
            queried_at: None,
        }
    }

    fn count(&self, pattern: &[u8]) -> usize {
        self.output
            .windows(pattern.len())
            .filter(|window| *window == pattern)
            .count()
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }

    /// Reads what the copy wrote and answers its queries.
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
        let queries = self.count(CURSOR_QUERY);
        while self.answered < queries && self.status.is_none() {
            self.send(CURSOR_REPORT);
            self.answered += 1;
        }
        let asked = [
            self.count(KEYBOARD_QUERY),
            self.count(BACKGROUND_QUERY),
            self.count(ATTRIBUTES_QUERY),
        ];
        if self.queried_at.is_none() && asked.iter().any(|count| *count > 0) {
            self.queried_at = Some(Instant::now());
        }
        let due = self
            .queried_at
            .is_some_and(|at| at.elapsed() >= self.terminal.hold);
        if !due || self.status.is_some() {
            return;
        }
        // In the order a terminal answers them: the order they were asked.
        for (index, reply) in [
            self.terminal.kitty.then(|| KEYBOARD_REPLY.to_vec()),
            self.terminal
                .background
                .map(|color| format!("\x1b]11;{color}\x1b\\").into_bytes()),
            (!self.terminal.silent).then(|| ATTRIBUTES_REPLY.to_vec()),
        ]
        .into_iter()
        .enumerate()
        {
            while self.replied[index] < asked[index] {
                if let Some(reply) = &reply {
                    self.send(reply);
                }
                self.replied[index] += 1;
            }
        }
    }

    fn until(&mut self, done: impl Fn(&Self) -> bool, what: &str) {
        self.until_within(done, what, DEADLINE);
    }

    fn until_within(&mut self, done: impl Fn(&Self) -> bool, what: &str, within: Duration) {
        let deadline = Instant::now() + within;
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
                panic!("no {what} within {within:?}:\n{}", self.text());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Waits up to 5 s until `count` logged events match, then 100 ms for
    /// anything else the same input makes. It does not fail on its own: the
    /// test's assertion says what arrived.
    fn wait_for(&mut self, matches: impl Fn(&str) -> bool, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline
            && self.events().iter().filter(|line| matches(line)).count() < count
        {
            self.pump();
            std::thread::sleep(Duration::from_millis(5));
        }
        self.settle(Duration::from_millis(100));
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

/// Mouse reports that arrive while the App has stopped reading, just before
/// it exits, are dropped at exit instead of reaching the shell as text. The
/// burst stays under 1 KiB, the input queue of a macOS pseudo-terminal: the
/// rest of a longer write would land after the discard, which a terminal told
/// to stop reporting would not send.
#[test]
fn mouse_reports_queued_at_exit_are_discarded() {
    let mut session = Session::start("leftover", "PROBE");
    session.send(b"s");
    session.settle(Duration::from_millis(50));
    let mut burst = Vec::new();
    for index in 0..60u16 {
        burst.extend(sgr(35, index % 70, 3, true));
    }
    session.send(&burst);
    session.exit();
    let leftover: Vec<String> = session
        .events()
        .into_iter()
        .filter(|line| line.starts_with("LEFTOVER"))
        .collect();
    assert_eq!(
        leftover,
        ["LEFTOVER 0"],
        "mouse reports were left queued for the shell after the App exited"
    );
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
    session.wait_for(|line| line.starts_with("root Paste"), 1);
    let events = session.events();
    // Every report and nothing else, in order; hover changes and the clicks
    // the App makes from releases are not reports.
    let received: Vec<&String> = events
        .iter()
        .filter(|line| line.starts_with("PROBE "))
        .filter(|line| {
            let kind = line.split(' ').nth(1).unwrap_or_default();
            !matches!(kind, "Enter" | "Leave") && !kind.ends_with("Click")
        })
        .collect();
    let missing: Vec<&String> = expected
        .iter()
        .filter(|line| !events.contains(line))
        .collect();
    let first_difference = received
        .iter()
        .zip(&expected)
        .position(|(got, want)| *got != want);
    assert!(
        missing.is_empty() && received.len() == expected.len() && first_difference.is_none(),
        "INP-002: {} of {} reports did not arrive as sent, for example {:?}; {} events arrived for {} reports, first differing at {:?}: {:?}",
        missing.len(),
        expected.len(),
        missing.iter().take(3).collect::<Vec<_>>(),
        received.len(),
        expected.len(),
        first_difference,
        first_difference.map(|at| (received.get(at), expected.get(at)))
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
    // Each report waits until an element has logged the one before: reports
    // that queue up behind a slow App are merged (INP-006).
    let element = |line: &str| line.starts_with("LEFT ") || line.starts_with("RIGHT ");
    session.send(&sgr(0, 5, 2, true));
    session.wait_for(|line| element(line) && line.contains(" Down Left "), 1);
    for (sent, x) in [20, 45, 60].into_iter().enumerate() {
        session.send(&sgr(32, x, 2, true));
        session.wait_for(
            |line| element(line) && line.contains(" Drag Left "),
            sent + 1,
        );
    }
    session.send(&sgr(0, 60, 2, false));
    session.wait_for(|line| line.contains(" Up Left "), 1);
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
    // Nine releases in all: 1, 2, 3, 2 and the one over the other element.
    session.wait_for(|line| line.contains(" Up Left "), 9);
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

/// Shift with the wheel down moves the view sideways: the wide first row
/// loses its start and no later row comes up to the top line.
#[test]
fn inp_005_shift_with_the_wheel_scrolls_sideways() {
    let mut session = Session::start("wheel-wide", "WIDE-START");
    for _ in 0..3 {
        session.send(&sgr(65 | 4, 5, 3, true));
        session.settle(Duration::from_millis(30));
    }
    session.settle(Duration::from_millis(300));
    let screen = session.screen();
    let top = screen.first().map(String::as_str).unwrap_or_default();
    assert!(
        !top.contains("WIDE-START") && !top.contains("outer-"),
        "INP-005: Shift with the wheel down did not scroll sideways, or scrolled vertically:\n{}",
        screen.join("\n")
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
    session.wait_for(|line| line.starts_with("PROBE Up Left 5,6"), 1);
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

// ---------------------------------------------------------------------------
// INP-007: the Kitty keyboard flags, and keys that keep their meaning.

/// Whether `text` pushes any keyboard flags (`ESC [ > digits u`).
fn pushes_keyboard_flags(text: &str) -> bool {
    text.match_indices("\x1b[>").any(|(at, found)| {
        let rest = &text[at + found.len()..];
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        digits > 0 && rest[digits..].starts_with('u')
    })
}

fn root_keys(session: &Session) -> Vec<String> {
    session
        .events()
        .into_iter()
        .filter(|line| line.starts_with("root Key "))
        .collect()
}

fn check_kitty(exit_key: char, how: &str) {
    let mut session = Session::start_with("modes", "PROBE", Terminal::KITTY);
    let text = session.text();
    let frame = text.find("PROBE").unwrap();
    assert!(
        text.find(KITTY_PUSH).is_some_and(|at| at < frame),
        "INP-007: on a terminal that reports the Kitty protocol the backend pushed no {KITTY_PUSH:?} before its first frame"
    );
    session.send(exit_key.to_string().as_bytes());
    session.exit();
    let text = session.text();
    assert!(
        text.rfind(KITTY_POP) > text.rfind(KITTY_PUSH),
        "INP-007: after {how} the last keyboard-flag write is not a pop"
    );
}

#[test]
fn inp_007_kitty_flags_on_and_off_after_a_normal_exit() {
    check_kitty('q', "a normal exit");
}

#[test]
fn inp_007_kitty_flags_on_and_off_after_an_error_exit() {
    check_kitty('e', "an error exit");
}

#[test]
fn inp_007_kitty_flags_on_and_off_after_a_panic() {
    check_kitty('p', "a panic");
}

#[test]
fn inp_007_no_flags_without_the_protocol() {
    let mut session = Session::start("modes", "PROBE");
    session.send(b"q");
    session.exit();
    assert!(
        !pushes_keyboard_flags(&session.text()),
        "INP-007: on a terminal that answers only the device attributes query the backend pushed keyboard flags"
    );
}

#[test]
fn inp_007_keys_keep_their_kitty_meaning() {
    let mut session = Session::start_with("full", "PROBE", Terminal::KITTY);
    // Shift+Enter, Ctrl+H, Ctrl+I, Ctrl+[ and Shift+1 typing '!'.
    for keys in [
        &b"\x1b[13;2u"[..],
        b"\x1b[104;5u",
        b"\x1b[105;5u",
        b"\x1b[91;5u",
        b"\x1b[49;2;33u",
    ] {
        session.send(keys);
        session.settle(Duration::from_millis(30));
    }
    session.wait_for(|line| line.starts_with("root Key "), 5);
    let keys = root_keys(&session);
    assert!(
        keys.len() == 5
            && keys[..4]
                == [
                    "root Key Enter S---",
                    "root Key Char('h') -C--",
                    "root Key Char('i') -C--",
                    "root Key Char('[') -C--",
                ]
            && keys[4].starts_with("root Key Char('!')"),
        "INP-007: Shift+Enter, Ctrl+H, Ctrl+I, Ctrl+[ and Shift+1 typing '!' reached the App as {keys:?}"
    );
}

// ---------------------------------------------------------------------------
// INP-008: lock and media keys keep their codes; a modifier alone is no event.

#[test]
fn inp_008_lock_and_media_keys_keep_their_codes() {
    let mut session = Session::start_with("full", "PROBE", Terminal::KITTY);
    let named = [
        (57358, "CapsLock"),
        (57360, "NumLock"),
        (57359, "ScrollLock"),
        (57428, "MediaPlay"),
        (57429, "MediaPause"),
        (57430, "MediaPlayPause"),
        (57432, "MediaStop"),
        (57435, "MediaNext"),
        (57436, "MediaPrevious"),
    ];
    for (code, _) in named {
        session.send(format!("\x1b[{code}u").as_bytes());
        session.settle(Duration::from_millis(20));
    }
    // Shift, Ctrl, Alt, Super, Hyper and Meta alone, left and right.
    for code in 57441..=57452 {
        session.send(format!("\x1b[{code}u").as_bytes());
        session.settle(Duration::from_millis(10));
    }
    session.send(b"z");
    session.wait_for(|line| line == "root Key Char('z') ----", 1);
    let mut expected: Vec<String> = named
        .iter()
        .map(|(_, name)| format!("root Key {name} ----"))
        .collect();
    expected.push("root Key Char('z') ----".to_owned());
    let keys = root_keys(&session);
    assert!(
        keys == expected,
        "INP-008: the lock, media and modifier keys reached the App as {keys:?}, not {expected:?}"
    );
}

// ---------------------------------------------------------------------------
// INP-009: the terminal's focus reports reach the root, not the focused element.

#[test]
fn inp_009_terminal_focus_reaches_only_the_root() {
    let mut session = Session::start("focus", "FOCUSED");
    session.settle(Duration::from_millis(100));
    let before = session.events().len();
    session.send(b"\x1b[O");
    session.settle(Duration::from_millis(50));
    session.send(b"\x1b[I");
    session.settle(Duration::from_millis(50));
    session.send(b"k");
    session.wait_for(|line| line.contains("Key Char('k')"), 1);
    let after = session.events()[before..].to_vec();
    let element: Vec<&String> = after
        .iter()
        .filter(|line| line.starts_with("FOCUSED Focus"))
        .collect();
    let root: Vec<&String> = after
        .iter()
        .filter(|line| line.starts_with("root Focus"))
        .collect();
    let kept = after
        .iter()
        .any(|line| line == "FOCUSED Key Char('k') ----");
    assert!(
        element.is_empty() && root == ["root Focus Lost", "root Focus Gained"] && kept,
        "INP-009: the focused element received {element:?}, the root {root:?}, and the element {} the next key; log {after:?}",
        if kept { "got" } else { "did not get" }
    );
}

// ---------------------------------------------------------------------------
// INP-010: Ctrl+Z suspends and resumes on Unix.

#[test]
fn inp_010_ctrl_z_suspends_and_resumes() {
    let mut session = Session::start_with("suspend", "PROBE", Terminal::KITTY);
    session.settle(Duration::from_millis(100));
    session.send(b"\x1a");
    session.until_within(
        |session| {
            session
                .events()
                .iter()
                .any(|line| line.starts_with("STOPPED"))
        },
        "stop after Ctrl+Z",
        Duration::from_secs(5),
    );
    session.pump();
    let stopped_at = session.output.len();
    let text = session.text();
    let events = session.events();
    let left_on: Vec<&str> = MODES
        .iter()
        .copied()
        .chain(["1049"])
        .filter(|mode| text.rfind(&set(mode)) > text.rfind(&reset(mode)))
        .collect();
    let flags_on = text.rfind(KITTY_PUSH) > text.rfind(KITTY_POP);
    let canonical = events.iter().any(|line| line == "STOPPED canonical=true");
    let stopped = events.iter().find(|line| line.starts_with("STOPPED"));
    assert!(
        left_on.is_empty() && !flags_on && canonical,
        "INP-010: the App stopped with the modes {left_on:?} still set, the keyboard flags {}, and raw mode {} ({stopped:?})",
        if flags_on { "pushed" } else { "popped" },
        if canonical { "off" } else { "on" }
    );
    session.until(
        |session| {
            let after = String::from_utf8_lossy(&session.output[stopped_at..]).into_owned();
            MODES
                .iter()
                .chain(&["1049"])
                .all(|mode| after.contains(&set(mode)))
                && after.contains(KITTY_PUSH)
                && after.contains("PROBE")
        },
        "a full frame with every mode set again after SIGCONT",
    );
    session.send(b"k");
    session.until_within(
        |session| {
            let events = session.events();
            let resumed = events.iter().position(|line| line == "CONTINUED");
            resumed.is_some_and(|at| {
                events[at..]
                    .iter()
                    .any(|line| line == "root Key Char('k') ----")
            })
        },
        "the first key after the resume",
        Duration::from_secs(5),
    );
    session.send(b"q");
    session.exit();
}

// ---------------------------------------------------------------------------
// INP-011: startup queries, typed keys kept, and the theme following the terminal.

const WHITE: &str = "rgb:ffff/ffff/ffff";
const BLACK: &str = "rgb:0000/0000/0000";

fn first_theme(session: &Session) -> Option<String> {
    session
        .events()
        .into_iter()
        .find(|line| line.starts_with("THEME "))
}

#[test]
fn inp_011_a_light_terminal_gets_the_light_preset() {
    let terminal = Terminal {
        background: Some(WHITE),
        ..Terminal::KITTY
    };
    let session = Session::start_with("theme", "PROBE", terminal);
    assert_eq!(
        first_theme(&session).as_deref(),
        Some("THEME light"),
        "INP-011: a terminal with a white background did not get the light preset"
    );
}

#[test]
fn inp_011_a_dark_terminal_and_an_application_theme_keep_theirs() {
    let dark = Terminal {
        background: Some(BLACK),
        ..Terminal::KITTY
    };
    let session = Session::start_with("theme", "PROBE", dark);
    let on_dark = first_theme(&session);
    let light = Terminal {
        background: Some(WHITE),
        ..Terminal::KITTY
    };
    let session = Session::start_with("theme-set", "PROBE", light);
    let set_by_app = first_theme(&session);
    assert!(
        on_dark.as_deref() == Some("THEME dark")
            && set_by_app.as_deref() == Some("THEME high_contrast"),
        "INP-011: a black background gave {on_dark:?} and an application theme on a white one gave {set_by_app:?}"
    );
}

#[test]
fn inp_011_replies_are_not_keys_and_typed_keys_keep_their_order() {
    let terminal = Terminal {
        background: Some(WHITE),
        hold: Duration::from_millis(150),
        ..Terminal::KITTY
    };
    let mut session = Session::spawn("theme", terminal);
    session.until_within(
        |session| session.queried_at.is_some(),
        "startup query",
        Duration::from_secs(5),
    );
    session.send(b"ab");
    session.until(|session| session.text().contains("PROBE"), "first frame");
    session.wait_for(|line| line.starts_with("root Key "), 2);
    let keys = root_keys(&session);
    assert!(
        keys == ["root Key Char('a') ----", "root Key Char('b') ----"],
        "INP-011: keys typed while the replies were pending, and the replies, reached the App as {keys:?}"
    );
}

#[test]
fn inp_011_a_silent_terminal_costs_at_most_the_wait() {
    let terminal = Terminal {
        silent: true,
        ..Terminal::PLAIN
    };
    let mut session = Session::spawn("theme", terminal);
    session.until_within(
        |session| session.queried_at.is_some(),
        "startup query",
        Duration::from_secs(5),
    );
    let asked = session.queried_at.unwrap();
    session.until(|session| session.text().contains("PROBE"), "first frame");
    let waited = asked.elapsed();
    assert!(
        waited <= Duration::from_millis(400),
        "INP-011: with no reply the first frame came {waited:?} after the queries"
    );
}
