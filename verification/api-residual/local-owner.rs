//! External lifetime and thread-confinement checks for scoped local references.
use reactive_tui::hooks::{use_local_ref, with_local_hooks};
use reactive_tui::reactive::Hooks;
use std::rc::Rc;

#[cfg(not(send_local))]
fn main() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<Hooks>();
    struct Tracked(Arc<AtomicUsize>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let destroyed = Arc::new(AtomicUsize::new(0));
    with_local_hooks(|| {
        let hooks = Hooks::new();
        drop(use_local_ref(
            &hooks,
            Rc::new(Tracked(Arc::clone(&destroyed))),
        ));
        assert_eq!(
            destroyed.load(Ordering::SeqCst),
            0,
            "local value was released before the Hooks owner"
        );
        std::thread::spawn(move || drop(hooks)).join().unwrap();
        assert_eq!(
            destroyed.load(Ordering::SeqCst),
            0,
            "foreign cleanup must defer to the creator"
        );
    });
    assert_eq!(
        destroyed.load(Ordering::SeqCst),
        1,
        "scope exit must release the retained share"
    );
    println!("LOCAL_OWNER_SCOPE_OK");
}

#[cfg(send_local)]
fn main() {
    with_local_hooks(|| {
        let hooks = Hooks::new();
        let local = use_local_ref(&hooks, Rc::new(1));
        // A compile failure is required: local handles must remain thread-confined.
        std::thread::spawn(move || drop(local)).join().unwrap();
    });
}
