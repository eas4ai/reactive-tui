use reactive_tui::platform::{unix::UnixTty, DirectTty};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::{Duration, Instant};

fn threads() -> BTreeSet<u32> {
    std::fs::read_dir("/proc/self/task")
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect()
}

fn wait_for_workers(before: &BTreeSet<u32>, count: usize) -> BTreeSet<u32> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let new = threads()
            .difference(before)
            .copied()
            .collect::<BTreeSet<_>>();
        if new.len() == count {
            println!("Observed {count} owned reader threads: {new:?}");
            return new;
        }
        assert!(
            Instant::now() < deadline,
            "reader threads did not start: {new:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn require_stopped(workers: &BTreeSet<u32>) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let live = threads().intersection(workers).copied().collect::<Vec<_>>();
        if live.is_empty() {
            println!("PASS owned reader threads stopped");
            return;
        }
        assert!(
            Instant::now() < deadline,
            "owned reader threads survived receiver drop: {live:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn fill_input(input: &mut File) {
    let fd = input.as_raw_fd();
    let flags = unsafe { libc_fcntl(fd, 3, 0) };
    assert!(flags >= 0);
    // Linux F_SETFL / O_NONBLOCK, confined to the private PTY master.
    assert_eq!(unsafe { libc_fcntl(fd, 4, flags | 0x800) }, 0);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last_progress = Instant::now();
    let mut written = 0;
    loop {
        match input.write(&[b'x'; 4096]) {
            Ok(0) => panic!("private input closed during queue fill"),
            Ok(count) => {
                written += count;
                assert!(
                    written < 1024 * 1024,
                    "input failed to apply bounded backpressure"
                );
                last_progress = Instant::now();
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if last_progress.elapsed() >= Duration::from_millis(150) {
                    println!("Backpressure stopped input after {written} bytes");
                    return;
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => panic!("private input fill failed: {error}"),
        }
        assert!(
            Instant::now() < deadline,
            "input did not settle at its queue bound"
        );
    }
}

// This external consumer depends only on the public facade. The diagnostic is
// Linux-only and uses fcntl solely for its private PTY and descriptor flag checks.
unsafe extern "C" {
    #[link_name = "fcntl"]
    fn libc_fcntl(fd: i32, command: i32, value: i32) -> i32;
}

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    let case = &arguments[1];
    // The controller passes an owned descriptor for this child's private PTY.
    let mut input = unsafe { File::from_raw_fd(arguments[2].parse().unwrap()) };
    if case == "controller-hold" {
        println!("CONTROLLER_HOLD_READY");
        std::io::stdout().flush().unwrap();
        loop {
            thread::park();
        }
    }
    if case.starts_with("parsed-") {
        let tty = DirectTty::init().unwrap();
        let before = threads();
        // Exercise the approved concrete return type in an external consumer.
        let receiver: reactive_tui::platform::InputReceiver<reactive_tui::platform::TerminalEvent> =
            tty.start_async_events().unwrap();
        let workers = wait_for_workers(&before, 1);
        input.write_all(b"a").unwrap();
        assert!(matches!(
            receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
            reactive_tui::platform::TerminalEvent::Key {
                code: reactive_tui::platform::KeyCode::Char('a'),
                ..
            }
        ));
        if case.ends_with("full") {
            fill_input(&mut input);
        }
        if case.starts_with("parsed-session-") {
            drop(tty);
            require_stopped(&workers);
            let buffered = receiver.try_iter().count();
            assert_eq!(buffered, if case.ends_with("full") { 64 } else { 0 });
            assert_eq!(
                receiver.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Disconnected)
            );
        } else {
            drop(receiver);
            require_stopped(&workers);
            drop(tty);
        }
        return;
    }

    let tty = UnixTty::init().unwrap();
    let before = threads();
    let descriptor = tty.as_raw_fd();
    let flags = unsafe { libc_fcntl(descriptor, 3, 0) };
    let receiver: reactive_tui::platform::InputReceiver<Vec<u8>> =
        tty.spawn_input_thread().unwrap();
    let workers = wait_for_workers(&before, 1);
    input.write_all(b"ready").unwrap();
    let mut ready = Vec::new();
    while ready.len() < 5 {
        ready.extend(receiver.recv_timeout(Duration::from_secs(2)).unwrap());
    }
    assert_eq!(ready, b"ready", "initial input control");
    assert_eq!(
        unsafe { libc_fcntl(descriptor, 3, 0) },
        flags,
        "worker changed the synchronous terminal descriptor flags"
    );

    match case.as_str() {
        "raw-receiver-idle" => {
            drop(receiver);
            require_stopped(&workers);
            drop(tty);
        }
        "raw-receiver-active" => {
            drop(receiver);
            input.write_all(b"after-disconnect").unwrap();
            require_stopped(&workers);
            drop(tty);
        }
        "raw-receiver-full" => {
            fill_input(&mut input);
            drop(receiver);
            require_stopped(&workers);
        }
        "raw-session-full" | "raw-session-idle" => {
            if case.ends_with("full") {
                fill_input(&mut input);
            }
            drop(tty);
            require_stopped(&workers);
            let buffered = receiver.try_iter().collect::<Vec<_>>();
            assert_eq!(buffered.len(), if case.ends_with("full") { 64 } else { 0 });
            assert!(buffered.iter().all(|chunk| chunk.len() <= 4096));
            assert_eq!(
                receiver.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Disconnected)
            );
        }
        "clone-owner" => {
            let survivor = tty.clone();
            drop(tty);
            input.write_all(b"clone").unwrap();
            let mut received = Vec::new();
            while received.len() < 5 {
                received.extend(receiver.recv_timeout(Duration::from_secs(2)).unwrap());
            }
            assert_eq!(received, b"clone");
            drop(survivor);
            require_stopped(&workers);
        }
        "independent-streams" => {
            let second = tty.spawn_input_thread().unwrap();
            let all = wait_for_workers(&before, 2);
            let second_workers = all.difference(&workers).copied().collect();
            drop(receiver);
            require_stopped(&workers);
            input.write_all(b"second").unwrap();
            let mut received = Vec::new();
            while received.len() < 6 {
                received.extend(second.recv_timeout(Duration::from_secs(2)).unwrap());
            }
            assert_eq!(received, b"second");
            drop(tty);
            require_stopped(&second_workers);
        }
        "owned-iterator" => {
            drop(receiver.into_iter());
            require_stopped(&workers);
        }
        "concurrent-drop" => {
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
            let one = std::sync::Arc::clone(&barrier);
            let two = std::sync::Arc::clone(&barrier);
            let owner = thread::spawn(move || {
                one.wait();
                drop(tty);
            });
            let consumer = thread::spawn(move || {
                two.wait();
                drop(receiver);
            });
            barrier.wait();
            owner.join().unwrap();
            consumer.join().unwrap();
            require_stopped(&workers);
        }
        "reused-descriptor" => {
            drop(tty);
            let (replacement, mut writer) = UnixStream::pair().unwrap();
            assert_eq!(
                replacement.as_raw_fd(),
                descriptor,
                "private descriptor reuse setup"
            );
            writer.write_all(b"PRIVATE_REPLACEMENT_INPUT").unwrap();
            let observed = receiver.recv_timeout(Duration::from_millis(750));
            assert!(
                observed.is_err(),
                "reader consumed bytes from a reused descriptor: {observed:?}"
            );
            require_stopped(&workers);
            drop(replacement);
        }
        _ => panic!("unknown lifecycle case"),
    }
}
