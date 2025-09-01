//! Responsive Grid Demo
//!
//! Demonstrates that grids are now responsive by default and fill the entire terminal window.

use reactive_tui::component::Element;
use reactive_tui::layout::grid::{DeclarativeGrid, GridScalar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Responsive Grid System Demo");
    println!("===============================");

    // Demo 1: Auto-grid is responsive by default
    println!("\n1. Auto-grid (responsive by default):");
    let items = vec![
        Element::text("Header"),
        Element::text("Nav"),
        Element::text("Main Content"),
        Element::text("Sidebar"),
        Element::text("Footer"),
    ];

    let auto_grid = DeclarativeGrid::auto_grid(3, 2, items);
    if let Some(css) = &auto_grid.css_class {
        println!("   CSS classes: {}", css);
        println!("   ✅ Contains w-full: {}", css.contains("w-full"));
        println!("   ✅ Contains h-full: {}", css.contains("h-full"));
    }

    // Demo 2: Manual responsive grid
    println!("\n2. Manual responsive grid:");
    let responsive_grid = DeclarativeGrid::new(4, 3)
        .column_sizes(vec![
            GridScalar::Fr(1.0),
            GridScalar::Fr(2.0),
            GridScalar::Fr(1.0),
            GridScalar::Cells(20),
        ])
        .responsive();

    if let Some(css) = &responsive_grid.css_class {
        println!("   CSS classes: {}", css);
        println!(
            "   ✅ Responsive: {}",
            css.contains("w-full") && css.contains("h-full")
        );
    }

    // Demo 3: Fixed size override (when you don't want responsive)
    println!("\n3. Fixed size override:");
    let fixed_grid = DeclarativeGrid::new(2, 2).fixed_size("80", "24");

    if let Some(css) = &fixed_grid.css_class {
        println!("   CSS classes: {}", css);
        println!("   ✅ Fixed width: {}", css.contains("w-80"));
        println!("   ✅ Fixed height: {}", css.contains("h-24"));
    }

    // Demo 4: Macro DSL is responsive by default
    println!("\n4. Macro DSL (responsive by default):");
    let macro_grid = reactive_tui::layout! {
        grid(cols: 3, rows: 3, gap: 1) {
            "header" at (0, 0) span (1, 3) class "bg-blue-600",
            "nav" at (1, 0) class "bg-gray-200",
            "main" at (1, 1) span (2, 1) class "bg-white",
            "aside" at (1, 2) class "bg-yellow-100",
            "footer" at (2, 0) span (1, 3) class "bg-gray-800",
        }
    };

    if let Some(css) = &macro_grid.css_class {
        println!("   CSS classes: {}", css);
        println!(
            "   ✅ Responsive: {}",
            css.contains("w-full") && css.contains("h-full")
        );
    }

    println!("\n🎯 Key Benefits:");
    println!("   • Grids fill entire terminal by default");
    println!("   • No need to manually set width/height");
    println!("   • Perfect for terminal applications");
    println!("   • Can override with fixed_size() when needed");
    println!("   • Works with all grid APIs (auto, manual, macro)");

    println!("\n✅ All grids are now responsive by default!");
    println!("   Terminal apps will automatically use the full window.");

    Ok(())
}
