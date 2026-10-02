use super::DialogEvent;
use futures_util::task::AtomicWaker;
use std::{
    collections::VecDeque,
    sync::Mutex,
    task::{Context, Poll},
};

#[derive(Default)]
pub(super) struct Mailbox {
    queue: Mutex<(VecDeque<DialogEvent>, bool)>,
    waker: AtomicWaker,
}
impl std::fmt::Debug for Mailbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogMailbox")
            .field("pending", &self.len())
            .finish()
    }
}
impl Mailbox {
    pub(super) fn len(&self) -> usize {
        self.queue.lock().unwrap().0.len()
    }
    pub(super) fn push(&self, event: DialogEvent) {
        let mut queue = self.queue.lock().unwrap();
        assert!(
            queue.0.len() < super::EVENT_CAPACITY,
            "dialog event reservation violated"
        );
        queue.0.push_back(event);
    }
    pub(super) fn take(&self) -> Option<DialogEvent> {
        self.queue.lock().unwrap().0.pop_front()
    }
    // Wakers are caller code: notify only after releasing all engine/queue locks.
    pub(super) fn wake(&self) {
        self.waker.wake();
    }
    pub(super) fn waiting(&self) -> Waiting<'_> {
        Waiting(self)
    }
    pub(super) fn close(&self) {
        self.queue.lock().unwrap().1 = true;
        self.wake();
    }
    pub(super) fn poll_next(&self, cx: &mut Context<'_>) -> Poll<Option<DialogEvent>> {
        self.waker.register(cx.waker());
        let result = {
            let mut queue = self.queue.lock().unwrap();
            if let Some(event) = queue.0.pop_front() {
                Poll::Ready(Some(event))
            } else if queue.1 {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        };
        if result.is_ready() {
            self.waker.take();
        }
        result
    }
}

pub(super) struct Waiting<'a>(&'a Mailbox);
impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        self.0.waker.take();
    }
}
