//! SIG-001: the thread-safe signal's read-modify-writes are applied once and
//! whole (docs/spec/reactive.md).

use reactive_tui::reactive::hooks::{use_reducer, Hooks};
use std::sync::{Arc, Barrier};

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
