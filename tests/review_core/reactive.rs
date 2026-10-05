//! Part of tests/review_core.rs.

use reactive_tui::reactive::{signal::Memo, RuntimeContext, Signal};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn sig_003_a_memo_recomputes_after_its_source_changes() {
    let source = Signal::new(1);
    let input = source.clone();
    let doubled = Memo::new(move || input.get() * 2);
    assert_eq!(
        doubled.get(),
        2,
        "SIG-003: the initial doubled value was not 2"
    );

    source.set(5);

    assert_eq!(
        doubled.get(),
        10,
        "SIG-003: after setting the source to 5, the memo still returned a stale doubled value"
    );
}

/// A memo that reads another memo reads, through it, the signals that memo
/// read: a change of one of them reaches the outer memo (review finding 2).
#[test]
fn sig_003_a_memo_reading_a_memo_recomputes_after_the_source_changes() {
    let source = Signal::new(1);
    let input = source.clone();
    let doubled = Memo::new(move || input.get() * 2);
    let inner = doubled.clone();
    let plus_one = Memo::new(move || inner.get() + 1);
    assert_eq!(
        plus_one.get(),
        3,
        "SIG-003: the initial chained value was not 3"
    );

    source.set(5);

    assert_eq!(
        plus_one.get(),
        11,
        "SIG-003: after setting the source to 5, the memo reading the doubled memo still returned a stale value"
    );
    assert_eq!(
        doubled.get(),
        10,
        "SIG-003: the doubled memo itself returned a stale value"
    );
}

/// A compute function may read a signal inside that signal's `with` (review
/// finding 7): subscribing the memo must not need the value borrowed.
#[test]
fn sig_003_a_memo_may_read_a_signal_inside_that_signals_with() {
    let source = Signal::new(1);
    let input = source.clone();
    let twice = Memo::new(move || input.with(|value| *value + input.get()));
    assert_eq!(
        twice.get(),
        2,
        "SIG-003: a nested read inside with gave the wrong initial value"
    );

    source.set(3);

    assert_eq!(
        twice.get(),
        6,
        "SIG-003: after setting the source to 3, the memo with a nested read returned a stale value"
    );
}

#[test]
fn sig_004_an_effect_reruns_after_cleanup_until_unregistered() {
    let context = RuntimeContext::new();
    let source = context.create_signal(0);
    let input = source.clone();
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = events.clone();
    let effect = context.create_effect(move || {
        let value = input.get();
        output.borrow_mut().push(format!("run {value}"));
        let cleanup_output = output.clone();
        Some(Box::new(move || {
            cleanup_output.borrow_mut().push(format!("cleanup {value}"));
        }))
    });
    assert_eq!(
        *events.borrow(),
        ["run 0"],
        "SIG-004: the effect did not run when created"
    );

    source.set(1);
    assert_eq!(
        *events.borrow(),
        ["run 0", "cleanup 0", "run 1"],
        "SIG-004: after the signal changed, the effect did not clean up its first run and run again"
    );

    source.set(2);
    assert_eq!(
        *events.borrow(),
        ["run 0", "cleanup 0", "run 1", "cleanup 1", "run 2"],
        "SIG-004: the effect did not clean up and rerun after the next signal change"
    );
    context.runtime().unregister_effect(effect);
    let after_unregister = events.borrow().clone();
    source.set(3);
    assert_eq!(
        *events.borrow(),
        after_unregister,
        "SIG-004: a signal change ran the effect after it was unregistered"
    );
}

/// The runtime's housekeeping removes effects that were disposed, not
/// effects that are merely registered: a registered effect lives until
/// `unregister_effect` or the context's drop (review finding 3).
#[test]
fn sig_004_periodic_cleanup_keeps_a_registered_effect() {
    let context = RuntimeContext::new();
    let source = context.create_signal(0);
    let input = source.clone();
    let runs = Rc::new(RefCell::new(0));
    let counter = runs.clone();
    let effect = context.create_effect(move || {
        input.get();
        *counter.borrow_mut() += 1;
        None
    });
    assert_eq!(*runs.borrow(), 1, "SIG-004: the effect did not run when created");

    context.periodic_cleanup();
    context.cleanup_dead_effects();
    source.set(1);
    assert_eq!(
        *runs.borrow(),
        2,
        "SIG-004: after the runtime's periodic cleanup, a signal change no longer ran the registered effect"
    );

    context.runtime().unregister_effect(effect);
    context.cleanup_dead_effects();
    source.set(2);
    assert_eq!(
        *runs.borrow(),
        2,
        "SIG-004: the effect ran after it was unregistered"
    );
}
