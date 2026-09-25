//! Private process-tree ownership; no breakaway flags or global process lookup.

use super::{error, TerminalResult};
use std::{
    io,
    mem::{size_of, zeroed},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::null,
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
    JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
    TerminateJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

pub(super) struct Job(OwnedHandle);

impl Job {
    pub(super) fn new() -> TerminalResult<Self> {
        // SAFETY: unnamed, non-inheritable job; a successful handle is owned here.
        let handle = unsafe { CreateJobObjectW(null(), null()) };
        if handle.is_null() {
            return Err(error(format!(
                "CreateJobObjectW: {}",
                io::Error::last_os_error()
            )));
        }
        let job = Self(unsafe { OwnedHandle::from_raw_handle(handle) });
        // SAFETY: zero initializes the documented limit structure.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: valid owned job and a correctly sized input structure.
        if unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(error(format!(
                "SetInformationJobObject: {}",
                io::Error::last_os_error()
            )));
        }
        Ok(job)
    }

    pub(super) fn assign(&self, process: &OwnedHandle) -> TerminalResult<()> {
        // SAFETY: both handles stay owned. The caller has not resumed the child.
        if unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), process.as_raw_handle()) } == 0
        {
            return Err(error(format!(
                "AssignProcessToJobObject: {}",
                io::Error::last_os_error()
            )));
        }
        Ok(())
    }

    pub(super) fn terminate(&self) -> TerminalResult<()> {
        // SAFETY: only this session's process tree is associated with this job.
        if unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) } == 0 {
            return Err(error(format!(
                "TerminateJobObject: {}",
                io::Error::last_os_error()
            )));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            // SAFETY: valid output structure and live owned job handle.
            let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
            if unsafe {
                QueryInformationJobObject(
                    self.0.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    std::ptr::null_mut(),
                )
            } == 0
            {
                return Err(error(format!(
                    "QueryInformationJobObject: {}",
                    io::Error::last_os_error()
                )));
            }
            if info.ActiveProcesses == 0 {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(error(
                    "PTY process job did not become empty after termination",
                ));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}
