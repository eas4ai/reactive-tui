//! Unix PTY ownership. All descriptors and the direct child have one owner.
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};

#[derive(Debug)]
pub(crate) struct PtyChild {
    pub master: File,
    child: Child,
    stopped: Option<ExitStatus>,
}

impl PtyChild {
    pub fn spawn(mut command: Command, width: u16, height: u16) -> io::Result<Self> {
        let mut size = winsize(width, height);
        let mut master = -1;
        let mut slave = -1;
        // SAFETY: openpty initializes both descriptor outputs on success. The
        // window size is initialized and lives for the duration of the call.
        if unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(size),
            )
        } < 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful openpty returns two unique owned descriptors.
        let master = unsafe { File::from_raw_fd(master) };
        let slave = unsafe { File::from_raw_fd(slave) };
        cloexec(master.as_raw_fd())?;
        cloexec(slave.as_raw_fd())?;
        // Master IO is always nonblocking; no child can hold shutdown hostage.
        // SAFETY: fcntl operates on this live, owned descriptor.
        let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) }
                < 0
        {
            return Err(io::Error::last_os_error());
        }
        command.stdin(Stdio::from(slave.try_clone()?));
        command.stdout(Stdio::from(slave.try_clone()?));
        command.stderr(Stdio::from(slave));
        // SAFETY: the child closure calls only async-signal-safe libc operations.
        // std::process has installed the slave on descriptors 0, 1, and 2 before
        // this callback. No allocation, locks, or parent state is accessed.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                for signal in [
                    libc::SIGINT,
                    libc::SIGQUIT,
                    libc::SIGTERM,
                    libc::SIGHUP,
                    libc::SIGTSTP,
                    libc::SIGTTIN,
                    libc::SIGTTOU,
                    libc::SIGPIPE,
                ] {
                    if libc::signal(signal, libc::SIG_DFL) == libc::SIG_ERR {
                        return Err(io::Error::last_os_error());
                    }
                }
                let mut mask = std::mem::zeroed();
                if libc::sigemptyset(&mut mask) < 0
                    || libc::sigprocmask(libc::SIG_SETMASK, &mask, std::ptr::null_mut()) < 0
                {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        // Command::spawn reports exec and pre_exec errors through its error pipe.
        let child = command.spawn()?;
        Ok(Self {
            master,
            child,
            stopped: None,
        })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub fn resize(&self, width: u16, height: u16) -> io::Result<()> {
        let size = winsize(width, height);
        // SAFETY: the ioctl reads an initialized winsize from this live PTY.
        if unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ as _, &size) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn wait_ready(&self, writable: bool) -> io::Result<()> {
        let mut poll = libc::pollfd {
            fd: self.master.as_raw_fd(),
            events: libc::POLLIN | if writable { libc::POLLOUT } else { 0 },
            revents: 0,
        };
        // SAFETY: poll receives one initialized entry; the descriptor stays open.
        if unsafe { libc::poll(&mut poll, 1, 10) } < 0 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.stopped {
            return Ok(status);
        }
        // Kill a shell's foreground job only after verifying its session.
        let pid = self.child.id() as libc::pid_t;
        // SAFETY: these queries do not borrow Rust memory or transfer ownership.
        let foreground = unsafe { libc::tcgetpgrp(self.master.as_raw_fd()) };
        let mut foreground_error = None;
        if foreground > 0 && foreground != pid && unsafe { libc::getsid(foreground) } == pid {
            // SAFETY: negative ID addresses only the verified child-session group.
            if unsafe { libc::kill(-foreground, libc::SIGKILL) } < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    foreground_error = Some(error);
                }
            }
        }
        // Reap the direct child even if foreground cleanup failed. Command's
        // cached status makes natural exit and repeated shutdown idempotent.
        let status = match self.child.try_wait()? {
            Some(status) => status,
            None => {
                let killed = self.child.kill();
                match (killed, self.reap()) {
                    (_, Ok(status)) => status,
                    (Err(error), _) | (_, Err(error)) => return Err(error),
                }
            }
        };
        self.stopped = Some(status);
        match foreground_error {
            Some(error) => Err(error),
            None => Ok(status),
        }
    }

    /// Wait for the killed child while reading and discarding what it left
    /// unread on the master. A plain wait never returns on macOS for a child
    /// that printed faster than it was read: a process that exits with output
    /// still unread on its pseudo-terminal stays in exit until that output is
    /// read, and by the time `stop` runs nothing else reads it (TRM-001). The
    /// master is non-blocking, so each turn takes what is there and otherwise
    /// pauses a millisecond before asking for the exit status again.
    fn reap(&mut self) -> io::Result<ExitStatus> {
        let mut unread = [0u8; 4096];
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Ok(status);
            }
            match self.master.read(&mut unread) {
                Ok(0) => {}
                Ok(_) => continue,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) || error.raw_os_error() == Some(libc::EIO) => {}
                Err(error) => return Err(error),
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl Drop for PtyChild {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            log::warn!("Embedded child cleanup failed: {error}");
        }
    }
}

fn winsize(width: u16, height: u16) -> libc::winsize {
    libc::winsize {
        ws_row: height,
        ws_col: width,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}

fn cloexec(fd: RawFd) -> io::Result<()> {
    // SAFETY: caller owns the descriptor and keeps it open during this call.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::PtyChild;
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::time::{Duration, Instant};

    /// TRM-001 at the child itself, with no worker in between: a flooding
    /// child whose master was read once and then left alone is stopped
    /// within a second, on Linux and on macOS, where a process that exits
    /// with unread pseudo-terminal output stays in exit until it is read.
    #[test]
    fn trm_001_stop_reaps_a_flooding_child_whose_output_went_unread() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join("flood");
        std::fs::write(&path, "#!/bin/sh\nwhile :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut child = PtyChild::spawn(Command::new(&path), 80, 24).unwrap();
        let pid = child.id();
        // Setup: one read proves the flood runs; then nothing reads for a
        // while, so the child blocks in write with output unread.
        let mut bytes = [0; 256];
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match child.master.read(&mut bytes) {
                Ok(n) if n > 0 => break,
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("reading the flood: {error}"),
            }
            assert!(
                Instant::now() < deadline,
                "the flood printed nothing in 30 s"
            );
            child.wait_ready(false).unwrap();
        }
        std::thread::sleep(Duration::from_millis(300));
        let (done, finished) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let started = Instant::now();
            let status = child.stop();
            let _ = done.send((started.elapsed(), status.map(|s| s.success())));
        });
        let (took, status) = finished
            .recv_timeout(Duration::from_secs(10))
            .expect("TRM-001: stop did not return within 10 s");
        assert!(status.is_ok(), "stop failed: {status:?}");
        assert!(took < Duration::from_secs(1), "stop took {took:?}");
        // SAFETY: a liveness query of the process this test created.
        assert_eq!(
            unsafe { libc::kill(pid as i32, 0) },
            -1,
            "the child still exists"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}
