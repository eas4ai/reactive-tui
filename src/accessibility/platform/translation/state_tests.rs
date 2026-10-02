use super::node::NodeWrapper;
use accesskit::{Action, AriaCurrent, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use accesskit_consumer::Tree;
use atspi_common::{Interface, Role as AtspiRole, State, StateSet};

fn translated_state(node: Node) -> StateSet {
    let id = NodeId(1);
    let tree = Tree::new(
        TreeUpdate {
            nodes: vec![(id, node)],
            tree: Some(TreeInfo::new(id)),
            tree_id: TreeId::ROOT,
            focus: id,
        },
        true,
    );
    NodeWrapper(&tree.state().root()).state(true)
}

fn translated_role(node: Node) -> AtspiRole {
    let id = NodeId(1);
    let tree = Tree::new(
        TreeUpdate {
            nodes: vec![(id, node)],
            tree: Some(TreeInfo::new(id)),
            tree_id: TreeId::ROOT,
            focus: id,
        },
        true,
    );
    NodeWrapper(&tree.state().root()).role()
}

#[test]
fn current_item_tokens_are_active_without_becoming_focused_or_selected() {
    for (current, token, active) in [
        (None, None, false),
        (Some(AriaCurrent::False), Some("false"), false),
        (Some(AriaCurrent::True), Some("true"), true),
        (Some(AriaCurrent::Page), Some("page"), true),
        (Some(AriaCurrent::Step), Some("step"), true),
        (Some(AriaCurrent::Location), Some("location"), true),
        (Some(AriaCurrent::Date), Some("date"), true),
        (Some(AriaCurrent::Time), Some("time"), true),
    ] {
        let mut root = Node::new(Role::Window);
        root.set_children([NodeId(2)]);
        let mut link = Node::new(Role::Link);
        if let Some(current) = current {
            link.set_aria_current(current);
        }
        let tree = Tree::new(
            TreeUpdate {
                nodes: vec![(NodeId(1), root), (NodeId(2), link)],
                tree: Some(TreeInfo::new(NodeId(1))),
                tree_id: TreeId::ROOT,
                focus: NodeId(1),
            },
            true,
        );
        let link = tree.state().root().children().next().unwrap();
        let translated = NodeWrapper(&link);
        for focused in [false, true] {
            let states = translated.state(focused);
            assert_eq!(states.contains(State::Active), active, "{current:?}");
            assert!(!states.contains(State::Focused));
            assert!(!states.contains(State::Selected));
        }
        assert_eq!(
            translated.attributes().get("current").map(String::as_str),
            token
        );
        assert!(NodeWrapper(&tree.state().root())
            .state(true)
            .contains(State::Active));
        assert!(!NodeWrapper(&tree.state().root())
            .state(false)
            .contains(State::Active));
    }
}

#[test]
fn disclosure_states_distinguish_plain_collapsed_and_expanded_buttons() {
    let plain = translated_state(Node::new(Role::Button));
    assert!(!plain.contains(State::Expandable | State::Expanded));
    for expanded in [false, true] {
        let mut node = Node::new(Role::Button);
        node.set_expanded(expanded);
        let states = translated_state(node);
        assert!(states.contains(State::Expandable));
        assert_eq!(states.contains(State::Expanded), expanded);
    }
}

#[test]
fn disabled_buttons_are_neither_enabled_nor_sensitive() {
    for disabled in [false, true] {
        let mut node = Node::new(Role::Button);
        node.add_action(Action::Focus);
        node.add_action(Action::Click);
        if disabled {
            node.set_disabled();
        }
        let states = translated_state(node);
        assert_eq!(states.contains(State::Enabled), !disabled);
        assert_eq!(states.contains(State::Sensitive), !disabled);
    }
}

#[test]
fn read_only_text_remains_enabled_without_becoming_editable() {
    let mut node = Node::new(Role::TextInput);
    node.set_read_only();
    let states = translated_state(node);
    assert!(states.contains(State::ReadOnly | State::Enabled | State::Sensitive));
    assert!(!states.contains(State::Editable));
}

#[test]
fn focused_node_reports_focused_while_siblings_do_not() {
    let mut root = Node::new(Role::Window);
    root.set_children([NodeId(2)]);
    let button = Node::new(Role::Button);
    let tree = Tree::new(
        TreeUpdate {
            nodes: vec![(NodeId(1), root), (NodeId(2), button)],
            tree: Some(TreeInfo::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: NodeId(2),
        },
        true,
    );
    let root_ref = tree.state().root();
    let child = root_ref.children().next().unwrap();
    assert!(NodeWrapper(&child).state(true).contains(State::Focused));
    assert!(!NodeWrapper(&root_ref).state(true).contains(State::Focused));
}

#[test]
fn selection_states_follow_is_selected() {
    let mut selected = Node::new(Role::ListBoxOption);
    selected.set_selected(true);
    let states = translated_state(selected);
    assert!(states.contains(State::Selectable | State::Selected));

    let mut unselected = Node::new(Role::ListBoxOption);
    unselected.set_selected(false);
    let states = translated_state(unselected);
    assert!(states.contains(State::Selectable));
    assert!(!states.contains(State::Selected));

    let plain = translated_state(Node::new(Role::ListBoxOption));
    assert!(!plain.contains(State::Selectable | State::Selected));
}

#[test]
fn named_forms_are_landmarks_unnamed_forms_are_panels() {
    let mut named = Node::new(Role::Form);
    named.set_label("Search");
    assert_eq!(translated_role(named), AtspiRole::Landmark);

    assert_eq!(translated_role(Node::new(Role::Form)), AtspiRole::Panel);
}

#[test]
fn focus_without_pixel_bounds_exposes_component_but_not_activation() {
    let mut root = Node::new(Role::Window);
    root.set_children([NodeId(2)]);
    let mut entry = Node::new(Role::TextInput);
    entry.add_action(Action::Focus);
    let tree = Tree::new(
        TreeUpdate {
            nodes: vec![(NodeId(1), root), (NodeId(2), entry)],
            tree: Some(TreeInfo::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: NodeId(2),
        },
        true,
    );
    let child = tree.state().root().children().next().unwrap();
    assert!(child.raw_bounds().is_none());
    let interfaces = NodeWrapper(&child).interfaces();
    assert!(interfaces.contains(Interface::Component));
    assert!(!interfaces.contains(Interface::Action));
}
