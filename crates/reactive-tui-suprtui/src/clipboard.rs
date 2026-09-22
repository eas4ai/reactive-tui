//! Clipboard lifecycle (SYS-003).
//!
//! Ports the `host.zig` operation lifecycle — start, poll, result,
//! destroy — with cancellation, over an injected backend so the
//! whole state machine runs headless. The reference spawns one
//! thread per operation and fans out to X11, Wayland, Windows, and
//! macOS backends; this port keeps the observable lifecycle and
//! replaces the platform fan-out with a [`Backend`] trait. Tests
//! drive it with [`LoopbackBackend`] (and deferred script backends
//! in the suite), never a display server.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Statuses (mirror `host.zig`).
// ---------------------------------------------------------------------------

/// Operation state. `InvalidHandle` reports polling an id the
/// service does not hold, as in the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    Pending,
    Read,
    Empty,
    Written,
    Cleared,
    Unsupported,
    Cancelled,
    TimedOut,
    LimitExceeded,
    Failed,
    InvalidHandle,
}

/// Outcome of [`Service::cancel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelOutcome {
    Requested,
    AlreadyTerminal,
    InvalidHandle,
}

/// Outcome of [`Service::destroy`]. Destroying a pending operation
/// is refused (`NotReady`); cancel first, as in the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyOutcome {
    Destroyed,
    NotReady,
    InvalidHandle,
}

/// Why [`Service::start`] refused an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartError {
    ShuttingDown,
    LimitExceeded,
    InvalidArgument,
}

/// Why [`Service::result`] has no bytes to hand out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultError {
    InvalidHandle,
    NotReady,
    Cancelled,
    Failed(OperationStatus),
}

/// Why [`Service::copy_into`] refused the copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyError {
    BufferTooSmall,
    InvalidHandle,
    NotReady,
    Cancelled,
    Failed(OperationStatus),
}

/// What to read, write, or clear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    Read,
    Write,
    Clear,
}

// ---------------------------------------------------------------------------
// Backend seam.
// ---------------------------------------------------------------------------

/// What the backend was asked to do.
#[derive(Debug, Clone)]
pub struct BackendRequest {
    pub kind: OperationKind,
    pub payload: Vec<u8>,
    pub max_bytes: usize,
}

/// One backend step. `Pending` keeps the operation pending; the
/// service polls again later, so completion is never reported
/// early.
#[derive(Debug, Clone)]
pub enum BackendStep {
    Pending,
    Complete(BackendOutcome),
}

/// A finished backend step.
#[derive(Debug, Clone)]
pub enum BackendOutcome {
    Read(Vec<u8>),
    Empty,
    Written,
    Cleared,
    Unsupported,
    LimitExceeded,
    Failed,
}

/// Platform clipboard behind the lifecycle. Implementations must be
/// callable headless when tests need them; the suite uses
/// [`LoopbackBackend`].
pub trait Backend {
    fn step(&mut self, request: &BackendRequest, polls: u64) -> BackendStep;
}

/// In-memory backend: reads return the stored bytes (`Empty` when
/// none), writes replace them, clears drop them. Enforces
/// `max_bytes`, reporting `LimitExceeded` past the cap.
#[derive(Debug, Default)]
pub struct LoopbackBackend {
    store: Vec<u8>,
}

impl LoopbackBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(seed: &[u8]) -> Self {
        Self {
            store: seed.to_vec(),
        }
    }
}

impl Backend for LoopbackBackend {
    fn step(&mut self, request: &BackendRequest, _polls: u64) -> BackendStep {
        match request.kind {
            OperationKind::Read => {
                if self.store.is_empty() {
                    BackendStep::Complete(BackendOutcome::Empty)
                } else {
                    BackendStep::Complete(BackendOutcome::Read(self.store.clone()))
                }
            }
            OperationKind::Write => {
                if request.payload.len() > request.max_bytes {
                    BackendStep::Complete(BackendOutcome::LimitExceeded)
                } else {
                    self.store = request.payload.clone();
                    BackendStep::Complete(BackendOutcome::Written)
                }
            }
            OperationKind::Clear => {
                self.store.clear();
                BackendStep::Complete(BackendOutcome::Cleared)
            }
        }
    }
}

