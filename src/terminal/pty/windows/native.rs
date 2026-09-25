//! Minimal owned ConPTY/process boundary. All handles stay on the supervisor.

use super::{error, TerminalConfig, TerminalError, TerminalResult};
use super::{job::Job, runtime::Runtime};
use std::{
    cmp::Ordering,
    ffi::{OsStr, OsString},
    fs::File,
    io,
    mem::{size_of, zeroed},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{ERROR_BROKEN_PIPE, ERROR_NO_DATA, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Globalization::CompareStringOrdinal,
    System::{
        Console::{COORD, HPCON},
        Pipes::CreatePipe,
        Threading::{
            CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
            InitializeProcThreadAttributeList, ResumeThread, TerminateProcess,
            UpdateProcThreadAttribute, WaitForSingleObject, CREATE_SUSPENDED,
            CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, PROCESS_INFORMATION,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTF_USESTDHANDLES, STARTUPINFOEXW,
        },
    },
};

pub(super) fn validate_size(width: u16, height: u16) -> TerminalResult<()> {
    if width == 0
        || height == 0
        || width > i16::MAX as u16
        || height > i16::MAX as u16
        || usize::from(width) * usize::from(height) > 262_144
    {
        Err(TerminalError::InvalidSize { width, height })
    } else {
        Ok(())
    }
}

pub(super) fn is_closed_pipe(failure: &io::Error) -> bool {
    matches!(failure.raw_os_error(), Some(code) if code == ERROR_BROKEN_PIPE as i32 || code == ERROR_NO_DATA as i32)
}

fn os_error(operation: &str) -> TerminalError {
    error(format!("{operation}: {}", io::Error::last_os_error()))
}

fn pipe() -> TerminalResult<(File, File)> {
    let (mut read, mut write) = (null_mut(), null_mut());
    // SAFETY: out pointers are valid. Null security attributes make both handles
    // non-inheritable; ownership transfers to File only on successful creation.
    if unsafe { CreatePipe(&mut read, &mut write, null(), 0) } == 0 {
        return Err(os_error("CreatePipe"));
    }
    Ok(unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) })
}

pub(super) struct Session {
    runtime: Runtime,
    console: Option<HPCON>,
    process: Option<OwnedHandle>,
    job: Job,
    pid: u32,
    input_read: Option<File>,
    output_write: Option<File>,
    input: Option<File>,
    output: Option<File>,
    exit: Option<i32>,
}

impl Session {
    pub(super) fn new(config: &TerminalConfig) -> TerminalResult<Self> {
        validate_size(config.size.0, config.size.1)?;
        let job = Job::new()?;
        let runtime = Runtime::load()?;
        let (input_read, input) = pipe()?;
        let (output, output_write) = pipe()?;
        let mut console = 0;
        // SAFETY: pipe handles stay alive through launch. Flags zero avoids the
        // cursor-inheritance handshake, which can deadlock an embedded host.
        let result = unsafe {
            (runtime.create)(
                COORD {
                    X: config.size.0 as i16,
                    Y: config.size.1 as i16,
                },
                input_read.as_raw_handle(),
                output_write.as_raw_handle(),
                0,
                &mut console,
            )
        };
        if result < 0 {
            return Err(error(format!("CreatePseudoConsole: HRESULT {result:#x}")));
        }
        Ok(Self {
            runtime,
            console: Some(console),
            process: None,
            job,
            pid: 0,
            input_read: Some(input_read),
            output_write: Some(output_write),
            input: Some(input),
            output: Some(output),
            exit: None,
        })
    }

    pub(super) fn take_output(&mut self) -> File {
        self.output.take().expect("ConPTY output taken once")
    }

    pub(super) fn take_input(&mut self) -> File {
        self.input.take().expect("ConPTY input taken once")
    }

