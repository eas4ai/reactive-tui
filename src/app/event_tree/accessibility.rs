use super::{EventTree, Slot};
use crate::{
    accessibility::{Snapshot, Target},
    backend::PaintedNode,
    component::{Element, ElementType},
    event::{
        hit::Bounds,
        router::{EventRouter, NodeId},
    },
};
use accesskit::{Action, Node, Role};
use std::collections::HashMap;

struct Frame<'a> {
    tree: &'a EventTree,
    geometry: HashMap<usize, Bounds>,
    focus: Option<NodeId>,
    index: usize,
    snapshot: Snapshot,
    screen_reader_only: bool,
    text_runs: HashMap<accesskit::NodeId, usize>,
}

impl EventTree {
    pub(in crate::app) fn accessibility_snapshot(
        &self,
        element: &Element,
        geometry: &[PaintedNode],
        router: &EventRouter,
        name: &str,
    ) -> Snapshot {
        let mut frame = Frame {
            tree: self,
            geometry: geometry
                .iter()
                .map(|node| (node.element_index, node.bounds))
                .collect(),
            focus: router.get_focus(),
            index: 0,
            snapshot: Snapshot::empty(name),
            screen_reader_only: false,
            text_runs: HashMap::new(),
        };
        let children = frame.visit(element, Vec::new(), 0, None, false, false);
        frame.snapshot.update.nodes[0].1.set_children(children);
        frame.snapshot
    }
}

