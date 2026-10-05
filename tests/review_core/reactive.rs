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
