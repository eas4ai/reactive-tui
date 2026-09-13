//! A public Crossterm consumer; the parent queues one burst before releasing us.
use crossterm::event::{self, Event, KeyCode};
use std::{
    io::{self, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

struct RawMode;
impl Drop for RawMode {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().expect("restore terminal mode");
    }
}

fn main() {
    let release = PathBuf::from(std::env::args_os().nth(1).expect("release path"));
    crossterm::terminal::enable_raw_mode().expect("enable raw mode");
    let raw = RawMode;
    assert!(!event::poll(Duration::ZERO).expect("initialize input poll"));
    print!("\x1b]900;READY\x07");
    io::stdout().flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !release.exists() {
        assert!(
            Instant::now() < deadline,
            "parent did not release queued burst"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut count = 0;
    while count < 2048 && Instant::now() < deadline {
        if event::poll(Duration::from_millis(50)).expect("poll queued input") {
            match event::read().expect("read queued input") {
                Event::Key(key) if key.code == KeyCode::Char('a') => count += 1,
                event => panic!("unexpected burst event: {event:?}"),
            }
        }
    }
    println!("COUNT {count}");
    assert_eq!(
        count, 2048,
        "queued bytes stalled without another terminal write"
    );
    let start = Instant::now();
    for _ in 0..32 {
        assert!(
            !event::poll(Duration::ZERO).expect("poll exhausted input"),
            "input remained ready after consuming the whole burst"
        );
    }
    let elapsed = start.elapsed();
    println!("ZERO_POLL_US {}", elapsed.as_micros());
    assert!(
        elapsed < Duration::from_millis(250),
        "zero-timeout polling did not return promptly"
    );
    drop(raw);
    println!("ENTRY_POINT_CLEAN_EXIT");
}
