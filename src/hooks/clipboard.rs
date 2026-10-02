//! Clipboard hooks with bounded commands and cancellation on owner cleanup.

use crate::reactive::hooks::{HookKind, Hooks, ThreadSafeSignal};
use std::process::Command;
use std::sync::Arc;

// Preserve the existing synchronous callback signatures.
type ClipboardWriter = Arc<dyn Fn(&str) + Send + Sync>;
type ClipboardReader = Arc<dyn Fn() -> Option<String> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardBackend {
    Wayland,
    Xsel,
    Xclip,
    #[cfg(target_os = "macos")]
    MacOS,
    #[cfg(target_os = "windows")]
    Windows,
    Unavailable,
}

impl ClipboardBackend {
    fn command_exists(program: &str) -> bool {
        let Some(path) = std::env::var_os("PATH") else {
            return false;
        };
        std::env::split_paths(&path).any(|directory| {
            #[cfg(windows)]
            let program = format!("{program}.exe");
            let Ok(metadata) = std::fs::metadata(directory.join(program)) else {
                return false;
            };
            if !metadata.is_file() {
                return false;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                true
            }
        })
    }

    fn detect() -> Self {
        #[cfg(target_os = "windows")]
        if Self::command_exists("powershell") {
            return Self::Windows;
        }
        #[cfg(target_os = "macos")]
        if Self::command_exists("pbcopy") && Self::command_exists("pbpaste") {
            return Self::MacOS;
        }
        if std::env::var_os("WAYLAND_DISPLAY").is_some_and(|display| !display.is_empty())
            && Self::command_exists("wl-copy")
            && Self::command_exists("wl-paste")
        {
            return Self::Wayland;
        }
        if std::env::var_os("DISPLAY").is_some_and(|display| !display.is_empty()) {
            if Self::command_exists("xsel") {
                return Self::Xsel;
            }
            if Self::command_exists("xclip") {
                return Self::Xclip;
            }
        }
        Self::Unavailable
    }

    fn command(&self, copy: bool) -> Result<Command, String> {
        let command = match self {
            Self::Wayland => {
                let mut command = Command::new(if copy { "wl-copy" } else { "wl-paste" });
                if copy {
                    command.args(["--type", "text/plain;charset=utf-8"]);
                } else {
                    command.arg("--no-newline");
                }
                command
            }
            Self::Xsel => {
                let mut command = Command::new("xsel");
                command.arg(if copy { "-ib" } else { "-ob" });
                command
            }
            Self::Xclip => {
                let mut command = Command::new("xclip");
                command.args(["-selection", "clipboard"]);
                if !copy {
                    command.arg("-o");
                }
                command
            }
            #[cfg(target_os = "macos")]
            Self::MacOS => {
                let mut command = Command::new(if copy { "pbcopy" } else { "pbpaste" });
                command.env("LC_CTYPE", "en_US.UTF-8");
                command
            }
            #[cfg(target_os = "windows")]
            Self::Windows => {
                let mut command = Command::new("powershell");
                let script = if copy {
                    "$ErrorActionPreference='Stop'; [Console]::InputEncoding=[System.Text.UTF8Encoding]::new($false); $text=[Console]::In.ReadToEnd(); if ($text.Length -eq 0) { Set-Clipboard -Value $null } else { Set-Clipboard -Value $text }"
                } else {
                    "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); [Console]::Write([string](Get-Clipboard -Raw))"
                };
                command.args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    script,
                ]);
                command
            }
            Self::Unavailable => {
                return Err("No clipboard backend is available for this desktop session".into())
            }
        };
        Ok(command)
    }

    fn copy(&self, text: &str, cancelled: impl Fn() -> bool) -> Result<(), String> {
        super::clipboard_process::run(self.command(true)?, Some(text), cancelled).map(|_| ())
    }

    fn paste(&self, cancelled: impl Fn() -> bool) -> Result<String, String> {
        let output = super::clipboard_process::run(self.command(false)?, None, cancelled)?;
        String::from_utf8(output)
            .map_err(|error| format!("Clipboard returned invalid UTF-8: {error}"))
    }
}

/// Clipboard state
#[derive(Clone, Debug, PartialEq)]
pub struct ClipboardState {
    /// Current clipboard content (cached)
    pub content: Option<String>,
    /// Last error if any
    pub error: Option<String>,
    /// Backend being used
    backend: ClipboardBackend,
}

