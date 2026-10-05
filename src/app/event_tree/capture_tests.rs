use super::*;
use crate::event::types::{KeyCode, KeyEvent};
use std::sync::Mutex;

#[test]
fn component_expansion_preserves_capture_before_activation() {
    use crate::{
        component::runtime::ComponentRuntime,
        reactive::{component_scope::ComponentScope, scheduler::Scheduler},
        widgets::display::{ProgressBar, ProgressBarProps},
    };
    let scope = ComponentScope::new(Arc::new(Scheduler::new()));
    let _binding = scope.enter(true);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut element = Element::typed::<ProgressBar>(ProgressBarProps::default());
    let capture = calls.clone();
    element.metadata.capture_events.push(Arc::new(move |_| {
        capture.lock().unwrap().push("capture");
        EventResult::Handled
    }));
    let activation = calls.clone();
    element.metadata.on_click.push(Arc::new(move || {
        activation.lock().unwrap().push("activation");
    }));
    let mut runtime = ComponentRuntime::default();
    let output = runtime.resolve(element).unwrap();
    let mut tree = EventTree::default();
    let mut router = EventRouter::new();
    tree.sync(&output, &[], None, None, &mut router);
    let id = *tree
        .nodes
        .iter()
        .min_by_key(|(path, _)| path.len())
        .unwrap()
        .1;
    assert_eq!(
        router.route_event(&Event::Key(KeyEvent::new(KeyCode::Enter)), id),
        EventResult::Consumed
    );
    assert_eq!(*calls.lock().unwrap(), ["capture", "activation"]);
    runtime.clear();
    scope.close();
}

fn target(tree: &EventTree) -> NodeId {
    *tree
        .nodes
        .get(&vec![Slot::Index(0), Slot::Index(0)])
        .unwrap()
}

#[test]
fn capture_observes_consuming_child_once_and_redraw_replaces_handlers() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut tree = EventTree::default();
    let mut router = EventRouter::new();
    for label in ["first capture", "replacement capture"] {
        let mut root = Element::text("");
        let observer = calls.clone();
        root.metadata.capture_events.push(Arc::new(move |_| {
            observer.lock().unwrap().push(label);
            EventResult::Handled
        }));
        let mut child = Element::text("TRIGGER");
        let callback = calls.clone();
        child.metadata.on_click.push(Arc::new(move || {
            callback.lock().unwrap().push("child");
        }));
        root.children.push(child);
        tree.sync(&root, &[], None, None, &mut router);
        assert_eq!(
            router.route_event(&Event::Key(KeyEvent::new(KeyCode::Enter)), target(&tree)),
            EventResult::Consumed
        );
    }
    assert_eq!(
        *calls.lock().unwrap(),
        ["first capture", "child", "replacement capture", "child"]
    );
}

#[test]
fn disabled_and_inert_capture_handlers_do_not_run_and_removal_releases_capture() {
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let mut root = Element::text("ROOT");
    root.metadata.capture_events.push(Arc::new(move |_| {
        let _owned = &marker;
        panic!("suppressed handler ran");
    }));
    root.children.push(Element::text("CHILD"));
    let mut tree = EventTree::default();
    let mut router = EventRouter::new();
    for (disabled, inert) in [(true, false), (false, true)] {
        root.metadata.disabled = disabled;
        root.metadata.inert = inert;
        tree.sync(&root, &[], None, None, &mut router);
        assert_eq!(
            router.route_event(&Event::Key(KeyEvent::new(KeyCode::Enter)), target(&tree)),
            EventResult::Ignored
        );
    }
    root.metadata.inert = false;
    tree.sync(&root, &[], None, None, &mut router);
    drop(root);
    assert!(weak.upgrade().is_some());
    tree.clear(&mut router);
    assert!(weak.upgrade().is_none());
}

/// An element hidden by `display: none` after a press on it keeps no hold on
/// the pointer: the drag and the release that follow reach no handler of it
/// or its children (STY-002, review finding 6).
#[test]
fn sty_002_an_element_hidden_after_a_press_gets_no_drag_or_release() {
    use crate::{
        backend::PaintedNode,
        event::{
            hit::Bounds,
            types::{MouseButton, MouseEvent, MouseEventKind, Position},
        },
    };
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut root = Element::text("ROOT");
    let mut target = Element::text("TARGET");
    let observer = seen.clone();
    target.metadata.events.push(Arc::new(move |event| {
        if let Event::Mouse(mouse) = event {
            if matches!(
                mouse.kind,
                MouseEventKind::Down | MouseEventKind::Drag | MouseEventKind::Up
            ) {
                observer.lock().unwrap().push(mouse.kind.clone());
            }
        }
        EventResult::Handled
    }));
    root.children.push(target);
    let mouse = |kind, x, y| {
        Event::Mouse(MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left))
    };
    let node = |element_index, width| PaintedNode {
        element_index,
        bounds: Bounds::new(0.0, 0.0, width, 1.0),
    };
    let mut tree = EventTree::default();
    let mut router = EventRouter::new();
    tree.sync(
        &root,
        &[node(0, 10.0), node(1, 5.0)],
        None,
        None,
        &mut router,
    );
    router.process_event(&mouse(MouseEventKind::Down, 1, 0));
    // The next frame paints the target no more: `display: none` leaves an
    // element out of the painted nodes.
    tree.sync(&root, &[node(0, 10.0)], None, None, &mut router);
    router.process_event(&mouse(MouseEventKind::Drag, 1, 0));
    router.process_event(&mouse(MouseEventKind::Up, 1, 0));
    assert_eq!(
        *seen.lock().unwrap(),
        [MouseEventKind::Down],
        "STY-002: the element hidden after the press still received the drag or the release"
    );
}
