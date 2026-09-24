use reactive_tui::hooks::{use_local_ref, use_ref, with_local_hooks};
use reactive_tui::reactive::Hooks;
use std::{
    cell::Cell,
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};

struct Tracked(Rc<Cell<usize>>);
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn rejects(f: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(f)).is_err());
}

#[test]
fn local_slots_retain_without_handles_and_isolate_owners() {
    with_local_hooks(|| {
        let a = Hooks::new();
        let b = Hooks::new();
        drop(use_local_ref(&a, Rc::new(Cell::new(7))));
        drop(use_local_ref(&a, String::from("second")));
        drop(use_local_ref(&b, Rc::new(Cell::new(99))));
        a.reset();
        b.reset();
        assert_eq!(use_local_ref(&a, Rc::new(Cell::new(0))).current().get(), 7);
        assert_eq!(use_local_ref(&a, String::new()).current(), "second");
        assert_eq!(use_local_ref(&b, Rc::new(Cell::new(0))).current().get(), 99);
    });
}

#[test]
fn cleanup_and_last_clone_drop_release_local_shares() {
    with_local_hooks(|| {
        let count = Rc::new(Cell::new(0));
        let a = Hooks::new();
        let b = a.clone();
        drop(use_local_ref(&a, Tracked(count.clone())));
        drop(a);
        assert_eq!(count.get(), 0);
        drop(b);
        assert_eq!(count.get(), 1);
        let a = Hooks::new();
        drop(use_local_ref(&a, Tracked(count.clone())));
        a.cleanup();
        assert_eq!(count.get(), 2);
        a.cleanup();
        assert_eq!(count.get(), 2);
        rejects(|| {
            use_local_ref(&a, 0);
        });
    });
}

#[test]
fn foreign_owner_drop_defers_until_creator_sweep_or_exit() {
    let count = Rc::new(Cell::new(0));
    with_local_hooks(|| {
        let a = Hooks::new();
        drop(use_local_ref(&a, Tracked(count.clone())));
        std::thread::spawn(move || drop(a)).join().unwrap();
        assert_eq!(count.get(), 0);
        let other = Hooks::new();
        use_local_ref(&other, 0);
        assert_eq!(count.get(), 1);
        let a = Hooks::new();
        drop(use_local_ref(&a, Tracked(count.clone())));
        std::thread::spawn(move || a.cleanup()).join().unwrap();
        assert_eq!(count.get(), 1);
    });
    assert_eq!(count.get(), 2);
}

#[test]
fn scopes_reject_missing_wrong_thread_and_expired_owners() {
    let a = Hooks::new();
    rejects(|| {
        use_local_ref(&a, 0);
    });
    with_local_hooks(|| {
        use_local_ref(&a, 0);
        a.reset();
        with_local_hooks(|| {
            rejects(|| {
                use_local_ref(&a, 0);
            })
        });
        let other = a.clone();
        std::thread::spawn(move || {
            with_local_hooks(|| {
                rejects(|| {
                    use_local_ref(&other, 0);
                })
            })
        })
        .join()
        .unwrap();
        a.reset();
        assert_eq!(use_local_ref(&a, 99).current(), 0);
    });
    a.reset();
    with_local_hooks(|| {
        rejects(|| {
            use_local_ref(&a, 0);
        })
    });
}

#[test]
fn local_slots_validate_kind_type_and_generated_count() {
    with_local_hooks(|| {
        let a = Hooks::new();
        {
            let _frame = a.begin_render();
            use_local_ref(&a, 1usize);
        }
        rejects(|| {
            let _frame = a.begin_render();
            use_local_ref(&a, String::new());
        });
        rejects(|| {
            let _frame = a.begin_render();
            use_ref(&a, 1usize);
        });
        rejects(|| {
            let _frame = a.begin_render();
        });
        rejects(|| {
            let _frame = a.begin_render();
            use_local_ref(&a, 1usize);
            use_local_ref(&a, 2usize);
        });
        {
            let _frame = a.begin_render();
            assert_eq!(use_local_ref(&a, 9usize).current(), 1);
        }
    });
}

