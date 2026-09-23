use reactive_tui::{
    core::geometry::{Point, Rect, Size},
    widgets::dialog::*,
};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

#[path = "api_dialog_lifecycle/app.rs"]
mod app;
mod common;
use common::app_input;

#[test]
fn close_preserves_the_actual_result_once_and_in_order() {
    let mut engine = DialogEngine::new();
    let id = engine.show_confirmation(Default::default());
    engine.close_dialog(id, DialogResult::Selected("archive".into()));
    engine.close_dialog(id, DialogResult::Cancelled);
    assert!(matches!(engine.take_event(), Some(DialogEvent::Opened(open)) if open == id));
    assert!(
        matches!(engine.take_event(), Some(DialogEvent::Closed(closed, DialogResult::Selected(value))) if closed == id && value == "archive")
    );
    assert!(engine.take_event().is_none());
    assert!(!engine.is_open(id));
}

#[test]
fn limits_reject_without_inventing_a_live_id_or_an_open_event() {
    for limit in [0, 2] {
        let mut engine = DialogEngine::with_config(DialogEngineConfig {
            max_dialogs: limit,
            ..Default::default()
        });
        for _ in 0..limit {
            assert_ne!(engine.show_input(Default::default()), DialogId::INVALID);
        }
        assert_eq!(
            engine.show_confirmation(Default::default()),
            DialogId::INVALID
        );
        assert_eq!(engine.last_error(), Some(DialogEngineError::LimitReached));
        assert_eq!(
            engine.try_show_toast(Default::default()),
            Err(DialogEngineError::LimitReached)
        );
        assert_eq!(engine.active_count(), limit);
        assert_eq!(std::iter::from_fn(|| engine.take_event()).count(), limit);
    }
}

#[test]
fn unread_events_are_bounded_and_every_accepted_dialog_can_still_close() {
    let mut engine = DialogEngine::with_config(DialogEngineConfig {
        max_dialogs: 2048,
        ..Default::default()
    });
    let mut ids = Vec::new();
    loop {
        match engine.try_show_progress(Default::default()) {
            Ok(id) => {
                ids.push(id);
                assert!(ids.len() <= 1024);
            }
            Err(DialogEngineError::EventQueueFull) => break,
            other => panic!("unexpected open result: {other:?}"),
        }
    }
    assert!(!ids.is_empty());
    for id in &ids {
        engine.close_dialog(*id, DialogResult::Confirmed(Some("done".into())));
    }
    let events: Vec<_> = std::iter::from_fn(|| engine.take_event()).collect();
    assert_eq!(events.len(), ids.len() * 2);
    for (event, id) in events[ids.len()..].iter().zip(ids) {
        assert!(
            matches!(event, DialogEvent::Closed(closed, DialogResult::Confirmed(Some(value))) if *closed == id && value == "done")
        );
    }
    assert!(engine.try_show_progress(Default::default()).is_ok());
}

#[test]
fn asynchronous_completion_waits_then_delivers_without_tokio() {
    let mut engine = DialogEngine::new();
    assert!(matches!(
        engine.events(),
        Err(DialogEngineError::AsyncDisabled)
    ));
    engine.enable_async();
    let id = engine.show_input(Default::default());
    let mut result = Box::pin(engine.completion(id).unwrap().result());
    assert!(matches!(
        engine.completion(id),
        Err(DialogEngineError::CompletionTaken)
    ));
    let mut context = Context::from_waker(futures_util::task::noop_waker_ref());
    assert!(result.as_mut().poll(&mut context).is_pending());
    engine.close_dialog(id, DialogResult::Confirmed(Some("界🌱".into())));
    assert!(
        matches!(result.as_mut().poll(&mut context), Poll::Ready(DialogResult::Confirmed(Some(value))) if value == "界🌱")
    );
}

