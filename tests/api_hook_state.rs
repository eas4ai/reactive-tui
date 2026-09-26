use reactive_tui::{
    component,
    component::{props::EmptyProps, Component, Element, ElementType},
    reactive::{use_memo, use_signal, Hooks},
};
use std::panic::{catch_unwind, AssertUnwindSafe};

#[component]
fn Persistent(hooks: &Hooks) -> Element {
    let count = use_signal(hooks, 0usize);
    count.update(|value| *value += 1);
    Element::text(count.get().to_string())
}
#[component]
fn PersistentProps(hooks: &Hooks, label: String) -> Element {
    let count = use_signal(hooks, 0usize);
    count.update(|value| *value += 1);
    Element::text(format!("{label}:{}", count.get()))
}
#[component]
fn Conditional(hooks: &Hooks, extra: bool) -> Element {
    let value = use_signal(hooks, 1usize);
    if *extra {
        let _extra = use_signal(hooks, 2usize);
    }
    Element::text(value.get().to_string())
}
fn text(element: Element) -> String {
    match element.element_type {
        ElementType::Text(text) => text,
        _ => panic!("expected text"),
    }
}
#[test]
fn generated_components_reuse_state_with_and_without_props() {
    let component = Persistent::new(EmptyProps);
    let with_props = PersistentProps::new(PersistentPropsProps {
        label: "initial".into(),
    });
    for n in 1..=1000 {
        assert_eq!(text(component.render(&EmptyProps, &())), n.to_string());
        assert_eq!(
            text(with_props.render(
                &PersistentPropsProps {
                    label: format!("label{n}")
                },
                &()
            )),
            format!("label{n}:{n}")
        );
    }
}
#[test]
fn memo_keeps_its_signal_and_refreshes_the_value() {
    let hooks = Hooks::new();
    let first = use_memo(&hooks, || 1usize);
    assert_eq!(first.get(), 1);
    hooks.reset();
    let second = use_memo(&hooks, || 2usize);
    assert_eq!(second.get(), 2);
    assert_eq!(
        first.get(),
        2,
        "the retained handle must see the refreshed value"
    );
    hooks.reset();
    let third = use_memo(&hooks, || 2usize);
    third.set(7);
    assert_eq!(first.get(), 7, "memo signal identity must be stable");
}
#[test]
fn changed_hook_types_are_rejected_instead_of_detached() {
    let hooks = Hooks::new();
    let original = use_signal(&hooks, 1usize);
    hooks.reset();
    assert!(catch_unwind(AssertUnwindSafe(|| use_signal(&hooks, String::new()))).is_err());
    hooks.reset();
    let recovered = use_signal(&hooks, 99usize);
    recovered.set(42);
    assert_eq!(original.get(), 42);
}
#[test]
fn generated_frames_reject_fewer_or_more_hooks() {
    for (first, second) in [(true, false), (false, true)] {
        let component = Conditional::new(ConditionalProps { extra: first });
        component.render(&ConditionalProps { extra: first }, &());
        assert!(catch_unwind(AssertUnwindSafe(
            || component.render(&ConditionalProps { extra: second }, &())
        ))
        .is_err());
        assert_eq!(
            text(component.render(&ConditionalProps { extra: first }, &())),
            "1"
        );
    }
}
#[test]
fn memo_and_state_cannot_silently_swap_the_same_value_type() {
    let hooks = Hooks::new();
    let _ = use_signal(&hooks, 1usize);
    hooks.reset();
    assert!(catch_unwind(AssertUnwindSafe(|| use_memo(&hooks, || 2usize))).is_err());
}