    pub(super) fn launch(&mut self, config: &TerminalConfig) -> TerminalResult<()> {
        let environment = environment(config)?;
        let directory = config
            .working_directory
            .as_ref()
            .map(|s| wide(OsStr::new(s)))
            .transpose()?;
        let executable = executable(config, &environment)?;
        let executable = wide(executable.as_os_str())?;
        let mut command = Vec::with_capacity(executable.len() + 2);
        command.push(b'"' as u16);
        command.extend_from_slice(&executable[..executable.len() - 1]);
        command.extend([b'"' as u16, 0]);
        if command.len() > 32767 {
            return Err(error(
                "PTY executable path exceeds Windows command-line limit",
            ));
        }
        let environment = environment_block(&environment);
        let console = self.console.ok_or_else(|| error("ConPTY is closed"))?;
        let mut attributes = Attributes::new(console)?;
        // SAFETY: zero is a valid initial value for these Win32 output structs;
        // cb and the attribute-list pointer are assigned before the OS call.
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        // Explicit null standard handles make ConPTY create console streams.
        // Otherwise Windows can copy redirected parent handles even with
        // bInheritHandles=false (e.g. a daemon or CI runner).
        // https://github.com/microsoft/terminal/discussions/15814
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.lpAttributeList = attributes.pointer();
        let mut process: PROCESS_INFORMATION = unsafe { zeroed() };
        // SAFETY: all UTF-16 strings and attributes outlive the synchronous call.
        // The executable is explicit, so a path containing spaces is unambiguous.
        let success = unsafe {
            CreateProcessW(
                executable.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                0,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_SUSPENDED,
                environment.as_ptr().cast(),
                directory.as_ref().map_or(null(), |s| s.as_ptr()),
                &startup.StartupInfo,
                &mut process,
            )
        };
        if success == 0 {
            let failure = os_error("CreateProcessW for PTY child");
            return Err(failure);
        }
        // SAFETY: successful CreateProcessW returns owned process/thread handles.
        self.process = Some(unsafe { OwnedHandle::from_raw_handle(process.hProcess) });
        let thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
        let child = self.process.as_ref().expect("just created PTY process");
        let started = self.job.assign(child).and_then(|()| {
            // SAFETY: the primary thread remains owned and was created suspended.
            if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
                Err(os_error("ResumeThread for PTY child"))
            } else {
                Ok(())
            }
        });
        if let Err(failure) = started {
            // Assignment can fail before the job owns the suspended process.
            // SAFETY: this is our newly created child, which has not run user code.
            if unsafe { TerminateProcess(child.as_raw_handle(), 1) } == 0 {
                return Err(error(format!(
                    "{failure}; {}",
                    os_error("TerminateProcess after launch failure")
                )));
            }
            if unsafe { WaitForSingleObject(child.as_raw_handle(), 5000) } != WAIT_OBJECT_0 {
                return Err(error(format!(
                    "{failure}; suspended PTY child did not exit"
                )));
            }
            return Err(failure);
        }
        self.pid = process.dwProcessId;
        self.input_read.take();
        self.output_write.take();
        Ok(())
    }

    pub(super) fn child_id(&self) -> u32 {
        self.pid
    }

    pub(super) fn try_wait(&self) -> TerminalResult<Option<i32>> {
        if self.exit.is_some() {
            return Ok(self.exit);
        }
        let Some(process) = &self.process else {
            return Ok(None);
        };
        // SAFETY: process remains owned throughout both calls. Waiting first
        // distinguishes a real exit code 259 from the STILL_ACTIVE sentinel.
        match unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                if unsafe { GetExitCodeProcess(process.as_raw_handle(), &mut code) } == 0 {
                    Err(os_error("GetExitCodeProcess"))
                } else {
                    Ok(Some(code as i32))
                }
            }
            _ => Err(os_error("WaitForSingleObject")),
        }
    }

    pub(super) fn resize(&self, width: u16, height: u16) -> TerminalResult<()> {
        validate_size(width, height)?;
        let console = self.console.ok_or_else(|| error("ConPTY is closed"))?;
        // SAFETY: this supervisor exclusively owns the live console handle.
        let result = unsafe {
            (self.runtime.resize)(
                console,
                COORD {
                    X: width as i16,
                    Y: height as i16,
                },
            )
        };
        if result < 0 {
            Err(error(format!("ResizePseudoConsole: HRESULT {result:#x}")))
        } else {
            Ok(())
        }
    }

    /// The caller must keep draining output while this closes the console.
    pub(super) fn close(&mut self) -> TerminalResult<i32> {
        // Observe natural exit before terminating any remaining owned descendants.
        self.exit = self.try_wait()?;
        let job_result = self.job.terminate();
        self.input_read.take();
        self.output_write.take();
        self.input.take();
        // If the reader never started, disconnect its pipe before closing.
        self.output.take();
        if let Some(console) = self.console.take() {
            // SAFETY: exactly one close; output is drained or disconnected.
            unsafe { (self.runtime.close)(console) };
        }
        job_result?;
        let Some(process) = &self.process else {
            return Ok(0);
        };
        // SAFETY: the handle stays owned. ConPTY normally terminates all attached
        // clients; explicitly terminate our executable if it detached itself.
        if unsafe { WaitForSingleObject(process.as_raw_handle(), 5000) } == WAIT_TIMEOUT {
            if unsafe { TerminateProcess(process.as_raw_handle(), 1) } == 0
                && self.try_wait()?.is_none()
            {
                return Err(os_error("TerminateProcess for detached PTY child"));
            }
            if unsafe { WaitForSingleObject(process.as_raw_handle(), 5000) } != WAIT_OBJECT_0 {
                return Err(error("PTY child did not exit after termination"));
            }
        }
        self.exit = self.try_wait()?;
        self.exit
            .ok_or_else(|| error("PTY child exit was not observed"))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.console.is_some() {
            if let Err(failure) = self.close() {
                log::warn!("ConPTY cleanup: {failure}");
            }
        }
    }
}

