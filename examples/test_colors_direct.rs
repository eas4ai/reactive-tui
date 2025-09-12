use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use reactive_tui::core::surface::Rgba;
use std::borrow::Cow;

fn main() -> reactive_tui::Result<()> {
    println!("Testing CSS colors with direct paint_tree...");
    
    let mut surface = Surface::new(60, 10);
    
    // Test red text with explicit sizing
    let spec = NodeSpec {
        class: Cow::Borrowed("text-red-500 w-20 h-2"),
        text: Some(Cow::Borrowed("RED TEXT")),
        children: vec![],
    };
    
    println!("Testing red text with class: text-red-500 w-20 h-2");

    // Test the CSS parsing directly
    use reactive_tui::layout::style::StyleBuilder;
    use reactive_tui::layout::css::apply_utility_classes;
    use reactive_tui::ui::paint::extract_paint_style;

    let mut sb = StyleBuilder::new();
    sb = apply_utility_classes("text-red-500 w-20 h-2", sb);
    let paint_style = extract_paint_style(&mut sb);
    if let Some(style) = &paint_style {
        println!("Extracted paint style: fg={:?}, bg={:?}", style.fg, style.bg);
    } else {
        println!("No paint style extracted");
    }

    // Enable debug output for paint_tree
    std::env::set_var("PAINT_TREE_DEBUG", "1");
    layout_and_paint_with(&spec, &mut surface, 60, &PaintOptions::default())?;
    
    // Check if the cell has color information
    println!("Checking first few cells for text:");
    for y in 0..3 {
        for x in 0..10 {
            let cell = surface.get(x, y);
            if cell.ch != ' ' && cell.ch != '\0' {
                println!("Cell at ({},{}): ch='{}', fg={:?}, bg={:?}", x, y, cell.ch, cell.fg, cell.bg);
            }
        }
    }
    
    // Clear surface and test background color
    surface.clear(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    let spec = NodeSpec {
        class: Cow::Borrowed("bg-blue-500 text-white"),
        text: Some(Cow::Borrowed("BLUE BG")),
        children: vec![],
    };
    
    println!("Testing blue background with class: bg-blue-500 text-white");
    layout_and_paint_with(&spec, &mut surface, 60, &PaintOptions::default())?;
    
    // Check if the cell has color information
    let cell = surface.get(0, 0);
    println!("Cell at (0,0): ch='{}', fg={:?}, bg={:?}", cell.ch, cell.fg, cell.bg);
    
    // Test multiple colors
    surface.clear(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    let spec = NodeSpec {
        class: Cow::Borrowed("flex flex-col"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("text-red-500"),
                text: Some(Cow::Borrowed("Red")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("text-green-500"),
                text: Some(Cow::Borrowed("Green")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("bg-yellow-500 text-black"),
                text: Some(Cow::Borrowed("Yellow BG")),
                children: vec![],
            },
        ],
    };
    
    println!("Testing multiple colors in flex layout");
    layout_and_paint_with(&spec, &mut surface, 60, &PaintOptions::default())?;
    
    // Check multiple cells
    for y in 0..3 {
        let cell = surface.get(0, y);
        println!("Cell at (0,{}): ch='{}', fg={:?}, bg={:?}", y, cell.ch, cell.fg, cell.bg);
    }
    
    println!("Color test complete!");
    Ok(())
}
