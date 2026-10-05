//! Native Windows acceptance probe for the retained PTY, including a real child.

#[cfg(windows)]
mod windows {
    use reactive_tui::terminal::{PseudoTerminal, TerminalConfig};
    use std::{
        io::{self, BufRead, Write},
        mem::zeroed,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::{self, Command},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::WAIT_OBJECT_0,
        System::{
            Console::{
                GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle,
                CONSOLE_SCREEN_BUFFER_INFO, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
            },
            Threading::{
                GetCurrentProcess, GetProcessHandleCount, OpenProcess, WaitForSingleObject,
                PROCESS_SYNCHRONIZE,
            },
        },
    };

    fn dimensions() -> (i16, i16) {
        // SAFETY: borrowed standard handles remain open; Win32 initializes info.
        unsafe {
            let output = GetStdHandle(STD_OUTPUT_HANDLE);
            let input = GetStdHandle(STD_INPUT_HANDLE);
            let mut mode = 0;
            assert_ne!(
                GetConsoleMode(input, &mut mode),
                0,
                "stdin is not a console"
            );
            assert_ne!(
                GetConsoleMode(output, &mut mode),
                0,
                "stdout is not a console"
            );
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
            assert_ne!(GetConsoleScreenBufferInfo(output, &mut info), 0);
            (info.dwSize.X, info.dwSize.Y)
        }
    }

