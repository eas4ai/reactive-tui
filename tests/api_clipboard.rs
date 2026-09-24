#![cfg(unix)]

use reactive_tui::hooks::clipboard::use_clipboard;
use reactive_tui::reactive::hooks::Hooks;
use std::fs;
use std::os::unix::{fs::PermissionsExt, process::CommandExt};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Fixture {
    directory: PathBuf,
    child: Option<Child>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            // A corrected library gives its command a separate group. If a
            // regression stalls the test itself, stop that recorded group too.
            if let Ok(pids) = fs::read_to_string(self.directory.join("pid")) {
                for pid in pids
                    .lines()
                    .filter_map(|pid| pid.trim().parse::<i32>().ok())
                {
                    if pid > 0 {
                        unsafe {
                            libc::kill(-pid, libc::SIGKILL);
                        }
                    }
                }
            }
            // Every fixture has a private process group. Reap its direct test
            // child and stop fixture descendants even when an assertion fails.
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.wait();
        }
        fs::remove_dir_all(&self.directory).expect("remove clipboard fixture");
    }
}

fn isolated(name: &str) -> bool {
    if std::env::var("RTUI_API_CLIPBOARD_CHILD").as_deref() == Ok(name) {
        return false;
    }
    let directory =
        std::env::temp_dir().join(format!("rtui-api-clipboard-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&directory).unwrap();
    let mut fixture = Fixture {
        directory,
        child: None,
    };
    let (copy, paste) = match name {
        "copy_nonzero_is_error" => ("/bin/cat >/dev/null\nexit 17", "exit 0"),
        "paste_nonzero_is_error" => ("exit 0", "printf partial\nexit 19"),
        "copy_has_deadline" | "cleanup_cancels_owned_child" => ("exec /bin/sleep 30", "exit 0"),
        "paste_has_deadline" => ("exit 0", "exec /bin/sleep 30"),
        "invalid_utf8_is_error" => ("exit 0", "printf '\\377'"),
        "oversized_output_is_error" => (
            "exit 0",
            "/bin/dd if=/dev/zero bs=1048576 count=65 2>/dev/null",
        ),
        "inherited_output_handles_do_not_block" => (
            "/bin/cat > \"$RTUI_CLIPBOARD_STORE\"",
            "/bin/sleep 30 &\n/bin/cat \"$RTUI_CLIPBOARD_STORE\"",
        ),
        _ => (
            "/bin/cat > \"$RTUI_CLIPBOARD_STORE\"",
            "/bin/cat \"$RTUI_CLIPBOARD_STORE\"",
        ),
    };
    let which = if name == "detection_does_not_spawn_which" {
        "exec /bin/sleep 30".to_owned()
    } else {
        "test -x \"${PATH}/$1\"".to_owned()
    };
    for (tool, body) in [
        ("which", which.as_str()),
        ("wl-copy", copy),
        ("wl-paste", paste),
    ] {
        if name == "missing_backend_is_error" && tool != "which" {
            continue;
        }
        let path = fixture.directory.join(tool);
        fs::write(
            &path,
            format!("#!/bin/sh\necho $$ >> \"$RTUI_CLIPBOARD_PID\"\n{body}\n"),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let output_path = fixture.directory.join("output");
    let output = fs::File::create(&output_path).unwrap();
    fixture.child = Some(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture"])
            .env("RTUI_API_CLIPBOARD_CHILD", name)
            .env("RTUI_CLIPBOARD_STORE", fixture.directory.join("stored"))
            .env("RTUI_CLIPBOARD_PID", fixture.directory.join("pid"))
            .env("WAYLAND_DISPLAY", "fixture")
            .env_remove("DISPLAY")
            .env("PATH", &fixture.directory)
            .stdin(Stdio::null())
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = fixture.child.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "{name}: clipboard call exceeded the outer 30-second safety deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(&output_path).unwrap();
    assert!(status.success(), "{name}: {output}");
    assert!(
        output.contains("1 passed"),
        "fixture did not execute its test: {output}"
    );
    true
}

fn assert_child_reaped() {
    let pid: i32 = fs::read_to_string(std::env::var("RTUI_CLIPBOARD_PID").unwrap())
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(
        unsafe { libc::kill(pid, 0) },
        -1,
        "owned clipboard child remains alive or unreaped: {pid}"
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[test]
fn copy_nonzero_is_error() {
    if isolated("copy_nonzero_is_error") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, _) = use_clipboard(&hooks);
    copy("payload");
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("17")));
    assert!(state.get().content.is_none());
    assert_child_reaped();
}

#[test]
fn paste_nonzero_is_error() {
    if isolated("paste_nonzero_is_error") {
        return;
    }
    let hooks = Hooks::new();
    let (state, _, paste) = use_clipboard(&hooks);
    assert_eq!(paste(), None);
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("19")));
    assert!(state.get().content.is_none());
    assert_child_reaped();
}

#[test]
fn missing_backend_is_error() {
    if isolated("missing_backend_is_error") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    copy("payload");
    assert!(state.get().error.is_some());
    assert!(state.get().content.is_none());
    assert_eq!(paste(), None);
}

#[test]
fn copy_has_deadline() {
    if isolated("copy_has_deadline") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, _) = use_clipboard(&hooks);
    let start = Instant::now();
    copy(&"x".repeat(1024 * 1024)); // Larger than the pipe buffer, never read.
    assert!(start.elapsed() < Duration::from_secs(3));
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("timed out")));
    assert!(state.get().content.is_none());
    assert_child_reaped();
}