/// Backend that always reports the platform missing. Models running
/// with no display server.
#[derive(Debug, Default)]
pub struct UnsupportedBackend;

impl Backend for UnsupportedBackend {
    fn step(&mut self, _request: &BackendRequest, _polls: u64) -> BackendStep {
        BackendStep::Complete(BackendOutcome::Unsupported)
    }
}

// ---------------------------------------------------------------------------
// Service.
// ---------------------------------------------------------------------------

/// Per-start options.
#[derive(Debug, Clone, Copy)]
pub struct StartOptions {
    pub timeout: Option<Duration>,
    pub max_bytes: usize,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            max_bytes: 1024 * 1024,
        }
    }
}

struct Operation {
    kind: OperationKind,
    payload: Vec<u8>,
    max_bytes: usize,
    status: OperationStatus,
    result: Vec<u8>,
    cancel_requested: bool,
    deadline: Option<Instant>,
    polls: u64,
}

/// Caller-owned clipboard service. Operation ids are scoped to
/// their service; every pool, backend, and operation lives in the
/// caller's structs, never in process globals.
pub struct Service<B: Backend> {
    backend: B,
    operations: HashMap<u64, Operation>,
    next_id: u64,
    max_operations: usize,
    shutting_down: bool,
}

impl<B: Backend> Service<B> {
    pub fn new(backend: B) -> Self {
        Self::with_capacity(backend, 64)
    }

    pub fn with_capacity(backend: B, max_operations: usize) -> Self {
        Self {
            backend,
            operations: HashMap::new(),
            next_id: 1,
            max_operations,
            shutting_down: false,
        }
    }

    /// Refuse further starts. In-flight operations still run.
    pub fn shutdown(&mut self) {
        self.shutting_down = true;
    }

    /// Start a read, write, or clear. Returns the operation id.
    pub fn start(
        &mut self,
        kind: OperationKind,
        payload: &[u8],
        options: StartOptions,
    ) -> Result<u64, StartError> {
        if self.shutting_down {
            return Err(StartError::ShuttingDown);
        }
        if self.operations.len() >= self.max_operations {
            return Err(StartError::LimitExceeded);
        }
        if options.max_bytes == 0 {
            return Err(StartError::InvalidArgument);
        }
        let id = self.next_id;
        self.next_id += 1;
        let deadline = options.timeout.map(|t| Instant::now() + t);
        self.operations.insert(
            id,
            Operation {
                kind,
                payload: payload.to_vec(),
                max_bytes: options.max_bytes,
                status: OperationStatus::Pending,
                result: Vec::new(),
                cancel_requested: false,
                deadline,
                polls: 0,
            },
        );
        Ok(id)
    }

