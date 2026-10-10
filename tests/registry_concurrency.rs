use reactive_tui::component::{
    instance::AnyComponentInstance, props::EmptyProps, registry::ComponentRegistry, Component,
    ComponentInstance, Element, LifecycleEvent, Props,
};
use reactive_tui::render::tree::NodeKey;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Barrier, Weak,
};
use std::time::Duration;

// A broken registry must fail a test rather than hang the test runner.
fn bounded(work: impl FnOnce() + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
        tx.send(result).unwrap();
    });
    let result = rx
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        .recv_timeout(Duration::from_secs(30))
        .expect("registry operation stalled for 30 seconds");
    worker.join().unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

struct Plain;
impl Component for Plain {
    type Props = EmptyProps;
    type State = ();
    fn new(_: EmptyProps) -> Self {
        Self
    }
    fn render(&self, _: &EmptyProps, _: &()) -> Element {
        Element::text("plain")
    }
}
fn plain() -> AnyComponentInstance {
    AnyComponentInstance::new(ComponentInstance::<Plain>::new(EmptyProps))
}

#[derive(Clone, Default)]
struct ProbeProps {
    registry: Weak<ComponentRegistry>,
    unmounts: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    constructor_reentry: bool,
}
impl PartialEq for ProbeProps {
    fn eq(&self, other: &Self) -> bool {
        self.registry.ptr_eq(&other.registry)
            && Arc::ptr_eq(&self.unmounts, &other.unmounts)
            && Arc::ptr_eq(&self.drops, &other.drops)
            && self.constructor_reentry == other.constructor_reentry
    }
}
impl Props for ProbeProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Probe(ProbeProps);
impl Component for Probe {
    type Props = ProbeProps;
    type State = ();
    fn new(props: ProbeProps) -> Self {
        if props.constructor_reentry {
            let registry = props.registry.upgrade().unwrap();
            registry
                .register::<Plain>(format!(
                    "nested-{}",
                    props.drops.fetch_add(1, Ordering::SeqCst)
                ))
                .unwrap();
            registry.performance_metrics().unwrap();
        }
        Self(props)
    }
    fn render(&self, _: &ProbeProps, _: &()) -> Element {
        Element::text("probe")
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if let Some(registry) = self.0.registry.upgrade() {
            registry.performance_metrics().unwrap();
            registry.registered_names().unwrap();
            if event == LifecycleEvent::Unmount {
                // Exercise writes as well as reads from cleanup callbacks.
                registry
                    .unregister_instance(&NodeKey::named("absent"))
                    .unwrap();
            }
        }
        if event == LifecycleEvent::Unmount {
            self.0.unmounts.fetch_add(1, Ordering::SeqCst);
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
fn probe_props(registry: &Arc<ComponentRegistry>) -> ProbeProps {
    ProbeProps {
        registry: Arc::downgrade(registry),
        ..Default::default()
    }
}
fn probe(props: ProbeProps) -> AnyComponentInstance {
    AnyComponentInstance::new(ComponentInstance::<Probe>::new(props))
}

#[test]
fn constructors_can_register_components() {
    bounded(|| {
        let registry = Arc::new(ComponentRegistry::new());
        registry.register::<Probe>("probe").unwrap();
        let props = ProbeProps {
            constructor_reentry: true,
            ..probe_props(&registry)
        };
        assert!(registry.create_by_name("probe", &props).unwrap().is_some());
        assert!(registry
            .create_by_type::<Probe>(props.clone())
            .unwrap()
            .is_some());
        assert!(registry
            .create_by_type_id(std::any::TypeId::of::<Probe>(), &props)
            .unwrap()
            .is_some());
        let key = NodeKey::named("clone");
        registry
            .register_instance(key.clone(), probe(props))
            .unwrap();
        assert!(registry.get_instance(&key).unwrap().is_some());
        registry.cleanup_all().unwrap();
    });
}

/// Run `operation` against a registered probe and return the probe's final
/// `(unmounts, drops)` counters once the registry has been cleaned up.
fn cleanup_case(operation: fn(&ComponentRegistry, &NodeKey)) -> (usize, usize) {
    let unmounts = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let counters = (Arc::clone(&unmounts), Arc::clone(&drops));
    bounded(move || {
        let registry = Arc::new(ComponentRegistry::new());
        let props = ProbeProps {
            unmounts: counters.0,
            drops: counters.1,
            ..probe_props(&registry)
        };
        let key = NodeKey::named("probe");
        registry
            .register_instance(key.clone(), probe(props.clone()))
            .unwrap();
        assert_eq!(registry.active_count().unwrap(), 1);
        operation(&registry, &key);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
        assert_eq!(props.drops.load(Ordering::SeqCst), 1);
        registry.cleanup_all().unwrap();
        assert_eq!(registry.active_count().unwrap(), 0);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
    });
    (
        unmounts.load(Ordering::SeqCst),
        drops.load(Ordering::SeqCst),
    )
}
#[test]
fn unregister_allows_callback_reentry() {
    assert_eq!(
        cleanup_case(|r, k| r.unregister_instance(k).unwrap()),
        (1, 1),
        "probe unmounted and dropped exactly once"
    );
}
#[test]
fn replacement_allows_callback_reentry() {
    assert_eq!(
        cleanup_case(|r, k| r.register_instance(k.clone(), plain()).unwrap()),
        (1, 1),
        "replaced probe unmounted and dropped exactly once"
    );
}
#[test]
fn element_replacement_allows_callback_reentry() {
    assert_eq!(
        cleanup_case(|r, k| {
            r.register_instance_with_element(k.clone(), plain(), &Element::text("replacement"))
                .unwrap()
        }),
        (1, 1),
        "element-replaced probe unmounted and dropped exactly once"
    );
}
#[test]
fn orphan_cleanup_allows_callback_reentry() {
    cleanup_case(|r, _| {
        assert_eq!(
            r.cleanup_orphaned_components(&Default::default()).unwrap(),
            1
        )
    });
}
#[test]
fn bulk_cleanup_allows_callback_reentry() {
    cleanup_case(|r, _| assert_eq!(r.cleanup_all().unwrap(), 1));
}
#[test]
fn clear_allows_callback_reentry() {
    assert_eq!(
        cleanup_case(|r, _| r.clear().unwrap()),
        (1, 1),
        "cleared probe unmounted and dropped exactly once"
    );
}
#[test]
fn clear_all_allows_callback_reentry() {
    assert_eq!(
        cleanup_case(|r, _| r.clear_all().unwrap()),
        (1, 1),
        "clear_all probe unmounted and dropped exactly once"
    );
}
#[test]
fn registry_drop_releases_owned_instances() {
    bounded(|| {
        let registry = Arc::new(ComponentRegistry::new());
        let props = probe_props(&registry);
        registry
            .register_instance(NodeKey::auto(), probe(props.clone()))
            .unwrap();
        drop(registry);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
        assert_eq!(props.drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn metrics_and_instance_mutation_make_concurrent_progress() {
    bounded(|| {
        let registry = Arc::new(ComponentRegistry::new());
        let start = Arc::new(Barrier::new(5));
        let mut workers = Vec::new();
        for worker in 0..4 {
            let registry = registry.clone();
            let start = start.clone();
            workers.push(std::thread::spawn(move || {
                start.wait();
                for _ in 0..1000 {
                    if worker == 0 {
                        registry.performance_metrics().unwrap();
                    } else if worker == 1 {
                        registry.cleanup_all().unwrap();
                    } else {
                        let key = NodeKey::auto();
                        registry.register_instance(key.clone(), plain()).unwrap();
                        registry.unregister_instance(&key).unwrap();
                    }
                }
            }));
        }
        start.wait();
        for worker in workers {
            worker.join().unwrap();
        }
        registry.cleanup_all().unwrap();
        let metrics = registry.performance_metrics().unwrap();
        assert_eq!(metrics.active_components, 0);
        assert_eq!(metrics.total_created, 2000);
        assert_eq!(metrics.total_destroyed, 2000);
    });
}

#[test]
fn orphan_sweep_preserves_live_instances_and_counts_removals_once() {
    bounded(|| {
        let registry = Arc::new(ComponentRegistry::new());
        let props = probe_props(&registry);
        let keep = NodeKey::named("keep");
        let orphan = NodeKey::named("orphan");
        registry
            .register_instance(keep.clone(), probe(props.clone()))
            .unwrap();
        registry
            .register_instance(orphan.clone(), probe(props.clone()))
            .unwrap();
        let active = [keep.clone()].into_iter().collect();
        assert_eq!(registry.cleanup_orphaned_components(&active).unwrap(), 1);
        assert_eq!(registry.active_count().unwrap(), 1);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
        registry.unregister_instance(&orphan).unwrap();
        assert_eq!(registry.cleanup_orphaned_components(&active).unwrap(), 0);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
        registry.unregister_instance(&keep).unwrap();
        let metrics = registry.performance_metrics().unwrap();
        assert_eq!(
            (
                metrics.total_created,
                metrics.total_destroyed,
                metrics.active_components
            ),
            (2, 2, 0)
        );
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn shared_registry_owns_instances_until_last_clone_drops() {
    bounded(|| {
        let registry = Arc::new(ComponentRegistry::new());
        let props = probe_props(&registry);
        let shared = registry.as_ref().clone();
        registry
            .register_instance(NodeKey::auto(), probe(props.clone()))
            .unwrap();
        drop(registry);
        assert_eq!(shared.active_count().unwrap(), 1);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 0);
        drop(shared);
        assert_eq!(props.unmounts.load(Ordering::SeqCst), 1);
        assert_eq!(props.drops.load(Ordering::SeqCst), 1);
    });
}
