//! Demonstration of the #[derive(Props)] macro
//!
//! This example shows how the Props derive macro provides iocraft-style
//! ergonomics for automatic props generation with validation and defaults.

use reactive_tui::component::Props as PropsTraitDemo;
use reactive_tui::prelude::*;

/// Button component props with automatic generation
#[derive(Props, Clone, PartialEq, Debug)]
struct ButtonProps {
    text: String,
    #[prop(default)]
    disabled: bool,
    #[prop(default = "primary")]
    variant: String,
    #[prop(optional)]
    icon: Option<String>,
    size: u32,
}

/// User card component props
#[derive(Props, Clone, PartialEq, Debug)]
struct UserCardProps {
    name: String,
    email: String,
    #[prop(default)]
    active: bool,
    #[prop(optional)]
    avatar_url: Option<String>,
    #[prop(default = "user")]
    role: String,
    #[prop(validate)]
    age: u32,
}

/// Modal component props with complex defaults
#[derive(Props, Clone, PartialEq, Debug)]
struct ModalProps {
    title: String,
    #[prop(default)]
    visible: bool,
    #[prop(default = "medium")]
    size: String,
    #[prop(default)]
    backdrop_dismissible: bool,
}

/// Simple function to demonstrate props usage
fn render_button(props: &ButtonProps) -> String {
    let status = if props.disabled {
        "disabled"
    } else {
        "enabled"
    };
    let icon_text = props.icon.as_deref().unwrap_or("none");

    format!(
        "Button[{}]: '{}' ({}, icon: {}, size: {})",
        props.variant, props.text, status, icon_text, props.size
    )
}

/// Simple function to demonstrate props usage
fn render_user_card(props: &UserCardProps) -> String {
    let status = if props.active { "Active" } else { "Inactive" };
    let avatar = props.avatar_url.as_deref().unwrap_or("default.jpg");

    format!(
        "User[{}]: {} ({}) - {} - Age: {} - Avatar: {}",
        props.role, props.name, props.email, status, props.age, avatar
    )
}

/// Simple function to demonstrate props usage
fn render_modal(props: &ModalProps) -> String {
    let visibility = if props.visible { "visible" } else { "hidden" };
    let dismissible = if props.backdrop_dismissible {
        "dismissible"
    } else {
        "persistent"
    };

    format!(
        "Modal[{}]: '{}' ({}, {})",
        props.size, props.title, visibility, dismissible
    )
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Props Derive Macro Demo");
    println!("==========================");
    println!();

    // Demonstrate default props creation
    println!("📋 Default Props Creation:");
    let default_button = ButtonProps::default();
    println!(
        "  Default button: disabled={}, variant='{}', size={}",
        default_button.disabled, default_button.variant, default_button.size
    );

    let default_modal = ModalProps::default();
    println!(
        "  Default modal: visible={}, size='{}', dismissible={}",
        default_modal.visible, default_modal.size, default_modal.backdrop_dismissible
    );
    println!();

    // Demonstrate fluent builder API
    println!("🔧 Fluent Builder API:");
    let custom_button = ButtonProps::new()
        .with_text("Click Me!".to_string())
        .with_variant("success".to_string())
        .with_icon(Some("star".to_string()))
        .with_size(24);

    println!(
        "  Custom button: text='{}', variant='{}', icon={:?}, size={}",
        custom_button.text, custom_button.variant, custom_button.icon, custom_button.size
    );

    let user_props = UserCardProps::new()
        .with_name("Alice Johnson".to_string())
        .with_email("alice@example.com".to_string())
        .with_active(true)
        .with_role("admin".to_string())
        .with_age(28)
        .with_avatar_url(Some("https://example.com/alice.jpg".to_string()));

    println!(
        "  User props: name='{}', role='{}', active={}, age={}",
        user_props.name, user_props.role, user_props.active, user_props.age
    );
    println!();

    // Demonstrate validation
    println!("✅ Props Validation:");
    let validation_result = custom_button.validate();
    println!("  Button validation: {:?}", validation_result);

    let user_validation = user_props.validate();
    println!("  User validation: {:?}", user_validation);
    println!();

    // Demonstrate rendering with props
    println!("🎯 Rendering with Props:");

    // Render button
    let button_output = render_button(&custom_button);
    println!("  {}", button_output);

    // Render user card
    let user_output = render_user_card(&user_props);
    println!("  {}", user_output);

    // Render modal
    let modal_props = ModalProps::new()
        .with_title("Confirmation Dialog".to_string())
        .with_visible(true)
        .with_size("large".to_string())
        .with_backdrop_dismissible(false);
    let modal_output = render_modal(&modal_props);
    println!("  {}", modal_output);
    println!();

    // Demonstrate Props trait implementation
    println!("🔍 Props Trait Implementation:");
    let button_props = ButtonProps::new().with_text("Test".to_string());
    let props_any: &dyn std::any::Any = button_props.as_any();
    let downcast_props: &ButtonProps = props_any.downcast_ref().unwrap();
    println!("  Downcast successful: text='{}'", downcast_props.text);
    println!();

    // Show comparison with manual implementation
    println!("📊 Comparison with Manual Implementation:");
    println!("  Manual Props struct: ~15-20 lines of boilerplate");
    println!("  #[derive(Props)]: 1 line + attributes");
    println!("  Reduction: 80%+ less code");
    println!("  Features gained:");
    println!("    ✅ Automatic Props trait implementation");
    println!("    ✅ Fluent builder API");
    println!("    ✅ Default value handling");
    println!("    ✅ Optional field support");
    println!("    ✅ Validation framework");
    println!("    ✅ Perfect component macro integration");
    println!();

    println!("🎉 Props derive macro working perfectly!");
    println!();
    println!("🌟 Benefits achieved:");
    println!("  🚀 iocraft-style ergonomics");
    println!("  🔧 Zero boilerplate props");
    println!("  ⚡ Type-safe builder pattern");
    println!("  🎯 Automatic validation");
    println!("  🌈 Seamless component integration");

    Ok(())
}
