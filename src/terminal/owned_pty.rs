//! Unix PTY ownership. All descriptors and the direct child have one owner.
use std::fs::File;
use std::io;
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
                match (killed, self.child.wait()) {
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
