use reactive_tui::component::{element_to_nodespec, Element, LayoutType};
use std::borrow::Cow;

#[test]
fn text_element_to_nodespec() {
    let el = Element::text("Hello");
    let ns = element_to_nodespec(&el);
    assert_eq!(ns.text, Some(Cow::<str>::Owned("Hello".to_string())));
}

#[test]
fn layout_default_class_mapping() {
    let el = Element::layout(LayoutType::Flex);
    let ns = element_to_nodespec(&el);
    assert_eq!(ns.class, Cow::<str>::Owned("flex".to_string()));

    let el2 = Element::layout(LayoutType::Grid);
    let ns2 = element_to_nodespec(&el2);
    assert_eq!(ns2.class, Cow::<str>::Owned("grid".to_string()));
}

#[test]
fn element_with_class_overrides_default() {
    let el = Element::layout(LayoutType::Flex).with_class("flex items-center p-2");
    let ns = element_to_nodespec(&el);
    assert_eq!(
        ns.class,
        Cow::<str>::Owned("flex items-center p-2".to_string())
    );
}
