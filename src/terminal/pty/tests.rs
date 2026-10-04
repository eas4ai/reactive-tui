use super::*;
#[cfg(unix)]
use crate::terminal::TerminalConfig;
#[cfg(unix)]
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
    // Setup for the behavior under test: wait for the flood's output, so the
    // drop below ends a running, flooding process. The wait is a hang guard.
    assert!(pty
        .read_output(Some(Duration::from_secs(30)))
        .unwrap()
        .is_some_and(|bytes| !bytes.is_empty()));
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

/// A shell that prints without pause, for TRM-001: once nothing reads the
/// pseudo-terminal, its output fills the buffer and the shell blocks in
/// write with bytes still unread.
#[cfg(unix)]
fn flooding_shell(dir: &std::path::Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("flood");
    std::fs::write(&path, "#!/bin/sh\nwhile :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path.to_str().unwrap().to_owned()
}

/// A terminal whose flooding child has been read once and then no more:
/// the child is known to be printing, and after a pause its unread output
/// fills the pseudo-terminal.
#[cfg(unix)]
fn flooding_terminal(dir: &std::path::Path) -> (PseudoTerminal, u32) {
    let mut pty = PseudoTerminal::new();
    pty.spawn(&TerminalConfig {
        shell: Some(flooding_shell(dir)),
        ..Default::default()
    })
    .unwrap();
    // Setup, with a hang guard: the first output proves the flood runs.
    assert!(pty
        .read_output(Some(Duration::from_secs(30)))
        .unwrap()
        .is_some_and(|bytes| !bytes.is_empty()));
    std::thread::sleep(Duration::from_millis(300));
    let pid = pty.child_id().unwrap();
    (pty, pid)
}

/// Runs the stop on a thread of its own and returns how long it took. A
/// stop that has not returned after 10 s is the defect TRM-001 names, and
/// fails the test instead of hanging the run; the test process's exit then
/// closes the master, which lets the child finish exiting.
#[cfg(unix)]
fn stop_within_guard(stop: impl FnOnce() + Send + 'static) -> Duration {
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        stop();
        let _ = done.send(started.elapsed());
    });
    finished
        .recv_timeout(Duration::from_secs(10))
        .expect("TRM-001: stopping the flooding child did not return within 10 s")
}

#[cfg(unix)]
fn assert_reaped(pid: u32) {
    // SAFETY: a liveness query of the process this test created.
    assert_eq!(
        unsafe { libc::kill(pid as i32, 0) },
        -1,
        "the child still exists"
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[cfg(unix)]
#[test]
fn trm_001_kill_reaps_a_flooding_child_within_a_second() {
    let fixture = tempfile::tempdir().unwrap();
    let (mut pty, pid) = flooding_terminal(fixture.path());
    let took = stop_within_guard(move || pty.kill().unwrap());
    assert!(took < Duration::from_secs(1), "kill took {took:?}");
    assert_reaped(pid);
}

#[cfg(unix)]
#[test]
fn trm_001_drop_reaps_a_flooding_child_within_a_second() {
    let fixture = tempfile::tempdir().unwrap();
    let (pty, pid) = flooding_terminal(fixture.path());
    let took = stop_within_guard(move || drop(pty));
    assert!(took < Duration::from_secs(1), "drop took {took:?}");
    assert_reaped(pid);
}

/// Finding 1 of the adversary's report: output a stop character (Ctrl-S,
/// `stty ixon`) has paused never drains, and macOS holds the child's exit
/// for it all the same, so a stop that only reads the master would wait
/// forever here. The flood enables flow control, is read once, is paused
/// with Ctrl-S through the terminal's input, and is then killed.
#[cfg(unix)]
#[test]
fn trm_001_kill_reaps_a_child_whose_output_a_stop_character_paused() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("paused-flood");
    std::fs::write(
        &path,
        "#!/bin/sh\nstty ixon -ixany\nwhile :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut pty = PseudoTerminal::new();
    pty.spawn(&TerminalConfig {
        shell: Some(path.to_str().unwrap().to_owned()),
        ..Default::default()
    })
    .unwrap();
    assert!(pty
        .read_output(Some(Duration::from_secs(30)))
        .unwrap()
        .is_some_and(|bytes| !bytes.is_empty()));
    // Setup: the stop character pauses the output while unread bytes queue.
    pty.write_input(&[0x13]).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let pid = pty.child_id().unwrap();
    let took = stop_within_guard(move || pty.kill().unwrap());
    assert!(took < Duration::from_secs(1), "kill took {took:?}");
    assert_reaped(pid);
}
