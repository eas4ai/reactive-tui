use reactive_tui::platform::{unix::UnixTty, DirectTty};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
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

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    let case = &arguments[1];
    // The controller passes an owned descriptor for this child's private PTY.
    let mut input = unsafe { File::from_raw_fd(arguments[2].parse().unwrap()) };
    let before = threads();
    if case == "parsed-receiver-idle" {
        let tty = DirectTty::init().unwrap();
        // Keep the existing concrete return type explicit in this consumer.
        let receiver: std::sync::mpsc::Receiver<reactive_tui::platform::TerminalEvent> =
            tty.start_async_events().unwrap();
        let workers = wait_for_workers(&before, 2);
        drop(receiver);
        require_stopped(&workers);
        drop(tty);
        return;
    }

    let tty = UnixTty::init().unwrap();
    let descriptor = tty.as_raw_fd();
    let receiver: std::sync::mpsc::Receiver<Vec<u8>> = tty.spawn_input_thread().unwrap();
    let workers = wait_for_workers(&before, 1);
    input.write_all(b"ready").unwrap();
    let mut ready = Vec::new();
    while ready.len() < 5 {
        ready.extend(receiver.recv_timeout(Duration::from_secs(2)).unwrap());
    }
    assert_eq!(ready, b"ready", "initial input control");

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