#[test]
fn close_callbacks_can_reenter_and_see_the_closed_session_removed() {
    let mut engine = DialogEngine::new();
    let controller = engine.clone();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = calls.clone();
    let id = engine.show_confirmation(ConfirmationDialogOptions {
        on_close: Some(Arc::new(move |result| {
            assert_eq!(controller.active_count(), 0);
            let mut controller = controller.clone();
            let next = controller.try_show_input(Default::default()).unwrap();
            observed.lock().unwrap().push((next, result));
        })),
        ..Default::default()
    });
    engine.close_dialog(id, DialogResult::Cancelled);
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(engine.active_count(), 1);
    engine.close_all();
}

#[test]
fn dropping_an_unmounted_engine_cancels_waiters_and_closes_the_event_stream() {
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let mut events = engine.events().unwrap();
    let id = engine.show_progress(Default::default());
    let completion = engine.completion(id).unwrap();
    drop(engine);
    assert!(matches!(
        futures_lite::future::block_on(completion.result()),
        DialogResult::Cancelled
    ));
    assert!(
        matches!(futures_lite::future::block_on(events.next()), Some(DialogEvent::Opened(open)) if open == id)
    );
    assert!(
        matches!(futures_lite::future::block_on(events.next()), Some(DialogEvent::Closed(closed, DialogResult::Cancelled)) if closed == id)
    );
    assert!(futures_lite::future::block_on(events.next()).is_none());
}

#[test]
fn shutdown_reentry_cannot_leave_replacement_dialogs_running() {
    let mut engine = DialogEngine::new();
    let controller = engine.clone();
    let observed = Arc::new(Mutex::new(None));
    let result = observed.clone();
    engine.show_confirmation(ConfirmationDialogOptions {
        on_close: Some(Arc::new(move |_| {
            let mut controller = controller.clone();
            controller.close_all();
            *result.lock().unwrap() = Some(controller.try_show_input(Default::default()));
        })),
        ..Default::default()
    });
    engine.close_all();
    assert_eq!(
        *observed.lock().unwrap(),
        Some(Err(DialogEngineError::ShuttingDown))
    );
    assert_eq!(engine.active_count(), 0);
    assert!(engine.try_show_input(Default::default()).is_ok());
}

#[test]
fn updates_reject_wrong_types_unknown_ids_and_nonfinite_progress() {
    let mut engine = DialogEngine::new();
    let id = engine.show_progress(Default::default());
    assert_eq!(
        engine.update(id, DialogUpdate::Progress(f32::NAN)),
        Err(DialogEngineError::InvalidProgress)
    );
    assert_eq!(
        engine.update(id, DialogUpdate::InputValue("bad".into())),
        Err(DialogEngineError::WrongType)
    );
    assert_eq!(
        engine.update(DialogId::INVALID, DialogUpdate::Progress(0.5)),
        Err(DialogEngineError::NotFound)
    );
    assert!(engine.update(id, DialogUpdate::Progress(1.5)).is_ok());
}

#[test]
fn dialog_buffer_orders_by_z_and_replaces_existing_ids_without_duplicate_hits() {
    let mut buffer = DialogBuffer::new(Size::new(20, 10));
    let low = DialogId::from_u32(1);
    let high = DialogId::from_u32(2);
    let middle = DialogId::from_u32(3);
    let bounds = Rect::new(Point::new(1, 1), Size::new(5, 4));
    buffer.add_dialog(high, bounds, 30);
    buffer.add_dialog(low, bounds, 10);
    buffer.add_dialog(middle, bounds, 20);
    assert_eq!(buffer.get_z_order(), &[low, middle, high]);
    assert_eq!(buffer.hit_test(Point::new(2, 2)), Some(high));
    buffer.add_dialog(low, bounds, 40);
    assert_eq!(buffer.get_z_order(), &[middle, high, low]);
    buffer.remove_dialog(low);
    assert_eq!(buffer.hit_test(Point::new(2, 2)), Some(high));
    buffer.clear();
    assert!(buffer.get_z_order().is_empty());
    assert_eq!(buffer.hit_test(Point::new(2, 2)), None);
}

