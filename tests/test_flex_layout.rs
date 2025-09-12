use reactive_tui::core::geometry::Point;
use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use std::borrow::Cow;

#[test]
fn test_flex_row_layout() {
    eprintln!("\n=== Testing flex row layout ===");
    
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-row w-40 h-10 gap-2"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-red-500"),
                text: Some(Cow::Borrowed("Item1")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-blue-500"),
                text: Some(Cow::Borrowed("Item2")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-green-500"),
                text: Some(Cow::Borrowed("Item3")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(50, 12);
    let opts = PaintOptions { debug_overlay: false };
    
    layout_and_paint_with(&root, &mut surface, 40, &opts).unwrap();
    
    println!("Flex row layout (should show items side by side):");
    for y in 0..5 {
        for x in 0..30 {
            let cell = surface.get_at(Point::new(x, y));
            print!("{}", cell.ch);
        }
        println!();
    }
}

#[test]
fn test_flex_column_layout() {
    eprintln!("\n=== Testing flex column layout ===");
    
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-col w-20 h-15 gap-1"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-red-500"),
                text: Some(Cow::Borrowed("Row 1")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-blue-500"),
                text: Some(Cow::Borrowed("Row 2")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-green-500"),
                text: Some(Cow::Borrowed("Row 3")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(25, 15);
    let opts = PaintOptions { debug_overlay: false };
    
    layout_and_paint_with(&root, &mut surface, 20, &opts).unwrap();
    
    println!("Flex column layout (should show items stacked):");
    for y in 0..10 {
        for x in 0..22 {
            let cell = surface.get_at(Point::new(x, y));
            print!("{}", cell.ch);
        }
        println!();
    }
}

#[test]
fn test_justify_content() {
    eprintln!("\n=== Testing justify-content ===");
    
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-row justify-center w-40 h-5"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("w-6 h-3"),
                text: Some(Cow::Borrowed("CENTER")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(45, 7);
    let opts = PaintOptions { debug_overlay: false };
    
    layout_and_paint_with(&root, &mut surface, 40, &opts).unwrap();
    
    println!("Justify-center (text should be centered horizontally):");
    for y in 0..4 {
        print!("'");
        for x in 0..40 {
            let cell = surface.get_at(Point::new(x, y));
            print!("{}", cell.ch);
        }
        println!("'");
    }
}

#[test]
fn test_grid_layout() {
    eprintln!("\n=== Testing grid layout ===");
    
    let root = NodeSpec {
        class: Cow::Borrowed("grid grid-cols-3 gap-1 w-30 h-10"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("A")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("B")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("C")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("D")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("E")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("F")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(35, 12);
    let opts = PaintOptions { debug_overlay: false };
    
    layout_and_paint_with(&root, &mut surface, 30, &opts).unwrap();
    
    println!("Grid 3-column layout:");
    for y in 0..6 {
        for x in 0..32 {
            let cell = surface.get_at(Point::new(x, y));
            print!("{}", cell.ch);
        }
        println!();
    }
}