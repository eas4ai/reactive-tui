//! Wait for Crossterm input and App notifications on the same thread.
use crate::app::AppWaker;
use crate::error::{ReactiveError, Result};
use crossterm::event::{Event, EventStream};
use futures_core::Stream;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::{Duration, Instant};

struct Unpark(thread::Thread);
impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
struct Registration<'a>(&'a AppWaker);
impl Drop for Registration<'_> {
    fn drop(&mut self) {
        self.0.unregister();
    }
}

pub(super) fn poll(
    stream: &mut EventStream,
    timeout: Option<Duration>,
    wake: &AppWaker,
) -> Result<Option<Event>> {
    let task = Waker::from(Arc::new(Unpark(thread::current())));
    wake.register(&task);
    let _registration = Registration(wake);
    let mut context = Context::from_waker(&task);
    let deadline = timeout.and_then(|duration| Instant::now().checked_add(duration));
    loop {
        // Check input even during notification bursts so keys cannot starve.
        match Pin::new(&mut *stream).poll_next(&mut context) {
            Poll::Ready(Some(event)) => return event.map(Some).map_err(Into::into),
            Poll::Ready(None) => return Err(ReactiveError::terminal("host input stream closed")),
            Poll::Pending => {}
        }
        if wake.is_pending() || wake.is_closed() {
            return Ok(None);
        }
        match deadline {
            Some(end) => {
                let remaining = end.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Ok(None);
                }
                thread::park_timeout(remaining);
            }
            None => thread::park(),
        }
        // An unpark token sent between the checks and park is retained by std.
        // Registration is refreshed after each notification takes its waker.
        wake.register(&task);
    }
}
