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

/// `Ok(())` when `name` resolves to component `C`; the error names what went wrong.
fn resolves<C: Component>(registry: &ComponentRegistry, name: &str) -> Result<(), String> {
    let instance = registry
        .create_by_name(name, &EmptyProps)
        .unwrap()
        .ok_or_else(|| format!("registered name {name:?} was missing"))?;
    if instance.type_id() != TypeId::of::<C>() {
        return Err(format!("wrong component for {name:?}"));
    }
    Ok(())
}
/// `Ok(())` when `name` resolves to nothing; the error names the stale alias.
fn missing(registry: &ComponentRegistry, name: &str) -> Result<(), String> {
    if registry
        .create_by_name(name, &EmptyProps)
        .unwrap()
        .is_some()
    {
        return Err(format!("stale name {name:?}"));
    }
    Ok(())
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
        assert_eq!(resolves::<Alpha>(&left, "shared"), Ok(()));
        assert_eq!(resolves::<Beta>(&right, "shared"), Ok(()));
        assert_eq!(resolves::<Beta>(&left, "other"), Ok(()));
        assert_eq!(resolves::<Alpha>(&right, "other"), Ok(()));
    }
}

#[test]
fn independent_registries_do_not_share_names() {
    let left = ComponentRegistry::new();
    let right = ComponentRegistry::new();
    left.register::<Alpha>("left").unwrap();
    right.register::<Alpha>("right").unwrap();
    assert_eq!(resolves::<Alpha>(&left, "left"), Ok(()));
    assert_eq!(missing(&right, "left"), Ok(()));
    assert_eq!(resolves::<Alpha>(&right, "right"), Ok(()));
    assert_eq!(missing(&left, "right"), Ok(()));
}

#[test]
fn dropped_registry_does_not_seed_the_next_registry() {
    for index in 0..40 {
        let registry = ComponentRegistry::new();
        let name = format!("registry-{index}");
        registry.register::<Alpha>(&name).unwrap();
        assert_eq!(resolves::<Alpha>(&registry, &name), Ok(()));
        if index > 0 {
            assert_eq!(
                missing(&registry, &format!("registry-{}", index - 1)),
                Ok(())
            );
        }
    }
}

#[test]
fn clone_before_registration_observes_shared_names() {
    let original = ComponentRegistry::new();
    let shared = original.clone();
    assert_eq!(missing(&shared, "alpha"), Ok(()));
    original.register::<Alpha>("alpha").unwrap();
    assert_eq!(resolves::<Alpha>(&shared, "alpha"), Ok(()));
}

#[test]
fn warmed_lookup_observes_registration_through_another_clone() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("alpha").unwrap();
    let shared = original.clone();
    assert_eq!(resolves::<Alpha>(&original, "alpha"), Ok(()));
    assert_eq!(missing(&original, "beta"), Ok(()));
    shared.register::<Beta>("beta").unwrap();
    assert_eq!(resolves::<Beta>(&original, "beta"), Ok(()));
}

#[test]
fn clear_and_reregister_through_clone_removes_old_aliases() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("old").unwrap();
    let shared = original.clone();
    assert_eq!(resolves::<Alpha>(&original, "old"), Ok(()));
    shared.clear().unwrap();
    shared.register::<Alpha>("new").unwrap();
    assert_eq!(missing(&original, "old"), Ok(()));
    assert_eq!(resolves::<Alpha>(&original, "new"), Ok(()));
    shared.clear_all().unwrap();
    shared.register::<Beta>("replacement").unwrap();
    assert_eq!(missing(&original, "new"), Ok(()));
    assert_eq!(resolves::<Beta>(&original, "replacement"), Ok(()));
}

#[test]
fn completed_cross_thread_mutations_reach_a_warmed_reader() {
    let original = ComponentRegistry::new();
    original.register::<Alpha>("old").unwrap();
    let reader = original.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (changed_tx, changed_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        assert_eq!(resolves::<Alpha>(&reader, "old"), Ok(()));
        ready_tx.send(()).unwrap();
        changed_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(missing(&reader, "old"), Ok(()));
        assert_eq!(resolves::<Beta>(&reader, "new"), Ok(()));
    });
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    original.clear_all().unwrap();
    original.register::<Beta>("new").unwrap();
    changed_tx.send(()).unwrap();
    worker.join().unwrap();
}
