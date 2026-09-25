use super::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Instant,
};

struct Released(Arc<AtomicBool>);
impl Drop for Released {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn stalled(wake: AppWaker) -> (Transport, Arc<AtomicBool>) {
    let (ready, started) = mpsc::sync_channel(1);
    let released = Arc::new(AtomicBool::new(false));
    let dropped = released.clone();
    let transport = Transport::spawn(wake, move |receiver, _sender| async move {
        let _receiver = receiver;
        let _released = Released(dropped);
        ready.send(()).unwrap();
        std::future::pending::<Result<()>>().await
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    (transport, released)
}

#[test]
fn queue_overload_is_bounded_wakes_app_and_cancels_the_worker() {
    let wake = AppWaker::new();
    let (mut transport, released) = stalled(wake.clone());
    for id in 0..QUEUE_CAPACITY {
        transport.sender.send(Message::RemoveAdapter { id });
    }
    assert_eq!(transport.sender.messages.len(), QUEUE_CAPACITY);
    assert!(transport.sender.check().is_ok());
    let start = Instant::now();
    transport
        .sender
        .send(Message::RemoveAdapter { id: QUEUE_CAPACITY });
    assert!(transport
        .sender
        .check()
        .unwrap_err()
        .contains("4096-message bound"));
    assert!(wake.is_pending());
    assert!(transport
        .close()
        .unwrap_err()
        .contains("4096-message bound"));
    assert!(released.load(Ordering::SeqCst));
    assert!(transport.worker.is_none());
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[test]
fn shutdown_cancels_a_pending_operation_and_joins_before_returning() {
    let (mut transport, released) = stalled(AppWaker::new());
    let start = Instant::now();
    transport.close().unwrap();
    transport.close().unwrap();
    assert!(released.load(Ordering::SeqCst));
    assert!(transport.worker.is_none());
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[test]
fn failure_is_reported_without_poisoning_other_or_later_apps() {
    let (mut independent, independent_released) = stalled(AppWaker::new());
    let wake = AppWaker::new();
    let mut failing = Transport::spawn(wake.clone(), |_receiver, _sender| async {
        Err("screen-reader fixture bus disconnected".into())
    })
    .unwrap();
    wake.wait(Some(Duration::from_secs(1)));
    assert!(wake.is_pending());
    assert!(failing
        .close()
        .unwrap_err()
        .contains("fixture bus disconnected"));
    assert!(independent.sender.check().is_ok());
    assert!(!independent_released.load(Ordering::SeqCst));
    let (mut later, later_released) = stalled(AppWaker::new());
    later.close().unwrap();
    independent.close().unwrap();
    assert!(independent_released.load(Ordering::SeqCst));
    assert!(later_released.load(Ordering::SeqCst));
}

#[test]
fn stalled_bus_operation_has_a_deadline() {
    let start = Instant::now();
    let error = block_on(deadline(
        "fixture operation",
        Duration::from_millis(20),
        std::future::pending::<zbus::Result<()>>(),
    ))
    .unwrap_err();
    assert!(error.contains("fixture operation exceeded its 20 ms deadline"));
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[test]
fn worker_panic_wakes_app_and_still_joins() {
    let wake = AppWaker::new();
    let mut transport = Transport::spawn(wake.clone(), |_receiver, _sender| async {
        panic!("disposable worker panic fixture");
        #[allow(unreachable_code)]
        Ok(())
    })
    .unwrap();
    wake.wait(Some(Duration::from_secs(1)));
    assert!(wake.is_pending());
    assert!(transport.close().unwrap_err().contains("worker panicked"));
    assert!(transport.worker.is_none());
}

#[test]
fn shutdown_during_an_operation_poll_does_not_report_queue_disconnect() {
    let (ready, started) = mpsc::sync_channel(1);
    let mut transport = Transport::spawn(AppWaker::new(), move |receiver, sender| async move {
        ready.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !sender.cancel.is_closed() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert!(sender.cancel.is_closed(), "shutdown did not start");
        // Hold the current operation poll across the shutdown signal. The outer
        // cancellation select cannot run until this poll returns.
        std::thread::sleep(Duration::from_millis(20));
        if receiver.is_closed() {
            return Err("screen-reader outgoing queue closed".into());
        }
        std::future::pending::<Result<()>>().await
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    transport.close().unwrap();
    assert!(transport.worker.is_none());
}
