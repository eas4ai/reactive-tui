use super::*;
use crate::terminal::TerminalConfig;
use std::time::Duration;

#[test]
fn test_pty_creation() {
    let pty = PseudoTerminal::new();
    assert_eq!(pty.size(), (80, 24));
}

#[test]
fn test_pty_resize() {
    let mut pty = PseudoTerminal::new();
    pty.resize(120, 30).unwrap();
    assert_eq!(pty.size(), (120, 30));
}

#[cfg(unix)]
#[test]
fn api_terminal_pty_is_a_real_controlling_terminal() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("shell");
    std::fs::write(&path, b"#!/bin/sh\nif ! test -t 0 || ! test -t 1 || ! test -t 2; then echo NOT_A_TTY; exit 2; fi\nprintf 'PTY-READY:'\nstty size\nread value\nprintf 'RESIZED:'\nstty size\nprintf 'ANSWER:%s\\n' \"$value\"\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut pty = PseudoTerminal::new();
    pty.spawn(&TerminalConfig {
        shell: Some(path.to_str().unwrap().to_owned()),
        size: (37, 9),
        ..Default::default()
    })
    .unwrap();
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    let mut output = Vec::new();
    while std::time::Instant::now() < deadline {
        match pty.read_output(Some(Duration::from_millis(30))) {
            Ok(Some(bytes)) => output.extend(bytes),
            Ok(None) => {}
            Err(_) => break,
        }
        if output.contains(&b'\n') {
            break;
        }
    }
    let text = String::from_utf8_lossy(&output);
    assert!(
        text.contains("PTY-READY:9 37"),
        "expected real child terminal size, got {text:?}"
    );
    assert_eq!(pty.try_wait().unwrap(), None);
    pty.resize(51, 11).unwrap();
    pty.write_input(b"hello\n").unwrap();
    while std::time::Instant::now() < deadline {
        if let Ok(Some(bytes)) = pty.read_output(Some(Duration::from_millis(30))) {
            output.extend(bytes);
        }
        if String::from_utf8_lossy(&output).contains("ANSWER:hello") {
            break;
        }
    }
    assert!(String::from_utf8_lossy(&output).contains("ANSWER:hello"));
    assert!(String::from_utf8_lossy(&output).contains("RESIZED:11 51"));
    while pty.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(pty.try_wait().unwrap(), Some(0));
    let pid = pty.child_id().unwrap();
    pty.kill().unwrap();
    pty.kill().unwrap();
    // SAFETY: zero queries existence of only this fixture's recorded PID.
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[cfg(unix)]
#[test]
fn api_terminal_pty_backpressure_and_drop_reap_a_flooding_child() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("flood");
    std::fs::write(&path, "#!/bin/sh\nwhile :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut pty = PseudoTerminal::new();
    assert!(pty.write_input(b"not running").is_err());
    assert!(pty
        .spawn(&TerminalConfig {
            size: (0, 1),
            ..Default::default()
        })
        .is_err());
    assert!(pty
        .spawn(&TerminalConfig {
            shell: Some(fixture.path().join("missing").to_str().unwrap().into()),
            ..Default::default()
        })
        .is_err());
    pty.spawn(&TerminalConfig {
        shell: Some(path.to_str().unwrap().into()),
        ..Default::default()
    })
    .unwrap();
    assert!(pty.spawn(&TerminalConfig::default()).is_err());
    assert!(pty.write_input(&vec![b'x'; 65 * 1024]).is_err());
    // Setup for the behavior under test: let the child start, so the drop
    // below ends a running process.
    std::thread::sleep(Duration::from_millis(50));
    let pid = pty.child_id().unwrap();
    let started = std::time::Instant::now();
    drop(pty);
    // The behavior under test: dropping kills the child at once.
    assert!(started.elapsed() < Duration::from_secs(1));
    // SAFETY: query only the process created by this test.
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}
