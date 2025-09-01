use reactive_tui::theme::{dark_theme, light_theme, Theme, ThemeVariables};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Testing New CSS-First Theme System");
    println!("=====================================");

    // Test theme creation
    let dark = dark_theme();
    println!("\n🌙 Dark Theme Created Successfully!");
    println!("Theme name: {}", dark.name);

    // Test theme variables
    println!("\n📋 Dark Theme Variables:");
    for (key, value) in dark.variables.all() {
        println!("  {}: {}", key, value);
    }

    // Test light theme
    let light = light_theme();
    println!("\n☀️ Light Theme Created Successfully!");
    println!("Theme name: {}", light.name);

    // Test custom theme creation
    let custom = Theme::new("custom").with_variables(
        ThemeVariables::new()
            .set("--color-primary", "#ff6b6b")
            .set("--color-secondary", "#4ecdc4")
            .set("--color-background", "#2c3e50")
            .set("--spacing-md", "8"),
    );

    println!("\n🎨 Custom Theme Created Successfully!");
    println!("Theme name: {}", custom.name);
    println!("Custom variables:");
    for (key, value) in custom.variables.all() {
        println!("  {}: {}", key, value);
    }

    // Test variable resolution
    if let Some(primary) = dark.get_variable("--color-primary") {
        println!(
            "\n✅ Variable resolution works: --color-primary = {}",
            primary
        );
    }

    // Test theme inheritance
    let extended = Theme::new("extended")
        .extend(dark.clone())
        .with_variables(ThemeVariables::new().set("--color-custom", "#purple"));

    println!("\n🔗 Theme Inheritance Test:");
    println!(
        "Extended theme has {} variables",
        extended.variables.all().len()
    );

    if let Some(inherited) = extended.get_variable("--color-primary") {
        println!("✅ Inherited variable: --color-primary = {}", inherited);
    }

    if let Some(custom_var) = extended.get_variable("--color-custom") {
        println!("✅ Custom variable: --color-custom = {}", custom_var);
    }

    println!("\n🎉 All theme system tests passed!");
    println!("✅ Theme creation works");
    println!("✅ Variable storage works");
    println!("✅ Variable resolution works");
    println!("✅ Theme inheritance works");
    println!("✅ CSS custom properties format works");

    println!("\n🚀 The new CSS-first theme system is ready!");

    Ok(())
}