    /// Drive one operation. Never reports completion early: a
    /// backend that is still working keeps the status pending, and
    /// a cancelled operation reports cancelled without consulting
    /// the backend again.
    pub fn poll(&mut self, id: u64) -> OperationStatus {
        let operation = match self.operations.get_mut(&id) {
            Some(operation) => operation,
            None => return OperationStatus::InvalidHandle,
        };
        if operation.status != OperationStatus::Pending {
            return operation.status;
        }
        if operation.cancel_requested {
            operation.status = OperationStatus::Cancelled;
            return operation.status;
        }
        if operation
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            operation.status = OperationStatus::TimedOut;
            return operation.status;
        }
        let request = BackendRequest {
            kind: operation.kind,
            payload: operation.payload.clone(),
            max_bytes: operation.max_bytes,
        };
        operation.polls += 1;
        match self.backend.step(&request, operation.polls) {
            BackendStep::Pending => OperationStatus::Pending,
            BackendStep::Complete(outcome) => {
                let status = match outcome {
                    BackendOutcome::Read(bytes) => {
                        operation.result = bytes;
                        OperationStatus::Read
                    }
                    BackendOutcome::Empty => OperationStatus::Empty,
                    BackendOutcome::Written => {
                        operation.result = operation.payload.clone();
                        OperationStatus::Written
                    }
                    BackendOutcome::Cleared => OperationStatus::Cleared,
                    BackendOutcome::Unsupported => OperationStatus::Unsupported,
                    BackendOutcome::LimitExceeded => OperationStatus::LimitExceeded,
                    BackendOutcome::Failed => OperationStatus::Failed,
                };
                // A cancel that raced completion still wins: the
                // reference drops late results the same way.
                operation.status = if operation.cancel_requested {
                    OperationStatus::Cancelled
                } else {
                    status
                };
                operation.status
            }
        }
    }

    /// Cancel a pending operation. A cancelled operation never
    /// delivers a result.
    pub fn cancel(&mut self, id: u64) -> CancelOutcome {
        match self.operations.get_mut(&id) {
            None => CancelOutcome::InvalidHandle,
            Some(operation) if operation.status != OperationStatus::Pending => {
                CancelOutcome::AlreadyTerminal
            }
            Some(operation) => {
                operation.cancel_requested = true;
                operation.status = OperationStatus::Cancelled;
                CancelOutcome::Requested
            }
        }
    }

    /// Read a finished operation's bytes: read data, or the echoed
    /// payload of a completed write. `Empty` and `Cleared` yield an
    /// empty slice; every other non-data terminal state is an
    /// error, and a cancelled operation delivers nothing.
    pub fn result(&self, id: u64) -> Result<&[u8], ResultError> {
        match self.operations.get(&id) {
            None => Err(ResultError::InvalidHandle),
            Some(operation) => match operation.status {
                OperationStatus::Pending => Err(ResultError::NotReady),
                OperationStatus::Cancelled => Err(ResultError::Cancelled),
                OperationStatus::Read
                | OperationStatus::Empty
                | OperationStatus::Written
                | OperationStatus::Cleared => Ok(&operation.result),
                status => Err(ResultError::Failed(status)),
            },
        }
    }

    /// Copy a finished operation's bytes into a caller buffer, as
    /// the reference copy-to-buffer call does.
    pub fn copy_into(&self, id: u64, buffer: &mut [u8]) -> Result<usize, CopyError> {
        let bytes = match self.result(id) {
            Ok(bytes) => bytes,
            Err(ResultError::InvalidHandle) => return Err(CopyError::InvalidHandle),
            Err(ResultError::NotReady) => return Err(CopyError::NotReady),
            Err(ResultError::Cancelled) => return Err(CopyError::Cancelled),
            Err(ResultError::Failed(status)) => return Err(CopyError::Failed(status)),
        };
        if buffer.len() < bytes.len() {
            return Err(CopyError::BufferTooSmall);
        }
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Destroy a finished operation and free it. Pending
    /// operations are refused; cancel first.
    pub fn destroy(&mut self, id: u64) -> DestroyOutcome {
        match self.operations.get(&id) {
            None => DestroyOutcome::InvalidHandle,
            Some(operation) if operation.status == OperationStatus::Pending => {
                DestroyOutcome::NotReady
            }
            Some(_) => {
                self.operations.remove(&id);
                DestroyOutcome::Destroyed
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle unit tests (the runner also drives tests/sys_clipboard.rs).
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Platform backends (SYS-009).
// ---------------------------------------------------------------------------

/// Platform backend selection over helper processes, with routing
/// ported from the reference `linux.zig` environment detection.
pub mod platform {
    use super::{Backend, BackendOutcome, BackendRequest, BackendStep};
    use std::collections::HashMap;
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// What the process environment reports, mirroring the
    /// reference `Environment`: WSL markers are presence-based,
    /// display variables must be non-empty.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Environment {
        pub is_wsl: bool,
        pub has_wayland_display: bool,
        pub has_x11_display: bool,
    }

    impl Environment {
        pub fn from_map(vars: &HashMap<String, String>) -> Self {
            let present = |key: &str| vars.contains_key(key);
            let non_empty = |key: &str| vars.get(key).is_some_and(|value| !value.is_empty());
            Self {
                is_wsl: present("WSL_DISTRO_NAME") || present("WSL_INTEROP"),
                has_wayland_display: non_empty("WAYLAND_DISPLAY") || non_empty("WAYLAND_SOCKET"),
                has_x11_display: non_empty("DISPLAY"),
            }
        }

        /// Detect from the live process environment plus the kernel
        /// release, as the reference `detectProcess` does.
        pub fn detect_process() -> Self {
            let vars: HashMap<String, String> = std::env::vars().collect();
            let mut environment = Self::from_map(&vars);
            if !environment.is_wsl {
                let release =
                    std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
                environment.is_wsl = is_wsl_kernel_release(release.trim());
            }
            environment
        }
    }

    /// Case-insensitive `microsoft`/`wsl` match on the kernel
    /// release, exactly like the reference.
    pub fn is_wsl_kernel_release(release: &str) -> bool {
        let lower = release.to_lowercase();
        lower.contains("microsoft") || lower.contains("wsl")
    }

    /// Which helper family applies. Mirrors the reference
    /// `Libraries` selection: Wayland and X11 are attempted in
    /// preference order wherever their display is present, and the
    /// WSL flag passes through untouched.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SelectedBackends {
        pub wayland: bool,
        pub x11: bool,
        pub is_wsl: bool,
    }

    /// Helper family under test or construction.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HelperSet {
        Wayland,
        X11,
    }

    /// Availability probe: true when the helper family can run.
    /// Tests count attempts with a fake; production checks the
    /// helpers on `PATH`.
    pub fn select_backends(
        env: Environment,
        available: &dyn Fn(HelperSet) -> bool,
    ) -> SelectedBackends {
        SelectedBackends {
            wayland: env.has_wayland_display && available(HelperSet::Wayland),
            x11: env.has_x11_display && available(HelperSet::X11),
            is_wsl: env.is_wsl,
        }
    }

    /// Operating system underfoot.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OsKind {
        Linux,
        MacOs,
        Windows,
        Other,
    }

    impl OsKind {
        pub fn current() -> Self {
            match std::env::consts::OS {
                "linux" => Self::Linux,
                "macos" => Self::MacOs,
                "windows" => Self::Windows,
                _ => Self::Other,
            }
        }
    }

    /// Concrete helper route. WSL without any display falls back to
    /// the Windows interop helpers; displays always win, as in the
    /// reference.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Route {
        Wayland,
        X11,
        WindowsClipboard,
        MacOsClipboard,
        Unsupported,
    }

    /// Pick the route for an environment on an OS.
    pub fn route(env: Environment, os: OsKind) -> Route {
        match os {
            OsKind::MacOs => Route::MacOsClipboard,
            OsKind::Windows => Route::WindowsClipboard,
            OsKind::Linux => {
                if env.has_wayland_display {
                    Route::Wayland
                } else if env.has_x11_display {
                    Route::X11
                } else if env.is_wsl {
                    Route::WindowsClipboard
                } else {
                    Route::Unsupported
                }
            }
            OsKind::Other => Route::Unsupported,
        }
    }

    /// One helper invocation.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CommandSpec {
        pub program: String,
        pub args: Vec<String>,
    }

    impl CommandSpec {
        fn new(program: &str, args: &[&str]) -> Self {
            Self {
                program: program.to_string(),
                args: args.iter().map(|arg| arg.to_string()).collect(),
            }
        }
    }

    /// A finished helper invocation.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CommandOutput {
        pub status: i32,
        pub stdout: Vec<u8>,
    }

    /// Why a helper could not run at all (missing binary, refused
    /// spawn). A helper that runs and fails maps to `Failed`, never
    /// here.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RunnerError {
        SpawnFailed(String),
    }

    /// Runs helper commands. Production uses [`ProcessRunner`];
    /// tests script expectations with a fake, so routing and
    /// command construction run headless.
    pub trait CommandRunner {
        fn run(&mut self, spec: &CommandSpec, stdin: &[u8]) -> Result<CommandOutput, RunnerError>;
    }

    /// Real runner over `std::process`. The only piece that needs a
    /// live system; everything above it runs against fakes.
    #[derive(Debug, Default)]
    pub struct ProcessRunner;

    impl CommandRunner for ProcessRunner {
        fn run(&mut self, spec: &CommandSpec, stdin: &[u8]) -> Result<CommandOutput, RunnerError> {
            let mut child = Command::new(&spec.program)
                .args(&spec.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|err| RunnerError::SpawnFailed(err.to_string()))?;
            child
                .stdin
                .as_mut()
                .ok_or_else(|| RunnerError::SpawnFailed("no stdin pipe".to_string()))?
                .write_all(stdin)
                .map_err(|err| RunnerError::SpawnFailed(err.to_string()))?;
            let output = child
                .wait_with_output()
                .map_err(|err| RunnerError::SpawnFailed(err.to_string()))?;
            Ok(CommandOutput {
                status: output.status.code().unwrap_or(-1),
                stdout: output.stdout,
            })
        }
    }

    /// Clipboard backend over helper processes. Construct with
    /// [`ProcessBackend::detect`] on a live system or
    /// [`ProcessBackend::with_route`] in tests.
    pub struct ProcessBackend<R: CommandRunner> {
        route: Route,
        runner: R,
    }

    impl<R: CommandRunner> ProcessBackend<R> {
        pub fn with_route(route: Route, runner: R) -> Self {
            Self { route, runner }
        }

        /// Detect the route from the live process environment and
        /// helper availability on `PATH`.
        pub fn detect(runner: R) -> Self {
            let env = Environment::detect_process();
            let selected = select_backends(env, &|set| match set {
                HelperSet::Wayland => which_helper(&["wl-copy", "wl-paste"]).is_some(),
                HelperSet::X11 => which_helper(&["xclip", "xsel"]).is_some(),
            });
            let routed = route(env, OsKind::current());
            Self::with_route(resolve_route(routed, env, selected), runner)
        }

        fn read_command(&self) -> Option<CommandSpec> {
            match self.route {
                Route::Wayland => Some(CommandSpec::new("wl-paste", &["--no-newline"])),
                Route::X11 => Some(CommandSpec::new(
                    "xclip",
                    &["-selection", "clipboard", "-out"],
                )),
                Route::MacOsClipboard => Some(CommandSpec::new("pbpaste", &[])),
                Route::WindowsClipboard => Some(CommandSpec::new(
                    "powershell",
                    &["-NoProfile", "-Command", "Get-Clipboard -Raw"],
                )),
                Route::Unsupported => None,
            }
        }

        fn write_commands(&self) -> Vec<CommandSpec> {
            match self.route {
                Route::Wayland => vec![CommandSpec::new("wl-copy", &[])],
                // Prefer xclip, fall back to xsel when it is missing.
                Route::X11 => vec![
                    CommandSpec::new("xclip", &["-selection", "clipboard", "-in"]),
                    CommandSpec::new("xsel", &["--clipboard", "--input"]),
                ],
                Route::MacOsClipboard => vec![CommandSpec::new("pbcopy", &[])],
                Route::WindowsClipboard => vec![CommandSpec::new("clip", &[])],
                Route::Unsupported => vec![],
            }
        }

        fn run_first_success(
            &mut self,
            commands: &[CommandSpec],
            stdin: &[u8],
        ) -> Result<CommandOutput, RunnerError> {
            let mut missing: Option<RunnerError> = None;
            for command in commands {
                match self.runner.run(command, stdin) {
                    Ok(output) => return Ok(output),
                    Err(err) => {
                        missing = Some(err);
                    }
                }
            }
            Err(missing.unwrap_or(RunnerError::SpawnFailed("no helper for route".to_string())))
        }
    }

    /// Settle a preferred route against helper availability,
    /// falling back to the other display when it is present and
    /// available. Mirrors the reference selection outcome, where
    /// both families are attempted and the available one wins.
    fn resolve_route(routed: Route, env: Environment, selected: SelectedBackends) -> Route {
        match routed {
            Route::Wayland if selected.wayland => Route::Wayland,
            Route::X11 if selected.x11 => Route::X11,
            Route::Wayland | Route::X11 => {
                if env.has_x11_display && selected.x11 {
                    Route::X11
                } else if env.has_wayland_display && selected.wayland {
                    Route::Wayland
                } else {
                    Route::Unsupported
                }
            }
            other => other,
        }
    }

    fn which_helper(candidates: &[&str]) -> Option<String> {
        let path = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path) {
            for candidate in candidates {
                if dir.join(candidate).is_file() {
                    return Some(candidate.to_string());
                }
            }
        }
        None
    }

    impl<R: CommandRunner> Backend for ProcessBackend<R> {
        fn step(&mut self, request: &BackendRequest, _polls: u64) -> BackendStep {
            if request.payload.len() > request.max_bytes {
                return BackendStep::Complete(BackendOutcome::LimitExceeded);
            }
            match request.kind {
                super::OperationKind::Read => match self.read_command() {
                    None => BackendStep::Complete(BackendOutcome::Unsupported),
                    Some(command) => match self.runner.run(&command, &[]) {
                        Err(_) => BackendStep::Complete(BackendOutcome::Unsupported),
                        Ok(output) if output.status != 0 => {
                            BackendStep::Complete(BackendOutcome::Failed)
                        }
                        Ok(output) if output.stdout.is_empty() => {
                            BackendStep::Complete(BackendOutcome::Empty)
                        }
                        Ok(output) => BackendStep::Complete(BackendOutcome::Read(output.stdout)),
                    },
                },
                super::OperationKind::Write => {
                    match self.run_first_success(&self.write_commands(), &request.payload) {
                        Err(_) => BackendStep::Complete(BackendOutcome::Unsupported),
                        Ok(output) if output.status != 0 => {
                            BackendStep::Complete(BackendOutcome::Failed)
                        }
                        Ok(_) => BackendStep::Complete(BackendOutcome::Written),
                    }
                }
                super::OperationKind::Clear => {
                    match self.run_first_success(&self.write_commands(), &[]) {
                        Err(_) => BackendStep::Complete(BackendOutcome::Unsupported),
                        Ok(output) if output.status != 0 => {
                            BackendStep::Complete(BackendOutcome::Failed)
                        }
                        Ok(_) => BackendStep::Complete(BackendOutcome::Cleared),
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn wsl_kernel_release_matching() {
            assert!(is_wsl_kernel_release("4.4.0-19041-Microsoft"));
            assert!(is_wsl_kernel_release("5.15.153.1-microsoft-standard-WSL2"));
            assert!(!is_wsl_kernel_release("6.12.31-1-lts"));
        }

        #[test]
        fn resolve_falls_back_to_available_display() {
            let both = Environment {
                is_wsl: false,
                has_wayland_display: true,
                has_x11_display: true,
            };
            let x11_only = SelectedBackends {
                wayland: false,
                x11: true,
                is_wsl: false,
            };
            assert_eq!(resolve_route(Route::Wayland, both, x11_only), Route::X11);
            let neither = SelectedBackends {
                wayland: false,
                x11: false,
                is_wsl: false,
            };
            assert_eq!(
                resolve_route(Route::Wayland, both, neither),
                Route::Unsupported
            );
            let wayland_only = SelectedBackends {
                wayland: true,
                x11: false,
                is_wsl: false,
            };
            assert_eq!(
                resolve_route(Route::Wayland, both, wayland_only),
                Route::Wayland
            );
        }

        #[test]
        fn environment_from_map_rules() {
            let vars = HashMap::from([
                ("WAYLAND_DISPLAY".to_string(), String::new()),
                ("DISPLAY".to_string(), ":0".to_string()),
                ("WSL_INTEROP".to_string(), String::new()),
                ("WAYLAND_SOCKET".to_string(), "7".to_string()),
            ]);
            let env = Environment::from_map(&vars);
            assert!(env.is_wsl);
            assert!(env.has_wayland_display);
            assert!(env.has_x11_display);
            let vars = HashMap::from([("WAYLAND_DISPLAY".to_string(), String::new())]);
            let env = Environment::from_map(&vars);
            assert!(!env.has_wayland_display);
        }
    }
}

#[cfg(test)]
mod lifecycle {
    use super::*;

    #[test]
    fn terminal_poll_repeats_status() {
        let mut service = Service::new(LoopbackBackend::new());
        let id = service
            .start(OperationKind::Write, b"hi", StartOptions::default())
            .unwrap();
        assert_eq!(service.poll(id), OperationStatus::Written);
        assert_eq!(service.poll(id), OperationStatus::Written);
    }

    #[test]
    fn cancel_after_completion_is_already_terminal() {
        let mut service = Service::new(LoopbackBackend::new());
        let id = service
            .start(OperationKind::Clear, b"", StartOptions::default())
            .unwrap();
        assert_eq!(service.poll(id), OperationStatus::Cleared);
        assert_eq!(service.cancel(id), CancelOutcome::AlreadyTerminal);
    }
}
