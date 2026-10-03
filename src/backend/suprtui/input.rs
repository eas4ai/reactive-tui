//! Wait for Crossterm input and App notifications on the same thread.
use crate::app::AppWaker;
use crate::error::{ReactiveError, Result};
use crossterm::event::{Event, EventStream, MouseEvent, MouseEventKind};
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

/// Whether `event` is a motion report: a move, or a drag with a button held.
pub(super) fn is_motion(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved | MouseEventKind::Drag(_),
            ..
        })
    )
}

/// Whether `next` continues `event`'s run of motion: the same kind, button
/// and modifiers.
pub(super) fn continues_motion(event: &Event, next: &Event) -> bool {
    match (event, next) {
        (Event::Mouse(first), Event::Mouse(then)) => {
            is_motion(event) && first.kind == then.kind && first.modifiers == then.modifiers
        }
        _ => false,
    }
}

/// The next event when one is already waiting, without blocking.
pub(super) fn ready(stream: &mut EventStream) -> Result<Option<Event>> {
    // A real waker for this thread: when nothing waits, the stream keeps it
    // and wakes it on the next input, as the blocking poll relies on.
    let task = Waker::from(Arc::new(Unpark(thread::current())));
    let mut context = Context::from_waker(&task);
    match Pin::new(stream).poll_next(&mut context) {
        Poll::Ready(Some(event)) => event.map(Some).map_err(Into::into),
        Poll::Ready(None) => Err(ReactiveError::terminal("host input stream closed")),
        Poll::Pending => Ok(None),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseButton};

    fn mouse(kind: MouseEventKind, modifiers: KeyModifiers) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column: 1,
            row: 1,
            modifiers,
        })
    }

    /// INP-006: a run is moves, or drags of one button, with one set of modifiers.
    #[test]
    fn a_motion_run_keeps_its_kind_button_and_modifiers() {
        let none = KeyModifiers::NONE;
        let moved = mouse(MouseEventKind::Moved, none);
        let left = mouse(MouseEventKind::Drag(MouseButton::Left), none);
        let right = mouse(MouseEventKind::Drag(MouseButton::Right), none);
        let press = mouse(MouseEventKind::Down(MouseButton::Left), none);
        assert!(continues_motion(&moved, &moved));
        assert!(continues_motion(&left, &left));
        assert!(!continues_motion(&moved, &left));
        assert!(!continues_motion(&left, &right));
        assert!(!continues_motion(
            &moved,
            &mouse(MouseEventKind::Moved, KeyModifiers::SHIFT)
        ));
        assert!(!is_motion(&press));
        assert!(!continues_motion(&press, &press));
        assert!(!continues_motion(&moved, &press));
    }
}
