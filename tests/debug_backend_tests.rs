use reactive_tui::backend::Backend;
use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::event::types as rt_event;
use reactive_tui::render::reconcile::Reconciler;
use reactive_tui::render::tree::{RenderTree, element_to_render_node};

#[test]
fn test_debug_backend_creation() {
    let backend = DebugBackend::new(80, 24);
    assert_eq!(backend.size(), (80, 24));
    assert_eq!(backend.frame_count(), 0);
}

#[test]
fn test_debug_backend_screen_content() {
    let backend = DebugBackend::new(10, 3);

    // Initially should be all spaces or null chars
    let content = backend.screen_content();
    assert_eq!(content.len(), 10 * 3 + 2); // 10 chars per line + 2 newlines

    // Check what the default character actually is
    let first_char = content.chars().next().unwrap();
    println!("First char: {:?} (code: {})", first_char, first_char as u32);

    // Default cell character is '\0' (null character)
    assert!(content.chars().all(|c| c == '\0' || c == '\n'));
}

#[test]
fn test_debug_backend_render_text() {
    let mut backend = DebugBackend::new(20, 5);

    // Create a simple text element
    let element = Element::text("Hello World");

    // Render it using the full render path
    backend.render_full(&element).unwrap();

    // Check that the text appears at position (0, 0)
    assert_eq!(backend.char_at(0, 0), Some('H'));
    assert_eq!(backend.char_at(1, 0), Some('e'));
    assert_eq!(backend.char_at(2, 0), Some('l'));
    assert_eq!(backend.char_at(3, 0), Some('l'));
    assert_eq!(backend.char_at(4, 0), Some('o'));
    assert_eq!(backend.char_at(5, 0), Some(' '));
    assert_eq!(backend.char_at(6, 0), Some('W'));

    // Find the 'H' character
    assert_eq!(backend.find_char('H'), Some((0, 0)));
    assert_eq!(backend.find_char('W'), Some((6, 0)));

    // Frame count should have incremented
    assert_eq!(backend.frame_count(), 1);
}

#[test]
fn test_debug_backend_patch_application() {
    let mut backend = DebugBackend::new(20, 5);
    let mut reconciler = Reconciler::new();

    // Create initial tree
    let element1 = Element::text("First");
    let root1 = element_to_render_node(element1);
    let mut tree1 = RenderTree::new();
    tree1.set_root(root1);

    // Create updated tree
    let element2 = Element::text("Second");
    let root2 = element_to_render_node(element2);
    let mut tree2 = RenderTree::new();
    tree2.set_root(root2);

    // Generate patches
    let diff_result = reconciler.diff(&tree1, &tree2);

    // Apply patches
    backend.apply_patches(&diff_result.patches, &tree2).unwrap();
    backend.present().unwrap();

    // Check that "Second" is rendered
    assert_eq!(backend.char_at(0, 0), Some('S'));
    assert_eq!(backend.char_at(1, 0), Some('e'));
    assert_eq!(backend.char_at(2, 0), Some('c'));

    // Check patch history
    assert_eq!(backend.patch_history().len(), 1);
    assert_eq!(backend.frame_count(), 1);
}

#[test]
fn test_debug_backend_event_queue() {
    let mut backend = DebugBackend::new(20, 5);

    // Initially no events
    assert!(backend.poll_event(None).unwrap().is_none());

    // Add some events
    let key_event = rt_event::Event::Key(rt_event::KeyEvent::new(rt_event::KeyCode::Char('a')));
    backend.push_event(key_event);

    let mouse_event = rt_event::Event::Mouse(
        rt_event::MouseEvent::new(
            rt_event::MouseEventKind::Down,
            rt_event::Position::cell(10, 5),
        )
        .with_button(rt_event::MouseButton::Left),
    );
    backend.push_event(mouse_event);

    // Poll events in order
    let first_event = backend.poll_event(None).unwrap();
    assert!(first_event.is_some());
    assert!(first_event.unwrap().is_key());

    let second_event = backend.poll_event(None).unwrap();
    assert!(second_event.is_some());
    assert!(second_event.unwrap().is_mouse());

    assert!(backend.poll_event(None).unwrap().is_none());
}

#[test]
fn test_debug_backend_clear() {
    let mut backend = DebugBackend::new(10, 3);

    // Render some text
    let element = Element::text("Test");
    backend.render_full(&element).unwrap();

    // Verify text is there
    assert_eq!(backend.char_at(0, 0), Some('T'));

    // Clear the screen
    backend.clear().unwrap();

    // Verify it's cleared (should be spaces)
    assert_eq!(backend.char_at(0, 0), Some(' '));
    assert_eq!(backend.char_at(1, 0), Some(' '));
}

#[test]
fn test_debug_backend_multiline_text() {
    let mut backend = DebugBackend::new(20, 5);

    // Create multiline text
    let element = Element::text("Line 1\nLine 2\nLine 3");
    backend.render_full(&element).unwrap();

    // Check first line
    assert_eq!(backend.char_at(0, 0), Some('L'));
    assert_eq!(backend.char_at(1, 0), Some('i'));
    assert_eq!(backend.char_at(2, 0), Some('n'));
    assert_eq!(backend.char_at(3, 0), Some('e'));
    assert_eq!(backend.char_at(5, 0), Some('1'));

    // Check second line
    assert_eq!(backend.char_at(0, 1), Some('L'));
    assert_eq!(backend.char_at(1, 1), Some('i'));
    assert_eq!(backend.char_at(2, 1), Some('n'));
    assert_eq!(backend.char_at(3, 1), Some('e'));
    assert_eq!(backend.char_at(5, 1), Some('2'));

    // Check third line
    assert_eq!(backend.char_at(0, 2), Some('L'));
    assert_eq!(backend.char_at(5, 2), Some('3'));
}

#[test]
fn test_debug_backend_patch_history() {
    let mut backend = DebugBackend::new(20, 5);

    let mut tree = RenderTree::new();
    let element = Element::text("Test");
    let root = element_to_render_node(element);
    tree.set_root(root);

    // Apply empty patches first
    backend.apply_patches(&[], &tree).unwrap();
    assert_eq!(backend.patch_history().len(), 1);
    assert_eq!(backend.patch_history()[0].len(), 0);

    // Apply another set of patches
    backend.apply_patches(&[], &tree).unwrap();
    assert_eq!(backend.patch_history().len(), 2);
}
