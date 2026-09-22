use reactive_tui::layout::manager::LayoutManager;
use reactive_tui::prelude::Element;

#[test]
fn test_absolute_positioning_debug() {
    let mut manager = LayoutManager::new(80, 24);

    // Create a relative container
    let mut container = Element::layout(reactive_tui::component::LayoutType::Flex);
    container.key = Some("container".to_string());
    container.class = Some("w-full h-full relative".to_string());

    // Create an absolutely positioned child at (10, 5) with red text
    let mut child1 = Element::text("A");
    child1.key = Some("child1".to_string());
    child1.class = Some("absolute left-10 top-5 text-red-500".to_string());

    // Create another absolutely positioned child at (20, 10) with green text on blue background
    let mut child2 = Element::text("B");
    child2.key = Some("child2".to_string());
    child2.class = Some("absolute left-20 top-10 text-green-500 bg-blue-600 font-bold".to_string());

    // Insert nodes
    use reactive_tui::layout::manager::LayoutKey;
    manager.insert_node(None, 0, container).unwrap();
    let container_key = LayoutKey::from("container");
    manager
        .insert_node(Some(container_key.clone()), 0, child1)
        .unwrap();
    manager.insert_node(Some(container_key), 1, child2).unwrap();

    // Compute layout
    manager.compute_dirty_layouts().unwrap();

    // Generate paint ops to see final positions
    let paint_ops = manager.generate_paint_ops().unwrap();

    println!("\n=== Paint Operations ===");
    for op in paint_ops {
        println!("{:?}", op);
    }
}
