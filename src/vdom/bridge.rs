//! VDOM -> Element conversion and ergonomics for using VDOM as an authoring DSL.
//!
//! This module intentionally performs a one-way conversion from VNode to the
//! unified Element tree so we keep a single rendering/reconcile pipeline.

use super::node::{VComponent, VElement, VFragment, VNode, VNodeKey, VText};
use crate::component::{Element, ElementType, LayoutType};

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

/// Convert a VNode (authoring DSL) into an Element (runtime tree)
pub fn vdom_to_element(node: VNode) -> Element {
    match node {
        VNode::Text(VText { content, key }) => {
            let mut el = Element::text(content);
            if let Some(k) = key_to_string(&key) {
                el.key = Some(k);
            }
            el
        }
        VNode::Element(VElement {
            tag,
            key,
            attrs,
            class,
            children,
            ..
        }) => {
            // Map common layout tags to layout elements; otherwise treat as components
            let mut el = match tag.as_str() {
                "flex" => Element::layout(LayoutType::Flex),
                "grid" => Element::layout(LayoutType::Grid),
                "stack" => Element::layout(LayoutType::Stack),
                _ => Element {
                    // component with unit props by default
                    element_type: ElementType::Component(tag),
                    props: std::sync::Arc::new(()),
                    children: Vec::new(),
                    key: None,
                    class: None,
                },
            };

            // Prefer explicit class field; fall back to attrs["class"] if present
            let cls = class.or_else(|| attrs.get("class").cloned());
            if let Some(c) = cls {
                el.class = Some(c);
            }
            if let Some(k) = key_to_string(&key) {
                el.key = Some(k);
            }
            el.children = convert_children(children);
            el
        }
        VNode::Component(VComponent {
            name,
            key,
            props,
            children,
        }) => {
            // Preserve opaque props by storing them directly; components can downcast as needed
            let mut el = Element {
                element_type: ElementType::Component(name),
                props, // already Arc<dyn Any + Send + Sync>
                children: Vec::new(),
                key: None,
                class: None,
            };
            if let Some(k) = key_to_string(&key) {
                el.key = Some(k);
            }
            el.children = convert_children(children);
            el
        }
        VNode::Fragment(VFragment { key, children }) => {
            let mut el = Element::fragment();
            if let Some(k) = key_to_string(&key) {
                el.key = Some(k);
            }
            el.children = convert_children(children);
            el
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
    use crate::component::element_to_nodespec;
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
