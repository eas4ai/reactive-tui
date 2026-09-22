//! Synchronous argument bridge for the public zero-argument menu action API.
//! This slot never owns checked state; each invocation restores its caller's slot.
use std::cell::Cell;

thread_local! {
    static CHECKED: Cell<Option<bool>> = const { Cell::new(None) };
}

struct Restore(Option<bool>);
impl Drop for Restore {
    fn drop(&mut self) {
        CHECKED.set(self.0);
    }
}

pub(super) fn run(checked: Option<bool>, callback: &dyn Fn()) {
    let _restore = Restore(CHECKED.replace(checked));
    callback();
}

pub(super) fn take_checked(fallback: bool) -> bool {
    CHECKED.take().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_restores_after_panics_and_consumes_only_once() {
        run(Some(true), &|| {
            let result = std::panic::catch_unwind(|| {
                run(Some(false), &|| panic!("intentional callback panic"));
            });
            assert!(result.is_err());
            assert!(take_checked(false));
            assert!(!take_checked(false));
        });
        assert!(!take_checked(false));
    }

    #[test]
    fn concurrent_scopes_are_isolated() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = [true, false]
            .into_iter()
            .map(|value| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    run(Some(value), &|| {
                        barrier.wait();
                        assert_eq!(take_checked(!value), value);
                    });
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