#[test]
fn unwinding_releases_scope_but_escaped_handles_keep_ownership() {
    let count = Rc::new(Cell::new(0));
    let hooks = Hooks::new();
    rejects(|| {
        with_local_hooks(|| {
            drop(use_local_ref(&hooks, Tracked(count.clone())));
            panic!("scope unwind");
        })
    });
    assert_eq!(count.get(), 1);
    let escaped = with_local_hooks(|| use_local_ref(&Hooks::new(), Tracked(count.clone())));
    assert_eq!(count.get(), 1);
    drop(escaped);
    assert_eq!(count.get(), 2);
}

#[test]
fn cleanup_destructors_can_reenter_another_local_owner() {
    struct Reenter(Hooks, Rc<Cell<usize>>);
    impl Drop for Reenter {
        fn drop(&mut self) {
            self.0.reset();
            assert_eq!(use_local_ref(&self.0, 99).current(), 7);
            self.1.set(self.1.get() + 1);
        }
    }
    with_local_hooks(|| {
        let other = Hooks::new();
        use_local_ref(&other, 7);
        let count = Rc::new(Cell::new(0));
        let a = Hooks::new();
        drop(use_local_ref(&a, Reenter(other, count.clone())));
        a.cleanup();
        assert_eq!(count.get(), 1);
    });
}

#[test]
fn smoke_thread_safe_hooks_and_generated_state_remain_send_sync() {
    fn require<T: Send + Sync>() {}
    require::<Hooks>();
}

