//! VDOM integration for the builder API
//!
//! This module provides interoperability between the builder API and the virtual DOM system,
//! allowing seamless mixing of builder-created elements and VDOM nodes.

use super::core::ElementBuilder;
use crate::component::{Element, ElementType, LayoutType};

/// Integration helpers for VDOM interop
///
/// Convert a VNode to Element (re-export for convenience)
pub fn from_vdom(vnode: crate::vdom::VNode) -> Element {
    crate::vdom::bridge::vdom_to_element(vnode)
}

/// Create an element that can contain both builder and VDOM children
pub fn mixed_container() -> MixedElementBuilder {
    MixedElementBuilder::new(ElementType::Layout(LayoutType::Flex))
}

/// Builder that accepts both Element and VNode children
pub struct MixedElementBuilder {
    builder: ElementBuilder,
}

impl MixedElementBuilder {
    fn new(element_type: ElementType) -> Self {
        Self {
            builder: ElementBuilder::new(element_type),
        }
    }

    /// Set CSS classes
    pub fn class(mut self, classes: &str) -> Self {
        self.builder = self.builder.class(classes);
        self
    }

    /// Add a builder Element child
    pub fn child_element(mut self, child: Element) -> Self {
        self.builder = self.builder.child(child);
        self
    }

    /// Add a VDOM VNode child (automatically converted)
    pub fn child_vdom(mut self, child: crate::vdom::VNode) -> Self {
        self.builder = self.builder.child(from_vdom(child));
        self
    }

    /// Add mixed children (Elements and VNodes)
    pub fn mixed_children(mut self, children: Vec<MixedChild>) -> Self {
        for child in children {
            match child {
                MixedChild::Element(el) => {
                    self.builder = self.builder.child(el);
                }
                MixedChild::VNode(vnode) => {
                    self.builder = self.builder.child(from_vdom(vnode));
                }
            }
        }
        self
    }

    /// Set a key
    pub fn key(mut self, key: &str) -> Self {
        self.builder = self.builder.key(key);
        self
    }

    /// Build the final Element
    pub fn build(self) -> Element {
        self.builder.build()
    }
}

/// Enum for mixed children types
// Keep the public Element(Element) constructor compatible. Element's optional
// accessibility payload is boxed; boxing this variant would break callers.
#[allow(clippy::large_enum_variant)]
pub enum MixedChild {
    /// A built Element from the builder API
    Element(Element),
    /// A virtual DOM node from the VDOM system
    VNode(crate::vdom::VNode),
}

impl From<Element> for MixedChild {
    fn from(element: Element) -> Self {
        MixedChild::Element(element)
    }
}

impl From<crate::vdom::VNode> for MixedChild {
    fn from(vnode: crate::vdom::VNode) -> Self {
        MixedChild::VNode(vnode)
    }
}
