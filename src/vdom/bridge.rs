//! VDOM -> Element conversion and ergonomics for using VDOM as an authoring DSL.
//!
//! This module intentionally performs a one-way conversion from VNode to the
//! unified Element tree so we keep a single rendering/reconcile pipeline.

use super::node::{VComponent, VElement, VFragment, VNode, VNodeKey, VText};
use crate::component::{Element, LayoutType};
use crate::event::router::EventResult;
use crate::event::types::Event;
use std::sync::Arc;

/// An opaque native component payload prevents a lossy second representation of
/// Element metadata, typed factories, focus and styles in the authoring DSL.
pub(super) struct NativeElement(pub Element);

pub(super) fn element_to_vdom(mut element: Element) -> VNode {
    let key = element
        .key
        .as_ref()
        .map_or(VNodeKey::None, |key| VNodeKey::String(key.clone()));
    let children = std::mem::take(&mut element.children)
        .into_iter()
        .map(element_to_vdom)
        .collect();
    VNode::Component(VComponent {
        name: element
            .component_name()
            .unwrap_or("native-element")
            .to_owned(),
        key,
        props: Arc::new(NativeElement(element)),
        children,
    })
}

fn attach_events(el: &mut Element, handlers: super::node::EventHandlerMap) {
    if handlers.is_empty() {
        return;
    }
    if handlers.contains_key("click") || handlers.contains_key("key") {
        el.focus.get_or_insert_with(Default::default);
    }
    el.metadata.events.push(Arc::new(move |event| {
        let name = match event {
            Event::Key(_) => "key",
            Event::Mouse(_) => "mouse",
            Event::Focus(_) => "focus",
            Event::Paste(_) => "paste",
            Event::Custom(_) => "custom",
            Event::Resize(_) => return EventResult::Ignored,
        };
        let observed = if let Some(handler) = handlers.get(name) {
            handler(event);
            true
        } else {
            false
        };
        if event.activates_control() {
            if let Some(handler) = handlers.get("click") {
                handler(event);
                return EventResult::Consumed;
            }
        }
        if observed {
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }));
}

fn key_to_string(key: &VNodeKey) -> Option<String> {
    match key {
        VNodeKey::String(s) => Some(s.clone()),
        VNodeKey::Number(n) => Some(n.to_string()),
        VNodeKey::None => None,
    }
}

fn convert_children(children: Vec<VNode>) -> Vec<Element> {
    children.into_iter().map(vdom_to_element).collect()
}

fn velement_to_element(node: VElement) -> Element {
    let VElement {
        tag,
        key,
        attrs,
        class,
        children,
        props,
        event_handlers,
        style,
    } = node;
    let mut element = match tag.as_str() {
        "flex" => Element::layout(LayoutType::Flex),
        "grid" => Element::layout(LayoutType::Grid),
        "stack" => Element::layout(LayoutType::Stack),
        _ => Element::component(tag),
    };
    element.metadata.inline_styles = style.or_else(|| attrs.get("style").cloned());
    element.metadata.disabled = attrs.get("disabled").is_some_and(|value| value != "false");
    if let Some(id) = attrs.get("id") {
        element = element.with_accessibility_id(id);
    }
    if attrs.get("autofocus").is_some_and(|value| value != "false") {
        element = element.auto_focus();
    }
    attach_events(&mut element, event_handlers);
    element.class = class.or_else(|| attrs.get("class").cloned());
    element.key = key_to_string(&key);
    element.props = Arc::new(super::node::VElementProps {
        values: props,
        attrs,
    });
    element.children = convert_children(children);
    element
}

/// Convert a VNode authoring tree into the native App tree, retaining behavior.
pub fn vdom_to_element(node: VNode) -> Element {
    match node {
        VNode::Text(VText { content, key }) => {
            let mut element = Element::text(content);
            element.key = key_to_string(&key);
            element
        }
        VNode::Element(node) => velement_to_element(node),
        VNode::Component(VComponent {
            name,
            key,
            props,
            children,
        }) => {
            let mut element = if let Some(native) = props.downcast_ref::<NativeElement>() {
                native.0.clone()
            } else {
                let mut element = Element::component(name);
                element.props = props;
                element
            };
            element.key = key_to_string(&key);
            element.children = convert_children(children);
            element
        }
        VNode::Fragment(VFragment { key, children }) => {
            let mut element = Element::fragment();
            element.key = key_to_string(&key);
            element.children = convert_children(children);
            element
        }
        VNode::Empty => Element::empty(),
    }
}

/// Convenience macro: convert any VNode expression into an Element using vdom_to_element.
///
/// Usage:
///   let el = vdom!(h!("div", { class: "p-2" }, [VNode::text("Hello") ]));
#[macro_export]
macro_rules! vdom {
    ($vnode_expr:expr) => {
        $crate::vdom::bridge::vdom_to_element($vnode_expr)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{element_to_nodespec, ElementType};
    use crate::vdom::VNode;

    #[test]
    fn text_node_converts() {
        let v = VNode::text("Hello");
        let e = vdom_to_element(v);
        assert!(matches!(e.element_type, ElementType::Text(_)));
    }

    #[test]
    fn element_with_class_converts() {
        let v = VNode::element("div")
            .class("container")
            .child(VNode::text("Hi"))
            .build();
        let e = vdom_to_element(v);
        assert!(matches!(e.element_type, ElementType::Component(ref n) if n == "div"));
        assert_eq!(e.class.as_deref(), Some("container"));
        assert_eq!(e.children.len(), 1);
    }

    #[test]
    fn keys_are_preserved() {
        let v = VNode::element("div").key("k1").build();
        let e = vdom_to_element(v);
        assert_eq!(e.key.as_deref(), Some("k1"));
    }

    #[test]
    fn roundtrip_to_nodespec_keeps_class_and_children() {
        let v = VNode::element("flex")
            .class("flex gap-2")
            .child(VNode::text("A"))
            .child(VNode::text("B"))
            .build();
        let e = vdom_to_element(v);
        let ns = element_to_nodespec(&e);
        assert_eq!(ns.class.as_ref(), "flex gap-2");
        assert_eq!(ns.children.len(), 2);
    }
}
