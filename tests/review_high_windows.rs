//! INP-012 (docs/spec/input.md): on Windows the direct TTY backend hands the
//! App the console input records that wait when the App polls, also when the
//! poll's timeout is zero. A pseudo console (ConPTY) runs this test binary as
//! its child; the child runs an App on `DirectTtyBackend` whose quit key is
//! `x`, and the parent types `x` into the pseudo console. Its one test is the
//! child too, which the environment tells it, since a pseudo console starts
//! an executable without arguments.

#![cfg(windows)]

mod windows {
    use reactive_tui::app::{App, RootComponent};
    use reactive_tui::backend::direct_tty::DirectTtyBackend;
    use reactive_tui::component::Element;
    use reactive_tui::event::types::{KeyCode, KeyModifiers};
    use reactive_tui::terminal::{PseudoTerminal, TerminalConfig};
    use std::time::{Duration, Instant};

    const CHILD: &str = "REACTIVE_INP012_CHILD";
    const READY: &str = "INP012-READY";

    struct Ready;
    impl RootComponent for Ready {
        fn render(&self) -> Element {
            Element::text(READY)
        }
    }

    /// The child: an App on the direct TTY backend that ends on `x`.
    fn child() {
        let backend =
            DirectTtyBackend::new().expect("the direct TTY backend starts in the pseudo console");
        App::builder()
            .backend(backend)
            .root(Ready)
            .quit_key(KeyCode::Char('x'), KeyModifiers::empty())
            .build()
            .expect("the App builds")
            .run()
            .expect("the App runs");
    }

    /// The pseudo console's output until it holds `marker`, at most 30
    /// seconds: a hang guard, not a timing check.
    fn read_until(pty: &PseudoTerminal, marker: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut bytes = Vec::new();
        while Instant::now() < deadline {
            if let Some(chunk) = pty
                .read_output(Some(Duration::from_millis(20)))
                .expect("the pseudo console's output")
            {
                bytes.extend(chunk);
                if String::from_utf8_lossy(&bytes).contains(marker) {
                    return String::from_utf8_lossy(&bytes).into_owned();
                }
            }
        }
        panic!(
            "the App in the pseudo console never painted {marker:?}; output: {:?}; exit: {:?}",
            String::from_utf8_lossy(&bytes),
            pty.try_wait()
        );
    }

    pub(super) fn run() {
        if std::env::var_os(CHILD).is_some() {
            child();
            return;
        }
        let mut pty = PseudoTerminal::new();
        pty.spawn(&TerminalConfig {
            shell: Some(
                std::env::current_exe()
                    .expect("the test binary")
                    .to_str()
                    .expect("a UTF-8 path")
                    .into(),
            ),
            env: vec![(CHILD.into(), "1".into())],
            size: (80, 24),
            ..Default::default()
        })
        .expect("the pseudo console starts the child");
        read_until(&pty, READY);
        pty.write_input(b"x")
            .expect("the key reaches the pseudo console");
        let deadline = Instant::now() + Duration::from_secs(5);
        while pty.try_wait().expect("the child's status").is_none() && Instant::now() < deadline {
            let _ = pty.read_output(Some(Duration::from_millis(20)));
        }
        let ended = pty.try_wait().expect("the child's status");
        if ended.is_none() {
            let _ = pty.kill();
        }
        assert!(
            ended.is_some(),
            "INP-012: an App on DirectTtyBackend in a pseudo console did not handle the key `x` within 5 seconds"
        );
        assert_eq!(
            ended,
            Some(0),
            "the child App ended cleanly on its quit key"
        );
    }
}

#[test]
fn inp_012_a_key_typed_in_a_pseudo_console_reaches_an_app_on_the_direct_tty_backend() {
    windows::run();
}