impl Frame<'_> {
    fn visit(
        &mut self,
        element: &Element,
        mut path: Vec<Slot>,
        index: usize,
        owner: Option<NodeId>,
        disabled: bool,
        hidden: bool,
    ) -> Vec<accesskit::NodeId> {
        Slot::append(&mut path, element, index);
        let event_id = self.tree.nodes.get(&path).copied();
        let bounds = self
            .geometry
            .get(&self.index)
            .copied()
            .filter(|b| b.width > 0.0 && b.height > 0.0);
        self.index += 1;
        let mut node = element
            .metadata
            .accessibility
            .as_ref()
            .map(|node| (*node.inner).clone())
            .unwrap_or_else(|| Node::new(Role::Unknown));
        let options = element.metadata.accessibility_options.as_deref();
        let ancestor_reader_only = self.screen_reader_only;
        self.screen_reader_only |= options.is_some_and(|o| o.screen_reader_only);
        let screen_reader_only = self.screen_reader_only;
        let focus_event = options.and_then(|options| options.focus_event.as_ref());
        if let Some(label) = options.and_then(|options| options.label.as_ref()) {
            node.set_label(label.clone());
        }
        let hidden = hidden || node.is_hidden() || element.metadata.inert;
        let disabled = disabled || node.is_disabled() || element.metadata.disabled;
        let interactive = !element.metadata.on_click.is_empty();
        let focusable = !disabled
            && element
                .focus
                .as_ref()
                .map_or(interactive, |focus| focus.focusable);
        let owner = if focusable { event_id } else { owner };
        let mut children = Vec::new();
        for (index, child) in element.children.iter().enumerate() {
            children.extend(self.visit(child, path.clone(), index, owner, disabled, hidden));
        }
        self.screen_reader_only = ancestor_reader_only;
        if hidden || (bounds.is_none() && children.is_empty() && !screen_reader_only) {
            return Vec::new();
        }
        let Some(event_id) = event_id else {
            return children;
        };
        let id = accesskit::NodeId(event_id.serial() as u64 + 1);
        if node.role() == Role::Unknown {
            node.set_role(
                if interactive || options.is_some_and(|options| options.clickable) {
                    Role::Button
                } else if matches!(element.element_type, ElementType::Text(_)) {
                    Role::Label
                } else {
                    Role::GenericContainer
                },
            );
        }
        if let ElementType::Text(text) = &element.element_type {
            if node.label().is_none() {
                node.set_label(text.clone());
            }
            if node.role() == Role::Label {
                node.set_value(node.label().unwrap_or(text).to_owned());
            }
        }
        if disabled {
            node.set_disabled();
        }
        let clickable = !disabled
            && (interactive
                || node.supports_action(Action::Click)
                || options.is_some_and(|options| options.clickable));
        node.clear_actions();
        if let (Some(owner), Some(bounds)) = (
            owner.filter(|_| !disabled && (focusable || clickable || focus_event.is_some())),
            bounds,
        ) {
            node.add_action(Action::Focus);
            if clickable {
                node.add_action(Action::Click);
            }
            self.snapshot.targets.insert(
                id,
                Target {
                    node: event_id,
                    owner,
                    focus_event: focus_event.cloned(),
                    click_event: options.and_then(|options| options.click_event.clone()),
                    bounds,
                    clickable,
                },
            );
        }
        if bounds.is_some()
            && !disabled
            && ((self.focus == Some(event_id)
                && self.snapshot.update.focus == accesskit::NodeId(0))
                || (owner.is_some()
                    && options.is_some_and(|options| options.focus)
                    && self.focus == owner))
        {
            self.snapshot.update.focus = id;
        }
        // Bounds stay in terminal cells for event routing. Do not advertise
        // fabricated screen-pixel coordinates to assistive technology.
        if let Some(selection) = element
            .metadata
            .accessibility
            .as_ref()
            .and_then(|node| node.text_selection)
        {
            let runs: Vec<_> = children
                .iter()
                .filter_map(|id| self.text_runs.get(id).map(|length| (*id, *length)))
                .collect();
            let position = |(run, character): (usize, usize)| {
                runs.get(run)
                    .filter(|(_, length)| character <= *length)
                    .map(|(id, _)| accesskit::TextPosition {
                        node: *id,
                        character_index: character,
                    })
            };
            if let (Some(anchor), Some(focus)) = (position(selection[0]), position(selection[1])) {
                node.set_text_selection(accesskit::TextSelection { anchor, focus });
            }
        }
        if node.role() == Role::TextRun {
            self.text_runs.insert(id, node.character_lengths().len());
        }
        node.set_children(children);
        self.snapshot.update.nodes.push((id, node));
        vec![id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        component::Component,
        widgets::input::{InputMode, TextInput, TextInputProps},
    };

    #[test]
    fn popover_click_action_preserves_typed_input_semantics() {
        use crate::{
            component::runtime::ComponentRuntime,
            reactive::{component_scope::ComponentScope, scheduler::Scheduler},
            widgets::display::popover::{Popover, PopoverProps},
        };
        use std::sync::Arc;
        fn input(element: &Element) -> Option<&Element> {
            if element
                .metadata
                .accessibility
                .as_ref()
                .is_some_and(|node| node.inner.role() == Role::TextInput)
            {
                Some(element)
            } else {
                element.children.iter().find_map(input)
            }
        }
        let scope = ComponentScope::new(Arc::new(Scheduler::new()));
        let _binding = scope.enter(true);
        let mut runtime = ComponentRuntime::default();
        let tree = runtime
            .resolve(Element::typed::<Popover>(PopoverProps {
                trigger_element: crate::builder::text_input().value("search").build(),
                ..Default::default()
            }))
            .unwrap();
        let entry = input(&tree).expect("the trigger must retain its input role");
        assert_eq!(
            entry.metadata.accessibility.as_ref().unwrap().inner.value(),
            Some("search")
        );
        assert!(entry
            .metadata
            .accessibility
            .as_ref()
            .unwrap()
            .text_selection
            .is_some());
        assert!(
            entry
                .metadata
                .accessibility_options
                .as_ref()
                .unwrap()
                .clickable
        );
        runtime.clear();
        scope.close();
    }

    #[test]
    fn text_input_exports_readable_unicode_lines_selection_and_password_privacy() {
        let long_cluster = format!("a{}", "\u{301}".repeat(150));
        for (value, mode, expected) in [
            ("", InputMode::SingleLine, ""),
            (
                long_cluster.as_str(),
                InputMode::SingleLine,
                long_cluster.as_str(),
            ),
            ("aé👩‍💻", InputMode::SingleLine, "aé👩‍💻"),
            (
                "first\n第二\n",
                InputMode::MultiLine { height: 3 },
                "first\n第二\n",
            ),
            ("secreté👩‍💻", InputMode::Password, "••••••••"),
        ] {
            let mut props = TextInputProps {
                value: value.into(),
                mode,
                ..Default::default()
            };
            let mut input = TextInput::new(props.clone());
            let mut state = input.initial_state(&props);
            state.is_focused = true;
            for selected in [false, true] {
                if selected {
                    use crate::event::types::{Event, KeyCode, KeyEvent, KeyModifiers};
                    input.handle_event(
                        &Event::Key(
                            KeyEvent::new(KeyCode::Char('a')).with_modifiers(KeyModifiers::ctrl()),
                        ),
                        &mut props,
                        &mut state,
                    );
                }
                let mut element = input.render(&props, &state);
                crate::accessibility::style::prepare(&mut element).unwrap();
                let geometry = [PaintedNode {
                    element_index: 0,
                    bounds: Bounds {
                        x: 0.0,
                        y: 0.0,
                        width: 30.0,
                        height: 3.0,
                    },
                }];
                let mut events = EventTree::default();
                let mut router = EventRouter::new();
                events.sync(&element, &geometry, None, None, &mut router);
                let snapshot = events.accessibility_snapshot(&element, &geometry, &router, "test");
                let tree = accesskit_consumer::Tree::new(snapshot.update, true);
                let tree_state = tree.state();
                let node = tree_state.root().children().next().unwrap();
                assert!(node.supports_text_ranges());
                assert_eq!(node.document_range().text(), expected);
                let selection = node.text_selection().unwrap();
                if selected {
                    assert_eq!(selection.text(), expected);
                } else {
                    assert!(selection.is_degenerate());
                }
            }
        }
    }
}