impl ClipboardState {
    /// Selected desktop backend, or `unavailable` when no usable tool was found.
    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            ClipboardBackend::Wayland => "wayland",
            ClipboardBackend::Xsel => "xsel",
            ClipboardBackend::Xclip => "xclip",
            #[cfg(target_os = "macos")]
            ClipboardBackend::MacOS => "macos",
            #[cfg(target_os = "windows")]
            ClipboardBackend::Windows => "windows",
            ClipboardBackend::Unavailable => "unavailable",
        }
    }
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self {
            content: None,
            error: None,
            backend: ClipboardBackend::detect(),
        }
    }
}

/// React-style clipboard hook
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn CopyButton(hooks: &Hooks) -> Element {
///     let (clipboard, copy, _paste) = use_clipboard(hooks);
///     let status = clipboard.get();
///     button().on_click(move || copy("Hello, clipboard!"))
///         .child(Element::text(if status.error.is_some() { "Copy failed" } else { "Copy" }))
///         .build()
/// }
/// ```
pub fn use_clipboard(
    hooks: &Hooks,
) -> (
    ThreadSafeSignal<ClipboardState>,
    ClipboardWriter,
    ClipboardReader,
) {
    // Detection happens only when the retained signal slot is first created.
    let storage = hooks.get_or_create_storage(HookKind::Signal, || {
        ThreadSafeSignal::new(ClipboardState::default())
    });
    let state = storage
        .lock()
        .expect("clipboard state lock poisoned")
        .clone();
    let state_copy = state.clone();
    let state_paste = state.clone();
    let copy_owner = hooks.liveness();
    let paste_owner = copy_owner.clone();

    let copy = Arc::new(move |text: &str| {
        let backend = state_copy.get().backend;
        let result = backend.copy(text, || !copy_owner.is_alive());
        if result.is_err() {
            // Keep copied text and tool output out of logs. The full hook exposes
            // its error signal; the simple hook retains its void copy callback.
            log::warn!("Clipboard copy failed; inspect use_clipboard's error state for details");
        }
        state_copy.update(|current| match result {
            Ok(()) => {
                current.content = Some(text.to_owned());
                current.error = None;
            }
            Err(error) => current.error = Some(error),
        });
    }) as ClipboardWriter;

    let paste = Arc::new(move || -> Option<String> {
        let backend = state_paste.get().backend;
        match backend.paste(|| !paste_owner.is_alive()) {
            Ok(text) => {
                state_paste.update(|current| {
                    current.content = Some(text.clone());
                    current.error = None;
                });
                Some(text)
            }
            Err(error) => {
                state_paste.update(|current| current.error = Some(error));
                None
            }
        }
    }) as ClipboardReader;

    (state, copy, paste)
}

/// Simplified clipboard hook that just returns copy and paste functions
///
/// # Example
/// ```rust,no_run
/// use reactive_tui::hooks::use_simple_clipboard;
/// use reactive_tui::reactive::Hooks;
///
/// fn copy_then_paste(hooks: &Hooks, selected: &str) -> Option<String> {
///     let (copy, paste) = use_simple_clipboard(hooks);
///     copy(selected);
///     paste()
/// }
/// // Use use_clipboard when the caller needs the error signal as well.
/// ```
pub fn use_simple_clipboard(hooks: &Hooks) -> (ClipboardWriter, ClipboardReader) {
    let (_state, copy, paste) = use_clipboard(hooks);
    (copy, paste)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Keep real backend detection and command I/O, but give each Unix test its
    // own clipboard commands. Only the child environment changes.
    #[cfg(unix)]
    fn run_with_clipboard_fixture(test: &str) -> bool {
        use std::os::unix::{fs::PermissionsExt, process::CommandExt};
        use std::time::{Duration, Instant};

        if std::env::var("RTUI_CLIPBOARD_TEST_CHILD").as_deref() == Ok(test) {
            return false;
        }

        struct Fixture {
            directory: std::path::PathBuf,
            child: Option<std::process::Child>,
        }
        impl Drop for Fixture {
            fn drop(&mut self) {
                if let Some(child) = &mut self.child {
                    if !matches!(child.try_wait(), Ok(Some(_))) {
                        // The child owns this process group; stop any fixture
                        // command too if the tested code fails to return.
                        unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
                        let _ = child.wait();
                    }
                }
                if let Err(error) = std::fs::remove_dir_all(&self.directory) {
                    eprintln!("Could not remove clipboard test fixture: {error}");
                }
            }
        }

        let directory =
            std::env::temp_dir().join(format!("rtui-clipboard-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let mut fixture = Fixture {
            directory,
            child: None,
        };
        for (name, script) in [
            (
                "which",
                "#!/bin/sh\ncase \"$1\" in wl-copy|wl-paste) exit 0;; *) exit 1;; esac\n",
            ),
            (
                "wl-copy",
                "#!/bin/sh\n/bin/cat > \"$RTUI_CLIPBOARD_FIXTURE_FILE\"\n",
            ),
            (
                "wl-paste",
                "#!/bin/sh\n/bin/cat \"$RTUI_CLIPBOARD_FIXTURE_FILE\"\n",
            ),
        ] {
            let path = fixture.directory.join(name);
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let output_path = fixture.directory.join("output");
        let output = std::fs::File::create(&output_path).unwrap();
        fixture.child = Some(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    &format!("hooks::clipboard::tests::{test}"),
                    "--nocapture",
                ])
                .env("RTUI_CLIPBOARD_TEST_CHILD", test)
                .env(
                    "RTUI_CLIPBOARD_FIXTURE_FILE",
                    fixture.directory.join("clipboard"),
                )
                .env("WAYLAND_DISPLAY", "fixture")
                .env("PATH", &fixture.directory)
                .stdout(output.try_clone().unwrap())
                .stderr(output)
                .process_group(0)
                .spawn()
                .unwrap(),
        );
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = fixture.child.as_mut().unwrap().try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "clipboard fixture test exceeded 30 seconds"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        let output = std::fs::read_to_string(output_path).unwrap();
        assert!(status.success(), "clipboard child failed: {output}");
        assert!(
            output.contains("1 passed"),
            "clipboard child did not run its test: {output}"
        );
        true
    }

    #[test]
    fn test_clipboard_backend_detection() {
        let backend = ClipboardBackend::detect();
        // Detection is read-only, including an unavailable desktop, so it is stable.
        assert_eq!(backend, ClipboardBackend::detect());
        let has_display = ["WAYLAND_DISPLAY", "DISPLAY"]
            .iter()
            .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()));
        if !has_display && !cfg!(any(target_os = "macos", target_os = "windows")) {
            assert_eq!(
                backend,
                ClipboardBackend::Unavailable,
                "no desktop session means no clipboard backend"
            );
        }
        println!("Detected clipboard backend: {backend:?}");
    }

    #[test]
    fn test_clipboard_state_default() {
        let state = ClipboardState::default();
        assert!(state.content.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn test_use_clipboard() {
        #[cfg(unix)]
        if run_with_clipboard_fixture("test_use_clipboard") {
            return;
        }
        let hooks = Hooks::new();
        let (state, copy, paste) = use_clipboard(&hooks);

        // Initial state
        assert!(state.get().content.is_none());

        // Exercise the actual command stdin and the hook cache.
        copy("Test text");

        let current = state.get();
        assert_eq!(current.content, Some("Test text".to_string()));
        #[cfg(unix)]
        {
            assert_eq!(current.backend, ClipboardBackend::Wayland);
            assert_eq!(paste(), Some("Test text".into()));
            let text = "clipboard '界'\nsecond line";
            copy(text);
            assert_eq!(paste(), Some(text.into()));
            assert_eq!(state.get().content, Some(text.into()));
            assert!(state.get().error.is_none());
        }
        #[cfg(not(unix))]
        let _ = paste;
    }

    #[test]
    fn test_use_simple_clipboard() {
        #[cfg(unix)]
        if run_with_clipboard_fixture("test_use_simple_clipboard") {
            return;
        }
        let hooks = Hooks::new();
        let (copy, paste) = use_simple_clipboard(&hooks);

        // Should be able to call functions without error
        copy("Test");
        let result = paste();
        #[cfg(unix)]
        assert_eq!(result, Some("Test".into()));
        #[cfg(not(unix))]
        let _ = result;
    }
}
