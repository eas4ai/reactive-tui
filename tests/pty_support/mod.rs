//! Runs one test of this binary again on a new pseudo-terminal (Unix), for
//! tests whose subject needs a real terminal: the C API's renderer and
//! terminal functions enable raw mode and enter the alternate screen, which
//! `cargo test`'s pipes refuse. The copy runs with `env` set; the caller
//! reads it to tell the copy from the parent.

use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

const COLUMNS: u16 = 80;
const ROWS: u16 = 24;
/// Bounds a hung copy; a copy starts and finishes in well under a second.
const DEADLINE: Duration = Duration::from_secs(30);

/// What the copy did: its exit status and every byte it wrote.
pub struct Run {
    pub status: ExitStatus,
    pub output: Vec<u8>,
}

impl Run {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }
}

/// Runs `test` (its path as libtest prints it) from this binary on a new
/// pseudo-terminal of 80 by 24 cells with `env` set, waits for it to exit,
/// and returns its status and output.
pub fn run_test(test: &str, env: &[(&str, &str)]) -> Run {
    run_test_with_input(test, env, &[])
}

/// [`run_test`], with the terminal side typing `input`: each chunk is written
/// to the pseudo-terminal's master once its delay since the copy started has
/// passed, in order.
pub fn run_test_with_input(test: &str, env: &[(&str, &str)], input: &[(Duration, &[u8])]) -> Run {
    let (master, slave) = open_pty().expect("a pseudo-terminal");
    let mut command = Command::new(std::env::current_exe().expect("the test binary's path"));
    command
        .args([
            test,
            "--exact",
            "--nocapture",
            "--test-threads=1",
            "--color=never",
        ])
        .envs(env.iter().copied())
        .stdin(Stdio::from(
            slave.try_clone().expect("the terminal for stdin"),
        ))
        .stdout(Stdio::from(
            slave.try_clone().expect("the terminal for stdout"),
        ))
        .stderr(Stdio::from(slave));
    // SAFETY: the closure calls only async-signal-safe libc functions and
    // touches no parent state; std has put the terminal on descriptors 0, 1
    // and 2 before it runs.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command
        .spawn()
        .expect("a copy of the test binary on the terminal");
    // The parent's copy of the slave is gone with `command`, so the master
    // reads EIO once the copy has exited and closed its end.
    let mut master = master;
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    let started = Instant::now();
    let mut typed = 0;
    let status = loop {
        while typed < input.len() && started.elapsed() >= input[typed].0 {
            use std::io::Write;
            master
                .write_all(input[typed].1)
                .expect("typing on the pseudo-terminal");
            typed += 1;
        }
        loop {
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => output.extend_from_slice(&buffer[..read]),
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock
                        || error.raw_os_error() == Some(libc::EIO) =>
                {
                    break
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => panic!("reading the pseudo-terminal failed: {error}"),
            }
        }
        if let Some(status) = child.try_wait().expect("the copy's status") {
            break status;
        }
        if started.elapsed() > DEADLINE {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "{test} did not finish on the pseudo-terminal within {DEADLINE:?}; it wrote:\n{}",
                String::from_utf8_lossy(&output)
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    // Whatever the copy wrote last, before its end of the terminal closed.
    loop {
        match master.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => output.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => break,
        }
    }
    Run { status, output }
}

/// A new pseudo-terminal: the master, nonblocking, and the slave, both
/// closed on exec.
fn open_pty() -> io::Result<(File, File)> {
    let mut size = libc::winsize {
        ws_row: ROWS,
        ws_col: COLUMNS,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut master = -1;
    let mut slave = -1;
    // SAFETY: openpty fills both descriptors on success; the size lives for
    // the call.
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
    // SAFETY: openpty returned two owned descriptors nothing else holds.
    let master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    for file in [&master, &slave] {
        // SAFETY: fcntl on a live descriptor this function owns.
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
        if flags < 0
            || unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    // SAFETY: as above, on the master.
    let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
    if flags < 0
        || unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok((master, slave))
}
