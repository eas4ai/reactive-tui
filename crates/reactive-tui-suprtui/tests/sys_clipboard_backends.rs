//! SYS-009: platform clipboard backends, headless.
//!
//! A scripted fake [`CommandRunner`] stands in for the helper
//! processes, so routing, command construction, and failure mapping
//! run with no display server.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use suprtui::clipboard::platform::{
    CommandOutput, CommandRunner, CommandSpec, Environment, HelperSet, OsKind, ProcessBackend,
    Route, RunnerError, SelectedBackends, route, select_backends,
};
use suprtui::clipboard::{
    DestroyOutcome, OperationKind, OperationStatus, ResultError, Service, StartOptions,
};

type CallLog = Rc<RefCell<Vec<(String, Vec<String>, Vec<u8>)>>>;

/// Scripted helper: each `run` logs the invocation, checks nothing
/// live, and answers canned.
struct FakeRunner {
    log: CallLog,
    script: VecDeque<FakeAnswer>,
}

enum FakeAnswer {
    Output(i32, Vec<u8>),
    Missing,
}

impl FakeRunner {
    fn scripted(log: &CallLog, answers: Vec<FakeAnswer>) -> Self {
        log.borrow_mut().clear();
        Self {
            log: Rc::clone(log),
            script: answers.into(),
        }
    }

    fn ok(log: &CallLog, stdout: &[u8]) -> Self {
        Self::scripted(log, vec![FakeAnswer::Output(0, stdout.to_vec())])
    }
}

impl CommandRunner for FakeRunner {
    fn run(&mut self, spec: &CommandSpec, stdin: &[u8]) -> Result<CommandOutput, RunnerError> {
        self.log
            .borrow_mut()
            .push((spec.program.clone(), spec.args.clone(), stdin.to_vec()));
        match self.script.pop_front().expect("fake runner out of script") {
            FakeAnswer::Output(status, stdout) => Ok(CommandOutput { status, stdout }),
            FakeAnswer::Missing => Err(RunnerError::SpawnFailed("not installed".to_string())),
        }
    }
}

fn log() -> CallLog {
    Rc::new(RefCell::new(Vec::new()))
}

fn env_of(pairs: &[(&str, &str)]) -> Environment {
    Environment::from_map(
        &pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<HashMap<_, _>>(),
    )
}

fn start<S: suprtui::clipboard::Backend>(
    service: &mut Service<S>,
    kind: OperationKind,
    payload: &[u8],
) -> u64 {
    service
        .start(kind, payload, StartOptions::default())
        .unwrap()
}

/// Availability fake that counts attempts, mirroring the
/// reference `FakeLoader`.
struct FakeAvailability {
    wayland_available: bool,
    x11_available: bool,
    wayland_attempts: std::cell::Cell<u32>,
    x11_attempts: std::cell::Cell<u32>,
}

impl FakeAvailability {
    fn check(&self, set: HelperSet) -> bool {
        match set {
            HelperSet::Wayland => {
                self.wayland_attempts.set(self.wayland_attempts.get() + 1);
                self.wayland_available
            }
            HelperSet::X11 => {
                self.x11_attempts.set(self.x11_attempts.get() + 1);
                self.x11_available
            }
        }
    }

    fn attempts(&self) -> (u32, u32) {
        (self.wayland_attempts.get(), self.x11_attempts.get())
    }
}

#[test]
fn req_009_select_backends_truth_table() {
    // Ported case-for-case from the reference routing test.
    struct SelectCase {
        env: Environment,
        available: (bool, bool),
        expected: SelectedBackends,
        attempts: (u32, u32),
    }
    let env = |is_wsl: bool, wayland: bool, x11: bool| Environment {
        is_wsl,
        has_wayland_display: wayland,
        has_x11_display: x11,
    };
    let selected = |wayland: bool, x11: bool, is_wsl: bool| SelectedBackends {
        wayland,
        x11,
        is_wsl,
    };
    let cases = [
        SelectCase {
            env: env(true, false, false),
            available: (true, true),
            expected: selected(false, false, true),
            attempts: (0, 0),
        },
        SelectCase {
            env: env(false, true, false),
            available: (true, false),
            expected: selected(true, false, false),
            attempts: (1, 0),
        },
        SelectCase {
            env: env(false, false, true),
            available: (false, true),
            expected: selected(false, true, false),
            attempts: (0, 1),
        },
        SelectCase {
            env: env(true, true, true),
            available: (true, true),
            expected: selected(true, true, true),
            attempts: (1, 1),
        },
        SelectCase {
            env: env(false, true, true),
            available: (false, true),
            expected: selected(false, true, false),
            attempts: (1, 1),
        },
        SelectCase {
            env: env(false, true, true),
            available: (true, false),
            expected: selected(true, false, false),
            attempts: (1, 1),
        },
    ];
    for case in cases {
        let fake = FakeAvailability {
            wayland_available: case.available.0,
            x11_available: case.available.1,
            wayland_attempts: std::cell::Cell::new(0),
            x11_attempts: std::cell::Cell::new(0),
        };
        let found = select_backends(case.env, &|set| fake.check(set));
        assert_eq!(found, case.expected);
        assert_eq!(fake.attempts(), case.attempts);
    }
}

