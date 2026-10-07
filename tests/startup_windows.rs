//! INP-013 (docs/spec/input.md): on Windows the default backend asks the
//! terminal for its background color before its first frame, ends the
//! exchange with a device-attributes query, waits at most 200 ms, keeps every
//! key typed meanwhile in order, and makes the light preset active on a light
//! terminal when the application set no theme.
//!
//! Each test runs this binary again as a pseudo console's (ConPTY) child, as
//! the environment tells it; the child runs an App on the default backend
//! for one scenario and logs its first theme and every key it receives to a
//! file. The test is the terminal: it reads what the App writes, answers the
//! questions it sees as a light, dark or silent terminal, types keys around
//! the replies, and reads the log.

#![cfg(windows)]

mod windows {
    use reactive_tui::app::{App, RootComponent};
    use reactive_tui::backend::SuprTuiBackend;
    use reactive_tui::component::Element;
    use reactive_tui::error::Result;
    use reactive_tui::event::router::EventResult;
    use reactive_tui::event::types::{Event, KeyCode, KeyEventKind, KeyModifiers};
    use reactive_tui::terminal::{PseudoTerminal, TerminalConfig};
    use reactive_tui::theme::{high_contrast_theme, Theme};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Names the scenario the child runs.
    const SCENARIO: &str = "REACTIVE_TUI_INP013_SCENARIO";
    /// The file the child appends its theme and its keys to.
    const LOG: &str = "REACTIVE_TUI_INP013_LOG";
    /// The first frame's text.
    const PROBE: &str = "PROBE";
    /// The background color query, ended by BEL or ST, and two answers.
    const BACKGROUND_QUERY: &[u8] = b"\x1b]11;?";
    const WHITE: &[u8] = b"\x1b]11;rgb:ffff/ffff/ffff\x1b\\";
    const BLACK: &[u8] = b"\x1b]11;rgb:0000/0000/0000\x1b\\";
    /// The primary device attributes query and a VT220-style reply.
    const ATTRIBUTES_QUERY: &[u8] = b"\x1b[c";
    const ATTRIBUTES_REPLY: &[u8] = b"\x1b[?62;22c";
    /// Bounds a hung child; a child starts and paints in well under a second.
    const DEADLINE: Duration = Duration::from_secs(30);

    // -----------------------------------------------------------------------
    // The child: an App on the default backend for one scenario.

    static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

    fn log(line: String) {
        if let Some(path) = LOG_PATH.lock().unwrap().as_ref() {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .expect("the event log opens");
            writeln!(file, "{line}").expect("the event log takes a line");
        }
    }

    struct Scenario;

    impl RootComponent for Scenario {
        fn render(&self) -> Element {
            log(format!("THEME {}", Theme::active().name));
            Element::text(PROBE)
        }

