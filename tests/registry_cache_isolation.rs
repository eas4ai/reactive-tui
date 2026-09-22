use reactive_tui::component::{props::EmptyProps, registry::ComponentRegistry, Component, Element};
use std::any::TypeId;
use std::sync::mpsc;
use std::time::Duration;

struct Alpha;
struct Beta;
macro_rules! component {
    ($name:ident) => {
        impl Component for $name {
            type Props = EmptyProps;
            type State = ();
            fn new(_: EmptyProps) -> Self {
                Self
            }
            fn render(&self, _: &EmptyProps, _: &()) -> Element {
                Element::text(stringify!($name))
            }
        }
    };
}
component!(Alpha);
component!(Beta);

fn resolves<C: Component>(registry: &ComponentRegistry, name: &str) {
    let instance = registry
        .create_by_name(name, &EmptyProps)
        .unwrap()
        .unwrap_or_else(|| panic!("registered name {name:?} was missing"));
    assert_eq!(
        instance.type_id(),
        TypeId::of::<C>(),
        "wrong component for {name:?}"
    );
}
fn missing(registry: &ComponentRegistry, name: &str) {
    assert!(
        registry
            .create_by_name(name, &EmptyProps)
            .unwrap()
            .is_none(),
        "stale name {name:?}"
    );
}

#[test]
fn independent_registries_resolve_their_own_types_at_equal_versions() {
    let left = ComponentRegistry::new();
    let right = ComponentRegistry::new();
    left.register::<Alpha>("shared").unwrap();
    left.register::<Beta>("other").unwrap();
    right.register::<Beta>("shared").unwrap();
    right.register::<Alpha>("other").unwrap();
    for _ in 0..20 {
        resolves::<Alpha>(&left, "shared");
        resolves::<Beta>(&right, "shared");
        resolves::<Beta>(&left, "other");
        resolves::<Alpha>(&right, "other");
    }
}

#[test]
fn independent_registries_do_not_share_names() {
    let left = ComponentRegistry::new();
    let right = ComponentRegistry::new();
    left.register::<Alpha>("left").unwrap();
    right.register::<Alpha>("right").unwrap();
    resolves::<Alpha>(&left, "left");
    missing(&right, "left");
    resolves::<Alpha>(&right, "right");
    missing(&left, "right");
}

#[test]
fn dropped_registry_does_not_seed_the_next_registry() {
    for index in 0..40 {
        let registry = ComponentRegistry::new();
        let name = format!("registry-{index}");
        registry.register::<Alpha>(&name).unwrap();
        resolves::<Alpha>(&registry, &name);
        if index > 0 {
            missing(&registry, &format!("registry-{}", index - 1));
        }
    }
}

#[test]
fn clone_before_registration_observes_shared_names() {
    let original = ComponentRegistry::new();
    let shared = original.clone();
    missing(&shared, "alpha");
    original.register::<Alpha>("alpha").unwrap();
    resolves::<Alpha>(&shared, "alpha");
}

#[test]
fn warmed_lookup_observes_registration_through_another_clone() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("alpha").unwrap();
    let shared = original.clone();
    resolves::<Alpha>(&original, "alpha");
    missing(&original, "beta");
    shared.register::<Beta>("beta").unwrap();
    resolves::<Beta>(&original, "beta");
}

#[test]
fn clear_and_reregister_through_clone_removes_old_aliases() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("old").unwrap();
    let shared = original.clone();
    resolves::<Alpha>(&original, "old");
    shared.clear().unwrap();
    shared.register::<Alpha>("new").unwrap();
    missing(&original, "old");
    resolves::<Alpha>(&original, "new");
    shared.clear_all().unwrap();
    shared.register::<Beta>("replacement").unwrap();
    missing(&original, "new");
    resolves::<Beta>(&original, "replacement");
}

#[test]
fn completed_cross_thread_mutations_reach_a_warmed_reader() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("old").unwrap();
    let reader = original.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (changed_tx, changed_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        resolves::<Alpha>(&reader, "old");
        ready_tx.send(()).unwrap();
        changed_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        missing(&reader, "old");
        resolves::<Beta>(&reader, "new");
    });
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    original.clear_all().unwrap();
    original.register::<Beta>("new").unwrap();
    changed_tx.send(()).unwrap();
    worker.join().unwrap();
}
