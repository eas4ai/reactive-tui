#![cfg(windows)]

use reactive_tui::platform::{
    windows::WindowsTty, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    TerminalEvent,
};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use windows_sys::Win32::{Foundation::*, System::Console::*, UI::Input::KeyboardAndMouse::*};

fn key(unit: u16, down: bool, repeat: u16, virtual_key: u16) -> INPUT_RECORD {
    INPUT_RECORD {
        EventType: KEY_EVENT as u16,
        Event: INPUT_RECORD_0 {
            KeyEvent: KEY_EVENT_RECORD {
                bKeyDown: i32::from(down),
                wRepeatCount: repeat,
                wVirtualKeyCode: virtual_key,
                wVirtualScanCode: 0,
                uChar: KEY_EVENT_RECORD_0 { UnicodeChar: unit },
                dwControlKeyState: 0,
            },
        },
    }
}

fn mouse(flags: u32, buttons: u32) -> INPUT_RECORD {
    INPUT_RECORD {
        EventType: MOUSE_EVENT as u16,
        Event: INPUT_RECORD_0 {
            MouseEvent: MOUSE_EVENT_RECORD {
                dwMousePosition: COORD { X: 7, Y: 3 },
                dwButtonState: buttons,
                dwControlKeyState: SHIFT_PRESSED | LEFT_CTRL_PRESSED,
                dwEventFlags: flags,
            },
        },
    }
}

fn inject(input: HANDLE, records: &[INPUT_RECORD]) {
    let mut written = 0;
    assert_ne!(
        unsafe { WriteConsoleInputW(input, records.as_ptr(), records.len() as u32, &mut written) },
        0
    );
    assert_eq!(written as usize, records.len());
}