#[test]
fn async_event_wakers_can_reenter_without_an_engine_lock() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Observe {
        engine: DialogEngine,
        calls: AtomicUsize,
    }
    impl futures_util::task::ArcWake for Observe {
        fn wake_by_ref(this: &Arc<Self>) {
            assert_eq!(this.engine.active_count(), 1);
            this.calls.fetch_add(1, Ordering::SeqCst);
        }
    }
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let mut events = engine.events().unwrap();
    assert!(matches!(
        engine.events(),
        Err(DialogEngineError::EventsTaken)
    ));
    let observed = Arc::new(Observe {
        engine: engine.clone(),
        calls: AtomicUsize::new(0),
    });
    let waker = futures_util::task::waker(observed.clone());
    let mut context = Context::from_waker(&waker);
    let mut next = Box::pin(events.next());
    assert!(next.as_mut().poll(&mut context).is_pending());
    let id = engine.try_show_confirmation(Default::default()).unwrap();
    assert_eq!(observed.calls.load(Ordering::SeqCst), 1);
    assert!(
        matches!(next.as_mut().poll(&mut context), Poll::Ready(Some(DialogEvent::Opened(open))) if open == id)
    );
    drop(next);
    // The ready future released its waker; closing must not call it again.
    engine.close_dialog(id, DialogResult::Cancelled);
    assert_eq!(observed.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn invalid_animation_configuration_is_an_observable_rejection() {
    let mut engine = DialogEngine::with_config(DialogEngineConfig {
        default_theme: DialogTheme {
            animation: DialogAnimation::Custom("named".into()),
            ..Default::default()
        },
        ..Default::default()
    });
    assert_eq!(
        engine.try_show_input(Default::default()),
        Err(DialogEngineError::UnknownAnimation("named".into()))
    );
    assert_eq!(engine.active_count(), 0);
    assert!(engine.take_event().is_none());
    engine
        .register_animation("named", |progress| DialogAnimationFrame {
            opacity: progress,
            ..Default::default()
        })
        .unwrap();
    assert!(engine.try_show_input(Default::default()).is_ok());
    let mut invalid = DialogEngine::with_config(DialogEngineConfig {
        animation_duration: std::time::Duration::MAX,
        ..Default::default()
    });
    assert_eq!(
        invalid.try_show_input(Default::default()),
        Err(DialogEngineError::InvalidDuration)
    );
}

#[test]
fn one_panicking_close_callback_does_not_strand_other_sessions_or_waiters() {
    let mut engine = DialogEngine::new();
    engine.enable_async();
    let other = engine.show_progress(Default::default());
    let other_done = engine.completion(other).unwrap();
    let failing = engine.show_confirmation(ConfirmationDialogOptions {
        on_close: Some(Arc::new(|_| panic!("intentional callback failure"))),
        ..Default::default()
    });
    let failed_done = engine.completion(failing).unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| engine.close_all())).is_err());
    assert_eq!(engine.active_count(), 0);
    assert!(matches!(
        other_done.try_result(),
        Some(DialogResult::Cancelled)
    ));
    assert!(matches!(
        failed_done.try_result(),
        Some(DialogResult::Cancelled)
    ));
    let closed = std::iter::from_fn(|| engine.take_event())
        .filter(|event| matches!(event, DialogEvent::Closed(_, _)))
        .count();
    assert_eq!(closed, 2);
    assert!(engine.try_show_progress(Default::default()).is_ok());
}

#[test]
fn weak_callbacks_do_not_keep_abandoned_sessions_alive() {
    let mut engine = DialogEngine::new();
    let weak = engine.downgrade();
    let stored = weak.clone();
    let calls = Arc::new(Mutex::new(0));
    let observed = calls.clone();
    engine.show_confirmation(ConfirmationDialogOptions {
        on_close: Some(Arc::new(move |_| {
            assert!(stored.upgrade().is_none());
            *observed.lock().unwrap() += 1;
        })),
        ..Default::default()
    });
    drop(engine);
    assert!(weak.upgrade().is_none());
    assert_eq!(*calls.lock().unwrap(), 1);
}
