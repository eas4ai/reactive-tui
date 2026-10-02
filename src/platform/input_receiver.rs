//! Owned, bounded Unix input streams.

use std::io;
use std::net::Shutdown;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Receiver, RecvError, RecvTimeoutError, SyncSender, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::Duration;

const QUEUE_CAPACITY: usize = 64;
const READ_SIZE: usize = 4096;

/// A bounded input stream whose removal cancels and joins its reader.
///
/// Returned by the Unix async input methods. The queue retains at most 64 items;
/// raw items contain at most 4096 bytes. A full queue applies backpressure.
/// Dropping the final terminal owner also stops the worker. Buffered items remain
/// receivable after producer shutdown, followed by the standard disconnect error.
/// Input errors close the stream, as does terminal EOF.
///
/// This replaces the concrete standard-library receiver. Explicit receiver and
/// iterator type annotations must migrate; ordinary inferred receive calls retain
/// their form. A receiver cannot be extracted without its cleanup owner.
pub struct InputReceiver<T> {
    receiver: Receiver<T>,
    worker: Arc<InputWorker>,
}

impl<T> std::fmt::Debug for InputReceiver<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InputReceiver")
            .finish_non_exhaustive()
    }
}

impl<T> InputReceiver<T> {
    /// Wait for the next item or producer disconnection.
    pub fn recv(&self) -> Result<T, RecvError> {
        let result = self.receiver.recv();
        if result.is_ok() {
            self.worker.cancellation.queue_space();
        }
        result
    }

    /// Receive an available item without waiting.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let result = self.receiver.try_recv();
        if result.is_ok() {
            self.worker.cancellation.queue_space();
        }
        result
    }

    /// Wait up to the supplied duration for an item.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let result = self.receiver.recv_timeout(timeout);
        if result.is_ok() {
            self.worker.cancellation.queue_space();
        }
        result
    }

    /// Iterate while waiting for items, ending on disconnection.
    pub fn iter(&self) -> InputIter<'_, T> {
        InputIter {
            receiver: self,
            wait: true,
        }
    }

    /// Iterate over currently available items without waiting.
    pub fn try_iter(&self) -> InputIter<'_, T> {
        InputIter {
            receiver: self,
            wait: false,
        }
    }
}

impl<T> Drop for InputReceiver<T> {
    fn drop(&mut self) {
        self.worker.stop();
    }
}

/// Borrowed iteration over an owned input receiver.
pub struct InputIter<'a, T> {
    receiver: &'a InputReceiver<T>,
    wait: bool,
}

impl<T> Iterator for InputIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.wait {
            self.receiver.recv().ok()
        } else {
            self.receiver.try_recv().ok()
        }
    }
}

/// Owning iteration; dropping the iterator also shuts down its input worker.
pub struct InputIntoIter<T> {
    receiver: InputReceiver<T>,
}

impl<T> Iterator for InputIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.receiver.recv().ok()
    }
}

impl<T> IntoIterator for InputReceiver<T> {
    type Item = T;
    type IntoIter = InputIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        InputIntoIter { receiver: self }
    }
}

impl<'a, T> IntoIterator for &'a InputReceiver<T> {
    type Item = T;
    type IntoIter = InputIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub(super) struct Cancellation {
    stopped: Mutex<bool>,
    space: Condvar,
    wake_reader: UnixStream,
    wake_writer: UnixStream,
}

impl Cancellation {
    pub(super) fn new() -> io::Result<Self> {
        let (wake_reader, wake_writer) = UnixStream::pair()?;
        Ok(Self {
            stopped: Mutex::new(false),
            space: Condvar::new(),
            wake_reader,
            wake_writer,
        })
    }

    pub(super) fn cancel(&self) {
        let mut stopped = self.stopped.lock().unwrap_or_else(|e| e.into_inner());
        if !*stopped {
            *stopped = true;
            self.space.notify_all();
            // Closing the peer's write side wakes poll without writing or filling a pipe.
            let _ = self.wake_writer.shutdown(Shutdown::Both);
        }
    }

    fn queue_space(&self) {
        // Pair notification with the producer's predicate lock to avoid lost wakeups.
        let _stopped = self.stopped.lock().unwrap_or_else(|e| e.into_inner());
        self.space.notify_one();
    }

