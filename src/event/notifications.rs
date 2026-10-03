//! Synchronous widget notifications owned by one App input dispatch.
use super::CustomEvent;
use std::cell::RefCell;

thread_local! {
    static FRAMES: RefCell<Vec<Vec<CustomEvent>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct Dispatch;

impl Dispatch {
    pub(crate) fn enter() -> Self {
        FRAMES.with_borrow_mut(|frames| frames.push(Vec::new()));
        Self
    }

    pub(crate) fn take(&self) -> Vec<CustomEvent> {
        FRAMES.with_borrow_mut(|frames| std::mem::take(frames.last_mut().unwrap()))
    }
}

impl Drop for Dispatch {
    fn drop(&mut self) {
        FRAMES.with_borrow_mut(|frames| {
            frames.pop();
        });
    }
}

pub(crate) fn emit(event: CustomEvent) {
    FRAMES.with_borrow_mut(|frames| {
        if let Some(frame) = frames.last_mut() {
            frame.push(event);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_dispatch_and_abandonment_do_not_leak_notifications() {
        let outer = Dispatch::enter();
        emit(CustomEvent::new("outer", vec![]));
        {
            let inner = Dispatch::enter();
            emit(CustomEvent::new("inner", vec![]));
            assert_eq!(inner.take()[0].name, "inner");
            assert!(inner.take().is_empty());
            emit(CustomEvent::new("abandoned", vec![]));
        }
        assert_eq!(outer.take()[0].name, "outer");
        assert!(outer.take().is_empty());
    }
}
