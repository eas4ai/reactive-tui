use crate::reactive::hooks::{Hooks, ThreadSafeSignal, use_signal};
use std::process::Command;
use std::sync::Arc;

// Type aliases for clipboard operations
type ClipboardWriter = Arc<dyn Fn(&str) + Send + Sync>;
type ClipboardReader = Arc<dyn Fn() -> Option<String> + Send + Sync>;

/// Clipboard backend detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardBackend {
    Wayland, // wl-copy/wl-paste
    Xsel,    // xsel
    Xclip,   // xclip
    #[cfg(target_os = "macos")]
    MacOS, // pbcopy/pbpaste
    #[cfg(target_os = "windows")]
    Windows, // clip.exe/powershell
    NoOp,    // Fallback when no clipboard available
}

impl ClipboardBackend {
    /// Check if a command exists
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Detect available clipboard backend
    fn detect() -> Self {
        // Check for Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok()
            && Self::command_exists("wl-copy")
            && Self::command_exists("wl-paste")
        {
            return ClipboardBackend::Wayland;
        }

        // Check for X11 tools
        if Self::command_exists("xsel") {
            return ClipboardBackend::Xsel;
        }
        if Self::command_exists("xclip") {
            return ClipboardBackend::Xclip;
        }

        // Platform-specific checks
        #[cfg(target_os = "macos")]
        {
            if Self::command_exists("pbcopy") && Self::command_exists("pbpaste") {
                return ClipboardBackend::MacOS;
            }
        }

        #[cfg(target_os = "windows")]
        {
            return ClipboardBackend::Windows;
        }

        // No clipboard available
        ClipboardBackend::NoOp
    }

    /// Copy text to clipboard
    fn copy(&self, text: &str) -> Result<(), String> {
        match self {
            ClipboardBackend::Wayland => {
                let mut child = Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn wl-copy: {e}"))?;

                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin
                        .write_all(text.as_bytes())
                        .map_err(|e| format!("Failed to write to wl-copy: {e}"))?;
                }

                child
                    .wait()
                    .map_err(|e| format!("Failed to wait for wl-copy: {e}"))?;
                Ok(())
            }

            ClipboardBackend::Xsel => {
                let mut child = Command::new("xsel")
                    .arg("-ib")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn xsel: {e}"))?;

                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin
                        .write_all(text.as_bytes())
                        .map_err(|e| format!("Failed to write to xsel: {e}"))?;
                }

                child
                    .wait()
                    .map_err(|e| format!("Failed to wait for xsel: {e}"))?;
                Ok(())
            }

            ClipboardBackend::Xclip => {
                let mut child = Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn xclip: {e}"))?;

                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin
                        .write_all(text.as_bytes())
                        .map_err(|e| format!("Failed to write to xclip: {e}"))?;
                }