    fn send<T>(&self, sender: &SyncSender<T>, mut item: T) -> bool {
        let mut stopped = self.stopped.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if *stopped {
                return false;
            }
            match sender.try_send(item) {
                Ok(()) => return true,
                Err(mpsc::TrySendError::Disconnected(_)) => return false,
                Err(mpsc::TrySendError::Full(value)) => item = value,
            }
            stopped = self.space.wait(stopped).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub(super) fn read(&self, descriptor: &OwnedFd, buffer: &mut [u8]) -> io::Result<usize> {
        let mut descriptors = [
            libc::pollfd {
                fd: descriptor.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: self.wake_reader.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        loop {
            // Both descriptors remain owned throughout poll and read. The input
            // descriptor is independently opened with O_NONBLOCK, so another
            // reader or a mode change cannot turn readiness into a blocking read.
            let ready = unsafe { libc::poll(descriptors.as_mut_ptr(), 2, -1) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if descriptors[1].revents != 0 {
                return Ok(0);
            }
            let count = unsafe {
                libc::read(
                    descriptor.as_raw_fd(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if count >= 0 {
                return Ok(count as usize);
            }
            let error = io::Error::last_os_error();
            if !matches!(
                error.kind(),
                io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
            ) {
                return Err(error);
            }
        }
    }
}

pub(super) struct InputWorker {
    cancellation: Arc<Cancellation>,
    thread_id: ThreadId,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl InputWorker {
    pub(super) fn spawn<T, F>(descriptor: OwnedFd, mut parse: F) -> io::Result<InputReceiver<T>>
    where
        T: Send + 'static,
        F: FnMut(&[u8]) -> Vec<T> + Send + 'static,
    {
        let cancellation = Arc::new(Cancellation::new()?);
        let task_cancellation = Arc::clone(&cancellation);
        let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
        let thread = thread::Builder::new()
            .name("rtui-input".into())
            .spawn(move || {
                let mut buffer = [0; READ_SIZE];
                while let Ok(count) = task_cancellation.read(&descriptor, &mut buffer) {
                    if count == 0 {
                        break;
                    }
                    for item in parse(&buffer[..count]) {
                        if !task_cancellation.send(&sender, item) {
                            return;
                        }
                    }
                }
            })?;
        let worker = Arc::new(Self {
            cancellation,
            thread_id: thread.thread().id(),
            thread: Mutex::new(Some(thread)),
        });
        Ok(InputReceiver { receiver, worker })
    }

    pub(super) fn registration<T>(receiver: &InputReceiver<T>) -> std::sync::Weak<Self> {
        Arc::downgrade(&receiver.worker)
    }

    pub(super) fn cancel(&self) {
        self.cancellation.cancel();
    }

    pub(super) fn stop(&self) {
        self.cancel();
        // A panic hook can temporarily become the last terminal-state owner on
        // this worker. It must not join itself; the receiver still owns the join.
        if self.thread_id == thread::current().id() {
            return;
        }
        // Hold this lock across join: concurrent receiver/session teardown must
        // both wait for completion before releasing their resources.
        let mut thread = self.thread.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(thread) = thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for InputWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Instant;

    fn stream() -> (UnixStream, InputReceiver<u8>, Arc<InputWorker>) {
        let (reader, writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let receiver = InputWorker::spawn(reader.into(), |bytes| bytes.to_vec()).unwrap();
        let worker = Arc::clone(&receiver.worker);
        (writer, receiver, worker)
    }

    #[test]
    fn dropping_owned_iterator_joins_idle_reader() {
        let (_writer, receiver, worker) = stream();
        let started = Instant::now();
        drop(receiver.into_iter());
        // The behavior under test: dropping joins the idle reader at once.
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(worker.thread.lock().unwrap().is_none());
    }

    #[test]
    fn receive_and_iteration_preserve_order_and_disconnect() {
        fn require_debug<T: std::fmt::Debug>() {}
        struct NotDebug;
        require_debug::<InputReceiver<NotDebug>>();
        let (mut writer, receiver, worker) = stream();
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Empty));
        // The behavior under test: nothing arrives before anything is written.
        assert_eq!(
            receiver.recv_timeout(Duration::from_millis(10)),
            Err(RecvTimeoutError::Timeout)
        );
        writer.write_all(&[1, 2, 3, 4, 5]).unwrap();
        drop(writer);
        assert_eq!(receiver.recv_timeout(Duration::from_secs(30)).unwrap(), 1);
        assert_eq!(receiver.iter().next(), Some(2));
        assert_eq!((&receiver).into_iter().next(), Some(3));
        assert_eq!(receiver.into_iter().collect::<Vec<_>>(), vec![4, 5]);
        assert!(worker.thread.lock().unwrap().is_none());
    }

    #[test]
    fn buffered_items_remain_after_session_shutdown() {
        let (mut writer, receiver, worker) = stream();
        writer.write_all(&[1, 2, 3]).unwrap();
        drop(writer);
        // EOF joins the producer without cancelling before it finishes enqueueing.
        worker
            .thread
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .join()
            .unwrap();
        worker.stop();
        assert_eq!(receiver.try_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
        assert_eq!(receiver.recv(), Err(RecvError));
        assert_eq!(receiver.try_recv(), Err(TryRecvError::Disconnected));
        assert_eq!(
            receiver.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Disconnected)
        );
    }

    #[test]
    fn draining_a_full_queue_wakes_the_producer() {
        let (mut writer, receiver, _worker) = stream();
        let expected = (0..=255).collect::<Vec<u8>>();
        writer.write_all(&expected).unwrap();
        drop(writer);
        let actual = (0..256)
            .map(|_| receiver.recv_timeout(Duration::from_secs(30)).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(30)),
            Err(RecvTimeoutError::Disconnected)
        );
    }

    #[test]
    fn cancellation_releases_a_full_queue_producer() {
        let cancellation = Arc::new(Cancellation::new().unwrap());
        let (sender, _receiver) = mpsc::sync_channel(1);
        assert!(cancellation.send(&sender, 1));
        let task = Arc::clone(&cancellation);
        let (done_sender, done_receiver) = mpsc::channel();
        let producer = thread::spawn(move || {
            done_sender.send(task.send(&sender, 2)).unwrap();
        });
        // The behavior under test: the send blocks until it is cancelled.
        assert_eq!(
            done_receiver.recv_timeout(Duration::from_millis(20)),
            Err(RecvTimeoutError::Timeout)
        );
        cancellation.cancel();
        assert!(!done_receiver.recv_timeout(Duration::from_secs(30)).unwrap());
        producer.join().unwrap();
    }
}
