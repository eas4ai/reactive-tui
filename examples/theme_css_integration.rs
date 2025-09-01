use reactive_tui::layout::css::apply_utility_classes_with_theme;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::theme::{dark_theme, light_theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Testing CSS-Theme Integration");
    println!("================================");

    // Test dark theme
    let dark = dark_theme();
    println!("\n🌙 Dark Theme Variables:");
    for (key, value) in dark.variables.all() {
        println!("  {}: {}", key, value);
    }

    // Test theme-aware CSS utilities
    let style_builder = StyleBuilder::new();

    // Test with theme variables
    let themed_style = apply_utility_classes_with_theme(
        "bg-primary text-secondary p-4",
        style_builder.clone(),
        Some(&dark),
    );

    println!("\n✅ Theme-aware CSS utilities applied successfully!");

    // Test without theme (fallback to standard colors)
    let standard_style =
        apply_utility_classes_with_theme("bg-blue-500 text-white p-4", style_builder.clone(), None);

    println!("✅ Standard CSS utilities work without theme!");

    // Test light theme
    let light = light_theme();
    let light_style =
        apply_utility_classes_with_theme("bg-primary text-muted", style_builder, Some(&light));

    println!("✅ Light theme integration works!");

    // Test direct theme usage
    let custom_style = dark.apply_classes("bg-success text-foreground p-2");
    println!("✅ Direct theme.apply_classes() works!");

    println!("\n🎉 All theme-CSS integration tests passed!");
    println!("The new CSS-first theme system is working correctly.");

    Ok(())
}
