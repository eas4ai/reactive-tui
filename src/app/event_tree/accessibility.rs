use super::{EventTree, Slot};
use crate::{
    accessibility::{Snapshot, Target},
    backend::PaintedNode,
    component::{Element, ElementType, FieldPart},
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
    /// Whether a shown child asked its parent to carry `busy` (DIS-004).
    child_busy: bool,
    /// The node of each marked form part, in the order the parts finish
    /// visiting (CTL-010).
    field_parts: Vec<(FieldPart, accesskit::NodeId)>,
    /// How many marked form parts are being visited now.
    open_parts: usize,
    /// While a form part is being visited: each node published for an
    /// element that carries a node of its own or a text, with the element's
    /// place in pre-order (CTL-010).
    part_nodes: Vec<(usize, accesskit::NodeId)>,
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
            child_busy: false,
            field_parts: Vec::new(),
            open_parts: 0,
            part_nodes: Vec::new(),
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
        path: Vec<Slot>,
        index: usize,
        owner: Option<NodeId>,
        disabled: bool,
        hidden: bool,
    ) -> Vec<accesskit::NodeId> {
        let parts = self.field_parts.len();
        let part_nodes = self.part_nodes.len();
        let options = element.metadata.accessibility_options.as_deref();
        let field_part = options.and_then(|options| options.field_part);
        let field_row = options.and_then(|options| options.field_row);
        self.open_parts += usize::from(field_part.is_some());
        let ids = self.visit_node(element, path, index, owner, disabled, hidden);
        if let Some(part) = field_part {
            self.open_parts -= 1;
            // The part's node is the first element under the part, in
            // pre-order, that carries a node of its own or a text: the
            // control's own node, not a text run inside it, and the label's
            // text, not the row that holds it.
            if let Some((_, id)) = self.part_nodes[part_nodes..]
                .iter()
                .min_by_key(|(order, _)| *order)
            {
                self.field_parts.push((part, *id));
            }
            if self.open_parts == 0 {
                self.part_nodes.clear();
            }
        }
        if let Some(row) = field_row {
            self.link_field_row(parts, row.required);
            // A row's parts are its own: a row around it links its control
            // to the parts it holds, not to these.
            self.field_parts.truncate(parts);
        }
        ids
    }

    /// Tell the control of the form row whose parts were visited since
    /// `parts` its label, its description and whether it is required
    /// (CTL-010).
    fn link_field_row(&mut self, parts: usize, required: bool) {
        let found = |wanted: FieldPart| {
            self.field_parts[parts..]
                .iter()
                .find(|(part, _)| *part == wanted)
                .map(|(_, id)| *id)
        };
        let label = found(FieldPart::Label);
        let description = found(FieldPart::Description);
        let Some(control) = found(FieldPart::Control) else {
            return;
        };
        let Some((_, node)) = self
            .snapshot
            .update
            .nodes
            .iter_mut()
            .find(|(id, _)| *id == control)
        else {
            return;
        };
        if let Some(label) = label {
            node.set_labelled_by(vec![label]);
        }
        if let Some(description) = description {
            node.set_described_by(vec![description]);
        }
        if required {
            node.set_required();
        }
    }

    fn visit_node(
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
        let order = self.index;
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
        let saved_child_busy = std::mem::take(&mut self.child_busy);
        let mut children = Vec::new();
        for (index, child) in element.children.iter().enumerate() {
            children.extend(self.visit(child, path.clone(), index, owner, disabled, hidden));
        }
        let child_busy = std::mem::replace(&mut self.child_busy, saved_child_busy);
        self.screen_reader_only = ancestor_reader_only;
        // A shown element with `busy_parent` asks its nearest node-carrying
        // ancestor to carry `busy`.
        if !hidden && bounds.is_some() && options.is_some_and(|options| options.busy_parent) {
            self.child_busy = true;
        }
        if hidden || (bounds.is_none() && children.is_empty() && !screen_reader_only) {
            // This element publishes no node, so a busy request passes on.
            self.child_busy |= child_busy;
            return Vec::new();
        }
        let Some(event_id) = event_id else {
            self.child_busy |= child_busy;
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
        if child_busy {
            node.set_busy();
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
            // Each value action the element takes an event for (CTL-005).
            let set_value_event = options.and_then(|options| options.set_value_event.clone());
            let increment_event = options.and_then(|options| options.increment_event.clone());
            let decrement_event = options.and_then(|options| options.decrement_event.clone());
            for (action, event) in [
                (Action::SetValue, &set_value_event),
                (Action::Increment, &increment_event),
                (Action::Decrement, &decrement_event),
            ] {
                if event.is_some() {
                    node.add_action(action);
                }
            }
            self.snapshot.targets.insert(
                id,
                Target {
                    node: event_id,
                    owner,
                    focus_event: focus_event.cloned(),
                    click_event: options.and_then(|options| options.click_event.clone()),
                    set_value_event,
                    increment_event,
                    decrement_event,
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
        if self.open_parts > 0
            && (element.metadata.accessibility.is_some()
                || matches!(element.element_type, ElementType::Text(_)))
        {
            self.part_nodes.push((order, id));
        }
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
    fn dis_004_a_skeleton_marks_its_parent_busy_in_the_published_tree() {
        use crate::{
            accessibility::Node,
            builder::{div, skeleton},
            component::runtime::ComponentRuntime,
            reactive::{component_scope::ComponentScope, scheduler::Scheduler},
        };
        use std::sync::Arc;
        let mut group = Node::new(Role::Group);
        group.set_label("Report");
        let scope = ComponentScope::new(Arc::new(Scheduler::new()));
        let _binding = scope.enter(true);
        let mut runtime = ComponentRuntime::default();
        // The skeleton's render runs when the tree resolves.
        let element = runtime
            .resolve(
                div()
                    .child(skeleton().rows(2).build())
                    .build()
                    .with_accessibility(group),
            )
            .unwrap();
        // Every element in pre-order gets bounds, so the skeleton shows.
        let geometry: Vec<PaintedNode> = (0..10)
            .map(|element_index| PaintedNode {
                element_index,
                bounds: Bounds {
                    x: 0.0,
                    y: element_index as f32,
                    width: 30.0,
                    height: 1.0,
                },
            })
            .collect();
        let mut events = EventTree::default();
        let mut router = EventRouter::new();
        events.sync(&element, &geometry, None, None, &mut router);
        let snapshot = events.accessibility_snapshot(&element, &geometry, &router, "test");
        let report = snapshot
            .update
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Report"))
            .map(|(_, node)| node)
            .expect("the group must publish its node");
        assert!(report.is_busy());
        let busy: Vec<_> = snapshot
            .update
            .nodes
            .iter()
            .filter(|(_, node)| node.is_busy())
            .collect();
        assert_eq!(busy.len(), 1, "only the parent carries busy");
        // The first node is the window's root, which is not part of the skeleton.
        for (_, node) in snapshot.update.nodes.iter().skip(1) {
            if node.label() != Some("Report") {
                assert!(
                    matches!(node.role(), Role::GenericContainer | Role::Unknown),
                    "the skeleton must not publish a node of its own"
                );
            }
        }
        runtime.clear();
        scope.close();
    }

    #[test]
    fn ctl_010_a_field_rows_control_is_labelled_and_described_by_its_parts() {
        use crate::{
            accessibility::Node,
            component::{FieldPart, FieldRow, LayoutType},
        };
        // A row of a label, a control and a description, the control required.
        let row = Element::layout(LayoutType::Flex)
            .children(vec![
                Element::text("Name").with_field_part(FieldPart::Label),
                Element::layout(LayoutType::Flex)
                    .with_accessibility(Node::new(Role::TextInput))
                    .with_field_part(FieldPart::Control),
                Element::text("Work address").with_field_part(FieldPart::Description),
            ])
            .with_field_row(FieldRow { required: true });
        let element = Element::layout(LayoutType::Flex).with_child(row);
        // Every element in pre-order gets bounds, so each one publishes.
        let geometry: Vec<PaintedNode> = (0..5)
            .map(|element_index| PaintedNode {
                element_index,
                bounds: Bounds {
                    x: 0.0,
                    y: element_index as f32,
                    width: 30.0,
                    height: 1.0,
                },
            })
            .collect();
        let mut events = EventTree::default();
        let mut router = EventRouter::new();
        events.sync(&element, &geometry, None, None, &mut router);
        let snapshot = events.accessibility_snapshot(&element, &geometry, &router, "test");
        let id_of = |label: &str| {
            snapshot
                .update
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .map(|(id, _)| *id)
                .unwrap_or_else(|| panic!("{label} publishes a node"))
        };
        let (label, description) = (id_of("Name"), id_of("Work address"));
        let (_, control) = snapshot
            .update
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::TextInput)
            .expect("the control publishes its node");
        assert_eq!(control.labelled_by(), &[label]);
        assert_eq!(control.described_by(), &[description]);
        assert!(control.is_required());
    }

    #[test]
    fn ctl_010_a_field_rows_link_names_the_controls_own_node_and_the_labels_text() {
        use crate::{
            accessibility::Node,
            component::{FieldPart, FieldRow, LayoutType},
        };
        // As the form builds a row: the label is a row of the label's text
        // and the required mark; the control part wraps a control whose own
        // node holds a text of its own, as a text input's runs are.
        let label = Element::layout(LayoutType::Flex)
            .children(vec![Element::text("Name"), Element::text("*")])
            .with_field_part(FieldPart::Label);
        let control = Element::layout(LayoutType::Flex)
            .with_child(
                Element::layout(LayoutType::Flex)
                    .with_child(Element::text("Ada"))
                    .with_accessibility(Node::new(Role::TextInput)),
            )
            .with_field_part(FieldPart::Control);
        let description = Element::text("Work address").with_field_part(FieldPart::Description);
        let row = Element::layout(LayoutType::Flex)
            .children(vec![label, control, description])
            .with_field_row(FieldRow { required: false });
        let element = Element::layout(LayoutType::Flex).with_child(row);
        // Every element in pre-order gets bounds, so each one publishes.
        let geometry: Vec<PaintedNode> = (0..9)
            .map(|element_index| PaintedNode {
                element_index,
                bounds: Bounds {
                    x: 0.0,
                    y: element_index as f32,
                    width: 30.0,
                    height: 1.0,
                },
            })
            .collect();
        let mut events = EventTree::default();
        let mut router = EventRouter::new();
        events.sync(&element, &geometry, None, None, &mut router);
        let snapshot = events.accessibility_snapshot(&element, &geometry, &router, "test");
        let id_of = |label: &str| {
            snapshot
                .update
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .map(|(id, _)| *id)
                .unwrap_or_else(|| panic!("{label} publishes a node"))
        };
        let (name, description) = (id_of("Name"), id_of("Work address"));
        let (_, control) = snapshot
            .update
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::TextInput)
            .expect("the control publishes its node");
        assert_eq!(
            control.labelled_by(),
            &[name],
            "the control's own node is labelled by the label's text"
        );
        assert_eq!(control.described_by(), &[description]);
        assert!(!control.is_required());
        let linked = snapshot
            .update
            .nodes
            .iter()
            .filter(|(_, node)| !node.labelled_by().is_empty())
            .count();
        assert_eq!(linked, 1, "only the control's own node is linked");
    }

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

    /// What an App keeps between frames, for NAV-011's test.
    #[derive(Default)]
    struct Frames {
        runtime: crate::component::runtime::ComponentRuntime,
        events: EventTree,
        router: EventRouter,
        focus: crate::app::focus_manager::FocusManager,
    }

    impl Frames {
        /// One frame: `root` resolved, every element laid out at 120 by 5
        /// cells, registered, the focus placed as the App places it, and the
        /// tree published.
        fn frame(&mut self, root: Element) -> (Element, Snapshot) {
            let mut element = self.runtime.resolve(root).unwrap();
            crate::accessibility::style::prepare(&mut element).unwrap();
            let geometry: Vec<PaintedNode> = (0..64)
                .map(|element_index| PaintedNode {
                    element_index,
                    bounds: Bounds {
                        x: 0.0,
                        y: 0.0,
                        width: 120.0,
                        height: 5.0,
                    },
                })
                .collect();
            let (plan, _) = self
                .events
                .sync(&element, &geometry, None, None, &mut self.router);
            self.focus.apply(&mut self.router, plan);
            let snapshot =
                self.events
                    .accessibility_snapshot(&element, &geometry, &self.router, "test");
            (element, snapshot)
        }
    }

    /// The one published node with `role`, and its id.
    fn only_of_role(snapshot: &Snapshot, role: Role) -> (accesskit::NodeId, Node) {
        let found: Vec<_> = snapshot
            .update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .collect();
        assert_eq!(found.len(), 1, "one {role:?} node is published");
        (found[0].0, found[0].1.clone())
    }

    /// The published nodes that are polite live regions of their own: the
    /// split's announcements.
    fn announcements(snapshot: &Snapshot) -> Vec<(accesskit::NodeId, Option<String>)> {
        snapshot
            .update
            .nodes
            .iter()
            .filter(|(_, node)| node.live() == Some(accesskit::Live::Polite))
            .map(|(id, node)| (*id, node.label().map(str::to_owned)))
            .collect()
    }

    /// NAV-011: a split whose element asks for the auto focus gives it to
    /// its first divider, on the first frame, and its outer box takes none;
    /// after each move of the divider the published tree carries the new
    /// size of the pane before it as screen-reader-only text in a polite
    /// live region beside the splitter, a new node each move, while the
    /// splitter's node is the one a split resting at that size publishes.
    #[test]
    #[serial_test::serial(keymap)]
    fn nav_011_a_split_focuses_its_first_divider_and_announces_each_move() {
        use crate::{
            builder,
            event::types::{Event, KeyCode, KeyEvent},
            reactive::{component_scope::ComponentScope, scheduler::Scheduler},
            widgets::layout::Pane,
        };
        use std::sync::Arc;
        let split = |first: u16| {
            builder::resizable()
                .pane_with(Pane::new(Element::text("Left pane")).size(first))
                .pane(Element::text("Right pane"))
                .aria_label("Panes")
                .build()
        };
        let scope = ComponentScope::new(Arc::new(Scheduler::new()));
        let _binding = scope.enter(true);

        // Without the auto focus the split takes no focus.
        let mut app = Frames::default();
        app.frame(split(40));
        app.frame(split(40));
        assert_eq!(
            app.router.get_focus(),
            None,
            "a split asks for no focus itself"
        );
        app.runtime.clear();

        // A split resting at 41 cells, for the splitter's node after a move.
        let mut app = Frames::default();
        app.frame(split(41).auto_focus());
        let (_, resting) = app.frame(split(41).auto_focus());
        let (_, resting) = only_of_role(&resting, Role::Splitter);
        assert_eq!(resting.numeric_value(), Some(41.0));
        app.runtime.clear();

        let mut app = Frames::default();
        let root = || split(40).auto_focus();
        // The first frame, before the split knows its length: the divider
        // is already there and takes the focus.
        let (_, first) = app.frame(root());
        let (splitter, _) = only_of_role(&first, Role::Splitter);
        assert_eq!(
            first.update.focus, splitter,
            "NAV-011: the split's auto focus lands on its first divider on the first frame"
        );
        let (_, rested) = app.frame(root());
        let (splitter, before) = only_of_role(&rested, Role::Splitter);
        assert_eq!(rested.update.focus, splitter, "the divider keeps the focus");
        assert_eq!(before.numeric_value(), Some(40.0));
        assert!(
            announcements(&rested).is_empty(),
            "nothing is announced before a move"
        );

        let mut heard = Vec::new();
        for (code, size) in [
            (KeyCode::Right, "41"),
            (KeyCode::Left, "40"),
            (KeyCode::Right, "41"),
        ] {
            app.router.process_event(&Event::Key(KeyEvent::new(code)));
            let (element, moved) = app.frame(root());
            let told = announcements(&moved);
            let labels: Vec<_> = told.iter().map(|(_, label)| label.as_deref()).collect();
            assert_eq!(
                labels,
                vec![Some(size)],
                "NAV-011: after the move the tree carries one polite announcement of the size"
            );
            heard.push(told[0].0);
            let announced = element
                .children
                .iter()
                .find(|child| {
                    child
                        .key
                        .as_deref()
                        .is_some_and(|key| key.starts_with("announced:"))
                })
                .expect("the announcement is a child of the split beside its dividers");
            assert!(
                announced
                    .metadata
                    .accessibility_options
                    .as_ref()
                    .is_some_and(|options| options.screen_reader_only),
                "the announcement is screen-reader-only text, which takes no cell"
            );
            let (id, after) = only_of_role(&moved, Role::Splitter);
            assert_eq!(id, splitter, "the splitter is the same node");
            assert_eq!(moved.update.focus, splitter, "the divider keeps the focus");
            if size == "41" {
                assert_eq!(
                    after, resting,
                    "NAV-010: the splitter's node is the one a split resting at 41 publishes"
                );
            }
        }
        assert!(
            heard[0] != heard[1] && heard[1] != heard[2] && heard[0] != heard[2],
            "each move's announcement is a new node, so a repeated size is heard again"
        );
        app.runtime.clear();
        scope.close();
    }
}
