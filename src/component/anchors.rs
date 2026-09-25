//! Named geometry owned by one App, replaced after acknowledged presentation
//! and, after a resize, from the layout the App prepares before presenting.

use super::Element;
use crate::{backend::PaintedNode, event::hit::Bounds};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

#[derive(Default)]
pub(crate) struct Anchors {
    requested: Mutex<HashSet<String>>,
    bounds: Mutex<HashMap<String, Option<Bounds>>>,
}

impl Anchors {
    pub(crate) fn begin_render(&self) {
        self.requested.lock().unwrap().clear();
    }
    pub(crate) fn lookup(&self, key: &str) -> Result<Bounds, String> {
        self.requested.lock().unwrap().insert(key.to_owned());
        match self.bounds.lock().unwrap().get(key) {
            Some(Some(bounds)) => Ok(*bounds),
            Some(None) => Err(format!("Dialog anchor is ambiguous: {key}")),
            None => Err(format!("Dialog anchor not resolved: {key}")),
        }
    }

    pub(crate) fn publish(&self, element: &Element, geometry: &[PaintedNode]) -> bool {
        let requested = self.requested.lock().unwrap().clone();
        if requested.is_empty() {
            self.clear();
            return false;
        }
        let geometry: HashMap<_, _> = geometry
            .iter()
            .map(|node| (node.element_index, node.bounds))
            .collect();
        let mut next = HashMap::new();
        let mut index = 0;
        collect(element, &geometry, &requested, &mut index, &mut next);
        let mut bounds = self.bounds.lock().unwrap();
        if *bounds == next {
            return false;
        }
        *bounds = next;
        true
    }

    pub(crate) fn clear(&self) -> bool {
        let mut bounds = self.bounds.lock().unwrap();
        let changed = !bounds.is_empty();
        bounds.clear();
        changed
    }
}

fn collect(
    element: &Element,
    geometry: &HashMap<usize, Bounds>,
    requested: &HashSet<String>,
    index: &mut usize,
    found: &mut HashMap<String, Option<Bounds>>,
) {
    if let Some(key) = element.key.as_ref().filter(|key| requested.contains(*key)) {
        if let Some(bounds) = geometry.get(index).filter(|bounds| {
            [bounds.x, bounds.y, bounds.width, bounds.height]
                .iter()
                .all(|value| value.is_finite())
                && bounds.width > 0.0
                && bounds.height > 0.0
        }) {
            found
                .entry(key.clone())
                .and_modify(|value| *value = None)
                .or_insert(Some(*bounds));
        }
    }
    *index += 1;
    for child in &element.children {
        collect(child, geometry, requested, index, found);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(index: usize, x: f32) -> PaintedNode {
        PaintedNode {
            element_index: index,
            bounds: Bounds::new(x, 2.0, 6.0, 2.0),
        }
    }

    #[test]
    fn anchors_replace_moved_and_removed_geometry_without_cross_app_state() {
        let first = Anchors::default();
        let second = Anchors::default();
        let target = Element::text("target").with_key("target");
        assert!(first.lookup("target").is_err());
        assert!(second.lookup("target").is_err());
        assert!(first.publish(&target, &[painted(0, 3.0)]));
        assert!(second.publish(&target, &[painted(0, 12.0)]));
        assert_eq!(first.lookup("target").unwrap().x, 3.0);
        assert_eq!(second.lookup("target").unwrap().x, 12.0);
        assert!(!first.publish(&target, &[painted(0, 3.0)]));
        assert!(first.publish(&target, &[painted(0, 5.0)]));
        assert_eq!(first.lookup("target").unwrap().x, 5.0);
        assert!(first.publish(&target, &[]));
        assert!(first.lookup("target").is_err());
        assert_eq!(second.lookup("target").unwrap().x, 12.0);
    }

    #[test]
    fn anchors_reject_ambiguity_and_release_unrequested_names() {
        let anchors = Anchors::default();
        let target = Element::text("target").with_key("target");
        let root = Element::layout(super::super::LayoutType::Flex)
            .with_children(vec![target.clone(), target]);
        assert!(anchors.lookup("target").is_err());
        assert!(anchors.publish(&root, &[painted(1, 3.0), painted(2, 12.0)]));
        assert!(anchors.lookup("target").unwrap_err().contains("ambiguous"));
        assert!(anchors.publish(&root, &[painted(1, 3.0)]));
        assert_eq!(anchors.lookup("target").unwrap().x, 3.0);
        anchors.begin_render();
        assert!(!anchors.publish(&root, &[painted(1, 3.0)]));
        assert!(anchors.bounds.lock().unwrap().is_empty());
        assert!(anchors.requested.lock().unwrap().is_empty());
    }
}