#[test]
fn paste_has_deadline() {
    if isolated("paste_has_deadline") {
        return;
    }
    let hooks = Hooks::new();
    let (state, _, paste) = use_clipboard(&hooks);
    let start = Instant::now();
    assert_eq!(paste(), None);
    assert!(start.elapsed() < Duration::from_secs(3));
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("timed out")));
    assert_child_reaped();
}

#[test]
fn cleanup_cancels_owned_child() {
    if isolated("cleanup_cancels_owned_child") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, _) = use_clipboard(&hooks);
    let cleanup = hooks.clone();
    let pid_file = PathBuf::from(std::env::var("RTUI_CLIPBOARD_PID").unwrap());
    let task = std::thread::spawn(move || {
        // Cancel once the owned child has started (it writes its pid first),
        // not after a fixed delay that a busy machine can overrun.
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline
            && !fs::read_to_string(&pid_file)
                .is_ok_and(|pids| pids.lines().any(|pid| pid.trim().parse::<i32>().is_ok()))
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        let cancelled = Instant::now();
        cleanup.cleanup();
        cancelled
    });
    copy("payload");
    let cancelled = task.join().unwrap();
    // Cleanup ends the pending copy promptly.
    assert!(cancelled.elapsed() < Duration::from_secs(1));
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("cancelled")));
    assert_child_reaped();
    copy("after cleanup");
    assert_child_reaped();
}

#[test]
fn detection_does_not_spawn_which() {
    if isolated("detection_does_not_spawn_which") {
        return;
    }
    let hooks = Hooks::new();
    let start = Instant::now();
    let (state, copy, paste) = use_clipboard(&hooks);
    copy("quote ' 界 e\u{301} 🙂\nsecond line\n");
    assert_eq!(
        paste(),
        Some("quote ' 界 e\u{301} 🙂\nsecond line\n".into())
    );
    assert!(state.get().error.is_none());
    // The `which` fixture sleeps 30 s: finishing well inside that shows it
    // never ran, with room for a busy machine.
    assert!(start.elapsed() < Duration::from_secs(10));
}

#[test]
fn unicode_roundtrip_retains_newlines() {
    if isolated("unicode_roundtrip_retains_newlines") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    let text = "quote ' 界 e\u{301} 🙂\nsecond line\n";
    copy(text);
    assert_eq!(paste(), Some(text.into()));
    assert_eq!(state.get().content, Some(text.into()));
    assert!(state.get().error.is_none());
    assert_child_reaped();
}

#[test]
fn invalid_utf8_is_error() {
    if isolated("invalid_utf8_is_error") {
        return;
    }
    let hooks = Hooks::new();
    let (state, _, paste) = use_clipboard(&hooks);
    assert_eq!(paste(), None);
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("UTF-8")));
    assert_child_reaped();
}

#[test]
fn oversized_output_is_error() {
    if isolated("oversized_output_is_error") {
        return;
    }
    let hooks = Hooks::new();
    let (state, _, paste) = use_clipboard(&hooks);
    assert_eq!(paste(), None);
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("64 MiB")));
    assert_child_reaped();
}

#[test]
fn inherited_output_handles_do_not_block() {
    if isolated("inherited_output_handles_do_not_block") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    copy("retained pipe control");
    let start = Instant::now();
    assert_eq!(paste(), Some("retained pipe control".into()));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(state.get().error.is_none());
    assert_child_reaped();
}

#[test]
fn dropping_hook_owner_prevents_new_commands() {
    if isolated("dropping_hook_owner_prevents_new_commands") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    drop(hooks);
    copy("after owner drop");
    assert_eq!(paste(), None);
    assert!(state
        .get()
        .error
        .as_deref()
        .is_some_and(|error| error.contains("cancelled")));
    assert!(!PathBuf::from(std::env::var("RTUI_CLIPBOARD_PID").unwrap()).exists());
}

#[test]
fn failed_copy_retains_last_successful_cache() {
    if isolated("failed_copy_retains_last_successful_cache") {
        return;
    }
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    copy("saved text");
    let path = PathBuf::from(std::env::var("PATH").unwrap()).join("wl-copy");
    fs::write(&path, "#!/bin/sh\nexit 17\n").unwrap();
    copy("failed replacement");
    assert_eq!(state.get().content, Some("saved text".into()));
    assert!(state.get().error.is_some());
    assert_eq!(paste(), Some("saved text".into()));
    assert!(state.get().error.is_none());
}