#[test]
fn req_009_display_vars_must_be_nonempty() {
    let env = env_of(&[
        ("WAYLAND_DISPLAY", ""),
        ("DISPLAY", ":0"),
        ("WSL_INTEROP", ""),
    ]);
    assert!(env.is_wsl);
    assert!(!env.has_wayland_display);
    assert!(env.has_x11_display);
    let env = env_of(&[("WAYLAND_SOCKET", "7")]);
    assert!(env.has_wayland_display);
}

#[test]
fn req_009_route_matrix() {
    let none = Environment {
        is_wsl: false,
        has_wayland_display: false,
        has_x11_display: false,
    };
    let wayland = Environment {
        has_wayland_display: true,
        ..none
    };
    let x11 = Environment {
        has_x11_display: true,
        ..none
    };
    let wsl = Environment {
        is_wsl: true,
        ..none
    };
    let wsl_wayland = Environment {
        is_wsl: true,
        has_wayland_display: true,
        ..none
    };
    // Displays always win, even under WSL, as in the reference.
    assert_eq!(route(wayland, OsKind::Linux), Route::Wayland);
    assert_eq!(route(x11, OsKind::Linux), Route::X11);
    assert_eq!(route(wsl_wayland, OsKind::Linux), Route::Wayland);
    assert_eq!(route(wsl, OsKind::Linux), Route::WindowsClipboard);
    assert_eq!(route(none, OsKind::Linux), Route::Unsupported);
    assert_eq!(route(none, OsKind::MacOs), Route::MacOsClipboard);
    assert_eq!(route(none, OsKind::Windows), Route::WindowsClipboard);
    assert_eq!(route(none, OsKind::Other), Route::Unsupported);
}

#[test]
fn req_009_wayland_write_command() {
    let calls = log();
    let runner = FakeRunner::ok(&calls, b"");
    let mut service = Service::new(ProcessBackend::with_route(Route::Wayland, runner));
    let id = start(&mut service, OperationKind::Write, b"hello");
    assert_eq!(service.poll(id), OperationStatus::Written);
    assert_eq!(service.result(id), Ok(b"hello".as_slice()));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    assert_eq!(
        calls.borrow().as_slice(),
        [(String::from("wl-copy"), Vec::new(), b"hello".to_vec())].as_slice()
    );
}

#[test]
fn req_009_read_commands_per_route() {
    for (route, program, args) in [
        (Route::Wayland, "wl-paste", vec!["--no-newline"]),
        (Route::X11, "xclip", vec!["-selection", "clipboard", "-out"]),
        (Route::MacOsClipboard, "pbpaste", vec![]),
        (
            Route::WindowsClipboard,
            "powershell",
            vec!["-NoProfile", "-Command", "Get-Clipboard -Raw"],
        ),
    ] {
        let calls = log();
        let runner = FakeRunner::ok(&calls, b"bytes");
        let mut service = Service::new(ProcessBackend::with_route(route, runner));
        let id = start(&mut service, OperationKind::Read, b"");
        assert_eq!(service.poll(id), OperationStatus::Read);
        assert_eq!(service.result(id), Ok(b"bytes".as_slice()));
        let expected_args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        assert_eq!(
            calls.borrow().as_slice(),
            [(program.to_string(), expected_args, Vec::new())].as_slice()
        );
        assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    }
}

#[test]
fn req_009_write_commands_per_route() {
    for (route, program, args) in [
        (Route::Wayland, "wl-copy", Vec::new()),
        (Route::X11, "xclip", vec!["-selection", "clipboard", "-in"]),
        (Route::MacOsClipboard, "pbcopy", Vec::new()),
        (Route::WindowsClipboard, "clip", Vec::new()),
    ] {
        let calls = log();
        let runner = FakeRunner::ok(&calls, b"");
        let mut service = Service::new(ProcessBackend::with_route(route, runner));
        let id = start(&mut service, OperationKind::Write, b"data");
        assert_eq!(service.poll(id), OperationStatus::Written);
        let expected_args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        assert_eq!(
            calls.borrow().as_slice(),
            [(program.to_string(), expected_args, b"data".to_vec())].as_slice()
        );
        assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    }
}

