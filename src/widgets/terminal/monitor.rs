//! The widget joins this monitor before releasing its terminal.
use super::*;
use crate::reactive::ThreadSafeSignal;
use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

pub(super) struct Monitor {
    closed: Arc<AtomicBool>,
    changed: ThreadSafeSignal<u64>,
    thread: Option<JoinHandle<()>>,
}
impl Monitor {
    pub(super) fn new(terminal: Arc<Mutex<Terminal>>) -> std::io::Result<Self> {
        let closed = Arc::new(AtomicBool::new(false));
        let changed = ThreadSafeSignal::new(0u64);
        let stop = closed.clone();
        let signal = changed.clone();
        let thread = thread::Builder::new()
            .name("terminal-widget-output".into())
            .spawn(move || {
                while !stop.load(Ordering::Acquire) {
                    let changed = match terminal.lock() {
                        Ok(mut terminal) => {
                            let before = terminal.revision();
                            let error = terminal.last_error().map(str::to_owned);
                            terminal.poll_events();
                            terminal.revision() != before
                                || terminal.last_error() != error.as_deref()
                        }
                        Err(_) => {
                            signal.update(|revision| *revision = revision.wrapping_add(1));
                            return;
                        }
                    };
                    if changed {
                        signal.update(|revision| *revision = revision.wrapping_add(1));
                    }
                    thread::park_timeout(Duration::from_millis(16));
                }
            })?;
        Ok(Self {
            closed,
            changed,
            thread: Some(thread),
        })
    }
    pub(super) fn observe(&self) {
        self.changed.get();
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            if thread.join().is_err() {
                log::error!("Terminal output monitor panicked");
            }
        }
    }
}
