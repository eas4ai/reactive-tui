//! One App owns this queue, cancellation signal and worker lifetime.

use super::{adapter::Message, context, executor::Executor, util::block_on};
use crate::{accessibility::platform::translation::AppContext, app::AppWaker};
use async_channel::{Receiver, Sender, TrySendError};
use futures_util::{pin_mut, select_biased, FutureExt};
use std::{
    future::Future,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

pub(super) type Result<T> = std::result::Result<T, String>;
const QUEUE_CAPACITY: usize = 4096;
const OPERATION_TIMEOUT: Duration = Duration::from_secs(3);

struct Health {
    failure: Mutex<Option<String>>,
    wake: AppWaker,
}

#[derive(Clone)]
pub(super) struct MessageSender {
    messages: Sender<Message>,
    cancel: Sender<()>,
    health: Arc<Health>,
}

impl MessageSender {
    pub(super) fn send(&self, message: Message) {
        if self.cancel.is_closed() {
            return;
        }
        match self.messages.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => self.fail(format!(
                "screen-reader outgoing queue exceeded its {QUEUE_CAPACITY}-message bound"
            )),
            Err(TrySendError::Closed(_)) => {
                self.fail("screen-reader transport closed unexpectedly".into());
            }
        }
    }

    fn fail(&self, message: String) {
        let mut failure = self
            .health
            .failure
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if failure.is_none() {
            *failure = Some(message);
        }
        drop(failure);
        self.messages.close();
        self.cancel.close();
        self.health.wake.request_redraw();
    }

    pub(super) fn check(&self) -> Result<()> {
        match self
            .health
            .failure
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

pub(super) struct Transport {
    pub(super) sender: MessageSender,
    worker: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transport")
            .field("queued", &self.sender.messages.len())
            .finish_non_exhaustive()
    }
}

impl Transport {
    pub(super) fn new(context: Arc<RwLock<AppContext>>, wake: AppWaker) -> std::io::Result<Self> {
        Self::spawn(wake, move |rx, sender| async move {
            let executor = Executor::new();
            executor
                .run(context::run(&executor, context, rx, sender))
                .await
        })
    }

    fn spawn<F, Fut>(wake: AppWaker, operation: F) -> std::io::Result<Self>
    where
        F: FnOnce(Receiver<Message>, MessageSender) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>>,
    {
        let (messages, receiver) = async_channel::bounded(QUEUE_CAPACITY);
        let (cancel, cancelled) = async_channel::bounded(1);
        let sender = MessageSender {
            messages,
            cancel,
            health: Arc::new(Health {
                failure: Mutex::new(None),
                wake,
            }),
        };
        let worker_sender = sender.clone();
        let worker = thread::Builder::new().name("rtui-accessibility".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                block_on(async {
                    let operation = operation(receiver, worker_sender.clone()).fuse();
                    let cancelled = cancelled.recv().fuse();
                    pin_mut!(operation, cancelled);
                    select_biased! {
                        _ = cancelled => Ok(()),
                        result = operation => result.and(Err("screen-reader worker stopped unexpectedly".into())),
                    }
                })
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => worker_sender.fail(error),
                Err(_) => worker_sender.fail("screen-reader worker panicked".into()),
            }
        })?;
        Ok(Self {
            sender,
            worker: Some(worker),
        })
    }

    pub(super) fn close(&mut self) -> Result<()> {
        self.sender.cancel.close();
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                self.sender
                    .fail("screen-reader worker panicked during shutdown".into());
            }
        }
        // The operation may still be inside a poll when cancellation starts.
        // Keep its queue open until it is dropped, so normal shutdown cannot
        // race with a spurious disconnected-queue error from that poll.
        self.sender.messages.close();
        self.sender.check()
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        if let Err(error) = self.close() {
            log::debug!("screen-reader transport closed after failure: {error}");
        }
    }
}

pub(super) async fn timed<T>(
    name: &str,
    operation: impl Future<Output = zbus::Result<T>>,
) -> Result<T> {
    deadline(name, OPERATION_TIMEOUT, operation).await
}

async fn deadline<T>(
    name: &str,
    timeout: Duration,
    operation: impl Future<Output = zbus::Result<T>>,
) -> Result<T> {
    let operation = operation.fuse();
    let timer = async_io::Timer::after(timeout).fuse();
    pin_mut!(operation, timer);
    select_biased! {
        result = operation => result.map_err(|error| format!("screen-reader {name}: {error}")),
        _ = timer => Err(format!("screen-reader {name} exceeded its {} ms deadline", timeout.as_millis())),
    }
}

#[cfg(test)]
mod tests;