use reactive_tui::{
    app::{App, AppWaker, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    component::{Component, Element},
    error::{ReactiveError, Result},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

struct LocalCounter {
    value: Cell<usize>,
    drops: Arc<AtomicUsize>,
}
impl Drop for LocalCounter {
    fn drop(&mut self) {
        if self.value.get() > 0 {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
}
#[derive(Clone, Debug, Default)]
struct DropCount(Arc<AtomicUsize>);
impl PartialEq for DropCount {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
#[reactive_tui::component]
fn ScopedCounter(hooks: &Hooks, drops: DropCount) -> Element {
    let counter = use_local_ref(
        hooks,
        Rc::new(LocalCounter {
            value: Cell::new(0),
            drops: drops.0.clone(),
        }),
    );
    let value = counter.current();
    value.value.set(value.value.get() + 1);
    Element::text(value.value.get().to_string())
}
impl Default for ScopedCounterProps {
    fn default() -> Self {
        Self {
            // Sentinel: registered components must receive the test counter.
            drops: DropCount(Arc::new(AtomicUsize::new(usize::MAX))),
        }
    }
}

struct LocalRoot {
    component: ScopedCounter,
    props: ScopedCounterProps,
    wake: Option<AppWaker>,
    rendered: AtomicUsize,
    observed: Arc<Mutex<Vec<String>>>,
    mode: u8,
}
impl RootComponent for LocalRoot {
    fn attach_waker(&mut self, wake: AppWaker) {
        self.wake = Some(wake);
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn render(&self) -> Element {
        let element = self.component.render(&self.props, &());
        if let reactive_tui::component::ElementType::Text(value) = &element.element_type {
            self.observed.lock().unwrap().push(value.clone());
        }
        let count = self.rendered.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(wake) = &self.wake {
            if count >= 2 {
                wake.request_stop();
            } else {
                wake.request_redraw();
            }
        }
        element
    }
    fn update(&mut self) -> Result<RootUpdate> {
        if self.rendered.load(Ordering::SeqCst) > 0 {
            if self.mode == 1 {
                return Err(ReactiveError::invalid_state("local scope error control"));
            }
            assert_ne!(self.mode, 2, "local scope panic control");
        }
        Ok(RootUpdate::Unchanged)
    }
}
fn local_root(mode: u8, drops: Arc<AtomicUsize>, observed: Arc<Mutex<Vec<String>>>) -> LocalRoot {
    let props = ScopedCounterProps {
        drops: DropCount(drops),
    };
    LocalRoot {
        component: ScopedCounter::new(props.clone()),
        props,
        wake: None,
        rendered: AtomicUsize::new(0),
        observed,
        mode,
    }
}
struct LocalBackend {
    inner: SuprTuiBackend,
    deadline: Instant,
}
impl reactive_tui::backend::Backend for LocalBackend {
    fn render_frame(&mut self, e: &Element) -> Result<bool> {
        self.inner.render_frame(e)
    }
    fn apply_patches(
        &mut self,
        _: &[reactive_tui::render::reconcile::PatchOp],
        _: &reactive_tui::render::RenderTree,
    ) -> Result<()> {
        panic!("full frame required")
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn present(&mut self) -> Result<()> {
        self.inner.present()
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<reactive_tui::event::types::Event>> {
        panic!("wake-aware input required")
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<std::time::Duration>,
        wake: &AppWaker,
    ) -> Result<Option<reactive_tui::event::types::Event>> {
        assert!(
            Instant::now() < self.deadline,
            "local App fixture did not stop"
        );
        wake.wait(Some(Duration::from_millis(10)));
        Ok(None)
    }
}
fn run_local_root(root: impl RootComponent + 'static) -> Result<()> {
    App::builder()
        .backend(LocalBackend {
            inner: SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap(),
            // a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
            deadline: Instant::now() + Duration::from_secs(30),
        })
        .root(root)
        .build()
        .unwrap()
        .run()
}

#[test]
fn app_supplies_scope_for_generated_components_and_closes_on_error_and_unwind() {
    for mode in 0..=2 {
        let drops = Arc::new(AtomicUsize::new(0));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let result = catch_unwind(AssertUnwindSafe(|| {
            run_local_root(local_root(mode, drops.clone(), observed.clone()))
        }));
        match mode {
            0 => assert!(result.unwrap().is_ok()),
            1 => assert!(result.unwrap().is_err()),
            _ => assert!(result.is_err()),
        }
        assert_eq!(
            drops.load(Ordering::SeqCst),
            1,
            "retained value must die on App exit"
        );
        assert_eq!(observed.lock().unwrap()[0], "1");
    }
}

#[test]
fn app_reuses_manual_scope_and_leaves_other_owners_alive() {
    with_local_hooks(|| {
        let other = Hooks::new();
        use_local_ref(&other, 42);
        let drops = Arc::new(AtomicUsize::new(0));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let root = local_root(0, drops.clone(), observed.clone());
        root.render();
        run_local_root(root).unwrap();
        assert_eq!(*observed.lock().unwrap(), ["1", "2"]);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        other.reset();
        assert_eq!(use_local_ref(&other, 99).current(), 42);
    });
}

#[test]
fn generated_local_component_can_move_before_its_first_render() {
    fn require<T: Send + Sync>() {}
    require::<ScopedCounter>();
    let drops = Arc::new(AtomicUsize::new(0));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let root = local_root(0, drops.clone(), observed);
    std::thread::spawn(move || run_local_root(root))
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn keyed_component_removal_releases_local_values_before_scope_exit() {
    struct KeyedRoot {
        drops: Arc<AtomicUsize>,
        frames: AtomicUsize,
        wake: Option<AppWaker>,
    }
    impl RootComponent for KeyedRoot {
        fn attach_waker(&mut self, wake: AppWaker) {
            self.wake = Some(wake);
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn render(&self) -> Element {
            let frame = self.frames.fetch_add(1, Ordering::SeqCst);
            let wake = self.wake.as_ref().unwrap();
            if frame == 2 {
                assert_eq!(self.drops.load(Ordering::SeqCst), 1);
                wake.request_stop();
            } else {
                wake.request_redraw();
            }
            if frame == 0 {
                ScopedCounter::element(DropCount(self.drops.clone())).key("local")
            } else {
                Element::text("removed")
            }
        }
    }
    reactive_tui::component::registry::register_component::<ScopedCounter>("ScopedCounter")
        .unwrap();
    with_local_hooks(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        run_local_root(KeyedRoot {
            drops: drops.clone(),
            frames: AtomicUsize::new(0),
            wake: None,
        })
        .unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn inner_scope_cleanup_reclaims_outer_owner_without_closing_inner() {
    with_local_hooks(|| {
        let count = Rc::new(Cell::new(0));
        let outer = Hooks::new();
        drop(use_local_ref(&outer, Tracked(count.clone())));
        with_local_hooks(|| {
            let inner = Hooks::new();
            use_local_ref(&inner, 7);
            outer.cleanup();
            assert_eq!(count.get(), 1);
            inner.reset();
            assert_eq!(use_local_ref(&inner, 99).current(), 7);
        });
    });
}
