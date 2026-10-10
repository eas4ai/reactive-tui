// Shared launcher for the internal lifecycle probe and the embedded-shell example.

#[cfg(all(unix, feature = "embedded-terminal"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use reactive_tui::{
        app::App,
        backend::{Backend, SuprTuiBackend},
        embedded::{EmbeddedSession, TerminalView},
        event::types::{KeyCode, KeyModifiers},
    };
    use std::process::Command;

    let backend = SuprTuiBackend::new()?;
    let (width, height) = backend.size();
    let mut args = std::env::args_os().skip(1);
    let executable = args
        .next()
        .unwrap_or_else(|| std::env::var_os("SHELL").unwrap_or_else(|| "/bin/sh".into()));
    let mut command = Command::new(executable);
    command.args(args).env("TERM", "xterm-256color");
    let session = EmbeddedSession::spawn(command, width, height)?;
    App::builder()
        .backend(backend)
        .root(TerminalView::new(session))
        .quit_key(
            KeyCode::Char('q'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::empty()
            },
        )
        .build()?
        .run()?;
    Ok(())
}

#[cfg(not(all(unix, feature = "embedded-terminal")))]
fn main() {
    eprintln!("embedded terminal probe requires Unix and --features embedded-terminal");
    std::process::exit(1);
}