    fn child(mode: &str) {
        let (width, height) = dimensions();
        println!("READY:{width}:{height}");
        println!("ENV:{}", std::env::var("REACTIVE_CONPTY_VALUE").unwrap());
        assert_eq!(
            std::env::current_dir().unwrap().canonicalize().unwrap(),
            std::path::Path::new(&std::env::var("REACTIVE_CONPTY_DIRECTORY").unwrap())
                .canonicalize()
                .unwrap()
        );
        println!("CWD_OK");
        io::stdout().flush().unwrap();
        if mode == "flood" {
            loop {
                io::stdout().write_all(&[b'X'; 4096]).unwrap();
            }
        }
        if mode == "tree" {
            let mut descendant = Command::new(std::env::current_exe().unwrap())
                .env("REACTIVE_CONPTY_CHILD", "idle")
                .spawn()
                .unwrap();
            println!("DESCENDANT:{}:END", descendant.id());
            io::stdout().flush().unwrap();
            let _ = descendant.wait();
            return;
        }
        if mode == "idle" {
            // The idle child: it never reads its input and never exits.
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        for line in io::stdin().lock().lines() {
            let line = line.unwrap();
            if line == "quit" {
                process::exit(259);
            }
            let (width, height) = dimensions();
            println!("INPUT:{line}:SIZE:{width}:{height}");
            io::stdout().flush().unwrap();
        }
    }

    fn config(mode: &str, directory: &std::path::Path) -> TerminalConfig {
        TerminalConfig {
            shell: Some(std::env::current_exe().unwrap().to_str().unwrap().into()),
            working_directory: Some(directory.to_str().unwrap().into()),
            env: vec![
                ("REACTIVE_CONPTY_CHILD".into(), mode.into()),
                ("REACTIVE_CONPTY_VALUE".into(), "wrong".into()),
                ("reactive_conpty_value".into(), "configured".into()),
                (
                    "REACTIVE_CONPTY_DIRECTORY".into(),
                    directory.to_str().unwrap().into(),
                ),
            ],
            size: (80, 24),
            ..Default::default()
        }
    }

    fn read_until(pty: &PseudoTerminal, marker: &str) -> String {
        // A hang guard, not a timing check: generous so a busy machine cannot
        // fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut bytes = Vec::new();
        while Instant::now() < deadline {
            if let Some(chunk) = pty.read_output(Some(Duration::from_millis(20))).unwrap() {
                bytes.extend(chunk);
                assert!(bytes.len() <= 256 * 1024, "unexpected unbounded output");
                let text = String::from_utf8_lossy(&bytes);
                if text.contains(marker) {
                    return text.into_owned();
                }
            }
        }
        panic!(
            "missing {marker:?}; output: {:?}; exit: {:?}",
            String::from_utf8_lossy(&bytes),
            pty.try_wait()
        );
    }

    fn process_handle(pid: u32) -> OwnedHandle {
        // SAFETY: OpenProcess returns a new owned handle; only wait rights requested.
        let raw = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        assert!(
            !raw.is_null(),
            "OpenProcess failed: {}",
            io::Error::last_os_error()
        );
        unsafe { OwnedHandle::from_raw_handle(raw) }
    }

    fn assert_exited(handle: &OwnedHandle) {
        // SAFETY: the owned process handle remains valid throughout the wait.
        assert_eq!(
            unsafe { WaitForSingleObject(handle.as_raw_handle(), 5000) },
            WAIT_OBJECT_0
        );
    }

    fn handle_count() -> u32 {
        let mut count = 0;
        // SAFETY: current process pseudo-handle is borrowed and out pointer valid.
        assert_ne!(
            unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) },
            0
        );
        count
    }

    pub(super) fn main() {
        if let Ok(mode) = std::env::var("REACTIVE_CONPTY_CHILD") {
            child(&mode);
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let mut pty = PseudoTerminal::new();
        assert!(pty.write_input(b"unstarted").is_err());
        let normal = config("normal", directory.path());
        pty.spawn(&normal).unwrap();
        assert!(pty.spawn(&normal).is_err());
        let process = process_handle(pty.child_id().unwrap());
        let text = read_until(&pty, "CWD_OK");
        assert!(text.contains("READY:80:24"), "{text:?}");
        assert!(text.contains("ENV:configured"), "{text:?}");
        // Ask for an independent post-start marker after the configured directory
        // output, which can be split into several ConPTY chunks.
        pty.write_input(b"hello\r").unwrap();
        read_until(&pty, "INPUT:hello:SIZE:80:24");
        pty.resize(100, 30).unwrap();
        assert_eq!(pty.size(), (100, 30));
        assert!(pty.resize(32768, 1).is_err());
        assert_eq!(pty.size(), (100, 30));
        pty.write_input(b"resized\r").unwrap();
        read_until(&pty, "INPUT:resized:SIZE:100:30");
        pty.write_input(b"quit\r").unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while pty.try_wait().unwrap().is_none() && Instant::now() < deadline {
            pty.read_output(Some(Duration::from_millis(20))).unwrap();
        }
        assert_eq!(pty.try_wait().unwrap(), Some(259));
        assert_eq!(pty.try_wait().unwrap(), Some(259));
        assert_exited(&process);
        assert!(pty.write_input(b"stopped").is_err());
        pty.kill().unwrap();
        pty.kill().unwrap();
        drop(process);
        println!("PASS console handles, input, environment, resize, real exit 259, repeated stop");

        let missing = directory.path().join("missing.exe");
        // Windows 11 lazily retains process-launch bookkeeping on the first
        // missing executable, independently of ConPTY. Initialize that OS path
        // before measuring our first failed PTY launch; keep its handles counted.
        assert!(std::process::Command::new(&missing).spawn().is_err());
        let before = handle_count();
        for attempt in 0..10 {
            let mut bad = normal.clone();
            bad.shell = Some(missing.to_str().unwrap().into());
            assert!(pty.spawn(&bad).is_err());
            println!(
                "failed launch {attempt}: handles {} (baseline {before})",
                handle_count()
            );
        }
        assert!(
            handle_count() <= before + 2,
            "failed launches leaked handles: before {before}, after {}",
            handle_count()
        );
        pty.spawn(&normal).unwrap();
        read_until(&pty, "READY:80:24");
        pty.kill().unwrap();
        println!("PASS failed-launch cleanup and restart");

        for mode in ["flood", "idle"] {
            let mut busy = PseudoTerminal::new();
            busy.spawn(&config(mode, directory.path())).unwrap();
            let process = process_handle(busy.child_id().unwrap());
            read_until(&busy, "READY:80:24");
            if mode == "idle" {
                let chunk = vec![b'a'; 65536];
                let mut rejected = false;
                for _ in 0..32 {
                    if busy.write_input(&chunk).is_err() {
                        rejected = true;
                        break;
                    }
                }
                assert!(rejected, "input remained unbounded");
            }
            // Setup for the behavior under test: give the flood time to fill
            // the output pipe, so the drop below meets backpressure.
            std::thread::sleep(Duration::from_millis(100));
            let started = Instant::now();
            drop(busy);
            // The behavior under test: dropping ends the child at once.
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "{mode} shutdown stalled"
            );
            assert_exited(&process);
            println!("PASS {mode} backpressure and drop cleanup");
        }

        println!("BEGIN attached descendant launch");
        let mut tree = PseudoTerminal::new();
        tree.spawn(&config("tree", directory.path())).unwrap();
        println!("BEGIN attached descendant read");
        let process = process_handle(tree.child_id().unwrap());
        let text = read_until(&tree, ":END");
        let pid: u32 = text
            .split("DESCENDANT:")
            .nth(1)
            .unwrap()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .unwrap();
        let descendant = process_handle(pid);
        println!("BEGIN attached descendant close");
        tree.kill().unwrap();
        println!("END attached descendant close");
        assert_exited(&process);
        assert_exited(&descendant);
        println!("PASS attached descendant cleanup");
    }
}

#[cfg(windows)]
fn main() {
    windows::main();
}

#[cfg(not(windows))]
fn main() {
    panic!("conpty_probe requires native Windows execution");
}