struct Attributes {
    storage: Vec<usize>,
}

impl Attributes {
    fn new(console: HPCON) -> TerminalResult<Self> {
        let mut bytes = 0;
        // SAFETY: documented size query; no list exists yet.
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(os_error("Query process attribute size"));
        }
        let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        // SAFETY: pointer-aligned allocation has at least the requested size.
        if unsafe {
            InitializeProcThreadAttributeList(storage.as_mut_ptr().cast(), 1, 0, &mut bytes)
        } == 0
        {
            return Err(os_error("InitializeProcThreadAttributeList"));
        }
        let mut list = Self { storage };
        // SAFETY: ConPTY's documented attribute value is the handle itself,
        // unlike attributes that expect a pointer to another value.
        if unsafe {
            UpdateProcThreadAttribute(
                list.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                console as *const _,
                size_of::<HPCON>(),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err(os_error("UpdateProcThreadAttribute ConPTY"));
        }
        Ok(list)
    }
    fn pointer(&mut self) -> *mut std::ffi::c_void {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: the list was initialized successfully and still owns storage.
        unsafe { DeleteProcThreadAttributeList(self.pointer()) };
    }
}

fn wide(value: &OsStr) -> TerminalResult<Vec<u16>> {
    let mut value: Vec<u16> = value.encode_wide().collect();
    if value.contains(&0) || value.len() > 32766 {
        return Err(error(
            "PTY configuration contains NUL or an overlong Windows string",
        ));
    }
    value.push(0);
    Ok(value)
}

// Windows environment names compare case-insensitively using ordinal Unicode
// rules. Preserve OsString values, including inherited non-Unicode data.
fn compare_name(left: &OsStr, right: &OsStr) -> Ordering {
    let left: Vec<_> = left.encode_wide().collect();
    let right: Vec<_> = right.encode_wide().collect();
    // SAFETY: nonempty validated environment names fit i32; pointers stay live.
    match unsafe {
        CompareStringOrdinal(
            left.as_ptr(),
            left.len() as i32,
            right.as_ptr(),
            right.len() as i32,
            1,
        )
    } {
        1 => Ordering::Less,
        3 => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn environment(config: &TerminalConfig) -> TerminalResult<Vec<(OsString, OsString)>> {
    let mut entries: Vec<_> = std::env::vars_os().collect();
    for (name, value) in entries.iter() {
        wide(name)?;
        wide(value)?;
    }
    for (name, value) in [("TERM", "xterm-256color"), ("COLORTERM", "truecolor")]
        .into_iter()
        .chain(config.env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
    {
        if name.is_empty() || name.contains('=') {
            return Err(error("PTY environment name is empty or contains '='"));
        }
        wide(OsStr::new(name))?;
        wide(OsStr::new(value))?;
        entries.retain(|(existing, _)| compare_name(existing, OsStr::new(name)) != Ordering::Equal);
        entries.push((name.into(), value.into()));
    }
    entries.sort_by(|(a, _), (b, _)| compare_name(a, b));
    Ok(entries)
}

fn environment_block(entries: &[(OsString, OsString)]) -> Vec<u16> {
    let mut block = Vec::new();
    for (key, value) in entries {
        block.extend(key.encode_wide());
        block.push(b'=' as u16);
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    block
}

fn executable(
    config: &TerminalConfig,
    environment: &[(OsString, OsString)],
) -> TerminalResult<PathBuf> {
    let lookup = |name: &str| {
        environment
            .iter()
            .find(|(key, _)| compare_name(key, OsStr::new(name)) == Ordering::Equal)
            .map(|(_, value)| value)
    };
    let shell = config
        .shell
        .as_ref()
        .map(OsString::from)
        .or_else(|| lookup("COMSPEC").cloned())
        .unwrap_or_else(|| "cmd.exe".into());
    if shell.is_empty() || shell.encode_wide().any(|ch| ch == b'"' as u16 || ch == 0) {
        return Err(error(
            "PTY shell must be an executable path without quotes or NUL",
        ));
    }
    let directory = match &config.working_directory {
        Some(directory) => std::env::current_dir().map_err(error)?.join(directory),
        None => std::env::current_dir().map_err(error)?,
    };
    let mut name = PathBuf::from(shell);
    if name.extension().is_none() {
        name.set_extension("exe");
    }
    if name.is_absolute() {
        return Ok(name);
    }
    if name.components().count() > 1 {
        return Ok(directory.join(name));
    }
    let mut paths = vec![directory.clone()];
    if let Some(path) = lookup("PATH") {
        paths.extend(std::env::split_paths(path));
    }
    for path in paths {
        let candidate = directory.join(path).join(&name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(error(format!(
        "PTY executable {} was not found in the working directory or PATH",
        Path::new(&name).display()
    )))
}