        fn try_handle_event(&mut self, event: &Event) -> Result<EventResult> {
            // A console reports each key's release too; a key typed once is
            // one press.
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Release {
                    log(format!("KEY {:?}", key.code));
                }
            }
            Ok(EventResult::Ignored)
        }
    }

    fn child(scenario: &str) {
        *LOG_PATH.lock().unwrap() = std::env::var_os(LOG).map(PathBuf::from);
        if scenario == "theme-set" {
            Theme::set_active(high_contrast_theme());
        }
        let backend = SuprTuiBackend::new().expect("the default backend in the pseudo console");
        App::builder()
            .backend(backend)
            .root(Scenario)
            .quit_key(KeyCode::Char('q'), KeyModifiers::empty())
            .build()
            .expect("the scenario's App")
            .run()
            .expect("the App runs");
    }

    // -----------------------------------------------------------------------
    // The test side: a pseudo console whose terminal answers as told.

    /// What the terminal answers: the background color unless `silent`, and
    /// the device attributes with it, `hold` after the query is first seen;
    /// `typed_before` goes in when the query is first seen and `typed_after`
    /// right after the replies. (A key typed inside a half-delivered reply
    /// is not a scenario: the pseudo console takes it into the string, so
    /// the collector's handling of that order is a unit test in the
    /// crossterm copy.)
    #[derive(Clone, Copy)]
    struct Terminal {
        background: Option<&'static [u8]>,
        silent: bool,
        hold: Duration,
        typed_before: &'static [u8],
        typed_after: &'static [u8],
    }

    impl Terminal {
        const LIGHT: Terminal = Terminal {
            background: Some(WHITE),
            silent: false,
            hold: Duration::ZERO,
            typed_before: b"",
            typed_after: b"",
        };
    }

    struct Run {
        theme: Option<String>,
        keys: Vec<String>,
        queried_after: Option<Duration>,
        painted_after: Option<Duration>,
        output: String,
    }

    fn run(scenario: &str, terminal: Terminal) -> Run {
        let log_path = std::env::temp_dir().join(format!(
            "reactive-tui-inp013-{}-{}.log",
            std::process::id(),
            scenario
        ));
        let _ = std::fs::remove_file(&log_path);
        let mut pty = PseudoTerminal::new();
        pty.spawn(&TerminalConfig {
            shell: Some(
                std::env::current_exe()
                    .expect("the test binary")
                    .to_str()
                    .expect("a UTF-8 path")
                    .into(),
            ),
            env: vec![
                (SCENARIO.into(), scenario.into()),
                (LOG.into(), log_path.to_str().expect("a UTF-8 path").into()),
            ],
            size: (80, 24),
            ..Default::default()
        })
        .expect("the pseudo console starts the child");
        let start = Instant::now();
        let mut bytes = Vec::new();
        let mut queried_after = None;
        let mut painted_after = None;
        let mut answered = false;
        let mut quit_sent = false;
        let mut ended = None;
        while start.elapsed() < DEADLINE {
            if let Some(chunk) = pty
                .read_output(Some(Duration::from_millis(20)))
                .expect("the pseudo console's output")
            {
                bytes.extend(chunk);
            }
            if queried_after.is_none() && contains(&bytes, BACKGROUND_QUERY) {
                queried_after = Some(start.elapsed());
                if !terminal.typed_before.is_empty() {
                    pty.write_input(terminal.typed_before)
                        .expect("keys typed before the replies");
                }
            }
            let due = |queried: Duration| start.elapsed() >= queried + terminal.hold;
            if !answered && queried_after.is_some_and(due) {
                answered = true;
                if !terminal.silent {
                    if let Some(color) = terminal.background {
                        pty.write_input(color).expect("the background reply");
                    }
                    if contains(&bytes, ATTRIBUTES_QUERY) {
                        pty.write_input(ATTRIBUTES_REPLY)
                            .expect("the attributes reply");
                    }
                }
                if !terminal.typed_after.is_empty() {
                    pty.write_input(terminal.typed_after)
                        .expect("keys typed after the replies");
                }
            }
            if painted_after.is_none() && contains(&bytes, PROBE.as_bytes()) {
                painted_after = Some(start.elapsed());
            }
            // The keys typed around the replies come first; a short wait
            // after the first frame lets the App log them before the quit
            // key ends it.
            let settled =
                |painted: Duration| start.elapsed() >= painted + Duration::from_millis(300);
            if !quit_sent && painted_after.is_some_and(settled) {
                pty.write_input(b"q").expect("the quit key");
                quit_sent = true;
            }
            ended = pty.try_wait().expect("the child's status");
            if ended.is_some() {
                break;
            }
        }
        if ended.is_none() {
            let _ = pty.kill();
        }
        let output = String::from_utf8_lossy(&bytes).into_owned();
        assert!(
            queried_after.is_some(),
            "INP-013 ({scenario}): the App asked the terminal nothing before its first frame; output: {output:?}"
        );
        assert!(
            painted_after.is_some(),
            "INP-013 ({scenario}): the App never painted its first frame; output: {output:?}"
        );
        assert_eq!(
            ended,
            Some(0),
            "INP-013 ({scenario}): the child App did not end cleanly on its quit key; output: {output:?}"
        );
        let lines: Vec<String> = std::fs::read_to_string(&log_path)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect();
        let _ = std::fs::remove_file(&log_path);
        Run {
            theme: lines
                .iter()
                .find(|line| line.starts_with("THEME "))
                .cloned(),
            keys: lines
                .iter()
                .filter(|line| line.starts_with("KEY "))
                .take_while(|line| *line != "KEY Char('q')")
                .cloned()
                .collect(),
            queried_after,
            painted_after,
            output,
        }
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    // -----------------------------------------------------------------------
    // INP-013.

    /// Runs one scenario when a test starts this binary as a pseudo console's
    /// child; a plain run of the binary passes it.
    #[test]
    fn smoke_inp_013_child() {
        let Some(scenario) = std::env::var_os(SCENARIO) else {
            eprintln!("SKIP: run by the INP-013 tests with {SCENARIO} set");
            return;
        };
        child(&scenario.to_string_lossy());
    }

    /// Whether this process is the terminal side: the child runs every test
    /// of this binary too, and in it the terminal-side tests do nothing.
    fn terminal_side() -> bool {
        if std::env::var_os(SCENARIO).is_some() {
            eprintln!("SKIP: the pseudo console's child runs no terminal-side test");
            return false;
        }
        true
    }

    #[test]
    #[serial_test::serial]
    fn inp_013_a_light_terminal_gets_the_light_preset() {
        if !terminal_side() {
            return;
        }
        let run = run("theme", Terminal::LIGHT);
        assert_eq!(
            run.theme.as_deref(),
            Some("THEME light"),
            "INP-013: a terminal with a white background did not get the light preset; output: {:?}",
            run.output
        );
    }

    #[test]
    #[serial_test::serial]
    fn inp_013_a_dark_terminal_and_an_application_theme_keep_theirs() {
        if !terminal_side() {
            return;
        }
        let on_dark = run(
            "theme",
            Terminal {
                background: Some(BLACK),
                ..Terminal::LIGHT
            },
        );
        let set_by_app = run("theme-set", Terminal::LIGHT);
        assert!(
            on_dark.theme.as_deref() == Some("THEME dark")
                && set_by_app.theme.as_deref() == Some("THEME high_contrast"),
            "INP-013: a black background gave {:?} and an application theme on a white one gave {:?}",
            on_dark.theme,
            set_by_app.theme
        );
    }

    #[test]
    #[serial_test::serial]
    fn inp_013_replies_are_not_keys_and_typed_keys_keep_their_order() {
        if !terminal_side() {
            return;
        }
        let run = run(
            "theme",
            Terminal {
                typed_before: b"x",
                typed_after: b"y",
                ..Terminal::LIGHT
            },
        );
        assert!(
            run.keys == ["KEY Char('x')", "KEY Char('y')"],
            "INP-013: the keys typed around the replies, and the replies, reached the App as {:?}",
            run.keys
        );
        assert_eq!(
            run.theme.as_deref(),
            Some("THEME light"),
            "INP-013: the replies read with typed keys did not reach the backend as replies"
        );
    }

    /// The adversary's finding 1: a reply that arrives after the 200 ms wait
    /// is consumed, not typed, and not used.
    #[test]
    #[serial_test::serial]
    fn inp_013_a_late_reply_is_consumed_not_typed() {
        if !terminal_side() {
            return;
        }
        let run = run(
            "theme",
            Terminal {
                hold: Duration::from_millis(350),
                typed_after: b"z",
                ..Terminal::LIGHT
            },
        );
        assert!(
            run.keys == ["KEY Char('z')"],
            "INP-013: a reply 350 ms after the questions reached the App as {:?}",
            run.keys
        );
        assert_eq!(
            run.theme.as_deref(),
            Some("THEME dark"),
            "INP-013: a reply after the wait changed the preset"
        );
    }

    #[test]
    #[serial_test::serial]
    fn inp_013_a_silent_terminal_costs_at_most_the_wait() {
        if !terminal_side() {
            return;
        }
        let run = run(
            "theme",
            Terminal {
                silent: true,
                ..Terminal::LIGHT
            },
        );
        let waited = run.painted_after.unwrap() - run.queried_after.unwrap();
        assert!(
            waited <= Duration::from_millis(400),
            "INP-013: with no reply the first frame came {waited:?} after the queries"
        );
        assert_eq!(
            run.theme.as_deref(),
            Some("THEME dark"),
            "INP-013: a silent terminal did not keep the dark preset"
        );
    }
}