                child
                    .wait()
                    .map_err(|e| format!("Failed to wait for xclip: {e}"))?;
                Ok(())
            }

            #[cfg(target_os = "macos")]
            ClipboardBackend::MacOS => {
                let mut child = Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn pbcopy: {}", e))?;

                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin
                        .write_all(text.as_bytes())
                        .map_err(|e| format!("Failed to write to pbcopy: {}", e))?;
                }

                child
                    .wait()
                    .map_err(|e| format!("Failed to wait for pbcopy: {}", e))?;
                Ok(())
            }

            #[cfg(target_os = "windows")]
            ClipboardBackend::Windows => {
                // Use PowerShell to set clipboard
                Command::new("powershell")
                    .arg("-Command")
                    .arg(format!(
                        "Set-Clipboard -Value '{}'",
                        text.replace("'", "''")
                    ))
                    .output()
                    .map_err(|e| format!("Failed to set clipboard via PowerShell: {}", e))?;
                Ok(())
            }

            _ => Ok(()), // NoOp and non-platform specific variants
        }
    }

    /// Paste text from clipboard
    fn paste(&self) -> Result<String, String> {
        match self {
            ClipboardBackend::Wayland => {
                let output = Command::new("wl-paste")
                    .output()
                    .map_err(|e| format!("Failed to run wl-paste: {e}"))?;

                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Invalid UTF-8 from wl-paste: {e}"))
            }

            ClipboardBackend::Xsel => {
                let output = Command::new("xsel")
                    .arg("-ob")
                    .output()
                    .map_err(|e| format!("Failed to run xsel: {e}"))?;

                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Invalid UTF-8 from xsel: {e}"))
            }

            ClipboardBackend::Xclip => {
                let output = Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .arg("-o")
                    .output()
                    .map_err(|e| format!("Failed to run xclip: {e}"))?;

                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Invalid UTF-8 from xclip: {e}"))
            }

            #[cfg(target_os = "macos")]
            ClipboardBackend::MacOS => {
                let output = Command::new("pbpaste")
                    .output()
                    .map_err(|e| format!("Failed to run pbpaste: {}", e))?;

                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Invalid UTF-8 from pbpaste: {}", e))
            }

            #[cfg(target_os = "windows")]
            ClipboardBackend::Windows => {
                let output = Command::new("powershell")
                    .arg("-Command")
                    .arg("Get-Clipboard")
                    .output()
                    .map_err(|e| format!("Failed to get clipboard via PowerShell: {}", e))?;

                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Invalid UTF-8 from PowerShell: {}", e))
                    .map(|s| s.trim_end().to_string())
            }

            _ => Ok(String::new()), // NoOp returns empty string
        }
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
/// ```rust
/// fn CopyButton(props: &Props, state: &State) -> Element {
///     let (clipboard, copy, paste) = use_clipboard(&hooks);
///     
///     Element::button()
///         .class("bg-blue-500 hover:bg-blue-700 text-white px-4 py-2")
///         .on_click(move |_| copy("Hello, clipboard!"))
///         .child(text!("Copy to clipboard"))
/// }
/// ```
pub fn use_clipboard(
    hooks: &Hooks,
) -> (
    ThreadSafeSignal<ClipboardState>,
    ClipboardWriter,
    ClipboardReader,
) {
    let state = use_signal(hooks, ClipboardState::default());
    let state_copy = state.clone();
    let state_paste = state.clone();

    // Copy function
    let copy = Arc::new(move |text: &str| {
        let mut current = state_copy.get();
        match current.backend.copy(text) {
            Ok(()) => {
                current.content = Some(text.to_string());
                current.error = None;
            }
            Err(e) => {
                current.error = Some(e);
            }
        }
        state_copy.set(current);
    }) as Arc<dyn Fn(&str) + Send + Sync>;

    // Paste function
    let paste = Arc::new(move || -> Option<String> {
        let mut current = state_paste.get();
        match current.backend.paste() {
            Ok(text) => {
                current.content = Some(text.clone());
                current.error = None;
                state_paste.set(current);
                Some(text)
            }
            Err(e) => {
                current.error = Some(e);
                state_paste.set(current);
                None
            }
        }
    }) as Arc<dyn Fn() -> Option<String> + Send + Sync>;

    (state, copy, paste)
}

/// Simplified clipboard hook that just returns copy and paste functions
///
/// # Example
/// ```rust
/// fn TextEditor(props: &Props, state: &mut State) -> Element {
///     let (copy, paste) = use_simple_clipboard(&hooks);
///     
///     Element::textarea()
///         .on_key_down(move |e| {
///             if e.modifiers.ctrl && e.code == KeyCode::Char('c') {
///                 copy(&state.selected_text);
///             } else if e.modifiers.ctrl && e.code == KeyCode::Char('v') {
///                 if let Some(text) = paste() {
///                     state.insert_text(text);
///                 }
///             }
///         })
/// }
/// ```
pub fn use_simple_clipboard(hooks: &Hooks) -> (ClipboardWriter, ClipboardReader) {
    let (_state, copy, paste) = use_clipboard(hooks);
    (copy, paste)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_backend_detection() {
        let backend = ClipboardBackend::detect();
        // Should detect something, even if it's NoOp
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
        let hooks = Hooks::new();
        let (state, copy, _paste) = use_clipboard(&hooks);

        // Initial state
        assert!(state.get().content.is_none());

        // Copy some text (may fail if no clipboard available)
        copy("Test text");

        // Check if it was copied (depends on backend availability)
        let current = state.get();
        if current.backend != ClipboardBackend::NoOp {
            assert_eq!(current.content, Some("Test text".to_string()));
        }
    }

    #[test]
    fn test_use_simple_clipboard() {
        let hooks = Hooks::new();
        let (copy, paste) = use_simple_clipboard(&hooks);

        // Should be able to call functions without error
        copy("Test");
        let _result = paste();
    }
}
