//! SIG-001: the thread-safe signal's read-modify-writes are applied once and
//! whole (docs/spec/reactive.md).

use reactive_tui::reactive::hooks::{use_reducer, Hooks, ThreadSafeSignal};
use std::sync::{Arc, Barrier};
use std::time::Duration;

/// Increments each of two threads applies; enough that two copies taken at
/// the same time, and the later store erasing the earlier, show in the sum.
const ROUNDS: usize = 10_000;

/// SIG-001: two threads that each dispatch 10,000 increments to one reducer
/// state leave exactly 20,000.
#[test]
fn sig_001_two_threads_dispatching_increments_lose_no_action() {
    let hooks = Hooks::new();
    let (state, dispatch) = {
        let _render = hooks.begin_render();
        use_reducer(&hooks, |count: &usize, step: usize| count + step, 0usize)
    };
    let start = Arc::new(Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let dispatch = Arc::clone(&dispatch);
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..ROUNDS {
                    dispatch(1);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("a dispatching thread panicked");
    }
    assert_eq!(
        state.get(),
        2 * ROUNDS,
        "SIG-001: every dispatched increment is applied once, so two threads of {ROUNDS} leave {}",
        2 * ROUNDS
    );
    drop(hooks);
}

/// SIG-001: two threads that each apply 10,000 increments through
/// `update_atomic` leave exactly 20,000.
#[test]
fn sig_001_two_threads_calling_update_atomic_lose_no_increment() {
    let signal = ThreadSafeSignal::new(0_usize);
    let start = Arc::new(Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let signal = signal.clone();
            let start = Arc::clone(&start);
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..ROUNDS {
                    signal.update_atomic(|n| *n += 1);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("an updating thread panicked");
    }
    assert_eq!(
        signal.get(),
        2 * ROUNDS,
        "SIG-001: every update_atomic increment is applied once"
    );
}

/// SIG-001: an `update_atomic` callback that calls back into its signal fails
/// at once with a message naming the call, instead of waiting on the lock.
#[test]
fn sig_001_a_call_back_into_the_signal_from_update_atomic_fails_with_a_message() {
    let signal = ThreadSafeSignal::new(1_i32);
    let nested = signal.clone();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            signal.update_atomic(|value| {
                nested.set(2);
                *value = 3;
            })
        }));
        let message = match outcome {
            Ok(()) => String::new(),
            Err(panic) => panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default(),
        };
        send.send((message, signal.get())).unwrap();
    });
    let (message, value) = receive
        .recv_timeout(Duration::from_secs(10))
        .expect("SIG-001: the nested set waited on the lock instead of failing");
    assert!(
        message.contains("ThreadSafeSignal::set called from the update_atomic callback"),
        "SIG-001: the failure names the call: {message:?}"
    );
    assert_eq!(value, 1, "a failed update_atomic stores nothing");
}
