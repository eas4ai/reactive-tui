use reactive_tui::hooks::{
    use_callback_ref, use_forwarded_ref, use_local_ref, use_multi_ref, use_ref, with_local_hooks,
    CallbackRef, Ref,
};
use reactive_tui::reactive::Hooks;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[test]
fn shared_reference_retains_value_and_identity_across_renders() {
    let hooks = Hooks::new();
    let first = use_ref(&hooks, String::from("initial"));
    first.set_current("retained".into());
    hooks.reset();
    let second = use_ref(&hooks, String::from("replacement initializer"));
    assert_eq!(second.current(), "retained");
    second.set_current("shared".into());
    assert_eq!(first.current(), "shared");
}

#[test]
fn local_reference_retains_non_send_value_across_renders() {
    with_local_hooks(|| {
        let hooks = Hooks::new();
        let value = std::rc::Rc::new(std::cell::Cell::new(1));
        let first = use_local_ref(&hooks, value.clone());
        first.current().set(7);
        hooks.reset();
        let second = use_local_ref(&hooks, std::rc::Rc::new(std::cell::Cell::new(99)));
        assert!(std::rc::Rc::ptr_eq(&value, &second.current()));
        assert_eq!(second.current().get(), 7);
        second.current().set(11);
        assert_eq!(first.current().get(), 11);
    });
}

#[test]
fn callback_reference_retains_current_value_across_renders() {
    let hooks = Hooks::new();
    let first = use_callback_ref(&hooks, |_: Option<String>| {});
    first.set(Some("selected".into()));
    hooks.reset();
    let second = use_callback_ref(&hooks, |_: Option<String>| {});
    assert_eq!(second.current().as_deref(), Some("selected"));
    second.set(None);
    assert_eq!(first.current(), None);
}

#[test]
fn multi_reference_retains_members_across_renders() {
    let hooks = Hooks::new();
    let first = use_multi_ref(&hooks);
    let member = first.add_ref(3);
    hooks.reset();
    let second = use_multi_ref(&hooks);
    assert_eq!(second.count(), 1);
    second.set_all(8);
    assert_eq!(member.current(), 8);
    second.clear();
    assert_eq!(first.count(), 0);
}

#[test]
fn forwarded_reference_follows_the_current_parent() {
    let hooks = Hooks::new();
    let first = Ref::new(1);
    let second = Ref::new(2);
    use_forwarded_ref(&hooks, Some(first.clone())).set_if_exists(3);
    hooks.reset();
    use_forwarded_ref(&hooks, Some(second.clone())).set_if_exists(4);
    assert_eq!(first.current(), 3);
    assert_eq!(second.current(), 4);
    hooks.reset();
    assert!(!use_forwarded_ref::<i32>(&hooks, None).is_set());
}

