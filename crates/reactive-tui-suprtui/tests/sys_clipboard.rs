//! SYS-003: clipboard lifecycle over an injected backend.
//!
//! Every vector runs headless: the loopback backend answers from
//! memory and the deferred backend below scripts pending steps, so
//! no test needs a display server.

use suprtui::clipboard::{
    Backend, BackendOutcome, BackendRequest, BackendStep, CancelOutcome, CopyError, DestroyOutcome,
    LoopbackBackend, OperationKind, OperationStatus, ResultError, Service, StartError,
    StartOptions, UnsupportedBackend,
};

/// Backend that stays pending for `pending_polls` polls, then
/// completes with `outcome`. Proves completion is never reported
/// early.
struct DeferredBackend {
    pending_polls: u64,
    outcome: BackendOutcome,
}

impl Backend for DeferredBackend {
    fn step(&mut self, _request: &BackendRequest, polls: u64) -> BackendStep {
        if polls <= self.pending_polls {
            BackendStep::Pending
        } else {
            BackendStep::Complete(self.outcome.clone())
        }
    }
}

fn deferred(pending_polls: u64, outcome: BackendOutcome) -> DeferredBackend {
    DeferredBackend {
        pending_polls,
        outcome,
    }
}

#[test]
fn req_003_write_lifecycle() {
    let mut service = Service::new(LoopbackBackend::new());
    let id = service
        .start(OperationKind::Write, b"hello", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Written);
    assert_eq!(service.result(id), Ok(b"hello".as_slice()));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    assert_eq!(service.destroy(id), DestroyOutcome::InvalidHandle);
}

#[test]
fn req_003_read_lifecycle() {
    let mut service = Service::new(LoopbackBackend::with_seed(b"seeded"));
    let id = service
        .start(OperationKind::Read, b"", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Read);
    assert_eq!(service.result(id), Ok(b"seeded".as_slice()));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_clear_lifecycle() {
    let mut service = Service::new(LoopbackBackend::with_seed(b"seeded"));
    let id = service
        .start(OperationKind::Clear, b"", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Cleared);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
    let id = service
        .start(OperationKind::Read, b"", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Empty);
    assert_eq!(service.result(id), Ok(b"".as_slice()));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_no_early_completion() {
    let mut service = Service::new(deferred(2, BackendOutcome::Written));
    let id = service
        .start(OperationKind::Write, b"late", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Pending);
    assert_eq!(service.poll(id), OperationStatus::Pending);
    assert_eq!(service.poll(id), OperationStatus::Written);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_cancel_suppresses_result() {
    let mut service = Service::new(deferred(5, BackendOutcome::Written));
    let id = service
        .start(OperationKind::Write, b"cancelled", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Pending);
    assert_eq!(service.cancel(id), CancelOutcome::Requested);
    assert_eq!(service.poll(id), OperationStatus::Cancelled);
    assert_eq!(service.result(id), Err(ResultError::Cancelled));
    assert_eq!(service.cancel(id), CancelOutcome::AlreadyTerminal);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_destroy_pending_is_not_ready() {
    let mut service = Service::new(deferred(5, BackendOutcome::Written));
    let id = service
        .start(OperationKind::Write, b"held", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Pending);
    assert_eq!(service.destroy(id), DestroyOutcome::NotReady);
    assert_eq!(service.cancel(id), CancelOutcome::Requested);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_timeout() {
    let options = StartOptions {
        timeout: Some(std::time::Duration::ZERO),
        ..StartOptions::default()
    };
    let mut service = Service::new(deferred(5, BackendOutcome::Written));
    let id = service
        .start(OperationKind::Write, b"slow", options)
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::TimedOut);
    assert!(matches!(
        service.result(id),
        Err(ResultError::Failed(OperationStatus::TimedOut))
    ));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_limits_and_shutdown() {
    let mut service = Service::with_capacity(LoopbackBackend::new(), 1);
    let _held = service
        .start(OperationKind::Write, b"one", StartOptions::default())
        .unwrap();
    assert_eq!(
        service.start(OperationKind::Write, b"two", StartOptions::default()),
        Err(StartError::LimitExceeded)
    );
    assert_eq!(
        service.start(
            OperationKind::Write,
            b"",
            StartOptions {
                max_bytes: 0,
                ..StartOptions::default()
            }
        ),
        Err(StartError::LimitExceeded)
    );
    service.shutdown();
    assert_eq!(
        service.start(OperationKind::Read, b"", StartOptions::default()),
        Err(StartError::ShuttingDown)
    );
    assert_eq!(service.poll(999), OperationStatus::InvalidHandle);
    assert_eq!(service.cancel(999), CancelOutcome::InvalidHandle);
    assert_eq!(service.result(999), Err(ResultError::InvalidHandle));
    assert_eq!(service.destroy(999), DestroyOutcome::InvalidHandle);
}

#[test]
fn req_003_invalid_argument() {
    let mut service = Service::new(LoopbackBackend::new());
    assert_eq!(
        service.start(
            OperationKind::Write,
            b"x",
            StartOptions {
                max_bytes: 0,
                ..StartOptions::default()
            }
        ),
        Err(StartError::InvalidArgument)
    );
}

#[test]
fn req_003_limit_exceeded_on_write() {
    let options = StartOptions {
        max_bytes: 2,
        ..StartOptions::default()
    };
    let mut service = Service::new(LoopbackBackend::new());
    let id = service
        .start(OperationKind::Write, b"too long", options)
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::LimitExceeded);
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_unsupported_backend() {
    let mut service = Service::new(UnsupportedBackend);
    let id = service
        .start(OperationKind::Read, b"", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Unsupported);
    assert!(matches!(
        service.result(id),
        Err(ResultError::Failed(OperationStatus::Unsupported))
    ));
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}

#[test]
fn req_003_copy_into_buffer() {
    let mut service = Service::new(LoopbackBackend::with_seed(b"copy me"));
    let id = service
        .start(OperationKind::Read, b"", StartOptions::default())
        .unwrap();
    assert_eq!(service.poll(id), OperationStatus::Read);
    let mut small = [0u8; 3];
    assert_eq!(
        service.copy_into(id, &mut small),
        Err(CopyError::BufferTooSmall)
    );
    let mut exact = [0u8; 7];
    assert_eq!(service.copy_into(id, &mut exact), Ok(7));
    assert_eq!(&exact, b"copy me");
    assert_eq!(service.destroy(id), DestroyOutcome::Destroyed);
}
