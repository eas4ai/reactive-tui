#![cfg(all(unix, feature = "embedded-terminal"))]
use reactive_tui::{
    embedded::{EmbeddedSession, SessionSnapshot},
    event::types::{KeyCode, KeyEvent, KeyModifiers},
};
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

fn shell(script: &str, width: u16, height: u16) -> EmbeddedSession {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", script]).env("TERM", "xterm-256color");
    EmbeddedSession::spawn(command, width, height).unwrap()
}

fn text(snapshot: &SessionSnapshot) -> String {
    snapshot
        .frame
        .cells()
        .chunks(usize::from(snapshot.frame.size().0))
        .map(|row| {
            row.iter()
                .map(|cell| cell.text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wait(
    session: &EmbeddedSession,
    condition: impl Fn(&SessionSnapshot) -> bool,
) -> SessionSnapshot {
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = session.snapshot().unwrap();
        assert!(snapshot.error.is_none(), "{:?}", snapshot.error);
        if condition(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "timed out: {:?}, screen: {:?}",
            snapshot.exit_status,
            text(&snapshot)
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn emb_001_real_tty_configuration_and_spawn_failure() {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "test -t 0 && test -t 1 && test -t 2 && printf 'TTY:%s:%s:' \"$DEMO_VALUE\" \"$1\"; pwd; stty size", "fixture", "argument"])
        .env("DEMO_VALUE", "configured").current_dir("/tmp");
    let session = EmbeddedSession::spawn(command, 40, 8).unwrap();
    let snapshot = wait(&session, |s| s.stopped);
    assert!(snapshot.exit_status.unwrap().success());
    assert!(
        text(&snapshot).contains("TTY:configured:argument:/tmp"),
        "{}",
        text(&snapshot)
    );
    assert!(text(&snapshot).contains("8 40"));
    assert!(EmbeddedSession::spawn(Command::new("/no/such/reactive-tui-program"), 40, 8).is_err());
    assert!(EmbeddedSession::spawn(Command::new("/bin/sh"), 0, 8).is_err());
}

#[test]
fn emb_003_text_enter_and_ctrl_c_reach_the_child() {
    let session = shell("trap 'printf INTERRUPTED; exit 0' INT; printf READY; while read -r value; do printf 'GOT:%s' \"$value\"; done", 60, 8);
    wait(&session, |s| text(s).contains("READY"));
    for ch in "hello".chars() {
        session.send_key(KeyEvent::new(KeyCode::Char(ch))).unwrap();
    }
    session.send_key(KeyEvent::new(KeyCode::Enter)).unwrap();
    wait(&session, |s| text(s).contains("GOT:hello"));
    session
        .send_key(
            KeyEvent::new(KeyCode::Char('c')).with_modifiers(KeyModifiers {
                ctrl: true,
                ..KeyModifiers::empty()
            }),
        )
        .unwrap();
    let snapshot = wait(&session, |s| s.stopped);
    assert!(
        text(&snapshot).contains("INTERRUPTED"),
        "{}",
        text(&snapshot)
    );
}

#[test]
fn emb_004_resize_updates_pty_and_snapshot_and_ignores_zero() {
    let session = shell(
        "printf READY; while read -r value; do stty size; done",
        30,
        5,
    );
    wait(&session, |s| text(s).contains("READY"));
    session.resize(44, 9).unwrap();
    assert_eq!(session.snapshot().unwrap().frame.size(), (44, 9));
    session.resize(0, 0).unwrap();
    assert_eq!(session.snapshot().unwrap().frame.size(), (44, 9));
    session.send_key(KeyEvent::new(KeyCode::Enter)).unwrap();
    wait(&session, |s| text(s).contains("9 44"));
}

#[test]
fn emb_005_delayed_output_exit_and_idle_shutdown_are_observable() {
    let session = shell("sleep 0.1; printf DELAYED; exit 7", 30, 4);
    let before = session.snapshot().unwrap().revision;
    let snapshot = wait(&session, |s| s.stopped);
    assert!(snapshot.revision > before);
    assert!(text(&snapshot).contains("DELAYED"));
    assert_eq!(snapshot.exit_status.unwrap().code(), Some(7));
    let mut idle = shell("exec sleep 30", 30, 4);
    let pid = idle.child_id() as libc::pid_t;
    let start = Instant::now();
    idle.shutdown().unwrap();
    // The behavior under test: shutdown kills the child that sleeps 30 s;
    // finishing well inside that shows it did not wait, with room for a
    // busy machine.
    assert!(start.elapsed() < Duration::from_secs(10));
    let stopped = idle.snapshot().unwrap();
    assert!(stopped.stopped);
    // A child that is waited out exits with code 0; a killed one ends by a signal.
    let status = stopped
        .exit_status
        .expect("shutdown records the exit status");
    assert_eq!(status.code(), None);
    assert!(std::os::unix::process::ExitStatusExt::signal(&status).is_some());
    // SAFETY: query only; WNOHANG cannot block. ECHILD proves this owner reaped it.
    assert_eq!(
        unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) },
        -1
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD)
    );
    idle.shutdown().unwrap();
}

#[test]
fn emb_002_styled_graphemes_alternate_screen_and_erasure_reach_host() {
    use reactive_tui::backend::cell_frame::UnderlineStyle;
    use reactive_tui::backend::{Backend, SuprTuiBackend};
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let script = r#"
import os, sys, termios
attrs = termios.tcgetattr(0)
attrs[3] &= ~termios.ECHO
termios.tcsetattr(0, termios.TCSANOW, attrs)
os.write(1, b'\x1b[2J\x1b[H\x1b[1;3;38;2;12;34;56;48;2;65;43;21m' + 'é界🙂'.encode() + b'\x1b[0m')
for i in range(1,6):
    os.write(1, ('\x1b[3;%dH\x1b[4:%dm\x1b[58:2::10:20:30m\x1b[53mX\x1b[0m' % (i,i)).encode())
os.write(1, b'\x1b[2;3H\x1b[?25l')
sys.stdin.buffer.readline()
os.write(1, b'\x1b[?1049h\x1b[2J\x1b[HALT\x1b[?25h')
sys.stdin.buffer.readline()
os.write(1, b'\x1b[?1049l')
sys.stdin.buffer.readline()
os.write(1, b'\x1b[H\x1b[2J')
"#;
    let mut command = Command::new("python3");
    command.args(["-u", "-c", script]);
    let session = EmbeddedSession::spawn(command, 20, 5).unwrap();
    let initial = wait(&session, |s| {
        s.frame.cell(4, 2).is_some_and(|c| c.text == "X") && s.frame.cursor().is_none()
    });
    assert_eq!(initial.frame.cell(0, 0).unwrap().text, "e\u{301}");
    assert_eq!(initial.frame.cell(1, 0).unwrap().text, "界");
    assert_eq!(initial.frame.cell(1, 0).unwrap().width, 2);
    assert_eq!(initial.frame.cell(2, 0).unwrap().width, 0);
    assert_eq!(initial.frame.cell(3, 0).unwrap().text, "🙂");
    assert_eq!(initial.frame.cell(4, 0).unwrap().width, 0);
    assert_eq!(initial.frame.cell(0, 0).unwrap().foreground, [12, 34, 56]);
    assert_eq!(initial.frame.cell(0, 0).unwrap().background, [65, 43, 21]);
    for (x, style) in [
        UnderlineStyle::Single,
        UnderlineStyle::Double,
        UnderlineStyle::Curly,
        UnderlineStyle::Dotted,
        UnderlineStyle::Dashed,
    ]
    .into_iter()
    .enumerate()
    {
        let decoration = initial.frame.cell(x as u16, 2).unwrap().decoration;
        assert_eq!(decoration.underline, style);
        assert_eq!(decoration.underline_color, Some([10, 20, 30]));
        assert!(decoration.overline);
    }
    let out = Capture::default();
    let mut backend = SuprTuiBackend::with_writer(20, 5, out.clone()).unwrap();
    let mut host = vt100::Parser::new(5, 20, 0);
    backend.render_cells(Arc::clone(&initial.frame)).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    let bytes = std::mem::take(&mut *out.0.lock().unwrap());
    host.process(&bytes);
    let emitted = String::from_utf8_lossy(&bytes);
    assert!(emitted.contains("\x1b[58:2::10:20:30m"));
    assert!(emitted.contains("\x1b[4:5m"));
    assert_eq!(emitted.matches("🙂").count(), 1);
    // Independent host parser verifies style, grapheme, and wide-cell semantics.
    assert_eq!(host.screen().cell(0, 0).unwrap().contents(), "e\u{301}");
    assert!(host.screen().cell(0, 0).unwrap().bold());
    assert!(host.screen().cell(0, 0).unwrap().italic());
    assert!(
        !host.screen().cell(0, 5).unwrap().bold(),
        "style leaked to the following blank"
    );
    assert!(host.screen().cell(0, 2).unwrap().is_wide_continuation());
    assert!(host.screen().hide_cursor());
    session.send_key(KeyEvent::new(KeyCode::Enter)).unwrap();
    let alternate = wait(&session, |s| text(s).starts_with("ALT"));
    backend.render_cells(Arc::clone(&alternate.frame)).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    host.process(&std::mem::take(&mut *out.0.lock().unwrap()));
    assert!(host.screen().contents().starts_with("ALT"));
    assert!(!host.screen().hide_cursor());
    assert_eq!(host.screen().cursor_position(), (0, 3));
    session.send_key(KeyEvent::new(KeyCode::Enter)).unwrap();
    let restored = wait(&session, |s| s.frame.cell(0, 0).unwrap().text == "e\u{301}");
    backend.render_cells(Arc::clone(&restored.frame)).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    host.process(&std::mem::take(&mut *out.0.lock().unwrap()));
    assert_eq!(host.screen().cell(0, 1).unwrap().contents(), "界");
    session.send_key(KeyEvent::new(KeyCode::Enter)).unwrap();
    let erased = wait(&session, |s| s.stopped);
    backend.render_cells(Arc::clone(&erased.frame)).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    host.process(&std::mem::take(&mut *out.0.lock().unwrap()));
    assert!(
        host.screen().contents().trim().is_empty(),
        "host={:?}; snapshot={:?}",
        host.screen().contents(),
        text(&erased)
    );
    backend
        .render_frame(&reactive_tui::component::Element::text("root"))
        .unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
    host.process(&std::mem::take(&mut *out.0.lock().unwrap()));
    assert!(host.screen().contents().starts_with("root"));
    assert!(host.screen().hide_cursor());
}

#[test]
fn emb_002_joined_emoji_remains_one_owned_grapheme() {
    // vt100's width oracle does not implement joined emoji. The test above
    // uses CJK and a single-scalar emoji for independent host-grid assertions;
    // this test checks the exact joined cluster and native occupancy separately.
    let session = shell("printf '👩‍💻'", 8, 2);
    let snapshot = wait(&session, |s| s.stopped);
    assert_eq!(snapshot.frame.cell(0, 0).unwrap().text, "👩‍💻");
    assert_eq!(snapshot.frame.cell(0, 0).unwrap().width, 2);
    assert_eq!(snapshot.frame.cell(1, 0).unwrap().width, 0);
}

#[test]
fn emb_002_utf8_c1_cells_are_safe_for_the_host_frame() {
    use reactive_tui::backend::{Backend, SuprTuiBackend};
    let session = shell(r"printf '\033[1;96H\302\205'", 96, 4);
    let snapshot = wait(&session, |s| s.stopped);
    let cell = snapshot.frame.cell(95, 0).unwrap();
    assert_eq!(cell.text, "\u{fffd}");
    assert_eq!(cell.width, 1);
    assert!(snapshot
        .frame
        .cells()
        .iter()
        .all(|cell| !cell.text.chars().any(char::is_control)));
    let mut backend = SuprTuiBackend::with_writer(96, 4, std::io::sink()).unwrap();
    backend.render_cells(snapshot.frame).unwrap();
    backend.present().unwrap();
    backend.sync().unwrap();
}

#[test]
fn emb_002_cell_frames_reject_invalid_occupancy_and_control_bytes() {
    use reactive_tui::backend::{CellFrame, FrameCell};
    let plain = FrameCell {
        text: "x".into(),
        width: 1,
        foreground: [255; 3],
        background: [0; 3],
        attributes: 0,
        decoration: Default::default(),
    };
    assert!(CellFrame::new(1, 1, vec![plain.clone()], None).is_ok());
    assert!(CellFrame::new(1, 1, vec![plain.clone()], Some((1, 0))).is_err());
    assert!(CellFrame::new(1, 1, vec![], None).is_err());
    for (text, width) in [("\x1b[31m", 1), ("xy", 1), ("", 0), ("界", 2)] {
        let mut invalid = plain.clone();
        invalid.text = text.into();
        invalid.width = width;
        assert!(CellFrame::new(1, 1, vec![invalid], None).is_err());
    }
}

#[test]
fn emb_005_app_renders_background_updates_and_restores_on_errors() {
    use reactive_tui::{
        app::{App, RootComponent, RootUpdate},
        backend::Backend,
        component::{Element, ElementType},
        error::{ReactiveError, Result},
        event::types::Event,
        render::{reconcile::PatchOp, tree::RenderTree},
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    struct IdleBackend {
        frames: Arc<Mutex<Vec<String>>>,
        staged: String,
        shutdowns: Arc<AtomicUsize>,
    }
    impl Backend for IdleBackend {
        fn render_frame(&mut self, element: &Element) -> Result<bool> {
            let ElementType::Text(text) = &element.element_type else {
                panic!("expected text frame");
            };
            self.staged = text.clone();
            Ok(true)
        }
        fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
            panic!("complete frame expected")
        }
        fn clear(&mut self) -> Result<()> {
            self.staged.clear();
            Ok(())
        }
        fn present(&mut self) -> Result<()> {
            self.frames.lock().unwrap().push(self.staged.clone());
            Ok(())
        }
        fn size(&self) -> (u16, u16) {
            (20, 4)
        }
        fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
            Ok(None)
        }
        fn shutdown(&mut self) -> Result<()> {
            self.shutdowns.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
    struct Root {
        count: u8,
        fail: bool,
    }
    impl RootComponent for Root {
        fn render(&self) -> Element {
            Element::text(self.count.to_string())
        }
        fn update(&mut self) -> Result<RootUpdate> {
            if self.fail {
                return Err(ReactiveError::terminal("controlled root update failure"));
            }
            self.count += 1;
            Ok(if self.count == 1 {
                RootUpdate::Redraw
            } else {
                RootUpdate::Exit
            })
        }
    }
    for fail in [false, true] {
        let frames = Arc::new(Mutex::new(Vec::new()));
        let shutdowns = Arc::new(AtomicUsize::new(0));
        let backend = IdleBackend {
            frames: Arc::clone(&frames),
            staged: String::new(),
            shutdowns: Arc::clone(&shutdowns),
        };
        let result = App::builder()
            .backend(backend)
            .root(Root { count: 0, fail })
            .build()
            .unwrap()
            .run();
        assert_eq!(result.is_err(), fail);
        assert_eq!(shutdowns.load(Ordering::Relaxed), 1);
        assert_eq!(
            *frames.lock().unwrap(),
            if fail { vec!["0"] } else { vec!["0", "1"] }
        );
    }
}

#[test]
fn emb_005_app_observes_natural_exit_and_final_frame_without_input() {
    use reactive_tui::{
        app::App,
        backend::{Backend, CellFrame},
        component::Element,
        embedded::TerminalView,
        error::{ReactiveError, Result},
        event::types::Event,
        render::{reconcile::PatchOp, tree::RenderTree},
    };
    use std::sync::{Arc, Mutex};
    struct CellBackend(Arc<Mutex<Vec<Arc<CellFrame>>>>);
    impl Backend for CellBackend {
        fn render_cells(&mut self, frame: Arc<CellFrame>) -> Result<()> {
            self.0.lock().unwrap().push(frame);
            Ok(())
        }
        fn render_frame(&mut self, _: &Element) -> Result<bool> {
            panic!("expected native cell frame")
        }
        fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
            panic!("expected native cell frame")
        }
        fn clear(&mut self) -> Result<()> {
            Ok(())
        }
        fn present(&mut self) -> Result<()> {
            Ok(())
        }
        fn size(&self) -> (u16, u16) {
            (20, 4)
        }
        fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
            panic!("wake-aware wait expected")
        }
        fn poll_event_with_wake(
            &mut self,
            timeout: Option<Duration>,
            wake: &reactive_tui::app::AppWaker,
        ) -> Result<Option<Event>> {
            let start = Instant::now();
            // A hang guard, not a timing check.
            wake.wait(Some(timeout.unwrap_or(Duration::from_secs(30))));
            if start.elapsed() >= Duration::from_secs(30) {
                return Err(ReactiveError::terminal("native App exit watchdog"));
            }
            Ok(None)
        }
    }
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "sleep 0.05; printf FINISHED"]);
    let session = EmbeddedSession::spawn(command, 20, 4).unwrap();
    let pid = session.child_id();
    let frames = Arc::new(Mutex::new(Vec::new()));
    App::builder()
        .backend(CellBackend(frames.clone()))
        .root(TerminalView::new(session))
        .build()
        .unwrap()
        .run()
        .unwrap();
    let frames = frames.lock().unwrap();
    let final_text: String = frames
        .last()
        .unwrap()
        .cells()
        .iter()
        .map(|cell| cell.text.as_str())
        .collect();
    assert!(
        final_text.contains("FINISHED"),
        "final child output must be rendered before exit"
    );
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}
