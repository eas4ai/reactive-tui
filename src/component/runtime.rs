//! App-owned instances. Registry entries are factories, not owners of this tree.

use super::{
    registry::get_global_registry, AnyComponentInstance, Element, ElementType, LifecycleEvent,
};
use crate::error::{ReactiveError, Result};
use crate::reactive::component_scope::ComponentScope;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Segment {
    Key(String),
    Index(usize),
    Output(String),
}

type Path = Vec<Segment>;

struct LiveComponent {
    name: String,
    instance: AnyComponentInstance,
    scope: Arc<ComponentScope>,
}

#[derive(Default)]
pub(crate) struct ComponentRuntime {
    instances: HashMap<Path, LiveComponent>,
}

impl ComponentRuntime {
    pub(crate) fn resolve(&mut self, element: Element) -> Result<Element> {
        let mut seen = HashSet::new();
        let path = vec![Self::slot(&element, 0)];
        let result = self.expand(element, path, &mut seen, 0)?;
        self.remove_where(|path| !seen.contains(path));
        Ok(result)
    }

    fn slot(element: &Element, index: usize) -> Segment {
        element
            .key
            .as_ref()
            .map_or(Segment::Index(index), |key| Segment::Key(key.clone()))
    }

    fn expand(
        &mut self,
        mut element: Element,
        path: Path,
        seen: &mut HashSet<Path>,
        depth: usize,
    ) -> Result<Element> {
        if depth > 128 {
            return Err(ReactiveError::invalid_state(
                "component tree exceeds expansion depth 128",
            ));
        }
        if let ElementType::Component(name) = &element.element_type {
            let name = name.clone();
            if self
                .instances
                .get(&path)
                .is_some_and(|live| live.name != name)
            {
                self.remove_where(|candidate| candidate.starts_with(&path));
            }
            let newly_created = !self.instances.contains_key(&path);
            if newly_created {
                let scope = ComponentScope::child(
                    crate::reactive::component_scope::current()
                        .expect("App component expansion requires its resource scope")
                        .scheduler(),
                );
                let _binding = scope.enter(true);
                // create_by_name releases registry locks before calling user code.
                if let Some(mut instance) =
                    get_global_registry().create_by_name(&name, element.props.as_ref())?
                {
                    instance.on_lifecycle(LifecycleEvent::Mount);
                    self.instances.insert(
                        path.clone(),
                        LiveComponent {
                            name: name.clone(),
                            instance,
                            scope: scope.clone(),
                        },
                    );
                }
            }
            if let Some(live) = self.instances.get_mut(&path) {
                let _binding = live.scope.enter(!newly_created);
                seen.insert(path.clone());
                live.instance.update(element.props.as_ref());
                let mut output = live.instance.render();
                output.children.append(&mut element.children);
                let mut output_path = path;
                output_path.push(Segment::Output(name));
                output_path.push(Self::slot(&output, 0));
                let mut resolved = self.expand(output, output_path, seen, depth + 1)?;
                resolved.metadata.on_click.extend(element.metadata.on_click);
                resolved.metadata.disabled |= element.metadata.disabled;
                // Caller styling belongs to the rendered root and takes precedence.
                if let Some(class) = element.class {
                    resolved.class = Some(match resolved.class {
                        Some(inner) => format!("{inner} {class}"),
                        None => class,
                    });
                }
                if element.focus.is_some() {
                    resolved.focus = element.focus;
                }
                if element.key.is_some() {
                    resolved.key = element.key;
                }
                return Ok(resolved);
            }
            // Unknown names retain the existing container behavior.
        }
        let mut keys = HashSet::new();
        let children = std::mem::take(&mut element.children);
        for (index, child) in children.into_iter().enumerate() {
            if let Some(key) = &child.key {
                if !keys.insert(key.clone()) {
                    return Err(ReactiveError::invalid_state(format!(
                        "duplicate sibling key: {key}"
                    )));
                }
            }
            let mut child_path = path.clone();
            child_path.push(Self::slot(&child, index));
            element
                .children
                .push(self.expand(child, child_path, seen, depth + 1)?);
        }
        Ok(element)
    }

    fn remove_where(&mut self, remove: impl Fn(&Path) -> bool) -> usize {
        let mut removed: Vec<_> = self
            .instances
            .keys()
            .filter(|path| remove(path))
            .cloned()
            .collect();
        // A parent cleanup may observe its children: release children first.
        removed.sort_by_key(|path| std::cmp::Reverse(path.len()));
        let count = removed.len();
        for path in removed {
            if let Some(mut live) = self.instances.remove(&path) {
                let _binding = live.scope.enter(false);
                live.instance.on_lifecycle(LifecycleEvent::Unmount);
                live.scope.close();
                drop(live);
            }
        }
        count
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.remove_where(|_| true)
    }
}

impl Drop for ComponentRuntime {
    fn drop(&mut self) {
        self.clear();
    }
}