#[test]
fn req_009_xclip_falls_back_to_xsel() {
    let calls = log();
    let runner = FakeRunner::scripted(
        &calls,
        vec![FakeAnswer::Missing, FakeAnswer::Output(0, Vec::new())],
    );
    let mut service = Service::new(ProcessBackend::with_route(Route::X11, runner));
    let id = start(&mut service, OperationKind::Write, b"data");
    assert_eq!(service.poll(id), OperationStatus::Written);
    let programs: Vec<String> = calls.borrow().iter().map(|call| call.0.clone()).collect();
    assert_eq!(programs, ["xclip", "xsel"]);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_009_missing_helper_is_unsupported() {
    // Every kind reports missing helpers as Unsupported; writes
    // and clears try xclip then xsel, so script two misses.
    for kind in [
        OperationKind::Read,
        OperationKind::Write,
        OperationKind::Clear,
    ] {
        let calls = log();
        let runner = FakeRunner::scripted(&calls, vec![FakeAnswer::Missing, FakeAnswer::Missing]);
        let mut service = Service::new(ProcessBackend::with_route(Route::X11, runner));
        let id = start(&mut service, kind, b"data");
        assert_eq!(
            service.poll(id),
            OperationStatus::Unsupported,
            "kind: {kind:?}"
        );
        assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    }
    let calls = log();
    let runner = FakeRunner::scripted(&calls, vec![]);
    let mut service = Service::new(ProcessBackend::with_route(Route::Unsupported, runner));
    let id = start(&mut service, OperationKind::Read, b"");
    assert_eq!(service.poll(id), OperationStatus::Unsupported);
}

#[test]
fn req_009_helper_failure_is_failed() {
    let calls = log();
    let runner = FakeRunner::scripted(&calls, vec![FakeAnswer::Output(1, b"nope".to_vec())]);
    let mut service = Service::new(ProcessBackend::with_route(Route::Wayland, runner));
    let id = start(&mut service, OperationKind::Read, b"");
    assert_eq!(service.poll(id), OperationStatus::Failed);
    assert!(matches!(
        service.result(id),
        Err(ResultError::Failed(OperationStatus::Failed))
    ));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_009_empty_read_is_empty() {
    let calls = log();
    let runner = FakeRunner::ok(&calls, b"");
    let mut service = Service::new(ProcessBackend::with_route(Route::Wayland, runner));
    let id = start(&mut service, OperationKind::Read, b"");
    assert_eq!(service.poll(id), OperationStatus::Empty);
    assert_eq!(service.result(id), Ok(b"".as_slice()));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_009_clear_sends_empty_payload() {
    let calls = log();
    let runner = FakeRunner::ok(&calls, b"");
    let mut service = Service::new(ProcessBackend::with_route(Route::MacOsClipboard, runner));
    let id = start(&mut service, OperationKind::Clear, b"ignored");
    assert_eq!(service.poll(id), OperationStatus::Cleared);
    assert_eq!(
        calls.borrow().as_slice(),
        [(String::from("pbcopy"), Vec::new(), Vec::new())].as_slice()
    );
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_009_write_over_cap_is_limited() {
    let calls = log();
    let runner = FakeRunner::ok(&calls, b"");
    let mut service = Service::new(ProcessBackend::with_route(Route::Wayland, runner));
    let id = service
        .start(
            OperationKind::Write,
            b"too long",
            StartOptions {
                max_bytes: 2,
                ..StartOptions::default()
            },
        )
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::LimitExceeded);
    assert!(calls.borrow().is_empty());
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_009_full_lifecycle_through_helpers() {
    let calls = log();
    let runner = FakeRunner::scripted(
        &calls,
        vec![
            FakeAnswer::Output(0, Vec::new()),
            FakeAnswer::Output(0, b"hello".to_vec()),
            FakeAnswer::Output(0, Vec::new()),
            FakeAnswer::Output(0, Vec::new()),
        ],
    );
    let mut service = Service::new(ProcessBackend::with_route(Route::Wayland, runner));
    let write = start(&mut service, OperationKind::Write, b"hello");
    assert_eq!(service.poll(write), OperationStatus::Written);
    assert_eq!(service.destroy(write), DestroyOutcome::Destroyed);
    let read = start(&mut service, OperationKind::Read, b"");
    assert_eq!(service.poll(read), OperationStatus::Read);
    assert_eq!(service.result(read), Ok(b"hello".as_slice()));
    assert_eq!(service.destroy(read), DestroyOutcome::Destroyed);
    let clear = start(&mut service, OperationKind::Clear, b"");
    assert_eq!(service.poll(clear), OperationStatus::Cleared);
    assert_eq!(service.destroy(clear), DestroyOutcome::Destroyed);
    let reread = start(&mut service, OperationKind::Read, b"");
    assert_eq!(service.poll(reread), OperationStatus::Empty);
    assert_eq!(service.destroy(reread), DestroyOutcome::Destroyed);
}