fn check_private_console() {
    let slots = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE];
    let inherited = slots.map(|slot| unsafe { GetStdHandle(slot) });
    // The parent redirects this detached child to NUL. Clear only this child's
    // standard-handle slots so AllocConsole installs handles for its new console.
    for handle in slots {
        assert_ne!(unsafe { SetStdHandle(handle, std::ptr::null_mut()) }, 0);
    }
    assert_ne!(unsafe { AllocConsole() }, 0, "allocate a private console");
    struct Console([HANDLE; 3]);
    impl Drop for Console {
        fn drop(&mut self) {
            let freed = unsafe { FreeConsole() };
            // The test harness still needs its redirected output after detaching.
            let mut restored = true;
            for (slot, handle) in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE]
                .into_iter()
                .zip(self.0)
            {
                restored &= unsafe { SetStdHandle(slot, handle) } != 0;
            }
            if !std::thread::panicking() {
                assert_ne!(freed, 0);
                assert!(restored);
            }
        }
    }
    let _console = Console(inherited);
    let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    let mut original_mode = 0;
    assert_ne!(unsafe { GetConsoleMode(input, &mut original_mode) }, 0);
    let tty = WindowsTty::init().expect("native console initialization");

    assert_ne!(unsafe { FlushConsoleInputBuffer(input) }, 0);
    let started = Instant::now();
    let error = tty
        .read(&mut [0; 8], Some(Duration::from_millis(50)))
        .unwrap_err();
    assert!(
        matches!(error, reactive_tui::error::ReactiveError::Io(ref error)
        if error.kind() == std::io::ErrorKind::TimedOut)
    );
    assert!(started.elapsed() < Duration::from_secs(1));

    inject(
        input,
        &[
            key(b'a' as u16, true, 2, 0),
            key('界' as u16, true, 1, 0),
            key(0xD83D, true, 1, 0),
            key(0xDE00, true, 1, 0),
        ],
    );
    assert_eq!(tty.read(&mut [], Some(Duration::ZERO)).unwrap(), 0);
    let expected = "aa界😀".as_bytes();
    let mut observed = Vec::new();
    for _ in 0..expected.len() {
        let mut byte = [0];
        assert_eq!(
            tty.read(&mut byte, Some(Duration::from_millis(100)))
                .unwrap_or_else(|error| panic!(
                    "byte {} of {expected:?}: {error}; received {observed:?}",
                    observed.len()
                )),
            1
        );
        observed.push(byte[0]);
    }
    assert_eq!(observed, expected);

    let records = [
        key(b'a' as u16, true, 1, 65),
        key(b'a' as u16, false, 1, 65),
        key('界' as u16, true, 1, 0),
        key(0xD83D, true, 1, 0),
        key(0xDE00, true, 1, 0),
        key(0, true, 1, VK_F24),
        key(b'x' as u16, true, 3, 88),
    ];
    let events: Vec<_> = records
        .iter()
        .filter_map(|r| tty.parse_input_record(r))
        .collect();
    let expected = [
        (KeyCode::Char('a'), KeyEventKind::Press),
        (KeyCode::Char('a'), KeyEventKind::Release),
        (KeyCode::Char('界'), KeyEventKind::Press),
        (KeyCode::Char('😀'), KeyEventKind::Press),
        (KeyCode::F(24), KeyEventKind::Press),
        (KeyCode::Char('x'), KeyEventKind::Repeat),
    ]
    .into_iter()
    .map(|(code, kind)| TerminalEvent::Key {
        code,
        kind,
        modifiers: KeyModifiers::empty(),
    })
    .collect::<Vec<_>>();
    assert_eq!(events, expected);
    for (record, kind, button) in [
        (mouse(MOUSE_MOVED, 0), MouseEventKind::Move, None),
        (
            mouse(MOUSE_MOVED, FROM_LEFT_1ST_BUTTON_PRESSED),
            MouseEventKind::Drag,
            Some(MouseButton::Left),
        ),
        (
            mouse(0, RIGHTMOST_BUTTON_PRESSED),
            MouseEventKind::Down,
            Some(MouseButton::Right),
        ),
        (mouse(0, 0), MouseEventKind::Up, None),
        (
            mouse(MOUSE_WHEELED, 120 << 16),
            MouseEventKind::ScrollUp,
            None,
        ),
        (
            mouse(MOUSE_HWHEELED, u32::from((-120i16) as u16) << 16),
            MouseEventKind::ScrollLeft,
            None,
        ),
    ] {
        assert_eq!(
            tty.parse_input_record(&record),
            Some(TerminalEvent::Mouse {
                kind,
                button,
                column: 7,
                row: 3,
                pixel_x: None,
                pixel_y: None,
                modifiers: KeyModifiers {
                    shift: true,
                    ctrl: true,
                    alt: false,
                    meta: false
                },
            })
        );
    }
    let resize = INPUT_RECORD {
        EventType: WINDOW_BUFFER_SIZE_EVENT as u16,
        Event: INPUT_RECORD_0 {
            WindowBufferSizeEvent: WINDOW_BUFFER_SIZE_RECORD {
                dwSize: COORD { X: 120, Y: 40 },
            },
        },
    };
    assert_eq!(
        tty.parse_input_record(&resize),
        Some(TerminalEvent::Resize {
            width: 120,
            height: 40
        })
    );
    for (focused, expected) in [
        (1, TerminalEvent::FocusGained),
        (0, TerminalEvent::FocusLost),
    ] {
        let record = INPUT_RECORD {
            EventType: FOCUS_EVENT as u16,
            Event: INPUT_RECORD_0 {
                FocusEvent: FOCUS_EVENT_RECORD { bSetFocus: focused },
            },
        };
        assert_eq!(tty.parse_input_record(&record), Some(expected));
    }

    assert_ne!(unsafe { FlushConsoleInputBuffer(input) }, 0);
    inject(
        input,
        &[
            key(b'z' as u16, true, 1, 90),
            key(b'z' as u16, false, 1, 90),
        ],
    );
    let kinds: Vec<_> = tty
        .read_input_events(Some(Duration::from_millis(100)))
        .unwrap()
        .into_iter()
        .filter_map(|event| match event {
            TerminalEvent::Key {
                code: KeyCode::Char('z'),
                kind,
                ..
            } => Some(kind),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, [KeyEventKind::Press, KeyEventKind::Release]);
    assert_eq!(tty.write(b"native console probe").unwrap(), 20);
    let (width, height) = tty.size().unwrap();
    assert!(width > 0 && height > 0);
    tty.restore().unwrap();
    let mut restored = 0;
    assert_ne!(unsafe { GetConsoleMode(input, &mut restored) }, 0);
    assert_eq!(restored, original_mode);
    assert_ne!(
        unsafe { SetStdHandle(STD_INPUT_HANDLE, INVALID_HANDLE_VALUE) },
        0
    );
    assert!(
        WindowsTty::init().is_err(),
        "invalid input handle must fail"
    );
    assert_ne!(unsafe { SetStdHandle(STD_INPUT_HANDLE, input) }, 0);
}

#[test]
fn windows_console_io_and_event_conversion() {
    if let Some(report) = std::env::var_os("RTUI_WINDOWS_CONSOLE_REPORT") {
        let panic_report = report.clone();
        std::panic::set_hook(Box::new(move |info| {
            let _ = std::fs::write(&panic_report, info.to_string());
        }));
        check_private_console();
        std::fs::write(report, "RTUI_WINDOWS_ADAPTER_OK").unwrap();
        return;
    }
    struct OwnedTest(Child);
    impl Drop for OwnedTest {
        fn drop(&mut self) {
            if !matches!(self.0.try_wait(), Ok(Some(_))) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let report = directory.path().join("report");
    let mut child = OwnedTest(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "windows_console_io_and_event_conversion",
                "--nocapture",
            ])
            .env("RTUI_WINDOWS_CONSOLE_REPORT", &report)
            // Start unattached, then allocate a new console only in this child.
            .creation_flags(0x00000008) // DETACHED_PROCESS
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "private console test exceeded its deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let report = std::fs::read_to_string(report).unwrap_or_default();
    assert!(status.success(), "private console test failed: {report}");
    assert_eq!(report, "RTUI_WINDOWS_ADAPTER_OK");
    println!("{report}");
}
