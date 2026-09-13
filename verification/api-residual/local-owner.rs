//! Compile and lifetime probes for the unchanged local-reference API.
use reactive_tui::hooks::use_local_ref;
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
    let hooks = Hooks::new();
    let reference = use_local_ref(&hooks, Rc::new(Tracked(Arc::clone(&destroyed))));
    drop(reference);
    let before_owner_drop = destroyed.load(Ordering::SeqCst);
    println!("Hooks is Send + Sync; arbitrary Rc local value compiled");
    println!("Destroyed after returned handle removal: {before_owner_drop}");
    std::thread::spawn(move || drop(hooks)).join().unwrap();
    println!("Hooks moved to and dropped on a different thread");
    println!(
        "Destroyed after owner removal: {}",
        destroyed.load(Ordering::SeqCst)
    );
    assert_eq!(
        before_owner_drop, 0,
        "local value was released before the Hooks owner"
    );
}

#[cfg(send_local)]
fn main() {
    let hooks = Hooks::new();
    let local = use_local_ref(&hooks, Rc::new(1));
    // A compile failure is required: local handles must remain thread-confined.
    std::thread::spawn(move || drop(local)).join().unwrap();
}
