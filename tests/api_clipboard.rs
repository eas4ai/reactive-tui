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
            format!("#!/bin/sh\necho $$ > \"$RTUI_CLIPBOARD_PID\"\n{body}\n"),
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
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = fixture.child.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "{name}: clipboard call exceeded outer five-second safety deadline"
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
    let task = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        cleanup.cleanup();
    });
    let start = Instant::now();
    copy("payload");
    task.join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(1));
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
    assert!(start.elapsed() < Duration::from_secs(1));
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