#[test]
fn callback_reference_allows_read_and_write_reentry() {
    const CHILD: &str = "RTUI_RESIDUAL_REF_REENTRY_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let owner: Arc<Mutex<Option<CallbackRef<i32>>>> = Arc::new(Mutex::new(None));
        let callback_owner = owner.clone();
        let reference = CallbackRef::new(move |value| {
            let reference = callback_owner.lock().unwrap().as_ref().unwrap().clone();
            assert_eq!(reference.current(), value);
            if value == Some(1) {
                reference.set(Some(2));
            }
        });
        *owner.lock().unwrap() = Some(reference.clone());
        reference.set(Some(1));
        assert_eq!(reference.current(), Some(2));
        owner.lock().unwrap().take();

        #[derive(Clone)]
        struct ReentrantDrop {
            armed: Arc<std::sync::atomic::AtomicBool>,
            owner: std::sync::Weak<Mutex<Option<CallbackRef<ReentrantDrop>>>>,
        }
        impl Drop for ReentrantDrop {
            fn drop(&mut self) {
                if self.armed.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    let owner = self.owner.upgrade().unwrap();
                    let reference = owner.lock().unwrap().as_ref().unwrap().clone();
                    reference.set(None);
                }
            }
        }
        let owner = Arc::new(Mutex::new(None));
        let reference = CallbackRef::new(|_: Option<ReentrantDrop>| {});
        *owner.lock().unwrap() = Some(reference.clone());
        let armed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        reference.set(Some(ReentrantDrop {
            armed: armed.clone(),
            owner: Arc::downgrade(&owner),
        }));
        armed.store(true, std::sync::atomic::Ordering::SeqCst);
        reference.set(None);
        assert!(reference.current().is_none());
        owner.lock().unwrap().take();
        return;
    }

    // A deadlock is confined to this child. Always kill and reap it on timeout.
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "callback_reference_allows_read_and_write_reentry",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .spawn()
        .unwrap();
    // a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(
                status.success(),
                "reentrant callback child failed: {status}"
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("callback reference deadlocked during read/write reentry");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn shared_reference_slots_and_owners_are_independent() {
    let hooks = Hooks::new();
    let other = Hooks::new();
    let first = use_ref(&hooks, 1);
    let second = use_ref(&hooks, 2);
    let independent = use_ref(&other, 3);
    first.set_current(10);
    second.set_current(20);
    hooks.reset();
    assert_eq!(use_ref(&hooks, 99).current(), 10);
    assert_eq!(use_ref(&hooks, 99).current(), 20);
    assert_eq!(independent.current(), 3);
}

#[test]
fn shared_hooks_retain_state_without_returned_handles() {
    let hooks = Hooks::new();
    {
        let reference = use_ref(&hooks, 1);
        reference.set_current(7);
        let callback = use_callback_ref(&hooks, |_: Option<i32>| {});
        callback.set(Some(8));
        let multi = use_multi_ref(&hooks);
        multi.add_ref(9);
    }
    hooks.reset();
    assert_eq!(use_ref(&hooks, 99).current(), 7);
    assert_eq!(
        use_callback_ref(&hooks, |_: Option<i32>| {}).current(),
        Some(8)
    );
    assert_eq!(use_multi_ref::<i32>(&hooks).count(), 1);
}

#[test]
fn retained_callback_handles_use_the_latest_render_capture() {
    let hooks = Hooks::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = calls.clone();
    let first = use_callback_ref(&hooks, move |value: Option<i32>| {
        first_calls.lock().unwrap().push((1, value));
    });
    first.set(Some(10));
    hooks.reset();
    let next_calls = calls.clone();
    let second = use_callback_ref(&hooks, move |value: Option<i32>| {
        next_calls.lock().unwrap().push((2, value));
    });
    first.set(Some(20));
    second.set(None);
    assert_eq!(
        *calls.lock().unwrap(),
        [(1, Some(10)), (2, Some(20)), (2, None)]
    );
}

#[test]
fn reference_hooks_preserve_existing_type_bounds() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Hooks>();
    assert_send_sync::<Ref<std::sync::atomic::AtomicUsize>>();
    assert_send_sync::<CallbackRef<std::cell::Cell<i32>>>();
    let hooks = Hooks::new();
    // AtomicUsize is not Clone; shared refs still support construction and set.
    let reference = use_ref(&hooks, std::sync::atomic::AtomicUsize::new(3));
    let pointer = unsafe { reference.as_ptr() };
    reference.set_current(std::sync::atomic::AtomicUsize::new(4));
    assert_eq!(pointer, unsafe { reference.as_ptr() });
    // SAFETY: `reference` is alive, and no mutation overlaps this load.
    assert_eq!(
        unsafe { (*pointer).load(std::sync::atomic::Ordering::SeqCst) },
        4
    );
    // Cell is Send but not Sync.
    let callback = use_callback_ref(&hooks, |_: Option<std::cell::Cell<i32>>| {});
    callback.set(Some(std::cell::Cell::new(4)));
    assert_eq!(callback.current().unwrap().get(), 4);
}
