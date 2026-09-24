//! Clipboard commands retain their platform deadlines and daemon ownership policy.

use std::process::Command;
use std::time::Duration;
#[cfg(test)]
use std::time::Instant;

#[cfg(not(windows))]
const TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(windows)]
const TIMEOUT: Duration = Duration::from_secs(15);

pub(super) fn run(
    command: Command,
    text: Option<&str>,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    crate::core::owned_process::run(
        command,
        text.map(str::as_bytes),
        crate::core::owned_process::Options {
            purpose: "clipboard",
            timeout: TIMEOUT,
            max_input: 64 * 1024 * 1024,
            max_output: 64 * 1024 * 1024,
            capture_output: text.is_none(),
            allow_background_after_success: text.is_some(),
        },
        cancelled,
    )
}

#[cfg(all(test, any(unix, windows)))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "child fixture invoked only by the process lifecycle tests"]
    fn smoke_sleeping_child_fixture() {
        let Some(path) = std::env::var_os("RTUI_PID_PATH") else {
            eprintln!("SKIP: a fixture the process lifecycle tests run with RTUI_PID_PATH set");
            return;
        };
        std::fs::write(path, std::process::id().to_string()).unwrap();
        std::thread::sleep(Duration::from_secs(30));
    }

    /// Stop a sleeping child by deadline or cancellation and return the error text.
    fn exercise_stop(cancel: bool) -> String {
        let directory = tempfile::tempdir().unwrap();
        let pid_path = directory.path().join("pid");
        // A native child isolates process ownership from shell startup latency.
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--ignored",
            "--exact",
            "hooks::clipboard_process::tests::smoke_sleeping_child_fixture",
        ]);
        #[cfg(windows)]
        let process = std::cell::RefCell::new(None::<std::os::windows::io::OwnedHandle>);
        command.env("RTUI_PID_PATH", &pid_path);
        let start = Instant::now();
        let error = run(command, None, || {
            let ready = std::fs::read_to_string(&pid_path)
                .ok()
                .and_then(|pid| pid.trim().parse::<u32>().ok());
            #[cfg(windows)]
            if let Some(pid) = ready {
                use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
                use windows_sys::Win32::{
                    Foundation::WAIT_TIMEOUT,
                    System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
                };
                if process.borrow().is_none() {
                    // Retain the exact live process identity across cancellation;
                    // querying a PID after exit could observe a reused identifier.
                    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
                    assert!(
                        !handle.is_null(),
                        "open child process: {}",
                        std::io::Error::last_os_error()
                    );
                    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
                    assert_eq!(
                        unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) },
                        WAIT_TIMEOUT,
                        "fixture must be live before stop"
                    );
                    *process.borrow_mut() = Some(handle);
                }
            }
            cancel && ready.is_some()
        })
        .unwrap_err();
        #[cfg(unix)]
        assert!(start.elapsed() < Duration::from_secs(3));
        #[cfg(windows)]
        assert!(start.elapsed() < Duration::from_secs(16));
        #[cfg(unix)]
        {
            let pid: i32 = std::fs::read_to_string(&pid_path)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "child remains live or unreaped"
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::{
                Foundation::WAIT_OBJECT_0, System::Threading::WaitForSingleObject,
            };
            let process = process.into_inner().expect("observed the live child");
            assert_eq!(
                unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
                WAIT_OBJECT_0,
                "owned Windows process must have exited"
            );
        }
        error
    }

    #[test]
    fn deadline_kills_and_reaps_child() {
        let error = exercise_stop(false);
        assert!(error.contains("timed out"), "{error}");
    }

    #[test]
    fn cancellation_kills_and_reaps_child() {
        let error = exercise_stop(true);
        assert!(error.contains("cancelled"), "{error}");
    }
}
